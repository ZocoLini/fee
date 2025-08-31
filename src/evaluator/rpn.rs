use crate::lexer::{Expr, Infix};
use crate::token::{Operator, Token};
use crate::{Error, EvalError, prelude::*};
use std::cell::UnsafeCell;
use std::panic;
use std::sync::{Arc, Mutex, RwLock};
use std::{borrow::Cow, ops::Deref};

#[derive(Debug, PartialEq)]
pub struct RPN;

impl Expr<'_, RPN>
{
    fn eval<'e>(&'e self, ctx: &impl Context, stack: &mut Vec<f64>) -> Result<f64, Error<'e>>
    {
        for tok in self.iter() {
            match tok {
                Token::Number(num) => stack.push(*num),
                Token::Variable(name) => stack.push(
                    *ctx.get_var(name)
                        .ok_or(Error::EvalError(EvalError::UnknownVar(Cow::Borrowed(name))))?,
                ),
                Token::FunctionCall(name, argc) => {
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
                Token::Operator(op) => {
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
                _ => {}
            }
        }

        match stack.pop() {
            Some(result) if stack.is_empty() => Ok(result),
            _ => Err(Error::EvalError(EvalError::MalformedExpression)),
        }
    }
}

impl<'e> TryFrom<Expr<'e, Infix>> for Expr<'e, RPN>
{
    type Error = crate::Error<'e>;

    // shunting yard algorithm
    fn try_from(expr: Expr<'e, Infix>) -> Result<Self, Self::Error>
    {
        let mut output: Vec<Token> = Vec::with_capacity(expr.len());
        let mut ops: Vec<Token> = Vec::new();

        let mut num_count = 0;

        for tok in expr.into_iter() {
            match tok {
                Token::Number(_) => {
                    output.push(tok);
                    num_count += 1;
                }
                Token::Variable(_) => {
                    output.push(tok);
                    num_count = 0;
                }
                Token::Operator(op) => {
                    while let Some(Token::Operator(top)) = ops.last() {
                        let should_pop = if op.is_right_associative() {
                            op.precedence() < top.precedence()
                        } else {
                            op.precedence() <= top.precedence()
                        };

                        if should_pop {
                            let op_token = ops
                                .pop()
                                .expect("stack already checked to contain an operator token");
                            pre_evaluate(&mut output, op_token, &mut num_count);
                        } else {
                            break;
                        }
                    }
                    ops.push(tok);
                }
                Token::LParen => ops.push(tok),
                Token::RParen => {
                    while let Some(top) = ops.pop() {
                        match top {
                            Token::LParen => break,
                            Token::Operator(op) => pre_evaluate(&mut output, top, &mut num_count),
                            _ => {
                                output.push(top);
                                num_count = 0;
                            }
                        }
                    }
                }
                Token::Function(name, args) => {
                    let fun_call_token = Token::FunctionCall(name, args.len());

                    for arg_tokens in args {
                        let rpn_arg: Expr<RPN> = arg_tokens.try_into()?;
                        output.extend(rpn_arg);
                    }

                    output.push(fun_call_token);
                    num_count = 0;
                }
                _ => {}
            }
        }

        while let Some(top) = ops.pop() {
            if let Token::Operator(op) = top {
                pre_evaluate(&mut output, top, &mut num_count);
            } else {
                output.push(top);
                num_count = 0;
            }
        }

        return Ok(Expr::new(output, RPN));

        #[inline(always)]
        fn pre_evaluate<'t>(output: &mut Vec<Token<'t>>, op_token: Token<'t>, num_count: &mut usize)
        {
            let op = if let Token::Operator(op) = op_token {
                op
            } else {
                panic!("expected an operator token")
            };

            // Each operand may have a different number of arguments
            if *num_count >= 2 {
                let b = if let Some(Token::Number(value)) = output.pop() {
                    value
                } else {
                    panic!("expected a number");
                };
                let a = if let Some(Token::Number(value)) = output.pop() {
                    value
                } else {
                    panic!("expected a number");
                };
                output.push(Token::Number(op.apply(a, b)));
                *num_count -= 1;
            } else {
                output.push(op_token);
                *num_count = 0;
            }
        }
    }
}

pub struct RPNEvaluator<'e>
{
    rpn: Expr<'e, RPN>,

    stack: Mutex<Vec<f64>>,
}

impl<'e> Evaluator<'e> for RPNEvaluator<'e>
{
    fn new(expr: &'e str) -> Result<Self, crate::Error<'e>>
    {
        let infix_expr = Expr::try_from(expr)?;
        let rpn_expr = Expr::try_from(infix_expr)?;

        let stack = Mutex::new(Vec::with_capacity(rpn_expr.len() / 2));

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
                // contención -> uso stack temporal en el heap
                let mut local_stack = Vec::with_capacity(self.rpn.len() / 2);
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
        let infix_expr = Expr::new(
            vec![
                Token::Number(2.0),
                Token::Operator(Operator::Sub),
                Token::LParen,
                Token::Number(4.0),
                Token::Operator(Operator::Add),
                Token::LParen,
                Token::Variable("p19"),
                Token::Operator(Operator::Sub),
                Token::Number(2.0),
                Token::RParen,
                Token::Operator(Operator::Mul),
                Token::LParen,
                Token::Variable("p19"),
                Token::Operator(Operator::Add),
                Token::Number(2.0),
                Token::RParen,
                Token::RParen,
            ],
            Infix,
        );

        let rpn_expr: Expr<RPN> = infix_expr.try_into().unwrap();
        assert_eq!(
            *rpn_expr,
            vec![
                Token::Number(2.0),
                Token::Number(4.0),
                Token::Variable("p19"),
                Token::Number(2.0),
                Token::Operator(Operator::Sub),
                Token::Variable("p19"),
                Token::Number(2.0),
                Token::Operator(Operator::Add),
                Token::Operator(Operator::Mul),
                Token::Operator(Operator::Add),
                Token::Operator(Operator::Sub)
            ]
        );

        //abs((2 + 3) * 4, sqrt(5))
        let infix_expr = Expr::new(
            vec![Token::Function(
                "abs",
                vec![
                    Expr::new(
                        vec![
                            Token::LParen,
                            Token::Number(2.0),
                            Token::Operator(Operator::Add),
                            Token::Number(3.0),
                            Token::RParen,
                            Token::Operator(Operator::Mul),
                            Token::Number(4.0),
                        ],
                        Infix,
                    ),
                    Expr::new(
                        vec![Token::Function(
                            "sqrt",
                            vec![Expr::new(vec![Token::Number(5.0)], Infix)],
                        )],
                        Infix,
                    ),
                ],
            )],
            Infix,
        );

        let rpn_expr: Expr<RPN> = infix_expr.try_into().unwrap();
        assert_eq!(
            *rpn_expr,
            vec![
                Token::Number(20.0),
                Token::Number(5.0),
                Token::FunctionCall("sqrt", 1),
                Token::FunctionCall("abs", 2),
            ]
        );
    }

    #[test]
    fn test_str_to_rpn()
    {
        let expr = "(2 * 21) + 3 - 35 - ((5 * 80) + 5) + 10";
        let infix_expr: Expr<'_, Infix> = Expr::try_from(expr).unwrap();
        let rpn_expr: Expr<RPN> = infix_expr.try_into().unwrap();
        assert_eq!(*rpn_expr, vec![Token::Number(-385.0),]);
    }
}
