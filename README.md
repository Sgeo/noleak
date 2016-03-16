# unleakable

This crate provides a container that cannot be leaked (e.g. via `std::mem::forget()`) once `lock()`ed: `Lock<'lock, T>`.

`Lock::lock()` returns an `Acceptor<'lock, T>`, which allows placing `T` inside the `Lock<'lock, T>`.

If the only safe way to create `T` requires an `Acceptor<'lock, T>`, then any `T` that can be constructed safely cannot be leaked.

## Examples

Exposing a type that cannot be leaked:

```rust
mod dontleakme {
    use unleakable::{Lock, Acceptor, Handle};
    pub struct DontLeakMe { _marker: () } // No public constructor
    impl DontLeakMe {
        // Requires an Acceptor, which is guaranteed to put DontLeakMe into an unleakable Lock.
        // Returns a Handle, which allows the user to use your type (via Deref/DerefMut)
        pub fn new<'lock>(acc: Acceptor<'lock, Self>) -> Handle<'lock, Self> {
            // Acceptor's .fill() method fills the Lock with the value,
            // and returns a Handle.
            acc.fill(DontLeakMe { _marker: () })
        }
    }
}
```

Using a type that cannot be leaked:

```rust
use unleakable::{Lock, Acceptor, Handle};

// Create the lock. Must be bound, and lock.lock() used later.
let mut lock = Lock::new();

// Passing in the relevent Acceptor.
// The DontLeakMe will be destructed when lock goes out of scope.
let mut dontleakme_handle = dontleakme::DontLeakMe::new(lock.lock());

// dontleakme_handle can now be used like a &DontLeakMe or &mut DontLeakMe.
// Some uses may require reborrows like &mut *dontleakme_handle.
```

## Limitations

* It might be more ergonomic in some circumstances to implement an immobile type directly than to use this crate. Note that self-referencing types cannot impl Drop, although their contents can.
* `Handle<'lock, T>` implements `DerefMut`, so the underlying `T` could potentially be moved around given a second `Handle<'lock, T>`. If a bare `T` can be constructed, the `Handle`'s `T` could be swapped with the bare `T` and potentially leaked.
* The user of an unleakable type needs to create a `Lock` themselves, and thus needs access to the `Lock` type defined here.
* The creation and use of a `Lock` cannot be abstracted out in a macro, due to [Rust bug #31856](https://github.com/rust-lang/rust/issues/31856).
* This crate assumes that if a value cannot be moved, then it cannot be leaked. If another crate allows for violating this assumption (hypothetical example: setjmp/longjmp tricks), unsafety could result.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.