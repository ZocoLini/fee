mod context;
mod error;
mod evaluator;
mod function;
mod lexer;
mod token;
mod variable;

#[cfg(feature = "bench-internal")]
pub mod benches;

pub mod prelude;

pub use crate::error::*;

pub use crate::context::DefaultContext;

pub use crate::function::DefaultFnResolver;

pub use crate::variable::{DefaultVarResolver, IndexedVarResolver, SmallVarResolver};

pub use crate::evaluator::RpnEvaluator;

pub struct Locked;
pub struct Unlocked;
