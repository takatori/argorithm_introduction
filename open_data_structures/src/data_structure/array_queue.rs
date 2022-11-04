use crate::interface::list::List;

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
