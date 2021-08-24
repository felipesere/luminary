#[macro_use]
extern crate derive_builder;

use std::collections::HashMap;
use std::fmt;

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

pub mod s3 {
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
}

pub mod iam {
    #[derive(Debug)]
    pub struct PolicyDocument {
        pub statements: Vec<PolicyStatement>,
    }

    #[derive(Clone, Debug)]
    pub enum Effect {
        Allow,
        Deny,
    }

    #[derive(Clone, Debug)]
    pub enum Principal {
        AWS(String),
    }

    #[derive(Clone, Debug)]
    pub struct Action(String);

    impl Action {
        pub fn new<S: Into<String>>(action: S) -> Action {
            Action(action.into())
        }
    }

    #[derive(Clone, Debug)]
    pub struct Resource(String);

    impl Resource {
        pub fn new<S: Into<String>>(action: S) -> Resource {
            Resource(action.into())
        }
    }

    #[derive(Builder, Debug, Clone)]
    pub struct PolicyStatement {
        #[builder(default)]
        pub sid: String,
        #[builder(default = "Effect::Allow")]
        pub effect: Effect,
        pub principal: Principal,
        pub actions: Vec<Action>,
        pub resources: Vec<Resource>,
    }

    impl PolicyStatementBuilder {
        pub fn allow(&mut self) -> &mut Self {
            let mut new = self;
            new.effect = Some(Effect::Allow);
            new
        }

        pub fn deny(&mut self) -> &mut Self {
            let mut new = self;
            new.effect = Some(Effect::Deny);
            new
        }

        pub fn action(&mut self, action: Action) -> &mut Self {
            let new = self;
            let actions = new.actions.get_or_insert_with(|| Vec::new());
            actions.push(action);
            new
        }

        pub fn resource(&mut self, resource: Resource) -> &mut Self {
            let new = self;
            let resources = new.resources.get_or_insert_with(|| Vec::new());
            resources.push(resource);
            new
        }
    }
}
