extern crate noleak;

use noleak::{Lock, Acceptor, Handle};

pub struct HiddenHole<'t, T: 't> {
    filled: bool,
    mut_t: &'t mut T
}

impl<'t, T> Drop for HiddenHole<'t, T> {
    fn drop(&mut self) {
        println!("Dropping HiddenHole!");
        if !self.filled {
            panic!("Unfilled hole!");
        }
    }
}

pub struct Hole<'lock, 't: 'lock, T: 't> {
    hidden_hole: Handle<'lock, HiddenHole<'t, T>>,
}

impl<'lock, 't, T> Hole<'lock, 't, T> {
    fn fill(mut self, t: T) {
        use std::ptr;
        //let mut hidden_hole = self.hidden_hole;
        unsafe {
            ptr::write(&mut *self.hidden_hole.mut_t, t); // Compiler was complaining without the reborrow
        }
        self.hidden_hole.filled = true;
    }
}

pub fn take<'lock, 't: 'lock, T: 't>(acc: Acceptor<'lock, HiddenHole<'t, T>>, mut_t: &'t mut T) -> (T, Hole<'lock, 't, T>) {
    use std::ptr;
    let t = unsafe { ptr::read(mut_t) };
    let hidden_hole = HiddenHole {
        filled: false,
        mut_t: mut_t
    };
    let hole = Hole {
        hidden_hole: acc.fill(hidden_hole)
    };
    (t, hole)
}

fn main() {
    struct Foo;
    impl Drop for Foo {
        fn drop(&mut self) {
            println!("Dropping Foo!");
        }
    }
    let mut foo = Foo;
    let mut lock = Lock::new();
    let (t, t_hole) = take(lock.lock(), &mut foo);
    drop(t);
    t_hole.fill(Foo);
}