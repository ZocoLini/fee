use crate::prelude::{FnResolver, VarResolver};

pub struct Context<V: VarResolver, F: FnResolver>
{
    vars: V,
    fns: F,
}

impl<V: VarResolver, F: FnResolver> Context<V, F>
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
        self.vars.get_var(name)
    }

    pub fn call_fn(&self, name: &str, args: &[f64]) -> Option<f64>
    {
        self.fns.call_fn(name, args)
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
