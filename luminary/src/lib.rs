use async_trait::async_trait;
use dyn_clone::DynClone;

mod value;

// Re-export
pub use value::Value;

// Will likely need some internal mutability
pub struct System<C> {
    resources: Vec<Box<dyn Resource<C>>>,
}

impl<C> System<C> {
    pub fn new() -> Self {
        System {
            resources: Vec::new(),
        }
    }

    pub fn add(&mut self, resource: Box<dyn Resource<C>>) {
        self.resources.push(resource);
    }

    pub async fn create_with(&mut self, provider: Box<dyn Provider<C>>) -> Result<(), String> {
        for resource in &self.resources {
            resource.create(&provider).await?;
        }
        Ok(())
    }
}

// TODO: move this somewhere better once its more flesshed out
pub trait Module: std::fmt::Debug {
    type Inputs;
    type Outputs;
    type Cloud: Cloud;

    fn new(sys: &mut System<Self::Cloud>, input: Self::Inputs) -> Self;

    fn outputs(&self) -> Self::Outputs;
}

#[async_trait]
pub trait Resource<C>: std::fmt::Debug {
    async fn create(&self, provider: &Box<dyn Provider<C>>) -> Result<State, String>; // Come up with a better error story
}

#[async_trait]
impl<T, C> Resource<C> for std::sync::Arc<T>
where
    T: Resource<C> + Send + Sync,
{
    async fn create(&self, provider: &Box<dyn Provider<C>>) -> Result<State, String> {
        self.as_ref().create(provider).await
    }
}

pub trait Cloud: Send + Sync {
    type SomethingFromTheProvider;
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
