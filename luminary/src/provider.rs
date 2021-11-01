use std::sync::Arc;

use async_trait::async_trait;
use clutter::ResourceState;
use depgraph::{Address, DependencyTracking};
use tracing::{info, instrument};


use crate::{Cloud, Creatable, Module, ModuleDefinition, RealState, Resource};

#[derive(Debug)]
pub struct Provider<C: Cloud> {
    api: C::ProviderApi,
    dependencies: DependencyTracking<Arc<dyn Creatable<C>>, DependencyKind>,
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

pub struct Meta<R> {
    inner: Arc<R>,
    address: Address,
}

impl<R> std::ops::Deref for Meta<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<R> AsRef<Address> for Meta<R> {
    fn as_ref(&self) -> &Address {
        &self.address
    }
}

impl<R> Clone for Meta<R> {
    fn clone(&self) -> Self {
        Meta {
            inner: Arc::clone(&self.inner),
            address: self.address.clone(),
        }
    }
}

/*
 * TODO: need to move this to a better place
 */
#[derive(Debug)]
struct FelipeFakeModule;

#[async_trait]
impl<C: Cloud> Creatable<C> for FelipeFakeModule {
    fn kind(&self) -> &'static str {
        "felipe_fake_module"
    }

    async fn create(
        &self,
        _provider: &<C as Cloud>::ProviderApi,
    ) -> Result<clutter::Fields, String> {
        Ok(clutter::Fields::empty())
    }
}

impl<C: Cloud> Provider<C> {
    pub fn new(api: C::ProviderApi) -> Self {
        Self {
            api,
            dependencies: DependencyTracking::new(),
        }
    }

    #[instrument(level="info", skip(self, builder, dependencies), fields(name, cloud = %C::NAME))]
    pub fn resource<F, O, const N: usize>(
        &mut self,
        name: &'static str,
        builder: F,
        dependencies: [&dyn AsRef<Address>; N],
    ) -> Meta<O>
    where
        F: FnOnce(&mut C::ProviderApi) -> O,
        O: Resource<C> + 'static,
    {
        let object = builder(&mut self.api);

        let kind = object.kind().to_string();
        let wrapped = Arc::new(object);

        let new_address = self.dependencies.child(
            kind,
            name.to_string(),
            Arc::clone(&wrapped) as Arc<dyn Creatable<C>>,
            DependencyKind::Resource,
        );

        for dependency in dependencies {
            self.dependencies.add_dependency(
                dependency.as_ref(),
                &new_address,
                DependencyKind::Resource,
            );
        }

        Meta {
            inner: wrapped,
            address: new_address,
        }
    }

    #[instrument(level="info", skip(self, definition, dependencies), fields(module_name, definition = std::any::type_name::<MD>(), cloud = %C::NAME))]
    pub fn module<MD, const N: usize>(
        &mut self,
        module_name: &'static str,
        definition: MD,
        dependencies: [&dyn AsRef<Address>; N],
    ) -> Meta<Module<MD, C>>
    where
        MD: ModuleDefinition<C>,
    {
        let not_really_the_module = FelipeFakeModule;
        let new_address = self.dependencies.child(
            "module",
            module_name,
            Arc::new(not_really_the_module),
            DependencyKind::Module,
        );

        info!("Hi there!");

        let old_address = self.dependencies.swap_own_address(new_address);
        let outputs = definition.define(self);
        let new_address = self.dependencies.swap_own_address(old_address);

        for dependency in dependencies {
            self.dependencies.add_dependency(
                dependency.as_ref(),
                &new_address,
                DependencyKind::Resource,
            );
        }

        Meta {
            inner: Arc::new(Module {
                name: module_name,
                outputs,
            }),
            address: new_address,
        }
    }

    pub async fn create(&self) -> Result<RealState, String> {
        let mut state = RealState::new();

        for (resource, address) in self.dependencies.iter() {
            let fields = resource.create(&self.api).await?;
            let resource_state = ResourceState::new(address, fields);

            state.add(resource_state);
        }

        Ok(state)
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

        async fn create(&self, _provider: &FakeApi) -> Result<clutter::Fields, String> {
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

        async fn create(&self, _provider: &FakeApi) -> Result<clutter::Fields, String> {
            // TODO: consider a sleep here...
            println!("Creating resource {} with {}", self.name, self.other.get());
            Ok(clutter::Fields::empty())
        }
    }

    struct FakeCloud;

    struct FakeApi;

    impl crate::Cloud for FakeCloud {
        type ProviderApi = FakeApi;
        const NAME: &'static str = "FakeCloud";
    }

    #[test]
    fn broad_idea_of_interdependencies() {
        smol::block_on(async {
            let mut provider: Provider<FakeCloud> = Provider::new(FakeApi);

            let slow = provider.resource("the_slow_one", |_api| FakeResource(23), []);

            let _fast = provider.resource(
                "the_fast_one",
                |_api| OtherResource {
                    name: "other_one",
                    other: slow.output(),
                },
                [&slow],
            );

            provider
                .create()
                .await
                .expect("should have been able to create resources from the provider");

            assert!(false);
        })
    }
}
