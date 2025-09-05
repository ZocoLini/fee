pub mod ifrpn;
pub mod infix;
pub mod irpn;
pub mod ivrpn;
pub mod lrpn;
pub mod rpn;

use std::borrow::Cow;

pub use ifrpn::IFRpnToken;
pub use irpn::IRpnToken;
pub use ivrpn::IVRpnToken;
pub use lrpn::LRpnToken;
pub use rpn::RpnToken;
use smallvec::{SmallVec, smallvec};

use crate::{
    ConstantResolver, DefaultResolver, EmptyResolver, Error, ExprFn, SmallResolver,
    context::Context,
    expr::infix::InfixToken,
    op::Op,
    prelude::{Locked, LockedResolver, Ptr, Resolver, ResolverState, Unlocked, UnlockedResolver},
};

#[derive(Debug, PartialEq)]
pub struct Expr<Token>
{
    tokens: Vec<Token>,
}

impl<Token> Expr<Token>
{
    pub fn len(&self) -> usize
    {
        self.tokens.len()
    }
}

trait NotIndexedResolver {}
impl<S: ResolverState, T> NotIndexedResolver for DefaultResolver<S, T> {}
impl<T> NotIndexedResolver for ConstantResolver<T> {}
impl<S: ResolverState, T> NotIndexedResolver for SmallResolver<S, T> {}
impl NotIndexedResolver for EmptyResolver {}

pub trait RpnExpr<'e, V, F, T>
where
    T: From<f64> + From<&'e str> + From<Op> + From<(&'e str, usize)>,
    V: Resolver<Unlocked, f64> + UnlockedResolver<f64>,
    F: Resolver<Unlocked, ExprFn> + UnlockedResolver<ExprFn>,
{
    fn compile_unlocked(expr: &'e str, _ctx: &Context<Unlocked, V, F>)
    -> Result<Expr<T>, Error<'e>>
    {
        Expr::try_from(expr)
    }

    fn eval_unlocked(
        &self,
        ctx: &Context<Unlocked, V, F>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'e>>;
}

pub trait LRpnExpr<'e, V, F, T>
where
    T: From<f64> + From<Ptr<'e, f64>> + From<Op> + From<(Ptr<'e, ExprFn>, usize)>,
    V: Resolver<Locked, f64> + LockedResolver<f64>,
    F: Resolver<Locked, ExprFn> + LockedResolver<ExprFn>,
{
    fn compile_locked(expr: &'e str, ctx: &'e Context<Locked, V, F>) -> Result<Expr<T>, Error<'e>>
    {
        Expr::try_from((expr, ctx))
    }

    fn eval_locked(
        &self,
        ctx: &Context<Locked, V, F>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'e>>;
}

impl<'e, T> TryFrom<&'e str> for Expr<T>
where
    T: From<f64> + From<&'e str> + From<Op> + From<(&'e str, usize)>,
{
    type Error = crate::Error<'e>;

    fn try_from(input: &'e str) -> Result<Self, Self::Error>
    {
        let infix_expr = Expr::<InfixToken>::try_from(input)?;
        Expr::<T>::try_from(infix_expr)
    }
}

impl<'e, T, V, F> TryFrom<(&'e str, &'e Context<Locked, V, F>)> for Expr<T>
where
    T: From<f64> + From<Ptr<'e, f64>> + From<Op> + From<(Ptr<'e, ExprFn>, usize)>,
    V: Resolver<Locked, f64> + LockedResolver<f64>,
    F: Resolver<Locked, ExprFn> + LockedResolver<ExprFn>,
{
    type Error = crate::Error<'e>;

    fn try_from((input, ctx): (&'e str, &'e Context<Locked, V, F>)) -> Result<Self, Self::Error>
    {
        let infix_expr = Expr::<InfixToken>::try_from(input)?;
        Expr::<T>::try_from((infix_expr, ctx))
    }
}

impl<'e, T, V, F> TryFrom<(Expr<InfixToken<'e>>, &'e Context<Locked, V, F>)> for Expr<T>
where
    T: From<f64> + From<Ptr<'e, f64>> + From<Op> + From<(Ptr<'e, ExprFn>, usize)>,
    V: Resolver<Locked, f64> + LockedResolver<f64>,
    F: Resolver<Locked, ExprFn> + LockedResolver<ExprFn>,
{
    type Error = Error<'e>;

    // shunting yard algorithm
    fn try_from(
        (expr, ctx): (Expr<InfixToken<'e>>, &'e Context<Locked, V, F>),
    ) -> Result<Self, Self::Error>
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
                    let var_ptr = ctx
                        .vars()
                        .get_ptr(name)
                        .ok_or_else(|| Error::UnknownVar(Cow::Borrowed(name)))?;
                    output.push(T::from(var_ptr));
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
                    let fn_ptr = ctx
                        .fns()
                        .get_ptr(name)
                        .ok_or_else(|| Error::UnknownVar(Cow::Borrowed(name)))?;
                    let fn_token = T::from((fn_ptr, args.len()));

                    for arg_tokens in args {
                        let rpn_arg: Expr<T> = Expr::try_from((arg_tokens, ctx))?;
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

impl<'e, T> TryFrom<Expr<InfixToken<'e>>> for Expr<T>
where
    T: From<f64> + From<&'e str> + From<Op> + From<(&'e str, usize)>,
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
