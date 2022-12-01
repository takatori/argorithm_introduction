use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    rc::Rc,
};

use crate::interface::list::List;

#[derive(Debug)]
pub struct Node<T> {
    x: T,
    prev: Option<Rc<RefCell<Node<T>>>>,
    next: Option<Rc<RefCell<Node<T>>>>,
}

impl<T: Default> Node<T> {
    fn new() -> Self {
        Self {
            x: T::default(),
            prev: None,
            next: None,
        }
    }
}

/// 双方向連結リスト
#[derive(Debug)]
pub struct DLList<T> {
    dummy: Rc<RefCell<Node<T>>>,
    n: usize,
}

impl<T: Default> DLList<T> {
    pub fn new() -> Self {
        let dummy: Rc<RefCell<Node<T>>> = Rc::new(RefCell::new(Node::new()));
        dummy.borrow_mut().next = Some(Rc::clone(&dummy));
        dummy.borrow_mut().prev = Some(Rc::clone(&dummy));
        Self { dummy, n: 0 }
    }

    pub fn get_node(&self, i: usize) -> Option<Rc<RefCell<Node<T>>>> {
        let mut p: Option<Rc<RefCell<Node<T>>>>;
        if i < self.n / 2 {
            p = (*self.dummy).borrow().next.clone();
            for _ in 0..i {
                if let Some(n) = p {
                    p = (*n).borrow().next.clone()
                } else {
                    break;
                }
            }
        } else {
            p = Some(self.dummy.clone());
            for _ in (i..self.n).rev() {
                if let Some(n) = p {
                    p = (*n).borrow().prev.clone();
                } else {
                    break;
                }
            }
        }
        p
    }
}

impl<T: Default> List<T> for DLList<T> {

    fn size(&self) -> usize {
        self.n
    }

    fn get(&self, i: usize) -> Option<&T> {
        self.get_node(i).as_deref()
    }

    fn set(&mut self, i: usize, x: T) -> T {
        
    }

    fn add(&mut self, i: usize, x: T) {
        
    }

    fn remove(&mut self, i: usize) -> T {
        
    }
}
