#[macro_use]
extern crate derive_builder;

use std::collections::HashMap;
use std::fmt;

pub mod iam;
pub mod s3;

#[derive(Debug, Default, Clone)]
pub struct Tags(HashMap<String, String>);

impl Tags {
    fn empty() -> Self {
        Tags(HashMap::new())
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(setter(strip_option, into), default)]
pub struct Arn<T> {
    #[builder(setter(skip))]
    _marker: std::marker::PhantomData<T>,
    partition: Option<String>,
    service: Option<String>,
    region: Option<String>,
    namespace: Option<String>,
    relative_id: Option<String>,
}

impl<T> fmt::Display for Arn<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let aws = "aws".into();
        let empty = "".into();

        write!(
            f,
            "arn:{}:{}:{}:{}:{}",
            self.partition.as_ref().unwrap_or(&aws),
            self.service.as_ref().unwrap_or(&empty),
            self.region.as_ref().unwrap_or(&empty),
            self.namespace.as_ref().unwrap_or(&empty),
            self.relative_id.as_ref().unwrap_or(&empty),
        )
    }
}

impl<T> Default for Arn<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
            partition: None,
            service: None,
            region: None,
            namespace: None,
            relative_id: None,
        }
    }
}
