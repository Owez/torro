//! Contains main [Torrent] structure used as a "key" to interact with other
//! parts of torro. Re-exported for convinience
//!
//! See [Torrent] and [TorrentFile] for more infomation

mod impl_bencode;
mod impl_download;

pub use impl_bencode::*;
pub use impl_download::*;

/// Represents the overall torrent directory structure for a given [Torrent]
///
/// This merges the [BEP0003](https://www.bittorrent.org/beps/bep_0003.html) spec
/// of either a single `length` for a file given or a list of dictionaries into
/// this singular enum for easier comprehension
///
/// ## Documentation sourcing
///
/// All "BitTorrent Description" headings are taken from
/// [BEP0003](https://www.bittorrent.org/beps/bep_0003.html) and is subject to
/// change, like any moving standard. This documentation is based off of version
/// `0e08ddf84d8d3bf101cdf897fc312f2774588c9e`
#[derive(Debug, PartialEq, Clone)]
pub enum TorrentFile {
    /// A single file with a [usize] determining it's length in bytes (`1` in
    /// usize == 1 byte)
    ///
    /// # BitTorrent Description
    ///
    /// ```none
    /// In the single file case, length maps to the length of the file in bytes.
    /// ```
    Single(usize),

    /// Multiple files with a similar [usize] to [TorrentFile::Single] as the first
    /// element and a [Vec] of [String] subdirectories
    ///
    /// # BitTorrent Description
    ///
    /// ```none
    /// length - The length of the file, in bytes.
    ///
    /// path - A list of UTF-8 encoded strings corresponding to subdirectory names,
    /// the last of which is the actual file name (a zero length list is an error
    /// case).
    /// ```
    MultiFile(Vec<(usize, Vec<String>)>),
}

/// The primary representation of a torrent, created from a parsing function
/// like [bencode::parse](crate::bencode::parse). This representation is used to
/// interact with many parts of torro.
///
/// # Examples
///
/// #### Loading a `.torrent` file
///
/// From a file path:
///
/// ```no_run
/// use std::path::PathBuf;
/// use torro::Torrent;
///
/// fn main() {
///     let file_path = PathBuf::from("example.torrent");
///     let my_torrent = Torrent::from_file(file_path).unwrap();
///
///     println!("my_torrent's values: {:#?}", my_torrent);
/// }
/// ```
///
/// From raw torrent bytes:
///
/// ```rust
/// use torro::Torrent;
///
/// fn main() {
///     let data = vec![
///         100, 56, 58, 97, 110, 110, 111, 117, 110, 99, 101, 48, 58, 52, 58, 105, 110, 102, 111, 100,
///         52, 58, 110, 97, 109, 101, 49, 50, 58, 116, 101, 115, 116, 95, 116, 111, 114, 114, 101,
///         110, 116, 49, 50, 58, 112, 105, 101, 99, 101, 32, 108, 101, 110, 103, 116, 104, 105, 48,
///         101, 54, 58, 112, 105, 101, 99, 101, 115, 48, 58, 54, 58, 108, 101, 110, 103, 116, 104,
///         105, 48, 101, 101, 101,
///     ];  // raw torrent bytes
///     let my_torrent = Torrent::new(data).unwrap();
///
///     println!("Torrent name: '{}'", my_torrent.name);
/// }
/// ```
///
/// # Documentation sourcing
///
/// All "BitTorrent Description" headings are taken from
/// [BEP0003](https://www.bittorrent.org/beps/bep_0003.html) and is subject to
/// change, like any moving standard. This documentation is based off of version
/// `0e08ddf84d8d3bf101cdf897fc312f2774588c9e`
#[derive(Debug, PartialEq, Clone)]
pub struct Torrent {
    /// URL for tracker
    ///
    /// # BitTorrent Description
    ///
    /// ```none
    /// The URL of the tracker.
    /// ```
    pub announce: String,

    /// Advised save name for torrent once leeched, is use by torro by default
    /// but may be changed
    ///
    /// # BitTorrent Description
    ///
    /// ```none
    /// The `name` key maps to a UTF-8 encoded string which is the suggested name
    /// to save the file (or directory) as. It is purely advisory.
    /// ```
    pub name: String, // TODO: allow changing once implemented

    /// File buffer (aka piece) length, commonly a power of 2 (e.g. `2`, `4`,
    /// `8`, `16`)
    ///
    /// # BitTorrent Description
    ///
    /// ```none
    /// `piece length` maps to the number of bytes in each piece the file is split
    /// into. For the purposes of transfer, files are split into fixed-size pieces
    /// which are all the same length except for possibly the last one which may
    /// be truncated. piece length is almost always a power of two, most commonly
    /// 2 18 = 256 K (BitTorrent prior to version 3.2 uses 2 20 = 1 M as default).
    /// ```
    pub piece_length: usize,

    /// A vector of binary-encoded SHA hashes corrosponding to each
    /// [Torrent::piece_length]
    ///
    /// # BitTorrent Description
    ///
    /// *Please note that torro represents this "(byte)string whose length is a
    /// multiple of 20" as a `Vec<Vec<u8>>` with each iteration of top-most vec
    /// containing a `Vec<u8>`-coded hash for simplicity*
    ///
    /// ```none
    /// `pieces` maps to a string whose length is a multiple of 20. It is to be
    /// subdivided into strings of length 20, each of which is the SHA1 hash of
    /// the piece at the corresponding index.
    /// ```
    pub pieces: Vec<Vec<u8>>,

    /// The overall file structure of the torrent, see the [TorrentFile] enum for
    /// more infomation
    ///
    /// # BitTorrent Description
    ///
    /// *We have merged the two options into a single enum for easier digesting
    /// inside of Rust*
    ///
    /// ```none
    /// There is also a key length or a key files, but not both or neither. If
    /// length is present then the download represents a single file, otherwise
    /// it represents a set of files which go in a directory structure.
    ///
    /// In the single file case, length maps to the length of the file in bytes.
    ///
    /// For the purposes of the other keys, the multi-file case is treated as
    /// only having a single file by concatenating the files in the order they
    /// appear in the files list. The files list is the value files maps to, and
    /// is a list of dictionaries containing the following keys:
    ///
    /// length - The length of the file, in bytes.
    ///
    /// path - A list of UTF-8 encoded strings corresponding to subdirectory names,
    /// the last of which is the actual file name (a zero length list is an error
    /// case).
    ///
    /// In the single file case, the name key is the name of a file, in the
    /// muliple file case, it's the name of a directory.
    /// ```
    pub file_structure: TorrentFile,
}
