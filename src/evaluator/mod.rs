use std::{
    borrow::Cow,
    ops::Deref,
    sync::{Arc, RwLock},
};

use crate::{Error, ParseError, prelude::*};

mod rpn;
pub use rpn::RPNEvaluator;

pub trait Evaluator<'e, 'c, V: VarResolver, F: FnResolver>: Sized
{
    fn new(expr: &'e str, ctx: &'c mut Context<V, F>) -> Result<Self, crate::Error<'e>>;
    fn eval(&self) -> f64;
    fn context_mut(&mut self) -> &mut Context<V, F>;
}
