use std::borrow::Cow;

use crate::{Error, EvalError, IndexedResolver, op::Op, parsing, prelude::*};

#[derive(Debug, PartialEq)]
pub enum IRpnToken
{
    Num(f64),
    Var(usize, usize),
    Fn(usize, usize, usize),
    Op(Op),
}

impl From<f64> for IRpnToken
{
    fn from(num: f64) -> Self
    {
        IRpnToken::Num(num)
    }
}

impl From<&str> for IRpnToken
{
    fn from(name: &str) -> Self
    {
        let name_bytes = name.as_bytes();
        let letter = name_bytes[0] - b'a';
        let idx = parsing::parse_usize(&name_bytes[1..]);
        IRpnToken::Var(letter as usize, idx)
    }
}

impl From<Op> for IRpnToken
{
    fn from(op: Op) -> Self
    {
        IRpnToken::Op(op)
    }
}

impl<'e> From<(&'e str, usize)> for IRpnToken
{
    fn from((name, argc): (&'e str, usize)) -> Self
    {
        let name_bytes = name.as_bytes();
        let letter = name_bytes[0] - b'a';
        let idx = parsing::parse_usize(&name_bytes[1..]);
        IRpnToken::Fn(letter as usize, idx, argc)
    }
}

impl<'e> RpnExpr<'e, IndexedResolver<f64>, IndexedResolver<ExprFn>, IRpnToken> for Expr<IRpnToken>
{
    fn eval(
        &self,
        ctx: &Context<IndexedResolver<f64>, IndexedResolver<ExprFn>>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'e>>
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::op::Op;

    #[test]
    fn test_new()
    {
        let expr = "2 - (4 + (p19 - 2) * (p19 + 2))";
        let rpn_expr = Expr::<IRpnToken>::try_from(expr).unwrap();
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

        let expr = "f0((2 + 3) * 4, f1(5))";
        let rpn_expr = Expr::<IRpnToken>::try_from(expr).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                IRpnToken::Num(20.0),
                IRpnToken::Num(5.0),
                IRpnToken::Fn((b'f' - b'a') as usize, 1, 1),
                IRpnToken::Fn((b'f' - b'a') as usize, 0, 2),
            ]
        );

        let expr = "(2 * 21) + 3 + -35 - ((5 * 80) + 5) + 10 + -p0";
        let rpn_expr = Expr::<IRpnToken>::try_from(expr).unwrap();
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
        let rpn_expr = Expr::<IRpnToken>::try_from(expr).unwrap();
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
