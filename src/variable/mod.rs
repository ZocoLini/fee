use std::collections::HashMap;

mod default;
mod indexed;

pub use default::DefaultVarResolver;
pub use indexed::IndexedVarResolver;

pub trait VarResolver
{
    fn get(&self, name: &str) -> Option<f64>;
}

impl VarResolver for HashMap<String, f64>
{
    fn get(&self, name: &str) -> Option<f64>
    {
        self.get(name).copied()
    }
}
