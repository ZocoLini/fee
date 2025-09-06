//! Fast Expression Evaluators
//!
//! This Rust crate provides ways to evaluate mathematical expressions efficiently.
//! Each evaluator and resolver has his own pros, cons and capabilities.
//!
//! # Introduction
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
//! # Evaluators
//!
//! Trait implemented by all the structs that can evaluate expressions.
//! It needs the expression and a context struct to do the evaluation.
//! The current available Evaluators are:
//! - RpnEvaluator
//!
//! # Resolvers
//!
//! The Resolver trait is implemented by all the objects that can resolve a variable or function name.
//! The current available Resolvers are:
//! - DefaultResolver
//! - IndexedResolver
//! - SmallResolver
//! - ConstantResolver
//! - EmptyResolver
//!
//! Each of which has its own pros and cons.

#![forbid(clippy::unwrap_used)]

mod context;
mod error;
mod expr;
mod op;
mod parsing;
mod resolver;

pub mod prelude;

use std::ops::Deref;

pub use crate::error::*;
pub use crate::resolver::{
    ConstantResolver, DefaultResolver, EmptyResolver, IndexedResolver, Ptr, SmallResolver,
};

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExprFn(fn(&[f64]) -> f64);

impl ExprFn
{
    pub fn new(f: fn(&[f64]) -> f64) -> Self
    {
        ExprFn(f)
    }
}

impl Deref for ExprFn
{
    type Target = fn(&[f64]) -> f64;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl Default for ExprFn
{
    fn default() -> Self
    {
        fn identity(_: &[f64]) -> f64
        {
            0.0
        }
        ExprFn(identity)
    }
}
