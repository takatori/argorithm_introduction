use crate::{data_structure::array_stack::ArrayStack, interface::list::List};

#[derive(Debug)]
pub struct DualArrayDeque<T> {
    front: ArrayStack<T>,
    back: ArrayStack<T>,
}

impl<T> DualArrayDeque<T>
where
    T: Default + Clone,
{
    pub fn new(size: usize) -> Self {
        Self {
            front: ArrayStack::new(size),
            back: ArrayStack::new(size / 2),
        }
    }

    /// frontとbackの要素数が極端に偏らないようにする
    ///
    /// 要素数が2以上の時、frontもbackもn/4以上の要素を含むようにする
    /// frontかbackにn/4以上の要素が含まれそうな場合は、
    /// 要素を動かしてfrontとbackにちょうどn/2個及びn/2個の要素が含まれるようにする
    fn balance(&mut self) {
        if 3 * self.front.size() < self.back.size() || 3 * self.back.size() < self.front.size() {
            let n = self.front.size() + self.back.size();
            let nf = n / 2;
            let mut af: Vec<T> = vec![T::default(); std::cmp::max(2 * nf, 1)];
            for i in 0..nf {
                af[nf - i - 1] = self.get(i).unwrap().clone(); // TODO: fix
            }
            let nb = n - nf;
            let mut ab: Vec<T> = vec![T::default(); std::cmp::max(2 * nb, 1)];
            for i in 0..nb {
                ab[i] = self.get(nf + i).unwrap().clone(); // TODO: fix
            }
            self.front.a = af.into_boxed_slice();
            self.front.n = nf;
            self.back.a = ab.into_boxed_slice();
            self.back.n = nb;
        }
    }
}

impl<T> List<T> for DualArrayDeque<T>
where
    T: Default + Clone,
{
    fn size(&self) -> usize {
        self.front.size() + self.back.size()
    }

    fn get(&self, i: usize) -> Option<&T> {
        if i < self.front.size() {
            self.front.get(self.front.size() - i - 1)
        } else {
            self.back.get(i - self.front.size())
        }
    }

    fn set(&mut self, i: usize, x: T) -> T {
        if i < self.front.size() {
            self.front.set(self.front.size() - i - 1, x)
        } else {
            self.back.set(i - self.front.size(), x)
        }
    }

    fn add(&mut self, i: usize, x: T) {
        if i < self.front.size() {
            self.front.add(self.front.size() - i, x)
        } else {
            self.back.add(i - self.front.size(), x)
        }
        self.balance();
    }

    fn remove(&mut self, i: usize) -> T {
        let x: T = if i < self.front.size() {
            self.front.remove(self.front.size() - i - 1)
        } else {
            self.back.remove(i - self.front.size())
        };
        self.balance();
        x
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_list() {
        let mut array = DualArrayDeque::new(0);
        array.add(0, "a");
        array.add(1, "b");
        array.add(2, "c");
        array.add(3, "d");
        assert_eq!(array.front.a, vec!["a", ""].into_boxed_slice());
        assert_eq!(array.front.n, 1);
        assert_eq!(array.back.a, vec!["b", "c", "d", ""].into_boxed_slice());
        assert_eq!(array.back.n, 3);
        assert_eq!(array.size(), 4);

        array.add(3, "x");
        assert_eq!(array.front.a, vec!["b", "a", "", ""].into_boxed_slice());
        assert_eq!(array.front.n, 2);
        assert_eq!(
            array.back.a,
            vec!["c", "x", "d", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.back.n, 3);
        assert_eq!(array.size(), 5);

        array.add(4, "y");
        assert_eq!(array.front.a, vec!["b", "a", "", ""].into_boxed_slice());
        assert_eq!(array.front.n, 2);
        assert_eq!(
            array.back.a,
            vec!["c", "x", "y", "d", "", ""].into_boxed_slice()
        );
        assert_eq!(array.back.n, 4);
        assert_eq!(array.size(), 6);

        array.remove(0);
        assert_eq!(array.front.a, vec!["c", "b", "", ""].into_boxed_slice());
        assert_eq!(array.front.n, 2);
        assert_eq!(
            array.back.a,
            vec!["x", "y", "d", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.back.n, 3);
        assert_eq!(array.size(), 5);
    }
}
