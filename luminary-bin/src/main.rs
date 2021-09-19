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

    // Somehow here I need to be able to attach the provider to the resource... Maybe `AwsProvider`
    // is the thing that I call `AwsProvider.s3().new() on?
    fn new(provider: &mut AwsProvider, name: Self::Inputs) -> (Self, DesiredState) {
        let bucket = provider.s3_bucket(name)
            .website(s3::Website {
                index_document: "index.html".into(),
            })
            .build()
            .unwrap();

        let bucket = Arc::new(bucket);

        // Will be replaced soon!
        provider.track(Box::new(Arc::clone(&bucket)));

        let file_object = provider.s3_bucket_object()
            .bucket(bucket.clone())
            .key("f.json")
            .content_type("application/json")
            .content("{\"key\": true}")
            .build()
            .unwrap();
        let object = Arc::new(file_object);

        provider.track(Box::new(Arc::clone(&object)));

        // Maybe the right signature is `(Outputs, DesiredState)` and the object itslef moves into
        // DesiredState???
        (Self { bucket, object }, DesiredState{})
    }

    fn outputs(&self) -> Self::Outputs {
        MyWebsiteOutput {
            arn: self.bucket.arn(),
        }
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

    fn new(providers: &mut Self::Providers, input: Self::Inputs) -> (Self, DesiredState) {
        let (first, s1) = MyWebsite::new(providers, input.0);
        let (second, s2) = MyWebsite::new(providers, input.1);
        let (third, s3) = MyWebsite::new(providers, input.2);

        (ThreeWebsites {
            sites: (first, second, third),
        }, s1.merge(s2).merge(s3))
    }

    fn outputs(&self) -> Self::Outputs {
        (
            self.sites.0.outputs(),
            self.sites.1.outputs(),
            self.sites.2.outputs(),
        )
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
