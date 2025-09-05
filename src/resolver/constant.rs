use crate::prelude::*;

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
pub struct ConstantResolver<T>
{
    value: T,
}

impl<T> LockedResolver for ConstantResolver<T> {}
impl<T> UnlockedResolver for ConstantResolver<T> {}

impl<T> Resolver<Locked, T> for ConstantResolver<T>
{
    fn resolve(&self, _name: &str) -> Option<&T>
    {
        Some(&self.value)
    }
}

impl<T> Resolver<Unlocked, T> for ConstantResolver<T>
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
