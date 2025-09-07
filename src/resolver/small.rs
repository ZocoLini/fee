use crate::{
    prelude::*,
    resolver::{Locked, LockedResolver, ResolverState, Unlocked, UnlockedResolver},
};

const CACHE_SIZE: usize = 10; // 30 is the 'limit'

/// A fast resolver with a small, fixed-size cache.
///
/// This resolver imposes **no restrictions** on the variable names you can use and is
/// generally faster than the default resolver due to its compact cache and improved
/// cache locality.
///
/// # Advantages
/// - No restrictions on variable or function names.
/// - The cache is small and compact, reducing memory usage and improving cache locality.
///
/// # Disadvantages
/// - Limited cache size. Not recommended more than 10 items.
///
/// # Performance
/// - **Up to 10 variables:** Maximum performance is achieved thanks to cache locality.
/// - **10 to 30 variables:** Performance remains good, though cache misses may occur more frequently.
/// - **More than 30 variables:** Not recommended; beyond this point, the default resolver is typically faster.
///
/// # Locking
/// This resolver can be locked to prevent further modifications to the cache.
/// When locked a method .get_var_pointer() is available to retrieve a pointer to a variable,
/// allowing modifications to the variable's value without having to search for it
/// in the cache.
///
/// ```rust
/// use fee::SmallResolver;
/// use fee::prelude::*;
///
/// let mut var_resolver = SmallResolver::new();
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
pub struct SmallResolver<S, K, V>
where
    S: ResolverState,
    K: AsRef<str> + Eq,
{
    cache: Vec<(K, V)>,
    _state: S,
}

impl<K, V> LockedResolver<V> for SmallResolver<Locked, K, V> where K: AsRef<str> + Eq {}
impl<K, V> UnlockedResolver<V, SmallResolver<Locked, K, V>> for SmallResolver<Unlocked, K, V>
where
    K: AsRef<str> + Eq,
{
    fn lock(self) -> SmallResolver<Locked, K, V>
    {
        SmallResolver {
            cache: self.cache,
            _state: Locked,
        }
    }
}

impl<S, K, V> Resolver<S, V> for SmallResolver<S, K, V>
where
    S: ResolverState,
    K: AsRef<str> + Eq,
{
    fn resolve(&self, name: &str) -> Option<&V>
    {
        for (key, value) in &self.cache {
            if key.as_ref() == name {
                return Some(value);
            }
        }
        None
    }
}

impl<K, V> SmallResolver<Unlocked, K, V>
where
    K: AsRef<str> + Eq,
{
    pub fn new() -> Self
    {
        Self {
            cache: Vec::with_capacity(CACHE_SIZE),
            _state: Unlocked,
        }
    }

    pub fn insert(&mut self, name: K, value: V)
    {
        for (i, (key, _)) in self.cache.iter().enumerate() {
            if *key == name {
                self.cache[i].1 = value;
                return;
            }
        }

        self.cache.push((name, value));
    }
}
