use std::borrow::Cow;

use crate::{
    Error, EvalError, IndexedResolver, expr::NotIndexedResolver, op::Op, parsing, prelude::*,
};

#[derive(Debug, PartialEq)]
pub enum IFRpnToken<'e>
{
    Num(f64),
    Var(&'e str),
    Fn(usize, usize, usize),
    Op(Op),
}

impl From<f64> for IFRpnToken<'_>
{
    fn from(num: f64) -> Self
    {
        IFRpnToken::Num(num)
    }
}

impl<'e> From<&'e str> for IFRpnToken<'e>
{
    fn from(name: &'e str) -> Self
    {
        IFRpnToken::Var(name)
    }
}

impl From<Op> for IFRpnToken<'_>
{
    fn from(op: Op) -> Self
    {
        IFRpnToken::Op(op)
    }
}

impl<'e> From<(&'e str, usize)> for IFRpnToken<'e>
{
    fn from((name, argc): (&'e str, usize)) -> Self
    {
        let name_bytes = name.as_bytes();
        let letter = name_bytes[0] - b'a';
        let idx = parsing::parse_usize(&name_bytes[1..]);
        IFRpnToken::Fn(letter as usize, idx, argc)
    }
}

impl<'e, V> RpnExpr<'e, V, IndexedResolver<ExprFn>, IFRpnToken<'e>> for Expr<IFRpnToken<'e>>
where
    V: Resolver<f64> + NotIndexedResolver,
{
    fn eval(
        &self,
        ctx: &Context<V, IndexedResolver<ExprFn>>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'e>>
    {
        if self.tokens.len() == 1 {
            if let IFRpnToken::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                IFRpnToken::Num(num) => stack.push(*num),
                IFRpnToken::Var(name) => {
                    stack.push(*ctx.get_var(name).ok_or_else(|| {
                        Error::EvalError(EvalError::UnknownVar(Cow::Borrowed(name)))
                    })?)
                }
                IFRpnToken::Fn(id, idx, argc) => {
                    if *argc > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - argc;
                    let args = &stack[start..];
                    let val = ctx.call_fn_by_index(*id, *idx, args).ok_or_else(|| {
                        Error::EvalError(EvalError::UnknownFn(Cow::Owned(format!(
                            "{}{}",
                            (*id as u8 + b'a') as char,
                            idx
                        ))))
                    })?;

                    stack.truncate(start);
                    stack.push(val);
                }
                IFRpnToken::Op(op) => {
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
