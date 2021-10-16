use async_trait::async_trait;
use dyn_clone::DynClone;

mod provider;
mod value;

// Re-export
pub use clutter::Fields;
pub use provider::Provider;
pub use value::Value;

// Will likely need some internal mutability
pub struct System {}

// The address of an object in Luminary
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Segment {
    pub name: String,
    pub kind: String,
}

impl ToString for Segment {
    fn to_string(&self) -> String {
        format!("{}.{}", self.kind, self.name)
    }
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

impl From<&Address> for String {
    fn from(address: &Address) -> Self {
        address
            .0
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("/")
    }
}

impl Address {
    pub fn root() -> Address {
        Address(vec![])
    }

    pub fn child(&self, child: Segment) -> Address {
        let mut other = self.clone();

        other.0.push(child);

        other
    }

    pub fn parent(&self) -> Address {
        let mut other = self.clone();
        if other.0.len() > 1 {
            other.0.pop();
        }
        other
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
    fn kind(&self) -> &'static str;
    async fn create(&self, provider: &<C as Cloud>::ProviderApi) -> Result<Fields, String>;
}

/// A very intersting trait that configures
/// how a a cloud works. Cloud here could be things
/// like `Aws`, or `Azure` and `GCP`
pub trait Cloud: Send + Sync {
    type ProviderApi: Send + Sync;
}

/// The state as it is known to our Cloud providers
/// We get this from refreshing the resources that
/// we see in `KnownState`.
pub type RealState = clutter::State;

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
