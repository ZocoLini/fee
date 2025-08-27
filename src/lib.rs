#![allow(unused)]

pub mod prelude;
mod error;
mod evaluator;
mod function;
mod variable;

pub use crate::error::Error;

pub struct Locked;
pub struct Unlocked;