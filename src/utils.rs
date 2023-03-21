use ethers_core::types::{Address, Bytes, U256};

pub fn count_leading_zeroes(address: Address) -> u8 {
    let mut leading_zeros = 0;
    for (_i, c) in format!("{:?}", address).chars().skip(2).enumerate() {
        if c == '0' {
            leading_zeros += 1;
        } else {
            break;
        }
    }
    leading_zeros
}

pub fn bytes32(n: U256) -> Bytes {
    let mut bytes = [0u8; 32];
    n.to_big_endian(&mut bytes);
    Bytes::from(bytes)
}

pub fn fmt_dms(seconds: u128) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{}d{}h{}m{}s", days, hours, minutes, seconds)
}

pub fn countdown(zeros: u8, rate: f64, elapsed_ms: u128) -> String {
    let expected_attempts: u128 = 16_u128.pow(zeros as u32);
    let expected_attempts_secs_at_current_rate = expected_attempts as f64 / rate;
    let expected_remaining_time_at_rate =
        (expected_attempts_secs_at_current_rate - (elapsed_ms as f64 / 1000.0)) as u128;
    return format!(
        " ({} 0s T-{})",
        zeros,
        fmt_dms(expected_remaining_time_at_rate)
    );
}
