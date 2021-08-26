use async_trait::async_trait;
use dyn_clone::DynClone;

mod value;

// Re-export
pub use value::Value;

// TODO: move this somewhere better once its more flesshed out
pub trait Module: std::fmt::Debug {
    type Inputs;
    type Outputs;

    fn new(input: Self::Inputs) -> Self;

    fn outputs(&self) -> Self::Outputs;
}

#[async_trait]
pub trait Resource: std::fmt::Debug {
    async fn create(&self, provider: &Provider) -> Result<State, String>; // Come up with a better error story
}

pub struct Provider {}

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

