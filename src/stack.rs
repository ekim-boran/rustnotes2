use std::{fmt::Display, iter::FromIterator, ops::Deref};

pub struct Stack<T>(Option<Box<Node<T>>>);
pub struct Node<T>(T, Stack<T>);

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack(None)
    }
    pub fn push(&mut self, a: T) {
        let x = self.0.take();
        self.0 = Some(Box::new(Node(a, Stack(x))));
    }
    pub fn pop(&mut self) -> Option<T> {
        self.0.take().map(|head| {
            let item = (*head).0;
            self.0 = ((*head).1).0;
            item
        })
    }
}

#[test]
fn create() {
    let mut s = Stack::new();
    s.push(2);
    println!("{:?}", s.pop());
    s.push(1);
    s.push(2);
    s.push(3);
    println!("{:?}", s.pop());
}
