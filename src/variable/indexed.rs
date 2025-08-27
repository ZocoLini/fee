use crate::prelude::VarResolver;

pub struct IndexedVarResolver
{
    vars: Vec<Vec<f64>>,
}

const ALPHABET_SIZE: usize = (b'z' - b'a' + 1) as usize;
const ALPHABET_START_USIZE: usize = b'a' as usize;

impl VarResolver for IndexedVarResolver
{
    #[inline(always)]
    fn get(&self, name: &str) -> Option<f64>
    {
        let name_bytes = name.as_bytes();

        let letter = (name_bytes[0] as usize - ALPHABET_START_USIZE);
        let idx = str_to_usize(&name_bytes[1..]);
        Some(self.vars[letter][idx])
    }
}

impl IndexedVarResolver
{
    pub fn new() -> Self
    {
        Self {
            vars: vec![vec![0.0; 0]; ALPHABET_SIZE],
        }
    }

    pub fn add_identifier(&mut self, identifier: char, len: usize)
    {
        self.vars[identifier as usize - ALPHABET_START_USIZE].resize(len, 0.0);
    }

    pub fn set(&mut self, identifier: char, index: usize, value: f64)
    {
        self.vars[identifier as usize - ALPHABET_START_USIZE][index] = value;
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
