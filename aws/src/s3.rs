use crate::iam::PolicyDocument;
use crate::{Arn, ArnBuilder, Aws, AwsProvider, Tags};
use async_trait::async_trait;
use aws_sdk_s3::{ByteStream, Client};

use luminary::{RealState, Resource, Value};

use std::default::Default;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Acl {
    Private,
    PublicRead,
}

impl Default for Acl {
    fn default() -> Self {
        Acl::Private
    }
}

#[derive(Builder, Clone, Debug)]
#[builder(setter(strip_option, into))]
pub struct Bucket {
    pub name: String,
    #[builder(default)]
    pub acl: Acl,
    #[builder(default)]
    pub website: Option<Website>,
    #[builder(default)]
    pub tags: Tags,
}

#[async_trait]
impl Resource<Aws> for Bucket {
    async fn create(&self, provider: &AwsProvider) -> Result<RealState, String> {
        let config = provider.config();
        let client = Client::from_conf(config);

        let request = client.create_bucket().bucket(self.name.clone());
        let response = request.send().await.map_err(|e| e.to_string())?;
        dbg!(response);
        Ok(RealState {})
    }
}

impl Bucket {
    pub fn new(name: String) -> Bucket {
        Bucket {
            name,
            acl: Acl::default(),
            website: None,
            tags: Tags::empty(),
        }
    }

    pub fn with<S: Into<String>>(name: S) -> BucketBuilder {
        let mut build = BucketBuilder::default();
        build.name(name);
        build
    }

    pub fn arn(&self) -> Arn<Bucket> {
        ArnBuilder::default()
            .partition("arn")
            .service("s3")
            .relative_id(self.name.clone())
            .build()
            .unwrap()
    }

    // This one is a bit contrived:
    // We know from `arn` that we can construct the arn just fine...
    // The only "point" might be that "name" itself could be a `Value<T>`?
    pub fn arn2(&self) -> Value<Arn<Bucket>> {
        // Would this be better if it was an RC?
        // Is there something that lives "long enough" that I could borrow from?
        let x = self.clone();
        Value::Reference(Box::new(move || x.arn()))
    }

    pub fn name(&self) -> Value<String> {
        Value::Real(self.name.clone())
    }
}

#[derive(Debug, Clone)]
pub struct Website {
    pub index_document: String,
}

#[derive(Debug)]
pub struct BucketObject {
    bucket: Arc<Bucket>, // TODO: something about id?
    key: String,
    content_type: String,
    content: String,
}

impl BucketObject {
    pub fn new(
        bucket: Arc<Bucket>,
        key: impl Into<String>,
        content_type: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        BucketObject {
            bucket,
            key: key.into(),
            content_type: content_type.into(),
            content: content.into(),
        }
    }
}

#[async_trait]
impl Resource<Aws> for BucketObject {
    async fn create(&self, provider: &AwsProvider) -> Result<RealState, String> {
        let config = provider.config();
        let client = Client::from_conf(config);

        let request = client
            .put_object()
            .bucket(&self.bucket.name)
            .key(&self.key)
            .content_type(&self.content_type)
            .body(ByteStream::from(self.content.clone().into_bytes()));
        let response = request.send().await.map_err(|e| e.to_string())?;

        dbg!(response);

        Ok(RealState {})
    }
}

#[derive(Debug)]
pub struct BucketPolicy {
    pub bucket: Rc<Bucket>,
    pub policy: Rc<PolicyDocument>,
}
