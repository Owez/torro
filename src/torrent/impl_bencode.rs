//! Links [Torrent] to bencode parsing and file digestion (pulling bytes from
//! given [PathBuf]) for easy creation

use crate::bencode::{self, Bencode};
use crate::error;
use crate::torrent::{Torrent, TorrentFile};
use crate::utils::read_file_bytes;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Used as an organisation enum for managing [Torrent::new] when pulling from a
/// bencode dict, see [get_dict_item] for the main usage of this emum
enum TorrentBencodeKey {
    /// `announce` top-level key
    Announce,
    /// `info` top-level key
    Info,
    /// `piece` key inside of the [TorrentBencodeKey::Info] dictionary
    Piece,
    /// `piece length` key which isn't part of the BEP0003 standard but used
    /// commonly anyway instead of [TorrentBencodeKey::Piece] for some reason.
    /// Fits inside of the [TorrentBencodeKey::Info] dictionary
    PieceLength,
    /// `pieces` key inside of the [TorrentBencodeKey::Info] dictionary
    Pieces,
    /// `name` key inside of the [TorrentBencodeKey::Info] dictionary
    Name,
    /// `length` key inside of the [TorrentBencodeKey::Info] dictionary or a
    /// dictionary inside of the [TorrentBencodeKey::Files] list
    Length,
    /// `files` key inside of the [TorrentBencodeKey::Info] dictionary
    Files,
    /// `path` key inside of a element of the [TorrentBencodeKey::Files] list
    Path,
}

impl TorrentBencodeKey {
    fn as_vecu8(&self) -> Vec<u8> {
        match &self {
            TorrentBencodeKey::Announce => "announce",
            TorrentBencodeKey::Info => "info",
            TorrentBencodeKey::Piece => "piece",
            TorrentBencodeKey::PieceLength => "piece length",
            TorrentBencodeKey::Pieces => "pieces",
            TorrentBencodeKey::Name => "name",
            TorrentBencodeKey::Length => "length",
            TorrentBencodeKey::Files => "files",
            TorrentBencodeKey::Path => "path",
        }
        .as_bytes()
        .to_vec()
    }

    /// Finds appropriate error to provide for downstream [get_dict_item] if an
    /// instance of [TorrentBencodeKey] is missing
    fn missing_err(&self) -> error::TorrentCreationError {
        match self {
            TorrentBencodeKey::Announce => error::TorrentCreationError::NoAnnounceFound,
            TorrentBencodeKey::Info => error::TorrentCreationError::NoInfoFound,
            TorrentBencodeKey::Piece | TorrentBencodeKey::PieceLength => {
                error::TorrentCreationError::NoPieceFound
            }
            TorrentBencodeKey::Pieces => error::TorrentCreationError::NoPiecesFound,
            TorrentBencodeKey::Name => error::TorrentCreationError::NoNameFound,
            TorrentBencodeKey::Length | TorrentBencodeKey::Files => {
                error::TorrentCreationError::NoLengthFiles
            }
            TorrentBencodeKey::Path => error::TorrentCreationError::NoPathFound,
        }
    }
}

/// Gets a dict value from given key or returns appropriate error based upon
/// [TorrentBencodeKey]
fn get_dict_item(
    dict: &BTreeMap<Vec<u8>, Bencode>,
    key: TorrentBencodeKey,
) -> Result<Bencode, error::TorrentCreationError> {
    match dict.get(&key.as_vecu8()) {
        Some(value) => Ok(value.clone()),
        None => Err(key.missing_err()),
    }
}

/// Wraps [String::from_utf8] inside a convinient
/// `Result<String, error::TorrentCreationError>` for simplified `.into()`/`?`
/// error processing
fn vecu8_to_string(input: Vec<u8>) -> Result<String, error::TorrentCreationError> {
    // TODO: better solution then `.clone()`
    match String::from_utf8(input.clone()) {
        Ok(value) => Ok(value),
        Err(_) => Err(error::TorrentCreationError::BadUTF8String(input)),
    }
}

