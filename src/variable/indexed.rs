use crate::prelude::VarResolver;

pub struct IndexedVarResolver
{
    vars: Vec<Vec<f64>>,
}

const ALPHABET_SIZE: usize = ('z' as u8 - 'a' as u8 + 1) as usize;
const ALPHABET_START_U8: u8 = 'a' as u8;
const ALPHABET_START_USIZE: usize = 'a' as usize;

impl VarResolver for IndexedVarResolver
{
    fn get(&self, name: &str) -> Option<f64>
    {
        let letter = (name.chars().next().unwrap() as u8 - ALPHABET_START_U8) as usize;
        let idx = name[1..].parse::<usize>().ok()?;
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
