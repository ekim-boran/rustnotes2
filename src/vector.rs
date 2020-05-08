use std::ptr::{self};

pub struct Vector<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
}
use std::alloc::{alloc, dealloc, realloc, Layout};
use std::mem;
impl<T> Vector<T> {
    fn new() -> Self {
        Vector {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }

    fn grow(&mut self) {
        unsafe {
            let align = mem::align_of::<T>();
            let elem_size = mem::size_of::<T>();
            let (new_cap, ptr) = if self.cap == 0 {
                let ptr = alloc(Layout::from_size_align(elem_size, align).unwrap());
                (1, ptr)
            } else {
                let new_cap = self.cap * 2;
                let oldlayout = Layout::from_size_align(self.cap * elem_size, align).unwrap();
                let newsize = new_cap * elem_size;
                let ptr = realloc(self.ptr.as_ptr() as *mut _, oldlayout, newsize);
                (new_cap, ptr)
            };
            self.ptr = NonNull::new(ptr as *mut _).unwrap();
            self.cap = new_cap;
        }
    }
    pub fn push(&mut self, elem: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            ptr::write(self.ptr.as_ptr().offset(self.len as isize), elem);
        }

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr.as_ptr().offset(self.len as isize))) }
        }
    }
    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "index out of bounds");
        if self.cap == self.len {
            self.grow();
        }

        unsafe {
            if index < self.len {
                ptr::copy(
                    self.ptr.as_ptr().offset(index as isize),
                    self.ptr.as_ptr().offset(index as isize + 1),
                    self.len - index,
                );
            }
            ptr::write(self.ptr.as_ptr().offset(index as isize), elem);
            self.len += 1;
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        // Note: `<` because it's *not* valid to remove after everything
        assert!(index < self.len, "index out of bounds");
        unsafe {
            self.len -= 1;
            let result = ptr::read(self.ptr.as_ptr().offset(index as isize));
            ptr::copy(
                self.ptr.as_ptr().offset(index as isize + 1),
                self.ptr.as_ptr().offset(index as isize),
                self.len - index,
            );
            result
        }
    }
}
impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        println!("i am droppping1");

        if self.cap != 0 {
            println!("i am droppping2");
            while let Some(_) = self.pop() {}

            let align = mem::align_of::<T>();
            let elem_size = mem::size_of::<T>();
            let num_bytes = elem_size * self.cap;
            unsafe {
                dealloc(
                    self.ptr.as_ptr() as *mut _,
                    Layout::from_size_align(num_bytes, align).unwrap(),
                );
            }
        }
    }
}
use ptr::NonNull;
use std::ops::Deref;

impl<T> Deref for Vector<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { ::std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
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
fn vecexample() {
    {
        let mut v = Vector::new();
        v.push(A(1));
        v.push(A(2));
        let a = v.pop();
        println!("{:?}", a);
    }
    println!("asd")
}

////create intoiter

struct IntoIter<T> {
    buf: NonNull<T>,
    cap: usize,
    start: *const T,
    end: *const T,
}
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let result = ptr::read(self.start);
                self.start = self.start.offset(1);
                Some(result)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize) / mem::size_of::<T>();
        (len, Some(len))
    }
}
impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = self.end.offset(-1);
                Some(ptr::read(self.end))
            }
        }
    }
}
impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            // drop any remaining elements
            for _ in &mut *self {}

            let align = mem::align_of::<T>();
            let elem_size = mem::size_of::<T>();
            let num_bytes = elem_size * self.cap;
            unsafe {
                dealloc(
                    self.buf.as_ptr() as *mut _,
                    Layout::from_size_align_unchecked(num_bytes, align),
                );
            }
        }
    }
}
