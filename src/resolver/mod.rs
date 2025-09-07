use std::{collections::HashMap, marker::PhantomData};

mod constant;
mod default;
mod empty;
mod indexed;
mod small;

pub use constant::ConstantResolver;
pub use default::DefaultResolver;
pub use empty::EmptyResolver;
pub use indexed::IndexedResolver;
pub use small::SmallResolver;

pub trait LockedResolver<T>: Resolver<Locked, T>
{
    fn get_ptr<'a>(&'a self, name: &str) -> Option<Ptr<'a, T>>
    {
        self.resolve(name).map(|v| Ptr {
            ptr: v as *const T as *mut T,
            _marker: std::marker::PhantomData,
        })
    }
}

pub trait UnlockedResolver<T, R: LockedResolver<T>>: Resolver<Unlocked, T>
{
    fn lock(self) -> R;
}

/// Trait representing the state of a resolver.
pub trait ResolverState {}

/// Unit struct representing a locked resolver state. This state means
/// that the resolver cannot be reallocated in memory.
pub struct Locked;
/// Unit struct representing an unlocked resolver state. This state means
/// that the resolver can be reallocated in memory.
pub struct Unlocked;

impl ResolverState for Locked {}
impl ResolverState for Unlocked {}

/// This struct holds a pointer to a value of type `T`. Used by
/// locked resolvers to safely access and modify a value without
/// having to resolve the name.
#[derive(Debug, PartialEq)]
pub struct Ptr<'a, T>
{
    ptr: *mut T,

    _marker: PhantomData<&'a ()>,
}

impl<'a, T> Ptr<'a, T>
where
    T: Copy,
{
    pub fn set(&self, value: T)
    {
        unsafe {
            *self.ptr = value;
        }
    }

    pub fn get(&self) -> T
    {
        unsafe { *self.ptr }
    }
}

/// Trait for resolving values by name.
///
/// This trait can be implemented to resolve any type of value by a string key.
/// Within this crate, it is primarily used for:
/// - Resolving **expression variables** to `f64` values.
/// - Resolving **function names** to [`ExprFn`] (functions taking a slice of `f64` and returning `f64`).
///
/// Implementors of this trait provide the lookup logic, for example:
/// - Using a [`HashMap`] or a small cache.
/// - Providing a custom storage mechanism.
/// - Or using a trivial resolver like [`EmptyResolver`].
pub trait Resolver<State, T>
where
    State: ResolverState,
{
    fn resolve(&self, name: &str) -> Option<&T>;
}

impl<T> Resolver<Unlocked, T> for HashMap<String, T>
{
    fn resolve(&self, name: &str) -> Option<&T>
    {
        self.get(name)
    }
}
