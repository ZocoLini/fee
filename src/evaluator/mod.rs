use crate::{Error, ExprFn, prelude::*};

pub(crate) mod rpn;
pub use rpn::RpnEvaluator;

pub trait Evaluator<'e>: Sized
{
    fn new(expr: &'e str) -> Result<Self, crate::Error<'e>>;
    fn eval<V: Resolver<f64>, F: Resolver<ExprFn>>(
        &'e self,
        ctx: &Context<V, F>,
    ) -> Result<f64, Error<'e>>;
}
