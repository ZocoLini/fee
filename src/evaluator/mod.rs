use std::{
    borrow::Cow,
    ops::Deref,
    sync::{Arc, RwLock},
};

use crate::{Error, ParseError, prelude::*};

pub(crate) mod rpn;
pub use rpn::RPNEvaluator;

pub trait Evaluator<'e>: Sized
{
    fn new(expr: &'e str) -> Result<Self, crate::Error<'e>>;
    fn eval(&'e self, ctx: &impl Context) -> Result<f64, Error<'e>>;
}
