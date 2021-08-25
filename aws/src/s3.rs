use crate::iam::PolicyDocument;
use crate::{Arn, ArnBuilder, Tags};
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

impl Bucket {
    pub fn new(name: String) -> Bucket {
        Bucket {
            name,
            acl: Acl::default(),
            website: None,
            tags: Tags::empty(),
        }
    }

    pub fn arn(&self) -> Arn<Bucket> {
        ArnBuilder::default()
            .partition("arn")
            .service("s3")
            .relative_id(self.name.clone())
            .build()
            .unwrap()
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
