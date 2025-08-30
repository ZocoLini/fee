use core::fmt;

use crate::lexer::{Expr, Infix};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator
{
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl Operator
{
    pub fn precedence(&self) -> u8
    {
        match self {
            Operator::Add | Operator::Sub => 1,
            Operator::Mul | Operator::Div => 2,
            Operator::Pow => 3,
        }
    }

    pub fn is_right_associative(&self) -> bool
    {
        matches!(self, Operator::Pow)
    }

    pub fn apply(&self, lhs: f64, rhs: f64) -> f64
    {
        match self {
            Operator::Add => lhs + rhs,
            Operator::Sub => lhs - rhs,
            Operator::Mul => lhs * rhs,
            Operator::Div => lhs / rhs,
            Operator::Pow => lhs.powf(rhs),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'e>
{
    Number(f64),
    Variable(&'e str),
    FunctionCall(&'e str, usize),
    Function(&'e str, Vec<Expr<'e, Infix>>),
    Operator(Operator),
    LParen,
    RParen,
}
