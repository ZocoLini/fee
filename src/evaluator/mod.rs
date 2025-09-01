use crate::{EmptyResolver};
use crate::{Error, ExprFn, prelude::*};

pub(crate) mod rpn;
pub use rpn::RpnEvaluator;

/// Trait that all evaluators must implement.
///
/// This trait ensures that all evaluators know how to be built from a string expression and
/// how to evaluate expressions, with or without a context.
///
/// # Example
/// ```
/// use std::borrow::Cow;
/// use fee::{ParseError, Error};
/// use fee::prelude::*;
/// 
/// 
/// struct CustomEvaluator
/// {
///     value: f64,
/// }
/// 
/// impl<'e> Evaluator<'e> for CustomEvaluator
/// {
///     fn new(expr: &'e str) -> Result<Self, Error<'e>>
///     {
///         Ok(CustomEvaluator {
///             value: expr.parse().map_err(|_| {
///                 Error::ParseError(ParseError::InvalidNumber(Cow::Borrowed(expr), 0))
///             })?,
///         })
///     }
/// 
///     fn eval<V: Resolver<f64>, F: Resolver<ExprFn>>(
///         &'e self,
///         _ctx: &Context<V, F>,
///     ) -> Result<f64, Error<'e>>
///     {
///         Ok(self.value)
///     }
/// }
/// 
/// let eval = CustomEvaluator::new("42").unwrap();
/// assert_eq!(eval.eval_without_context().unwrap(), 42.0);
/// ```
pub trait Evaluator<'e>: Sized
{
    fn new(expr: &'e str) -> Result<Self, crate::Error<'e>>;

    fn eval<V: Resolver<f64>, F: Resolver<ExprFn>>(
        &'e self,
        ctx: &Context<V, F>,
    ) -> Result<f64, Error<'e>>;

    fn eval_without_context(&'e self) -> Result<f64, Error<'e>>
    {
        self.eval(&Context::new(EmptyResolver::new(), EmptyResolver::new()))
    }
}
