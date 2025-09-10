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
