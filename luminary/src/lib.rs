use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use dyn_clone::DynClone;

mod value;

// Re-export
pub use value::Value;

// Will likely need some internal mutability
pub struct System {}

// The address of an object in Luminary
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Segment {
    pub name: String,
    pub kind: String,
}

impl std::fmt::Debug for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.kind, self.name)
    }
}

// The address of an object in Luminary
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Address(Vec<Segment>);

impl std::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let address = self
            .0
            .iter()
            .map(|s| format!("{:?}", s))
            .collect::<Vec<_>>()
            .join("/");

        write!(f, "{}", address)
    }
}

impl Address {
    pub fn root() -> Address {
        Address(vec![Segment {
            kind: "".into(),
            name: "".into(),
        }])
    }

    pub fn child(&self, child: Segment) -> Address {
        let mut other = self.clone();

        other.0.push(child);

        other
    }

    pub fn parent(&self) -> Address {
        let mut other = self.clone();
        if other.0.len() <= 1 {
            other
        } else {
            other.0.pop();
            other
        }
    }
}

// Sort of part of the addressing system?
// A module should form "a scope",
// Any submodule should a fresh scope with
// a parent scope...
pub struct Scope {}

pub struct Module<MD, C>
where
    C: Cloud,
    MD: ModuleDefinition<C>,
{
    pub name: &'static str,
    pub outputs: <MD as ModuleDefinition<C>>::Outputs,
}

impl<C, MD> Module<MD, C>
where
    C: Cloud,
    MD: ModuleDefinition<C>,
    <MD as ModuleDefinition<C>>::Outputs: Clone,
{
    pub fn outputs(&self) -> MD::Outputs {
        self.outputs.clone()
    }
}

pub trait ModuleDefinition<C: Cloud>: std::fmt::Debug
where
    Self: Sized,
{
    type Inputs;
    type Outputs;

    // TODO is this right?
    fn define(&self, providers: &mut Provider<C>) -> Self::Outputs;
}

#[async_trait]
pub trait Resource<C: Cloud>: Creatable<C> + std::fmt::Debug + Send + Sync {}

#[async_trait]
pub trait Creatable<C: Cloud>: std::fmt::Debug + Send + Sync {
    async fn create(&self, provider: &<C as Cloud>::ProviderApi) -> Result<RealState, String>;
}

#[async_trait]
impl<T, C> Resource<C> for std::sync::Arc<T>
where
    C: Cloud,
    T: Resource<C> + Send + Sync,
{
}

#[async_trait]
impl<T, C> Creatable<C> for std::sync::Arc<T>
where
    C: Cloud,
    T: Resource<C> + Send + Sync,
{
    async fn create(&self, provider: &<C as Cloud>::ProviderApi) -> Result<RealState, String> {
        self.as_ref().create(provider).await
    }
}

/// A very intersting trait that configures
/// how a a cloud works. Cloud here could be things
/// like `Aws`, or `Azure` and `GCP`
pub trait Cloud: Send + Sync {
    type Provider: Send + Sync; // TODO: deprecate this
    type ProviderApi: Send + Sync;
}

pub struct Provider<C: Cloud> {
    api: C::ProviderApi,
    tracked_resources: RwLock<HashMap<Address, Arc<dyn Creatable<C>>>>,
    current_address: RwLock<Address>,
}

impl<C: Cloud> Deref for Provider<C> {
    type Target = C::ProviderApi;

    fn deref(&self) -> &Self::Target {
        &self.api
    }
}

impl<C: Cloud> DerefMut for Provider<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.api
    }
}

impl<C: Cloud> Provider<C> {
    pub fn new(api: C::ProviderApi) -> Self {
        Self {
            api,
            tracked_resources: Default::default(),
            current_address: RwLock::new(Address::root()),
        }
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
}

/// The state as it is known to our Cloud providers
/// We get this from refreshing the resources that
/// we see in `KnownState`.
pub struct RealState {}

/// The state as it was reloaded from storage and is known to luminary.
/// It may not be what is desired or even real, but it represents
/// what we knew last time we ran.
/// It will contain references to providers, resources, and attributes.
pub struct KnownState {}

/// The state as the operator would like to have it.
/// It is a more portable, generirc representation of what we have designed with Rust as code.
/// It will contain references to providers, resources, and attributes.
/// If an `apply` operation is successful the `DesiredState` should become the `KnownState`
/// and match up with `RealState`
pub struct DesiredState {}

impl DesiredState {
    /// Combines both states to be tracked together
    pub fn merge(self, _other: DesiredState) -> Self {
        // TODO
        self
    }
}

pub trait Produce<T>: DynClone {
    fn get(&self) -> T;
}

// Here be dragons...
dyn_clone::clone_trait_object!(<T>Produce<T>);

impl<T, F> Produce<T> for F
where
    F: Fn() -> T + Clone,
{
    fn get(&self) -> T {
        self()
    }
}
