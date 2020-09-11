//! Bencode parsing-related functions used inside of
//! [Torrent::new](crate::torrent::Torrent::new) and
//! [Torrent::from_path](crate::torrent::Torrent::from_path)
//!
//! Based on the [BEP0003](https://www.bittorrent.org/beps/bep_0003.html) bencode
//! parsing specifications

use crate::error::BencodeError;
use std::iter::Enumerate;

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
/// [Err]([BencodeError::UnexpectedEOF]) is given). Does not return last element
/// which is equivalent to `stop_byte`
fn read_until(
    bytes_iter: &mut Enumerate<impl Iterator<Item = u8>>,
    stop_byte: u8,
) -> Result<Vec<u8>, BencodeError> {
    let mut new_bytes: Vec<u8> = vec![];

    loop {
        match bytes_iter.next() {
            Some((_, new_byte)) => {
                if new_byte == stop_byte {
                    break;
                } else {
                    new_bytes.push(new_byte)
                }
            }
            None => return Err(BencodeError::UnexpectedEOF),
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
fn decode_num(bytes: Vec<u8>, byte_ind: usize) -> Result<u32, BencodeError> {
    if bytes.len() == 0 {
        return Err(BencodeError::NoIntGiven(byte_ind));
    } else if bytes[0] == 48 && bytes.len() > 1 {
        return Err(BencodeError::LeadingZeros(byte_ind));
    }

    match std::str::from_utf8(&bytes) {
        Ok(numstr) => match numstr.parse::<u32>() {
            Ok(num) => Ok(num),
            Err(_) => Err(BencodeError::InvalidInt(byte_ind)),
        },
        Err(_) => Err(BencodeError::InvalidInt(byte_ind)),
    }
}

/// Decodes a full signed integer value using [decode_num] and adding minuses
fn decode_int(
    bytes_iter: &mut Enumerate<impl Iterator<Item = u8>>,
    byte_ind: usize,
) -> Result<i64, BencodeError> {
    let mut got_bytes = read_until(bytes_iter, END)?;

    let mut is_negative = false;

    if got_bytes.len() == 0 {
        // this is in decode_num but need to safeguard here too
        return Err(BencodeError::NoIntGiven(byte_ind));
    } else if got_bytes[0] == 45 {
        if got_bytes.len() == 1 {
            return Err(BencodeError::NoIntGiven(byte_ind));
        }

        got_bytes.remove(0);
        is_negative = true;
    }

    if is_negative {
        if got_bytes[0] == 48 {
            return Err(BencodeError::NegativeZero(byte_ind));
        }

        Ok(-(decode_num(got_bytes, byte_ind)? as i64))
    } else {
        Ok(decode_num(got_bytes, byte_ind)? as i64)
    }
}

/// Decodes a dynamically-typed vector (list) from bencode
fn decode_list(
    bytes_iter: &mut Enumerate<impl Iterator<Item = u8>>,
) -> Result<Vec<Bencode>, BencodeError> {
    let mut bencode_out = vec![];

    loop {
        match bytes_iter.next() {
            Some(cur_byte) => {
                if cur_byte.1 == END {
                    break;
                }

                bencode_out.push(get_next(Some(cur_byte), bytes_iter)?);
            }
            None => return Err(BencodeError::UnexpectedEOF),
        };
    }

    Ok(bencode_out)
}

/// Finds the next full [Bencode] block or returns a [BencodeError::UnexpectedEOF]
fn get_next(
    cur_byte: Option<(usize, u8)>,
    bytes_iter: &mut Enumerate<impl Iterator<Item = u8>>,
) -> Result<Bencode, BencodeError> {
    match cur_byte {
        Some((byte_ind, byte)) => match byte {
            INT_START => Ok(Bencode::Int(decode_int(bytes_iter, byte_ind)?)),
            LIST_START => Ok(Bencode::List(decode_list(bytes_iter)?)),
            48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
                // bytestring

                let mut num_utf8 = read_until(bytes_iter, STR_SEP)?; // utf-8 encoded number
                num_utf8.push(byte);

                let str_length = decode_num(num_utf8, byte_ind)? as usize;
                let u8string = bytes_iter
                    .take(str_length)
                    .map(|x| x.1)
                    .collect::<Vec<u8>>();

                Ok(Bencode::ByteString(u8string))
            }
            _ => Err(BencodeError::UnexpectedByte((byte_ind, byte))),
        },
        None => Err(BencodeError::UnexpectedEOF),
    }
}

/// Parses provided `Vec<u8>` input into a [Bencode] that contains the entirety of
/// the parsed bencode file
///
/// Please see [Torrent](crate::torrent::Torrent) if you are searching for a
/// fully-complete torrent representation
pub fn parse(data: Vec<u8>) -> Result<Bencode, BencodeError> {
    if data.len() == 0 {
        return Err(BencodeError::EmptyFile);
    }

    let mut bytes_iter = data.into_iter().enumerate();

    match get_next(bytes_iter.next(), &mut bytes_iter) {
        Ok(bencode_out) => {
            if bytes_iter.count() != 0 {
                Err(BencodeError::MultipleValues)
            } else {
                Ok(bencode_out)
            }
        }
        Err(e) => Err(e),
    }
}

/// Alias to [parse] which allows a [u8] [slice](std::slice), e.g. &[[u8]]
pub fn parse_slice(data: &[u8]) -> Result<Bencode, BencodeError> {
    parse(data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Turns a &[str] into a [Vec]<[u8]> for code nicity
    fn str_to_vecu8(i: &str) -> Vec<u8> {
        i.as_bytes().to_vec()
    }

    /// Tests [parse] makes a proper [Bencode::Int] and handles any errors that
    /// may occur (from [decode_int])
    #[test]
    fn integers() {
        assert_eq!(parse(str_to_vecu8("i50e")), Ok(Bencode::Int(50)));
        assert_eq!(parse(str_to_vecu8("i0e")), Ok(Bencode::Int(0)));
        assert_eq!(parse(str_to_vecu8("i1000000e")), Ok(Bencode::Int(1000000)));
        assert_eq!(
            parse(str_to_vecu8("i-1000000e")),
            Ok(Bencode::Int(-1000000))
        );

        assert_eq!(parse(str_to_vecu8("ie")), Err(BencodeError::NoIntGiven(0)));
        assert_eq!(
            parse(str_to_vecu8("i00e")),
            Err(BencodeError::LeadingZeros(0))
        );
        assert_eq!(
            parse(str_to_vecu8("i-0e")),
            Err(BencodeError::NegativeZero(0))
        );
        assert_eq!(
            parse(str_to_vecu8("i-00e")),
            Err(BencodeError::NegativeZero(0))
        );
    }

    /// Tests [parse] makes a proper [Bencode::ByteString] (from [decode_bytestring])
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

    /// Tests [parse] makes a well-formed list (from [decode_list])
    #[test]
    fn lists() {
        assert_eq!(parse(str_to_vecu8("le")), Ok(Bencode::List(vec![])));
        assert_eq!(
            parse(str_to_vecu8("li64ee")),
            Ok(Bencode::List(vec![Bencode::Int(64)]))
        );
        assert_eq!(
            parse(str_to_vecu8("li-200ei0ee")),
            Ok(Bencode::List(vec![Bencode::Int(-200), Bencode::Int(0)]))
        );
        assert_eq!(
            parse(str_to_vecu8("l6:stringi0ei0ee")),
            Ok(Bencode::List(vec![
                Bencode::ByteString(str_to_vecu8("string")),
                Bencode::Int(0),
                Bencode::Int(0)
            ]))
        );
    }

    /// Tests that [read_until] correctly stops at end marks rather then going over
    #[test]
    fn correct_end_mark() {
        assert_eq!(
            parse(str_to_vecu8("i64ee")),
            Err(BencodeError::MultipleValues)
        );
        assert_eq!(
            parse(str_to_vecu8("lee")),
            Err(BencodeError::MultipleValues)
        );
    }
}
