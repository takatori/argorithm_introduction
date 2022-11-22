use crate::interface::queue::Queue;

pub struct SList<T> {
    head: Option<Box<Node<T>>>,
    tail: Option<Box<Node<T>>>,
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

pub struct Node<T> {
    x: T,
    next: Option<Box<Node<T>>>,
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
        self.head = Some(Box::new(node));
        if self.n == 0 {
            self.tail = Some(Box::new(node));
        }
        self.n += 1;
    }

    fn remove(&mut self) -> Option<T> {
        None
    }

    /*
    fn remove(&mut self) -> Option<T> {
        if self.n == 0 {
            None
        } else {
            let x = self.head.map(|b| b.x);
            self.head = self.head.map(|b| b.next);
            x
        }
    }*/
}
