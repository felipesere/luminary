use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use clutter::ResourceState;

use crate::{Address, Cloud, Creatable, Module, ModuleDefinition, RealState, Resource, Segment};

#[derive(Debug)]
pub struct Provider<C: Cloud> {
    api: C::ProviderApi,
    tracked_resources: RwLock<HashMap<Address, Arc<dyn Creatable<C>>>>,
    current_address: RwLock<Address>,
}

impl<C: Cloud> Provider<C> {
    pub fn new(api: C::ProviderApi) -> Self {
        Self {
            api,
            tracked_resources: Default::default(),
            current_address: RwLock::new(Address::root()),
        }
    }

    pub fn resource<F, O>(&mut self, name: &'static str, builder: F) -> Arc<O>
    where
        F: FnOnce(&mut C::ProviderApi) -> O,
        O: Resource<C> + 'static,
    {
        let object = builder(&mut self.api);
        let address = Segment {
            name: name.to_string(),
            kind: object.kind().to_string(),
        };

        let wrapped = Arc::new(object);

        self.track(address, Arc::clone(&wrapped) as Arc<dyn Creatable<C>>);

        wrapped
    }

    pub fn track(&mut self, relative_address: Segment, resource: Arc<dyn Creatable<C>>) {
        let real = self.current_address.read().unwrap().child(relative_address);
        println!("Tracking {:?}", real);

        self.tracked_resources
            .write()
            .unwrap()
            .insert(real, resource);
    }

    pub fn module<MD>(&mut self, module_name: &'static str, definition: MD) -> Module<MD, C>
    where
        MD: ModuleDefinition<C>,
    {
        let current_address = self.current_address.read().unwrap().clone();
        let module_address = current_address.child(Segment {
            kind: "module".into(),
            name: module_name.into(),
        });
        *self.current_address.write().unwrap() = module_address;

        let outputs = definition.define(self);

        *self.current_address.write().unwrap() = current_address;

        Module {
            name: module_name,
            outputs,
        }
    }

    pub async fn create(&self) -> Result<RealState, String> {
        let mut state = RealState::new();
        for (address, resource) in self.tracked_resources.write().unwrap().iter_mut() {
            let fields = resource.create(&self.api).await?;
            let resource_state = ResourceState::new(address, fields);

            state.add(resource_state);
        }

        Ok(state)
    }
}

#[derive(Debug)]
struct Anchor {
    idx: smol_graph::NodeIndex,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Dependenable, Dependent, Value};
    use async_trait::async_trait;

    #[derive(Debug)]
    struct FakeResource(i32);

    impl AsRef<Dependent> for FakeResource {
        fn as_ref(&self) -> &Dependent {
            &Dependent
        }
    }

    impl FakeResource {
        fn output(&self) -> Value<i32> {
            let other = self.0.clone();
            Value::Reference(Box::new(move || other))
        }
    }

    #[async_trait]
    impl Resource<FakeCloud> for FakeResource {}

    impl Dependenable for FakeResource {}

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

    impl Dependenable for OtherResource {}

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

            let fast = provider
                .resource("the_fast_one", |_api| OtherResource {
                    name: "other_one",
                    other: slow.output(),
                })
                .depends_on([&slow]);

            provider.create().await;

            assert!(false);
        })
    }
}
