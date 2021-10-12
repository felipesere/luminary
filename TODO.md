
## TODO:

* [x] Create a bucket from the definition
    * [x] Thread through credentials, probably with a "Provider"
* [x] Create an S3 object within the S3 bucket

Traits for types of things:
* [ ] Figure out a trait for "Resources" doing CRUD
    * [ ] Creating resources and producing state
    * [ ] RUD...
* [ ] Figure out a trait for DataSources

* [ ] dependencies between resources
    * [ ] Make the futures follow the dependency graph
    * [ ] Be able to write them to state...
    * [ ] ...and read them back from serialized state

* [ ] Refresh state

* [ ] Diff the S3 bucket state vs code definition
   * [ ] No changes present
   * [ ] Changes present

