use std::collections::HashMap;

mod default;
mod indexed;
mod small;

pub use default::DefaultVarResolver;
pub use indexed::IndexedVarResolver;
pub use small::SmallVarResolver;

pub trait VarResolver
{
    fn get_var(&self, name: &str) -> Option<&f64>;
}

impl VarResolver for HashMap<String, f64>
{
    fn get_var(&self, name: &str) -> Option<&f64>
    {
        self.get(name)
    }
}
