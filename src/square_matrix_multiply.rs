
fn square_matrix_multiply(a: Vec<Vec<i32>>, b: Vec<Vec<i32>>) -> Vec<Vec<i32>> {

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

    return C
}

fn square_matrix_multiply_recursive(c: &mut Vec<Vec<i32>>, a: &[&[i32]], b: &[&[i32]], a_p: (usize, usize), b_p: (usize, usize), n: usize) {
    // let n = a.len();
    // let mut C = vec![vec![0; n]; n];
    let (a_11_x, a_11_y) =  a_p;
    let (a_12_x, a_12_y) = (a_p.0 + n/2, a_p.1);
    let (a_21_x, a_21_y) = (a_p.0, a_p.1 + n/2);
    let (a_22_x, a_22_y) = (a_p.0 + n/2, a_p.1 + n/2);

    let (b_11_x, b_11_y) =  b_p;
    let (b_12_x, b_12_y) = (b_p.0 + n/2, b_p.1);
    let (b_21_x, b_21_y) = (b_p.0, b_p.1 + n/2);
    let (b_22_x, b_22_y) = (b_p.0 + n/2, b_p.1 + n/2);    

    if n == 1 {
        c[0][0] = a[a_11_y][a_11_x] * b[b_11_y][b_11_y];
    } else {
        for i in a_11_y..a_11_y+n/2 {
            for j in a_11_x..a_11_x+n/2 {
                square_matrix_multiply_recursive(c, a, b, (a_11_x, a_11_y), (b_11_x, b_11_y),n/2);
                square_matrix_multiply_recursive(c, a, b, (a_12_x, a_12_y), (b_21_x, b_21_y), n/2);
                c[i][j] = ???
            }
        }
    }

    // return C;
}