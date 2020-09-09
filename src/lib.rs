//! An easy-to-use BitTorrent library
//!
//! ## Status
//!
//! Heavily work-in-progress with no BitTorrent features currently fully supported. You may contribute [here](https://github.com/owez/torro) if you'd like.
//!
//! ## Specific modules
//!
//! You may also choose to just use some modules inside of Torro for a specific
//! purpose, here is a summary of available ones:
//!
//! | Module name             | About                                                                        |
//! |-------------------------|------------------------------------------------------------------------------|
//! | [parser](crate::parser) | `.torrent` (bencode) parser core, used to interpret downloaded torrent files |

pub mod parser;
mod utils;

/// [BitTorrent prefix](https://wiki.theory.org/BitTorrentSpecification#peer_id)
/// for all torro-based clients.
///
/// **If this library is forked and used heavily in a production enviroment, it
/// is advised to change this**
pub const CLIENT_PREFIX: &str = "TO";

/// The primary representation of a torrent, created from the [parse](crate::parser::parse)
/// function. This representation is used to interact with many parts of torro.
///
/// *All "BitTorrent Description" headings are taken from
/// [BEP0003](https://www.bittorrent.org/beps/bep_0003.html) and is subject to
/// change, like any moving standard. This documentation is based off of version
/// `0e08ddf84d8d3bf101cdf897fc312f2774588c9e`*
pub struct Torrent {
    /// URL for tracker
    ///
    /// # BitTorrent Description
    ///
    /// ```none
    /// The URL of the tracker.
    /// ```
    pub announce_url: String,

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
    /// `piece` length maps to the number of bytes in each piece the file is split
    /// into. For the purposes of transfer, files are split into fixed-size pieces
    /// which are all the same length except for possibly the last one which may
    /// be truncated. piece length is almost always a power of two, most commonly
    /// 2 18 = 256 K (BitTorrent prior to version 3.2 uses 2 20 = 1 M as default).
    /// ```
    pub piece: usize,

    /// A vector of SHA hashes corrosponding to each [Torrent::piece]
    ///
    /// # BitTorrent Description
    ///
    /// *Please note that torro represents this "string whose length is a multiple
    /// of 20" as a [Vec]<[String]> with each string containing a hash for simplicity*
    ///
    /// ```none
    /// `pieces` maps to a string whose length is a multiple of 20. It is to be
    /// subdivided into strings of length 20, each of which is the SHA1 hash of
    /// the piece at the corresponding index.
    /// ```
    pub pieces: Vec<String>,
    
    // TODO: Finish adding values from https://www.bittorrent.org/beps/bep_0003.html
}
