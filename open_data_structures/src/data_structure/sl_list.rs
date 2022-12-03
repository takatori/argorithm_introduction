use std::fmt::{self, Debug};
use std::{cell::RefCell, rc::Rc};

use crate::interface::queue::Queue;
use crate::interface::stack::Stack;

pub struct Node<T> {
    x: T,
    next: Option<Rc<RefCell<Node<T>>>>,
}

impl<T> Node<T> {
    fn new(x: T) -> Self {
        Self { x, next: None }
    }
}

impl<T: Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(n) = &self.next {
            write!(f, "{:?} -> {:?}", self.x, n.borrow_mut())
        } else {
            write!(f, "{:?}", self.x)
        }
    }
}

/// Singley-Linked List(単方向連結リスト)
///
/// StackとQueueインタフェースを実装する
/// push(x),pop(),add(x),remove()の実行時間はいずれもO(1)
///
/// Dequeの操作もほぼ全て実装できるが、末尾を削除する操作が足りない
/// 末尾を削除する場合、末尾の一つ前のノードを探す必要があるが
/// これには、各ノードをheadから順にn-2回辿る必要がある
pub struct SLList<T> {
    head: Option<Rc<RefCell<Node<T>>>>,
    tail: Option<Rc<RefCell<Node<T>>>>,
    n: usize,
}

impl<T> SLList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            n: 0,
        }
    }
}

impl<T: Debug> fmt::Debug for SLList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(n) = &self.head {
            write!(f, "haed -> {:?}", n.borrow_mut())
        } else {
            write!(f, "null")
        }
    }
}

impl<T> Stack<T> for SLList<T> {
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

impl<T> Queue<T> for SLList<T> {
    fn add(&mut self, x: T) {
        let node = Rc::new(RefCell::new(Node::new(x)));
        if self.n == 0 {
            self.head = Some(Rc::clone(&node));
        } else {
            self.tail
                .as_ref()
                .map(|rc| rc.borrow_mut().next = Some(Rc::clone(&node)));
        }
        self.tail = Some(Rc::clone(&node));
        self.n += 1;
    }

    fn remove(&mut self) -> Option<T> {
        self.pop()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_stack() {
        let mut list = SLList::new();
        for x in "abcde".chars() {
            list.push(x);
        }
        assert_eq!(list.n, 5);

        list.push('y');
        assert_eq!(list.n, 6);

        assert_eq!(list.pop(), Some('y'));
        assert_eq!(list.n, 5);
        println!("{:?}", list);
    }

    #[test]
    fn test_queue() {
        let mut list = SLList::new();
        for x in "abcde".chars() {
            list.add(x);
        }
        assert_eq!(list.n, 5);

        list.add('x');
        assert_eq!(list.n, 6);

        assert_eq!(list.remove(), Some('a'));
        assert_eq!(list.n, 5);
        println!("{:?}", list);
    }
}
