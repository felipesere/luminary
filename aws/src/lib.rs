#[macro_use]
extern crate derive_builder;

use luminary::{Address, Cloud, Creatable, Module, ModuleDefinition, Segment};
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
    tracked_resources: RwLock<HashMap<Address, Arc<dyn Creatable<Aws>>>>,
    current_address: RwLock<Address>, // Really? this is annoying? because its behind an Arc?
}

pub struct AwsProvider(Arc<Inner>);

impl fmt::Debug for AwsProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AwsProvider")
            .field("region", &self.0.region)
            .field("credentials", &self.0.creds)
            .field("resources", &self.0.tracked_resources)
            .field("current_address", &self.0.current_address)
            .finish()
    }
}

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
            current_address: RwLock::new(Address::root()),
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
        let fresh_copy = self.clone();
        s3::BucketObjectBuilder::new(fresh_copy)
    }

    // Can this be done better?
    pub fn track(&mut self, relative_address: Segment, resource: Arc<dyn Creatable<Aws>>) {
        let real = self
            .0
            .current_address
            .read()
            .unwrap()
            .child(relative_address);
        println!("Tracking {:?}", real);
        self.0
            .tracked_resources
            .write()
            .unwrap()
            .insert(real, resource);
    }

    pub async fn create(&mut self) -> Result<(), String> {
        let resources = self.0.tracked_resources.read().unwrap();
        for (_, resource) in resources.iter() {
            resource.create(self).await?;
        }

        Ok(())
    }

    pub fn module<MD>(&mut self, module_name: &'static str, definition: MD) -> Module<MD, Aws>
    where
        MD: ModuleDefinition<Aws, Providers = <Aws as Cloud>::Provider>,
    {
        let current_address = self.0.current_address.read().unwrap().clone();
        let module_address = current_address.child(Segment {
            kind: "module".into(),
            name: module_name.into(),
        });
        *self.0.current_address.write().unwrap() = module_address;

        let outputs = definition.define(self);

        *self.0.current_address.write().unwrap() = current_address;

        Module {
            name: module_name,
            outputs,
            definition: std::marker::PhantomData,
            cloud: std::marker::PhantomData,
        }
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
