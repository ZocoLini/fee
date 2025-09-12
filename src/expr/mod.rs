mod lexer;

pub mod ifrpn;
pub mod irpn;
pub mod ivrpn;
pub mod lrpn;
pub mod rpn;

use std::borrow::Borrow;
use std::hash::Hash;

use crate::ExprFn;
use crate::prelude::Resolver;
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
    Mod,

    Neg,
    Not,

    Or,
    And,

    Low,
    Great,
    LowEq,
    GreatEq,
    Eq,
    NotEq,

    BitAnd,
    BitOr,
    BitXor,

    Shl,
    Shr,
}

impl Op
{
    #[inline]
    fn precedence(&self) -> u8
    {
        match self {
            Op::Or => 0,
            Op::And => 1,
            Op::Low | Op::Great | Op::LowEq | Op::GreatEq | Op::Eq | Op::NotEq => 2,
            Op::BitAnd | Op::BitOr | Op::BitXor => 3,
            Op::Shl | Op::Shr => 4,
            Op::Add | Op::Sub => 5,
            Op::Mul | Op::Div | Op::Mod => 6,
            Op::Neg | Op::Not => 7,
            Op::Pow => 8,
        }
    }

    #[inline]
    fn num_operands(&self) -> usize
    {
        match self {
            Op::Neg | Op::Not => 1,
            _ => 2,
        }
    }

    #[inline]
    fn is_right_associative(&self) -> bool
    {
        *self == Op::Pow
    }

    #[inline]
    fn apply(&self, x: &[f64]) -> f64
    {
        match self {
            Op::Add => x[0] + x[1],
            Op::Sub => x[0] - x[1],
            Op::Mul => x[0] * x[1],
            Op::Div => x[0] / x[1],
            Op::Pow => {
                if f64_is_i64(x[1]) {
                    x[0].powi(x[1] as i32)
                } else {
                    x[0].powf(x[1])
                }
            }
            Op::Mod => x[0] % x[1],

            Op::Neg => -x[0],
            Op::Not => bool_to_f64(!f64_to_bool(x[0])),

            Op::Or => bool_to_f64(f64_to_bool(x[0]) || f64_to_bool(x[1])),
            Op::And => bool_to_f64(f64_to_bool(x[0]) && f64_to_bool(x[1])),

            Op::Low => bool_to_f64(x[0] < x[1]),
            Op::Great => bool_to_f64(x[0] > x[1]),
            Op::LowEq => bool_to_f64(x[0] <= x[1]),
            Op::GreatEq => bool_to_f64(x[0] >= x[1]),
            Op::Eq => bool_to_f64(x[0] == x[1]),
            Op::NotEq => bool_to_f64(x[0] != x[1]),

            Op::BitAnd => ((x[0] as i64) & (x[1] as i64)) as f64,
            Op::BitOr => ((x[0] as i64) | (x[1] as i64)) as f64,
            Op::BitXor => ((x[0] as i64) ^ (x[1] as i64)) as f64,

            Op::Shl => ((x[0] as i64) << (x[1] as i64)) as f64,
            Op::Shr => ((x[0] as i64) >> (x[1] as i64)) as f64,
        }
    }
}

#[inline]
fn f64_is_i64(num: f64) -> bool
{
    num == num as i64 as f64
}

#[inline]
fn f64_to_bool(num: f64) -> bool
{
    num != 0.0
}

#[inline]
fn bool_to_f64(a: bool) -> f64
{
    if a { 1.0 } else { 0.0 }
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

#[allow(unused)]
trait ParseableToken<'a, 'c, S, V, F, LV, LF>
where
    S: ResolverState,
    V: Resolver<S, f64>,
    F: Resolver<S, ExprFn>,
{
    fn f64(num: f64) -> Self;
    fn i64(num: i64) -> Self;
    fn bool(val: bool) -> Self;
    fn op(op: Op) -> Self;
    fn var(name: &'a str, ctx: &'c Context<S, V, F, LV, LF>) -> Self;
    fn fun(name: &'a str, argc: usize, ctx: &'c Context<S, V, F, LV, LF>) -> Self;
}

pub trait ExprCompiler<'e, 'c, S, V, F, LV, LF, T>
where
    S: ResolverState,
    V: Resolver<S, f64>,
    F: Resolver<S, ExprFn>,
{
    fn compile(expr: &'e str, ctx: &'c Context<S, V, F, LV, LF>) -> Result<Expr<T>, Error<'e>>;
}

pub trait ExprEvaluator<'e, S, V, F, LV, LF>
where
    S: ResolverState,
    V: Resolver<S, f64>,
    F: Resolver<S, ExprFn>,
{
    fn eval(&self, ctx: &Context<S, V, F, LV, LF>, stack: &mut Vec<f64>) -> Result<f64, Error<'e>>;
}
