pub struct Hidden<T> {
    opt_val: Option<T>
}

impl<T> Hidden<T> {
    /// Creates a Hidden<T> container, which should never be directly visible
    /// 
    /// Should not be called directly. Only public so that `noleak!()` can call it
    pub unsafe fn new() -> Self {
        Hidden { opt_val: None }
    }
}

pub struct Acceptor<'a, T: 'a> { 
    mut_hidden: &'a mut Hidden<T>
}

impl<'a, T: 'a> Acceptor<'a, T> {
    pub unsafe fn new(hid: &'a mut Hidden<T>) -> Self {
        Acceptor { mut_hidden: hid }
    }
    
    pub fn fill(self, t: T) -> Handle<'a, T> {
        self.mut_hidden.opt_val = Some(t);
        Handle { mut_hidden: self.mut_hidden }
    }
    
    pub fn fill_from<'b>(self, handle_t: Handle<'b, T>) -> Handle<'a, T> {
        let new = handle_t.mut_hidden.opt_val.take().unwrap();
        self.fill(new)
    }
}

pub struct Handle<'a, T: 'a> {
    mut_hidden: &'a mut Hidden<T>
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
