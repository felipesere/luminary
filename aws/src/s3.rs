use crate::iam::PolicyDocument;
use crate::{Arn, ArnBuilder, Tags};
use std::default::Default;
use std::rc::Rc;
use luminary::{Resource, Provider, State, Value};
use aws_sdk_s3::{Client};
use async_trait::async_trait;

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
impl Resource for Bucket {
    async fn create(&self, _provider: &Provider) -> Result<State, String> {
        let client = Client::from_env();
        let request = client.create_bucket().set_bucket(Some(self.name.clone()));
        let response = request.send().await.map_err(|e| e.to_string())?;
        dbg!(response);
        Ok(State {})
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
    bucket: Rc<Bucket>, // TODO: something about id?
    key: String,
    content_type: String,
    content: String,
}

impl BucketObject {
    pub fn new(bucket: Rc<Bucket>, key: String, content_type: String, content: String) -> Self {
        BucketObject {
            bucket,
            key,
            content_type,
            content,
        }
    }
}

#[derive(Debug)]
pub struct BucketPolicy {
    pub bucket: Rc<Bucket>,
    pub policy: Rc<PolicyDocument>,
}
