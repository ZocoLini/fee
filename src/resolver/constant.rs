use crate::{
    prelude::*,
    resolver::{Locked, LockedResolver, ResolverState, Unlocked, UnlockedResolver},
};

/// For the cases where any variable or function name should resolve for the same value
/// we encourage the use of this resolver instead of any other one.
///
/// # Advantages
/// - Best performance between all resolvers.
///
/// # Disadvantages
/// - Always returns the same value regardless of the variable or function name.
///
/// # Examples
/// ```rust
/// use fee::prelude::*;
/// use fee::{ RpnEvaluator, EmptyResolver, ConstantResolver };
///
/// let expr = "x + y";
///
/// let var_resolver = ConstantResolver::new(1.0);
/// let context = Context::new(var_resolver, EmptyResolver::new());
///
/// let evaluator = RpnEvaluator::new(expr).unwrap();
/// let result = evaluator.eval(&context).unwrap();
/// assert_eq!(result, 2.0);
/// ```
pub struct ConstantResolver<S, T>
where
    S: ResolverState,
{
    value: T,

    _state: S,
}

impl<T> LockedResolver<T> for ConstantResolver<Locked, T> {}
impl<T> UnlockedResolver<T, ConstantResolver<Locked, T>> for ConstantResolver<Unlocked, T>
{
    fn lock(self) -> ConstantResolver<Locked, T>
    {
        ConstantResolver {
            value: self.value,
            _state: Locked,
        }
    }
}

impl<S, T> Resolver<S, T> for ConstantResolver<S, T>
where
    S: ResolverState,
{
    fn resolve(&self, _name: &str) -> Option<&T>
    {
        Some(&self.value)
    }
}

impl<T> ConstantResolver<Unlocked, T>
{
    pub fn new(value: T) -> Self
    {
        ConstantResolver {
            value,
            _state: Unlocked,
        }
    }
}

impl<S, T> ConstantResolver<S, T>
where
    S: ResolverState,
{
    pub fn set(&mut self, value: T)
    {
        self.value = value;
    }
}
