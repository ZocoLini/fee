use crate::prelude::{FnResolver, VarResolver};

pub trait Context 
{
    fn get_var(&self, name: &str) -> Option<&f64>;
    fn call_fn(&self, name: &str, args: &[f64]) -> Option<f64>;    
}

pub struct DefaultContext<V: VarResolver, F: FnResolver>
{
    vars: V,
    fns: F,
}

impl<V: VarResolver, F: FnResolver> Context for DefaultContext<V, F>
{
    fn get_var(&self, name: &str) -> Option<&f64>
    {
        self.vars.get_var(name)
    }

    fn call_fn(&self, name: &str, args: &[f64]) -> Option<f64>
    {
        self.fns.call_fn(name, args)
    }
}

impl<V: VarResolver, F: FnResolver> DefaultContext<V, F>
{
    pub fn new(vals: V, funcs: F) -> Self
    {
        DefaultContext {
            vars: vals,
            fns: funcs,
        }
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
