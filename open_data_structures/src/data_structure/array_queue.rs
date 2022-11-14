use crate::interface::{list::List, queue::Queue};

struct ArrayQueue<T> {
    a: Box<[T]>,
    j: usize,
    n: usize,
}

impl<T: Default + Clone> ArrayQueue<T> {
    fn new(size: usize) -> Self {
        Self {
            a: vec![T::default(); size].into_boxed_slice(),
            j: 0,
            n: 0,
        }
    }

    fn resize(&mut self) {
        let mut b = vec![T::default(); std::cmp::max(2 * self.n, 1)].into_boxed_slice();
        for k in 0..self.n {
            b[k] = self.a[(self.j + k) % self.a.len()].clone();
        }
        self.a = b;
        self.j = 0;
    }
}

impl<T> List<T> for ArrayQueue<T>
where
    T: Default + Clone,
{
    fn size(&self) -> usize {
        self.n
    }

    fn get(&self, i: usize) -> Option<&T> {
        self.a.get(i)
    }

    fn set(&mut self, i: usize, x: T) -> T {
        std::mem::replace(&mut self.a[i], x)
    }

    fn add(&mut self, _i: usize, x: T) {
        if self.n >= self.a.len() {
            self.resize();
        }
        self.a[(self.j + self.n) % self.a.len()] = x;
        self.n += 1;
    }

    fn remove(&mut self, _i: usize) -> T {
        let x = self.a[self.j].clone();
        self.j = (self.j + 1) % self.a.len();
        self.n -= 1;
        if self.a.len() >= 3 * self.n {
            self.resize();
        }
        x
    }
}

impl<T> Queue<T> for ArrayQueue<T> 
where T: Default + Clone {
    
    fn enqueue(&mut self, x: T) {
        ArrayQueue::add(self, 0, x)
    }

    fn dequeue(&mut self) -> Option<T> {
        Some(ArrayQueue::remove(self, 0))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_resize() {
        let mut array: ArrayQueue<i32> = ArrayQueue::new(1);
        array.resize();
        assert_eq!(array.a.len(), 1);
        assert_eq!(array.n, 0);

        array.add(0, 0);
        assert_eq!(array.a.len(), 1);
        assert_eq!(array.n, 1);
    }

    #[test]
    fn test_list() {
        let mut array = ArrayQueue::new(6);

        array.add(0, "b");
        array.add(1, "r");
        array.add(2, "e");
        array.add(3, "d");
        assert_eq!(array.a, vec!["b", "r", "e", "d", "", ""].into_boxed_slice());
        assert_eq!(array.n, 4);

        array.add(2, "e");
        assert_eq!(
            array.a,
            vec!["b", "r", "e", "e", "d", ""].into_boxed_slice()
        );
        assert_eq!(array.n, 5);

        array.add(5, "r");
        assert_eq!(
            array.a,
            vec!["b", "r", "e", "e", "d", "r"].into_boxed_slice()
        );
        assert_eq!(array.n, 6);

        array.add(5, "e");
        assert_eq!(
            array.a,
            vec!["b", "r", "e", "e", "d", "e", "r", "", "", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.n, 7);

        array.remove(4);
        assert_eq!(
            array.a,
            vec!["b", "r", "e", "e", "e", "r", "r", "", "", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.n, 6);

        array.remove(4);
        assert_eq!(
            array.a,
            vec!["b", "r", "e", "e", "r", "r", "r", "", "", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.n, 5);

        array.remove(4);
        assert_eq!(
            array.a,
            vec!["b", "r", "e", "e", "", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.n, 4);

        array.set(2, "i");
        assert_eq!(
            array.a,
            vec!["b", "r", "i", "e", "", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.n, 4);
    }
}
