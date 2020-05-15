use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::{
    cell::{Cell, UnsafeCell},
    fmt::Debug,
    ops::Deref,
    sync::{Arc, Mutex},
    thread,
};
use thread::JoinHandle;

pub struct List<T> {
    head: Option<Box<Node<T>>>,
}
struct Node<T>(T, List<T>);

impl<T> List<T> {
    pub fn new() -> List<T> {
        List { head: None }
    }
    pub fn add(&mut self, a: T) {
        let head1 = self.head.take();
        self.head = Some(Box::new(Node(a, List { head: head1 })));
    }

    pub fn add_ordered(&mut self, a: T)
    where
        T: PartialOrd,
    {
        if let Some(x) = &mut self.head {
            if x.0 < a {
                x.1.add_ordered(a);
            } else {
                self.add(a)
            }
        } else {
            self.add(a)
        }
    }
    pub fn remove(&mut self, a: T)
    where
        T: PartialOrd + Debug,
    {
        if let Some(x) = &mut self.head {
            if x.0 < a {
                x.1.remove(a);
            } else if x.0 == a {
                self.pop();
            } else {
                return;
            }
        } else {
            return;
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        if let Some(x) = self.head.take() {
            let Node(v, mut list) = *x;
            let mut rest = list.head.take();
            self.head = rest;
            Some(v)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        let mut it = &self.head;
        let mut count = 0;
        while let Some(node) = it {
            count = count + 1;
            it = &node.1.head;
        }
        count
    }
    pub fn take(&self) -> Vec<&T>
    where
        T: Debug,
    {
        let mut it = &self.head;
        let mut vec = vec![];
        while let Some(node) = it {
            print!("{:?}-", node.0);
            it = &node.1.head;
        }
        vec
    }
}
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.1.head.take();
        }
    }
}
struct CoarseList<T> {
    head: Mutex<List<T>>,
}
impl<T> CoarseList<T> {
    pub fn new() -> CoarseList<T> {
        CoarseList {
            head: Mutex::new(List::new()),
        }
    }
    pub fn add(&self, a: T) {
        let mut head1 = self.head.lock().unwrap();
        head1.add(a)
    }
    pub fn remove(&self, a: T)
    where
        T: PartialOrd + Debug,
    {
        let mut head1 = self.head.lock().unwrap();

        head1.remove(a);
    }
    pub fn pop(&self) -> Option<T> {
        let mut head1 = self.head.lock().unwrap();
        head1.pop()
    }
    pub fn add_ordered(&self, a: T)
    where
        T: PartialOrd + Debug,
    {
        let mut head1 = self.head.lock().unwrap();

        head1.add_ordered(a);
    }

    pub fn len(&self) -> usize {
        let head1 = self.head.lock().unwrap();
        head1.len()
    }
}

unsafe impl<T> Send for CoarseList<T> {}
unsafe impl<T> Sync for CoarseList<T> {}

#[test]
pub fn coarse_test() {
    let e = Arc::new(CoarseList::<usize>::new());
    let mut jhs = vec![];
    for _ in 0..4 {
        let l = e.clone();
        let tid = thread::spawn(move || {
            for i in 1..1000 {
                l.add_ordered(i);
                l.remove(i)
            }
        });
        jhs.push(tid)
    }

    for jh in jhs {
        jh.join().unwrap()
    }

    println!("value is  {:?}", e.len())
}

pub struct FGList<T> {
    head: Mutex<Option<Box<FGNode<T>>>>,
}
struct FGNode<T>(T, FGList<T>);
unsafe impl<T> Send for FGList<T> {}
unsafe impl<T> Sync for FGList<T> {}

impl<T> FGList<T> {
    pub fn new() -> FGList<T> {
        FGList {
            head: Mutex::new(None),
        }
    }
    pub fn add(&self, a: T) {
        let mut head1 = self.head.lock().unwrap();
        let oldhead = head1.take();
        *head1 = Some(Box::new(FGNode(
            a,
            FGList {
                head: Mutex::new(oldhead),
            },
        )));
    }
    pub fn add_ordered(&self, a: T)
    where
        T: PartialOrd,
    {
        let l = self.head.lock().unwrap();
        match l.deref() {
            Some(node) if node.0 < a => {
                node.1.add_ordered(a);
                std::mem::drop(l)
            }
            _ => self.add(a),
        }
    }

    pub fn print(&self)
    where
        T: Debug,
    {
        if let Some(node) = &*self.head.lock().unwrap() {
            print!("{:?}-", node.0);
            node.1.print();
        }
    }
}
#[test]
pub fn fine_test() {
    let e = Arc::new(FGList::<usize>::new());
    let mut jhs = vec![];
    for _ in 0..4 {
        let l = e.clone();
        let tid = thread::spawn(move || {
            for i in 1..10 {
                l.add_ordered(i);
            }
        });
        jhs.push(tid)
    }

    for jh in jhs {
        jh.join().unwrap()
    }
    e.print();
}
