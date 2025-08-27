#![allow(unused)]

mod error;
mod evaluator;
mod function;
pub mod prelude;
mod variable;

pub use crate::error::Error;

pub struct Locked;
pub struct Unlocked;
