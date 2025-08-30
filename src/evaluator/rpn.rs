use crate::lexer::{Expr, Infix};
use crate::token::{Operator, Token};
use crate::{Error, EvalError, prelude::*};
use std::sync::{Arc, RwLock};
use std::{borrow::Cow, ops::Deref};

#[derive(Debug, PartialEq)]
struct RPN;

impl Expr<'_, RPN>
{
    fn eval<'e>(
        &'e self,
        ctx: &Context<impl VarResolver, impl FnResolver>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'e>>
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

                    let start_index = stack.len() - argc;

                    let val = {
                        let args = stack.drain(start_index..stack.len());
                        let args = args.as_slice();

                        match ctx.call_fn(name, args) {
                            Some(value) => value,
                            None => {
                                return Err(Error::EvalError(EvalError::UnknownFn(Cow::Borrowed(
                                    name,
                                ))));
                            }
                        }
                    };

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

        if stack.len() != 1 {
            return Err(Error::EvalError(EvalError::MalformedExpression));
        }

        Ok(stack.pop().unwrap()) // TODO: Remove this unwrap
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

        for (i, tok) in expr.into_iter().enumerate() {
            match tok {
                Token::Number(_) | Token::Variable(_) => {
                    output.push(tok);
                }
                Token::Operator(op) => {
                    while let Some(Token::Operator(top)) = ops.last() {
                        let should_pop = if op.is_right_associative() {
                            op.precedence() < top.precedence()
                        } else {
                            op.precedence() <= top.precedence()
                        };

                        if should_pop {
                            output.push(ops.pop().unwrap());
                        } else {
                            break;
                        }
                    }
                    ops.push(tok);
                }
                Token::LParen => ops.push(tok),
                Token::RParen => {
                    while let Some(top) = ops.pop() {
                        if let Token::LParen = top {
                            break;
                        } else {
                            output.push(top);
                        }
                    }
                }
                Token::Function(name, args) => {
                    let fun_call_token = Token::FunctionCall(name, args.len());

                    let rpn_args: Result<Vec<Expr<RPN>>, _> = args
                        .into_iter()
                        .map(|arg_tokens| arg_tokens.try_into())
                        .collect();

                    let rpn_args = rpn_args?;

                    rpn_args
                        .iter()
                        .for_each(|rpn_arg| output.extend_from_slice(&rpn_arg));

                    output.push(fun_call_token);
                }
                _ => {}
            }
        }

        while let Some(op) = ops.pop() {
            output.push(op);
        }

        Ok(Expr::new(output, RPN))
    }
}

pub struct RPNEvaluator<'e, 'c, V: VarResolver, F: FnResolver>
{
    ctx: &'c mut Context<V, F>,
    rpn: Expr<'e, RPN>,
}

impl<'e, 'c, V: VarResolver, F: FnResolver> Evaluator<'e, 'c, V, F> for RPNEvaluator<'e, 'c, V, F>
{
    fn new(expr: &'e str, ctx: &'c mut Context<V, F>) -> Result<Self, crate::Error<'e>>
    {
        let infix_expr = Expr::try_from(expr)?;
        let rpn_expr = Expr::try_from(infix_expr)?;

        Ok(RPNEvaluator { ctx, rpn: rpn_expr })
    }

    fn eval(&'e self) -> Result<f64, Error<'e>>
    {
        let mut stack = Vec::new();
        self.rpn.eval(self.ctx, &mut stack)
    }

    fn context_mut(&mut self) -> &mut Context<V, F>
    {
        self.ctx
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
                Token::Number(2.0),
                Token::Number(3.0),
                Token::Operator(Operator::Add),
                Token::Number(4.0),
                Token::Operator(Operator::Mul),
                Token::Number(5.0),
                Token::FunctionCall("sqrt", 1),
                Token::FunctionCall("abs", 2),
            ]
        );
    }
}
