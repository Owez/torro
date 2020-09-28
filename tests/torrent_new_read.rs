//! Ensures that [Torrent::from_file] can properly read a `.torrent` file

use std::path::PathBuf;
use torro::Torrent;

const DATA_PATH_PREFIX: &str = "./tests/data/";

/// Tests the proper reading for [Torrent::from_file] with the `tiny.torrent` file
#[test]
fn torrent_from_tinytorrent() {
    let file = PathBuf::from(format!("{}tiny.torrent", DATA_PATH_PREFIX));

    Torrent::from_file(file).unwrap();
}

/// Tests the proper reading for [Torrent::from_file] with the
/// `ubuntu-20.04.1-live-server-amd64.iso.torrent` file
#[test]
fn torrent_from_ubuntu() {
    let file = PathBuf::from(format!(
        "{}ubuntu-20.04.1-live-server-amd64.iso.torrent",
        DATA_PATH_PREFIX
    ));

    Torrent::from_file(file).unwrap();
}

/// Tests the proper reading for [Torrent::from_file] with the
/// `tails-amd64-4.10.img.torrent` file
#[test]
fn torrent_from_tails() {
    let file = PathBuf::from(format!("{}tails-amd64-4.10.img.torrent", DATA_PATH_PREFIX));

    Torrent::from_file(file).unwrap();
}
