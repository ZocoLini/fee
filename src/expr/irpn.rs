use std::borrow::Cow;

use crate::{
    Error, EvalError, IndexedResolver, UContext,
    expr::{ExprCompiler, Op, ParseableToken},
    parsing,
    prelude::*,
    resolver::ResolverState,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum IRpn
{
    Num(f64),
    Var(usize, usize),
    Fn(usize, usize, usize),
    Op(Op),
}

impl<'a, 'c, S, V, F, LV, LF> ParseableToken<'a, 'c, S, V, F, LV, LF> for IRpn
where
    S: ResolverState,
{
    #[inline]
    fn f64(num: f64) -> Self
    {
        IRpn::Num(num)
    }

    #[inline]
    fn i64(num: i64) -> Self
    {
        IRpn::Num(num as f64)
    }

    #[inline]
    fn bool(val: bool) -> Self
    {
        IRpn::Num(if val { 1.0 } else { 0.0 })
    }

    #[inline]
    fn op(op: Op) -> Self
    {
        IRpn::Op(op)
    }

    #[inline]
    fn var(name: &'a str, _ctx: &'c Context<S, V, F, LV, LF>) -> Self
    {
        let name_bytes = name.as_bytes();
        let letter = name_bytes[0] - b'a';
        let idx = parsing::parse_usize(&name_bytes[1..]);
        IRpn::Var(letter as usize, idx)
    }

    #[inline]
    fn fun(name: &'a str, argc: usize, _ctx: &'c Context<S, V, F, LV, LF>) -> Self
    {
        let name_bytes = name.as_bytes();
        let letter = name_bytes[0] - b'a';
        let idx = parsing::parse_usize(&name_bytes[1..]);
        IRpn::Fn(letter as usize, idx, argc)
    }
}

impl<'e, 'c>
    ExprCompiler<
        'e,
        'c,
        Unlocked,
        IndexedResolver<Unlocked, f64>,
        IndexedResolver<Unlocked, ExprFn>,
        IndexedResolver<Locked, f64>,
        IndexedResolver<Locked, ExprFn>,
        IRpn,
    > for Expr<IRpn>
{
    fn compile(
        expr: &'e str,
        ctx: &'c UContext<
            IndexedResolver<Unlocked, f64>,
            IndexedResolver<Unlocked, ExprFn>,
            IndexedResolver<Locked, f64>,
            IndexedResolver<Locked, ExprFn>,
        >,
    ) -> Result<Expr<IRpn>, Error<'e>>
    {
        Expr::try_from((expr, ctx))
    }
}

impl<'e>
    ExprEvaluator<
        'e,
        Unlocked,
        IndexedResolver<Unlocked, f64>,
        IndexedResolver<Unlocked, ExprFn>,
        IndexedResolver<Locked, f64>,
        IndexedResolver<Locked, ExprFn>,
    > for Expr<IRpn>
{
    fn eval(
        &self,
        ctx: &UContext<
            IndexedResolver<Unlocked, f64>,
            IndexedResolver<Unlocked, ExprFn>,
            IndexedResolver<Locked, f64>,
            IndexedResolver<Locked, ExprFn>,
        >,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'e>>
    {
        if self.tokens.len() == 1 {
            if let IRpn::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                IRpn::Num(num) => stack.push(*num),
                IRpn::Var(id, idx) => {
                    stack.push(*ctx.get_var_by_index(*id, *idx).ok_or_else(|| {
                        Error::UnknownVar(Cow::Owned(format!(
                            "{}{}",
                            (*id as u8 + b'a') as char,
                            idx
                        )))
                    })?)
                }
                IRpn::Fn(id, idx, argc) => {
                    if *argc > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - argc;
                    let args = unsafe { stack.get_unchecked(start..) };
                    let val = ctx.call_fn_by_index(*id, *idx, args).ok_or_else(|| {
                        Error::UnknownFn(Cow::Owned(format!(
                            "{}{}",
                            (*id as u8 + b'a') as char,
                            idx
                        )))
                    })?;
                    stack.truncate(start);
                    stack.push(val);
                }
                IRpn::Op(op) => {
                    if op.num_operands() > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - op.num_operands();
                    let args = unsafe { stack.get_unchecked(start..) };
                    let res = op.apply(args);
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

    #[test]
    fn test_new()
    {
        let ctx = Context::empty();

        let expr = "2 - (4 + (p19 - 2) * (p19 + 2))";
        let rpn_expr = Expr::<IRpn>::try_from((expr, &ctx)).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                IRpn::Num(2.0),
                IRpn::Num(4.0),
                IRpn::Var((b'p' - b'a') as usize, 19),
                IRpn::Num(2.0),
                IRpn::Op(Op::Sub),
                IRpn::Var((b'p' - b'a') as usize, 19),
                IRpn::Num(2.0),
                IRpn::Op(Op::Add),
                IRpn::Op(Op::Mul),
                IRpn::Op(Op::Add),
                IRpn::Op(Op::Sub)
            ]
        );

        let expr = "f0((2 + 3) * 4, f1(5))";
        let rpn_expr = Expr::<IRpn>::try_from((expr, &ctx)).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                IRpn::Num(20.0),
                IRpn::Num(5.0),
                IRpn::Fn((b'f' - b'a') as usize, 1, 1),
                IRpn::Fn((b'f' - b'a') as usize, 0, 2),
            ]
        );

        let expr = "(2 * 21) + 3 + -35 - ((5 * 80) + 5) + 10 + -p0";
        let rpn_expr = Expr::<IRpn>::try_from((expr, &ctx)).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                IRpn::Num(-385.0),
                IRpn::Var((b'p' - b'a') as usize, 0),
                IRpn::Op(Op::Neg),
                IRpn::Op(Op::Add),
            ]
        );

        let expr = "-y1 * (p2 - p3*y0)";
        let rpn_expr = Expr::<IRpn>::try_from((expr, &ctx)).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                IRpn::Var((b'y' - b'a') as usize, 1),
                IRpn::Op(Op::Neg),
                IRpn::Var((b'p' - b'a') as usize, 2),
                IRpn::Var((b'p' - b'a') as usize, 3),
                IRpn::Var((b'y' - b'a') as usize, 0),
                IRpn::Op(Op::Mul),
                IRpn::Op(Op::Sub),
                IRpn::Op(Op::Mul),
            ]
        );
    }
}
