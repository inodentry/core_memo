//! # Simple Memoization Library
//!
//! `core_memo` is a simple, straightforward, zero-cost library for lazy
//! evaluation and memoization. It does not do memory allocations or dynamic
//! dispatch and it is `#![no_std]`-compatible.
//!
//! You must define a custom type to represent your computation and implement
//! the `Memoize` trait for it. Then, you can use it with the `Memo`, `MemoExt`,
//! or `MemoOnce` types to lazily evaluate and cache the value.
//!
//! Here is an example:
//!
//! ```
//! use core_memo::{Memoize, Memo, MemoExt};
//!
//! #[derive(Debug, PartialEq, Eq)] // for assert_eq! later
//! struct MemoSum(i32);
//!
//! impl Memoize for MemoSum {
//!     type Param = [i32];
//!
//!     fn memoize(p: &[i32]) -> MemoSum {
//!         MemoSum(p.iter().sum())
//!     }
//! }
//!
//!
//! // The `Memo` type holds ownership over the parameter for the calculation
//!
//! let mut memo: Memo<MemoSum, _> = Memo::new(vec![1, 2]);
//!
//! // Our `memoize` method is called the first time we call `memo.get()`
//! assert_eq!(memo.get(), &MemoSum(3));
//!
//! // Further calls to `memo.get()` return the cached value without reevaluating
//! assert_eq!(memo.get(), &MemoSum(3));
//!
//!
//! // We can mutate the parameter held inside the `Memo`:
//!
//! // via a mutable reference
//! memo.param_mut().push(3);
//! // via a closure
//! memo.update_param(|p| p.push(4));
//!
//! // either way, the `Memo` forgets any cached value and it will be
//! // reevaluated on the next call to `memo.get()`
//!
//! assert_eq!(memo.get(), &MemoSum(10)); // the vec is now `[1, 2, 3, 4]`
//! ```
//!
//! There are 3 different wrapper types: `Memo`, `MemoExt`, and `MemoOnce`.
//!
//!   - `Memo` contains / holds ownership over the parameter for the computation.
//!     This makes it the easiest and safest to use, but could limit your
//!     flexibility in more complex scenarios.
//!
//!   - `MemoExt` does not keep track of the parameter. This means that you have
//!     to provide it externally with every call to `get()` and manually call
//!     `clear()` whenever the value needs to be reevaluated. This makes it more
//!     cumbersome and error-prone, but gives you full flexibility to manage the
//!     input parameter however you want.
//!
//!   - `MemoOnce` holds a shared reference to the parameter. This lets you
//!     manage the parameter externally, but you cannot mutate it as long as the
//!     `MemoOnce` is alive. This could be useful for one-off computations.
//!
//! ## Implementation Notes
//!
//! ### Why do the types not implement `Deref`/`DerefMut`?
//!
//! `Deref` takes `&self`, so it cannot mutate the object to cache the result
//! of the computation. `DerefMut` requires `Deref`.
//!
//! Also, while such implementations would save some typing by making the use
//! implicit, I believe this is undesirable. It is against the spirit of Rust to
//! make potentially-expensive computations implicit and hide them. This is why
//! `.clone()` is explicit and why `Cell`/`RefCell` need explicit method calls.
//!
//! ### Why not use interior mutability?
//!
//! The design of this library follows KISS principles. It should be simple,
//! straightforward, composable. No hidden magic. No surprises.
//!
//! You are free to wrap these objects in whatever you like if you need to use
//! them in an immutable context.
//!
//! The current design of the library makes it as widely-useful as possible.

#![no_std]

// enable std when testing
#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests;

use core::borrow::Borrow;

