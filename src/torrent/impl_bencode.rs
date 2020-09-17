//! Links [Torrent] to bencode parsing and file digestion (pulling bytes from
//! given [PathBuf]) for easy creation

use crate::bencode::{self, Bencode};
use crate::error;
use crate::torrent::Torrent;
use crate::utils::read_file_bytes;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Used as an organisation enum for managing [Torrent::new] when pulling from a
/// bencode dict, see [get_dict_item] for the main usage of this emum
enum TorrentBencodeKey {
    Piece,
    Pieces,
    AnnounceURL,
}

impl TorrentBencodeKey {
    fn as_vecu8(&self) -> Vec<u8> {
        match &self {
            TorrentBencodeKey::Piece => "piece",
            TorrentBencodeKey::Pieces => "pieces",
            TorrentBencodeKey::AnnounceURL => "announce_url",
        }
        .as_bytes()
        .to_vec()
    }

    /// Finds appropriate error to provide for downstream [get_dict_item] if an
    /// instance of [TorrentBencodeKey] is missing
    fn missing_err(&self) -> error::TorroError {
        match self {
            TorrentBencodeKey::Piece => {
                error::TorroError::TorrentCreationError(error::TorrentCreationError::NoPieceFound)
            }
            TorrentBencodeKey::Pieces => {
                error::TorroError::TorrentCreationError(error::TorrentCreationError::NoPiecesFound)
            }
            TorrentBencodeKey::AnnounceURL => error::TorroError::TorrentCreationError(
                error::TorrentCreationError::NoAnnounceURLFound,
            ),
        }
    }
}

/// Gets a dict value from given key or returns appropriate error based upon
/// [TorrentBencodeKey]
fn get_dict_item(
    dict: &BTreeMap<Vec<u8>, Bencode>,
    key: TorrentBencodeKey,
) -> Result<Bencode, error::TorroError> {
    match dict.get(&key.as_vecu8()) {
        Some(value) => Ok(value.clone()),
        None => Err(key.missing_err()),
    }
}

impl Torrent {
    /// Creates a new [Torrent] from given `torrent_data` formatted as [Vec]<[u8]>
    ///
    /// If an error is encountered, it will be a
    /// [TorrentCreationError](error::TorrentCreationError) wrapped inside of
    /// [TorroError::TorrentCreationError](error::TorroError::TorrentCreationError)
    pub fn new(torrent_data: Vec<u8>) -> Result<Self, error::TorroError> {
        let parsed_bencode = bencode::parse(torrent_data)?;

        match parsed_bencode {
            Bencode::Dict(dict_data) => {
                let piece = match get_dict_item(&dict_data, TorrentBencodeKey::Piece)? {
                    Bencode::Int(found_piece) => found_piece,
                    other => {
                        return Err(
                            error::TorrentCreationError::PieceWrongType(other.clone()).into()
                        )
                    }
                };
                let pieces_raw = match get_dict_item(&dict_data, TorrentBencodeKey::Pieces)? {
                    Bencode::ByteString(found_pieces_raw) => found_pieces_raw,
                    other => {
                        return Err(
                            error::TorrentCreationError::PiecesWrongType(other.clone()).into()
                        )
                    }
                };
                let announce_url = get_dict_item(&dict_data, TorrentBencodeKey::AnnounceURL)?;

                // TODO: finish

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
