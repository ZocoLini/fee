use std::collections::HashMap;

use super::Resolver;
use crate::{ExprFn, prelude::*};

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
    pub fn get_var_mut(&mut self, name: &str) -> Option<&mut f64>
    {
        self.vars.get_mut(name)
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

        let x = resolver.get_var_mut("x").unwrap();
        *x = 20.0;

        assert_eq!(resolver.resolve("x"), Some(&20.0));
    }
}
