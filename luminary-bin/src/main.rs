use aws::iam::{self, Action, Principal, Resource};
use aws::s3;
use aws::Arn;
use std::rc::Rc;

trait Module: std::fmt::Debug {
    type Inputs;
    type Outputs;

    fn new(input: Self::Inputs) -> Self;

    fn outputs(&self) -> Self::Outputs;
}

#[derive(Debug)]
struct MyWebsite {
    bucket: Rc<s3::Bucket>,
    index_page: s3::BucketObject,
    public_can_read: Rc<iam::PolicyDocument>,
    can_read: s3::BucketPolicy,
}

struct MyWebsiteOutput {
    pub arn: Arn<s3::Bucket>,
}

impl Module for MyWebsite {
    type Inputs = &'static str;
    type Outputs = MyWebsiteOutput;

    fn new(input: Self::Inputs) -> Self {
        let bucket = s3::BucketBuilder::default()
            .name(input.to_string())
            .website(s3::Website {
                index_document: "index.html".into(),
            })
            .build()
            .unwrap();
        let bucket = Rc::new(bucket);

        let index_page = s3::BucketObject::new(
            bucket.clone(),
            "index.html".into(),
            "text/html".into(),
            "hi this is the content!".into(),
        );

        let public_can_read = Rc::new(iam::PolicyDocument {
            statements: vec![iam::PolicyStatementBuilder::default()
                .allow()
                .principal(Principal::AWS("*".into()))
                .action(Action::new("s3:GetObject"))
                .resource(Resource::new(bucket.arn().to_string()))
                .resource(Resource::new(format!("{}/*", bucket.arn().to_string())))
                .build()
                .unwrap()],
        });

        let can_read = s3::BucketPolicy {
            bucket: bucket.clone(),
            policy: public_can_read.clone(),
        };

        Self {
            bucket,
            index_page,
            public_can_read,
            can_read,
        }
    }

    fn outputs(&self) -> Self::Outputs {
        MyWebsiteOutput {
            arn: self.bucket.arn(),
        }
    }
}

fn main() {
    let module = MyWebsite::new("my-unique-bucket");
    dbg!(&module);

    let out = module.outputs();
    println!("{}", out.arn);
}
