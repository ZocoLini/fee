use crate::lexer::{Infix, InfixExpr};
use crate::token::{InfixToken, Op};
use crate::{Error, EvalError, ParseError, prelude::*};
use std::cell::UnsafeCell;
use std::panic;
use std::sync::{Arc, Mutex, RwLock};
use std::{borrow::Cow, ops::Deref};

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
    fn eval(&'e self, ctx: &impl Context, stack: &mut Vec<f64>) -> Result<f64, Error<'e>>
    {
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
                    let b = match stack.pop() {
                        Some(value) => value,
                        None => return Err(Error::EvalError(EvalError::RPNStackUnderflow)),
                    };
                    let a = match stack.pop() {
                        Some(value) => value,
                        None => return Err(Error::EvalError(EvalError::RPNStackUnderflow)),
                    };
                    let res = op.apply(a, b);
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
        let mut output: Vec<RpnToken> = Vec::with_capacity(expr.len());
        let mut ops: Vec<InfixToken> = Vec::new();

        let mut num_count = 0;

        for tok in expr.into_iter() {
            match tok {
                InfixToken::Num(num) => {
                    output.push(RpnToken::Num(num));
                    num_count += 1;
                }
                InfixToken::Var(var) => {
                    output.push(RpnToken::Var(var));
                    num_count = 0;
                }
                InfixToken::Op(op) => {
                    while let Some(InfixToken::Op(top)) = ops.last() {
                        let should_pop = if op.is_right_associative() {
                            op.precedence() < top.precedence()
                        } else {
                            op.precedence() <= top.precedence()
                        };

                        if should_pop {
                            if let Some(InfixToken::Op(op)) = ops.pop() {
                                pre_evaluate(&mut output, op, &mut num_count);
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
                            InfixToken::Op(op) => pre_evaluate(&mut output, op, &mut num_count),
                            _ => {
                                output.push(top.into());
                                num_count = 0;
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
                    num_count = 0;
                }
            }
        }

        while let Some(top) = ops.pop() {
            if let InfixToken::Op(op) = top {
                pre_evaluate(&mut output, op, &mut num_count);
            } else {
                output.push(top.into());
                num_count = 0;
            }
        }

        return Ok(RpnExpr { tokens: output });

        #[inline(always)]
        fn pre_evaluate<'t>(output: &mut Vec<RpnToken<'t>>, op: Op, num_count: &mut usize)
        {
            // Each operand may have a different number of arguments
            if *num_count >= 2 {
                let b = if let Some(RpnToken::Num(value)) = output.pop() {
                    value
                } else {
                    unreachable!("expected a number");
                };
                let a = if let Some(RpnToken::Num(value)) = output.pop() {
                    value
                } else {
                    unreachable!("expected a number");
                };
                output.push(RpnToken::Num(op.apply(a, b)));
                *num_count -= 1;
            } else {
                output.push(RpnToken::Op(op));
                *num_count = 0;
            }
        }
    }
}

pub struct RPNEvaluator<'e>
{
    rpn: RpnExpr<'e>,

    stack: Mutex<Vec<f64>>,
}

impl<'e> Evaluator<'e> for RPNEvaluator<'e>
{
    fn new(expr: &'e str) -> Result<Self, crate::Error<'e>>
    {
        let infix_expr = InfixExpr::try_from(expr)?;
        let rpn_expr = RpnExpr::try_from(infix_expr)?;

        let stack = Mutex::new(Vec::with_capacity(rpn_expr.tokens.len() / 2));

        Ok(RPNEvaluator {
            rpn: rpn_expr,
            stack,
        })
    }

    fn eval(&'e self, ctx: &impl Context) -> Result<f64, Error<'e>>
    {
        match self.stack.try_lock() {
            Ok(mut guard) => {
                guard.clear();
                self.rpn.eval(ctx, &mut guard)
            }
            Err(_) => {
                let mut local_stack = Vec::with_capacity(self.rpn.tokens.len() / 2);
                self.rpn.eval(ctx, &mut local_stack)
            }
        }
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
        let expr = "(2 * 21) + 3 - 35 - ((5 * 80) + 5) + 10";
        let infix_expr: InfixExpr = InfixExpr::try_from(expr).unwrap();
        let rpn_expr: RpnExpr = infix_expr.try_into().unwrap();
        assert_eq!(rpn_expr.tokens, vec![RpnToken::Num(-385.0),]);
    }
}
