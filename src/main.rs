#[macro_use]
extern crate derive_builder;

pub mod aws {
    use std::collections::HashMap;
    use std::fmt;

    pub struct Provider {
        region: String // TODO: Use enums for regions...
    }

    #[derive(Debug, Clone)]
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

    impl <T> fmt::Display for Arn<T> {
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
        use crate::aws::{Arn, ArnBuilder, Tags};
        use std::default::Default;

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
            index_document: String
        }
    }
}

fn main() {
    let x = aws::s3::Bucket::new("my_bucket".into());
    let arn = x.arn();

    println!("arn: {}", arn);
}
