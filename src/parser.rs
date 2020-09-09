//! Contains `.torrent` (bencode) parsing-related functions. See [parse] and it's
//! returned [BencodeObj] vector for more infomation regarding torrent parsing.

/// Control char for detecting int starts
const INT_START: char = 'i';

/// Control char for detecting list starts
const LIST_START: char = 'l';

/// Control char for detecting dict starts
const DICT_START: char = 'd';

/// Control char for detecting end of data structures
const END: char = 'e';

/// Control char for seperating string length number from contents
const STR_SEP: char = ':';

/// Errors relating to parsing with [parse]/[parse_str]
#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    /// When the file ends prematurely without stopping (more specific
    /// [ParseError::UnexpectedToken])
    UnexpectedEOF,

    /// A character has been placed in an unexpected area, this occurs commonly with
    /// integers that have a misc character
    UnexpectedChar(char),

    /// An unexpected token was found, this is a more general version of
    /// [ParseError::UnexpectedChar] and [ParseError::UnexpectedEOF]
    UnexpectedToken,

    /// An integer block was left empty, e.g. `ie`
    NoIntGiven,

    /// A `i-0e` was given (negative zero) which is not allowed by the spec
    NegativeZero,

    /// Zeros where given before any significant number, e.g. `i002e`
    LeadingZeros,
}

/// Parsed `.torrent` (bencode) file line, containing a variety of outcomes
#[derive(Debug, PartialEq, Clone)]
pub enum BencodeObj {
    /// Similar to a HashMap
    Dict(Vec<(String, Box<BencodeObj>)>),
    /// Array of lower-level [BencodeObj] instances
    List(Vec<BencodeObj>),
    /// Number (can be either num or snum, both fit into [i64])
    Int(i64),
    /// Byte string
    ByteString(Vec<u8>),
}

/// Internal types for tokens in scanner
#[derive(Debug, PartialEq, Clone)]
enum TokenType {
    /// 'i' start char
    IntStart,
    /// 'l' start char
    ListStart,
    /// 'd' start char
    DictStart,
    /// 'e' end char
    End,
    /// ':' seperator char
    StringSep,
    /// Misc char used for data
    Char(char),
}

impl From<TokenType> for char {
    fn from(token: TokenType) -> Self {
        match token {
            TokenType::IntStart => INT_START,
            TokenType::ListStart => LIST_START,
            TokenType::DictStart => DICT_START,
            TokenType::End => END,
            TokenType::StringSep => STR_SEP,
            TokenType::Char(c) => c,
        }
    }
}

impl From<char> for TokenType {
    fn from(character: char) -> Self {
        match character {
            INT_START => TokenType::IntStart,
            LIST_START => TokenType::ListStart,
            DICT_START => TokenType::DictStart,
            END => TokenType::End,
            STR_SEP => TokenType::StringSep,
            c => TokenType::Char(c),
        }
    }
}

/// Lexes data and returns an output of [Vec]<[TokenType]> corrosponding
/// to each
fn scan_data(data: &str) -> Vec<TokenType> {
    data.chars().map(|c| c.into()).collect()
}

/// Matches next peeked token in `token_iter` and consumes it until a relevant
/// data structure has been made
fn match_next_bencodeobj(
    peeked_token: &TokenType,
    token_iter: &mut std::iter::Peekable<std::slice::Iter<TokenType>>,
) -> Result<BencodeObj, ParseError> {
    match peeked_token {
        TokenType::IntStart => Ok(BencodeObj::Int(decode_int(token_iter)?)),
        TokenType::ListStart => Ok(BencodeObj::List(decode_list(token_iter)?)),
        TokenType::Char(c) => match c {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                Ok(BencodeObj::ByteString(decode_bytestring(token_iter)?))
            }
            _ => return Err(ParseError::UnexpectedChar(*c)), // TODO better error
        },
        _ => unimplemented!(),
    }
}

