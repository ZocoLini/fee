use std::borrow::Cow;

use crate::{
    Error, EvalError, IndexedResolver, expr::rpn::NotIndexedResolver, op::Op, parsing, prelude::*,
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

impl super::FromNamedFn<'_, IFRpnToken<'_>> for IFRpnToken<'_>
{
    fn from_fn(name: &str, argc: usize) -> Self
    {
        let name_bytes = name.as_bytes();
        let letter = name_bytes[0] - b'a';
        let idx = parsing::parse_usize(&name_bytes[1..]);
        IFRpnToken::Fn(letter as usize, idx, argc)
    }
}

impl<'e, V> EvalRpn<V, IndexedResolver<ExprFn>> for Expr<IFRpnToken<'e>>
where
    V: Resolver<f64> + NotIndexedResolver,
{
    type Error = Error<'e>;

    fn eval(
        &self,
        ctx: &Context<V, IndexedResolver<ExprFn>>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Self::Error>
    {
        if self.tokens.len() == 1 {
            if let IFRpnToken::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                IFRpnToken::Num(num) => stack.push(*num),
                IFRpnToken::Var(name) => stack.push(
                    *ctx.get_var(name)
                        .ok_or(Error::EvalError(EvalError::UnknownVar(Cow::Borrowed(name))))?,
                ),
                IFRpnToken::Fn(id, idx, argc) => {
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
