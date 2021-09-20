use std::any::Any;

use async_trait::async_trait;
use dyn_clone::DynClone;

mod value;

// Re-export
pub use value::Value;

// Will likely need some internal mutability
pub struct System {}

pub trait Module<C: Cloud>: std::fmt::Debug
where
    Self: Sized,
{
    type Inputs;
    type Outputs;
    type Providers;

    // TODO: Not sure about DesiredState here... it might move into some kind of Trait
    // that extracts connections?
    fn new(providers: &mut Self::Providers, input: Self::Inputs) -> Self::Outputs;
}

#[async_trait]
pub trait Resource<C: Cloud>: std::fmt::Debug + Send + Sync {
    async fn create(&self, provider: &<C as Cloud>::Provider) -> Result<RealState, String>; // Come up with a better error story
}

#[async_trait]
impl<T, C> Resource<C> for std::sync::Arc<T>
where
    C: Cloud,
    T: Resource<C> + Send + Sync,
{
    async fn create(&self, provider: &<C as Cloud>::Provider) -> Result<RealState, String> {
        self.as_ref().create(provider).await
    }
}

pub trait Cloud: Send + Sync {
    type Provider: Send + Sync;
}

pub trait Provider<C: Cloud>: Send + Sync {}

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
