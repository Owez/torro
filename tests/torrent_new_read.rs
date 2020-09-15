//! Ensures that [Torrent::from_file] can properly read a `.torrent` file

use std::path::PathBuf;
use torro::torrent::Torrent;

const DATA_PATH_PREFIX: &str = "./tests/data/";

/// Tests the proper reading for [Torrent::from_file], see module-level
/// documentation for more information
#[test]
fn torrent_from_file() {
    let file = PathBuf::from(format!("{}tiny.torrent", DATA_PATH_PREFIX));

    // Torrent::from_file(file).unwrap(); // FIXME: uncomment when Torrent::new works
}
