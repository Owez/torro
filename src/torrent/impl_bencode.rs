//! Links [Torrent] to bencode parsing and file digestion (pulling bytes from
//! given [PathBuf]) for easy creation

use crate::bencode::{self, Bencode};
use crate::error::{TorrentCreationError, TorroError};
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
    /// `piece length` key inside of the [TorrentBencodeKey::Info] dictionary
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
    fn missing_err(&self) -> TorrentCreationError {
        match self {
            TorrentBencodeKey::Announce => TorrentCreationError::NoAnnounceFound,
            TorrentBencodeKey::Info => TorrentCreationError::NoInfoFound,
            TorrentBencodeKey::PieceLength => TorrentCreationError::NoPieceLengthFound,
            TorrentBencodeKey::Pieces => TorrentCreationError::NoPiecesFound,
            TorrentBencodeKey::Name => TorrentCreationError::NoNameFound,
            TorrentBencodeKey::Length | TorrentBencodeKey::Files => {
                TorrentCreationError::NoLengthFiles
            }
            TorrentBencodeKey::Path => TorrentCreationError::NoPathFound,
        }
    }
}

/// Gets a dict value from given key or returns appropriate error based upon
/// [TorrentBencodeKey]
fn get_dict_item(
    dict: &BTreeMap<Vec<u8>, Bencode>,
    key: TorrentBencodeKey,
) -> Result<Bencode, TorrentCreationError> {
    match dict.get(&key.as_vecu8()) {
        Some(value) => Ok(value.clone()),
        None => Err(key.missing_err()),
    }
}

/// Wraps [String::from_utf8] inside a convinient
/// `Result<String, TorrentCreationError>` for simplified `.into()`/`?`
/// error processing
fn vecu8_to_string(input: Vec<u8>) -> Result<String, TorrentCreationError> {
    String::from_utf8(input.clone()).map_err(|_| TorrentCreationError::BadUTF8String(input))
}

/// Makes a new element for [TorrentFile::MultiFile] from given unparsed, raw
/// `file_raw` [Bencode::Dict]. It is not required to check the `file_raw`
/// [Bencode] type beforehand, this method will do for you
fn make_multifile(file_raw: Bencode) -> Result<(usize, Vec<String>), TorrentCreationError> {
    match file_raw {
        Bencode::Dict(file_dict) => {
            let length = get_dict_item(&file_dict, TorrentBencodeKey::Length)?
                .int()
                .ok_or(TorrentCreationError::LengthWrongType)? as usize;
            let path_raw_vec = get_dict_item(&file_dict, TorrentBencodeKey::Path)?
                .list()
                .ok_or(TorrentCreationError::PathWrongType)?;

            if path_raw_vec.len() == 0 {
                return Err(TorrentCreationError::NoPathFound);
            }

            let mut path = vec![];

            for subdir_raw in path_raw_vec {
                path.push(match subdir_raw {
                    Bencode::ByteString(found_subdir) => vecu8_to_string(found_subdir)?,
                    _ => return Err(TorrentCreationError::SubdirWrongType),
                })
            }

            Ok((length, path))
        }
        _ => return Err(TorrentCreationError::FileWrongType),
    }
}

impl Torrent {
    /// Creates a new [Torrent] from given `torrent_data` formatted as [Vec]<[u8]>
    ///
    /// If an error is encountered, it will be a
    /// [TorrentCreationError](TorrentCreationError) wrapped inside of
    /// [TorroError::TorrentCreationError](TorroError::TorrentCreationError)
    pub fn new(torrent_data: Vec<u8>) -> Result<Self, TorroError> {
        let parsed_bencode = bencode::parse(torrent_data)?;

        match parsed_bencode {
            Bencode::Dict(dict_data) => {
                // top-level dictionary
                let announce = vecu8_to_string(
                    get_dict_item(&dict_data, TorrentBencodeKey::Announce)?
                        .bytestring()
                        .ok_or(TorrentCreationError::AnnounceWrongType)?,
                )?;
                let info_dict = get_dict_item(&dict_data, TorrentBencodeKey::Info)?
                    .dict()
                    .ok_or(TorrentCreationError::InfoWrongType)?;

                // inside info_dict
                let piece_length = get_dict_item(&info_dict, TorrentBencodeKey::PieceLength)?
                    .int()
                    .ok_or(TorrentCreationError::PieceLengthWrongType)?
                    as usize;
                let pieces_raw = get_dict_item(&info_dict, TorrentBencodeKey::Pieces)?
                    .bytestring()
                    .ok_or(TorrentCreationError::PiecesWrongType)?;
                let name = vecu8_to_string(
                    get_dict_item(&info_dict, TorrentBencodeKey::Name)?
                        .bytestring()
                        .ok_or(TorrentCreationError::NameWrongType)?,
                )?;
                let length = match get_dict_item(&info_dict, TorrentBencodeKey::Length) {
                    Ok(length_raw) => Some(
                        length_raw
                            .int()
                            .ok_or(TorrentCreationError::LengthWrongType)?
                            as usize,
                    ),
                    Err(_) => None,
                };
                let files_raw: Option<Vec<Bencode>> =
                    match get_dict_item(&info_dict, TorrentBencodeKey::Files) {
                        Ok(files_bencode) => Some(
                            files_bencode
                                .list()
                                .ok_or(TorrentCreationError::FilesWrongType)?,
                        ),
                        Err(_) => None,
                    };

                let pieces: Vec<Vec<u8>> = pieces_raw
                    .as_slice()
                    .chunks(20)
                    .map(|c| c.to_vec())
                    .collect();

                let file_structure = if files_raw.is_some() {
                    if length.is_some() {
                        return Err(TorrentCreationError::BothLengthFiles.into());
                    }

                    let mut files = vec![];

                    for file_raw in files_raw.unwrap() {
                        files.push(make_multifile(file_raw)?);
                    }

                    TorrentFile::MultiFile(files)
                } else if length.is_some() {
                    TorrentFile::Single(length.unwrap() as usize)
                } else {
                    return Err(TorrentCreationError::NoLengthFiles.into());
                };

                Ok(Self {
                    announce,
                    name,
                    piece_length,
                    pieces,
                    file_structure,
                })
            }
            _ => Err(TorrentCreationError::NoTLDictionary.into()),
        }
    }