/// Similar to [read_until] but collects while [BencodeObj]s and then checks
/// token after to see if it matches. Used throughout for lists and recusive
/// sections
fn match_until(
    query: TokenType,
    token_iter: &mut std::iter::Peekable<std::slice::Iter<TokenType>>,
) -> Result<Vec<BencodeObj>, ParseError> {
    let mut output_vec = vec![];
    let mut peeked_token = match token_iter.peek() {
        Some(p) => p,
        None => return Err(ParseError::UnexpectedEOF),
    };

    while peeked_token != &&query {
        output_vec.push(match_next_bencodeobj(peeked_token, token_iter)?);
        peeked_token = match token_iter.peek() {
            Some(p) => p,
            None => return Err(ParseError::UnexpectedEOF),
        };
    }

    token_iter.next(); // consume queried token

    Ok(output_vec)
}

/// Iterates over token_iter and adds to output vec until query is found then
/// returns (without adding last found token)
fn read_until(
    query: TokenType,
    token_iter: &mut std::iter::Peekable<std::slice::Iter<TokenType>>,
) -> Result<Vec<TokenType>, ParseError> {
    let mut token_output = vec![];

    loop {
        let token = match token_iter.next() {
            Some(t) => t,
            None => return Err(ParseError::UnexpectedEOF),
        };

        if token == &query {
            break;
        } else {
            token_output.push(token.clone())
        }
    }

    Ok(token_output)
}

/// Matches a digit char to ensure it isn't incorrectly formatted
fn digitstr_from_token(token: TokenType) -> Result<char, ParseError> {
    match token {
        TokenType::Char(c) => match c {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => Ok(c),
            uc => Err(ParseError::UnexpectedChar(uc)),
        },
        _ => Err(ParseError::UnexpectedToken),
    }
}

/// Digests a [Vec] of [TokenType] into a basic number. See [decode_int] for
/// signed, blocked `i-1e` version
fn decode_num(tokens: Vec<TokenType>) -> Result<u32, ParseError> {
    if tokens.len() == 0 {
        return Err(ParseError::NoIntGiven);
    }

    let numstring = tokens
        .iter()
        .map(|t| digitstr_from_token(t.clone()))
        .collect::<Result<String, _>>()?;

    if numstring.len() > 1 && numstring.chars().nth(0).unwrap() == '0' {
        return Err(ParseError::LeadingZeros);
    }

    Ok(numstring.parse().unwrap())
}

/// Decodes full int block which is an snum with `i` prefix and `e` char
fn decode_int(
    token_iter: &mut std::iter::Peekable<std::slice::Iter<TokenType>>,
) -> Result<i64, ParseError> {
    token_iter.next(); // skip `i` prefix

    let mut tokens = read_until(TokenType::End, token_iter)?;
    let mut neg_number = false;

    if tokens.first() == Some(&TokenType::Char('-')) {
        neg_number = true;
        tokens.remove(0);
    }

    let parsed_num = decode_num(tokens)?;

    if parsed_num == 0 && neg_number {
        Err(ParseError::NegativeZero)
    } else if neg_number {
        Ok(-(parsed_num as i64))
    } else {
        Ok(parsed_num as i64)
    }
}

/// Decodes string using unsigned/basic [decode_num] and counts chars until it
/// is satisfied or [ParseError::UnexpectedEOF]
fn decode_bytestring(
    token_iter: &mut std::iter::Peekable<std::slice::Iter<TokenType>>,
) -> Result<Vec<u8>, ParseError> {
    let prefix_num = decode_num(read_until(TokenType::StringSep, token_iter)?)?;
    let mut output_bytevec: Vec<u8> = Vec::with_capacity(token_iter.len());

    for _ in 0..prefix_num {
        output_bytevec.push(match token_iter.next() {
            Some(c) => char::from(c.clone()) as u8,
            None => return Err(ParseError::UnexpectedEOF),
        });
    }

    Ok(output_bytevec)
}

/// Decodes list by recursively decending through other bencode objects
fn decode_list(
    token_iter: &mut std::iter::Peekable<std::slice::Iter<TokenType>>,
) -> Result<Vec<BencodeObj>, ParseError> {
    token_iter.next(); // skip `l` prefix
    match_until(TokenType::End, token_iter)
}

