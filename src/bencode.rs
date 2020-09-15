//! Bencode parsing-related functions used inside of
//! [Torrent::new](crate::torrent::Torrent::new) and
//! [Torrent::from_file](crate::torrent::Torrent::from_file)
//!
//! Based on the [BEP0003](https://www.bittorrent.org/beps/bep_0003.html) bencode
//! parsing specifications

use crate::error::BencodeError;
use std::collections::BTreeMap;
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
#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub enum Bencode {
    /// [Lexographically-ordered](https://en.wikipedia.org/wiki/Lexicographical_order)
    /// dictionary from `d3:keyi6e9:other key12:second valuee`.
    ///
    /// The first value is equivilant to a [Bencode::ByteString] and second is a
    /// recursive [Bencode] block
    Dict(BTreeMap<Vec<u8>, Bencode>),

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
    stop_byte: u8,
    bytes_iter: &mut Enumerate<impl Iterator<Item = u8>>,
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
fn decode_num(byte_ind: usize, bytes: Vec<u8>) -> Result<u32, BencodeError> {
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
    byte_ind: usize,
    bytes_iter: &mut Enumerate<impl Iterator<Item = u8>>,
) -> Result<i64, BencodeError> {
    let mut got_bytes = read_until(END, bytes_iter)?;

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

        Ok(-(decode_num(byte_ind, got_bytes)? as i64))
    } else {
        Ok(decode_num(byte_ind, got_bytes)? as i64)
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

/// Decodes a given bytestring into `Vec<u8>`. This requires that the `start_byte`,
/// a base-10 number byte that indicated the start of the bytestring, to be passed
/// due to the no-peek method of this [bytecode] parser
fn decode_bytestring(
    cur_byte: (usize, u8),
    bytes_iter: &mut Enumerate<impl Iterator<Item = u8>>,
) -> Result<Vec<u8>, BencodeError> {
    let mut len_utf8 = vec![cur_byte.1];
    len_utf8.append(&mut read_until(STR_SEP, bytes_iter)?);

    let string_len = decode_num(cur_byte.0, len_utf8)?;

    Ok(bytes_iter.take(string_len as usize).map(|x| x.1).collect())
}

/// Checks the lexographic order of many individual items against each other in
/// a dictionary. `byte_ind` required for any errors that may occur
fn check_dict_order(
    byte_ind: usize,
    to_check: &BTreeMap<Vec<u8>, Bencode>,
) -> Result<(), BencodeError> {
    let mut to_check_iter = to_check.iter().map(|(k, _)| k);

    let last_element = match to_check_iter.next() {
        Some(le) => le,
        None => return Ok(()), // zero-element iterator
    };

    for element in to_check_iter {
        if element < last_element {
            return Err(BencodeError::UnorderedDictionary((
                byte_ind,
                to_check.clone(),
            )));
        }
    }

    Ok(())
}

/// Decodes a dictionary (json-like object or equivilant to a `BTreeMap<Vec<u8>, Bencode>`)
fn decode_dict(
    bytes_iter: &mut Enumerate<impl Iterator<Item = u8>>,
) -> Result<BTreeMap<Vec<u8>, Bencode>, BencodeError> {
    let mut start_ind: Option<usize> = None;
    let mut btree_out = BTreeMap::new();

    let mut key_buf = None;
    let mut val_buf = None;

    loop {
        match bytes_iter.next() {
            Some(cur_byte) => {
                if start_ind == None {
                    start_ind = Some(cur_byte.0);
                }

                if key_buf != None && val_buf != None {
                    btree_out.insert(key_buf.take().unwrap(), val_buf.take().unwrap());
                }

                if cur_byte.1 == END {
                    break;
                }

                if key_buf == None {
                    key_buf = Some(decode_bytestring(cur_byte, bytes_iter)?);
                } else if val_buf == None {
                    val_buf = Some(get_next(Some(cur_byte), bytes_iter)?);
                }
            }
            None => return Err(BencodeError::UnexpectedEOF),
        }
    }

    check_dict_order(start_ind.unwrap(), &btree_out)?;

    Ok(btree_out)
}

/// Finds the next full [Bencode] block or returns a [BencodeError::UnexpectedEOF]
fn get_next(
    cur_byte: Option<(usize, u8)>,
    bytes_iter: &mut Enumerate<impl Iterator<Item = u8>>,
) -> Result<Bencode, BencodeError> {
    match cur_byte {
        Some((byte_ind, byte)) => match byte {
            INT_START => Ok(Bencode::Int(decode_int(byte_ind, bytes_iter)?)),
            LIST_START => Ok(Bencode::List(decode_list(bytes_iter)?)),
            DICT_START => Ok(Bencode::Dict(decode_dict(bytes_iter)?)),
            48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => Ok(Bencode::ByteString(
                decode_bytestring(cur_byte.unwrap(), bytes_iter)?,
            )),
            _ => Err(BencodeError::UnexpectedByte(cur_byte.unwrap())),
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

    /// Tests [parse] makes a proper [Bencode::Int] and handles any errors that
    /// may occur (from [decode_int])
    #[test]
    fn integers() {
        assert_eq!(parse("i50e".as_bytes().to_vec()), Ok(Bencode::Int(50)));
        assert_eq!(parse("i0e".as_bytes().to_vec()), Ok(Bencode::Int(0)));
        assert_eq!(
            parse("i1000000e".as_bytes().to_vec()),
            Ok(Bencode::Int(1000000))
        );
        assert_eq!(
            parse("i-1000000e".as_bytes().to_vec()),
            Ok(Bencode::Int(-1000000))
        );

        assert_eq!(
            parse("ie".as_bytes().to_vec()),
            Err(BencodeError::NoIntGiven(0))
        );
        assert_eq!(
            parse("i00e".as_bytes().to_vec()),
            Err(BencodeError::LeadingZeros(0))
        );
        assert_eq!(
            parse("i-0e".as_bytes().to_vec()),
            Err(BencodeError::NegativeZero(0))
        );
        assert_eq!(
            parse("i-00e".as_bytes().to_vec()),
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
            "12:helloi64eee12:i30000e",
            "udp://tracker.torrent.eu.org:451",
            "",
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
        assert_eq!(parse("le".as_bytes().to_vec()), Ok(Bencode::List(vec![])));
        assert_eq!(
            parse("li64ee".as_bytes().to_vec()),
            Ok(Bencode::List(vec![Bencode::Int(64)]))
        );
        assert_eq!(
            parse("li-200ei0ee".as_bytes().to_vec()),
            Ok(Bencode::List(vec![Bencode::Int(-200), Bencode::Int(0)]))
        );
        assert_eq!(
            parse("l6:stringi0ei0ee".as_bytes().to_vec()),
            Ok(Bencode::List(vec![
                Bencode::ByteString("string".as_bytes().to_vec()),
                Bencode::Int(0),
                Bencode::Int(0)
            ]))
        );
    }

    /// Tests that [read_until] correctly stops at end marks rather then going over
    #[test]
    fn correct_end_mark() {
        assert_eq!(
            parse("i64ee".as_bytes().to_vec()),
            Err(BencodeError::MultipleValues)
        );
        assert_eq!(
            parse("lee".as_bytes().to_vec()),
            Err(BencodeError::MultipleValues)
        );
    }

    /// Tests that dict parsing (from [decode_dict]) works correctly with
    /// well-formatted values
    #[test]
    fn dicts() {
        let mut btree_test = BTreeMap::new();

        assert_eq!(
            parse("de".as_bytes().to_vec()),
            Ok(Bencode::Dict(btree_test.clone()))
        );

        btree_test.insert("int".as_bytes().to_vec(), Bencode::Int(64));
        assert_eq!(
            parse("d3:inti64ee".as_bytes().to_vec()),
            Ok(Bencode::Dict(btree_test))
        );

        btree_test = BTreeMap::new();

        btree_test.insert(
            "str".as_bytes().to_vec(),
            Bencode::ByteString("ok".as_bytes().to_vec()),
        );
        assert_eq!(
            parse("d3:str2:oke".as_bytes().to_vec()),
            Ok(Bencode::Dict(btree_test))
        );

        btree_test = BTreeMap::new();

        btree_test.insert(
            "first".as_bytes().to_vec(),
            Bencode::ByteString("value".as_bytes().to_vec()),
        );
        btree_test.insert(
            "list".as_bytes().to_vec(),
            Bencode::List(vec![
                Bencode::Int(-1000),
                Bencode::ByteString("lastelement".as_bytes().to_vec()),
            ]),
        );
        assert_eq!(
            parse(
                "d5:first5:value4:listli-1000e11:lastelementee"
                    .as_bytes()
                    .to_vec()
            ),
            Ok(Bencode::Dict(btree_test))
        );

        btree_test = BTreeMap::new();

        btree_test.insert(
            "announce".as_bytes().to_vec(),
            Bencode::ByteString("udp://tracker.torrent.eu.org:451".as_bytes().to_vec()),
        );
        assert_eq!(
            parse(
                "d8:announce32:udp://tracker.torrent.eu.org:451e"
                    .as_bytes()
                    .to_vec()
            ),
            Ok(Bencode::Dict(btree_test))
        );
    }

    /// Tests that parsed dicts (from [decode_dict]) properly error when given
    /// invalid data
    #[test]
    fn badf_dicts() {
        assert_eq!(
            parse("d".as_bytes().to_vec()),
            Err(BencodeError::UnexpectedEOF)
        );
        assert_eq!(
            parse("dd".as_bytes().to_vec()),
            Err(BencodeError::UnexpectedEOF)
        );
        assert_eq!(
            parse("dddddddddddddddi64eeeeeeeeeeeeeee".as_bytes().to_vec()),
            Err(BencodeError::UnexpectedEOF)
        ); // 15 starts, 14 ends
    }
}
