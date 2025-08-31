use crate::prelude::Resolver;

pub struct ConstantResolver<T>
{
    value: T,
}

impl<T> Resolver<T> for ConstantResolver<T>
{
    fn resolve(&self, _name: &str) -> Option<&T>
    {
        Some(&self.value)
    }
}

impl<T> ConstantResolver<T>
{
    pub fn new(value: T) -> Self
    {
        ConstantResolver { value }
    }

    pub fn set(&mut self, value: T)
    {
        self.value = value;
    }
}
