pub fn is_power_of_2(n: usize) -> bool {
    if n == 0 {
        false
    } else {
        // very clever...
        (n & (n - 1)) == 0
    }
}
