//! Links [Torrent] to bencode parsing and file digestion (pulling bytes from
//! given [PathBuf]) for easy creation

use crate::bencode::{self, Bencode};
use crate::error;
use crate::torrent::Torrent;
use crate::utils::read_file_bytes;
use std::path::PathBuf;

impl Torrent {
    /// Creates a new [Torrent] from given `torrent_data` formatted as [Vec]<[u8]>
    pub fn new(torrent_data: Vec<u8>) -> Result<Self, error::TorroError> {
        let parsed_bencode = bencode::parse(torrent_data)?;

        match parsed_bencode {
            Bencode::Dict(dict_data) => {
                let mut piece: Option<usize> = None;
                let mut pieces_str: Option<String> = None;

                Err(error::TorroError::Unimplemented)
            }
            _ => Err(error::TorrentCreationError::NoTLDictionary.into()),
        }
    }

    /// Creates a new [Torrent] from given `.torrent` file path
    pub fn from_file(file: PathBuf) -> Result<Self, error::TorroError> {
        match read_file_bytes(&file) {
            Ok(bytes) => Ok(Torrent::new(bytes)?),
            Err(_) => Err(error::TorroError::BadFileRead(file)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests purposefully false data on [Torrent] to ensure correct errors
    #[test]
    fn torrent_new_err() {
        assert_eq!(
            Torrent::new("i64e".as_bytes().to_vec()),
            Err(error::TorrentCreationError::NoTLDictionary.into())
        );
        assert_eq!(
            Torrent::new("ldee".as_bytes().to_vec()),
            Err(error::TorrentCreationError::NoTLDictionary.into())
        );
    }
}
