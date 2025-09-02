use smallvec::{SmallVec, smallvec};

use crate::lexer::InfixExpr;
use crate::token::{InfixToken, Op};
use crate::{Error, EvalError, ExprFn, prelude::*};
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub enum RpnToken<'e>
{
    Num(f64),
    Var(&'e str),
    Fn(&'e str, usize),
    Op(Op),
}

impl<'e> From<InfixToken<'e>> for RpnToken<'e>
{
    fn from(token: InfixToken<'e>) -> Self
    {
        match token {
            InfixToken::Num(num) => RpnToken::Num(num),
            InfixToken::Var(name) => RpnToken::Var(name),
            InfixToken::Fn(name, argc) => RpnToken::Fn(name, argc.len()),
            InfixToken::Op(op) => RpnToken::Op(op),
            _ => unreachable!("logic bug found"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RpnExpr<'e>
{
    tokens: Vec<RpnToken<'e>>,
}

impl<'e> RpnExpr<'e>
{
    fn eval<V: Resolver<f64>, F: Resolver<ExprFn>>(
        &'e self,
        ctx: &Context<V, F>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'e>>
    {
        if self.tokens.len() == 1 {
            if let RpnToken::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                RpnToken::Num(num) => stack.push(*num),
                RpnToken::Var(name) => stack.push(
                    *ctx.get_var(name)
                        .ok_or(Error::EvalError(EvalError::UnknownVar(Cow::Borrowed(name))))?,
                ),
                RpnToken::Fn(name, argc) => {
                    if *argc > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - argc;
                    let args = &stack[start..];
                    let val = match ctx.call_fn(name, args) {
                        Some(value) => value,
                        None => {
                            return Err(Error::EvalError(EvalError::UnknownFn(Cow::Borrowed(
                                name,
                            ))));
                        }
                    };

                    stack.truncate(start);
                    stack.push(val);
                }
                RpnToken::Op(op) => {
                    let start = stack.len() - op.num_operands();
                    let res = op.apply(&stack[start..]);
                    stack.truncate(start);
                    stack.push(res);
                }
            }
        }

        match stack.pop() {
            Some(result) if stack.is_empty() => Ok(result),
            _ => Err(Error::EvalError(EvalError::MalformedExpression)),
        }
    }
}

impl<'e> TryFrom<InfixExpr<'e>> for RpnExpr<'e>
{
    type Error = crate::Error<'e>;

    // shunting yard algorithm
    fn try_from(expr: InfixExpr<'e>) -> Result<Self, Self::Error>
    {
        let mut f64_cache: SmallVec<[f64; 4]> = smallvec![];
        let mut output: Vec<RpnToken> = Vec::with_capacity(expr.len());
        let mut ops: Vec<InfixToken> = Vec::new();

        for tok in expr.into_iter() {
            match tok {
                InfixToken::Num(num) => {
                    output.push(RpnToken::Num(num));
                    f64_cache.push(num);
                }
                InfixToken::Var(var) => {
                    output.push(RpnToken::Var(var));
                    f64_cache.clear();
                }
                InfixToken::Op(op) => {
                    while let Some(InfixToken::Op(top)) = ops.last() {
                        let prec = op.precedence();
                        let top_prec = top.precedence();
                        let should_pop =
                            top_prec > prec || (!op.is_right_associative() && top_prec == prec);

                        if should_pop {
                            if let Some(InfixToken::Op(op)) = ops.pop() {
                                pre_evaluate(&mut output, &mut f64_cache, op);
                            }
                        } else {
                            break;
                        }
                    }
                    ops.push(InfixToken::Op(op));
                }
                InfixToken::LParen => ops.push(tok),
                InfixToken::RParen => {
                    while let Some(top) = ops.pop() {
                        match top {
                            InfixToken::LParen => break,
                            InfixToken::Op(op) => pre_evaluate(&mut output, &mut f64_cache, op),
                            _ => {
                                output.push(top.into());
                                f64_cache.clear();
                            }
                        }
                    }
                }
                InfixToken::Fn(name, args) => {
                    let fun_call_token = RpnToken::Fn(name, args.len());

                    for arg_tokens in args {
                        let rpn_arg: RpnExpr = arg_tokens.try_into()?;
                        output.extend(rpn_arg.tokens);
                    }

                    output.push(fun_call_token);
                    f64_cache.clear();
                }
            }
        }

        while let Some(top) = ops.pop() {
            if let InfixToken::Op(op) = top {
                pre_evaluate(&mut output, &mut f64_cache, op);
            } else {
                output.push(top.into());
                f64_cache.clear(); // TODO: After this else the if doesn't need to be call because no operator uses 0 elements
            }
        }

        return Ok(RpnExpr { tokens: output });

        fn pre_evaluate<'t>(
            output: &mut Vec<RpnToken<'t>>,
            f64_cache: &mut SmallVec<[f64; 4]>,
            op: Op,
        )
        {
            let n_operands = op.num_operands();

            if f64_cache.len() >= n_operands {
                let output_len = output.len();
                let f64_cache_len = f64_cache.len();

                let start = f64_cache_len - n_operands;
                let num = op.apply(&f64_cache[start..]);
                let token = RpnToken::Num(num);

                output.truncate(output_len - n_operands + 1);
                output[output_len - n_operands] = token;

                f64_cache.truncate(f64_cache_len - n_operands + 1);
                f64_cache[f64_cache_len - n_operands] = num;
            } else {
                output.push(RpnToken::Op(op));
                f64_cache.clear();
            }
        }
    }
}

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
/// The RPN evaluator uses an internal stack to evaluate Reverse Polish Notation expressions.  
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
pub struct RpnEvaluator<'e>
{
    rpn: RpnExpr<'e>,
}

impl<'e> Evaluator<'e> for RpnEvaluator<'e>
{
    fn new(expr: &'e str) -> Result<Self, crate::Error<'e>>
    {
        let infix_expr = InfixExpr::try_from(expr)?;
        let rpn_expr = RpnExpr::try_from(infix_expr)?;

        Ok(RpnEvaluator { rpn: rpn_expr })
    }

    fn eval<V: Resolver<f64>, F: Resolver<ExprFn>>(
        &'e self,
        ctx: &Context<V, F>,
    ) -> Result<f64, Error<'e>>
    {
        let mut stack = Vec::with_capacity(self.rpn.tokens.len() / 2);
        self.eval_with_stack(ctx, &mut stack)
    }
}

impl<'e> RpnEvaluator<'e>
{
    pub fn eval_with_stack<V: Resolver<f64>, F: Resolver<ExprFn>>(
        &'e self,
        ctx: &Context<V, F>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'e>>
    {
        self.rpn.eval(ctx, stack)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_infix_to_rpn()
    {
        // 2 - (4 + (p19 - 2) * (p19 + 2))
        let infix_expr = InfixExpr::new(vec![
            InfixToken::Num(2.0),
            InfixToken::Op(Op::Sub),
            InfixToken::LParen,
            InfixToken::Num(4.0),
            InfixToken::Op(Op::Add),
            InfixToken::LParen,
            InfixToken::Var("p19"),
            InfixToken::Op(Op::Sub),
            InfixToken::Num(2.0),
            InfixToken::RParen,
            InfixToken::Op(Op::Mul),
            InfixToken::LParen,
            InfixToken::Var("p19"),
            InfixToken::Op(Op::Add),
            InfixToken::Num(2.0),
            InfixToken::RParen,
            InfixToken::RParen,
        ]);

        let rpn_expr: RpnExpr = infix_expr.try_into().unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                RpnToken::Num(2.0),
                RpnToken::Num(4.0),
                RpnToken::Var("p19"),
                RpnToken::Num(2.0),
                RpnToken::Op(Op::Sub),
                RpnToken::Var("p19"),
                RpnToken::Num(2.0),
                RpnToken::Op(Op::Add),
                RpnToken::Op(Op::Mul),
                RpnToken::Op(Op::Add),
                RpnToken::Op(Op::Sub)
            ]
        );

        //abs((2 + 3) * 4, sqrt(5))
        let infix_expr = InfixExpr::new(vec![InfixToken::Fn(
            "abs",
            vec![
                InfixExpr::new(vec![
                    InfixToken::LParen,
                    InfixToken::Num(2.0),
                    InfixToken::Op(Op::Add),
                    InfixToken::Num(3.0),
                    InfixToken::RParen,
                    InfixToken::Op(Op::Mul),
                    InfixToken::Num(4.0),
                ]),
                InfixExpr::new(vec![InfixToken::Fn(
                    "sqrt",
                    vec![InfixExpr::new(vec![InfixToken::Num(5.0)])],
                )]),
            ],
        )]);

        let rpn_expr: RpnExpr = infix_expr.try_into().unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                RpnToken::Num(20.0),
                RpnToken::Num(5.0),
                RpnToken::Fn("sqrt", 1),
                RpnToken::Fn("abs", 2),
            ]
        );
    }

    #[test]
    fn test_str_to_rpn()
    {
        let expr = "(2 * 21) + 3 + -35 - ((5 * 80) + 5) + 10 + -p0";
        let infix_expr: InfixExpr = InfixExpr::try_from(expr).unwrap();
        let rpn_expr: RpnExpr = infix_expr.try_into().unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                RpnToken::Num(-385.0),
                RpnToken::Var("p0"),
                RpnToken::Op(Op::Neg),
                RpnToken::Op(Op::Add),
            ]
        );

        let expr = "-y1 * (p2 - p3*y0)";
        let infix_expr: InfixExpr = InfixExpr::try_from(expr).unwrap();
        let rpn_expr: RpnExpr = infix_expr.try_into().unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                RpnToken::Var("y1"),
                RpnToken::Op(Op::Neg),
                RpnToken::Var("p2"),
                RpnToken::Var("p3"),
                RpnToken::Var("y0"),
                RpnToken::Op(Op::Mul),
                RpnToken::Op(Op::Sub),
                RpnToken::Op(Op::Mul),
            ]
        );
    }
}
