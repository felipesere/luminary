use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use clutter::ResourceState;

use crate::{Address, Cloud, Creatable, Module, ModuleDefinition, RealState, Segment};

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
        O: Creatable<C> + 'static,
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
