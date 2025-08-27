use std::collections::HashMap;

pub trait FnResolver
{
    fn call(&self, name: &str, args: &[f64]) -> Option<f64>;
}

pub struct DefaultFnResolver
{
    functions: HashMap<String, fn(&[f64]) -> f64>,
}

impl FnResolver for DefaultFnResolver
{
    fn call(&self, name: &str, args: &[f64]) -> Option<f64>
    {
        self.functions.get(name).map(|f| f(args))
    }
}

impl DefaultFnResolver
{
    pub fn new() -> Self
    {
        DefaultFnResolver {
            functions: HashMap::new(),
        }
    }

    pub fn add_function(&mut self, name: String, func: fn(&[f64]) -> f64)
    {
        self.functions.insert(name, func);
    }
}
