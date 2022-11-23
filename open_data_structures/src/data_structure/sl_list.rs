use std::{cell::RefCell, rc::Rc};

use crate::interface::queue::Queue;

#[derive(Debug)]
pub struct SList<T> {
    head: Option<Rc<RefCell<Node<T>>>>,
    tail: Option<Rc<RefCell<Node<T>>>>,
    n: usize,
}

impl<T> SList<T> {
    fn new() -> Self {
        Self {
            head: None,
            tail: None,
            n: 0,
        }
    }
}

#[derive(Debug)]
pub struct Node<T> {
    x: T,
    next: Option<Rc<RefCell<Node<T>>>>,
}

impl<T> Node<T> {
    fn new(x0: T) -> Self {
        Self { x: x0, next: None }
    }
}

impl<T> Queue<T> for SList<T> {
    fn add(&mut self, x: T) {
        let mut node = Node::new(x);
        node.next = std::mem::replace(&mut self.head, None);
        let node = Rc::new(RefCell::new(node));
        self.head = Some(Rc::clone(&node));
        if self.n == 0 {
            self.tail = Some(Rc::clone(&node));
        }
        self.n += 1;
    }

    fn remove(&mut self) -> Option<T> {
        if self.n == 0 {
            None
        } else {
            let target = std::mem::replace(&mut self.head, None);
            let next = target
                .as_ref()
                .and_then(|rc| std::mem::replace(&mut rc.borrow_mut().next, None));
            self.head = next;
            target.map(|rc| Rc::try_unwrap(rc).ok().unwrap().into_inner().x)
        }
    }
}
