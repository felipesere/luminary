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

// TODO: move this somewhere better once its more flesshed out
pub trait Module<C: Cloud>: std::fmt::Debug {
    type Inputs;
    type Outputs;
    type Providers;

    fn new(providers: &mut Self::Providers, input: Self::Inputs) -> Self;

    fn outputs(&self) -> Self::Outputs;
}

#[async_trait]
pub trait Resource<C>: std::fmt::Debug + Send + Sync
where
    C: Cloud,
{
    async fn create(&self, provider: &<C as Cloud>::Provider) -> Result<State, String>; // Come up with a better error story
}

#[async_trait]
impl<T, C> Resource<C> for std::sync::Arc<T>
where
    C: Cloud,
    T: Resource<C> + Send + Sync,
{
    async fn create(&self, provider: &<C as Cloud>::Provider) -> Result<State, String> {
        self.as_ref().create(provider).await
    }
}

pub trait Cloud: Send + Sync {
    type SomethingFromTheProvider;
    type Provider: Send + Sync;
}

pub trait Provider<C: Cloud>: Send + Sync {
    fn get(&self) -> <C as Cloud>::SomethingFromTheProvider;
}

// This will somehow be used to store and refresh state?
pub struct State {}

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
