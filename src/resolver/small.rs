use crate::prelude::*;

const CACHE_SIZE: usize = 10; // 30 is the 'limit'

pub struct SmallResolver<State, T>
{
    cache: Vec<(String, T)>,
    _state: State
}

impl<State, T> Resolver<T> for SmallResolver<State, T>
{
    fn resolve(&self, name: &str) -> Option<&T>
    {
        for (key, value) in &self.cache {
            if key == name {
                return Some(value);
            }
        }
        None
    }
}

impl<T> SmallResolver<Unlocked, T>
{
    pub fn new() -> Self
    {
        Self {
            cache: Vec::with_capacity(CACHE_SIZE),
            _state: Unlocked,
        }
    }

    pub fn insert(&mut self, name: String, value: T)
    {
        for (i, (key, _)) in self.cache.iter().enumerate() {
            if *key == name {
                self.cache[i].1 = value;
                return;
            }
        }
        
        self.cache.push((name, value));
    }
    
    pub fn lock(self) -> SmallResolver<Locked, T>
    {
        SmallResolver {
            cache: self.cache,
            _state: Locked,
        }
    }
}

impl SmallResolver<Locked, f64>
{
    pub fn get_var_mut(&mut self, name: &str) -> Option<&mut f64>
    {
        for (key, value) in &mut self.cache {
            if key == name {
                return Some(value);
            }
        }
        None
    }
}
