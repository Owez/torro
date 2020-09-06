//! Contains `.torrent` (bencode) parsing-related functions. See [parse] and it's
//! returned [BencodeLine] vector for more infomation regarding torrent parsing.

/// Errors relating to parsing with [parse]/[parse_str]
#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    /// When an expected token on line was not found
    TokenNotFound,

    /// When the file ends prematurely without stopping (more specific
    /// [ParseError::TokenNotFound])
    UnexpectedEOF,

    /// A character has been placed in an unexpected area, this occurs commonly with
    /// integers that have a misc character
    UnexpectedChar(char),

    /// An integer block was left empty, e.g. `ie`
    NoIntGiven,

    /// A `i-0e` was given (negative zero) which is not allowed by the spec
    NegativeZero,

    /// Zeros where given before any significant number, e.g. `i002e`
    LeadingZeros,

    /// Whitespace was given in an incorrect position, e.g. in middle of integer
    BadWhitespace,

    /// A negative integer was given for a unsigned integer. Note that negatives
    /// are only typically allowed in an explicit `i0e` snum block
    UnsignedNegativeInt,
}

/// Parsed `.torrent` (bencode) file line, containing a variety of outcomes
pub enum BencodeLine {
    /// Similar to a HashMap
    Dict(Vec<(String, Box<BencodeLine>)>),
    /// Array of lower-level [BencodeLine] instances
    List(Vec<Box<BencodeLine>>),
    /// Number (can be either num or snum, both fit into [i64])
    Num(i64),
    /// String
    Str(String),
}

/// Internal types for tokens in scanner
#[derive(Debug, PartialEq, Clone)]
enum TokenType {
    /// 'i' start char
    NumStart,
    /// 'l' start char
    ListStart,
    /// 'd' start char
    DictStart,
    /// 'e' end char
    End,
    /// ' '/space char, included as spec is strict on whitespace
    Whitespace,
    /// ':' seperator char
    StringSep,
    /// Misc char used for data
    Char(char),
}

impl From<TokenType> for char {
    fn from(token: TokenType) -> Self {
        match token {
            TokenType::NumStart => 'i',
            TokenType::ListStart => 'l',
            TokenType::DictStart => 'd',
            TokenType::End => 'e',
            TokenType::Whitespace => ' ',
            TokenType::StringSep => ':',
            TokenType::Char(c) => c,
        }
    }
}

impl From<char> for TokenType {
    fn from(character: char) -> Self {
        match character {
            'i' => TokenType::NumStart,
            'l' => TokenType::ListStart,
            'd' => TokenType::DictStart,
            'e' => TokenType::End,
            ' ' => TokenType::Whitespace,
            ':' => TokenType::StringSep,
            c => TokenType::Char(c),
        }
    }
}

/// Lexes data and returns an output of [Vec]<[Vec]<[TokenType]>> corrosponding
/// to each
///
/// First vec contains many lines, each inner vec contains a long array of
/// [TokenType]
fn scan_data(data: &str) -> Vec<Vec<TokenType>> {
    let mut output_vec: Vec<Vec<TokenType>> = vec![vec![]];

    for character in data.chars() {
        if character == '\n' {
            output_vec.push(vec![]);
            continue;
        }

        output_vec.last_mut().unwrap().push(character.into())
    }

    output_vec
}

