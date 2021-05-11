//! Misc helper utilities
//!
pub mod bufmath;


/// Trim leading zeroes from a slice.
///
pub fn trim(bytes: &[u8]) -> &[u8] {
    let start = bytes.iter().position(|b| *b != 0)
        .unwrap_or(bytes.len());
    &bytes[start..]
}
