#![allow(unused)]

mod error;
mod evaluator;
mod function;
pub mod prelude;
mod variable;

pub use crate::error::*;

pub use crate::variable::{DefaultVarResolver, IndexedVarResolver};

pub use crate::function::DefaultFnResolver;

pub use crate::evaluator::RPNEvaluator;

pub struct Locked;
pub struct Unlocked;
