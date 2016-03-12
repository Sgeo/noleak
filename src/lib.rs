use std::marker::PhantomData;

pub struct Lock<'a, T: 'a> {
    opt_val: Option<T>,
    phantom_lock: PhantomData<&'a mut ()>
}

impl<'a, T: 'a> Lock<'a, T> {
    /// Creates a Lock<T> container, which should never be directly visible
    /// 
    /// Should not be called directly. Only public so that `noleak!()` can call it
    pub fn new() -> Self {
        Lock { opt_val: None, phantom_lock: PhantomData }
    }
    
    pub fn lock<R, F: FnOnce(Acceptor<'a, T>) -> R>(&'a mut self, f: F) -> R {
        let acc = Acceptor::new(self);
        f(acc)
    }
}

pub struct Acceptor<'a, T: 'a> { 
    mut_lock: &'a mut Lock<'a, T>
}

impl<'a, T: 'a> Acceptor<'a, T> {
    fn new(lock: &'a mut Lock<'a, T>) -> Self {
        Acceptor { mut_lock: lock }
    }
    
    pub fn fill(self, t: T) -> Handle<'a, T> {
        self.mut_lock.opt_val = Some(t);
        Handle { mut_lock: self.mut_lock }
    }
    
    pub fn fill_from<'b>(self, handle_t: Handle<'b, T>) -> Handle<'a, T> {
        let new = handle_t.mut_lock.opt_val.take().expect("Attempting to .fill_from() an emptied Handle!");
        self.fill(new)
    }
}

pub struct Handle<'a, T: 'a> {
    mut_lock: &'a mut Lock<'a, T>
}

impl<'a, T: 'a> std::ops::Deref for Handle<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.mut_lock.opt_val.as_ref().expect("Tried to deref() an emptied Handle!")
    }
}

impl <'a, T: 'a> std::ops::DerefMut for Handle<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.mut_lock.opt_val.as_mut().expect("Tried to deref_mut() an emptied Handle!")
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
