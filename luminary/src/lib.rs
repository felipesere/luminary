use async_trait::async_trait;
use dyn_clone::DynClone;

mod value;

// Re-export
pub use value::Value;

// Will likely need some internal mutability
pub struct System<C> {
    resources: Vec<Box<dyn Resource<C>>>,
}

// TODO: This whole notion of "system" will need to change
impl<C: Cloud> System<C> {
    pub fn new() -> Self {
        System {
            resources: Vec::new(),
        }
    }

    pub fn add(&mut self, resource: Box<dyn Resource<C>>) {
        self.resources.push(resource);
    }

    pub async fn create(&mut self, provider: impl Provider<C>) {}

    pub async fn create_with(&mut self, provider: Box<dyn Provider<C>>) -> Result<(), String> {
        for resource in &self.resources {
            // TODO resource.create(&provider).await?;
        }
        Ok(())
    }
}

pub trait Module<C: Cloud>: std::fmt::Debug {
    type Inputs;
    type Outputs;
    type Providers;

    fn new(providers: &mut Self::Providers, input: Self::Inputs) -> Self;

    fn outputs(&self) -> Self::Outputs;
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
