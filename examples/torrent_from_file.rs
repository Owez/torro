//! A small example of making an interactable `Torrent` structure from a path

use std::path::PathBuf;
use torro::Torrent;

fn main() {
    let file_path = PathBuf::from("example.torrent");
    let my_torrent = Torrent::from_file(file_path).unwrap();

    println!("Torrent name: '{}'", my_torrent.name);
}
