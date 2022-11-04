use std::vec;

use crate::interface::list::List;

struct ArrayStack<T> {
    a: Box<[T]>,
    n: usize, // 要素に入っているリストの要素数
}

impl<T: Default + Clone> ArrayStack<T> {
    fn new(size: usize) -> Self {
        Self {
            a: vec![T::default(); size].into_boxed_slice(),
            n: 0,
        }
    }

    // 配列の長さを変更する
    fn resize(&mut self) {
        let mut b = vec![T::default(); std::cmp::max(2 * self.n, 1)].into_boxed_slice();
        for i in 0..self.n {
            b[i] = self.a[i].clone();
        }
        self.a = b;
    }
}

impl<T> List<T> for ArrayStack<T>
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

    fn add(&mut self, i: usize, x: T) {
        if self.n + 1 >= self.a.len() {
            self.resize();
        }
        for j in (i..=self.n).rev().step_by(1) {
            self.a[j] = self.a[j - 1].clone();
        }
        self.a[i] = x;
        self.n += 1;
    }

    fn remove(&mut self, i: usize) -> T {
        let x = self.a[i].clone();
        for j in i..(self.n - 1) {
            self.a[j] = self.a[j + 1].clone();
        }
        self.n -= 1;
        // 配列の長さに対して要素が少なすぎない場合はresizeする
        if self.a.len() >= 3 * self.n {
            self.resize();
        }
        x
    }
}
