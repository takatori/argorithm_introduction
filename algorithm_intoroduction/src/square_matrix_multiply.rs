fn square_matrix_multiply(a: &[Vec<i32>], b: &[Vec<i32>]) -> Vec<Vec<i32>> {
    let n = a.len();
    let mut C = vec![vec![0; n]; n];

    for i in 0..n {
        for j in 0..n {
            let mut c = 0;
            for k in 0..n {
                c = c + a[i][k] * b[k][j]
            }
            C[i][j] = c;
        }
    }

    return C;
}

fn square_matrix_multiply_recursive(
    a: &[Vec<i32>],
    b: &[Vec<i32>],
    a_p: (usize, usize),
    b_p: (usize, usize),
    n: usize,
) -> Vec<Vec<i32>> {
    let mut c = vec![vec![0; n]; n];

    let size = n / 2;
    let (a_11_x, a_11_y) = a_p;
    let (a_12_x, a_12_y) = (a_p.0 + size, a_p.1);
    let (a_21_x, a_21_y) = (a_p.0, a_p.1 + size);
    let (a_22_x, a_22_y) = (a_p.0 + size, a_p.1 + size);

    let (b_11_x, b_11_y) = b_p;
    let (b_12_x, b_12_y) = (b_p.0 + size, b_p.1);
    let (b_21_x, b_21_y) = (b_p.0, b_p.1 + size);
    let (b_22_x, b_22_y) = (b_p.0 + size, b_p.1 + size);

    if n == 1 {
        c[0][0] = a[a_11_y][a_11_x] * b[b_11_y][b_11_y];
    } else {
        // C11 = A11 * B11 + A12 * B21
        add(
            &mut c,
            square_matrix_multiply_recursive(a, b, (a_11_x, a_11_y), (b_11_x, b_11_y), size),
            square_matrix_multiply_recursive(a, b, (a_12_x, a_12_y), (b_21_x, b_21_y), size),
            (0, 0),
        );

        // C12 = A11 * B12 + A12 * B22
        add(
            &mut c,
            square_matrix_multiply_recursive(a, b, (a_11_x, a_11_y), (b_12_x, b_12_y), size),
            square_matrix_multiply_recursive(a, b, (a_12_x, a_12_y), (b_22_x, b_22_y), size),
            (size, 0),
        );

        // C21 = A21 * B11 + A22 * B21
        add(
            &mut c,
            square_matrix_multiply_recursive(a, b, (a_21_x, a_21_y), (b_11_x, b_11_y), size),
            square_matrix_multiply_recursive(a, b, (a_22_x, a_22_y), (b_21_x, b_21_y), size),
            (0, size),
        );

        // C22 = A21 * B12 + A22 * B22
        add(
            &mut c,
            square_matrix_multiply_recursive(a, b, (a_21_x, a_21_y), (b_12_x, b_12_y), size),
            square_matrix_multiply_recursive(a, b, (a_22_x, a_22_y), (b_22_x, b_22_y), size),
            (size, size),
        );
    }
    println!("n: {}, a_p: {:?}, b_p:{:?}, c: {:?}", n, a_p, b_p, c);
    return c;
}

fn add(c: &mut Vec<Vec<i32>>, a: Vec<Vec<i32>>, b: Vec<Vec<i32>>, c_p: (usize, usize)) {
    let n = a.len();
    for i in 0..n {
        for j in 0..n {
            c[c_p.1 + i][c_p.0 + j] = a[i][j] + b[i][j];
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_square_matrix_multiply_recursive() {
        let a = &[
            vec![2, 3, 9, 2],
            vec![1, 4, 3, 9],
            vec![2, 1, 4, 8],
            vec![1, 2, 3, 1],
        ];
        let b = &[
            vec![3, 1, 2, 3],
            vec![3, 0, 9, 2],
            vec![4, 1, 4, 1],
            vec![3, 0, 0, 2],
        ];
        let actual = square_matrix_multiply_recursive(a, b, (0, 0), (0, 0), a.len());
        let expected = vec![
            vec![57, 11, 67, 25],
            vec![54, 4, 50, 32],
            vec![49, 6, 29, 28],
            vec![24, 4, 32, 12],
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_square_matrix_multiply() {
        let a = &[vec![2, 3, 9], vec![1, 4, 3], vec![2, 1, 4]];
        let b = &[vec![3, 1, 2], vec![2, 4, 2], vec![8, 2, 3]];
        let actual = square_matrix_multiply(a, b);
        let expected = vec![vec![84, 32, 37], vec![35, 23, 19], vec![40, 14, 18]];
        assert_eq!(actual, expected);
    }
}
