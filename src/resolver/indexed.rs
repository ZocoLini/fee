use crate::{parsing, prelude::*};

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
pub struct IndexedResolver<S: ResolverState, T>
{
    vars: Vec<Vec<T>>,

    _state: S,
}

impl<T> LockedResolver<T> for IndexedResolver<Locked, T> {}
impl<T> UnlockedResolver<T, IndexedResolver<Locked, T>> for IndexedResolver<Unlocked, T>
{
    fn lock(self) -> IndexedResolver<Locked, T>
    {
        IndexedResolver {
            vars: self.vars,
            _state: Locked,
        }
    }
}

const ALPHABET_SIZE: usize = (b'z' - b'a' + 1) as usize;
const ALPHABET_START_USIZE: usize = b'a' as usize;

impl<S, T> Resolver<S, T> for IndexedResolver<S, T>
where
    S: ResolverState,
{
    #[inline(always)]
    fn resolve(&self, name: &str) -> Option<&T>
    {
        let name_bytes = name.as_bytes();

        let letter = name_bytes[0] as usize - ALPHABET_START_USIZE;
        let idx = parsing::parse_usize(&name_bytes[1..]);
        Some(&self.vars[letter][idx])
    }
}

impl<S, T> IndexedResolver<S, T>
where
    S: ResolverState,
{
    pub fn set(&mut self, identifier: char, index: usize, value: T)
    {
        self.vars[identifier as usize - ALPHABET_START_USIZE][index] = value;
    }

    pub(crate) fn get_by_index(&self, identifier: usize, index: usize) -> Option<&T>
    {
        self.vars[identifier].get(index)
    }
}

impl IndexedResolver<Unlocked, f64>
{
    pub fn new_var_resolver() -> Self
    {
        Self {
            vars: vec![vec![]; ALPHABET_SIZE],
            _state: Unlocked,
        }
    }

    // TODO: Use default trait to avoid duplicate implementations
    pub fn add_var_identifier(&mut self, identifier: char, len: usize)
    {
        self.vars[identifier as usize - ALPHABET_START_USIZE] = vec![0.0; len]
    }
}

impl IndexedResolver<Unlocked, ExprFn>
{
    pub fn new_fn_resolver() -> Self
    {
        Self {
            vars: vec![vec![]; ALPHABET_SIZE],
            _state: Unlocked,
        }
    }

    pub fn add_fn_identifier(&mut self, identifier: char, len: usize)
    {
        self.vars[identifier as usize - ALPHABET_START_USIZE] = vec![|_| { 0.0 }; len]
    }
}
