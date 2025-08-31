#![cfg(feature = "bench-internal")]

use crate::evaluator::rpn::RPN;
use crate::{
    Error,
    lexer::{Expr, Infix},
};

pub fn parse_infix<'e>(expr: &'e str) -> Result<Expr<'e, Infix>, Error<'e>>
{
    Expr::try_from(expr)
}

pub fn parse_rpn<'e>(expr: Expr<'e, Infix>) -> Result<Expr<'e, RPN>, Error<'e>>
{
    Expr::try_from(expr)
}
