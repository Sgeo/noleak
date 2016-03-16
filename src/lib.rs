use std::marker::PhantomData;

pub struct Lock<'lock, T: 'lock> {
    opt_val: Option<T>,
    phantom_lock: PhantomData<&'lock mut ()>
}

impl<'lock, T: 'lock> Lock<'lock, T> {
    /// Creates an empty Lock<T> container.
    ///
    /// Needs to be stored in a binding (e.g. `let lock = Lock::new();`) to be useful.
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