/// Iterates over line_iter and adds to output vec until query is found then
/// returns (without adding last found token)
fn read_until(
    query: TokenType,
    line_iter: &mut std::iter::Peekable<std::slice::Iter<TokenType>>,
) -> Result<Vec<TokenType>, ParseError> {
    let mut token_output = vec![];

    loop {
        let token = match line_iter.next() {
            Some(t) => t,
            None => return Err(ParseError::TokenNotFound),
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
        TokenType::Whitespace => return Err(ParseError::BadWhitespace),
        _ => Err(ParseError::UnexpectedChar(token.into())),
    }
}

/// Digests a [Vec] of [TokenType] into a basic number. See [decode_int] for
/// signed, blocked `i-1e` version
fn decode_num(tokens: Vec<TokenType>) -> Result<u32, ParseError> {
    if tokens.len() == 0 {
        return Err(ParseError::NoIntGiven);
    } else if tokens.first().unwrap() == &TokenType::Char('-') {
        return Err(ParseError::UnsignedNegativeInt);
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
    line_iter: &mut std::iter::Peekable<std::slice::Iter<TokenType>>,
) -> Result<i64, ParseError> {
    line_iter.next(); // skip `i` prefix

    let mut tokens = read_until(TokenType::End, line_iter)?;
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

/// Parses `.torrent` (bencode) file into a [BencodeLine] for each line
pub fn parse(data: &str) -> Result<Vec<BencodeLine>, ParseError> {
    let mut output_vec = vec![];
    let scanned_data = scan_data(data);

    for line in scanned_data.iter() {
        // line-level

        let mut line_iter = line.iter().peekable();
        // let mut char_ind: usize = 0;

        loop {
            let next_token = match line_iter.peek() {
                Some(nt) => nt,
                None => return Err(ParseError::UnexpectedEOF),
            };

            match next_token {
                TokenType::NumStart => {
                    output_vec.push(BencodeLine::Num(decode_int(&mut line_iter)?));
                }
                _ => unimplemented!("This kind of token coming soon!"),
            }
        }
    }

    Ok(output_vec)
}

/// Alias for [parse] which allows a [String] `data` rather than a &[str] `data`
pub fn parse_str(data: String) -> Result<Vec<BencodeLine>, ParseError> {
    parse(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Basic assert_eq tests for [scan_data] with simple single-line data
    #[test]
    fn scan_basic_eq() {
        assert_eq!(scan_data(""), vec![vec![]]);
        assert_eq!(
            scan_data("i32e"),
            vec![vec![
                TokenType::NumStart,
                TokenType::Char('3'),
                TokenType::Char('2'),
                TokenType::End
            ]]
        );
        assert_eq!(
            scan_data("ilde:_"),
            vec![vec![
                TokenType::NumStart,
                TokenType::ListStart,
                TokenType::DictStart,
                TokenType::End,
                TokenType::StringSep,
                TokenType::Char('_')
            ]]
        );
    }

    /// Basic assert_ne tests for [scan_data] with simple single-line data
    #[test]
    fn scan_basic_ne() {
        assert_ne!(scan_data("l"), vec![vec![TokenType::Char('l')]]);
    }

    /// Trips for newlines and whitespace
    #[test]
    fn scan_newlines_whitespace() {
        assert_eq!(
            scan_data("   \n \n      i       "),
            vec![
                vec![
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                ],
                vec![TokenType::Whitespace],
                vec![
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::NumStart,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                ]
            ]
        );
        assert_eq!(scan_data("\n"), vec![vec![], vec![]]);
    }

    /// Checks the basic [decode_num] works correctly and in turn
    /// [digitstr_from_token] also works correctly
    #[test]
    fn vec_nums() {
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
            Err(ParseError::UnsignedNegativeInt)
        );
    }

    /// Checks positive int digests
    #[test]
    fn int_digest() {
        assert_eq!(
            decode_int(&mut scan_data("i1e")[0].iter().peekable()),
            Ok(1)
        );
        assert_eq!(
            decode_int(&mut scan_data("i324e")[0].iter().peekable()),
            Ok(324)
        );
        assert_eq!(
            decode_int(&mut scan_data("i10000e")[0].iter().peekable()),
            Ok(10000)
        );
        assert_eq!(
            decode_int(&mut scan_data("i00234000e")[0].iter().peekable()),
            Err(ParseError::LeadingZeros)
        );
        assert_eq!(
            decode_int(&mut scan_data("i0e")[0].iter().peekable()),
            Ok(0)
        );
        assert_eq!(
            decode_int(&mut scan_data("i000000e")[0].iter().peekable()),
            Err(ParseError::LeadingZeros)
        );
        assert_eq!(
            decode_int(&mut scan_data("i4 0 9 6e")[0].iter().peekable()),
            Err(ParseError::BadWhitespace)
        );
        assert_eq!(
            decode_int(&mut scan_data("i10 0  0e")[0].iter().peekable()),
            Err(ParseError::BadWhitespace)
        );
        assert_eq!(
            decode_int(&mut scan_data("ie")[0].iter().peekable()),
            Err(ParseError::NoIntGiven)
        );
    }

    /// Checks negative int digests
    #[test]
    fn neg_int_digest() {
        assert_eq!(
            decode_int(&mut scan_data("i-1e")[0].iter().peekable()),
            Ok(-1)
        );
        assert_eq!(
            decode_int(&mut scan_data("i-324e")[0].iter().peekable()),
            Ok(-324)
        );
        assert_eq!(
            decode_int(&mut scan_data("i-10000e")[0].iter().peekable()),
            Ok(-10000)
        );
        assert_eq!(
            decode_int(&mut scan_data("i-0e")[0].iter().peekable()),
            Err(ParseError::NegativeZero)
        );
        assert_eq!(
            decode_int(&mut scan_data("i-000000e")[0].iter().peekable()),
            Err(ParseError::LeadingZeros)
        );
        assert_eq!(
            decode_int(&mut scan_data("i-00 3 2e")[0].iter().peekable()),
            Err(ParseError::BadWhitespace)
        );
        assert_eq!(
            decode_int(&mut scan_data("i-34-22-234e")[0].iter().peekable()),
            Err(ParseError::UnexpectedChar('-'))
        );
        assert_eq!(
            decode_int(&mut scan_data("i-e")[0].iter().peekable()),
            Err(ParseError::NoIntGiven)
        );
    }
}
