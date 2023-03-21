use ethers_core::types::{Address, Bytes, U256};

/// Convert a U256 to a 32-byte array
///
/// # Arguments
/// * `n` - The U256 to convert
///
/// # Returns
/// A 32-byte array
///
pub fn bytes32(n: U256) -> Bytes {
    let mut bytes = [0u8; 32];
    n.to_big_endian(&mut bytes);
    Bytes::from(bytes)
}

/// Count the number of leading zeroes in an address
///
/// # Arguments
/// * `address` - The address to count the leading zeroes in
///
/// # Returns
/// The number of leading zeroes
///
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

/// Format a number of seconds into days, hours, minutes, seconds
///
/// # Arguments
/// * `seconds` - The number of seconds to format
///
/// # Returns
/// A string in the format "XdYhZmSs" where X is the number of days,
/// Y is the number of hours, Z is the number of minutes, and S is
/// the number of seconds
pub fn fmt_dms(seconds: u128) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{}d{}h{}m{}s", days, hours, minutes, seconds)
}
