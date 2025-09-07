use std::{borrow::Borrow, collections::HashMap, hash::Hash};

use super::Resolver;
use crate::{
    prelude::*,
    resolver::{Locked, LockedResolver, ResolverState, Unlocked, UnlockedResolver},
};

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
pub struct DefaultResolver<S, K, V>
where
    S: ResolverState,
    K: Borrow<str> + PartialEq<String> + Eq + Hash,
{
    vars: HashMap<K, V>,
    _state: S,
}

impl<K, V> LockedResolver<V> for DefaultResolver<Locked, K, V> where
    K: Borrow<str> + PartialEq<String> + Eq + Hash
{
}
impl<K, V> UnlockedResolver<V, DefaultResolver<Locked, K, V>> for DefaultResolver<Unlocked, K, V>
where
    K: Borrow<str> + PartialEq<String> + Eq + Hash,
{
    fn lock(self) -> DefaultResolver<Locked, K, V>
    {
        DefaultResolver {
            vars: self.vars,
            _state: Locked,
        }
    }
}

impl<S, K, V> Resolver<S, V> for DefaultResolver<S, K, V>
where
    S: ResolverState,
    K: Borrow<str> + PartialEq<String> + Eq + Hash,
{
    fn resolve(&self, name: &str) -> Option<&V>
    {
        self.vars.get(name)
    }
}

impl<K, V> DefaultResolver<Unlocked, K, V>
where
    K: Borrow<str> + PartialEq<String> + Eq + Hash,
{
    pub fn empty() -> Self
    {
        DefaultResolver {
            vars: HashMap::new(),
            _state: Unlocked,
        }
    }

    pub fn insert(&mut self, name: K, val: V)
    {
        self.vars.insert(name, val);
    }
}

impl DefaultResolver<Unlocked, String, f64>
{
    pub fn new_vars() -> Self
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
}

impl DefaultResolver<Unlocked, String, ExprFn>
{
    pub fn new_fns() -> Self
    {
        let mut hashmap: HashMap<String, ExprFn> = HashMap::new();

        hashmap.insert("abs".to_string(), ExprFn(abs));
        hashmap.insert("sqrt".to_string(), ExprFn(sqrt));

        return DefaultResolver {
            vars: hashmap,
            _state: Unlocked,
        };
    }
}

fn abs(x: &[f64]) -> f64
{
    x[0].abs()
}

fn sqrt(x: &[f64]) -> f64
{
    x[0].sqrt()
}
