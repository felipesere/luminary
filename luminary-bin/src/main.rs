use aws::s3;
use aws::Arn;
use async_trait::async_trait;
use aws_sdk_s3::{Client,Error};

use std::rc::Rc;

// TODO: move this somewhere better once its more flesshed out
trait Module: std::fmt::Debug {
    type Inputs;
    type Outputs;

    fn new(input: Self::Inputs) -> Self;

    fn outputs(&self) -> Self::Outputs;
}

// Will this be the bit that impls the API? Or just passes in some creds?
// What about Provider<T> for Provider<AWS> vs Provider<Azure>?
// Or even quirkier: Provider<AWS, S3>?
struct Provider {}

// This will somehow be used to store and refresh state?
struct State {}

#[async_trait]
trait Resource: std::fmt::Debug {
    async fn create(&self, provider: &Provider) -> Result<State, String>; // Come up with a better error story
}

#[async_trait]
impl Resource for s3::Bucket {
    async fn create(&self, _provider: &Provider) -> Result<State, String> {
        let client = Client::from_env();
        let request = client.create_bucket().set_bucket(Some(self.name.clone()));
        let response = request.send().await.map_err(|e| e.to_string())?;
        dbg!(response);
        Ok(State{})
    }
}

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

        Self {
            bucket,
        }
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

    let provider =  Provider{};

    let bucket_to_be_build = module.bucket;
    bucket_to_be_build.create(&provider).await?;

    Ok(())
}
