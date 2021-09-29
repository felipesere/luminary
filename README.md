# Luminary

:wave: Welcome to Luminary!

This is a toy-like implementation of Terraform in Rust.
The idea is to see if I can write a nice Rust "frontend" that
leverages the type system and produces a Terraform-like experience.

## Non-goals

*Exhaustiveness*: There is a very tight Terraform `main.tf` in examples/main.tf and that should be enough. Not covering the entire AWS API.

*Flexibility*: No need to support multiple providers or anything. No need for versions either.

*Performance*: No need to make it fast. Let's see if it works in the first place.

## Things I want to play with

Code *structure/layout*: I want to figure out multiple-workspaces to emulate how an end-user would use this.

*Errors*: Get beautiful **and** useful errors from the get go. Lean on things like `miette` and other error libraries.

*AWS SDK*: Start with Rusoto and then lean on the new SDK when it becomes available.


## TODO:

* [ ] Create a bucket from the definition
    * [ ] Thread through credentials, probably with a "Provider"
    * [ ] Implement some kind of trait _for S3 bucket_ so each thing can define CRUD
* [ ] Empty state
* [ ] Create state entry from S3
* [ ] Refresh state
* [ ] Diff the S3 bucket state vs code definition
   * [ ] No changes present
   * [ ] Changes present
* [ ] Create an S3 object within the S3 bucket


## Links to Playground

I am currently stuck on a structural issue.
To share this with friends, I've created a _miniscule_
reproduction in the Playground.
You can find it [here](https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=8180ed521350b6b26ef723c992410d2f) and the
matching Gist [here](https://gist.github.com/rust-play/8180ed521350b6b26ef723c992410d2f)
