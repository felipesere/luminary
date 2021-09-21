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

// Will be used for something meaningful down the line
#[allow(dead_code)]
impl<T: Clone + 'static> Value<T> {
    fn get(&self) -> T {
        match self {
            Value::Real(ref s) => s.clone(),
            Value::Reference(producer) => producer.get(),
        }
    }

    fn map<F, U>(&self, transform: F) -> Value<U>
    where
        F: 'static + Clone + Fn(T) -> U,
    {
        match self {
            Value::Real(real) => Value::Real(transform(real.clone())),
            Value::Reference(producer) => {
                let step_one = producer.clone();

                Value::Reference(Box::new(move || {
                    let v = step_one.get();
                    transform(v)
                }))
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

    #[test]
    fn it_can_map_real_values() {
        let value = Value::Real(12);

        let transformed = value.map(|v| v + 100);

        assert_eq!(value.get(), 12);
        assert_eq!(transformed.get(), 112);
    }

    #[test]
    fn it_can_map_referenced_value() {
        let value = Value::Reference(Box::new(|| 12));

        let transformed = value.map(|v| v + 100);

        assert_eq!(value.get(), 12);
        assert_eq!(transformed.get(), 112);
    }
}
