use std::collections::HashMap;

pub trait VarResolver
{
    fn get(&self, name: &str) -> Option<f64>;
}

pub struct DefaultVarResolver
{
    vars: HashMap<String, f64>,
}

impl VarResolver for DefaultVarResolver 
{
    fn get(&self, name: &str) -> Option<f64> {
        self.vars.get(name).copied()
    }
}

impl DefaultVarResolver 
{
    pub fn new() -> Self {
        DefaultVarResolver {
            vars: HashMap::new()
        }
    }
    
    pub fn add_function(&mut self, name: String, val: f64) {
        self.vars.insert(name, val);
    }
}