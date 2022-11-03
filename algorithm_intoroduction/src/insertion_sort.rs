// 挿入ソート
// Θ(n^2)
fn insertion_sort(list: &mut Vec<i32>) {
    for j in 1..list.len() {
        let mut i = j;
        while i > 0 && list[i] < list[i - 1] {
            list.swap(i, i - 1);
            i = i - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::insertion_sort;

    #[test]
    fn test_insert_sort() {
        let mut input = vec![5, 2, 4, 6, 1, 3];
        insertion_sort(&mut input);
        let expected = vec![1, 2, 3, 4, 5, 6];
        assert_eq!(input, expected);
    }
}
