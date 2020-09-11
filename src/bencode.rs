//! Contains bencode parsing-related functions used inside of
//! [Torrent::new](crate::torrent::Torrent::new) and
//! [Torrent::from_path](crate::torrent::Torrent::from_path)
//!
//! Based on the [BEP0003](https://www.bittorrent.org/beps/bep_0003.html) bencode
//! parsing specifications

use std::iter::Enumerate;
use std::slice::Iter;

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

/// Error enum for errors during parsing. If a [usize] is given, it typically
/// represents last parsed byte's posision
#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    /// When the file ends prematurely without stopping
    UnexpectedEOF,

    /// A character has been placed in an unexpected area, this occurs commonly with
    /// integers that have a misc character. The first item in tuple represents
    /// placement and second represents the unexpected byte
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

    /// Bencode provided to [parse] had multiple values given. Bencode is only
    /// allowed to have 1 toplevel value, if you'd like more, use a list or dict
    /// as the toplevel
    MultipleValues,
}

/// A found bencode object whilst parsing, only one is returned from [parse] due
/// to the bencode spec
#[derive(Debug, PartialEq, Clone)]
pub enum Bencode {
    /// [Lexographically-ordered](https://en.wikipedia.org/wiki/Lexicographical_order)
    /// dictionary from `d3:keyi6e9:other key12:second valuee`
    Dict(Vec<(u8, Bencode)>),

    /// Variable-sized array (list) of further [Bencode]s from `l4:this2:is1:a4:liste`
    List(Vec<Bencode>),

    /// A bytestring containing multiple bytes from `11:string here`
    ByteString(Vec<u8>),

    /// Parsed integer from a direct `i0e`
    Int(i64),
}

/// Steps over `bytes` until `stop_byte` is met or EOF (in which case
/// [Err]([ParseError::UnexpectedEOF]) is given). Does not return last element
/// which is equivilant to `stop_byte`
fn read_until(bytes_iter: &mut Enumerate<Iter<u8>>, stop_byte: u8) -> Result<Vec<u8>, ParseError> {
    let mut new_bytes: Vec<u8> = vec![];

    loop {
        match bytes_iter.next() {
            Some((_, new_byte)) => {
                if new_byte == &stop_byte {
                    break;
                } else {
                    new_bytes.push(*new_byte)
                }
            }
            None => return Err(ParseError::UnexpectedEOF),
        }
    }

    Ok(new_bytes)
}

/// Decodes simple, unsigned number from given Vec<u8> UTF-8
///
/// This requires a `byte_ind` incase a number of errors occur it needs to be
/// referenced back
///
/// If you want to decode a whole `i3432e` block, see [decode_int] instead
fn decode_num(bytes: Vec<u8>, byte_ind: usize) -> Result<u32, ParseError> {
    match std::str::from_utf8(&bytes) {
        Ok(numstr) => match numstr.parse::<u32>() {
            Ok(num) => Ok(num),
            Err(_) => Err(ParseError::InvalidInt(byte_ind)),
        },
        Err(_) => Err(ParseError::InvalidInt(byte_ind)),
    }
}

fn get_next(bytes_iter: &mut Enumerate<Iter<u8>>) -> Result<Bencode, ParseError> {
    match bytes_iter.next() {
        Some((byte_ind, byte)) => match byte {
            48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
                // bytestring

                let mut num_utf8 = read_until(bytes_iter, STR_SEP)?; // utf-8 encoded number
                num_utf8.push(*byte);

                let str_length = decode_num(num_utf8, byte_ind)? as usize;
                let u8string = bytes_iter
                    .take(str_length)
                    .map(|x| *x.1)
                    .collect::<Vec<u8>>();

                Ok(Bencode::ByteString(u8string))
            }
            _ => unimplemented!(),
        },
        None => Err(ParseError::UnexpectedEOF),
    }
}

/// Parses provided `Vec<u8>` input into a [Bencode] that contains the entirety of
/// the parsed bencode file
///
/// Please see [Torrent](crate::torrent::Torrent) if you are searching for a
/// fully-complete torrent representation
pub fn parse(data: Vec<u8>) -> Result<Bencode, ParseError> {
    let mut bytes_iter = data.iter().enumerate();

    match get_next(&mut bytes_iter) {
        Ok(bencode_out) => {
            if bytes_iter.count() != 0 {
                Err(ParseError::MultipleValues)
            } else {
                Ok(bencode_out)
            }
        }
        Err(ParseError::UnexpectedEOF) => Err(ParseError::EmptyFile), // map to empty file error
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Turns a &[str] into a [Vec]<[u8]> for code nicity
    fn str_to_vecu8(i: &str) -> Vec<u8> {
        i.as_bytes().to_vec()
    }

    /// Tests [parse] makes a proper [Bencode::Int] and handles any errors that
    /// may occur
    #[test]
    fn integers() {
        assert_eq!(parse(str_to_vecu8("i50e")), Ok(Bencode::Int(50)));
        assert_eq!(parse(str_to_vecu8("i0e")), Ok(Bencode::Int(0)));
        assert_eq!(parse(str_to_vecu8("i1000000e")), Ok(Bencode::Int(1000000)));
        assert_eq!(
            parse(str_to_vecu8("i-1000000e")),
            Ok(Bencode::Int(-1000000))
        );

        assert_eq!(parse(str_to_vecu8("ie")), Err(ParseError::NoIntGiven(0)));
        assert_eq!(
            parse(str_to_vecu8("i00e")),
            Err(ParseError::LeadingZeros(0))
        );
        assert_eq!(
            parse(str_to_vecu8("i-0e")),
            Err(ParseError::NegativeZero(0))
        );
        assert_eq!(
            parse(str_to_vecu8("i-00e")),
            Err(ParseError::LeadingZeros(0))
        );
    }

    /// Tests [parse] makes a proper [Bencode::ByteString]
    #[test]
    fn bytestring() {
        let inputs = vec![
            "hello there",
            "another_string",
            "e",
            "",
            "00",
            "i00e",
            "0x\\1",
        ];

        for input in inputs {
            let formatted_input = format!("{}:{}", input.len(), input).as_bytes().to_vec();

            assert_eq!(
                parse(formatted_input),
                Ok(Bencode::ByteString(input.as_bytes().to_vec()))
            );
        }
    }

    // TODO: more tests
}
