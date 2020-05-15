use chrono::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
use std::{
    cell::{Cell, RefCell, UnsafeCell},
    sync::Arc,
    thread,
};
use thread::JoinHandle;
// test and test and set lock with exponential backoff
pub struct TTASLock<T>(UnsafeCell<T>, AtomicBool);
impl<T> TTASLock<T> {
    pub fn new(a: T) -> TTASLock<T> {
        TTASLock(UnsafeCell::new(a), AtomicBool::new(false))
    }
    pub fn lock(&self) -> &mut T {
        let mut bo = Backoff::new(500, 10000);
        loop {
            while self.1.load(Ordering::Relaxed) {}
            if self.1.compare_and_swap(false, true, Ordering::SeqCst) {
                let t = bo.next();
                thread::sleep(std::time::Duration::from_micros(t));
            } else {
                break;
            }
        }

        unsafe { &mut *self.0.get() }
    }
    pub fn unlock(&self) {
        self.1.store(false, Ordering::Relaxed)
    }
}
unsafe impl<T> Send for TTASLock<T> {}
unsafe impl<T> Sync for TTASLock<T> {}

pub fn tas_test() {
    let lo = Arc::new(TTASLock::new(0));
    let mut jhs = vec![];
    let now = std::time::SystemTime::now();
    for _ in 0..50 {
        let l = lo.clone();
        let tid = thread::spawn(move || {
            for _ in 0..3000 {
                let num = l.lock();
                *num = *num + 1;
                l.unlock()
            }
        });
        jhs.push(tid)
    }

    for jh in jhs {
        jh.join().unwrap()
    }

    println!("value is {} {:?}", lo.lock(), now.elapsed())
}
#[test]
pub fn tt() {
    tas_test()
}
use rand::Rng;
pub struct Backoff {
    max: u64,
    current: u64,
}

impl Backoff {
    fn new(min: u64, max: u64) -> Backoff {
        Backoff { max, current: min }
    }
    fn next(&mut self) -> u64 {
        let mut rng = rand::thread_rng();
        let n1: u64 = rng.gen::<u64>() % self.current;
        self.current = std::cmp::min(self.max, self.current * 2);
        n1
    }
}

pub struct ALock<T> {
    value: UnsafeCell<T>,
    ticket: AtomicUsize,
    flags: [Cell<usize>; 100], // in order to prevent false sharing of cache lines
}
pub struct ALockGuard<'a, T> {
    position: usize,
    pub value: &'a mut T,
    flags: &'a [Cell<usize>],
}

impl<T> ALock<T> {
    fn next(&self) -> usize {
        let old = self.ticket.fetch_add(1, Ordering::AcqRel);
        old % 100
    }
    pub fn new(a: T, size: usize) -> ALock<T> {
        let mut arr: [Cell<usize>; 100] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        for i in 0..100 {
            arr[i] = Cell::new(0)
        }
        let alock = ALock {
            value: UnsafeCell::new(a),
            ticket: AtomicUsize::new(0),
            flags: arr,
        };
        alock.flags[0].set(1);
        alock
    }
    pub fn lock<'a>(&'a self) -> ALockGuard<'a, T> {
        let num = self.next();
        while self.flags[num].get() == 0 {
            thread::yield_now();
        }

        ALockGuard {
            position: num,
            value: unsafe { &mut *self.value.get() },
            flags: &self.flags,
        }
    }
}

unsafe impl<T> Send for ALock<T> {}
unsafe impl<T> Sync for ALock<T> {}

impl<'a, T> Drop for ALockGuard<'a, T> {
    fn drop(&mut self) {
        self.flags[self.position].set(0);
        self.flags[(self.position + 1) % 100].set(1);
    }
}

pub fn array_test() {
    let lo = Arc::new(ALock::new(0, 100));
    let mut jhs = vec![];
    let now = std::time::SystemTime::now();
    for _ in 0..50 {
        let l = lo.clone();
        let tid = thread::spawn(move || {
            for _ in 0..3000 {
                let num = l.lock();
                *num.value = *num.value + 1;
            }
        });
        jhs.push(tid)
    }

    for jh in jhs {
        jh.join().unwrap()
    }

    println!("value is {} {:?}", lo.lock().value, now.elapsed())
}
#[test]
pub fn at() {
    array_test()
}

thread_local! {
      pub static mynode: AtomicBool = AtomicBool::new(false);
      pub static mypred: Cell<&'static AtomicBool> = Cell::new(&falsenode);
}

pub static falsenode: AtomicBool = AtomicBool::new(false);
struct CLHLock<T>(UnsafeCell<T>, AtomicPtr<AtomicBool>);

impl<T> CLHLock<T> {
    fn new(a: T) -> CLHLock<T> {
        CLHLock(
            UnsafeCell::new(a),
            AtomicPtr::new(&falsenode as *const AtomicBool as *mut AtomicBool),
        )
    }

    fn lock(&self) -> &mut T {
        mynode.with(|n| {
            n.store(true, Ordering::SeqCst);
            println!("boran");
            let p = self
                .1
                .swap(n as *const AtomicBool as *mut AtomicBool, Ordering::SeqCst);
            mypred.with(|m| {
                m.set(unsafe { p.as_ref().unwrap() });
                while m.get().load(Ordering::SeqCst) {
                    thread::yield_now()
                }
            });
        });
        println!("asd");
        unsafe { &mut *self.0.get() }
    }

    fn unlock(&self) {
        mynode.with(|n| {
            println!("dropped");

            n.store(false, Ordering::SeqCst);
        });
    }
}

unsafe impl<T> Send for CLHLock<T> {}
unsafe impl<T> Sync for CLHLock<T> {}

#[test]
pub fn clh() {
    let lo = Arc::new(CLHLock::new(0));
    let mut jhs = vec![];
    let now = std::time::SystemTime::now();
    for _ in 0..1 {
        let l = lo.clone();
        let tid = thread::spawn(move || {
            for _ in 0..3000 {
                let num = l.lock();
                *num = *num + 1;
                l.unlock();
            }
        });
        jhs.push(tid)
    }

    for jh in jhs {
        jh.join().unwrap()
    }

    println!("value is {} {:?}", lo.lock(), now.elapsed())
}
