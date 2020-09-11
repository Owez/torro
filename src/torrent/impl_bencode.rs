//! Links [Torrent] to bencode parsing, allowing easy creation of said [Torrent]

use crate::bencode;
use crate::error;
use crate::torrent::Torrent;
use std::path::PathBuf;

impl Torrent {
    /// Creates a new [Torrent] from given `torrent_data` formatted as [Vec]<[u8]>
    pub fn new(torrent_data: Vec<u8>) -> Result<Self, error::TorroError> {
        let parsed_bencode = bencode::parse(torrent_data)?;

        Err(error::TorroError::Unimplemented)
    }

    /// Creates a new [Torrent] from given `.torrent` path
    pub fn from_path(path: PathBuf) -> Result<Self, error::TorroError> {
        Err(error::TorroError::Unimplemented)
    }
}
