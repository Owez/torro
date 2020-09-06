//! Internal/private utilities like (insecure) RNG for use globally inside torro

use crate::CLIENT_PREFIX;
use std::time::{SystemTime, UNIX_EPOCH};

/// Pseudorandom time-based 1 pass xorshift
///
/// **Do not use this function for crpytographically secure usages or anywhere
/// where leaking of creation date is unwelcome. If this is used inside of a
/// public ID, create a new one for each torrent, not a single id for entire
/// lifetime of client**
pub fn randish() -> u128 {
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();

    seed = seed << 13;
    seed = seed >> 4;

    seed << 5
}

/// Generates torro id using [randish]
///
/// **WARNING: THIS CAN LEAK CREATION TIME AND IS NOT SECURE, SEE [randish] FOR
/// MORE DETAILS**
pub fn generate_torro_id() -> String {
    let mut rand_num = format!("{}{}", CLIENT_PREFIX, randish());

    if rand_num.len() > 20 {
        rand_num.drain(20..);
    } else {
        rand_num = format!("{}{}", rand_num, "0".repeat(20 - rand_num.len()))
    }

    rand_num
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks that multiple calls to [xorshift128] don't result in same number
    #[test]
    fn xorshift128_nodupe() {
        for _ in 0..1000 {
            assert_ne!(randish(), randish());
        }
    }

    #[test]
    fn check_torro_id() {
        for _ in 0..1000 {
            assert_eq!(generate_torro_id().len(), 20);
        }
    }
}
