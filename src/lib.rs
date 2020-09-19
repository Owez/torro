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
//! Actively developed but heavily work-in-progress with only `.torrent` to
//! Torrent user-friendly struct currently fully supported, see the torro
//! [roadmap](https://github.com/Owez/torro/issues/20) for future plans.
//!
//! ## Final notes
//!
//! - If you wish to use torro without using the [Torrent](crate::torrent::Torrent)
//! structure, you may use the publically exposed lower-level functions that are
//! not attached to it (like [bencode::parse] for example)

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
