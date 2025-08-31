use crate::prelude::VarResolver;

const CACHE_SIZE: usize = 10; // 30 is the 'limit'

pub struct SmallVarResolver
{
    cache: Vec<(String, f64)>,
}

impl SmallVarResolver
{
    pub fn new() -> Self
    {
        Self {
            cache: Vec::with_capacity(CACHE_SIZE),
        }
    }

    pub fn add_var(&mut self, name: String, value: f64)
    {
        self.cache.push((name, value));
    }
}

impl VarResolver for SmallVarResolver
{
    fn get_var(&self, name: &str) -> Option<&f64>
    {
        for (key, value) in &self.cache {
            if key == name {
                return Some(value);
            }
        }
        None
    }
}
