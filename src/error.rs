//! All public error enums, see [TorroError] for the container enum which provides
//! access to more specific errors

/// Main error enum containing multiple module-specific errors like [BencodeError]
/// for `.torrent` (bencode) parsing
///
/// All module-specific errors have a [From] trait implemented by default for
/// this [TorroError] and are required to have at least the `Debug`, `PartialEq`
/// and `Clone` derives added
#[derive(Debug, PartialEq, Clone)]
pub enum TorroError {
    /// An error relating to the [crate::bencode] module
    BencodeError(BencodeError),

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