/// Represents a computation that is to be memoized
///
/// To use this library, you should define a custom type representing the output
/// of your computation and implement this trait on it to specify how to compute
/// it. You then wrap your custom type in a `Memo`, `MemoExt`, or `MemoOnce`,
/// depending on the semantics you need.
///
/// ## Notes
///
/// If you need more that one input parameter for your computation, you can use
/// a tuple for the `Param` type or define a custom type for it.
///
/// If your computation does not use any input, `Param` can be `()` or `!`.
///
/// `Param` can also be an unsized type like `str` or `[T]` to work on slices.
///
/// ## Example
///
/// ```
/// use core_memo::{Memoize, Memo};
///
/// #[derive(Debug, PartialEq, Eq)]
/// struct Repeater(String);
///
/// impl Memoize for Repeater {
///     type Param = (String, usize);
///
///     fn memoize(p: &Self::Param) -> Self {
///         let (string, count) = p;
///         let mut r = String::new();
///         for _i in 0..*count {
///             r.push_str(&string);
///         }
///         Repeater(r)
///     }
/// }
///
/// let mut memo: Memo<Repeater, _> = Memo::new(("abc".into(), 3));
///
/// assert_eq!(memo.get().0, "abcabcabc");
/// ```
///
pub trait Memoize {
    type Param: ?Sized;

    fn memoize(p: &Self::Param) -> Self;
}

/// Memoized value with a parameter provided externally
///
/// See the crate-level documentation for information how to use the library.
///
/// This is the simplest memoization type, but the trickiest to use.
///
/// This type does not keep track of the parameter for the computation and
/// requires you to provide it externally with every call to `get()`.
///
/// You need to make sure to manually call `clear()` to invalidate the cached
/// output whenever you need it to be reevaluated (typically after mutating
/// the input parameter). You probably also want to make sure that you provide
/// the same parameter every time you call `get()`.
///
/// It is very easy to introduce logic bugs in your program with this type. You
/// should prefer the `Memo` or `MemoOnce` types, unless you need the extra
/// flexibility provided by `MemoExt`.
///
/// ## Example
///
/// ```
/// use core_memo::{Memoize, MemoExt};
///
/// // something trivial for the sake of example
/// struct CopyInt(i32);
///
/// impl Memoize for CopyInt {
///     type Param = i32;
///     fn memoize(p: &i32) -> Self {
///         CopyInt(*p)
///     }
/// }
///
/// // notice we do not provide a param to `new()`:
/// let mut memo: MemoExt<CopyInt> = MemoExt::new();
///
/// let param = 420;
///
/// // we have to manually provide the param with each call to get
/// assert_eq!(memo.get(&param).0, 420);
///
/// let meaning_of_life = 42;
///
/// // we have the freedom to do whatever we want with the param and
/// // provide anything to the `MemoExt`:
///
/// println!("The meaning of life is {}.", memo.get(&meaning_of_life).0);
///
/// // ^ WHOOPS: This actually prints "The meaning of life is 420." :D
/// // This is because we haven't called `clear()` to invalidate the previous
/// // cached value! This could be a bug in your program!
///
/// memo.clear();
///
/// // now our computation will be reevaluated:
/// assert_eq!(memo.get(&meaning_of_life).0, 42);
///
/// ```
///
#[derive(Debug)]
pub struct MemoExt<T: Memoize> {
    value: Option<T>,
}

/// Memoized value which holds ownership over the parameter for its computation
///
/// See the crate-level documentation for information how to use the library.
///
/// This type holds ownership over the input parameter to your computation. It
/// keeps everything nicely together and is the safest to use. If this is too
/// restrictive for you, consider using `MemoExt` instead.
///
/// You can modify the parameter using `param_mut()` or `update_param()`. Any
/// cached value will be cleared and will be recomputed on the next access.
///
/// ## Example
///
/// See the crate-level documentation for an example.
///
#[derive(Debug)]
pub struct Memo<T: Memoize, P: Borrow<T::Param> = <T as Memoize>::Param> {
    value: Option<T>,
    param: P,
}

