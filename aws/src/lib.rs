#[macro_use]
extern crate derive_builder;

use luminary::{Cloud, Provider};
use std::collections::HashMap;
use std::env::VarError;
use std::fmt;

use aws_sdk_s3::{Config, Credentials};

pub mod iam;
pub mod s3;

pub struct AwsProvider {
    creds: Credentials,
    region: String,
}

pub enum Aws {}

impl Cloud for Aws {
    type SomethingFromTheProvider = Config;
}

impl AwsProvider {
    pub fn from_keys(
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
    ) -> Self {
        AwsProvider {
            creds: Credentials::from_keys(access_key_id, secret_access_key, None),
            region: "us-east-1".into(), // TODO: pass in
        }
    }

    pub fn from_env() -> Result<Self, VarError> {
        dotenv::dotenv().ok();
        let access_key_id = std::env::var("AWS_ACCESS_KEY_ID")?;
        let secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY")?;

        Ok(Self::from_keys(access_key_id, secret_access_key))
    }
}

impl Provider<Aws> for AwsProvider {
    fn get(&self) -> Config {
        let region = aws_sdk_s3::Region::new(self.region.clone());
        aws_sdk_s3::Config::builder()
            .region(region)
            .credentials_provider(self.creds.clone())
            .build()
    }
}

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
