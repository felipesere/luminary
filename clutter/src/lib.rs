use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    version: usize,
    resources: Vec<ResourceState>,
}

impl State {
    pub fn new() -> Self {
        Self {
            version: 1,
            resources: Vec::new(),
        }
    }

    pub fn add(&mut self, resource: ResourceState) {
        self.resources.push(resource);
    }

    pub fn print(&self) {
        println!("{}", serde_json::to_string_pretty(&self).unwrap());
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceState {
    address: String,
    fields: Fields,
}

impl ResourceState {
    pub fn new(address: impl Into<String>, fields: Fields) -> Self {
        Self {
            address: address.into(),
            fields,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Change {
    OnlyLeft(Field),
    Changed(Field, Field),
    OnlyRight(Field),
}

impl<L, R> Into<Change> for (L, R)
where
    L: Into<Field>,
    R: Into<Field>,
{
    fn into(self) -> Change {
        let (l, r) = self;
        Change::Changed(l.into(), r.into())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Difference {
    field_name: String,
    change: Change,
}

impl Fields {
    pub fn empty() -> Fields {
        Fields(HashMap::new())
    }

    pub fn with_text(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.0.insert(name.into(), Field::Text(value.into()));
        self
    }

    pub fn with_number(mut self, name: impl Into<String>, value: impl Into<i32>) -> Self {
        self.0.insert(name.into(), Field::Number(value.into()));
        self
    }

    pub fn with_boolean(mut self, name: impl Into<String>, value: impl Into<bool>) -> Self {
        self.0.insert(name.into(), Field::Boolean(value.into()));
        self
    }

    pub fn with_object<F>(mut self, name: impl Into<String>, f: F) -> Self
    where
        F: Fn(Fields) -> Fields,
    {
        self.0
            .insert(name.into(), Field::Object(f(Fields::empty())));
        self
    }

    pub fn remove(mut self, name: impl AsRef<str>) -> Self {
        self.0.remove(name.as_ref());
        self
    }

    pub fn diff(&self, other: &Fields) -> Vec<Difference> {
        let our_keys: Vec<_> = self.0.keys().collect();
        let mut other_keys: Vec<_> = other.0.keys().collect();

        let mut differences = Vec::new();

        for k in &our_keys {
            let idx = other_keys.iter().position(|key| key == k);

            if let Some(idx) = idx {
                let ours = self.0.get(*k).expect("checked the key before");
                let others = other.0.get(*k).expect("checked the key before");

                if ours != others {
                    differences.push(Difference {
                        field_name: k.to_string(),
                        change: Change::Changed(ours.clone(), others.clone()),
                    });
                }
                other_keys.remove(idx);
            } else {
                let ours = self.0.get(*k).expect("checked the key before");

                differences.push(Difference {
                    field_name: k.to_string(),
                    change: Change::OnlyLeft(ours.clone()),
                });
            }
        }

        for k in &other_keys {
            let others = other
                .0
                .get(*k)
                .expect("the field must exist because grab all keys first");
            differences.push(Difference {
                field_name: k.to_string(),
                change: Change::OnlyRight(others.clone()),
            });
        }

        differences
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Fields(HashMap<String, Field>);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
enum Field {
    Text(String),
    Number(i32),
    Boolean(bool),
    Object(Fields),
    Array(Vec<Field>),
}

impl From<String> for Field {
    fn from(raw: String) -> Self {
        Field::Text(raw)
    }
}

impl From<&str> for Field {
    fn from(raw: &str) -> Self {
        Field::Text(raw.to_string())
    }
}

impl From<i32> for Field {
    fn from(raw: i32) -> Self {
        Field::Number(raw)
    }
}

impl From<f32> for Field {
    fn from(raw: f32) -> Self {
        Field::Number(raw as i32)
    }
}

impl From<bool> for Field {
    fn from(raw: bool) -> Self {
        Field::Boolean(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differences_between_field() {
        assert!(Field::Text("hi there".into()) != Field::Number(32));

        let hi_there = Field::Text("hi there".into());
        assert!(hi_there == hi_there);

        let person = Fields::empty()
            .with_text("name", "Steve")
            .with_number("age", 42);

        let other = Fields::empty()
            .with_text("name", "Steve")
            .with_number("age", 42);

        assert!(person == other);
    }

    #[test]
    fn differences_between_fields() {
        let mut person = Fields::empty()
            .with_text("name", "Steve")
            .with_number("age", 42)
            .with_boolean("some_bool", true);

        let other = person.clone().remove("some_bool");

        assert!(person != other);

        let diff = person.diff(&other);

        assert_eq!(diff.len(), 1);
        assert_eq!(
            diff[0],
            Difference {
                field_name: "some_bool".to_string(),
                change: Change::OnlyLeft(true.into())
            }
        );

        let diff = other.diff(&person);
        assert_eq!(diff.len(), 1);
        assert_eq!(
            diff[0],
            Difference {
                field_name: "some_bool".to_string(),
                change: Change::OnlyRight(true.into())
            }
        );

        person.0.remove("some_bool");
        person.0.insert("age".to_string(), Field::Number(1000));

        let diff = person.diff(&other);
        assert_eq!(diff.len(), 1);
        assert_eq!(
            diff[0],
            Difference {
                field_name: "age".to_string(),
                change: (1000, 42).into()
            }
        );
    }
}
