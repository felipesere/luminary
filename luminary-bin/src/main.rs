use aws::s3;
use aws::Arn;
use aws::Aws;
use aws::AwsProvider;

use std::sync::Arc;

use luminary::{Module, Provider, Resource, System};

#[derive(Debug)]
struct MyWebsite {
    bucket: Arc<s3::Bucket>,
    object: Arc<s3::BucketObject>,
}

struct MyWebsiteOutput {
    pub arn: Arn<s3::Bucket>,
}

impl Module for MyWebsite {
    type Inputs = &'static str;
    type Outputs = MyWebsiteOutput;
    type Cloud = Aws;

    fn new(system: &mut System<Aws>, name: Self::Inputs) -> Self {
        let bucket = s3::Bucket::with(name)
            .website(s3::Website {
                index_document: "index.html".into(),
            })
            .build()
            .unwrap();

        let bucket = Arc::new(bucket);

        system.add(Box::new(bucket.clone()));

        let file_object =
            s3::BucketObject::new(bucket.clone(), "f.json", "json", "{\"key\": true}");
        let object = Arc::new(file_object);

        system.add(Box::new(object.clone()));

        Self { bucket, object }
    }

    fn outputs(&self) -> Self::Outputs {
        MyWebsiteOutput {
            arn: self.bucket.arn(),
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), String> {
    let mut system = System::new();

    let module = MyWebsite::new(&mut system, "luminary-rs-unique-v1");

    let provider =
        Box::new(AwsProvider::from_env().map_err(|e| format!("Missing env key: {}", e))?);

    system.create_with(provider).await?;

    Ok(())
}
