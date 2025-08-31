//! Fast Expression Evaluators
//!
//! This Rust crate provides ways to evaluate mathematical expressions efficiently.
//! Each evaluator, function and variable resolvers have their own pros, cons and capabilities.
//!
//! ## Introduction
//!
//! ```rust
//! use fee::{prelude::*, DefaultResolver, RpnEvaluator};
//!
//! let mut var_resolver = DefaultResolver::new_empty();
//! var_resolver.insert("p0".to_string(), 10.0);
//! var_resolver.insert("p1".to_string(), 4.0);
//!
//! let mut fn_resolver = DefaultResolver::new_empty();
//! fn_resolver.insert("abs".to_string(), abs as ExprFn);
//!
//! let context = Context::new(var_resolver, fn_resolver);
//!
//! let expr = "abs((2 + 4) * 6 / (p1 + 2)) + abs(-2)";
//! let evaluator = RpnEvaluator::new(expr).unwrap();
//! let result = evaluator.eval(&context).unwrap();
//! assert_eq!(result, 8.0);
//!
//! fn abs(x: &[f64]) -> f64 {
//!    x[0].abs()
//! }
//! ```
//!
//! ## Evaluators
//!
//! Trait implemented by all the structs that can evaluate expressions.
//! Right now the trait is implemented by the RpnEvaluator struct.
//! It needs the expression and a context object to do the evaluation.
//!
//! ## Contexts
//!
//! The context trait defines the interface for a context object that can be used to evaluate expressions.
//! It provides methods for resolving variables and functions.
//! Right now, the trait is implemented by the DefaultContext struct.
//!
//! ## Resolvers
//!
//! The Resolver trait is implemented by all the objects that can resolve a variable or function name.
//! The current available Resolvers are:
//! - DefaultResolver
//! - IndexedResolver
//! - SmallResolver
//! - ConstantResolver
//!
//! Each of which has its own pros and cons.

mod context;
mod error;
mod evaluator;
mod lexer;
mod resolver;
mod token;

#[cfg(feature = "bench-internal")]
pub mod benches;

pub mod prelude;

pub use crate::error::*;

pub use crate::resolver::{
    ConstantResolver, DefaultResolver, EmptyResolver, IndexedResolver, SmallResolver,
};

pub use crate::evaluator::RpnEvaluator;

pub type ExprFn = fn(&[f64]) -> f64;

pub struct Locked;
pub struct Unlocked;
