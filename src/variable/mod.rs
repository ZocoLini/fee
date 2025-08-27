use std::collections::HashMap;

mod default;

pub use default::DefaultVarResolver;

pub trait VarResolver
{
    fn get(&self, name: &str) -> Option<f64>;
}

