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
//! ## First Steps
//!
//! 1. You should first create a [torrent::Torrent] structure by using a parsing
//! function like [bencode::parse] by inputting a `.torrent` file as a plain
//! string.
//! 2. Once you have a [torrent::Torrent], you have access to other parts of torro
//! like **`COMING SOON`** or **`COMING SOON`**.

mod utils;

pub mod torrent;
pub mod bencode;

/// [BitTorrent prefix](https://wiki.theory.org/BitTorrentSpecification#peer_id)
/// for all torro-based clients.
///
/// **If this library is forked and used heavily in a production enviroment, it
/// is advised to change this**
pub const CLIENT_PREFIX: &str = "TO";
