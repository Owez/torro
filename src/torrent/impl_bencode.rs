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
    /// ` piece` key
    Piece,
    /// `pieces` key
    Pieces,
    /// `announce_url` key
    AnnounceURL,
    /// `name` key
    Name,
    /// `length` key
    Length,
    /// `files` key
    Files,
}

impl TorrentBencodeKey {
    fn as_vecu8(&self) -> Vec<u8> {
        match &self {
            TorrentBencodeKey::Piece => "piece",
            TorrentBencodeKey::Pieces => "pieces",
            TorrentBencodeKey::AnnounceURL => "announce_url",
            TorrentBencodeKey::Name => "name",
            TorrentBencodeKey::Length => "length",
            TorrentBencodeKey::Files => "files",
        }
        .as_bytes()
        .to_vec()
    }

    /// Finds appropriate error to provide for downstream [get_dict_item] if an
    /// instance of [TorrentBencodeKey] is missing
    fn missing_err(&self) -> error::TorrentCreationError {
        match self {
            TorrentBencodeKey::Piece => error::TorrentCreationError::NoPieceFound,
            TorrentBencodeKey::Pieces => error::TorrentCreationError::NoPiecesFound,
            TorrentBencodeKey::AnnounceURL => error::TorrentCreationError::NoAnnounceURLFound,
            TorrentBencodeKey::Name => error::TorrentCreationError::NoNameFound,
            TorrentBencodeKey::Length | TorrentBencodeKey::Files => {
                error::TorrentCreationError::NoLengthFiles
            }
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
        None => Err(key.missing_err().into()),
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
                    other => return Err(error::TorrentCreationError::PieceWrongType(other).into()),
                };
                let pieces_raw = match get_dict_item(&dict_data, TorrentBencodeKey::Pieces)? {
                    Bencode::ByteString(found_pieces_raw) => found_pieces_raw,
                    other => return Err(error::TorrentCreationError::PiecesWrongType(other).into()),
                };
                let announce_url = match get_dict_item(&dict_data, TorrentBencodeKey::AnnounceURL)?
                {
                    Bencode::ByteString(found_announce_url) => found_announce_url,
                    other => {
                        return Err(error::TorrentCreationError::AnnounceURLWrongType(other).into())
                    }
                };
                let name = match get_dict_item(&dict_data, TorrentBencodeKey::Name)? {
                    Bencode::ByteString(found_name) => found_name,
                    other => return Err(error::TorrentCreationError::NameWrongType(other).into()),
                };

                let length: Option<i64> = match get_dict_item(&dict_data, TorrentBencodeKey::Length)
                {
                    Ok(length_raw) => match length_raw {
                        Bencode::Int(found_length) => Some(found_length),
                        other => {
                            return Err(error::TorrentCreationError::LengthWrongType(other).into())
                        }
                    },
                    Err(_) => None,
                };
                let files_raw: Option<BTreeMap<Vec<u8>, Bencode>> =
                    match get_dict_item(&dict_data, TorrentBencodeKey::Files) {
                        Ok(files_bencode) => match files_bencode {
                            Bencode::Dict(found_files_raw) => Some(found_files_raw),
                            other => {
                                return Err(
                                    error::TorrentCreationError::FilesWrongType(other).into()
                                )
                            }
                        },
                        Err(_) => None,
                    };

                if length.is_none() && files_raw.is_none() {
                    return Err(error::TorrentCreationError::NoLengthFiles.into());
                } else if length.is_some() && files_raw.is_some() {
                    return Err(error::TorrentCreationError::BothLengthFiles.into());
                }

                Err(error::TorroError::Unimplemented) // TODO: finish
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

    /// Tests that [TorrentBencodeKey::AnnounceURL] returns the wrong type
    /// correctly as an error
    #[test]
    fn announce_url_badtype() {
        assert_eq!(
            Torrent::new(
                "d12:announce_urli64e5:piecei0e6:pieces0:e"
                    .as_bytes()
                    .to_vec()
            ),
            Err(error::TorrentCreationError::AnnounceURLWrongType(Bencode::Int(64)).into())
        );
    }

    /// Tests that [TorrentBencodeKey::Piece] returns the wrong type correctly
    /// as an error
    #[test]
    fn piece_badtype() {
        assert_eq!(
            Torrent::new(
                "d12:announce_url0:5:piece5:wrong6:pieces0:e"
                    .as_bytes()
                    .to_vec()
            ),
            Err(
                error::TorrentCreationError::PieceWrongType(Bencode::ByteString(
                    "wrong".as_bytes().to_vec()
                ))
                .into()
            )
        );
    }

    /// Tests that [TorrentBencodeKey::Pieces] returns the wrong type correctly
    /// as an error
    #[test]
    fn pieces_badtype() {
        assert_eq!(
            Torrent::new(
                "d12:announce_url0:5:piecei0e6:piecesi9999ee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(error::TorrentCreationError::PiecesWrongType(Bencode::Int(9999)).into())
        );
    }

    /// Tests that all [TorrentBencodeKey]'s are correctly reported missing when
    /// non-existant
    #[test]
    fn missing_torrent_types() {
        assert_eq!(
            Torrent::new("d5:piecei0e6:pieces0:e".as_bytes().to_vec()),
            Err(error::TorrentCreationError::NoAnnounceURLFound.into())
        );
        assert_eq!(
            Torrent::new("d12:announce_url0:6:pieces0:e".as_bytes().to_vec()),
            Err(error::TorrentCreationError::NoPieceFound.into())
        );
        assert_eq!(
            Torrent::new("d12:announce_url0:5:piecei0ee".as_bytes().to_vec()),
            Err(error::TorrentCreationError::NoPiecesFound.into())
        );
    }
}
