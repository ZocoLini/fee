use crate::ExprFn;

use super::Resolver;

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
