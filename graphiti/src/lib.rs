use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use dyn_clone::DynClone;

///
/// Can I construct a data structure "thing" that does the following:
/// * There is some kind of Anchor that holds its own name and all referrents
/// * That anchor aware of all places that "use" its value
/// * When we hand out a "Value<T>" to some other thing with an Anchor, we form a link
/// * Down the road, I want to use this to resolve dependencies
///   * for `A` to be produced, `B` needs to be produced first, which is based on links between A
///   and B
///   * In the above example `A` uses values from `B`

#[derive(Clone)]
pub struct Anchor(Arc<Inner>);

struct Inner {
    name: String,
    downstream: Mutex<HashSet<String>>,
}

impl Anchor {
    fn new<I: Into<String>>(name: I) -> Anchor {
        Anchor(Arc::new(
        Inner {
            name: name.into(),
            downstream: Mutex::new(HashSet::default()),
        }
        ))
    }

    fn attached_value<F, T>(&self, f: F) -> Value<T>
        where
            F: Fn() -> T + 'static + Clone,
    {
        Value::Tracked {
            parent: self.clone(),
            producer: Box::new(f),
        }
    }

    fn connect_to(&self, other: impl Into<String>) {
        self.0.downstream.lock().unwrap().insert(other.into());
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

#[derive(Clone)]
pub enum Value<T> {
    Real(T),
    Tracked {
        parent: Anchor,
        producer: Box<dyn Produce<T>>
    }, // still not sure about this one
}

/*
impl<T: ?Sized + Clone> Clone for Value<T> {
    fn clone(&self) -> Self {
        match self {
            Value::Real(r) => Value::Real(r.clone()),
            Value::Tracked(producer) => Value::Tracked(producer.clone()),
        }
    }
}
*/

// Will be used for something meaningful down the line
#[allow(dead_code)]
impl<T: Clone + 'static> Value<T> {
    fn get(&self, name: impl Into<String>) -> T {
        match self {
            Value::Real(ref s) => s.clone(),
            Value::Tracked{
                parent,
                producer,
            } => {
                parent.connect_to(name);
                producer.get()
            }
        }
    }

    fn map<F, U>(&self, transform: F) -> Value<U>
    where
        F: 'static + Clone + Fn(T) -> U,
    {
        match self {
            Value::Real(real) => Value::Real(transform(real.clone())),
            Value::Tracked{parent, producer} => {
                let step_one = producer.clone();
                let transformed = Box::new(move || {
                    let v = step_one.get();
                    transform(v)
                });

                Value::Tracked {
                    parent: parent.clone(),
                    producer: transformed,
                }
            }
        }
    }
}

// We'd have more impls for basic things here
impl From<String> for Value<String> {
    fn from(content: String) -> Self {
        Value::Real(content)
    }
}

impl From<&'static str> for Value<&'static str> {
    fn from(content: &'static str) -> Self {
        Value::Real(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct A {
        value: Value<String>,
    }

    struct B {
        anchor: Anchor,
    }

    impl B {
        fn some_value(&self) -> Value<String> {
            self.anchor.attached_value(|| {
                "Foo".to_string()
            })
        }
    }

    struct C {
        value: Value<String>,
    }

    impl C {
        fn new(value: Value<String>) -> Self {
            Self {
                value
            }
        }
    }

    #[test]
    fn it_works() {
        let b = B { anchor: Anchor::new("B") };
        let a = A { value: b.some_value() };

        assert!(b.anchor.0.downstream.lock().unwrap().contains("A"));
        assert_eq!(a.value.get("C"), "Foo".to_string());

        let c = C::new(b.some_value());
    }
}
