/// helper function to round a number to given decimal places.
pub fn round_dp(value: f64, dp: usize) -> f64 {
    let x = 10u32.pow(dp as u32) as f64;
    (value * x).round() / x 
}