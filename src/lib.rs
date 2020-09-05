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
//! | [Parser](crate::parser) | `.torrent` (bencode) parser core, used to interpret downloaded torrent files |

pub mod parser;
