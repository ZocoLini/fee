use smallvec::{SmallVec, smallvec};

use crate::lexer::InfixExpr;
use crate::token::{InfixToken, Op};
use crate::{Error, EvalError, ExprFn, IndexedResolver, parsing, prelude::*};
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub enum IRpnToken
{
    Num(f64),
    Var(usize, usize),
    Fn(usize, usize, usize),
    Op(Op),
}

impl<'e> From<InfixToken<'e>> for IRpnToken
{
    fn from(token: InfixToken<'e>) -> Self
    {
        match token {
            InfixToken::Num(num) => IRpnToken::Num(num),
            InfixToken::Var(name) => {
                let name_bytes = name.as_bytes();

                let letter = name_bytes[0] - b'a';
                let idx = parsing::parse_usize(&name_bytes[1..]);

                IRpnToken::Var(letter as usize, idx)
            },
            InfixToken::Fn(name, args) => {
                let name_bytes = name.as_bytes();

                let letter = name_bytes[0] - b'a';
                let idx = parsing::parse_usize(&name_bytes[1..]);

                IRpnToken::Fn(letter as usize, idx, args.len())
            },
            InfixToken::Op(op) => IRpnToken::Op(op),
            _ => unreachable!("logic bug found"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct IRpnExpr
{
    tokens: Vec<IRpnToken>,
}

impl IRpnExpr
{
    fn eval(
        &self,
        ctx: &Context<IndexedResolver<f64>, IndexedResolver<ExprFn>>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'_>>
    {
        if self.tokens.len() == 1 {
            if let IRpnToken::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                IRpnToken::Num(num) => stack.push(*num),
                IRpnToken::Var(id, idx) => {
                    stack.push(*ctx.get_var_by_index(*id, *idx).ok_or_else(|| {
                        Error::EvalError(EvalError::UnknownVar(Cow::Owned(format!(
                            "{}{}",
                            (*id as u8 + b'a') as char,
                            idx
                        ))))
                    })?)
                }
                IRpnToken::Fn(id, idx, argc) => {
                    if *argc > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - argc;
                    let args = &stack[start..];
                    let val = match ctx.call_fn_by_index(*id, *idx, args) {
                        Some(value) => value,
                        None => {
                            return Err(Error::EvalError(EvalError::UnknownFn(Cow::Owned(
                                format!("{}{}", (*id as u8 + b'a') as char, idx),
                            ))));
                        }
                    };

                    stack.truncate(start);
                    stack.push(val);
                }
                IRpnToken::Op(op) => {
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

impl<'e> TryFrom<InfixExpr<'e>> for IRpnExpr
{
    type Error = crate::Error<'e>;

    // shunting yard algorithm
    fn try_from(expr: InfixExpr<'e>) -> Result<Self, Self::Error>
    {
        let mut f64_cache: SmallVec<[f64; 4]> = smallvec![];
        let mut output: Vec<IRpnToken> = Vec::with_capacity(expr.len());
        let mut ops: Vec<InfixToken> = Vec::new();

        for tok in expr.into_iter() {
            match tok {
                InfixToken::Num(num) => {
                    output.push(IRpnToken::Num(num));
                    f64_cache.push(num);
                }
                InfixToken::Var(name) => {
                    let name_bytes = name.as_bytes();
    
                    let letter = name_bytes[0] - b'a';
                    let idx = parsing::parse_usize(&name_bytes[1..]);
    
                    output.push(IRpnToken::Var(letter as usize, idx));
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
                    let name_bytes = name.as_bytes();
    
                    let letter = name_bytes[0] - b'a';
                    let idx = parsing::parse_usize(&name_bytes[1..]);
                    
                    let token = IRpnToken::Fn(letter as usize, idx, args.len());
                    
                    for arg_tokens in args {
                        let rpn_arg: IRpnExpr = arg_tokens.try_into()?;
                        output.extend(rpn_arg.tokens);
                    }
    
                    output.push(token);
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

        return Ok(IRpnExpr { tokens: output });

        fn pre_evaluate(
            output: &mut Vec<IRpnToken>,
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
                let token = IRpnToken::Num(num);

                output.truncate(output_len - n_operands + 1);
                output[output_len - n_operands] = token;

                f64_cache.truncate(f64_cache_len - n_operands + 1);
                f64_cache[f64_cache_len - n_operands] = num;
            } else {
                output.push(IRpnToken::Op(op));
                f64_cache.clear();
            }
        }
    }
}

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
pub struct IRpnEvaluator
{
    rpn: IRpnExpr,
}

impl IRpnEvaluator
{
    pub fn new(expr: &str) -> Result<Self, crate::Error<'_>>
    {
        let infix_expr = InfixExpr::try_from(expr)?;
        let rpn_expr = IRpnExpr::try_from(infix_expr)?;

        Ok(IRpnEvaluator { rpn: rpn_expr })
    }

    pub fn eval(
        &self,
        ctx: &Context<IndexedResolver<f64>, IndexedResolver<ExprFn>>,
    ) -> Result<f64, Error<'_>>
    {
        let mut stack = Vec::with_capacity(self.rpn.tokens.len() / 2);
        self.eval_with_stack(ctx, &mut stack)
    }
    
    pub fn eval_with_stack(
        &self,
        ctx: &Context<IndexedResolver<f64>, IndexedResolver<ExprFn>>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'_>>
    {
        self.rpn.eval(ctx, stack)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_infix_to_irpn()
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

        let rpn_expr: IRpnExpr = infix_expr.try_into().unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                IRpnToken::Num(2.0),
                IRpnToken::Num(4.0),
                IRpnToken::Var((b'p' - b'a') as usize, 19),
                IRpnToken::Num(2.0),
                IRpnToken::Op(Op::Sub),
                IRpnToken::Var((b'p' - b'a') as usize, 19),
                IRpnToken::Num(2.0),
                IRpnToken::Op(Op::Add),
                IRpnToken::Op(Op::Mul),
                IRpnToken::Op(Op::Add),
                IRpnToken::Op(Op::Sub)
            ]
        );

        //abs((2 + 3) * 4, sqrt(5))
        let infix_expr = InfixExpr::new(vec![InfixToken::Fn(
            "f0",
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
                    "f1",
                    vec![InfixExpr::new(vec![InfixToken::Num(5.0)])],
                )]),
            ],
        )]);

        let rpn_expr: IRpnExpr = infix_expr.try_into().unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                IRpnToken::Num(20.0),
                IRpnToken::Num(5.0),
                IRpnToken::Fn((b'f' - b'a') as usize, 1, 1),
                IRpnToken::Fn((b'f' - b'a') as usize, 0, 2),
            ]
        );
    }

    #[test]
    fn test_str_to_irpn()
    {
        let expr = "(2 * 21) + 3 + -35 - ((5 * 80) + 5) + 10 + -p0";
        let infix_expr: InfixExpr = InfixExpr::try_from(expr).unwrap();
        let rpn_expr: IRpnExpr = infix_expr.try_into().unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                IRpnToken::Num(-385.0),
                IRpnToken::Var((b'p' - b'a') as usize, 0),
                IRpnToken::Op(Op::Neg),
                IRpnToken::Op(Op::Add),
            ]
        );

        let expr = "-y1 * (p2 - p3*y0)";
        let infix_expr: InfixExpr = InfixExpr::try_from(expr).unwrap();
        let rpn_expr: IRpnExpr = infix_expr.try_into().unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                IRpnToken::Var((b'y' - b'a') as usize, 1),
                IRpnToken::Op(Op::Neg),
                IRpnToken::Var((b'p' - b'a') as usize, 2),
                IRpnToken::Var((b'p' - b'a') as usize, 3),
                IRpnToken::Var((b'y' - b'a') as usize, 0),
                IRpnToken::Op(Op::Mul),
                IRpnToken::Op(Op::Sub),
                IRpnToken::Op(Op::Mul),
            ]
        );
    }
}
