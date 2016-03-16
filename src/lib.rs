#![warn(missing_docs)]

//! This crate provides a container that cannot be leaked (e.g. via `std::mem::forget()`) once `lock()`ed: `Lock<'lock, T>`.
//!
//! `Lock::lock()` returns an `Acceptor<'lock, T>`, which allows placing `T` inside the `Lock<'lock, T>`.
//!
//! If the only safe way to create `T` requires an `Acceptor<'lock, T>`, then any `T` that can be constructed safely cannot be leaked.
//!
//! # Examples
//!
//! Exposing a type that cannot be leaked:
//!
//! ```
//! mod dontleakme {
//!     use unleakable::{Lock, Acceptor, Handle};
//!     pub struct DontLeakMe { _marker: () } // No public constructor
//!     impl DontLeakMe {
//!         // Requires an Acceptor, which is guaranteed to put DontLeakMe into an unleakable Lock.
//!         // Returns a Handle, which allows the user to use your type (via Deref/DerefMut)
//!         pub fn new<'lock>(acc: Acceptor<'lock, Self>) -> Handle<'lock, Self> {
//!             // Acceptor's .fill() method fills the Lock with the value,
//!             // and returns a Handle.
//!             acc.fill(DontLeakMe { _marker: () })
//!         }
//!     }
//! }
//! ```
//!
//! Using a type that cannot be leaked:
//! 
//! ```
//! # mod dontleakme {
//! #     use unleakable::{Lock, Acceptor, Handle};
//! #     pub struct DontLeakMe { _marker: () } // No public constructor
//! #     impl DontLeakMe {
//! #         // Requires an Acceptor, which is guaranteed to put DontLeakMe into an unleakable Lock.
//! #         // Returns a Handle, which allows the user to use your type (via Deref/DerefMut)
//! #         pub fn new<'lock>(acc: Acceptor<'lock, Self>) -> Handle<'lock, Self> {
//! #             // Acceptor's .fill() method fills the Lock with the value,
//! #             // and returns a Handle.
//! #             acc.fill(DontLeakMe { _marker: () })
//! #         }
//! #     }
//! # }
//! use unleakable::{Lock, Acceptor, Handle};
//! 
//! // Create the lock. Must be bound, and lock.lock() used later.
//! let mut lock = Lock::new();
//! 
//! // Passing in the relevent Acceptor.
//! // The DontLeakMe will be destructed when lock goes out of scope.
//! let mut dontleakme_handle = dontleakme::DontLeakMe::new(lock.lock());
//! 
//! // dontleakme_handle can now be used like a &DontLeakMe or &mut DontLeakMe.
//! // Some uses may require reborrows like &mut *dontleakme_handle.
//! ```
//! 
//! # Limitations
//! 
//! * It might be more ergonomic in some circumstances to implement an immobile type directly than to use this crate. Note that self-referencing types cannot impl Drop, although their contents can.
//! * `Handle<'lock, T>` implements `DerefMut`, so the underlying `T` could potentially be moved around given a second `Handle<'lock, T>`. If a bare `T` can be constructed, the `Handle`'s `T` could be swapped with the bare `T` and potentially leaked.
//! * The user of an unleakable type needs to create a `Lock` themselves, and thus needs access to the `Lock` type defined here.
//! * The creation and use of a `Lock` cannot be abstracted out in a macro, due to [Rust bug #31856](https://github.com/rust-lang/rust/issues/31856).
//! * This crate assumes that if a value cannot be moved, then it cannot be leaked. If another crate allows for violating this assumption (hypothetical example: setjmp/longjmp tricks), unsafety could result.

use std::marker::PhantomData;

pub struct Lock<'lock, T: 'lock> {
    opt_val: Option<T>,
    phantom_lock: PhantomData<&'lock mut ()>
}

impl<'lock, T: 'lock> Lock<'lock, T> {
    /// Creates an empty Lock<T> container.
    ///
    /// Needs to be stored in a binding (e.g. `let mut lock = Lock::new();`) to be useful.
    pub fn new() -> Self {
        Lock { opt_val: None, phantom_lock: PhantomData }
    }
    
    // Needs to be `&'lock mut self`, as that makes the Lock's `'lock` the same lifetime as the passed-in `&mut self`, causing it to refer to itself.
    // Taking `&mut self` instead of `&self` prevents another `Acceptor` from being created, which can help prevent confusion.
    pub fn lock(&'lock mut self) -> Acceptor<'lock, T> {
        Acceptor::new(self)
    }
}

pub struct Acceptor<'lock, T: 'lock> { 
    mut_lock: &'lock mut Lock<'lock, T>
}

impl<'lock, T: 'lock> Acceptor<'lock, T> {
    fn new(lock: &'lock mut Lock<'lock, T>) -> Self {
        Acceptor { mut_lock: lock }
    }
    
    pub fn fill(self, t: T) -> Handle<'lock, T> {
        self.mut_lock.opt_val = Some(t);
        Handle { mut_lock: self.mut_lock }
    }
    
    pub fn fill_from<'other>(self, handle_t: Handle<'other, T>) -> Handle<'lock, T> {
        let new = handle_t.mut_lock.opt_val.take().expect("Attempting to .fill_from() an emptied Handle!");
        self.fill(new)
    }
}

pub struct Handle<'lock, T: 'lock> {
    mut_lock: &'lock mut Lock<'lock, T>
}

impl<'lock, T: 'lock> std::ops::Deref for Handle<'lock, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.mut_lock.opt_val.as_ref().expect("Tried to deref() an emptied Handle!")
    }
}

impl<'lock, T: 'lock> std::ops::DerefMut for Handle<'lock, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.mut_lock.opt_val.as_mut().expect("Tried to deref_mut() an emptied Handle!")
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
        use super::*;
        let mut lock = Lock::new();
        let mut handle = lock.lock().fill(0i32);
        assert_eq!(*handle, 0i32);
    }
}
