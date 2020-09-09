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
