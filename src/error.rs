use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error
{
    #[error("unknown variable: {0}")]
    UnknownVariable(String),
    
    #[error("unknown operator: {0}")]
    UnknownOperator(String),
    
    #[error("unknown function: {0}")]
    UnknownFunction(String),
    
    #[error("unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("invalid number: {0}")]
    InvalidNumber(String),
}
