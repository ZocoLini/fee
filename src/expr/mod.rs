mod lexer;

pub mod ifrpn;
pub mod irpn;
pub mod ivrpn;
pub mod lrpn;
pub mod rpn;

use std::borrow::Borrow;
use std::hash::Hash;

use crate::resolver::ResolverState;
use crate::{
    ConstantResolver, DefaultResolver, EmptyResolver, Error, SmallResolver, context::Context,
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Op
{
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Neg,
    Mod,
}

impl Op
{
    fn precedence(&self) -> u8
    {
        match self {
            Op::Add | Op::Sub => 10,
            Op::Mul | Op::Div | Op::Mod => 20,
            Op::Neg => 30,
            Op::Pow => 40,
        }
    }

    fn num_operands(&self) -> usize
    {
        match self {
            Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Pow | Op::Mod => 2,
            Op::Neg => 1,
        }
    }

    fn is_right_associative(&self) -> bool
    {
        matches!(self, Op::Pow)
    }

    fn apply(&self, x: &[f64]) -> f64
    {
        match self {
            Op::Add => x[0] + x[1],
            Op::Sub => x[0] - x[1],
            Op::Mul => x[0] * x[1],
            Op::Div => x[0] / x[1],
            Op::Pow => {
                if x[1] == x[1] as i64 as f64 {
                    x[0].powi(x[1] as i32)
                } else {
                    x[0].powf(x[1])
                }
            }
            Op::Neg => -x[0],
            Op::Mod => x[0] % x[1],
        }
    }
}

/// Represents the compiled expression.
///
/// This struct should be built by using the [`Expr::compile`] method. This
/// method automatically returns the best available representation of the expression
/// depending on the context you will be using it with.
///
/// After compilation, the expression can be evaluated using the [`Expr::eval`] method.
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
impl<S: ResolverState, K: Borrow<str> + PartialEq<String> + Eq + Hash, T> NotIndexedResolver
    for DefaultResolver<S, K, T>
{
}
impl<S: ResolverState, T> NotIndexedResolver for ConstantResolver<S, T> {}
impl<S: ResolverState, K: AsRef<str> + Eq, T> NotIndexedResolver for SmallResolver<S, K, T> {}
impl<S: ResolverState> NotIndexedResolver for EmptyResolver<S> {}

trait ParseableToken<'a, 'c, S, V, F, LV, LF>
where
    S: ResolverState,
{
    fn num(num: f64) -> Self;
    fn op(op: Op) -> Self;
    fn var(name: &'a str, ctx: &'c Context<S, V, F, LV, LF>) -> Self;
    fn fun(name: &'a str, argc: usize, ctx: &'c Context<S, V, F, LV, LF>) -> Self;
}

pub trait ExprCompiler<'e, 'c, S, V, F, LV, LF, T>
where
    S: ResolverState,
{
    fn compile(expr: &'e str, ctx: &'c Context<S, V, F, LV, LF>) -> Result<Expr<T>, Error<'e>>;
}

pub trait ExprEvaluator<'e, S, V, F, LV, LF>
where
    S: ResolverState,
{
    fn eval(&self, ctx: &Context<S, V, F, LV, LF>, stack: &mut Vec<f64>) -> Result<f64, Error<'e>>;
}
