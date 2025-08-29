use std::collections::HashMap;

mod default;

pub use default::DefaultFnResolver;

pub trait FnResolver
{
    fn call_fn(&self, name: &str, args: &[f64]) -> Option<f64>;
}

impl FnResolver for HashMap<String, fn(&[f64]) -> f64>
{
    fn call_fn(&self, name: &str, args: &[f64]) -> Option<f64>
    {
        self.get(name).map(|f| f(args))
    }
}
