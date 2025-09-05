use std::borrow::Cow;

use crate::{Error, EvalError, expr::NotIndexedResolver, op::Op, prelude::*};

#[derive(Debug, PartialEq)]
pub enum RpnToken<'e>
{
    Num(f64),
    Var(&'e str),
    Fn(&'e str, usize),
    Op(Op),
}

impl From<f64> for RpnToken<'_>
{
    fn from(num: f64) -> Self
    {
        RpnToken::Num(num)
    }
}

impl<'e> From<&'e str> for RpnToken<'e>
{
    fn from(name: &'e str) -> Self
    {
        RpnToken::Var(name)
    }
}

impl From<Op> for RpnToken<'_>
{
    fn from(op: Op) -> Self
    {
        RpnToken::Op(op)
    }
}

impl<'e> From<(&'e str, usize)> for RpnToken<'e>
{
    fn from((name, argc): (&'e str, usize)) -> Self
    {
        RpnToken::Fn(name, argc)
    }
}

impl<'e, V, F> RpnExpr<'e, V, F, RpnToken<'e>> for Expr<RpnToken<'e>>
where
    V: Resolver<Unlocked, f64> + NotIndexedResolver + UnlockedResolver,
    F: Resolver<Unlocked, ExprFn> + NotIndexedResolver + UnlockedResolver,
{
    fn eval(&self, ctx: &Context<Unlocked, V, F>, stack: &mut Vec<f64>) -> Result<f64, Error<'e>>
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
                        .ok_or_else(|| Error::UnknownVar(Cow::Borrowed(name)))?,
                ),
                RpnToken::Fn(name, argc) => {
                    if *argc > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - argc;
                    let args = &stack[start..];
                    let val = ctx
                        .get_fn(name)
                        .ok_or_else(|| Error::UnknownFn(Cow::Borrowed(name)))?(
                        args
                    );

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

#[cfg(test)]
mod tests
{
    use crate::op::Op;

    use super::*;

    #[test]
    fn test_new()
    {
        let expr = "2 - (4 + (p19 - 2) * (p19 + 2))";
        let rpn_expr = Expr::<RpnToken>::try_from(expr).unwrap();
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

        let expr = "abs((2 + 3) * 4, sqrt(5))";
        let rpn_expr = Expr::<RpnToken>::try_from(expr).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                RpnToken::Num(20.0),
                RpnToken::Num(5.0),
                RpnToken::Fn("sqrt", 1),
                RpnToken::Fn("abs", 2),
            ]
        );

        let expr = "(2 * 21) + 3 + -35 - ((5 * 80) + 5) + 10 + -p0";
        let rpn_expr = Expr::<RpnToken>::try_from(expr).unwrap();
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
        let rpn_expr = Expr::<RpnToken>::try_from(expr).unwrap();
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
