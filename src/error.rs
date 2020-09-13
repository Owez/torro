//! All public error enums, see [TorroError] for the container enum which provides
//! access to more specific errors

use std::path::PathBuf;

/// Main error enum containing multiple module-specific errors like [BencodeError]
/// for `.torrent` (bencode) parsing
///
/// All module-specific errors have a [From] trait implemented by default for
/// this [TorroError] and are required to have at least the `Debug` derive added
#[derive(Debug, PartialEq, Clone)]
pub enum TorroError {
    /// An error relating to the [crate::bencode] module
    BencodeError(BencodeError),

    /// An error relating to the creation of [Torrent](crate::torrent::Torrent)'s
    /// (from [Torrent::new](crate::torrent::Torrent::new) or
    /// [Torrent::from_file](crate::torrent::Torrent::from_file))
    TorrentCreationError(TorrentCreationError),

    /// When an attemped file read failed, typically happens with
    /// [Torrent::from_file](crate::torrent::Torrent::from_file). See
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

/// Error enum used inside of [Torrent::new](crate::torrent::Torrent::new) and
/// [Torrent::from_file](crate::torrent::Torrent::from_file). These errors relate
/// to the creation of new [Torrent](crate::torrent::Torrent) structures
#[derive(Debug, PartialEq, Clone)]
pub enum TorrentCreationError {
    /// BEP0003 dictates that the toplevel of a bencoded `.torrent` file should
    /// be a dictionary but it is not
    NoTLDictionary,
}

impl From<TorrentCreationError> for TorroError {
    fn from(error: TorrentCreationError) -> Self {
        TorroError::TorrentCreationError(error)
    }
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
