use aws::s3;
use aws::Arn;
use aws::Aws;
use aws::AwsProvider;
use luminary::DesiredState;

use std::sync::Arc;

use luminary::Module;

#[derive(Debug)]
struct MyWebsite {
    bucket: Arc<s3::Bucket>,
    object: Arc<s3::BucketObject>,
}

struct MyWebsiteOutput {
    pub arn: Arn<s3::Bucket>,
}

impl Module<Aws> for MyWebsite {
    type Inputs = &'static str;
    type Outputs = MyWebsiteOutput;
    type Providers = AwsProvider;

    fn new(provider: &mut AwsProvider, name: Self::Inputs) -> MyWebsiteOutput {
        let bucket = provider
            .s3_bucket(name)
            .website(s3::Website {
                index_document: "index.html".into(),
            })
            .build()
            .unwrap();

        let object = provider
            .s3_bucket_object()
            .bucket(Arc::clone(&bucket))
            .key("f.json")
            .content_type("application/json")
            .content("{\"key\": true}")
            .build()
            .unwrap();

        MyWebsiteOutput { arn: bucket.arn() }
    }
}

#[derive(Debug)]
struct ThreeWebsites {
    sites: (MyWebsite, MyWebsite, MyWebsite),
}

impl Module<Aws> for ThreeWebsites {
    type Inputs = (
        <MyWebsite as Module<Aws>>::Inputs,
        <MyWebsite as Module<Aws>>::Inputs,
        <MyWebsite as Module<Aws>>::Inputs,
    );
    type Outputs = (
        <MyWebsite as Module<Aws>>::Outputs,
        <MyWebsite as Module<Aws>>::Outputs,
        <MyWebsite as Module<Aws>>::Outputs,
    );
    type Providers = AwsProvider;

    fn new(providers: &mut Self::Providers, input: Self::Inputs) -> Self::Outputs {
        let first = MyWebsite::new(providers, input.0);
        let second = MyWebsite::new(providers, input.1);
        let third = MyWebsite::new(providers, input.2);

        (first, second, third)
    }
}

#[tokio::main]
pub async fn main() -> Result<(), String> {
    let mut provider = AwsProvider::from_env().map_err(|e| format!("Missing env key: {}", e))?;

    // let module = MyWebsite::new(&mut provider, "luminary-rs-unique-v1");
    //
    let _module = ThreeWebsites::new(
        &mut provider,
        ("luminary-rs-1", "luminary-rs-2", "luminary-rs-3"),
    );

    provider.create().await?;

    // let mut system = System::new();

    // system.create(provider).await;

    Ok(())
}
