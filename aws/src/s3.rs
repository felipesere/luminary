use crate::iam::PolicyDocument;
use crate::{Arn, ArnBuilder, Aws, AwsApi, Tags};
use async_trait::async_trait;
use aws_sdk_s3::{ByteStream, Client};

use luminary::{Creatable, RealState, Resource, Value};

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

#[derive(Clone, Debug)]
pub struct Bucket {
    pub name: String,
    pub acl: Acl,
    pub website: Option<Website>,
    pub tags: Tags,
}

pub struct BucketBuilder {
    name: String,
    acl: Acl,
    website: Option<Website>,
    tags: Tags,
}

impl BucketBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        BucketBuilder {
            name: name.into(),
            acl: Acl::default(),
            website: None,
            tags: Tags::empty(),
        }
    }

    pub fn website(mut self, website: Website) -> Self {
        self.website = Some(website);
        self
    }

    pub fn build(self) -> Bucket {
        Bucket {
            name: self.name.clone(),
            acl: self.acl,
            website: self.website,
            tags: self.tags,
        }
    }
}
impl Resource<Aws> for Bucket {}

#[async_trait]
impl Creatable<Aws> for Bucket {
    async fn create(&self, provider: &AwsApi) -> Result<RealState, String> {
        let config = provider.details.config();
        let client = Client::from_conf(config);

        let request = client.create_bucket().bucket(self.name.clone());
        let response = request.send().await.map_err(|e| e.to_string())?;
        dbg!(response);
        Ok(RealState {})
    }

    fn kind(&self) -> &'static str {
        "s3_bucket"
    }
}

impl Bucket {
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

#[derive(Clone, Debug)]
pub struct BucketObject {
    bucket: Arc<Bucket>, // TODO: something about id?
    key: String,
    content_type: String,
    content: String,
}

pub struct BucketObjectBuilder {
    bucket: Option<Arc<Bucket>>,
    key: Option<String>,
    content_type: Option<String>,
    content: Option<String>,
}

impl BucketObjectBuilder {
    pub fn new() -> Self {
        BucketObjectBuilder {
            bucket: None,
            key: None,
            content_type: None,
            content: None,
        }
    }

    pub fn build(self) -> BucketObject {
        BucketObject {
            bucket: self.bucket.unwrap(),
            key: self.key.unwrap(),
            content_type: self.content_type.unwrap(),
            content: self.content.unwrap(),
        }
    }

    pub fn bucket(mut self, bucket: Arc<Bucket>) -> Self {
        self.bucket = Some(bucket);
        self
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }
}

impl Resource<Aws> for BucketObject {}

#[async_trait]
impl Creatable<Aws> for BucketObject {
    async fn create(&self, provider: &AwsApi) -> Result<RealState, String> {
        let config = provider.details.config();
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

    fn kind(&self) -> &'static str {
        "s3_bucket_object"
    }
}

#[derive(Debug)]
pub struct BucketPolicy {
    pub bucket: Rc<Bucket>,
    pub policy: Rc<PolicyDocument>,
}
