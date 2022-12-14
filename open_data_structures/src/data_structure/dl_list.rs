use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::interface::clone_list::CloneList;

type StrongLink<T> = Rc<RefCell<Node<T>>>;
type WeakLink<T> = Weak<RefCell<Node<T>>>;

#[derive(Debug)]
pub struct Node<T> {
    x: T,
    next: Option<StrongLink<T>>,
    prev: Option<WeakLink<T>>,
}

impl<T: Default> Node<T> {
    fn new() -> Self {
        Self {
            x: T::default(),
            prev: None,
            next: None,
        }
    }

    fn new_link() -> StrongLink<T> {
        Rc::new(RefCell::new(Self::new()))
    }
}


pub trait Link<L> {
    fn new() -> L;
}

impl<T: Default> Link<StrongLink<T>> for StrongLink<T> {

    fn new() -> StrongLink<T> {
        Rc::new(RefCell::new(Node::new()))
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
        let dummy: StrongLink<T> = Node::new_link();
        dummy.as_ref().borrow_mut().next = Some(Rc::clone(&dummy));
        dummy.as_ref().borrow_mut().prev = Some(Rc::downgrade(&dummy));
        Self { dummy, n: 0 }
    }

    pub fn get_link(&self, i: usize) -> Option<StrongLink<T>> {
        let mut p: Option<StrongLink<T>>;
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

    pub fn add_before(&mut self, w: Option<StrongLink<T>>, x: T) {
        let u = Node::new_link();
        u.as_ref().borrow_mut().x = x;
        u.as_ref().borrow_mut().prev = w.as_ref().and_then(|rc| rc.as_ref().borrow().prev.clone());
        if let Some(p) = w.as_ref() {
            p.as_ref().borrow_mut().prev = Some(Rc::downgrade(&u));
        }
        u.as_ref().borrow_mut().next = w;
        u.as_ref().borrow().prev.as_ref().and_then(|p| {
            p.upgrade()
                .map(|p| p.as_ref().borrow_mut().next = Some(Rc::clone(&u)))
        });
        self.n += 1;
    }

    pub fn remove_node(&mut self, w: Option<Rc<RefCell<Node<T>>>>) {
        let prev = w.as_ref().and_then(|p| p.as_ref().borrow_mut().prev.take());
        let next = w.and_then(|p| p.as_ref().borrow_mut().next.take());
        prev.as_ref().and_then(|weak| {
            weak.upgrade()
                .map(|p| p.as_ref().borrow_mut().next = next.clone())
        });
        next.map(|p| p.as_ref().borrow_mut().prev = prev);
        self.n -= 1;
    }
}

impl<T: Default + Clone> CloneList<T> for DLList<T> {
    fn size(&self) -> usize {
        self.n
    }

    fn get(&self, i: usize) -> Option<T> {
        self.get_link(i).map(|rc| rc.as_ref().borrow().x.clone())
    }

    fn set(&mut self, i: usize, x: T) -> T {
        let u = self.get_link(i);
        let y = u.map(|rc| std::mem::replace(&mut rc.as_ref().borrow_mut().x, x));
        y.unwrap()
    }

    fn add(&mut self, i: usize, x: T) {
        self.add_before(self.get_link(i), x);
    }

    fn remove(&mut self, i: usize) -> T {
        let node = self.get_link(i);
        let x = node.as_ref().map(|rc| rc.as_ref().borrow().x.clone());
        self.remove_node(node);
        x.unwrap()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_clone_list() {
        let mut list = DLList::new();
        list.add(0, 'a');
        list.add(1, 'b');
        list.add(2, 'c');
        list.add(3, 'd');
        list.add(4, 'e');
        assert_eq!(list.size(), 5);
        assert_eq!(list.get(0).unwrap(), 'a');
        assert_eq!(list.get(1).unwrap(), 'b');
        assert_eq!(list.get(2).unwrap(), 'c');
        assert_eq!(list.get(3).unwrap(), 'd');
        assert_eq!(list.get(4).unwrap(), 'e');

        list.remove(3);
        assert_eq!(list.size(), 4);
        assert_eq!(list.get(0).unwrap(), 'a');
        assert_eq!(list.get(1).unwrap(), 'b');
        assert_eq!(list.get(2).unwrap(), 'c');
        assert_eq!(list.get(3).unwrap(), 'e');
    }
}
