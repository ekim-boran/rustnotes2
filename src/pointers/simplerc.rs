use std::{
    alloc::{alloc, dealloc, Layout},
    cell::Cell,
    fmt::Display,
    marker::PhantomData,
    ptr::{self, NonNull},
    rc::Rc,
};

pub struct SimpleRc<T: ?Sized> {
    ptr: NonNull<SimpleRcBox<T>>,
    phantom: PhantomData<SimpleRcBox<T>>,
}

struct SimpleRcBox<T: ?Sized> {
    strong: Cell<usize>,
    value: T,
}

impl<T> SimpleRc<T> {
    pub fn new(value: T) -> SimpleRc<T> {
        let b = Box::new(SimpleRcBox {
            strong: Cell::new(1),
            value,
        });
        SimpleRc {
            ptr: NonNull::from(Box::leak(b)),
            phantom: PhantomData,
        }
    }
}
impl<T: ?Sized> SimpleRc<T> {
    fn strong(&self) -> usize {
        unsafe { self.ptr.as_ref().strong.get() }
    }
    fn dec_strong(&self) {
        unsafe {
            self.ptr.as_ref().strong.set(self.strong() - 1);
        }
    }
    fn inc_strong(&self) {
        unsafe {
            self.ptr.as_ref().strong.set(self.strong() + 1);
        }
    }
}

impl<T: ?Sized> Clone for SimpleRc<T> {
    fn clone(&self) -> Self {
        self.inc_strong();
        SimpleRc {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}

impl<T: ?Sized> Drop for SimpleRc<T> {
    fn drop(&mut self) {
        println!("inside drop");

        unsafe {
            self.dec_strong();
            if self.strong() == 0 {
                println!("real drop");

                ptr::drop_in_place(self.ptr.as_mut());

                dealloc(
                    self.ptr.as_ptr() as *mut _,
                    Layout::for_value(self.ptr.as_ref()),
                );
            } else {
                println!("not zero")
            }
        }
    }
}

#[test]
fn test() {
    let rc = Rc::new("asd".to_string());
    let rc2 = rc.clone();
    let asd = &*rc;
    let rc3 = Rc::new("asdasd".to_string());
    let rc4: Rc<dyn Display> = rc3;
}
#[derive(Debug)]
struct A(i32);
impl Drop for A {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

trait MyTrait {
    fn sayHello(&self);
}

impl MyTrait for A {
    fn sayHello(&self) {
        println!("Hello form A({})", self.0);
    }
}
#[test]
fn mytest() {
    let rc = SimpleRc::new(A(11));
    {
        let k = rc.clone();
    }
    let a = rc.clone();
    println!("test is fnished");
    let k = SimpleRc::new(A(11));

    // let x: MyRc<dyn MyTrait> = k;
}
