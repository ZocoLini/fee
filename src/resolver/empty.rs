use crate::{
    prelude::*,
    resolver::{Locked, LockedResolver, ResolverState, Unlocked, UnlockedResolver},
};

/// A resolver that does not resolve any values.
///
/// Every call to `resolve()` returns `None`.
/// This resolver is useful when expressions do not contain any variables or function calls.
///
/// When calling `eval_without_context()` on any `Evaluator`, the method internally calls
/// `eval()` using a `Context` that contains two `EmptyResolver`s.
pub struct EmptyResolver<S>
where
    S: ResolverState,
{
    _state: S,
}

impl<T> LockedResolver<T> for EmptyResolver<Locked> {}
impl<T> UnlockedResolver<T, EmptyResolver<Locked>> for EmptyResolver<Unlocked>
{
    fn lock(self) -> EmptyResolver<Locked>
    {
        EmptyResolver { _state: Locked }
    }
}

impl<S, T> Resolver<S, T> for EmptyResolver<S>
where
    S: ResolverState,
{
    fn resolve(&self, _name: &str) -> Option<&T>
    {
        None
    }
}

impl EmptyResolver<Unlocked>
{
    pub fn new() -> Self
    {
        EmptyResolver { _state: Unlocked }
    }
}
