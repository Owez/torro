use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use torro::bencode::parse;

const DATA_PATH_PREFIX: &str = "./tests/data/";

/// Tests the [Ubuntu 20.04 torrent](https://releases.ubuntu.com/20.04/ubuntu-20.04.1-live-server-amd64.iso.torrent)
/// that is downloaded in `tests/data/` for successful parsing
#[test]
fn ubuntu_20_04_torrent() {
    let torrent_path = PathBuf::from(format!(
        "{}ubuntu-20.04.1-live-server-amd64.iso.torrent",
        DATA_PATH_PREFIX
    ));

    parse(get_file(torrent_path)).unwrap(); // if panic, error occurred whilst parsing
}

/// Tests the [Tails 4.10 (AMD64)
/// torrent](https://tails.boum.org/torrents/files/tails-amd64-4.10.img.torrent)
/// that is downloaded in `tests/data` for successful parsing
#[test]
fn tails_4_10_torrent() {
    let torrent_path = PathBuf::from(format!("{}tails-amd64-4.10.img.torrent", DATA_PATH_PREFIX));

    parse(get_file(torrent_path)).unwrap(); // if panic, error occurred whilst parsing
}

/// Gets file from given [PathBuf], similar to the private
/// [torro::utils::read_file_bytes]
fn get_file(file: PathBuf) -> Vec<u8> {
    let mut file = File::open(file).unwrap();
    let mut contents = vec![];
    file.read_to_end(&mut contents).unwrap();

    contents
}
