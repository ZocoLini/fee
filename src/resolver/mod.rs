use std::collections::HashMap;

mod constant;
mod default;
mod empty;
mod indexed;
mod small;

pub use constant::ConstantResolver;
pub use default::DefaultResolver;
pub use empty::EmptyResolver;
pub use indexed::IndexedResolver;
pub use small::SmallResolver;

pub trait ResolverState {}

pub struct Locked;
pub struct Unlocked;

impl ResolverState for Locked {}
impl ResolverState for Unlocked {}

type VarPtr = *mut f64;
type FnPtr = fn(&[f64]) -> f64;

/// Trait for resolving values by name.
///
/// This trait can be implemented to resolve any type of value by a string key.  
/// Within this crate, it is primarily used for:
/// - Resolving **expression variables** to `f64` values.
/// - Resolving **function names** to `ExprFn` (functions taking a slice of `f64` and returning `f64`).
///
/// Implementors of this trait provide the lookup logic, for example:
/// - Using a `HashMap` or a small cache.
/// - Providing a custom storage mechanism.
/// - Or using a trivial resolver like `EmptyResolver`.
///
/// # Examples
/// ```rust
/// use fee::prelude::*;
///
/// pub struct ExampleResolver;
///
/// impl Resolver<f64> for ExampleResolver {
///     fn resolve(&self, _name: &str) -> Option<&f64> {
///         Some(&50.0)
///     }
/// }
/// ```
pub trait Resolver<State, T>
where
    State: ResolverState,
{
    fn resolve(&self, name: &str) -> Option<&T>;
}

impl<T> Resolver<Unlocked, T> for HashMap<String, T>
{
    fn resolve(&self, name: &str) -> Option<&T>
    {
        self.get(name)
    }
}
