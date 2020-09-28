//! All public error enums, see [TorroError] for the container enum which provides
//! access to more specific errors

use std::path::PathBuf;

/// Main error enum containing multiple module-specific errors like [BencodeError]
/// for `.torrent` (bencode) parsing
///
/// All module-specific errors have a [From] trait implemented by default for
/// this [TorroError] and are required to have at least the `Debug` derive added
#[non_exhaustive]
#[derive(Debug, PartialEq, Clone)]
pub enum TorroError {
    /// An error relating to the [crate::bencode] module
    BencodeError(BencodeError),

    /// An error relating to the creation of [Torrent](crate::Torrent)'s
    /// (from [Torrent::new](crate::Torrent::new) or
    /// [Torrent::from_file](crate::Torrent::from_file))
    TorrentCreationError(TorrentCreationError),

    /// An error relating to the [crate::tracker_udp] module (which is used inside
    /// of [Torrent::download](crate::Torrent::download))
    TrackerError(TrackerError),

    /// When an attemped file read failed, typically happens with
    /// [Torrent::from_file](crate::Torrent::from_file). See
    /// [TorroError::BadFileWrite] for errors related to file writes
    BadFileRead(PathBuf),

    /// A bad file write occured, typically happens when trying to safe a result
    /// of a download without corrent write permissions. See
    /// [TorroError::BadFileRead] for errors related to file reads
    BadFileWrite(PathBuf),

    /// Indicates that a call has reached an unimplemented section of the library,
    /// used for placeholder returns instead of the less graceful
    /// `unimplemented!()` macro
    Unimplemented,
}

/// Error enum for errors during parsing. If a [usize] is given, it typically
/// represents last parsed byte's posision
#[derive(Debug, PartialEq, Clone)]
pub enum BencodeError {
    /// When the file ends prematurely without stopping
    UnexpectedEOF,

    /// A character has been placed in an unexpected area, this occurs commonly
    /// with integers that have a misc character. The first item in tuple
    /// represents placement and second represents the unexpected byte
    UnexpectedByte((usize, u8)),

    /// An integer block was left empty, e.g. `ie`
    NoIntGiven(usize),

    /// Integer contains invalid (not 0-9) characters
    InvalidInt(usize),

    /// A `i-0e` was given (negative zero) which is not allowed by the spec
    NegativeZero(usize),

    /// Zeros where given before any significant number, e.g. `i002e`
    LeadingZeros(usize),

    /// No bencode data given
    EmptyFile,

    /// Bencode provided to bencode parser had multiple values given. Bencode is
    /// only allowed to have 1 toplevel value, if you'd like more, use a list or
    /// dict as the toplevel
    MultipleValues,
}

impl From<BencodeError> for TorroError {
    fn from(error: BencodeError) -> Self {
        TorroError::BencodeError(error)
    }
}

/// Error enum used inside of [Torrent::new](crate::Torrent::new) and
/// [Torrent::from_file](crate::Torrent::from_file). These errors relate
/// to the creation of new [Torrent](crate::Torrent) structures
#[derive(Debug, PartialEq, Clone)]
pub enum TorrentCreationError {
    /// BEP0003 dictates that the toplevel of a bencoded `.torrent` file should
    /// be a dictionary but it is not
    NoTLDictionary,

    /// A bytestring was designated to be a standard UTF-8 plaintext string but
    /// contained invalid bytes that [String::from_utf8] could not parse
    BadUTF8String(Vec<u8>),

    /// When the given `announce` key was given the wrong type. The
    /// `announce` key should be a bytestring (e.g.
    /// [Bencode::ByteString](crate::bencode::Bencode::ByteString))
    AnnounceWrongType,

    /// When the given `info` key was given the wrong type. The
    /// `info` key should be a dictionary (e.g.
    /// [Bencode::Dict](crate::bencode::Bencode::Dict))
    InfoWrongType,

    /// When the given `piece length` key was given the wrong type. The `piece length` key
    /// should be an integer (e.g. [Bencode::Int](crate::bencode::Bencode::Int))
    ///
    /// Not to be confused with [TorrentCreationError::PiecesWrongType]
    PieceLengthWrongType,

