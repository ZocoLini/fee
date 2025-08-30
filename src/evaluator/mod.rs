use std::{
    borrow::Cow,
    ops::Deref,
    sync::{Arc, RwLock},
};

use crate::{Error, ParseError, prelude::*};

mod rpn;
pub use rpn::RPNEvaluator;

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

    pub fn vars_mut(&mut self) -> &mut V
    {
        &mut self.vars
    }

    pub fn fns_mut(&mut self) -> &mut F
    {
        &mut self.fns
    }
}

pub trait Evaluator<'e, 'c, V: VarResolver, F: FnResolver>: Sized
{
    fn new(expr: &'e str, ctx: &'c mut Context<V, F>) -> Result<Self, crate::Error<'e>>;
    fn eval(&self) -> f64;
    fn context_mut(&mut self) -> &mut Context<V, F>;
}
