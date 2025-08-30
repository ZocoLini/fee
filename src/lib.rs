#![allow(unused)]

mod error;
mod evaluator;
mod function;
mod token;
mod variable;
mod context;

#[cfg(feature = "bench-internal")]
pub mod lexer;

#[cfg(not(feature = "bench-internal"))]
mod lexer;

pub mod prelude;

pub use crate::error::*;

pub use crate::function::DefaultFnResolver;

pub use crate::variable::{DefaultVarResolver, IndexedVarResolver};

pub use crate::evaluator::RPNEvaluator;

pub struct Locked;
pub struct Unlocked;
