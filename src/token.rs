use crate::lexer::InfixExpr;

#[derive(Debug, PartialEq, Eq)]
pub enum Op
{
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl Op
{
    pub fn precedence(&self) -> u8
    {
        match self {
            Op::Add | Op::Sub => 1,
            Op::Mul | Op::Div => 2,
            Op::Pow => 3,
        }
    }

    pub fn is_right_associative(&self) -> bool
    {
        matches!(self, Op::Pow)
    }

    pub fn apply(&self, lhs: f64, rhs: f64) -> f64
    {
        match self {
            Op::Add => lhs + rhs,
            Op::Sub => lhs - rhs,
            Op::Mul => lhs * rhs,
            Op::Div => lhs / rhs,
            Op::Pow => lhs.powf(rhs),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum InfixToken<'e>
{
    Num(f64),
    Var(&'e str),
    Fn(&'e str, Vec<InfixExpr<'e>>),
    Op(Op),
    LParen,
    RParen,
}
