use std::marker::PhantomData;

use crate::Ptr;
use crate::resolver::Locked;
use crate::resolver::LockedResolver;
use crate::resolver::ResolverState;
use crate::resolver::Unlocked;
use crate::resolver::UnlockedResolver;
use crate::{EmptyResolver, ExprFn, IndexedResolver, prelude::*};

/// Container for the resolvers required to compile and evaluate
/// expressions.
///
/// `Context` holds:
/// - a variable resolver (`V`) that implements `Resolver<f64>`
/// - a function resolver (`F`) that implements `Resolver<ExprFn>`
///
/// This struct is passed to evaluators to provide variable values and function
/// implementations.
///
/// # Locking
/// It is possible to 'lock' the resolvers held by the context to prevent
/// them from being reallocated in memory. This can be done by calling the
/// [`Context::lock`] method, allowing a better performance when evaluating
/// expressions taking advantage of the use of pointers to the values held by
/// the resolvers.
///
/// If the Context is locked, the user can obtain a [`Ptr`] to a value held
/// by one of the resolvers and modify it directly.
pub struct Context<S, V, F, LV, LF>
where
    S: ResolverState,
{
    vars: V,
    fns: F,

    _state: S,
    _locked_vars: PhantomData<LV>,
    _locked_fns: PhantomData<LF>,
}

impl<V, F, LV, LF> Context<Unlocked, V, F, LV, LF>
where
    V: UnlockedResolver<f64, LV>,
    F: UnlockedResolver<ExprFn, LF>,
    LV: LockedResolver<f64>,
    LF: LockedResolver<ExprFn>,
{
    pub fn new(vals: V, funcs: F) -> Self
    {
        Context {
            vars: vals,
            fns: funcs,

            _state: Unlocked,
            _locked_vars: PhantomData,
            _locked_fns: PhantomData,
        }
    }

    pub fn lock(self) -> Context<Locked, LV, LF, LV, LF>
    {
        Context {
            vars: self.vars.lock(),
            fns: self.fns.lock(),

            _state: Locked,
            _locked_vars: PhantomData,
            _locked_fns: PhantomData,
        }
    }
}

impl<S, V, F, LV, LF> Context<S, V, F, LV, LF>
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

impl<'a, S, V, F, LV, LF> Context<S, V, F, LV, LF>
where
    V: LockedResolver<f64>,
    F: LockedResolver<ExprFn>,
    S: ResolverState,
{
    pub fn get_var_ptr(&'a self, name: &str) -> Option<Ptr<'a, f64>>
    {
        self.vars.get_ptr(name)
    }

    pub fn get_fn_ptr(&'a self, name: &str) -> Option<Ptr<'a, ExprFn>>
    {
        self.fns.get_ptr(name)
    }
}

impl<S, F, LF> Context<S, IndexedResolver<S, f64>, F, IndexedResolver<Locked, f64>, LF>
where
    S: ResolverState,
    F: Resolver<S, ExprFn>,
{
    pub(crate) fn get_var_by_index(&self, identifier: usize, index: usize) -> Option<&f64>
    {
        self.vars.get(identifier, index)
    }
}

impl<S, V, LV> Context<S, V, IndexedResolver<S, ExprFn>, LV, IndexedResolver<Locked, ExprFn>>
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
        Some(self.fns.get(identifier, index)?(args))
    }
}

impl
    Context<
        Unlocked,
        EmptyResolver<Unlocked>,
        EmptyResolver<Unlocked>,
        EmptyResolver<Locked>,
        EmptyResolver<Locked>,
    >
{
    pub fn empty() -> Self
    {
        Context::new(EmptyResolver::new(), EmptyResolver::new())
    }
}
