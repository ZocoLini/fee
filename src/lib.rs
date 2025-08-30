#![allow(unused)]

mod context;
mod error;
mod function;
mod token;
mod variable;
mod evaluator;
mod lexer;

#[cfg(feature = "bench-internal")] pub mod benches;

pub mod prelude;

pub use crate::error::*;

pub use crate::context::DefaultContext;

pub use crate::function::DefaultFnResolver;

pub use crate::variable::{DefaultVarResolver, IndexedVarResolver};

pub use crate::evaluator::RPNEvaluator;

pub struct Locked;
pub struct Unlocked;
