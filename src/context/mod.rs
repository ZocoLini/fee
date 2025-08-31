use crate::{ExprFn, prelude::Resolver};

pub struct Context<V: Resolver<f64>, F: Resolver<ExprFn>>
{
    vars: V,
    fns: F,
}

impl<V: Resolver<f64>, F: Resolver<ExprFn>> Context<V, F>
{
    pub fn new(vals: V, funcs: F) -> Self
    {
        Context {
            vars: vals,
            fns: funcs,
        }
    }

    pub fn get_var(&self, name: &str) -> Option<&f64>
    {
        self.vars.resolve(name)
    }

    pub fn call_fn(&self, name: &str, args: &[f64]) -> Option<f64>
    {
        Some(self.fns.resolve(name)?(args))
    }

    pub fn vars_mut(&mut self) -> &mut V
    {
        &mut self.vars
    }

    pub fn fns_mut(&mut self) -> &mut F
    {
        &mut self.fns
    }
}
