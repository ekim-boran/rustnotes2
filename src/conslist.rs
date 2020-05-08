use std::{fmt::Display, iter::FromIterator, ops::Deref, rc::Rc};

pub struct Stack<T>(Option<Rc<Node<T>>>);

pub struct Node<T>(T, Stack<T>);

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack(None)
    }
    pub fn cons(&self, a: T) -> Stack<T> {
        Stack(Some(Rc::new(Node(a, Stack(self.0.clone())))))
    }
    pub fn uncons(&self) -> Option<(&T, Stack<T>)> {
        match self.0 {
            None => None,
            Some(ref x) => {
                let cloned = Stack((x.1).0.clone());

                Some((&(x.0), cloned))
            }
        }
    }
}

impl<'a, T> IntoIterator for &'a Stack<T> {
    type Item = &'a T;
    type IntoIter = StackIterator<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        StackIterator(self.0.as_deref())
    }
}

pub struct StackIterator<'a, T>(Option<&'a Node<T>>);

impl<'a, T> Iterator for StackIterator<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            None => None,
            Some(x) => {
                self.0 = (x.1).0.as_deref();
                Some(&x.0)
            }
        }
    }
}
impl<T> FromIterator<T> for Stack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        iter.into_iter().fold(Stack::new(), |s, i| s.cons(i))
    }
}

impl<T: Display> Display for Stack<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (count, v) in (self).into_iter().enumerate() {
            if count != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", v)?;
        }
        write!(f, "]")
    }
}
#[test]
fn create() {
    let s1: Stack<i32> = vec![1, 2, 3, 4].into_iter().rev().collect();
    println!("{}", s1);
    if let Some((item, r)) = s1.uncons() {
        println!("{}", s1);

        println!("{}", r)
    }
}
