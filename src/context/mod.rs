use crate::{EmptyResolver, ExprFn, IndexedResolver, prelude::Resolver};

/// Container for the resolvers required to evaluate expressions containing variables or functions.
///
/// `Context` holds:
/// - a variable resolver (`V`) that implements `Resolver<f64>`
/// - a function resolver (`F`) that implements `Resolver<ExprFn>`
///
/// This struct is passed to evaluators to provide variable values and function implementations.
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

    pub(crate) fn get_var(&self, name: &str) -> Option<&f64>
    {
        self.vars.resolve(name)
    }

    pub(crate) fn call_fn(&self, name: &str, args: &[f64]) -> Option<f64>
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

impl<F: Resolver<ExprFn>> Context<IndexedResolver<f64>, F>
{
    pub(crate) fn get_var_by_index(&self, identifier: usize, index: usize) -> Option<&f64>
    {
        self.vars.get_by_index(identifier, index)
    }
}

impl<V: Resolver<f64>> Context<V, IndexedResolver<ExprFn>>
{
    pub(crate) fn call_fn_by_index(
        &self,
        identifier: usize,
        index: usize,
        args: &[f64],
    ) -> Option<f64>
    {
        Some(self.fns.get_by_index(identifier, index)?(args))
    }
}

impl Context<EmptyResolver, EmptyResolver>
{
    pub fn empty() -> Self
    {
        Context {
            vars: EmptyResolver,
            fns: EmptyResolver,
        }
    }
}
