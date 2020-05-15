use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::{
    cell::{Cell, UnsafeCell},
    sync::Arc,
    thread,
};
use thread::JoinHandle;
#[allow(dead_code)]
pub fn spawn<F>(f: F) -> thread::JoinHandle<()>
where
    F: Fn(usize) + 'static + Send,
{
    static mut COUNTER: usize = 0;
    unsafe {
        let c = COUNTER;

        COUNTER += 1;
        thread::spawn(move || f(c))
    }
}

//trait Lock {
//  type Item;
//  type Guard;
//  fn new(a: Self::Item) -> Self;
//  fn lock(&self, tid: usize) -> Self::Guard;
//  fn unlock(&self, tid: usize);
//  fn get(g: &Self::Guard) -> &mut Self::Item;
//}

// it deadlocks in concurrent executions
pub struct LockOne<T>(UnsafeCell<T>, [AtomicBool; 2]);
impl<T> LockOne<T> {
    pub fn new(a: T) -> LockOne<T> {
        LockOne(
            UnsafeCell::new(a),
            [AtomicBool::new(false), AtomicBool::new(false)],
        )
    }
    pub fn lock<'a>(&'a self, tid: usize) -> LockGuard<'a, T> {
        self.1[tid].store(true, Ordering::Release);
        while self.1[1 - tid].load(Ordering::Acquire) {}
        LockGuard(&self.0, &self.1, tid)
    }
}

pub struct LockGuard<'a, T>(&'a UnsafeCell<T>, &'a [AtomicBool; 2], usize);
impl<'a, T> LockGuard<'a, T> {
    pub fn get(&self) -> &mut T {
        unsafe { &mut *(self.0).get() }
    }
}
impl<'a, T> Drop for LockGuard<'a, T> {
    fn drop(&mut self) {
        self.1[self.2].store(false, Ordering::Release);
        println!("dropped")
    }
}

unsafe impl<T> Send for LockOne<T> {}
unsafe impl<T> Sync for LockOne<T> {}

#[test]
fn test() {
    let lo = Arc::new(LockOne::new(0));
    let mut jhs = vec![];
    for _ in 0..2 {
        let l = lo.clone();
        let tid = spawn(move |tid| {
            for _ in 0..20 {
                let num = l.lock(tid);
                *num.get() = *num.get() + 1;
                println!("{}", tid)
            }
        });
        jhs.push(tid)
    }

    for jh in jhs {
        jh.join().unwrap()
    }
    println!("value is {}", lo.lock(1).get())
}

// it deadlocks in sequential executions
pub struct LockTwo<T>(UnsafeCell<T>, AtomicUsize);
unsafe impl<T> Send for LockTwo<T> {}
unsafe impl<T> Sync for LockTwo<T> {}

impl<T> LockTwo<T> {
    pub fn new(a: T) -> LockTwo<T> {
        LockTwo(UnsafeCell::new(a), AtomicUsize::new(3))
    }
    pub fn lock<'a>(&'a self, tid: usize) -> &mut T {
        unsafe {
            self.1.store(tid, Ordering::Release);
            while self.1.load(Ordering::Acquire) == tid {}
            let t = &mut *self.0.get();
            t
        }
    }
}

#[test]
pub fn testlock2() {
    let lo = Arc::new(LockTwo::new(0));
    let mut jhs = vec![];
    for _ in 0..2 {
        let l = lo.clone();
        let tid: JoinHandle<()> = spawn(move |tid| {
            for _ in 0..20 {
                let num = l.lock(tid);
                *num = *num + 1;
                println!("{} - {}", tid, num)
            }
        });
        jhs.push(tid)
    }

    for jh in jhs {
        jh.join().unwrap()
    }
}

pub struct Peterson<T> {
    value: UnsafeCell<T>,
    interested: [AtomicBool; 2],
    victim: AtomicUsize,
}
impl<T> Peterson<T> {
    fn new(a: T) -> Peterson<T> {
        Peterson {
            value: UnsafeCell::new(a),
            interested: [AtomicBool::new(false), AtomicBool::new(false)],
            victim: AtomicUsize::new(3),
        }
    }
    fn lock<'a>(&'a self, me: usize) -> LockGuard<'a, T> {
        let other = 1 - me;

        self.interested[me].store(true, Ordering::Relaxed);
        self.victim.swap(me, Ordering::AcqRel);

        while self.interested[other].load(Ordering::Acquire)
            && self.victim.load(Ordering::Relaxed) == me
        {}
        LockGuard(&self.value, &self.interested, me)
    }
}

