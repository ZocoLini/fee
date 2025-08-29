use super::{Expr, Operator, Token};
use crate::{Error, evaluator::Infix, prelude::*};
use std::{borrow::Cow, ops::Deref};

#[derive(Debug, PartialEq)]
struct RPN;

impl Expr<'_, RPN>
{
    fn eval(&self, ctx: &Context<impl VarResolver, impl FnResolver>, stack: &mut Vec<f64>) -> f64
    {
        for tok in self.iter() {
            match tok {
                Token::Number(num) => stack.push(*num),
                Token::Variable(name) => stack.push(*ctx.vals.get(name).expect("Missing variable")),
                Token::FunctionCall(name, argc) => {
                    if *argc > stack.len() {
                        panic!("Not enough args to call {name}")
                    }

                    let start_index = stack.len() - argc;

                    let val = {
                        let args = stack.drain(start_index..stack.len());
                        let args = args.as_slice();

                        ctx.funcs
                            .call(name, &args)
                            .unwrap_or_else(|| panic!("Unknown function: {}", name))
                    };

                    stack.push(val);
                }
                Token::Operator(op) => {
                    let b = stack.pop().expect("Stack underflow for operator");
                    let a = stack.pop().expect("Stack underflow for operator");
                    let res = op.apply(a, b);
                    stack.push(res);
                }
                _ => panic!("Unexpected token in RPN: {:?}", tok),
            }
        }

        if stack.len() != 1 {
            panic!("Stack didn't contain exactly one element after evaluation")
        } else {
            stack.pop().unwrap()
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

        for (i, tok) in expr.tokens.into_iter().enumerate() {
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
                _ => {
                    return Err(Error::UnexpectedToken(Cow::Owned(format!("{:?}", tok,)), i));
                }
            }
        }

        while let Some(op) = ops.pop() {
            output.push(op);
        }

        Ok(Self {
            tokens: output,
            type_: RPN,
        })
    }
}

pub struct RPNEvaluator<'e, 'c, V: VarResolver, F: FnResolver>
{
    ctx: &'c Context<V, F>,
    rpn: Expr<'e, RPN>,
}

impl<'e, 'c, V: VarResolver, F: FnResolver> RPNEvaluator<'e, 'c, V, F>
{
    pub fn new(expr: &'e str, ctx: &'c Context<V, F>) -> Result<Self, crate::Error<'e>>
    {
        let infix_expr = Expr::new(expr)?;
        let rpn_expr = Expr::try_from(infix_expr)?;

        Ok(RPNEvaluator { ctx, rpn: rpn_expr })
    }
}

impl<'e, 'c, V: VarResolver, F: FnResolver> Evaluator for RPNEvaluator<'e, 'c, V, F>
{
    fn eval(&self) -> f64
    {
        let mut stack = Vec::new();
        self.rpn.eval(self.ctx, &mut stack)
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
        let infix_expr = Expr {
            tokens: vec![
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
            type_: Infix,
        };

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
        let infix_expr = Expr {
            tokens: vec![Token::Function(
                "abs",
                vec![
                    Expr {
                        tokens: vec![
                            Token::LParen,
                            Token::Number(2.0),
                            Token::Operator(Operator::Add),
                            Token::Number(3.0),
                            Token::RParen,
                            Token::Operator(Operator::Mul),
                            Token::Number(4.0),
                        ],
                        type_: Infix,
                    },
                    Expr {
                        tokens: vec![Token::Function(
                            "sqrt",
                            vec![Expr {
                                tokens: vec![Token::Number(5.0)],
                                type_: Infix,
                            }],
                        )],
                        type_: Infix,
                    },
                ],
            )],
            type_: Infix,
        };

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

    #[test]
    fn test_errors()
    {
        // TODO: The indices of the errors are relative to the start of the functions due
        //  to the recursion used to convert the expression

        let infix_expr = Expr {
            tokens: vec![
                Token::Number(2.0),
                Token::Operator(Operator::Sub),
                Token::Number(4.0),
                Token::FunctionCall("asd", 3),
            ],
            type_: Infix,
        };

        let result: Result<Expr<RPN>, Error> = infix_expr.try_into();

        assert_eq!(
            result,
            Err(Error::UnexpectedToken(
                Cow::Borrowed("FunctionCall(\"asd\", 3)"),
                3
            ))
        );
    }
}
