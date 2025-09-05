use std::borrow::Cow;

use crate::{
    Error, EvalError, IndexedResolver, expr::NotIndexedResolver, op::Op, parsing, prelude::*,
};

#[derive(Debug, PartialEq)]
pub enum IVRpnToken<'e>
{
    Num(f64),
    Var(usize, usize),
    Fn(&'e str, usize),
    Op(Op),
}

impl From<f64> for IVRpnToken<'_>
{
    fn from(num: f64) -> Self
    {
        IVRpnToken::Num(num)
    }
}

impl<'e> From<&'e str> for IVRpnToken<'e>
{
    fn from(name: &'e str) -> Self
    {
        let name_bytes = name.as_bytes();
        let letter = name_bytes[0] - b'a';
        let idx = parsing::parse_usize(&name_bytes[1..]);
        IVRpnToken::Var(letter as usize, idx)
    }
}

impl From<Op> for IVRpnToken<'_>
{
    fn from(op: Op) -> Self
    {
        IVRpnToken::Op(op)
    }
}

impl<'e> From<(&'e str, usize)> for IVRpnToken<'e>
{
    fn from((name, argc): (&'e str, usize)) -> Self
    {
        IVRpnToken::Fn(name, argc)
    }
}

impl<'e, F, LF>
    RpnExpr<'e, IndexedResolver<Unlocked, f64>, F, IndexedResolver<Locked, f64>, LF, IVRpnToken<'e>>
    for Expr<IVRpnToken<'e>>
where
    F: NotIndexedResolver + UnlockedResolver<ExprFn, LF>,
    LF: LockedResolver<ExprFn>,
{
    fn eval_unlocked(
        &self,
        ctx: &Context<
            Unlocked,
            IndexedResolver<Unlocked, f64>,
            F,
            IndexedResolver<Locked, f64>,
            LF,
        >,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'e>>
    {
        if self.tokens.len() == 1 {
            if let IVRpnToken::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                IVRpnToken::Num(num) => stack.push(*num),
                IVRpnToken::Var(id, idx) => {
                    stack.push(*ctx.get_var_by_index(*id, *idx).ok_or_else(|| {
                        Error::UnknownVar(Cow::Owned(format!(
                            "{}{}",
                            (*id as u8 + b'a') as char,
                            idx
                        )))
                    })?)
                }
                IVRpnToken::Fn(name, argc) => {
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
                IVRpnToken::Op(op) => {
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
