pub mod ifrpn;
pub mod irpn;
pub mod ivrpn;
pub mod rpn;

pub use ifrpn::IFRpnToken;
pub use irpn::IRpnToken;
pub use ivrpn::IVRpnToken;
pub use rpn::RpnToken;
use smallvec::{SmallVec, smallvec};

use crate::{
    ConstantResolver, DefaultResolver, EmptyResolver, Error, SmallResolver,
    expr::{Expr, infix::InfixToken},
    op::Op,
};

trait NotIndexedResolver {}
impl<State, T> NotIndexedResolver for DefaultResolver<State, T> {}
impl<T> NotIndexedResolver for ConstantResolver<T> {}
impl<State, T> NotIndexedResolver for SmallResolver<State, T> {}
impl NotIndexedResolver for EmptyResolver {}

impl<T> Expr<T> {}

impl<'e, T> TryFrom<&'e str> for Expr<T>
where
    T: TryFrom<InfixToken<'e>, Error = Error<'e>>,
{
    type Error = crate::Error<'e>;

    fn try_from(input: &'e str) -> Result<Self, Self::Error>
    {
        let infix_expr = Expr::<InfixToken>::try_from(input)?;
        Expr::<T>::try_from(infix_expr)
    }
}

impl<'e, T> TryFrom<Expr<InfixToken<'e>>> for Expr<T>
where
    T: TryFrom<InfixToken<'e>, Error = Error<'e>>,
{
    type Error = Error<'e>;

    fn try_from(expr: Expr<InfixToken<'e>>) -> Result<Self, Self::Error>
    {
        let mut f64_cache: SmallVec<[f64; 4]> = smallvec![];
        let mut output: Vec<T> = Vec::with_capacity(expr.len());
        let mut ops: Vec<InfixToken> = Vec::new();

        for tok in expr.into_iter() {
            match tok {
                InfixToken::Num(num) => {
                    output.push(tok.try_into()?);
                    f64_cache.push(num);
                }
                InfixToken::Var(_) => {
                    output.push(tok.try_into()?);
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
                                pre_evaluate(&mut output, &mut f64_cache, op)?;
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
                            InfixToken::Op(op) => pre_evaluate(&mut output, &mut f64_cache, op)?,
                            _ => {
                                output.push(top.try_into()?);
                                f64_cache.clear();
                            }
                        }
                    }
                }
                InfixToken::Fn(name, args) => {
                    for arg_tokens in args.clone() {
                        let rpn_arg: Expr<T> = arg_tokens.try_into()?;
                        output.extend(rpn_arg.tokens);
                    }
                    let fn_token = InfixToken::Fn(name, args);
                    output.push(fn_token.try_into()?);

                    f64_cache.clear();
                }
            }
        }

        while let Some(top) = ops.pop() {
            if let InfixToken::Op(op) = top {
                pre_evaluate(&mut output, &mut f64_cache, op)?;
            } else {
                output.push(top.try_into()?);
                f64_cache.clear(); // TODO: After this else the if doesn't need to be call because no operator uses 0 elements
            }
        }

        return Ok(Expr { tokens: output });

        fn pre_evaluate<'e, T>(
            output: &mut Vec<T>,
            f64_cache: &mut SmallVec<[f64; 4]>,
            op: Op,
        ) -> Result<(), Error<'e>>
        where
            T: TryFrom<InfixToken<'e>, Error = Error<'e>>,
        {
            let n_operands = op.num_operands();

            if f64_cache.len() >= n_operands {
                // const folding
                let output_len = output.len();
                let f64_cache_len = f64_cache.len();

                let start = f64_cache_len - n_operands;
                let num = op.apply(&f64_cache[start..]);

                let token: T = InfixToken::Num(num).try_into()?;

                output.truncate(output_len - n_operands + 1);
                output[output_len - n_operands] = token;

                f64_cache.truncate(f64_cache_len - n_operands + 1);
                f64_cache[f64_cache_len - n_operands] = num;
            } else {
                let token: T = InfixToken::Op(op).try_into()?;
                output.push(token);
                f64_cache.clear();
            }

            Ok(())
        }
    }
}
