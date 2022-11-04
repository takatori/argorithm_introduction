
use crate::interface::list::List;

struct ArrayStack<T> {
    a: Vec<T>
}


impl <T> List<T> for ArrayStack<T> {
    fn size(&self) -> usize {
        self.a.len()
    }

    fn get(&self, i: usize) -> Option<&T> {
        self.a.get(i)
    }

    fn set(&mut self, i: usize, x: T) -> T {
        std::mem::replace(&mut self.a[i], x)
    }

    fn add(&mut self, i: usize, x: T) {
        todo!()
    }

    fn remove(&mut self, i: usize) {
        todo!()
    }

}

impl ArrayStack<T> {

    fn resize() {
        todo!()
    }
}
