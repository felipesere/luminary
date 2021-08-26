use aws::s3;
use aws::Arn;

use std::rc::Rc;

use luminary::{Module, Provider, Resource};

#[derive(Debug)]
struct MyWebsite {
    bucket: Rc<s3::Bucket>,
}

struct MyWebsiteOutput {
    pub arn: Arn<s3::Bucket>,
}

impl Module for MyWebsite {
    type Inputs = &'static str;
    type Outputs = MyWebsiteOutput;

    fn new(name: Self::Inputs) -> Self {
        let bucket = s3::Bucket::with(name)
            .website(s3::Website {
                index_document: "index.html".into(),
            })
            .build()
            .unwrap();

        let bucket = Rc::new(bucket);

        Self { bucket }
    }

    fn outputs(&self) -> Self::Outputs {
        MyWebsiteOutput {
            arn: self.bucket.arn(),
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), String> {
    let module = MyWebsite::new("my-unique-bucket");
    dbg!(&module);

    let provider = Provider {};

    let bucket_to_be_build = module.bucket;
    bucket_to_be_build.create(&provider).await?;

    Ok(())
}
