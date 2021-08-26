use crate::Produce;

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

// We'd have more impls for basic things here

impl Into<Value<String>> for String {
    fn into(self) -> Value<String> {
        Value::Real(self)
    }
}

impl Into<Value<&'static str>> for &'static str {
    fn into(self) -> Value<&'static str> {
        Value::Real(self)
    }
}