unsafe impl<T> Send for Peterson<T> {}
unsafe impl<T> Sync for Peterson<T> {}

#[test]
fn testPeterson() {
    let lo = Arc::new(Peterson::new(0));
    let mut jhs = vec![];
    for _ in 0..2 {
        let l = lo.clone();
        let tid = spawn(move |tid| {
            for _ in 0..20 {
                let num = l.lock(tid);
                *num.get() = *num.get() + 1;
                println!("{}", tid)
            }
        });
        jhs.push(tid)
    }

    for jh in jhs {
        jh.join().unwrap()
    }
    println!("value is {}", lo.lock(1).get())
}

pub struct FilterLock<T> {
    value: UnsafeCell<T>,
    levels: Vec<AtomicUsize>,
    victim: Vec<AtomicUsize>,
}
impl<T> FilterLock<T> {
    pub fn new(a: T, n: usize) -> FilterLock<T> {
        let mut levels = vec![];
        let mut victims = vec![];

        for _ in 0..n {
            levels.push(AtomicUsize::new(n));
            victims.push(AtomicUsize::new(n));
        }

        FilterLock {
            value: UnsafeCell::new(a),
            levels: levels,
            victim: victims,
        }
    }
    pub fn lock(&self, me: usize) -> &mut T {
        for i in (0..self.levels.len()).rev() {
            self.levels[me].store(i, Ordering::Relaxed);
            self.victim[i].swap(me, Ordering::Relaxed);

            while self
                .levels
                .iter()
                .enumerate()
                .any(|(t, x)| t != me && x.load(Ordering::Relaxed) <= i)
                && self.victim[i].load(Ordering::Relaxed) == me
            {}
        }
        unsafe { &mut *self.value.get() }
    }
    pub fn unlock(&self, me: usize) {
        self.levels[me].store(self.levels.len(), Ordering::Release)
    }
}
unsafe impl<T> Send for FilterLock<T> {}
unsafe impl<T> Sync for FilterLock<T> {}

#[test]
fn test_filter_lock() {
    let cpus = num_cpus::get();
    println!("{}", cpus);
    let lo = Arc::new(FilterLock::new(0, 30));
    let mut jhs = vec![];
    for _ in 0..30 {
        let l = lo.clone();
        let tid = spawn(move |tid| {
            for _ in 0..300 {
                let num = l.lock(tid);
                *num = *num + 1;
                l.unlock(tid)
            }
        });
        jhs.push(tid)
    }

    for jh in jhs {
        jh.join().unwrap()
    }
    println!("value is {}", lo.lock(1))
}

struct Bakery<T> {
    value: UnsafeCell<T>,
    labels: Vec<AtomicUsize>,
    flags: Vec<AtomicBool>,
}
impl<T> Bakery<T> {
    pub fn new(a: T, n: usize) -> Bakery<T> {
        let mut labels = vec![];
        let mut flags = vec![];

        for _ in 0..n {
            labels.push(AtomicUsize::new(0));
            flags.push(AtomicBool::new(false));
        }
        Bakery {
            value: UnsafeCell::new(a),
            labels: labels,
            flags: flags,
        }
    }

    pub fn lock(&self, me: usize) -> &mut T {
        self.flags[me].store(true, Ordering::Relaxed);

        let m = self.labels.iter().fold(0, |a, x| {
            let xx = x.load(Ordering::Relaxed);
            if a < xx {
                xx
            } else {
                a
            }
        }) + 1;

        self.labels[me].store(m, Ordering::Relaxed);

        while (0..self.flags.len()).any(|t| {
            t != me
                && self.flags[t].load(Ordering::Relaxed)
                && (self.labels[t].load(Ordering::Relaxed), t) < (m, me)
        }) {
            thread::yield_now()
        }

        unsafe { &mut *self.value.get() }
    }

    fn unlock(&self, me: usize) {
        self.flags[me].store(false, Ordering::SeqCst);
    }
}
unsafe impl<T> Send for Bakery<T> {}
unsafe impl<T> Sync for Bakery<T> {}

#[test]
fn test_bakery() {
    let cpus = num_cpus::get();
    println!("{}", cpus);
    let lo = Arc::new(Bakery::new(0, 100));
    let mut jhs = vec![];
    for _ in 0..100 {
        let l = lo.clone();
        let a = spawn(move |tid| {
            for _ in 0..600 {
                let num = l.lock(tid);
                *num = *num + 1;
                l.unlock(tid)
            }
        });
        jhs.push(a)
    }

    for jh in jhs {
        jh.join().unwrap()
    }
    println!("value is {}", lo.lock(0))
}
