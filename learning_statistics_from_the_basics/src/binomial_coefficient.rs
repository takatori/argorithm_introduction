
pub fn binom_fact(n: i32, k: i32) -> i32 {
    factorial(n) / factorial(k) / factorial(n-k)
}

fn factorial(i: i32) -> i32 {
    if i <= 1 {
        1
    } else {
        i * factorial(i-1)
    }
}