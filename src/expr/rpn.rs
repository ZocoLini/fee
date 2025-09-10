use std::borrow::Cow;

use crate::{
    Error, EvalError, UContext,
    expr::{ExprCompiler, NotIndexedResolver},
    op::Op,
    prelude::*,
    resolver::{LockedResolver, ResolverState, Unlocked, UnlockedResolver},
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Rpn<'e>
{
    Num(f64),
    Var(&'e str),
    Fn(&'e str, usize),
    Op(Op),
}

impl From<f64> for Rpn<'_>
{
    fn from(num: f64) -> Self
    {
        Rpn::Num(num)
    }
}

impl From<Op> for Rpn<'_>
{
    fn from(op: Op) -> Self
    {
        Rpn::Op(op)
    }
}

impl<'a, S, V, F, LV, LF> From<(&'a str, &'a Context<S, V, F, LV, LF>)> for Rpn<'a>
where
    S: ResolverState,
{
    fn from((name, _): (&'a str, &'a Context<S, V, F, LV, LF>)) -> Self
    {
        Rpn::Var(name)
    }
}

impl<'a, S, V, F, LV, LF> From<(&'a str, usize, &'a Context<S, V, F, LV, LF>)> for Rpn<'a>
where
    S: ResolverState,
{
    fn from((name, argc, _): (&'a str, usize, &'a Context<S, V, F, LV, LF>)) -> Self
    {
        Rpn::Fn(name, argc)
    }
}

impl<'e: 'c, 'c: 'e, V, F, LV, LF> ExprCompiler<'e, 'c, Unlocked, V, F, LV, LF, Rpn<'e>>
    for Expr<Rpn<'e>>
where
    V: NotIndexedResolver + UnlockedResolver<f64, LV>,
    F: NotIndexedResolver + UnlockedResolver<ExprFn, LF>,
    LV: LockedResolver<f64>,
    LF: LockedResolver<ExprFn>,
{
    fn compile(expr: &'e str, ctx: &'c UContext<V, F, LV, LF>) -> Result<Expr<Rpn<'e>>, Error<'e>>
    {
        Expr::try_from((expr, ctx))
    }
}

impl<'e, V, F, LV, LF> ExprEvaluator<'e, Unlocked, V, F, LV, LF> for Expr<Rpn<'e>>
where
    V: NotIndexedResolver + UnlockedResolver<f64, LV>,
    F: NotIndexedResolver + UnlockedResolver<ExprFn, LF>,
    LV: LockedResolver<f64>,
    LF: LockedResolver<ExprFn>,
{
    fn eval(&self, ctx: &UContext<V, F, LV, LF>, stack: &mut Vec<f64>) -> Result<f64, Error<'e>>
    {
        if self.tokens.len() == 1 {
            if let Rpn::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                Rpn::Num(num) => stack.push(*num),
                Rpn::Var(name) => stack.push(
                    *ctx.get_var(name)
                        .ok_or_else(|| Error::UnknownVar(Cow::Borrowed(name)))?,
                ),
                Rpn::Fn(name, argc) => {
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
                Rpn::Op(op) => {
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
        let ctx = Context::empty();

        let expr = "2 - (4 + (p19 - 2) * (p19 + 2))";
        let rpn_expr = Expr::<Rpn>::try_from((expr, &ctx)).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                Rpn::Num(2.0),
                Rpn::Num(4.0),
                Rpn::Var("p19"),
                Rpn::Num(2.0),
                Rpn::Op(Op::Sub),
                Rpn::Var("p19"),
                Rpn::Num(2.0),
                Rpn::Op(Op::Add),
                Rpn::Op(Op::Mul),
                Rpn::Op(Op::Add),
                Rpn::Op(Op::Sub)
            ]
        );

        let expr = "sqrt(5)";
        let rpn_expr = Expr::<Rpn>::try_from((expr, &ctx)).unwrap();
        assert_eq!(rpn_expr.tokens, vec![Rpn::Num(5.0), Rpn::Fn("sqrt", 1),]);

        let expr = "abs(4, sqrt(5))";
        let rpn_expr = Expr::<Rpn>::try_from((expr, &ctx)).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                Rpn::Num(4.0),
                Rpn::Num(5.0),
                Rpn::Fn("sqrt", 1),
                Rpn::Fn("abs", 2),
            ]
        );

        let expr = "(2 * 21) + 3 + -35 - ((5 * 80) + 5) + 10 + -p0";
        let rpn_expr = Expr::<Rpn>::try_from((expr, &ctx)).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                Rpn::Num(-385.0),
                Rpn::Var("p0"),
                Rpn::Op(Op::Neg),
                Rpn::Op(Op::Add),
            ]
        );

        let expr = "-y1 * (p2 - p3*y0)";
        let rpn_expr = Expr::<Rpn>::try_from((expr, &ctx)).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                Rpn::Var("y1"),
                Rpn::Op(Op::Neg),
                Rpn::Var("p2"),
                Rpn::Var("p3"),
                Rpn::Var("y0"),
                Rpn::Op(Op::Mul),
                Rpn::Op(Op::Sub),
                Rpn::Op(Op::Mul),
            ]
        );
    }
}
