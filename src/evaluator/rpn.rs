use crate::{Error, ExprFn, RpnExpr, prelude::*};

/// Evaluator that internally uses a stack to evaluate Reverse Polish Notation expressions.
///
/// # Parsing
/// The new() method receives a string expression and first tokenizes it into a vector of Infix Tokens.
/// If the expression was parsed successfully, it is converted into a Reverse Polish Notation expression.
/// During the conversion to RPN, the operations that can be pre-evaluated are evaluated immediately.
/// For example, the expression "2 + 3 * 4" would be converted to "2 3 4 * +" without pre-evaluation, but
/// because we pre-evaluate when possible, the expression is converted to "14" reducing evaluation time
/// and improving performance when the RpnEvaluator is used more than once.
///
/// # Evaluation
/// The RPN evaluator uses an stack to evaluate Reverse Polish Notation expressions.  
/// By default, [`RpnEvaluator::eval()`] creates a new temporary stack on each call, which can add overhead.  
/// If the evaluator is called multiple times, consider reusing a preallocated stack via  
/// [`RpnEvaluator::eval_with_stack()`] to improve performance.
///
/// ```rust
/// use fee::prelude::*;
/// use fee::RpnEvaluator;
///
/// let expr = "2 + 3 * 4";
/// let evaluator = RpnEvaluator::new(expr).unwrap();
/// let mut stack = Vec::with_capacity(3);
/// let result = evaluator.eval_with_stack(&Context::empty(), &mut stack).unwrap();
/// assert_eq!(result, 14.0);
/// ```
///
/// The speed when resolving vars or functions depends on the Provider chosen when creating the context using
/// during the evaluation process. If no context is provided, there will not be such thing as time
/// resolving variables or functions.
///
/// # Support
/// This evaluator supports f64 operations and the operators +. -. *. /, ^.
///
/// # Errors
/// Will return an fee::ParseError if parsing fails and a fee::EvalError if evaluation fails.
///
/// # Examples
/// ```
/// use fee::prelude::*;
/// use fee::RpnEvaluator;
///
/// let expr = "2 + 3 * 4";
/// let evaluator = RpnEvaluator::new(expr).unwrap();
/// let result = evaluator.eval_without_context().unwrap();
/// assert_eq!(result, 14.0);
/// ```
pub struct RpnEvaluator;

impl RpnEvaluator
{
    pub fn new() -> Self
    {
        RpnEvaluator
    }

    pub fn eval<'e, V: Resolver<f64>, F: Resolver<ExprFn>>(
        &self,
        expr: &'e RpnExpr,
        ctx: &Context<V, F>,
    ) -> Result<f64, Error<'e>>
    {
        let mut stack = Vec::with_capacity(expr.len() / 2);
        self.eval_with_stack(expr, ctx, &mut stack)
    }

    pub fn eval_with_stack<'e, V: Resolver<f64>, F: Resolver<ExprFn>>(
        &self,
        expr: &'e RpnExpr,
        ctx: &Context<V, F>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'e>>
    {
        expr.eval(ctx, stack)
    }

    pub fn eval_without_context<'e>(&self, expr: &'e RpnExpr) -> Result<f64, Error<'e>>
    {
        self.eval(expr, &Context::empty())
    }
}
