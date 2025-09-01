use crate::prelude::Resolver;

/// A resolver that does not resolve any values.
///
/// Every call to `resolve()` returns `None`.  
/// This resolver is useful when expressions do not contain any variables or function calls.
///
/// When calling `eval_without_context()` on any `Evaluator`, the method internally calls
/// `eval()` using a `Context` that contains two `EmptyResolver`s.
pub struct EmptyResolver;

impl<T> Resolver<T> for EmptyResolver
{
    fn resolve(&self, _name: &str) -> Option<&T>
    {
        None
    }
}

impl EmptyResolver
{
    pub fn new() -> Self
    {
        EmptyResolver
    }
}
