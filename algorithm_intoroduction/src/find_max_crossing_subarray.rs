fn find_max_subarray(list: &[i32], low: usize, high: usize) -> (usize, usize, i32) {
    if high == low {
        return (low, high, list[low]);
    } else {
        // 配列を3つの部分配列に分割する
        let mid = (low + high) / 2;
        let (left_low, left_high, left_sum) = find_max_subarray(list, low, mid);
        let (right_low, right_high, righ_sum) = find_max_subarray(list, mid + 1, high);
        let (cross_low, cross_high, cross_sum) = find_max_crossing_subarray(list, low, mid, high);

        // 最大の部分配列を取り出す
        if left_sum >= righ_sum && left_sum >= cross_sum {
            (left_low, left_high, left_sum)
        } else if righ_sum >= left_sum && righ_sum >= cross_sum {
            (right_low, right_high, righ_sum)
        } else {
            (cross_low, cross_high, cross_sum)
        }
    }
}

// 配列の中央点を跨ぐ最大部分配列を発見する関数
// 中央点を跨ぐ最大部分配列の境界を定める2つの添字と最大部分配列に属する要素の和の3つの要素を返す
fn find_max_crossing_subarray(
    list: &[i32],
    low: usize,
    mid: usize,
    high: usize,
) -> (usize, usize, i32) {
    // 左半分の最大部分配列を決定する
    // 必ず中央を跨ぐのでmid -> lowにむけて左側に進んでいって
    // 一番合計が大きくなるところの添字を探せばよい
    let mut left_sum = std::i32::MIN;
    let mut sum = 0;
    let mut max_left = 0;
    for i in (low..=mid).rev() {
        sum = sum + list[i];
        if sum > left_sum {
            left_sum = sum;
            max_left = i;
        }
    }

    // 右半分の最大部分配列を決定する
    let mut right_sum = std::i32::MIN;
    sum = 0;
    let mut max_right = 0;
    for j in (mid + 1)..=high {
        sum = sum + list[j];
        if sum > right_sum {
            right_sum = sum;
            max_right = j;
        }
    }

    (max_left, max_right, left_sum + right_sum)
}

#[cfg(test)]
mod tests {
    use super::find_max_subarray;

    #[test]
    fn test_find_max_subarray() {
        let input = vec![
            13, -3, -25, 20, -3, -16, -23, 18, 20, -7, 12, -5, -22, 15, -4, 7,
        ];
        let (low, high, sum) = find_max_subarray(&input, 0, input.len() - 1);
        assert_eq!(low, 7);
        assert_eq!(high, 10);
        assert_eq!(sum, 43);
    }
}
