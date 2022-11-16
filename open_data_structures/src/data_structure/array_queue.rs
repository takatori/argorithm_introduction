use crate::interface::queue::Queue;

/// Queueインタフェースの実装
///
/// resize()のコストを無視すると
/// add(x), remove()の実行時間はO(1)
/// 空のArrayQueueに対して任意のm個のadd(i,x)およびremove(i)からなる操作の列を実行する。
/// このときreizeにかかる時間はO(m)
struct ArrayQueue<T> {
    a: Box<[T]>, // 循環配列
    j: usize,    // 次に削除する要素を追跡するインデックス
    n: usize,    // キューの要素数
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

impl<T> Queue<T> for ArrayQueue<T>
where
    T: Default + Clone,
{
    fn add(&mut self, x: T) {
        if self.n >= self.a.len() {
            self.resize();
        }
        self.a[(self.j + self.n) % self.a.len()] = x;
        self.n += 1;
    }

    fn remove(&mut self) -> Option<T> {
        let x = self.a[self.j].clone();
        self.j = (self.j + 1) % self.a.len();
        self.n -= 1;
        if self.a.len() >= 3 * self.n {
            self.resize();
        }
        Some(x)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_queue() {
        let mut array = ArrayQueue::new(6);

        array.add("0");
        assert_eq!(array.a, vec!["0", "", "", "", "", ""].into_boxed_slice());
        assert_eq!(array.j, 0);
        assert_eq!(array.n, 1);

        array.add("1");
        assert_eq!(array.a, vec!["0", "1", "", "", "", ""].into_boxed_slice());
        assert_eq!(array.j, 0);
        assert_eq!(array.n, 2);

        array.add("a");
        array.add("b");
        array.add("c");
        assert_eq!(
            array.a,
            vec!["0", "1", "a", "b", "c", ""].into_boxed_slice()
        );
        assert_eq!(array.j, 0);
        assert_eq!(array.n, 5);

        array.remove();
        array.remove();
        assert_eq!(
            array.a,
            vec!["0", "1", "a", "b", "c", ""].into_boxed_slice()
        );
        assert_eq!(array.j, 2);
        assert_eq!(array.n, 3);

        array.add("d");
        assert_eq!(
            array.a,
            vec!["0", "1", "a", "b", "c", "d"].into_boxed_slice()
        );
        assert_eq!(array.j, 2);
        assert_eq!(array.n, 4);

        array.add("e");
        assert_eq!(
            array.a,
            vec!["e", "1", "a", "b", "c", "d"].into_boxed_slice()
        );
        assert_eq!(array.j, 2);
        assert_eq!(array.n, 5);

        array.remove();
        assert_eq!(
            array.a,
            vec!["e", "1", "a", "b", "c", "d"].into_boxed_slice()
        );
        assert_eq!(array.j, 3);
        assert_eq!(array.n, 4);

        array.add("f");
        assert_eq!(
            array.a,
            vec!["e", "f", "a", "b", "c", "d"].into_boxed_slice()
        );
        assert_eq!(array.j, 3);
        assert_eq!(array.n, 5);

        array.add("g");
        assert_eq!(
            array.a,
            vec!["e", "f", "g", "b", "c", "d"].into_boxed_slice()
        );
        assert_eq!(array.j, 3);
        assert_eq!(array.n, 6);

        array.add("h");
        assert_eq!(
            array.a,
            vec!["b", "c", "d", "e", "f", "g", "h", "", "", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.j, 0);
        assert_eq!(array.n, 7);

        array.remove();
        assert_eq!(
            array.a,
            vec!["b", "c", "d", "e", "f", "g", "h", "", "", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.j, 1);
        assert_eq!(array.n, 6);
    }
}
