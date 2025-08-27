use std::collections::HashMap;

mod default;
mod indexed;

pub use default::DefaultVarResolver;
pub use indexed::IndexedVarResolver;

pub trait VarResolver
{
    fn get(&self, name: &str) -> Option<f64>;
}