/// Parses `.torrent` (bencode) file into a [BencodeObj] for each line
pub fn parse(data: &str) -> Result<Vec<BencodeObj>, ParseError> {
    let mut output_vec = vec![];
    let scanned_data = scan_data(data);

    let mut token_iter = scanned_data.iter().peekable();

    loop {
        let peeked_token = match token_iter.peek() {
            Some(nt) => nt,
            None => break,
        };

        output_vec.push(match_next_bencodeobj(peeked_token, &mut token_iter)?);
    }

    Ok(output_vec)
}

/// Alias for [parse] which allows a [String] `data` rather than a &[str] `data`
pub fn parse_str(data: String) -> Result<Vec<BencodeObj>, ParseError> {
    parse(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Basic assert_eq tests for [scan_data] with simple single-line data
    #[test]
    fn scan_basic_eq() {
        assert_eq!(scan_data(""), vec![]);
        assert_eq!(
            scan_data("i32e"),
            vec![
                TokenType::IntStart,
                TokenType::Char('3'),
                TokenType::Char('2'),
                TokenType::End
            ]
        );
        assert_eq!(
            scan_data("ilde:_"),
            vec![
                TokenType::IntStart,
                TokenType::ListStart,
                TokenType::DictStart,
                TokenType::End,
                TokenType::StringSep,
                TokenType::Char('_')
            ]
        );
    }

    /// Basic assert_ne tests for [scan_data] with simple single-line data
    #[test]
    fn scan_basic_ne() {
        assert_ne!(scan_data("l"), vec![TokenType::Char('l')]);
    }

    /// Trips for newlines and whitespace
    #[test]
    fn scan_newlines_whitespace() {
        assert_eq!(
            scan_data("   \n \n      i       "),
            vec![
                TokenType::Char(' '),
                TokenType::Char(' '),
                TokenType::Char(' '),
                TokenType::Char('\n'),
                TokenType::Char(' '),
                TokenType::Char('\n'),
                TokenType::Char(' '),
                TokenType::Char(' '),
                TokenType::Char(' '),
                TokenType::Char(' '),
                TokenType::Char(' '),
                TokenType::Char(' '),
                TokenType::IntStart,
                TokenType::Char(' '),
                TokenType::Char(' '),
                TokenType::Char(' '),
                TokenType::Char(' '),
                TokenType::Char(' '),
                TokenType::Char(' '),
                TokenType::Char(' '),
            ]
        );
        assert_eq!(scan_data("\n"), vec![TokenType::Char('\n')]);
    }

    /// Checks the basic [decode_num] works correctly and in turn
    /// [digitstr_from_token] also works correctly
    #[test]
    fn num_digest() {
        assert_eq!(
            decode_num(vec![
                TokenType::Char('3'),
                TokenType::Char('4'),
                TokenType::Char('2')
            ]),
            Ok(342)
        );
        assert_eq!(decode_num(vec![TokenType::Char('0')]), Ok(0));
        assert_eq!(
            decode_num(vec![
                TokenType::Char('1'),
                TokenType::Char('0'),
                TokenType::Char('0'),
                TokenType::Char('0'),
                TokenType::Char('0'),
                TokenType::Char('0'),
                TokenType::Char('0'),
                TokenType::Char('0')
            ]),
            Ok(10000000)
        );
        assert_eq!(
            decode_num(vec![TokenType::Char('0'), TokenType::Char('0')]),
            Err(ParseError::LeadingZeros)
        );
        assert_eq!(
            decode_num(vec![
                TokenType::Char('-'),
                TokenType::Char('3'),
                TokenType::Char('4'),
                TokenType::Char('2')
            ]),
            Err(ParseError::UnexpectedChar('-'))
        );
    }

    /// Checks positive int digests
    #[test]
    fn int_digest() {
        assert_eq!(decode_int(&mut scan_data("i1e").iter().peekable()), Ok(1));
        assert_eq!(
            decode_int(&mut scan_data("i324e").iter().peekable()),
            Ok(324)
        );
        assert_eq!(
            decode_int(&mut scan_data("i10000e").iter().peekable()),
            Ok(10000)
        );
        assert_eq!(
            decode_int(&mut scan_data("i00234000e").iter().peekable()),
            Err(ParseError::LeadingZeros)
        );
        assert_eq!(decode_int(&mut scan_data("i0e").iter().peekable()), Ok(0));
        assert_eq!(
            decode_int(&mut scan_data("i000000e").iter().peekable()),
            Err(ParseError::LeadingZeros)
        );
        assert_eq!(
            decode_int(&mut scan_data("i4 0 9 6e").iter().peekable()),
            Err(ParseError::UnexpectedChar(' '))
        );
        assert_eq!(
            decode_int(&mut scan_data("i10 0  0e").iter().peekable()),
            Err(ParseError::UnexpectedChar(' '))
        );
        assert_eq!(
            decode_int(&mut scan_data("ie").iter().peekable()),
            Err(ParseError::NoIntGiven)
        );
    }

    /// Checks negative int digests
    #[test]
    fn neg_int_digest() {
        assert_eq!(decode_int(&mut scan_data("i-1e").iter().peekable()), Ok(-1));
        assert_eq!(
            decode_int(&mut scan_data("i-324e").iter().peekable()),
            Ok(-324)
        );
        assert_eq!(
            decode_int(&mut scan_data("i-10000e").iter().peekable()),
            Ok(-10000)
        );
        assert_eq!(
            decode_int(&mut scan_data("i-0e").iter().peekable()),
            Err(ParseError::NegativeZero)
        );
        assert_eq!(
            decode_int(&mut scan_data("i--10e").iter().peekable()),
            Err(ParseError::UnexpectedChar('-'))
        );
        assert_eq!(
            decode_int(&mut scan_data("i-000000e").iter().peekable()),
            Err(ParseError::LeadingZeros)
        );
        assert_eq!(
            decode_int(&mut scan_data("i-00 3 2e").iter().peekable()),
            Err(ParseError::UnexpectedChar(' '))
        );
        assert_eq!(
            decode_int(&mut scan_data("i-34-22-234e").iter().peekable()),
            Err(ParseError::UnexpectedChar('-'))
        );
        assert_eq!(
            decode_int(&mut scan_data("i-e").iter().peekable()),
            Err(ParseError::NoIntGiven)
        );
    }

    /// Checks that [decode_bytestring] (bytestring decoding) is working correctly
    #[test]
    fn str_parsing() {
        assert_eq!(
            decode_bytestring(&mut scan_data("4:test").iter().peekable()),
            Ok("test".into())
        );
        assert_eq!(
            decode_bytestring(&mut scan_data("0:").iter().peekable()),
            Ok(vec![])
        );
        assert_eq!(
            decode_bytestring(&mut scan_data("1:f").iter().peekable()),
            Ok("f".into())
        );
        assert_eq!(
            decode_bytestring(&mut scan_data("7:try4:toerror").iter().peekable()),
            Ok("try4:to".into())
        );
    }

    /// Ensures that [parse] properly matches to the desired type
    #[test]
    fn parse_correctness() {
        assert_eq!(parse("i32e"), Ok(vec![BencodeObj::Int(32)]));
        assert_eq!(
            parse("4:test8:working?i1e"),
            Ok(vec![
                BencodeObj::ByteString("test".into()),
                BencodeObj::ByteString("working?".into()),
                BencodeObj::Int(1)
            ])
        );
    }

    /// Check [match_until] works correctly
    #[test]
    fn check_match_until() {
        assert_eq!(
            match_until(
                TokenType::End,
                &mut scan_data("i32ei4546ee5:norun").iter().peekable()
            ),
            Ok(vec![BencodeObj::Int(32), BencodeObj::Int(4546)]) // only ints and dupe `e` consumed
        );
    }

    /// Checks that lists work correctly
    #[test]
    fn list_parsing() {
        assert_eq!(
            decode_list(&mut scan_data("li60e4:test4:cooli0ee").iter().peekable()),
            Ok(vec![
                BencodeObj::Int(60),
                BencodeObj::ByteString("test".into()),
                BencodeObj::ByteString("cool".into()),
                BencodeObj::Int(0)
            ])
        )
    }
}
