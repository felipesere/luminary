use aws::s3;
use aws::Arn;
use aws::Aws;
use aws::AwsProvider;

use std::sync::Arc;

use luminary::ModuleDefinition;

#[derive(Debug)]
struct MyWebsite {
    bucket_name: &'static str,
}

// Will be used for something meaningful down the line
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct MyWebsiteOutput {
    pub arn: Arn<s3::Bucket>,
}

impl ModuleDefinition<Aws> for MyWebsite {
    type Inputs = &'static str;
    type Outputs = MyWebsiteOutput;
    type Providers = AwsProvider;

    fn define(&self, provider: &mut AwsProvider) -> MyWebsiteOutput {
        let bucket = provider
            .s3_bucket(self.bucket_name)
            .website(s3::Website {
                index_document: "index.html".into(),
            })
            .build()
            .unwrap();

        let _object = provider
            .s3_bucket_object()
            // Just pass down a reference to a bucket,
            // .bucket(&bucket)
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
    sites: (
        <MyWebsite as ModuleDefinition<Aws>>::Inputs,
        <MyWebsite as ModuleDefinition<Aws>>::Inputs,
        <MyWebsite as ModuleDefinition<Aws>>::Inputs,
    ),
}

impl ModuleDefinition<Aws> for ThreeWebsites {
    type Inputs = (
        <MyWebsite as ModuleDefinition<Aws>>::Inputs,
        <MyWebsite as ModuleDefinition<Aws>>::Inputs,
        <MyWebsite as ModuleDefinition<Aws>>::Inputs,
    );
    type Outputs = (
        <MyWebsite as ModuleDefinition<Aws>>::Outputs,
        <MyWebsite as ModuleDefinition<Aws>>::Outputs,
        <MyWebsite as ModuleDefinition<Aws>>::Outputs,
    );
    type Providers = AwsProvider;

    fn define(&self, providers: &mut Self::Providers) -> Self::Outputs {
        let first = providers.module(
            "first",
            MyWebsite {
                bucket_name: self.sites.0,
            },
        );
        let second = providers.module(
            "second",
            MyWebsite {
                bucket_name: self.sites.1,
            },
        );
        let third = providers.module(
            "third",
            MyWebsite {
                bucket_name: self.sites.2,
            },
        );

        (first.outputs(), second.outputs(), third.outputs())
    }
}

#[tokio::main]
pub async fn main() -> Result<(), String> {
    let mut provider = AwsProvider::from_env().map_err(|e| format!("Missing env key: {}", e))?;

    provider.s3_bucket("lonely-bucket-rs-v1").build().unwrap();

    let _x = provider.module(
        "my-fancy-module",
        MyWebsite {
            bucket_name: "my-bucket-name",
        },
    );

    let _three_sites = provider.module(
        "three-websites",
        ThreeWebsites {
            sites: ("luminary-rs-1", "luminary-rs-2", "luminary-rs-3"),
        },
    );

    // dbg!(&provider);
    // provider.create().await?;

    Ok(())
}
