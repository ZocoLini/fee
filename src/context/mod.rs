use crate::{
    EmptyResolver, ExprFn, IndexedResolver,
    prelude::{Locked, LockedResolver, Resolver, ResolverState, Unlocked, UnlockedResolver},
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
    V: Resolver<Unlocked, f64> + UnlockedResolver<f64>,
    F: Resolver<Unlocked, ExprFn> + UnlockedResolver<ExprFn>,
{
    pub fn new_unlocked(vals: V, funcs: F) -> Self
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
    V: Resolver<Locked, f64> + LockedResolver<f64>,
    F: Resolver<Locked, ExprFn> + LockedResolver<ExprFn>,
{
    pub fn new_locked(vals: V, funcs: F) -> Self
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

    pub(crate) fn get_fn(&self, name: &str) -> Option<&ExprFn>
    {
        self.fns.resolve(name)
    }

    pub fn vars(&self) -> &V
    {
        &self.vars
    }

    pub fn fns(&self) -> &F
    {
        &self.fns
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
        Self::new_locked(EmptyResolver, EmptyResolver)
    }
}
