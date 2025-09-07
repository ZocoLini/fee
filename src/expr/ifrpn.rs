use std::borrow::Cow;

use crate::{
    Error, EvalError, IndexedResolver, UContext,
    expr::{ExprCompiler, NotIndexedResolver},
    op::Op,
    parsing,
    prelude::*,
    resolver::{Locked, LockedResolver, Unlocked, UnlockedResolver},
};

#[derive(Debug, PartialEq)]
pub enum IFRpn<'e>
{
    Num(f64),
    Var(&'e str),
    Fn(usize, usize, usize),
    Op(Op),
}

impl From<f64> for IFRpn<'_>
{
    fn from(num: f64) -> Self
    {
        IFRpn::Num(num)
    }
}

impl<'e> From<&'e str> for IFRpn<'e>
{
    fn from(name: &'e str) -> Self
    {
        IFRpn::Var(name)
    }
}

impl From<Op> for IFRpn<'_>
{
    fn from(op: Op) -> Self
    {
        IFRpn::Op(op)
    }
}

impl<'e> From<(&'e str, usize)> for IFRpn<'e>
{
    fn from((name, argc): (&'e str, usize)) -> Self
    {
        let name_bytes = name.as_bytes();
        let letter = name_bytes[0] - b'a';
        let idx = parsing::parse_usize(&name_bytes[1..]);
        IFRpn::Fn(letter as usize, idx, argc)
    }
}

impl<'e, V, LV>
    ExprCompiler<
        'e,
        '_,
        Unlocked,
        V,
        IndexedResolver<Unlocked, ExprFn>,
        LV,
        IndexedResolver<Locked, ExprFn>,
        IFRpn<'e>,
    > for Expr<IFRpn<'e>>
where
    V: NotIndexedResolver + UnlockedResolver<f64, LV>,
    LV: LockedResolver<f64>,
{
    fn compile(
        expr: &'e str,
        _ctx: &UContext<V, IndexedResolver<Unlocked, ExprFn>, LV, IndexedResolver<Locked, ExprFn>>,
    ) -> Result<Expr<IFRpn<'e>>, Error<'e>>
    {
        Expr::try_from(expr)
    }
}

impl<'e, V, LV>
    ExprEvaluator<
        'e,
        Unlocked,
        V,
        IndexedResolver<Unlocked, ExprFn>,
        LV,
        IndexedResolver<Locked, ExprFn>,
    > for Expr<IFRpn<'e>>
where
    V: NotIndexedResolver + UnlockedResolver<f64, LV>,
    LV: LockedResolver<f64>,
{
    fn eval(
        &self,
        ctx: &UContext<V, IndexedResolver<Unlocked, ExprFn>, LV, IndexedResolver<Locked, ExprFn>>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'e>>
    {
        if self.tokens.len() == 1 {
            if let IFRpn::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                IFRpn::Num(num) => stack.push(*num),
                IFRpn::Var(name) => stack.push(
                    *ctx.get_var(name)
                        .ok_or_else(|| Error::UnknownVar(Cow::Borrowed(name)))?,
                ),
                IFRpn::Fn(id, idx, argc) => {
                    if *argc > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - argc;
                    let args = &stack[start..];
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
                IFRpn::Op(op) => {
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
