//! Contains `.torrent` parsing-related functions. See [parse] and it's returned
//! [TorrentLine] vector for more infomation regarding torrent parsing.

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
}

/// Parsed torrent file line, containing a variety of outcomes
pub enum TorrentLine {
    /// Similar to a HashMap
    Dict(Vec<(String, Box<TorrentLine>)>),
    /// Array of lower-level [TorrentLine] instances
    List(Vec<Box<TorrentLine>>),
    /// Integer
    Int(i32),
    /// String
    Str(String),
}

/// Internal types for tokens in scanner
#[derive(Debug, PartialEq, Clone)]
enum TokenType {
    /// 'i' start char
    Int,
    /// 'l' start char
    List,
    /// 'd' start char
    Dict,
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
            TokenType::Int => 'i',
            TokenType::List => 'l',
            TokenType::Dict => 'd',
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
            'i' => TokenType::Int,
            'l' => TokenType::List,
            'd' => TokenType::Dict,
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

/// Decodes found integer block and consumes last [TokenType::End], ensure
/// starting [TokenType::Int] control char is consumed before running
fn decode_int(
    line_iter: &mut std::iter::Peekable<std::slice::Iter<TokenType>>,
) -> Result<i32, ParseError> {
    let tokens = read_until(TokenType::End, line_iter)?;

    let mut int_buf = String::with_capacity(tokens.len());
    let mut neg_number = false;
    let mut leading_zero_check = false;

    for token in tokens {
        if int_buf.len() == 0 {
            if token == TokenType::Char('-') {
                neg_number = true;
                continue;
            } else if token == TokenType::Char('0') {
                leading_zero_check = true;
            }
        }

        match token {
            TokenType::Char(c) => match c {
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => int_buf.push(c),
                uc => return Err(ParseError::UnexpectedChar(uc)),
            },
            TokenType::Whitespace => return Err(ParseError::BadWhitespace),
            _ => return Err(ParseError::UnexpectedChar(token.into())),
        }
    }

    if int_buf.len() == 0 {
        return Err(ParseError::NoIntGiven);
    } else if leading_zero_check && int_buf.len() > 1 {
        return Err(ParseError::LeadingZeros);
    }

    let parsed_int = int_buf.parse::<i32>().unwrap();

    if parsed_int == 0 && neg_number {
        Err(ParseError::NegativeZero)
    } else if neg_number {
        Ok(-parsed_int)
    } else {
        Ok(parsed_int)
    }
}

/// Parses `.torrent` file into a [TorrentLine] for each line
pub fn parse(data: &str) -> Result<Vec<TorrentLine>, ParseError> {
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
                TokenType::Int => {
                    line_iter.next();
                    output_vec.push(TorrentLine::Int(decode_int(&mut line_iter)?));
                }
                _ => unimplemented!("This kind of token coming soon!"),
            }
        }
    }

    Ok(output_vec)
}

/// Alias for [parse] which allows a [String] `data` rather than a &[str] `data`
pub fn parse_str(data: String) -> Result<Vec<TorrentLine>, ParseError> {
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
                TokenType::Int,
                TokenType::Char('3'),
                TokenType::Char('2'),
                TokenType::End
            ]]
        );
        assert_eq!(
            scan_data("ilde:_"),
            vec![vec![
                TokenType::Int,
                TokenType::List,
                TokenType::Dict,
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
                vec![TokenType::Whitespace,],
                vec![
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Whitespace,
                    TokenType::Int,
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

    /// Checks positive integer digests
    ///
    /// Note that [decode_int] doesn't consume preceding `i` so it is not included
    /// in inputted strings
    #[test]
    fn integer_digest() {
        assert_eq!(decode_int(&mut scan_data("1e")[0].iter().peekable()), Ok(1));
        assert_eq!(
            decode_int(&mut scan_data("324e")[0].iter().peekable()),
            Ok(324)
        );
        assert_eq!(
            decode_int(&mut scan_data("10000e")[0].iter().peekable()),
            Ok(10000)
        );
        assert_eq!(
            decode_int(&mut scan_data("00234000e")[0].iter().peekable()),
            Err(ParseError::LeadingZeros)
        );
        assert_eq!(decode_int(&mut scan_data("0e")[0].iter().peekable()), Ok(0));
        assert_eq!(
            decode_int(&mut scan_data("000000e")[0].iter().peekable()),
            Err(ParseError::LeadingZeros)
        );
        assert_eq!(
            decode_int(&mut scan_data("4 0 9 6e")[0].iter().peekable()),
            Err(ParseError::BadWhitespace)
        );
        assert_eq!(
            decode_int(&mut scan_data("10 0  0e")[0].iter().peekable()),
            Err(ParseError::BadWhitespace)
        );
        assert_eq!(
            decode_int(&mut scan_data("e")[0].iter().peekable()),
            Err(ParseError::NoIntGiven)
        );
    }

    /// Checks negative integer digests
    ///
    /// Note that [decode_int] doesn't consume preceding `i` so it is not included
    /// in inputted strings
    #[test]
    fn neg_integer_digest() {
        assert_eq!(
            decode_int(&mut scan_data("-1e")[0].iter().peekable()),
            Ok(-1)
        );
        assert_eq!(
            decode_int(&mut scan_data("-324e")[0].iter().peekable()),
            Ok(-324)
        );
        assert_eq!(
            decode_int(&mut scan_data("-10000e")[0].iter().peekable()),
            Ok(-10000)
        );
        assert_eq!(
            decode_int(&mut scan_data("-0e")[0].iter().peekable()),
            Err(ParseError::NegativeZero)
        );
        assert_eq!(
            decode_int(&mut scan_data("-000000e")[0].iter().peekable()),
            Err(ParseError::LeadingZeros)
        );
        assert_eq!(
            decode_int(&mut scan_data("-00 3 2e")[0].iter().peekable()),
            Err(ParseError::BadWhitespace)
        );
        assert_eq!(
            decode_int(&mut scan_data("-34-22-234e")[0].iter().peekable()),
            Err(ParseError::UnexpectedChar('-'))
        );
        assert_eq!(
            decode_int(&mut scan_data("-e")[0].iter().peekable()),
            Err(ParseError::NoIntGiven)
        );
    }
}
