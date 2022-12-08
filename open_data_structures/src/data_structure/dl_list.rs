use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::interface::clone_list::CloneList;

#[derive(Debug)]
pub struct Node<T> {
    x: T,
    next: Option<Rc<RefCell<Node<T>>>>,
    prev: Option<Weak<RefCell<Node<T>>>>,
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

impl<T: Default + Clone> DLList<T> {
    pub fn new() -> Self {
        let dummy: Rc<RefCell<Node<T>>> = Rc::new(RefCell::new(Node::new()));
        dummy.as_ref().borrow_mut().next = Some(Rc::clone(&dummy));
        dummy.as_ref().borrow_mut().prev = Some(Rc::downgrade(&dummy));
        Self { dummy, n: 0 }
    }

    pub fn get_node(&self, i: usize) -> Option<Rc<RefCell<Node<T>>>> {
        let mut p: Option<Rc<RefCell<Node<T>>>>;
        if i < self.n / 2 {
            p = self.dummy.as_ref().borrow().next.clone();
            for _ in 0..i {
                if let Some(n) = p {
                    p = n.as_ref().borrow().next.clone()
                } else {
                    break;
                }
            }
        } else {
            p = Some(self.dummy.clone());
            for _ in (i..self.n).rev() {
                if let Some(n) = p {
                    p = n.as_ref().borrow().prev.clone().and_then(|w| w.upgrade());
                } else {
                    break;
                }
            }
        }
        p
    }

    pub fn add_before(&mut self, w: &Rc<RefCell<Node<T>>>, x: T) -> Rc<RefCell<Node<T>>> {
        let mut u = Node::new();
        u.x = x;
        u.prev = w.as_ref().borrow_mut().prev.take();
        u.next = Some(Rc::clone(w));
        let mut u = Rc::new(RefCell::new(u));
        u.as_ref()
            .borrow_mut()
            .next
            .map(|rc| rc.as_ref().borrow_mut().prev = Some(Rc::downgrade(&u)));
        u.as_ref().borrow_mut().prev.map(|weak| {
            weak.upgrade()
                .map(|rc| rc.as_ref().borrow_mut().next = Some(Rc::clone(&u)))
        });
        self.n += 1;
        u
    }
}

impl<T: Default + Clone> CloneList<T> for DLList<T> {
    fn size(&self) -> usize {
        self.n
    }

    fn get(&self, i: usize) -> Option<T> {
        self.get_node(i).map(|rc| rc.as_ref().borrow().x.clone())
    }

    fn set(&mut self, i: usize, x: T) -> T {
        let u = self.get_node(i);
        let y = u.map(|rc| std::mem::replace(&mut rc.as_ref().borrow_mut().x, x));
        y.unwrap()
    }

    fn add(&mut self, i: usize, x: T) {}

    fn remove(&mut self, i: usize) -> T {
        T::default()
    }
}
