#[derive(Debug)]
pub struct PolicyDocument {
    pub statements: Vec<PolicyStatement>,
}

#[derive(Clone, Debug)]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Clone, Debug)]
pub enum Principal {
    AWS(String),
}

#[derive(Clone, Debug)]
pub struct Action(String);

impl Action {
    pub fn new<S: Into<String>>(action: S) -> Action {
        Action(action.into())
    }
}

#[derive(Clone, Debug)]
pub struct Resource(String);

impl Resource {
    pub fn new<S: Into<String>>(action: S) -> Resource {
        Resource(action.into())
    }
}

#[derive(Builder, Debug, Clone)]
pub struct PolicyStatement {
    #[builder(default)]
    pub sid: String,
    #[builder(default = "Effect::Allow")]
    pub effect: Effect,
    pub principal: Principal,
    pub actions: Vec<Action>,
    pub resources: Vec<Resource>,
}

impl PolicyStatementBuilder {
    pub fn allow(&mut self) -> &mut Self {
        let mut new = self;
        new.effect = Some(Effect::Allow);
        new
    }

    pub fn deny(&mut self) -> &mut Self {
        let mut new = self;
        new.effect = Some(Effect::Deny);
        new
    }

    pub fn action(&mut self, action: Action) -> &mut Self {
        let new = self;
        let actions = new.actions.get_or_insert_with(Vec::new);
        actions.push(action);
        new
    }

    pub fn resource(&mut self, resource: Resource) -> &mut Self {
        let new = self;
        let resources = new.resources.get_or_insert_with(Vec::new);
        resources.push(resource);
        new
    }
}
