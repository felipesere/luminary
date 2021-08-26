use async_trait::async_trait;
use dyn_clone::DynClone;

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

// Will this be the bit that impls the API? Or just passes in some creds?
// What about Provider<T> for Provider<AWS> vs Provider<Azure>?
// Or even quirkier: Provider<AWS, S3>?
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

pub enum Value<T> {
    Real(T),
    Reference(Box<dyn Produce<T>>), // still not sure about this one
}

impl<T: ?Sized + Clone> Clone for Value<T> {
    fn clone(&self) -> Self {
        match self {
            Value::Real(r) => Value::Real(r.clone()),
            Value::Reference(producer) => Value::Reference(producer.clone()),
        }
    }
}

impl<T: Clone> Value<T> {
    fn get(&self) -> T {
        match self {
            Value::Real(ref s) => s.clone(),
            Value::Reference(producer) => producer.get(),
        }
    }

    /*
    fn map<F, U>(&self, f: F) -> Value<U>
    where
        F: Fn(T) -> U,
    {
        match self {
            Value::Real(r) => {
                Value::Real(f(r.clone()))
            },
            Value::Reference(producer) => {
                let other = producer.clone();

                Value::Reference(Box::new(move || {
                    let v = other.get();
                    f(v)
                }));
                todo!()
            },
        }
    }
    */
}

impl Into<Value<String>> for String {
    fn into(self) -> Value<String> {
        Value::Real(self)
    }
}

