use std::alloc::{alloc, dealloc, realloc, Layout};
use std::{
    fmt::Display,
    intrinsics::drop_in_place,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

/// figure out how to work with trait objects / slices
pub struct SimpleBox<T>(NonNull<T>);

impl<T> SimpleBox<T> {
    fn new(a: T) -> SimpleBox<T> {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        unsafe {
            let ptr = alloc(Layout::from_size_align_unchecked(size, align));
            let ptr = ptr as *mut _;
            std::ptr::write::<T>(ptr as *mut _, a);
            SimpleBox(NonNull::new(ptr).unwrap())
        }
    }
}

impl<T> Drop for SimpleBox<T> {
    fn drop(&mut self) {
        unsafe {
            //cannot use because of trait objects let _ = std::ptr::read::<T>(self.0.as_ptr());
            drop_in_place(self.0.as_ptr());
            dealloc(self.0.as_ptr() as *mut _, Layout::new::<T>())
        }
        println!("drop finished")
    }
}

impl<T> Deref for SimpleBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}
impl<T> DerefMut for SimpleBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}
#[derive(Debug)]
struct A(i32);
impl Drop for A {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

#[test]
fn mytest() {
    fn testhelper(mut a: SimpleBox<A>) -> SimpleBox<A> {
        *a = A(91);
        a
    }
    {
        let k = Box::new(2);
        let c: Box<dyn Display> = k;
        //let ka = MyBox::new(2);
        //let ca: MyBox<dyn Display> = ka;

        let b = SimpleBox::new(A(13));
        let c = testhelper(b);
    }
    println!("finished")
}
