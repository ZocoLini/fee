use std::borrow::Cow;

use smallvec::{SmallVec, smallvec};

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

impl<'e> From<InfixToken<'e>> for IVRpnToken<'e>
{
    fn from(token: InfixToken<'e>) -> Self
    {
        match token {
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
        }
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

impl<'e> TryFrom<&'e str> for Expr<IVRpnToken<'e>>
{
    type Error = crate::Error<'e>;

    fn try_from(input: &'e str) -> Result<Self, Self::Error>
    {
        let infix_expr = Expr::<InfixToken>::try_from(input)?;
        Self::try_from(infix_expr)
    }
}

impl<'e> TryFrom<Expr<InfixToken<'e>>> for Expr<IVRpnToken<'e>>
{
    type Error = crate::Error<'e>;

    // shunting yard algorithm
    fn try_from(expr: Expr<InfixToken<'e>>) -> Result<Self, Self::Error>
    {
        let mut f64_cache: SmallVec<[f64; 4]> = smallvec![];
        let mut output: Vec<IVRpnToken> = Vec::with_capacity(expr.len());
        let mut ops: Vec<InfixToken> = Vec::new();

        for tok in expr.into_iter() {
            match tok {
                InfixToken::Num(num) => {
                    output.push(IVRpnToken::Num(num));
                    f64_cache.push(num);
                }
                InfixToken::Var(name) => {
                    let name_bytes = name.as_bytes();

                    let letter = name_bytes[0] - b'a';
                    let idx = parsing::parse_usize(&name_bytes[1..]);

                    output.push(IVRpnToken::Var(letter as usize, idx));
                    f64_cache.clear();
                }
                InfixToken::Op(op) => {
                    while let Some(InfixToken::Op(top)) = ops.last() {
                        let prec = op.precedence();
                        let top_prec = top.precedence();
                        let should_pop =
                            top_prec > prec || (!op.is_right_associative() && top_prec == prec);

                        if should_pop {
                            if let Some(InfixToken::Op(op)) = ops.pop() {
                                pre_evaluate(&mut output, &mut f64_cache, op);
                            }
                        } else {
                            break;
                        }
                    }
                    ops.push(InfixToken::Op(op));
                }
                InfixToken::LParen => ops.push(tok),
                InfixToken::RParen => {
                    while let Some(top) = ops.pop() {
                        match top {
                            InfixToken::LParen => break,
                            InfixToken::Op(op) => pre_evaluate(&mut output, &mut f64_cache, op),
                            _ => {
                                output.push(top.into());
                                f64_cache.clear();
                            }
                        }
                    }
                }
                InfixToken::Fn(name, args) => {
                    let fun_call_token = IVRpnToken::Fn(name, args.len());

                    for arg_tokens in args {
                        let rpn_arg: Expr<IVRpnToken<'e>> = arg_tokens.try_into()?;
                        output.extend(rpn_arg.tokens);
                    }

                    output.push(fun_call_token);
                    f64_cache.clear();
                }
            }
        }

        while let Some(top) = ops.pop() {
            if let InfixToken::Op(op) = top {
                pre_evaluate(&mut output, &mut f64_cache, op);
            } else {
                output.push(top.into());
                f64_cache.clear(); // TODO: After this else the if doesn't need to be call because no operator uses 0 elements
            }
        }

        return Ok(Expr { tokens: output });

        fn pre_evaluate(output: &mut Vec<IVRpnToken>, f64_cache: &mut SmallVec<[f64; 4]>, op: Op)
        {
            let n_operands = op.num_operands();

            if f64_cache.len() >= n_operands {
                let output_len = output.len();
                let f64_cache_len = f64_cache.len();

                let start = f64_cache_len - n_operands;
                let num = op.apply(&f64_cache[start..]);
                let token = IVRpnToken::Num(num);

                output.truncate(output_len - n_operands + 1);
                output[output_len - n_operands] = token;

                f64_cache.truncate(f64_cache_len - n_operands + 1);
                f64_cache[f64_cache_len - n_operands] = num;
            } else {
                output.push(IVRpnToken::Op(op));
                f64_cache.clear();
            }
        }
    }
}
