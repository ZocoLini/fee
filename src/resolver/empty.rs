use crate::prelude::Resolver;

pub struct EmptyResolver;

impl<T> Resolver<T> for EmptyResolver {
    fn resolve(&self, _name: &str) -> Option<&T> {
        None
    }
}

impl EmptyResolver {
    pub fn new() -> Self {
        EmptyResolver
    }
}