//! Links [Torrent] to bencode parsing and file digestion (pulling bytes from
//! given [PathBuf]) for easy creation

use crate::bencode;
use crate::error;
use crate::torrent::Torrent;
use crate::utils::read_file_bytes;
use std::path::PathBuf;

impl Torrent {
    /// Creates a new [Torrent] from given `torrent_data` formatted as [Vec]<[u8]>
    pub fn new(torrent_data: Vec<u8>) -> Result<Self, error::TorroError> {
        let parsed_bencode = bencode::parse(torrent_data)?;

        Err(error::TorroError::Unimplemented)
    }

    /// Creates a new [Torrent] from given `.torrent` file path
    pub fn from_file(file: PathBuf) -> Result<Self, error::TorroError> {
        match read_file_bytes(&file) {
            Ok(bytes) => Ok(Torrent::new(bytes)?),
            Err(_) => Err(error::TorroError::BadFileRead(file)),
        }
    }
}
