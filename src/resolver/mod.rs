use std::collections::HashMap;

mod constant;
mod default;
mod indexed;
mod small;

pub use constant::ConstantResolver;
pub use default::DefaultResolver;
pub use indexed::IndexedResolver;
pub use small::SmallResolver;

pub trait Resolver<T>
{
    fn resolve(&self, name: &str) -> Option<&T>;
}

impl<T> Resolver<T> for HashMap<String, T>
{
    fn resolve(&self, name: &str) -> Option<&T>
    {
        self.get(name)
    }
}
