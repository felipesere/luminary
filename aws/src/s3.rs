use crate::iam::PolicyDocument;
use crate::{Arn, ArnBuilder, Aws, AwsApi, Tags};
use async_trait::async_trait;
use aws_sdk_s3::{ByteStream, Client};

use luminary::{Creatable, Fields, Resource, Value};
use tracing::{info};

use std::default::Default;
use std::rc::Rc;

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
#[builder(setter(strip_option, into), pattern = "owned")]
pub struct Bucket {
    pub name: String,
    #[builder(default)]
    pub acl: Acl,
    #[builder(default)]
    pub website: Option<Website>,
    #[builder(default)]
    pub tags: Tags,
}

impl BucketBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        BucketBuilder::default().name(name)
    }
}
impl Resource<Aws> for Bucket {}

#[async_trait]
impl Creatable<Aws> for Bucket {
    async fn create(&self, provider: &AwsApi) -> Result<Fields, String> {
        let config = provider.details.config();
        let client = Client::from_conf(config);

        let request = client.create_bucket().bucket(&self.name);
        info!("creating {}", self.name);
        request.send().await.map_err(|e| e.to_string())?;
        info!("created {}", self.name);

        let mut fields = Fields::empty().with_text("id", self.name.clone());

        if let Some(website) = &self.website {
            fields = fields.with_object("website", |o| {
                o.with_text("index_document", &website.index_document)
            });
        }

        Ok(fields)
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

    pub fn name(&self) -> Value<String> {
        Value::Real(self.name.clone())
    }
}

#[derive(Debug, Clone)]
pub struct Website {
    pub index_document: String,
}

#[derive(Builder, Clone, Debug)]
#[builder(setter(strip_option, into), pattern = "owned")]
pub struct BucketObject {
    bucket: Value<String>,
    key: String,
    content_type: String,
    content: String,
}

impl BucketObjectBuilder {
    pub fn new() -> Self {
        BucketObjectBuilder::default()
    }
}

impl Resource<Aws> for BucketObject {}

#[async_trait]
impl Creatable<Aws> for BucketObject {
    async fn create(&self, provider: &AwsApi) -> Result<Fields, String> {
        let config = provider.details.config();
        let client = Client::from_conf(config);

        let bucket_name = self.bucket.get();

        let request = client
            .put_object()
            .bucket(&bucket_name)
            .key(&self.key)
            .content_type(&self.content_type)
            .body(ByteStream::from(self.content.clone().into_bytes()));

        info!("creating object for {}/{}", &bucket_name, &self.key);
        request.send().await.map_err(|e| e.to_string())?;

        Ok(Fields::empty())
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
