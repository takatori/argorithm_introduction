
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

fn square_matrix_multiply_recursive(a: Vec<Vec<i32>>, b: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
    let n = a.len();
    let mut C = vec![vec![0; n]; n];

    if n == 1 {
        C[0][0] = a[0][0] * b[0][0];
    } else {
        
    }
    return C;
}