//! Contains bencode parsing-related functions used inside of
//! [Torrent::new](crate::torrent::Torrent::new) and
//! [Torrent::from_path](crate::torrent::Torrent::from_path)
//! 
//! Based on the [BEP0003](https://www.bittorrent.org/beps/bep_0003.html) bencode
//! parsing specifications

use crate::torrent::Torrent;

/// Control char num for detecting int starts, equates to `i`
const INT_START: u8 = 105;

/// Control char num for detecting list starts, equates to `l`
const LIST_START: u8 = 108;

/// Control char num for detecting dict starts, equates to `d`
const DICT_START: u8 = 100;

/// Control char num for detecting end of data structures, equates to `e`
const END: u8 = 101;

/// Control char num for seperating string length number from contents, equates to `:`
const STR_SEP: u8 = 58;

/// Error enum for errors during parsing
pub enum ParseError {

}

/// A found bencode object whilst parsing, only one is returned from [parse] due
/// to the bencode spec
pub enum Bencode {
    Dict(Vec<(u8, Bencode)>),
    List(Vec<Bencode>),
    
}

/// Parses provided `&[u8]` input into a [Bencode] that contains the entirety of
/// the parsed bencode file
/// 
/// Please see [Torrent](crate::torrent::Torrent) if you are searching for a
/// fully-complete torrent representation
fn parse(data: &[u8]) -> Result<Bencode, ParseError> {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::*;

    // tests here
}