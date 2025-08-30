use std::borrow::Cow;

use thiserror::Error;

use crate::{error, token::Token};

#[derive(Debug, Error, PartialEq)]
pub enum Error<'a>
{
    #[error("unexpected token '{0}' at {1}")]
    UnexpectedToken(Cow<'a, str>, usize),

    #[error("parse error: {0}")]
    ParseError(ParseError<'a>),

    #[error("eval error: {0}")]
    EvalError(EvalError<'a>),
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseError<'a>
{
    #[error("unexpected character '{0}' at {1}")]
    UnexpectedChar(Cow<'a, char>, usize),

    #[error("invalid number '{0}' at {1}")]
    InvalidNumber(Cow<'a, str>, usize),
}

#[derive(Debug, Error, PartialEq)]
pub enum EvalError<'a>
{
    #[error("unknown variable '{0}'")]
    UnknownVar(Cow<'a, str>),

    #[error("unknown function '{0}'")]
    UnknownFn(Cow<'a, str>),

    #[error("RPN stack underflow")]
    RPNStackUnderflow,

    #[error("malformed expression")]
    MalformedExpression,
}
