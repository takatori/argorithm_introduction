use crate::interface::list::List;
struct ArrayDeque<T> {
    a: Box<[T]>,
    j: usize,
    n: usize,
}

impl<T> ArrayDeque<T>
where
    T: Default + Clone,
{
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

impl<T> List<T> for ArrayDeque<T>
where
    T: Default + Clone,
{
    fn size(&self) -> usize {
        self.n
    }

    fn get(&self, i: usize) -> Option<&T> {
        self.a.get((self.j + i) % self.a.len())
    }

    fn set(&mut self, i: usize, x: T) -> T {
        let i = (self.j + i) % self.a.len();
        std::mem::replace(&mut self.a[i], x)
    }

    fn add(&mut self, i: usize, x: T) {
        if self.n >= self.a.len() {
            self.resize();
        }

        if i < self.n / 2 {
            // a[0],...,a[i-1]を左に1つずらす
            self.j = if self.j == 0 {
                self.a.len() - 1
            } else {
                self.j - 1
            };

            for k in 0..i {
                self.a[(self.j + k) % self.a.len()] =
                    self.a[(self.j + k + 1) % self.a.len()].clone();
            }
        } else {
            // a[i],...,a[n-1]を右に1つずらす
            for k in (i + 1..=self.n).rev() {
                self.a[(self.j + k) % self.a.len()] =
                    self.a[(self.j + k - 1) % self.a.len()].clone();
            }
        }
        self.a[(self.j + i) % self.a.len()] = x;
        self.n += 1;
    }

    fn remove(&mut self, i: usize) -> T {
        let x = self.a[(self.j + i) % self.a.len()].clone();

        if i < self.n / 2 {
            // a[0],...,a[i-1]を右に1つずらす
            for k in (1..=i).rev() {
                self.a[(self.j + k) % self.a.len()] =
                    self.a[(self.j + k - 1) % self.a.len()].clone();
            }
            self.j = (self.j + 1) % self.a.len();
        } else {
            // a[i+1],...,a[n-1]を左に1つずらす
            for k in i..self.n - 1 {
                self.a[(self.j + k) % self.a.len()] =
                    self.a[(self.j + k + 1) % self.a.len()].clone();
            }
        }

        self.n -= 1;
        if self.n * 3 < self.a.len() {
            self.resize();
        }

        x
    }
}
