use crate::interface::list::List;

/// 両端に対して追加と削除が効率的にできる
struct ArrayDeque<T> {
    a: Box<[T]>,
    j: usize,
    n: usize,
}

impl<T> ArrayDeque<T>
where
    T: Default + Clone,
{
    pub fn new(size: usize) -> Self {
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

    /// 要素を追加する
    ///
    /// iが小さい時(0に近いとき)とiが大きい時(nに近い時)に効率が良くなる
    /// i < n/2の場合左にずらす、そうでないなら右にずらす
    /// 移動する要素の数が高々 min{i, n-i}個に保証される
    ///
    /// ## 計算量
    /// * O(1 + min{i,n-i})
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

    ///　要素を削除する
    ///
    /// i < n/2 の場合右にずらす、そうでない場合左にずらす
    ///
    /// ## 計算量
    /// * O(1 + min{i,n-i})
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

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_list() {
        let mut array = ArrayDeque::new(12);

        array.add(0, "a");
        array.add(1, "b");
        array.add(2, "c");
        array.add(3, "d");
        array.add(4, "e");
        array.add(5, "f");
        array.add(6, "g");
        array.add(7, "h");
        assert_eq!(
            array.a,
            vec!["a", "b", "c", "d", "e", "f", "g", "h", "", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.j, 0);
        assert_eq!(array.n, 8);

        array.remove(2);
        assert_eq!(
            array.a,
            vec!["a", "a", "b", "d", "e", "f", "g", "h", "", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.j, 1);
        assert_eq!(array.n, 7);

        array.add(4, "x");
        assert_eq!(
            array.a,
            vec!["a", "a", "b", "d", "e", "x", "f", "g", "h", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.j, 1);
        assert_eq!(array.n, 8);

        array.add(3, "y");
        assert_eq!(
            array.a,
            vec!["a", "b", "d", "y", "e", "x", "f", "g", "h", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.j, 0);
        assert_eq!(array.n, 9);

        array.add(3, "z");
        assert_eq!(
            array.a,
            vec!["b", "d", "z", "y", "e", "x", "f", "g", "h", "", "", "a"].into_boxed_slice()
        );
        assert_eq!(array.j, 11);
        assert_eq!(array.n, 10);
    }
}
