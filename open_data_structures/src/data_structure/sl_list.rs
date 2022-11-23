use std::{cell::RefCell, rc::Rc};

use crate::interface::queue::Queue;
use crate::interface::stack::Stack;

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

impl<T> Stack<T> for SList<T> {
    fn push(&mut self, x: T) {
        let mut node = Node::new(x);
        node.next = std::mem::replace(&mut self.head, None);
        let node = Rc::new(RefCell::new(node));
        self.head = Some(Rc::clone(&node));
        if self.n == 0 {
            self.tail = Some(Rc::clone(&node));
        }
        self.n += 1;
    }

    fn pop(&mut self) -> Option<T> {
        if self.n == 0 {
            None
        } else {
            let target = std::mem::replace(&mut self.head, None);
            let next = target
                .as_ref()
                .and_then(|rc| std::mem::replace(&mut rc.borrow_mut().next, None));
            self.head = next;
            self.n -= 1;
            if self.n == 0 {
                self.tail = None;
            }
            target.map(|rc| Rc::try_unwrap(rc).ok().unwrap().into_inner().x)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_stack() {
        let mut list = SList::new();
        for x in "abcde".chars() {
            list.push(x);
        }
        assert_eq!(list.n, 5);

        list.push('y');
        assert_eq!(list.n, 6);

        assert_eq!(list.pop(), Some('y'));
        assert_eq!(list.n, 5);
    }
}
