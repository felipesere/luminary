use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use clutter::ResourceState;

use crate::{Address, Cloud, Creatable, Module, ModuleDefinition, RealState, Resource, Segment};

use petgraph::{graph::NodeIndex, Graph};

#[derive(Debug)]
pub struct Provider<C: Cloud> {
    api: C::ProviderApi,
    tracked_resources: RwLock<HashMap<Address, Arc<dyn Creatable<C>>>>,
    current_address: RwLock<Address>,
    // TODO: Could this just be a segment?
    pub dependency_graph: Graph<Address, DependencyKind>,
    pub root_idx: RwLock<NodeIndex>,
}

#[derive(Debug)]
pub enum DependencyKind {
    Resource,
    Module,
}

impl std::fmt::Display for DependencyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyKind::Resource => write!(f, "resource"),
            DependencyKind::Module => write!(f, "module"),
        }
    }
}

pub type Dependent = NodeIndex;

pub struct Meta<R> {
    inner: Arc<R>,
    anchor: Anchor,
}

impl<R> Meta<R> {
    fn new(object: Arc<R>, address: NodeIndex) -> Self {
        Meta {
            inner: object,
            anchor: Anchor::new(address),
        }
    }
    pub fn depends_on<const N: usize>(
        &self,
        graph: &mut Graph<Address, DependencyKind>,
        other: [&dyn AsRef<Dependent>; N],
    ) {
        // TODO: how do I get this?!
        for dependant in other {
            let node_idx = dependant.as_ref();
            graph.add_edge(self.anchor.idx, *node_idx, DependencyKind::Resource);
        }
    }
}

impl<R> std::ops::Deref for Meta<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<R> AsRef<Dependent> for Meta<R> {
    fn as_ref(&self) -> &Dependent {
        &self.anchor.idx
    }
}

impl<R> Clone for Meta<R> {
    fn clone(&self) -> Self {
        Meta {
            inner: Arc::clone(&self.inner),
            anchor: self.anchor.clone(),
        }
    }
}

impl<C: Cloud> Provider<C> {
    pub fn new(api: C::ProviderApi) -> Self {
        let root = Address::root();
        let mut deps = Graph::new();

        let root_idx = deps.add_node(root.clone());
        Self {
            api,
            tracked_resources: Default::default(),
            current_address: RwLock::new(root),
            dependency_graph: deps,
            root_idx: RwLock::new(root_idx),
        }
    }

    pub fn resource<F, O>(&mut self, name: &'static str, builder: F) -> Meta<O>
    where
        F: FnOnce(&mut C::ProviderApi) -> O,
        O: Resource<C> + 'static,
    {
        let object = builder(&mut self.api);
        let object_segment = Segment {
            name: name.to_string(),
            kind: object.kind().to_string(),
        };

        let wrapped = Arc::new(object);

        let real = self.current_address.read().unwrap().child(object_segment);

        self.track(real.clone(), Arc::clone(&wrapped) as Arc<dyn Creatable<C>>);

        let node_idx = self.dependency_graph.add_node(real);

        let root_idx = self.root_idx.read().unwrap().clone();
        self.dependency_graph
            .add_edge(root_idx, node_idx, DependencyKind::Resource);

        let anchor = Anchor::new(node_idx);

        Meta {
            inner: wrapped,
            anchor,
        }
    }

    // TODO: This will likely go away at some point
    pub fn track(&mut self, real: Address, resource: Arc<dyn Creatable<C>>) {
        println!("Tracking {:?}", real);

        self.tracked_resources
            .write()
            .unwrap()
            .insert(real, resource);
    }

    pub fn module<MD>(&mut self, module_name: &'static str, definition: MD) -> Meta<Module<MD, C>>
    where
        MD: ModuleDefinition<C>,
    {
        let current_address = self.current_address.read().unwrap().clone();
        let current_idx = self.root_idx.read().unwrap().clone();

        let module_segment = Segment {
            kind: "module".into(),
            name: module_name.into(),
        };
        let module_address = current_address.child(module_segment.clone());
        *self.current_address.write().unwrap() = module_address.clone();

        // TODO: Very likely I will have to update `self` to use the thix idx as its root
        // so that children of this module are attached correctly?
        let idx = self.dependency_graph.add_node(module_address.clone());
        self.dependency_graph
            .add_edge(current_idx, idx, DependencyKind::Module);

        *self.root_idx.write().unwrap() = idx;

        let outputs = definition.define(self);

        *self.current_address.write().unwrap() = current_address;
        *self.root_idx.write().unwrap() = current_idx;

        Meta {
            inner: Arc::new(Module {
                name: module_name,
                outputs,
            }),
            anchor: Anchor::new(idx),
        }
    }

    pub async fn create(&self) -> Result<RealState, String> {
        let mut state = RealState::new();
        for (object_segment, resource) in self.tracked_resources.write().unwrap().iter_mut() {
            let fields = resource.create(&self.api).await?;
            let resource_state = ResourceState::new(object_segment, fields);

            state.add(resource_state);
        }

        Ok(state)
    }
}

#[derive(Clone, Debug)]
struct Anchor {
    idx: NodeIndex,
}

// TODO not really sure how this will work
impl Anchor {
    fn new(idx: NodeIndex) -> Self {
        Anchor { idx }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Value;
    use async_trait::async_trait;

    #[derive(Debug)]
    struct FakeResource(i32);

    impl FakeResource {
        fn output(&self) -> Value<i32> {
            let other = self.0.clone();
            Value::Reference(Box::new(move || other))
        }
    }

    #[async_trait]
    impl Resource<FakeCloud> for FakeResource {}

    #[async_trait]
    impl Creatable<FakeCloud> for FakeResource {
        fn kind(&self) -> &'static str {
            "fake_resource"
        }

        async fn create(&self, provider: &FakeApi) -> Result<clutter::Fields, String> {
            use async_io::Timer;
            use std::time::Duration;

            Timer::after(Duration::from_secs(5)).await;
            println!("Creating resource {}", self.0);
            Ok(clutter::Fields::empty())
        }
    }

    #[derive(Debug)]
    struct OtherResource {
        name: &'static str,
        other: Value<i32>,
    }

    #[async_trait]
    impl Resource<FakeCloud> for OtherResource {}

    #[async_trait]
    impl Creatable<FakeCloud> for OtherResource {
        fn kind(&self) -> &'static str {
            "other_resource"
        }

        async fn create(&self, provider: &FakeApi) -> Result<clutter::Fields, String> {
            // TODO: consider a sleep here...
            println!("Creating resource {} with {}", self.name, self.other.get());
            Ok(clutter::Fields::empty())
        }
    }

    struct FakeCloud;

    struct FakeApi;

    impl crate::Cloud for FakeCloud {
        type ProviderApi = FakeApi;
    }

    #[test]
    fn broad_idea_of_interdependencies() {
        smol::block_on(async {
            let mut provider: Provider<FakeCloud> = Provider::new(FakeApi);

            let slow = provider.resource("the_slow_one", |_api| FakeResource(23));

            let _fast = provider
                .resource("the_fast_one", |_api| OtherResource {
                    name: "other_one",
                    other: slow.output(),
                })
                .depends_on(&mut provider.dependency_graph, [&slow]);

            provider
                .create()
                .await
                .expect("should have been able to create resources from the provider");

            assert!(false);
        })
    }
}
