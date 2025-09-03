use crate::lexer::InfixExpr;

#[derive(Debug, PartialEq, Eq)]
pub enum Op
{
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Neg,
}

impl Op
{
    pub fn precedence(&self) -> u8
    {
        match self {
            Op::Add | Op::Sub => 10,
            Op::Mul | Op::Div => 20,
            Op::Neg => 30,
            Op::Pow => 40,
        }
    }

    pub fn num_operands(&self) -> usize
    {
        match self {
            Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Pow => 2,
            Op::Neg => 1,
        }
    }

    pub fn is_right_associative(&self) -> bool
    {
        matches!(self, Op::Pow)
    }

    pub fn apply(&self, x: &[f64]) -> f64
    {
        match self {
            Op::Add => x[0] + x[1],
            Op::Sub => x[0] - x[1],
            Op::Mul => x[0] * x[1],
            Op::Div => x[0] / x[1],
            Op::Pow => x[0].powf(x[1]),
            Op::Neg => -x[0],
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
