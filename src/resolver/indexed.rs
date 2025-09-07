use crate::{
    parsing,
    resolver::{Locked, LockedResolver, ResolverState, Unlocked, UnlockedResolver},
};

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
/// - High performance due to indexed lookup.
/// - Unlimited storage capacity.
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
/// use fee::{EmptyResolver, IndexedResolver};
///
/// let y0_expr = "y0 * (p0 - p1*y1)";
/// let y1_expr = "-y1 * (p2 - p3*y0)";
///
/// let mut var_resolver = IndexedResolver::new();
/// var_resolver.add_id('y', 2);
/// var_resolver.set('y', 0, 1.0);
/// var_resolver.set('y', 1, 2.0);
/// var_resolver.add_id('p', 4);
/// var_resolver.set('p', 0, 1.0);
/// var_resolver.set('p', 1, 0.0);
/// var_resolver.set('p', 2, 1.0);
/// var_resolver.set('p', 3, 0.0);
///
/// let context = Context::new(var_resolver, EmptyResolver::new());
/// let mut stack = Vec::new();
///
/// let y0_expr = Expr::compile(y0_expr, &context).unwrap();
/// let y1_expr = Expr::compile(y1_expr, &context).unwrap();
///
/// assert_eq!(y0_expr.eval(&context, &mut stack), Ok(1.0));
/// assert_eq!(y1_expr.eval(&context, &mut stack), Ok(-2.0));
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
    pub fn set(&mut self, id: char, index: usize, value: T)
    {
        self.vars[id as usize - ALPHABET_START_USIZE][index] = value;
    }

    pub(crate) fn get(&self, id: usize, index: usize) -> Option<&T>
    {
        self.vars[id].get(index)
    }
}

impl<T: Default + Clone> IndexedResolver<Unlocked, T>
{
    pub fn new() -> Self
    {
        Self {
            vars: vec![vec![]; ALPHABET_SIZE],
            _state: Unlocked,
        }
    }

    pub fn add_id(&mut self, id: char, len: usize)
    {
        self.vars[id as usize - ALPHABET_START_USIZE] = vec![T::default(); len]
    }
}
