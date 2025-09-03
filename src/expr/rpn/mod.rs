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
    ConstantResolver, DefaultResolver, EmptyResolver, Error, ExprFn, SmallResolver,
    context::Context,
    expr::{Expr, infix::InfixToken},
    op::Op,
    prelude::Resolver,
};

trait NotIndexedResolver {}
impl<State, T> NotIndexedResolver for DefaultResolver<State, T> {}
impl<T> NotIndexedResolver for ConstantResolver<T> {}
impl<State, T> NotIndexedResolver for SmallResolver<State, T> {}
impl NotIndexedResolver for EmptyResolver {}

pub trait FMLF {}
impl FMLF for RpnToken<'_> {}
impl FMLF for IVRpnToken<'_> {}
impl FMLF for IFRpnToken<'_> {}
impl FMLF for IRpnToken {}

pub trait RpnExpr<'e, V: Resolver<f64>, F: Resolver<ExprFn>, T>
where
    T: From<f64> + From<&'e str> + From<Op> + From<(&'e str, usize)> + FMLF,
{
    type Error;

    fn compile(expr: &'e str, _ctx: &Context<V, F>) -> Result<Expr<T>, Self::Error>;

    fn eval(&self, ctx: &Context<V, F>, stack: &mut Vec<f64>) -> Result<f64, Self::Error>;
}

impl<'e, T> TryFrom<&'e str> for Expr<T>
where
    T: From<f64> + From<&'e str> + From<Op> + From<(&'e str, usize)> + FMLF,
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
    T: From<f64> + From<&'e str> + From<Op> + From<(&'e str, usize)> + FMLF,
{
    type Error = Error<'e>;

    // shunting yard algorithm
    fn try_from(expr: Expr<InfixToken<'e>>) -> Result<Self, Self::Error>
    {
        let mut f64_cache: SmallVec<[f64; 4]> = smallvec![];
        let mut output: Vec<T> = Vec::with_capacity(expr.len());
        let mut ops: Vec<InfixToken> = Vec::new();

        for tok in expr.into_iter() {
            match tok {
                InfixToken::Num(num) => {
                    output.push(num.into());
                    f64_cache.push(num);
                }
                InfixToken::Var(name) => {
                    output.push(name.into());
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
                            _ => unreachable!("no more elements should be inside ops"),
                        }
                    }
                }
                InfixToken::Fn(name, args) => {
                    let fn_token = T::from((name, args.len()));

                    for arg_tokens in args {
                        let rpn_arg: Expr<T> = arg_tokens.try_into()?;
                        output.extend(rpn_arg.tokens);
                    }

                    output.push(fn_token);
                    f64_cache.clear();
                }
            }
        }

        while let Some(InfixToken::Op(op)) = ops.pop() {
            pre_evaluate(&mut output, &mut f64_cache, op);
        }

        debug_assert!(ops.is_empty());

        return Ok(Expr { tokens: output });

        // const folding
        fn pre_evaluate<'e, T>(output: &mut Vec<T>, f64_cache: &mut SmallVec<[f64; 4]>, op: Op)
        where
            T: From<f64> + From<Op>,
        {
            let n_operands = op.num_operands();

            if f64_cache.len() >= n_operands {
                let output_len = output.len();
                let f64_cache_len = f64_cache.len();

                let start = f64_cache_len - n_operands;
                let num = op.apply(&f64_cache[start..]);

                let token: T = num.into();

                output.truncate(output_len - n_operands);
                output.push(token);

                f64_cache.truncate(f64_cache_len - n_operands);
                f64_cache.push(num);
            } else {
                let token: T = op.into();
                output.push(token);
                f64_cache.clear();
            }
        }
    }
}
