use crate::{ExprFn, context::Context, prelude::Resolver};

pub mod infix;
pub mod rpn;

#[derive(Debug, PartialEq)]
pub struct Expr<Token>
{
    tokens: Vec<Token>,
}

pub trait EvalRpn<V: Resolver<f64>, F: Resolver<ExprFn>>
{
    type Error;

    fn eval(&self, ctx: &Context<V, F>, stack: &mut Vec<f64>) -> Result<f64, Self::Error>;
}

impl<Token> Expr<Token>
{
    pub fn len(&self) -> usize
    {
        self.tokens.len()
    }
}
