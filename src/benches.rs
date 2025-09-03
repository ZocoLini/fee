#![cfg(feature = "bench-internal")]

use crate::{Error, expr::IRpnExpr, expr::RpnExpr, expr::infix::InfixExpr};

pub fn parse_infix<'e>(expr: &'e str) -> Result<InfixExpr<'e>, Error<'e>>
{
    InfixExpr::try_from(expr)
}

pub fn parse_rpn<'e>(expr: &'e str) -> Result<RpnExpr<'e>, Error<'e>>
{
    RpnExpr::try_from(expr)
}

pub fn parse_irpn<'e>(expr: &'e str) -> Result<IRpnExpr, Error<'e>>
{
    IRpnExpr::try_from(expr)
}
