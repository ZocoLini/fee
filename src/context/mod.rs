use crate::{
    EmptyResolver, ExprFn, IndexedResolver,
    prelude::{Locked, Resolver, ResolverState, Unlocked},
};

/// Container for the resolvers required to evaluate expressions containing variables or functions.
///
/// `Context` holds:
/// - a variable resolver (`V`) that implements `Resolver<f64>`
/// - a function resolver (`F`) that implements `Resolver<ExprFn>`
///
/// This struct is passed to evaluators to provide variable values and function implementations.
pub struct Context<S, V, F>
where
    V: Resolver<S, f64>,
    F: Resolver<S, ExprFn>,
    S: ResolverState,
{
    vars: V,
    fns: F,

    _state: S,
}

impl<V, F> Context<Unlocked, V, F>
where
    V: Resolver<Unlocked, f64>,
    F: Resolver<Unlocked, ExprFn>,
{
    pub fn new(vals: V, funcs: F) -> Self
    {
        Context {
            vars: vals,
            fns: funcs,

            _state: Unlocked,
        }
    }
}

impl<V, F> Context<Locked, V, F>
where
    V: Resolver<Locked, f64>,
    F: Resolver<Locked, ExprFn>,
{
    pub fn new(vals: V, funcs: F) -> Self
    {
        Context {
            vars: vals,
            fns: funcs,

            _state: Locked,
        }
    }
}

impl<S, V, F> Context<S, V, F>
where
    V: Resolver<S, f64>,
    F: Resolver<S, ExprFn>,
    S: ResolverState,
{
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

impl<S, F> Context<S, IndexedResolver<S, f64>, F>
where
    S: ResolverState,
    F: Resolver<S, ExprFn>,
{
    pub(crate) fn get_var_by_index(&self, identifier: usize, index: usize) -> Option<&f64>
    {
        self.vars.get_by_index(identifier, index)
    }
}

impl<S, V> Context<S, V, IndexedResolver<S, ExprFn>>
where
    S: ResolverState,
    V: Resolver<S, f64>,
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

impl Context<Locked, EmptyResolver, EmptyResolver>
{
    pub fn empty() -> Self
    {
        Self::new(EmptyResolver, EmptyResolver)
    }
}
