use std::{fmt::Display, iter::FromIterator, ops::Deref};

pub struct Queue<T>(Option<Box<Node<T>>>);
pub struct Node<T>(T, Stack<T>);