/// Memoized value which holds a reference to the parameter for its computation
///
/// See the crate-level documentation for information how to use the library.
///
/// This type is designed for one-shot lazy computations. It holds a reference
/// to the input parameter for the computation, meaning that it cannot be
/// mutated while the `MemoOnce` is alive.
///
/// ## Example
///
/// ```
/// use core_memo::{Memoize, MemoOnce};
///
/// // something trivial for the sake of example
/// struct MemoLength(usize);
///
/// impl Memoize for MemoLength {
///     type Param = String;
///
///     fn memoize(p: &String) -> Self {
///         MemoLength(p.len())
///     }
/// }
///
/// let mut my_string = String::from("My length is important!");
///
/// // ... some fancy computations ...
///
/// {
///     // we want to use our hard-to-compute value many times in this block,
///     // so we want to memoize it:
///     let mut len: MemoOnce<MemoLength> = MemoOnce::new(&my_string);
///
///     // ... more fancy computations ...
///
///     assert_eq!(len.get().0, 23);
///
///     // ... more stuff ...
///
///     println!("{}", len.get().0);
/// }
///
/// // now our `MemoOnce` has been dropped, our String is no longer borrowed,
/// // and we are free to mutate it:
/// my_string.push_str(" Not anymore!");
/// ```
///
#[derive(Debug)]
pub struct MemoOnce<'p, T: Memoize>
where
    T::Param: 'p,
{
    value: Option<T>,
    param: &'p T::Param,
}

impl<T: Memoize> MemoExt<T> {
    /// Creates a new `MemoExt` instance
    pub fn new() -> Self {
        Self { value: None }
    }

    /// Clears any cached value
    ///
    /// You must call this whenever it is invalid.
    ///
    /// The value will be reevaluated the next time it is needed.
    pub fn clear(&mut self) {
        self.value = None
    }

    /// Check if there is a cached value
    ///
    /// If this method returns `true`, the next call to `get()` will return a
    /// stored memoized value.
    ///
    /// If this method returns `false`, the next call to `get()` will recompute
    /// the value.
    pub fn is_ready(&self) -> bool {
        self.value.is_some()
    }

    /// If the value is not ready, compute it and cache it
    ///
    /// Call this method if you want to make sure that future `get()` calls can
    /// return instantly without computing the value.
    pub fn ready(&mut self, p: &T::Param) {
        if self.value.is_none() {
            self.value = Some(T::memoize(p));
        }
    }

    /// Force the value to be recomputed
    ///
    /// This discards any stored value and computes a new one immediately.
    ///
    /// It is probably better to call `clear()` instead, to compute the value
    /// lazily when it is next needed.
    pub fn update(&mut self, p: &T::Param) {
        self.value = Some(T::memoize(p));
    }

    /// Get the value
    ///
    /// If the value has already been computed, this function returns the cached
    /// value. If not, it is computed and cached for future use.
    ///
    /// If you need to make sure this method always returns quickly, call
    /// `ready()` beforehand or use `try_get()`.
    pub fn get(&mut self, p: &T::Param) -> &T {
        self.ready(p);
        self.try_get().unwrap()
    }

    /// Get the value if it is available
    ///
    /// If there is a cached value, returns it. If the value needs to be
    /// computed, returns `None`.
    pub fn try_get(&self) -> Option<&T> {
        self.value.as_ref()
    }
}

impl<T: Memoize, P: Borrow<T::Param>> Memo<T, P> {
    /// Creates a new `Memo` instance
    ///
    /// You must pass in the object which will be used as the parameter
    /// for your computation. The `Memo` will take ownership over it.
    pub fn new(p: P) -> Self {
        Self {
            value: None,
            param: p,
        }
    }

    /// Clears any cached value
    ///
    /// The value will be reevaluated the next time it is needed.
    pub fn clear(&mut self) {
        self.value = None
    }

