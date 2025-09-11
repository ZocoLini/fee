use std::borrow::Cow;

use crate::{
    expr::{ParseableToken, ExprCompiler, NotIndexedResolver, Op}, parsing, prelude::*, resolver::{LockedResolver, ResolverState, UnlockedResolver}, Error, EvalError, IndexedResolver, UContext
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum IVRpn<'e>
{
    Num(f64),
    Var(usize, usize),
    Fn(&'e str, usize),
    Op(Op),
}

impl<'a, 'c, S, V, F, LV, LF> ParseableToken<'a, 'c, S, V, F, LV, LF> for IVRpn<'a>
where S: ResolverState 
{
    #[inline]
    fn num(num: f64) -> Self {
        IVRpn::Num(num)
    }

    #[inline]
    fn op(op: Op) -> Self {
        IVRpn::Op(op)
    }

    #[inline]
    fn var(name: &'a str, _ctx: &'c Context<S, V, F, LV, LF>) -> Self {
        let name_bytes = name.as_bytes();
        let letter = name_bytes[0] - b'a';
        let idx = parsing::parse_usize(&name_bytes[1..]);
        IVRpn::Var(letter as usize, idx)
    }

    #[inline]
    fn fun(name: &'a str, argc: usize, _ctx: &'c Context<S, V, F, LV, LF>) -> Self {
        IVRpn::Fn(name, argc)
    }
}

impl<'e, 'c, F, LF>
    ExprCompiler<
        'e,
        'c,
        Unlocked,
        IndexedResolver<Unlocked, f64>,
        F,
        IndexedResolver<Locked, f64>,
        LF,
        IVRpn<'e>,
    > for Expr<IVRpn<'e>>
where
    F: NotIndexedResolver + UnlockedResolver<ExprFn, LF>,
    LF: LockedResolver<ExprFn>,
{
    fn compile(
        expr: &'e str,
        ctx: &'c UContext<IndexedResolver<Unlocked, f64>, F, IndexedResolver<Locked, f64>, LF>,
    ) -> Result<Expr<IVRpn<'e>>, Error<'e>>
    {
        Expr::try_from((expr, ctx))
    }
}

impl<'e, F, LF>
    ExprEvaluator<'e, Unlocked, IndexedResolver<Unlocked, f64>, F, IndexedResolver<Locked, f64>, LF>
    for Expr<IVRpn<'e>>
where
    F: NotIndexedResolver + UnlockedResolver<ExprFn, LF>,
    LF: LockedResolver<ExprFn>,
{
    fn eval(
        &self,
        ctx: &UContext<IndexedResolver<Unlocked, f64>, F, IndexedResolver<Locked, f64>, LF>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'e>>
    {
        if self.tokens.len() == 1 {
            if let IVRpn::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                IVRpn::Num(num) => stack.push(*num),
                IVRpn::Var(id, idx) => {
                    stack.push(*ctx.get_var_by_index(*id, *idx).ok_or_else(|| {
                        Error::UnknownVar(Cow::Owned(format!(
                            "{}{}",
                            (*id as u8 + b'a') as char,
                            idx
                        )))
                    })?)
                }
                IVRpn::Fn(name, argc) => {
                    if *argc > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - argc;
                    let args = unsafe { stack.get_unchecked(start..) };
                    let val = ctx
                        .get_fn(name)
                        .ok_or_else(|| Error::UnknownFn(Cow::Borrowed(name)))?(
                        args
                    );

                    stack.truncate(start);
                    stack.push(val);
                }
                IVRpn::Op(op) => {
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
