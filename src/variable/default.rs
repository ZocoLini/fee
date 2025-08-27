use std::collections::HashMap;

use crate::prelude::*;

pub struct DefaultVarResolver<State>
{
    vars: HashMap<String, f64>,
    state: State,
}

impl<State> VarResolver for DefaultVarResolver<State>
{
    fn get(&self, name: &str) -> Option<f64>
    {
        self.vars.get(name).copied()
    }
}

impl DefaultVarResolver<Unlocked>
{
    pub fn new() -> Self
    {
        DefaultVarResolver {
            vars: HashMap::new(),
            state: Unlocked,
        }
    }

    pub fn add_var(&mut self, name: String, val: f64)
    {
        self.vars.insert(name, val);
    }

    pub fn lock(self) -> DefaultVarResolver<Locked>
    {
        DefaultVarResolver {
            vars: self.vars,
            state: Locked,
        }
    }
}

impl DefaultVarResolver<Locked>
{
    pub fn get_var_mut(&mut self, name: &str) -> Option<&mut f64>
    {
        self.vars.get_mut(name)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_locked_resolver_get_var_mut_updates_value()
    {
        let mut resolver = DefaultVarResolver::new();
        resolver.add_var("x".to_string(), 10.0);

        let mut resolver = resolver.lock();

        let mut x = resolver.get_var_mut("x").unwrap();
        *x = 20.0;

        assert_eq!(resolver.get("x"), Some(20.0));
    }
}
