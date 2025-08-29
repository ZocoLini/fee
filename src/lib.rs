#![allow(unused)]

mod error;
mod evaluator;
mod function;
mod lexer;
mod token;
mod variable;

pub mod prelude;

pub use crate::error::*;

pub use crate::function::DefaultFnResolver;

pub use crate::variable::{DefaultVarResolver, IndexedVarResolver};

pub use crate::evaluator::RPNEvaluator;

pub struct Locked;
pub struct Unlocked;
