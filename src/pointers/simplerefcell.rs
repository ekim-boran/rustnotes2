use std::{
    cell::{Cell, RefCell, UnsafeCell},
    ops::{Deref, DerefMut},
};
pub struct SimpleRefCell<T: ?Sized> {
    borrow: Cell<isize>,
    value: UnsafeCell<T>,
}

pub struct BorrowRef<'a, T>(&'a Cell<isize>, &'a mut T);

impl<T> Drop for BorrowRef<'_, T> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.0.get();
        self.0.set(borrow + 1);
    }
}
impl<'a, T> BorrowRef<'a, T> {
    fn new(arg: &'a SimpleRefCell<T>) -> Result<BorrowRef<'a, T>, ()> {
        if arg.borrow.get() == 0 {
            let c = &arg.borrow;
            let b = unsafe { arg.value.get().as_mut().unwrap() };
            Ok(BorrowRef(c, b))
        } else {
            Err(())
        }
    }
}

impl<T> Deref for BorrowRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.1
    }
}
impl<T> DerefMut for BorrowRef<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.1
    }
}

pub struct Ref<'a, T>(&'a Cell<isize>, &'a T);

impl<T> Drop for Ref<'_, T> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.0.get();
        self.0.set(borrow - 1);
    }
}
impl<'a, T> Ref<'a, T> {
    fn new(arg: &'a SimpleRefCell<T>) -> Result<Ref<'a, T>, ()> {
        if arg.borrow.get() > 0 {
            let c = &arg.borrow;
            let b = unsafe { arg.value.get().as_ref().unwrap() };
            Ok(Ref(c, b))
        } else {
            Err(())
        }
    }
}
impl<T> Deref for Ref<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.1
    }
}
impl<T> SimpleRefCell<T> {
    pub fn new(value: T) -> SimpleRefCell<T> {
        SimpleRefCell {
            value: UnsafeCell::new(value),
            borrow: Cell::new(0),
        }
    }
    pub fn borrow_mut(&self) -> BorrowRef<'_, T> {
        match BorrowRef::new(self) {
            Err(()) => panic!("error"),
            Ok(r) => r,
        }
    }
    pub fn borrow(&self) -> Ref<'_, T> {
        match Ref::new(self) {
            Err(()) => panic!("error"),
            Ok(r) => r,
        }
    }
}

#[test]
fn test() {
    let a = RefCell::new(2);
    {
        let mut c = a.borrow_mut();
        *c = 12;
    }
    let k = a.borrow();
    println!("{}", *k);
}

#[test]
fn simpletest() {
    let a = SimpleRefCell::new(2);
    {
        let mut c = a.borrow_mut();
        *c = 12;
    }
    let k = a.borrow();
    println!("{}", *k);
}
