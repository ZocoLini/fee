use crate::prelude::VarResolver;

pub struct IndexedVarResolver
{
    vars: Vec<f64>,
}

impl VarResolver for IndexedVarResolver
{
    fn get(&self, name: &str) -> Option<f64>
    {
        let idx = name[1..].parse::<usize>().ok()? - 1;
        self.vars.get(idx).copied()
    }
}

impl IndexedVarResolver
{
    pub fn new(len: usize) -> Self
    {
        Self {
            vars: vec![0.0; len],
        }
    }

    pub fn set(&mut self, index: usize, value: f64)
    {
        if index < self.vars.len() {
            self.vars[index] = value;
        }
    }
}