    /// Check if there is a cached value
    ///
    /// If this method returns `true`, the next call to `get()` will return a
    /// stored memoized value.
    ///
    /// If this method returns `false`, the next call to `get()` will recompute
    /// the value.
    pub fn is_ready(&self) -> bool {
        self.value.is_some()
    }

    /// If the value is not ready, compute it and cache it
    ///
    /// Call this method if you want to make sure that future `get()` calls can
    /// return instantly without computing the value.
    pub fn ready(&mut self) {
        if self.value.is_none() {
            self.value = Some(T::memoize(self.param.borrow()));
        }
    }

    /// Force the value to be recomputed
    ///
    /// This discards any stored value and computes a new one immediately.
    ///
    /// It is probably better to call `clear()` instead, to compute the value
    /// lazily when it is next needed.
    pub fn update(&mut self) {
        self.value = Some(T::memoize(self.param.borrow()));
    }

    /// Get the value
    ///
    /// If the value has already been computed, this function returns the cached
    /// value. If not, it is computed and cached for future use.
    ///
    /// If you need to make sure this method always returns quickly, call
    /// `ready()` beforehand or use `try_get()`.
    pub fn get(&mut self) -> &T {
        self.ready();
        self.try_get().unwrap()
    }

    /// Get the value if it is available
    ///
    /// If there is a cached value, returns it. If the value needs to be
    /// computed, returns `None`.
    pub fn try_get(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Get a reference to the parameter used for the computation
    pub fn param(&self) -> &P {
        &self.param
    }

    /// Get a mutable reference to the parameter used for the computation
    ///
    /// This clears any cached value.
    pub fn param_mut(&mut self) -> &mut P {
        self.clear();
        &mut self.param
    }

    /// Modify the parameter used for the computation
    ///
    /// Takes a closure and applies it to the parameter.
    ///
    /// This clears any cached value.
    pub fn update_param<F>(&mut self, op: F)
    where
        F: FnOnce(&mut P),
    {
        self.clear();
        op(&mut self.param);
    }
}

impl<'p, T: Memoize> MemoOnce<'p, T> {
    /// Creates a new `MemoOnce` instance
    ///
    /// You must pass a reference to the object which will be used as the
    /// parameter for your computation.
    pub fn new(p: &'p T::Param) -> Self {
        Self {
            value: None,
            param: p,
        }
    }

    /// Clears any cached value
    ///
    /// The value will be reevaluated the next time it is needed.
    pub fn clear(&mut self) {
        self.value = None
    }

    /// Check if there is a cached value
    ///
    /// If this method returns `true`, the next call to `get()` will return a
    /// stored memoized value.
    ///
    /// If this method returns `false`, the next call to `get()` will recompute
    /// the value.
    pub fn is_ready(&self) -> bool {
        self.value.is_some()
    }

    /// If the value is not ready, compute it and cache it
    ///
    /// Call this method if you want to make sure that future `get()` calls can
    /// return instantly without computing the value.
    pub fn ready(&mut self) {
        if self.value.is_none() {
            self.value = Some(T::memoize(self.param));
        }
    }

    /// Force the value to be recomputed
    ///
    /// This discards any stored value and computes a new one immediately.
    ///
    /// It is probably better to call `clear()` instead, to compute the value
    /// lazily when it is next needed.
    pub fn update(&mut self) {
        self.value = Some(T::memoize(self.param));
    }

    /// Get the value
    ///
    /// If the value has already been computed, this function returns the cached
    /// value. If not, it is computed and cached for future use.
    ///
    /// If you need to make sure this method always returns quickly, call
    /// `ready()` beforehand or use `try_get()`.
    pub fn get(&mut self) -> &T {
        self.ready();
        self.try_get().unwrap()
    }

    /// Get the value if it is available
    ///
    /// If there is a cached value, returns it. If the value needs to be
    /// computed, returns `None`.
    pub fn try_get(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Get a reference to the parameter used for the computation
    pub fn param(&self) -> &T::Param {
        &self.param
    }
}
