#![cfg(feature = "bench-internal")]

use crate::{
    Error,
    evaluator::rpn::RpnExpr,
    lexer::{Infix, InfixExpr},
};

pub fn parse_infix<'e>(expr: &'e str) -> Result<InfixExpr<'e>, Error<'e>>
{
    InfixExpr::try_from(expr)
}

pub fn parse_rpn<'e>(expr: InfixExpr<'e>) -> Result<RpnExpr<'e>, Error<'e>>
{
    RpnExpr::try_from(expr)
}
