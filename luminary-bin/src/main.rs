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

fn main() {
    let module = MyWebsite::new("my-unique-bucket");
    dbg!(&module);

    let out = module.outputs();
    println!("{}", out.arn);
}
