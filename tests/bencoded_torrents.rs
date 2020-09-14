use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use torro::bencode::parse;

/// Tests the [Ubuntu 20.04 torrent](https://releases.ubuntu.com/20.04/ubuntu-20.04.1-live-server-amd64.iso.torrent)
/// that is downloaded in `tests/data/` for successful parsing
#[test]
fn ubuntu_20_04_torrent() {
    let torrent_path = PathBuf::from("./tests/data/ubuntu-20.04.1-live-server-amd64.iso.torrent");

    parse(get_file(torrent_path)).unwrap(); // if panic, error occurred whilst parsing
}

/// Gets file from given [PathBuf], similar to the private
/// [torro::utils::read_file_bytes]
pub fn get_file(file: PathBuf) -> Vec<u8> {
    let mut file = File::open(file).unwrap();
    let mut contents = vec![];
    file.read_to_end(&mut contents).unwrap();

    contents
}
