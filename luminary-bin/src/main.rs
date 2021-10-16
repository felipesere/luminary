use aws::s3;
use aws::Arn;
use aws::Aws;
use aws::AwsApi;
use aws::AwsDetails;
use luminary::Address;
use luminary::DependencyKind;
use luminary::Provider;

use luminary::ModuleDefinition;

use petgraph::{dot::Dot, Graph};

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

    fn define(&self, provider: &mut Provider<Aws>) -> MyWebsiteOutput {
        let bucket = provider.resource("my-other-bucket", |api| {
            api.s3_bucket(self.bucket_name)
                .website(s3::Website {
                    index_document: "index.html".into(),
                })
                .build()
                .unwrap()
        });

        let _object = provider.resource("the-object", |api| {
            api.s3_bucket_object()
                // Just pass down a reference to a bucket,
                // .bucket(&bucket)
                .bucket(bucket.name())
                .key("f.json")
                .content_type("application/json")
                .content("{\"key\": true}")
                .build()
                .unwrap()
        });

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

    fn define(&self, providers: &mut Provider<Aws>) -> Self::Outputs {
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
    let details = AwsDetails::from_env().map_err(|e| format!("Missing env key: {}", e))?;

    let api = AwsApi::new(details);

    let mut provider: Provider<Aws> = Provider::new(api);

    let b = provider.resource("my-bucket", |api| {
        api.s3_bucket("lonely-bucket-rs-v1").build().unwrap()
    });

    let x = provider.module(
        "my-fancy-module",
        MyWebsite {
            bucket_name: "luminary-rs-module-1",
        },
    );
    b.depends_on(&mut provider.dependency_graph.write().unwrap(), [&x]);

    /*
    let _three_sites = provider.module(
        "three-websites",
        ThreeWebsites {
            sites: ("luminary-rs-1", "luminary-rs-2", "luminary-rs-3"),
        },
    );
    */

    let state = provider.create().await?;

    // state.print();

    let deps: Graph<Address, DependencyKind> = provider.dependency_graph.into_inner().unwrap();

    println!("{}", Dot::new(&deps));

    Ok(())
}
