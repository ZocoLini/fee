use std::borrow::Cow;

use crate::{
    Error, EvalError, IndexedResolver,
    expr::{infix::InfixToken, rpn::NotIndexedResolver},
    op::Op,
    parsing,
    prelude::*,
};

#[derive(Debug, PartialEq)]
pub enum IVRpnToken<'e>
{
    Num(f64),
    Var(usize, usize),
    Fn(&'e str, usize),
    Op(Op),
}

impl<'e> TryFrom<InfixToken<'e>> for IVRpnToken<'e>
{
    type Error = crate::Error<'e>;

    fn try_from(token: InfixToken<'e>) -> Result<Self, Self::Error>
    {
        let out = match token {
            InfixToken::Num(num) => IVRpnToken::Num(num),
            InfixToken::Var(name) => {
                let name_bytes = name.as_bytes();
                let letter = name_bytes[0] - b'a';
                let idx = parsing::parse_usize(&name_bytes[1..]);
                IVRpnToken::Var(letter as usize, idx)
            }
            InfixToken::Fn(name, args) => IVRpnToken::Fn(name, args.len()),
            InfixToken::Op(op) => IVRpnToken::Op(op),
            _ => unreachable!("logic bug found"),
        };

        Ok(out)
    }
}

impl<'e, F> EvalRpn<IndexedResolver<f64>, F> for Expr<IVRpnToken<'e>>
where
    F: Resolver<ExprFn> + NotIndexedResolver,
{
    type Error = Error<'e>;

    fn eval(
        &self,
        ctx: &Context<IndexedResolver<f64>, F>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Self::Error>
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
                        Error::EvalError(EvalError::UnknownVar(Cow::Owned(format!(
                            "{}{}",
                            (*id as u8 + b'a') as char,
                            idx
                        ))))
                    })?)
                }
                IVRpnToken::Fn(name, argc) => {
                    if *argc > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - argc;
                    let args = &stack[start..];
                    let val = match ctx.call_fn(name, args) {
                        Some(value) => value,
                        None => {
                            return Err(Error::EvalError(EvalError::UnknownFn(Cow::Borrowed(
                                name,
                            ))));
                        }
                    };

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
