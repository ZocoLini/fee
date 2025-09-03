use crate::{Error, ExprFn, IRpnExpr, IndexedResolver, context::Context};

/// Evaluator that internally uses a stack to evaluate Reverse Polish Notation expressions
/// optimized for IndexedResolvers.
///
/// # Parsing
/// The new() method receives a string expression and first tokenizes it into a vector of Infix Tokens.
/// If the expression was parsed successfully, it is converted into a Reverse Polish Notation expression.
/// During the conversion to RPN, the operations that can be pre-evaluated are evaluated immediately.
/// For example, the expression "2 + 3 * 4" would be converted to "2 3 4 * +" without pre-evaluation, but
/// because we pre-evaluate when possible, the expression is converted to "14" reducing evaluation time
/// and improving performance when the RpnEvaluator is used more than once.
///
/// This evaluator differs from the standard RpnEvaluator in that it parses variable and function
/// identifiers and indices before evaluation.
///
/// # Evaluation
/// The RPN evaluator uses an stack to evaluate Reverse Polish Notation expressions.
/// By default, [`IRpnEvaluator::eval()`] creates a new temporary stack on each call, which can add overhead.
/// If the evaluator is called multiple times, consider reusing a preallocated stack via
/// [`IRpnEvaluator::eval_with_stack()`] to improve performance.
///
/// # Support
/// This evaluator supports f64 operations and the operators +. -. *. /, ^.
///
/// # Errors
/// Will return an fee::ParseError if parsing fails and a fee::EvalError if evaluation fails.
///
/// # Examples
/// ```rust
/// use fee::prelude::*;
/// use fee::{IRpnEvaluator, IndexedResolver};
///
/// let expr = "2 + 3 * 4";
///
/// let var_resolver = IndexedResolver::new_var_resolver();
/// let fn_resolver = IndexedResolver::new_fn_resolver();
///
/// let evaluator = IRpnEvaluator::new(expr).unwrap();
/// let mut stack = Vec::with_capacity(3);
/// let result = evaluator.eval_with_stack(&Context::new(var_resolver, fn_resolver), &mut stack).unwrap();
/// assert_eq!(result, 14.0);
/// ```
pub struct IRpnEvaluator;

impl IRpnEvaluator
{
    pub fn new() -> Self
    {
        IRpnEvaluator
    }

    pub fn eval<'e>(
        &self,
        expr: &'e IRpnExpr,
        ctx: &Context<IndexedResolver<f64>, IndexedResolver<ExprFn>>,
    ) -> Result<f64, Error<'e>>
    {
        let mut stack = Vec::with_capacity(expr.len() / 2);
        self.eval_with_stack(expr, ctx, &mut stack)
    }

    pub fn eval_with_stack<'e>(
        &self,
        expr: &'e IRpnExpr,
        ctx: &Context<IndexedResolver<f64>, IndexedResolver<ExprFn>>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'e>>
    {
        expr.eval(ctx, stack)
    }
}
