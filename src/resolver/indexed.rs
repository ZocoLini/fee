use crate::ExprFn;

use super::Resolver;

/// High-performance resolver with O(1) lookup for variables and functions.
///
/// `IndexedResolver` is designed for cases where the number of variables is large and
/// the variable naming follows a strict convention: a single English letter followed
/// by a numeric index (e.g., `"a0"`, `"b1"`). The letter represents the variable
/// identifier and the number its index in the internal storage.
///
/// This resolver significantly outperforms other resolvers due to its flat
/// vector-of-vectors storage. The trade-off is the restricted naming convention.
///
/// # Advantages
/// - High performance due to O(1) lookup.
/// - Efficient memory usage.
/// - Fast variable and function resolution.
///
/// # Disadvantages
/// - Limited naming convention.
///
/// # Panics
/// The `resolve()` method will panic if:
/// - The variable name does not follow the expected `"letter + number"` format.
/// - The letter or index is out of bounds of the internal storage.
///
/// # Examples
/// ```rust
/// use fee::prelude::*;
/// use fee::{EmptyResolver, IndexedResolver, RpnEvaluator};
///
/// // Lotka-Volterra model
/// let y0_dot_expr = "y0 * (p0 - p1*y1)";
/// let y1_dot_expr = "-y1 * (p2 - p3*y0)";
///
/// let mut var_resolver = IndexedResolver::new_var_resolver();
/// var_resolver.add_var_identifier('y', 2);
/// var_resolver.set('y', 0, 1.0);
/// var_resolver.set('y', 1, 2.0);
/// var_resolver.add_var_identifier('p', 4);
/// var_resolver.set('p', 0, 1.0);
/// var_resolver.set('p', 1, 0.0);
/// var_resolver.set('p', 2, 1.0);
/// var_resolver.set('p', 3, 0.0);
///
/// let context = Context::new(var_resolver, EmptyResolver::new());
/// let y0_eval = RpnEvaluator::new(y0_dot_expr).unwrap();
/// let y1_eval = RpnEvaluator::new(y1_dot_expr).unwrap();
///
/// assert_eq!(y0_eval.eval(&context), Ok(1.0));
/// assert_eq!(y1_eval.eval(&context), Ok(-2.0));
/// ```
pub struct IndexedResolver<T>
{
    vars: Vec<Vec<T>>,
}

const ALPHABET_SIZE: usize = (b'z' - b'a' + 1) as usize;
const ALPHABET_START_USIZE: usize = b'a' as usize;

impl<T> Resolver<T> for IndexedResolver<T>
{
    #[inline(always)]
    fn resolve(&self, name: &str) -> Option<&T>
    {
        let name_bytes = name.as_bytes();

        let letter = name_bytes[0] as usize - ALPHABET_START_USIZE;
        let idx = str_to_usize(&name_bytes[1..]);
        Some(&self.vars[letter][idx])
    }
}

impl<T> IndexedResolver<T>
{
    pub fn set(&mut self, identifier: char, index: usize, value: T)
    {
        self.vars[identifier as usize - ALPHABET_START_USIZE][index] = value;
    }
}

impl IndexedResolver<f64>
{
    pub fn new_var_resolver() -> Self
    {
        Self {
            vars: vec![vec![]; ALPHABET_SIZE],
        }
    }

    pub fn add_var_identifier(&mut self, identifier: char, len: usize)
    {
        self.vars[identifier as usize - ALPHABET_START_USIZE] = vec![0.0; len]
    }
}

impl IndexedResolver<ExprFn>
{
    pub fn new_fn_resolver() -> Self
    {
        Self {
            vars: vec![vec![]; ALPHABET_SIZE],
        }
    }

    pub fn add_fn_identifier(&mut self, identifier: char, len: usize)
    {
        self.vars[identifier as usize - ALPHABET_START_USIZE] = vec![|_| { 0.0 }; len]
    }
}

fn str_to_usize(s: &[u8]) -> usize
{
    let mut result = 0;

    for &byte in s {
        result = result * 10 + (byte - b'0');
    }

    result as usize
}
