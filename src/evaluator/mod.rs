mod rpn;

use crate::{Error, prelude::*};

pub use rpn::RPNEvaluator;

pub struct Context<V: VarResolver, F: FnResolver>
{
    vals: V,
    funcs: F,
}

impl<V: VarResolver, F: FnResolver> Context<V, F>
{
    pub fn new(vals: V, funcs: F) -> Self
    {
        Context { vals, funcs }
    }
}

pub trait Evaluator: Sized
{
    fn eval(&self) -> f64;
}
