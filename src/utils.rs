//! Internal/private utilities like (insecure) RNG for use globally inside torro

use crate::CLIENT_PREFIX;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Pseudorandom 128-bit time-based 1 pass xorshift
///
/// # Usage notice
///
/// Do not use this function for crpytographically secure usages or anywhere
/// where leaking of creation date is unwelcome.
///
/// If this is used inside of a public ID, create a new one for each torrent,
/// not a single id for entire lifetime of client
pub fn randish_128() -> u128 {
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();

    seed = seed << 13;
    seed = seed >> 4;

    seed << 5
}

/// Generates torro id using [randish_128]
///
/// **WARNING: THIS CAN LEAK CREATION TIME AND IS NOT SECURE, SEE [randish_128] FOR
/// MORE DETAILS**
pub fn generate_torro_id() -> String {
    let mut rand_num = format!("{}{}", CLIENT_PREFIX, randish_128());

    if rand_num.len() > 20 {
        rand_num.drain(20..);
    } else {
        rand_num = format!("{}{}", rand_num, "0".repeat(20 - rand_num.len()))
    }

    rand_num
}

/// Gets bytes from given `file` &[PathBuf] or returns a [std::io::Error]
///
/// A reference is used for `file` for optimisations with
/// [torrent::Torrent::from_file]'s passed [PathBuf]
pub fn read_file_bytes(file: &PathBuf) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(file)?;
    let mut contents = vec![];

    file.read_to_end(&mut contents)?;

    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time;

    /// Checks that multiple calls to [xorshift128] don't result in same number
    #[test]
    fn xorshift128_nodupe() {
        for _ in 0..100 {
            let first_collect = randish_128();

            thread::sleep(time::Duration::from_millis(1));

            assert_ne!(first_collect, randish_128());
        }
    }

    #[test]
    fn check_torro_id() {
        for _ in 0..1000 {
            assert_eq!(generate_torro_id().len(), 20);
        }
    }
}
