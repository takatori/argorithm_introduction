// マージソート
fn merge_sort(list: &mut Vec<i32>, p: usize, r: usize) {
    if p < r {
        let q = (p + r) / 2;
        merge_sort(list, p, q);
        merge_sort(list, q + 1, r);
        merge(list, p, q, r);
    }
}

fn merge(list: &mut Vec<i32>, p: usize, q: usize, r: usize) {
    let n1 = q - p + 1;
    let n2 = r - q;

    let mut left = vec![0; n1 + 1];
    let mut right = vec![0; n2 + 1];

    for i in 0..n1 {
        left[i] = list[p + i];
    }

    for j in 0..n2 {
        right[j] = list[q + j + 1];
    }

    // 番兵(sentinel)
    left[n1] = std::i32::MAX;
    right[n2] = std::i32::MAX;

    let mut i = 0;
    let mut j = 0;

    for k in p..=r {
        if left[i] <= right[j] {
            list[k] = left[i];
            i = i + 1;
        } else {
            list[k] = right[j];
            j = j + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::merge_sort;

    #[test]
    fn test_merge_sort() {
        let mut input = vec![5, 2, 4, 6, 1, 3];
        let r = input.len() - 1;
        merge_sort(&mut input, 0, r);
        let expected = vec![1, 2, 3, 4, 5, 6];
        assert_eq!(input, expected);
    }
}
