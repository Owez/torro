//! A **correct** and **easy-to-use** BitTorrent library
//!
//! ## Objectives
//!
//! - Easy-to-use library interface that assumes by default with customisability
//! if needed
//! - Extremely low dependency count (none ideally)
//! - High amount of documentation, no data structures/functions implemented
//! without a line of docstring
//! - Correctness with the BitTorrent protocols
//!
//! ## Development/Production Status
//!
//! Heavily work-in-progress with no BitTorrent features currently fully supported.
//! You may contribute [here](https://github.com/owez/torro) if you'd like.
//!
//! ## First steps
//!
//! 1. You should first create a torrent structure by inputting a `.torrent` file
//! formatted as `&[u8]` into [Torrent::new](crate::torrent::Torrent::new) or by
//! passing a `.torrent` file to [Torrent::from_path](crate::torrent::Torrent::from_path)
//! 2. Once you have a [Torrent](crate::torrent::Torrent) struct, you have access
//! to other parts of torro like **`COMING SOON`** or **`COMING SOON`**.
//!
//! ## Final notes
//!
//! - If you wish to use torro without using the [torrent::Torrent] structure, you
//! may use the publically exposed lower-level functions that are not attached to
//! it (like [bencode::parse] for example)

mod utils;

pub mod bencode;
pub mod error;
pub mod torrent;

/// [BitTorrent prefix](https://wiki.theory.org/BitTorrentSpecification#peer_id)
/// for all torro-based clients.
///
/// **If this library is forked and used heavily in a production enviroment, it
/// is advised to change this**
pub const CLIENT_PREFIX: &str = "TO";
