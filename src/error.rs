use std::borrow::Cow;

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error<'a>
{
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
    UnknownVariable(Cow<'a, str>),

    #[error("unknown function '{0}'")]
    UnknownFunction(Cow<'a, str>),
}