/// Makes a new element for [TorrentFile::MultiFile] from given unparsed, raw
/// `file_raw` [Bencode::Dict]. It is not required to check the `file_raw`
/// [Bencode] type beforehand, this method will do for you
fn make_multifile(file_raw: Bencode) -> Result<(usize, Vec<String>), error::TorrentCreationError> {
    match file_raw {
        Bencode::Dict(file_dict) => {
            let length = match get_dict_item(&file_dict, TorrentBencodeKey::Length)? {
                Bencode::Int(found_length) => found_length as usize,
                other => return Err(error::TorrentCreationError::LengthWrongType(other)),
            };
            let path_raw_vec = match get_dict_item(&file_dict, TorrentBencodeKey::Path)? {
                Bencode::List(found_path_raw_vec) => found_path_raw_vec,
                other => return Err(error::TorrentCreationError::PathWrongType(other)),
            };

            if path_raw_vec.len() == 0 {
                return Err(error::TorrentCreationError::NoPathFound);
            }

            let mut path = vec![];

            for subdir_raw in path_raw_vec {
                path.push(match subdir_raw {
                    Bencode::ByteString(found_subdir) => vecu8_to_string(found_subdir)?,
                    other => return Err(error::TorrentCreationError::SubdirWrongType(other)),
                })
            }

            Ok((length, path))
        }
        other => return Err(error::TorrentCreationError::FileWrongType(other)),
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
                // top-level dictionary
                let announce = match get_dict_item(&dict_data, TorrentBencodeKey::Announce)? {
                    Bencode::ByteString(found_announce) => vecu8_to_string(found_announce)?,
                    other => {
                        return Err(error::TorrentCreationError::AnnounceWrongType(other).into())
                    }
                };
                let info_dict = match get_dict_item(&dict_data, TorrentBencodeKey::Info)? {
                    Bencode::Dict(found_info) => found_info,
                    other => return Err(error::TorrentCreationError::InfoWrongType(other).into()),
                };

                // inside `info` dictionary
                // (see [TorrentBencodeKey::PieceLength] as to why this is ugly)
                let piece = match get_dict_item(&info_dict, TorrentBencodeKey::Piece) {
                    Ok(piece_raw) => match piece_raw {
                        Bencode::Int(found_piece) => found_piece,
                        other => {
                            return Err(error::TorrentCreationError::PieceWrongType(other).into())
                        }
                    },
                    Err(_) => match get_dict_item(&info_dict, TorrentBencodeKey::PieceLength)? {
                        Bencode::Int(found_piece) => found_piece,
                        other => {
                            return Err(error::TorrentCreationError::PieceWrongType(other).into())
                        }
                    },
                };
                let pieces_raw = match get_dict_item(&info_dict, TorrentBencodeKey::Pieces)? {
                    Bencode::ByteString(found_pieces_raw) => found_pieces_raw,
                    other => return Err(error::TorrentCreationError::PiecesWrongType(other).into()),
                };
                let name = match get_dict_item(&info_dict, TorrentBencodeKey::Name)? {
                    Bencode::ByteString(found_name) => vecu8_to_string(found_name)?,
                    other => return Err(error::TorrentCreationError::NameWrongType(other).into()),
                };
                let length: Option<i64> = match get_dict_item(&info_dict, TorrentBencodeKey::Length)
                {
                    Ok(length_raw) => match length_raw {
                        Bencode::Int(found_length) => Some(found_length),
                        other => {
                            return Err(error::TorrentCreationError::LengthWrongType(other).into())
                        }
                    },
                    Err(_) => None,
                };
                let files_raw: Option<Vec<Bencode>> =
                    match get_dict_item(&info_dict, TorrentBencodeKey::Files) {
                        Ok(files_bencode) => match files_bencode {
                            Bencode::List(found_files_raw) => Some(found_files_raw),
                            other => {
                                return Err(
                                    error::TorrentCreationError::FilesWrongType(other).into()
                                )
                            }
                        },
                        Err(_) => None,
                    };

                let file_structure = if files_raw.is_some() {
                    if length.is_some() {
                        return Err(error::TorrentCreationError::BothLengthFiles.into());
                    }

                    let mut files = vec![];

                    for file_raw in files_raw.unwrap() {
                        files.push(make_multifile(file_raw)?);
                    }

                    TorrentFile::MultiFile(files)
                } else if length.is_some() {
                    TorrentFile::Single(length.unwrap() as usize)
                } else {
                    return Err(error::TorrentCreationError::NoLengthFiles.into());
                };

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

    /// Tests that [TorrentBencodeKey::Announce] returns the wrong type
    /// correctly as an error
    #[test]
    fn announce_badtype() {
        assert_eq!(
            Torrent::new("d8:announcei0e5:piecei0e6:pieces0:e".as_bytes().to_vec()),
            Err(error::TorrentCreationError::AnnounceWrongType(Bencode::Int(0)).into())
        );
    }

    /// Tests that [TorrentBencodeKey::Name] returns the wrong type correctly as
    /// an error
    #[test]
    fn name_badtype() {
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod4:namei0e5:piecei0e6:pieces0:6:lengthi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(error::TorrentCreationError::NameWrongType(Bencode::Int(0)).into())
        )
    }

    /// Tests that [TorrentBencodeKey::Files] returns the wrong type correctly
    /// as an error
    #[test]
    fn files_badtype() {
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod4:name12:test_torrent5:piecei0e6:pieces0:5:filesi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(error::TorrentCreationError::FilesWrongType(Bencode::Int(0)).into())
        )
    }

    /// Tests that a file element inside of [TorrentBencodeKey::Files] returns
    /// the wrong type correctly as an error
    #[test]
    fn file_element_badtype() {
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod4:name12:test_torrent5:piecei0e6:pieces0:5:filesli0eeee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(error::TorrentCreationError::FileWrongType(Bencode::Int(0)).into())
        )
    }

    /// Tests that the `length` element of a file inside of [TorrentBencodeKey::Files]
    /// returns the wrong type correctly as an error
    #[test]
    fn length_file_element_badtype() {
        assert_eq!(
            Torrent::new("d8:announce0:4:infod4:name12:test_torrent5:piecei0e6:pieces0:5:filesld6:length0:4:pathl0:eeeee".as_bytes().to_vec()),
            Err(error::TorrentCreationError::LengthWrongType(Bencode::ByteString(vec![])).into())
        )
    }

    /// Tests that the `path` element of a file inside of [TorrentBencodeKey::Files]
    /// returns the wrong type correctly as an error
    #[test]
    fn path_file_element_badtype() {
        assert_eq!(
            Torrent::new("d8:announce0:4:infod4:name12:test_torrent5:piecei0e6:pieces0:5:filesld6:lengthi0e4:pathi0eeeee".as_bytes().to_vec()),
            Err(error::TorrentCreationError::PathWrongType(Bencode::Int(0)).into())
        )
    }

    /// Tests that [TorrentBencodeKey::Piece] returns the wrong type correctly
    /// as an error
    #[test]
    fn piece_badtype() {
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod5:piece5:wrong6:pieces0:ee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(
                error::TorrentCreationError::PieceWrongType(Bencode::ByteString(
                    "wrong".as_bytes().to_vec()
                ))
                .into()
            )
        ); // `piece`
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod12:piece length5:wrong6:pieces0:ee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(
                error::TorrentCreationError::PieceWrongType(Bencode::ByteString(
                    "wrong".as_bytes().to_vec()
                ))
                .into()
            )
        ); // `piece length`
    }

    /// Tests that [TorrentBencodeKey::Pieces] returns the wrong type correctly
    /// as an error
    #[test]
    fn pieces_badtype() {
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod5:piecei0e6:piecesi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(error::TorrentCreationError::PiecesWrongType(Bencode::Int(0)).into())
        );
    }

    /// Tests that [TorrentBencodeKey::Info] returns the wrong type correctly
    /// as an error
    #[test]
    fn info_badtype() {
        assert_eq!(
            Torrent::new("d8:announce0:4:infoi0ee".as_bytes().to_vec()),
            Err(error::TorrentCreationError::InfoWrongType(Bencode::Int(0)).into())
        );
    }

    /// Tests that all [TorrentBencodeKey]'s are correctly reported missing when
    /// non-existant
    #[test]
    fn missing_torrent_types() {
        assert_eq!(
            Torrent::new("d8:announce0:e".as_bytes().to_vec()),
            Err(error::TorrentCreationError::NoInfoFound.into())
        );
        assert_eq!(
            Torrent::new(
                "d4:infod4:name12:test_torrent5:piecei0e6:pieces0:6:lengthi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(error::TorrentCreationError::NoAnnounceFound.into())
        );
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod5:piecei0e6:pieces0:6:lengthi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(error::TorrentCreationError::NoNameFound.into())
        );
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod4:name12:test_torrent6:pieces0:6:lengthi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(error::TorrentCreationError::NoPieceFound.into())
        );
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod4:name12:test_torrent5:piecei0e6:lengthi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(error::TorrentCreationError::NoPiecesFound.into())
        );
    }
}
