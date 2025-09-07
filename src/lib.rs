//! # Fast Expression Evaluators
//!
//! The fastest expression evaluators
//!
//! ## Introduction
//!
//! ```rust
//! use fee::{prelude::*, DefaultResolver};
//!
//! let mut var_resolver = DefaultResolver::empty();
//! var_resolver.insert("p0", 10.0);
//! var_resolver.insert("p1", 4.0);
//!
//! let mut fn_resolver = DefaultResolver::empty();
//! fn_resolver.insert("abs", ExprFn::new(abs));
//!
//! let context = Context::new(var_resolver, fn_resolver);
//! let mut stack = Vec::with_capacity(10);
//!
//! let expr = Expr::compile("abs((2 + 4) * 6 / (p1 + 2)) + abs(-2)", &context).unwrap();
//! let result = expr.eval(&context, &mut stack).unwrap();
//! assert_eq!(result, 8.0);
//!
//! fn abs(x: &[f64]) -> f64 {
//!    x[0].abs()
//! }
//! ```
//!
//! ## Expression
//! A generic struct representing a mathematical expression.  
//! Use [`Expr::<T>::compile`] to parse a string into a specialized [`Expr<T>`] depending
//! on the provided [`Context`].
//!
//! ## Context
//! A struct that holds resolvers for variables and functions used in an expression.  
//! Contexts can be **locked** to prevent reallocation of inner resolvers, allowing
//! expressions to be evaluated using raw pointers instead of name lookups for maximum performance.
//!
//! ## Resolvers
//! A [`Resolver`] maps variable or function names to their values/implementations.  
//! Available resolvers include:
//!
//! - [`DefaultResolver`]: No size or naming restrictions, but slower than specialized resolvers.  
//! - [`IndexedResolver`]: No size restrictions, but requires specific naming patterns. Very fast.  
//! - [`SmallResolver`]: Restricted size, but allows arbitrary names with good performance.  
//! - [`ConstantResolver`]: Always resolves to the same value; offers the best performance.  
//! - [`EmptyResolver`]: Always resolves to `None`; useful for expressions without variables or functions.  

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