    /// When the given `pieces` key was given the wrong type. The `pieces` key
    /// should be a bytestring (e.g.
    /// [Bencode::ByteString](crate::bencode::Bencode::ByteString))
    ///
    /// Not to be confused with [TorrentCreationError::PieceLengthWrongType]
    PiecesWrongType,

    /// When the given `name` key was given the wrong type. The `name` key should
    /// be a bytestring (e.g. [Bencode::ByteString](crate::bencode::Bencode::ByteString))
    NameWrongType,

    /// When a/the given `length` key was given the wrong type. The `length` key
    /// should be an integer (e.g. [Bencode::Int](crate::bencode::Bencode::Int))
    LengthWrongType,

    /// When the given `announce` key was given the wrong type. The
    /// `announce` key should be a dictionary (e.g.
    /// [Bencode::Dict](crate::bencode::Bencode::Dict))
    ///
    /// Not to be confused with [TorrentCreationError::FileWrongType]
    FilesWrongType,

    /// When an element in the `files` list was not a bencoded dictionary (e.g.
    /// [Bencode::Dict](crate::bencode::Bencode::Dict))
    ///
    /// Not to be confused with [TorrentCreationError::FilesWrongType]
    FileWrongType,

    /// When the `path` key in an element of the `files` list was given the
    /// wrong type. The `path` key should be a list (e.g.
    /// [Bencode::List](crate::bencode::Bencode::List))
    PathWrongType,

    /// When a subdirectory inside of a `path` key in an element of the `files`
    /// list was given the wrong type. All subdirectory elements inside the `path`
    /// key should be bytestrings (e.g.
    /// [Bencode::ByteString](crate::bencode::Bencode::ByteString))
    SubdirWrongType,

    /// [Torrent](crate::Torrent) requires an `announce` key inside
    /// the top-level dictionary but it wasn't found
    NoAnnounceFound,

    /// [Torrent](crate::Torrent) requires an `info` key inside
    /// the top-level dictionary but it wasn't found
    NoInfoFound,

    /// [Torrent](crate::Torrent) requires a `piece length` key inside the
    /// `info` dictionary but it wasn't found
    ///
    /// Not to be confused with [TorrentCreationError::NoPiecesFound]
    NoPieceLengthFound,

    /// [Torrent](crate::Torrent) requires a `pieces` key inside the
    /// info dictionary but it wasn't found
    ///
    /// Not to be confused with [TorrentCreationError::NoPieceLengthFound]
    NoPiecesFound,

    /// [Torrent](crate::Torrent) requires a `name` key inside
    /// the info dictionary but it wasn't found
    NoNameFound,

    /// Neither the `length` or `files` keys where given in the top-level of the
    /// given bencode
    ///
    /// According to BEP0003, either `length` or `files` should be given, not
    /// neither or both
    NoLengthFiles,

    /// Both the `length` and `files` where passed when only one should be given
    ///
    /// According to BEP0003, either `length` or `files` should be given, not
    /// neither or both
    BothLengthFiles,

    /// No `path` was given for a file element in the `files` list or the
    /// (byte)string given was of length 0
    NoPathFound,
}

impl From<TorrentCreationError> for TorroError {
    fn from(error: TorrentCreationError) -> Self {
        TorroError::TorrentCreationError(error)
    }
}

/// Error enum used inside of [Torrent::download](crate::Torrent::download)
/// which extends from the [crate::tracker_udp] module (where it originates).
/// This type of error happens when torro could not properly connect to a tracker
/// to maintain infomation
#[derive(Debug, PartialEq, Clone)]
pub enum TrackerError {
    /// An error occured relating to creating a udp socket to connect to the
    /// tracker. The address used to try to connect is provided as a [String],
    /// typically the [crate::tracker_udp::TORRO_BIND_ADDR] constant
    BadSocketBind(&'static str),

    /// After sending a connection request to the tracker, torro occured an error
    /// when trying to recieve a response from the tracker
    BadConnectRecieve
}

impl From<TrackerError> for TorroError {
    fn from(error: TrackerError) -> Self {
        TorroError::TrackerError(error)
    }
}
