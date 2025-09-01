use std::collections::HashMap;

use super::Resolver;
use crate::{ExprFn, prelude::*};

/// General-purpose resolver that stores values indexed by name.
///
/// `DefaultResolver` uses a `HashMap` internally. While the `resolve()` method
/// has O(1) complexity, its performance is slower compared to other specialized
/// resolvers such as `SmallResolver` or `IndexedResolver`.  
///
/// # Advantages
/// - No restrictions on variable or function names.
/// - Performance remains stable even with a large number of items.
/// 
/// # Disadvantages
/// - Slow resolving names due to the hashing algorithm.
///
/// # Locking
/// This resolver can be locked to prevent further modifications to the cache.
/// When locked a method .get_var_pointer() is available to retrieve a pointer to a variable, 
/// allowing modifications to the variable's value without having to search for it 
/// in the cache.
/// 
/// ```rust
/// use fee::DefaultResolver;
/// use fee::prelude::*;
/// 
/// let mut var_resolver = DefaultResolver::new_empty();
/// var_resolver.insert("p0".to_string(), 10.0);
/// let mut var_resolver = var_resolver.lock();
/// 
/// let p0_ref = var_resolver.get_var_pointer("p0").unwrap();
/// 
/// unsafe {
///     *p0_ref = 20.0;
/// }
/// 
/// assert_eq!(*var_resolver.resolve("p0").unwrap(), 20.0);
/// ```
pub struct DefaultResolver<State, T>
{
    vars: HashMap<String, T>,
    _state: State,
}

impl<State, T> Resolver<T> for DefaultResolver<State, T>
{
    fn resolve(&self, name: &str) -> Option<&T>
    {
        self.vars.get(name)
    }
}

impl<T> DefaultResolver<Unlocked, T>
{
    pub fn new_empty() -> Self
    {
        DefaultResolver {
            vars: HashMap::new(),
            _state: Unlocked,
        }
    }

    pub fn insert(&mut self, name: String, val: T)
    {
        self.vars.insert(name, val);
    }
}

impl DefaultResolver<Unlocked, f64>
{
    pub fn new_var_resolver() -> Self
    {
        let mut hashmap = HashMap::new();

        hashmap.insert("pi".to_string(), std::f64::consts::PI);
        hashmap.insert("e".to_string(), std::f64::consts::E);
        hashmap.insert("tau".to_string(), std::f64::consts::TAU);
        hashmap.insert("sqrt2".to_string(), std::f64::consts::SQRT_2);

        DefaultResolver {
            vars: hashmap,
            _state: Unlocked,
        }
    }

    pub fn lock(self) -> DefaultResolver<Locked, f64>
    {
        DefaultResolver {
            vars: self.vars,
            _state: Locked,
        }
    }
}

impl DefaultResolver<Locked, f64>
{
    pub fn get_var_pointer(&mut self, name: &str) -> Option<*mut f64>
    {
        self.vars.get_mut(name).map(|v| v as *mut f64)
    }
}

impl DefaultResolver<Unlocked, ExprFn>
{
    pub fn new_fn_resolver() -> Self
    {
        let mut hashmap: HashMap<String, ExprFn> = HashMap::new();

        hashmap.insert("abs".to_string(), |x| x[0].abs());
        hashmap.insert("sqrt".to_string(), |x| x[0].sqrt());

        return DefaultResolver {
            vars: hashmap,
            _state: Unlocked,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_locked_resolver_get_var_mut_updates_value()
    {
        let mut var_resolver = DefaultResolver::new_empty();
        var_resolver.insert("x".to_string(), 10.0);

        let mut resolver = var_resolver.lock();

        let x = resolver.get_var_pointer("x").unwrap();
        unsafe { *x = 20.0 };

        assert_eq!(resolver.resolve("x"), Some(&20.0));
    }
}