    /// Creates a new [Torrent] from given `.torrent` file path
    pub fn from_file(file: PathBuf) -> Result<Self, TorroError> {
        match read_file_bytes(&file) {
            Ok(bytes) => Ok(Torrent::new(bytes)?),
            Err(_) => Err(TorroError::BadFileRead(file)),
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
            Err(TorrentCreationError::NoTLDictionary.into())
        );
        assert_eq!(
            Torrent::new("ldee".as_bytes().to_vec()),
            Err(TorrentCreationError::NoTLDictionary.into())
        );
    }

    /// Tests that [TorrentBencodeKey::Announce] returns the wrong type
    /// correctly as an error
    #[test]
    fn announce_badtype() {
        assert_eq!(
            Torrent::new(
                "d8:announcei0e12:piece lengthi0e6:pieces0:e"
                    .as_bytes()
                    .to_vec()
            ),
            Err(TorrentCreationError::AnnounceWrongType.into())
        );
    }

    /// Tests that [TorrentBencodeKey::Name] returns the wrong type correctly as
    /// an error
    #[test]
    fn name_badtype() {
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod4:namei0e12:piece lengthi0e6:pieces0:6:lengthi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(TorrentCreationError::NameWrongType.into())
        )
    }

    /// Tests that [TorrentBencodeKey::Files] returns the wrong type correctly
    /// as an error
    #[test]
    fn files_badtype() {
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod4:name12:test_torrent12:piece lengthi0e6:pieces0:5:filesi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(TorrentCreationError::FilesWrongType.into())
        )
    }

    /// Tests that a file element inside of [TorrentBencodeKey::Files] returns
    /// the wrong type correctly as an error
    #[test]
    fn file_element_badtype() {
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod4:name12:test_torrent12:piece lengthi0e6:pieces0:5:filesli0eeee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(TorrentCreationError::FileWrongType.into())
        )
    }

    /// Tests that the `length` element of a file inside of [TorrentBencodeKey::Files]
    /// returns the wrong type correctly as an error
    #[test]
    fn length_file_element_badtype() {
        assert_eq!(
            Torrent::new("d8:announce0:4:infod4:name12:test_torrent12:piece lengthi0e6:pieces0:5:filesld6:length0:4:pathl0:eeeee".as_bytes().to_vec()),
            Err(TorrentCreationError::LengthWrongType.into())
        )
    }

    /// Tests that the `path` element of a file inside of [TorrentBencodeKey::Files]
    /// returns the wrong type correctly as an error
    #[test]
    fn path_file_element_badtype() {
        assert_eq!(
            Torrent::new("d8:announce0:4:infod4:name12:test_torrent12:piece lengthi0e6:pieces0:5:filesld6:lengthi0e4:pathi0eeeee".as_bytes().to_vec()),
            Err(TorrentCreationError::PathWrongType.into())
        )
    }

    /// Tests that [TorrentBencodeKey::Piece] returns the wrong type correctly
    /// as an error
    #[test]
    fn piece_badtype() {
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod12:piece length5:wrong6:pieces0:ee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(TorrentCreationError::PieceLengthWrongType.into())
        ); // `piece length`
    }

    /// Tests that [TorrentBencodeKey::Pieces] returns the wrong type correctly
    /// as an error
    #[test]
    fn pieces_badtype() {
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod12:piece lengthi0e6:piecesi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(TorrentCreationError::PiecesWrongType.into())
        );
    }

    /// Tests that [TorrentBencodeKey::Info] returns the wrong type correctly
    /// as an error
    #[test]
    fn info_badtype() {
        assert_eq!(
            Torrent::new("d8:announce0:4:infoi0ee".as_bytes().to_vec()),
            Err(TorrentCreationError::InfoWrongType.into())
        );
    }

    /// Tests that all [TorrentBencodeKey]'s are correctly reported missing when
    /// non-existant
    #[test]
    fn missing_torrent_types() {
        assert_eq!(
            Torrent::new("d8:announce0:e".as_bytes().to_vec()),
            Err(TorrentCreationError::NoInfoFound.into())
        );
        assert_eq!(
            Torrent::new(
                "d4:infod4:name12:test_torrent12:piece lengthi0e6:pieces0:6:lengthi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(TorrentCreationError::NoAnnounceFound.into())
        );
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod12:piece lengthi0e6:pieces0:6:lengthi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(TorrentCreationError::NoNameFound.into())
        );
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod4:name12:test_torrent6:pieces0:6:lengthi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(TorrentCreationError::NoPieceLengthFound.into())
        );
        assert_eq!(
            Torrent::new(
                "d8:announce0:4:infod4:name12:test_torrent12:piece lengthi0e6:lengthi0eee"
                    .as_bytes()
                    .to_vec()
            ),
            Err(TorrentCreationError::NoPiecesFound.into())
        );
    }
}
