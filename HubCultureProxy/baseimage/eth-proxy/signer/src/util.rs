use std::time::{SystemTime,UNIX_EPOCH};


/// Get current unix timestamp
///
/// ## Panics
///
/// This function panics if called prior to the unix epoch
///
pub fn unix_time() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH)
        .expect("Must be called after the unix epoch")
        .as_secs()
}


