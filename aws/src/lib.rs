#[macro_use]
extern crate derive_builder;

use luminary::{Cloud, Resource};
use std::collections::HashMap;
use std::env::VarError;
use std::fmt;
use std::sync::{Arc, RwLock};

use aws_sdk_s3::{Config, Credentials};

pub mod iam;
pub mod s3;

struct Inner {
    creds: Credentials,
    region: String,
    tracked_resources: RwLock<Vec<Box<dyn Resource<Aws>>>>,
}

pub struct AwsProvider(Arc<Inner>);

impl Clone for AwsProvider {
    fn clone(&self) -> Self {
        AwsProvider(Arc::clone(&self.0))
    }
}

pub enum Aws {}

impl Cloud for Aws {
    type Provider = AwsProvider;
}

impl AwsProvider {
    pub fn from_keys(
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
    ) -> Self {
        AwsProvider(Arc::new(Inner {
            creds: Credentials::from_keys(access_key_id, secret_access_key, None),
            region: "us-east-1".into(), // TODO: pass in
            tracked_resources: RwLock::default(),
        }))
    }

    pub fn creds(&self) -> Credentials {
        self.0.creds.clone()
    }

    pub fn region(&self) -> String {
        self.0.region.clone()
    }

    pub fn from_env() -> Result<Self, VarError> {
        dotenv::dotenv().ok();
        let access_key_id = std::env::var("AWS_ACCESS_KEY_ID")?;
        let secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY")?;

        Ok(Self::from_keys(access_key_id, secret_access_key))
    }
}

impl AwsProvider {
    fn config(&self) -> Config {
        let region = aws_sdk_s3::Region::new(self.region());
        aws_sdk_s3::Config::builder()
            .region(region)
            .credentials_provider(self.creds())
            .build()
    }

    pub fn s3_bucket(&self, name: impl Into<String>) -> s3::BucketBuilder {
        let fresh_copy = self.clone();
        s3::BucketBuilder::new(fresh_copy, name)
    }

    pub fn s3_bucket_object(&mut self) -> s3::BucketObjectBuilder {
        s3::BucketObjectBuilder::default()
    }

    pub fn track(&mut self, resource: Box<dyn Resource<Aws>>) {
        // Huge TODO here!!!
        self.0.tracked_resources.write().unwrap().push(resource)
    }

    pub async fn create(&mut self) -> Result<(), String> {
        let resources = self.0.tracked_resources.read().unwrap();
        for resource in resources.iter() {
            resource.create(self).await?;
        }

        Ok(())
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
