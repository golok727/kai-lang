use crate::ast::span::Span;

use super::error::{LexerError, LexerErrorKind};
use super::token::Token;

// start  token  end
pub type SpannedToken = (usize, Token, usize);

impl From<SpannedToken> for Span {
    fn from(value: SpannedToken) -> Self {
        Span {
            start: value.0,
            end: value.2,
        }
    }
}

pub struct Lexer<T: Iterator<Item = char>> {
    chars: T,
    ch0: Option<char>,
    ch1: Option<char>,
    start: usize,
    end: usize,
    token_queue: Vec<SpannedToken>,
}

pub type LexerResult = Result<SpannedToken, LexerError>;

impl<T> Lexer<T>
where
    T: Iterator<Item = char>,
{
    pub fn new(items: T) -> Self {
        let mut lexer = Lexer {
            chars: items,
            ch0: None,
            ch1: None,
            start: 0,
            end: 0,
            token_queue: Vec::new(),
        };

        lexer.next_char();
        lexer.next_char();

        lexer
    }

    fn advance(&mut self) -> LexerResult {
        while self.token_queue.is_empty() {
            self.consume()?;
        }

        Ok(self.token_queue.remove(0))
    }

    fn consume(&mut self) -> Result<(), LexerError> {
        if let Some(c) = self.ch0 {
            // println!("char({})", c);
            if self.is_name_start(c) {
                let name = self.eat_name()?;
                self.queue(name)
            } else if self.is_number_start(c) {
                let number = self.eat_number()?;
                self.queue(number);
            } else {
                self.eat_single_character()?;
            }
        } else {
            let pos = self.cursor();
            self.queue((pos, Token::Eof, pos));
        }

        Ok(())
    }

    #[inline]
    fn eat_single_token(&mut self, tk: Token) -> Token {
        self.next_char();
        tk
    }

    fn eat_single_character(&mut self) -> Result<(), LexerError> {
        if let Some(c) = self.ch0 {
            let start = self.cursor();

            let token: Option<Token> = match c {
                '\r' => {
                    self.next_char();
                    if self.ch0 == Some('\n') {
                        self.next_char(); // consume '\n' after '\r'
                    }
                    Some(Token::NewLine) // return NewLine token for CR or CRLF
                }
                '\n' => {
                    Some(self.eat_single_token(Token::NewLine)) // return NewLine token for LF
                }
                ' ' | '\t' | '\x0C' => {
                    self.next_char(); // eat whitespace
                    None
                }
                '(' => Some(self.eat_single_token(Token::LParen)),
                ')' => Some(self.eat_single_token(Token::RParen)),

                '[' => Some(self.eat_single_token(Token::LBracket)),
                ']' => Some(self.eat_single_token(Token::RBracket)),

                '{' => Some(self.eat_single_token(Token::LCurly)),
                '}' => Some(self.eat_single_token(Token::RCurly)),

                '+' | '*' | '/' | '!' | '=' => {
                    let tok = match c {
                        '+' => (Token::Plus, Token::PlusEq),
                        '*' => (Token::Star, Token::MulEq),
                        '/' => (Token::Slash, Token::DivEq),
                        '!' => (Token::Bang, Token::NotEq),
                        '=' => (Token::Eq, Token::EqEq),
                        _ => unreachable!(),
                    };

                    if self.ch1 == Some('=') {
                        self.next_char();
                        Some(self.eat_single_token(tok.1))
                    } else {
                        Some(self.eat_single_token(tok.0))
                    }
                }
                '-' => match self.ch1 {
                    Some('=') => {
                        self.next_char();
                        self.next_char();
                        Some(Token::MinusEq)
                    }
                    Some('>') => {
                        self.next_char();
                        self.next_char();
                        Some(Token::ArrowRight)
                    }
                    _ => Some(self.eat_single_token(Token::Minus)),
                },

                '.' => {
                    self.next_char();

                    let mut n_dots = 1;

                    if self.ch0 == Some('.') {
                        n_dots += 1;
                        self.next_char();
                    }

                    if self.ch0 == Some('.') {
                        n_dots += 1;
                        self.next_char();
                    }

                    let token = match n_dots {
                        1 => Token::Dot,
                        2 => Token::DotDot,
                        3 => Token::DotDotDot,
                        _ => unreachable!(),
                    };

                    Some(token)
                }

                ';' => Some(self.eat_single_token(Token::SemiColon)),
                ':' => Some(self.eat_single_token(Token::Colon)),
                '"' => {
                    let spanned = self.eat_double_quoted_string()?;
                    self.queue(spanned);
                    None
                }
                _ => {
                    self.next_char();
                    Some(Token::Unknown)
                }
            };

            // TODO error unknown token
            if let Some(token) = token {
                let end = self.cursor();
                self.queue((start, token, end));
            }
        };

        Ok(())
    }

    // name
    fn eat_name(&mut self) -> LexerResult {
        let mut name = String::new();

        let name_start = self.cursor();

        while self.is_name_continuation() {
            name.push(self.next_char().expect("parse_name: expected a char"))
        }

        let name_end = self.cursor();

        let token = match get_keyword_from_str(&name) {
            Some(token) => token,
            None => {
                if name.starts_with('_') {
                    Token::DiscardName { name: name.into() }
                } else {
                    Token::Name { name: name.into() }
                }
            }
        };

        Ok((name_start, token, name_end))
    }

    fn is_name_start(&self, c: char) -> bool {
        matches!(c, '_' | 'a'..='z')
    }

    fn is_name_continuation(&self) -> bool {
        self.ch0
            .map(|c| matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
            .unwrap_or(false)
    }

    fn eat_double_quoted_string(&mut self) -> LexerResult {
        let start = self.cursor();
        debug_assert_eq!(self.next_char().expect("expected a double quote"), '"');

        let mut string = String::new();

        loop {
            match self.next_char() {
                Some('\\') => {
                    // Handle escape sequences
                    if let Some(escape_char) = self.next_char() {
                        match escape_char {
                            // Recognized escape sequences
                            'n' => string.push('\n'),
                            'r' => string.push('\r'),
                            't' => string.push('\t'),
                            'f' => string.push('\x0C'), // Form feed (U+000C)
                            '\\' => string.push('\\'),
                            '"' => string.push('"'),
                            _ => {
                                string.push('\\');
                                string.push(escape_char);
                            }
                        }
                    }
                }
                Some('"') => break,        // End of the string literal
                Some(c) => string.push(c), // Regular characters
                None => {
                    // Non-terminated string literal
                    return Err(LexerError {
                        kind: LexerErrorKind::NonTerminatedStringLiteral,
                        location: Span {
                            start,
                            end: self.cursor(),
                        },
                    });
                }
            }
        }

        Ok((
            start,
            Token::String {
                value: string.into(),
            },
            self.cursor(),
        ))
    }

    // parse number
    fn eat_number(&mut self) -> LexerResult {
        let number: SpannedToken = match self.ch0 {
            Some('0') => match self.ch1 {
                Some('x') | Some('X') => {
                    let start = self.cursor();
                    self.next_char().expect("number: expected 0x");
                    self.next_char().expect("number: expected 0x");
                    self.eat_radix_number(start, 16, "0x")?
                }
                Some('o') | Some('O') => {
                    let start = self.cursor();
                    self.next_char().expect("number: expected 0x");
                    self.next_char().expect("number: expected 0x");
                    self.eat_radix_number(start, 8, "0o")?
                }
                Some('b') | Some('B') => {
                    let start = self.cursor();
                    self.next_char().expect("number: expected 0x");
                    self.next_char().expect("number: expected 0x");
                    self.eat_radix_number(start, 2, "0b")?
                }
                _ => self.eat_decimal_number()?,
            },
            _ => self.eat_decimal_number()?,
        };

        if self.ch0 == Some('_') {
            let location = self.cursor();
            return Err(LexerError {
                kind: LexerErrorKind::NumberTrailingUnderScore,
                location: Span {
                    start: location,
                    end: location,
                },
            });
        }

        Ok(number)
    }

    fn eat_radix_number(&mut self, start: usize, radix: u32, prefix: &str) -> LexerResult {
        let num = self.parse_number_with_radix(radix);

        if num.is_empty() {
            let location = self.cursor();
            Err(LexerError {
                kind: LexerErrorKind::EmptyRadix,
                location: Span {
                    start: location,
                    end: location,
                },
            })
        } else {
            let value = format!("{prefix}{num}");
            let int_value =
                super::parse_int_value(&value).expect("should have parsed into an int value");

            Ok((
                start,
                Token::Int {
                    value: value.into(),
                    int_value,
                },
                self.cursor(),
            ))
        }
    }

    fn eat_decimal_number(&mut self) -> LexerResult {
        let start = self.cursor();
        let mut value = String::new();

        // 1e110
        // 1.112e100
        // 1e-100
        // 1e+100
        // 1100.1e-100

        let mut is_decimal = false;

        // try to parse an integer;
        value.push_str(&self.parse_number_with_radix(10));

        // TODO: check for names;

        // if matches!(self.ch0, Some('e') | Some('E') | Some('.')) {

        if (self.ch0 == Some('.')
            && (self.ch1 != Some('.') || (!self.ch1.is_some_and(|c| self.is_name_start(c)))))
            || matches!(self.ch0, Some('e') | Some('E'))
        {
            is_decimal = true;

            let exp_or_dot = self
                .next_char()
                .expect("eat_decimal_number: Expected a e, E, or . ");

            value.push(exp_or_dot);

            match exp_or_dot {
                'e' | 'E' => {
                    value.push_str(self.parse_exponent()?.as_str());
                }
                '.' => {
                    let after_decimal_values = self.parse_number_with_radix(10);

                    value.push_str(after_decimal_values.as_str());

                    if !after_decimal_values.is_empty() && matches!(self.ch0, Some('e') | Some('E'))
                    {
                        value.push(
                            self.next_char()
                                .expect("eat_decimal_number: expected a e or E"),
                        );

                        value.push_str(self.parse_exponent()?.as_str());
                    }
                }
                _ => unreachable!("eat_decimal_number: Something went wrong"),
            }
        }

        let token = if is_decimal {
            Token::Float {
                value: value.into(),
            }
        } else {
            let int_value =
                super::parse_int_value(&value).expect("should have parsed to an int value");

            Token::Int {
                value: value.into(),
                int_value,
            }
        };

        let end = self.cursor();

        Ok((start, token, end))
    }

    // the e or E should be eaten before caliing this
    fn parse_exponent(&mut self) -> Result<String, LexerError> {
        let mut value = String::new();
        let start = self.cursor();

        if matches!(self.ch0, Some('-') | Some('+')) {
            value.push(
                self.next_char()
                    .expect("eat_decimal_number: expected a + or - "),
            );
        }

        let exp_val = self.parse_number_with_radix(10);

        if exp_val.is_empty() {
            Err(LexerError {
                kind: LexerErrorKind::MissingExponentValue,
                location: Span {
                    start,
                    end: self.cursor(),
                },
            })
        } else {
            value.push_str(&exp_val);
            Ok(value)
        }
    }

    fn parse_number_with_radix(&mut self, radix: u32) -> String {
        let mut number = String::new();

        loop {
            if let Some(ch) = self.eat_digit(radix) {
                number.push(ch)
            } else if self.ch0 == Some('_') && Self::match_radix(self.ch1, radix) {
                self.next_char();
            } else {
                break;
            }
        }

        number
    }

    fn eat_digit(&mut self, radix: u32) -> Option<char> {
        if Self::match_radix(self.ch0, radix) {
            Some(self.next_char().expect("eat_digit: expected a digit"))
        } else {
            None
        }
    }

    fn match_radix(c: Option<char>, radix: u32) -> bool {
        match radix {
            2 | 8 | 10 | 16 => c.filter(|c| c.is_digit(radix)).is_some(),
            unknown => panic!("Radix not found! {} ", unknown),
        }
    }

    fn is_number_start(&self, c: char) -> bool {
        c.is_ascii_digit()
    }

    fn next_char(&mut self) -> Option<char> {
        let cur = self.ch0;
        let next = self.chars.next();

        self.ch0 = self.ch1;
        self.ch1 = next;

        if cur.is_some() || self.ch0.is_some() {
            self.start = self.end;
            self.end += 1;
        }

        cur
    }

    #[inline]
    fn queue(&mut self, value: SpannedToken) {
        self.token_queue.push(value)
    }

    #[inline]
    fn cursor(&self) -> usize {
        self.start
    }
}

impl<T> Iterator for Lexer<T>
where
    T: Iterator<Item = char>,
{
    type Item = LexerResult;

    fn next(&mut self) -> Option<Self::Item> {
        match self.advance() {
            Ok((_, Token::Eof, _)) => None,
            t => Some(t),
        }
    }
}

#[allow(unused)]
fn get_keyword_from_str(string: &str) -> Option<Token> {
    let may_be_keyword = match string {
        "as" => Token::As,
        "class" => Token::Class,
        "let" => Token::Let,
        "mut" => Token::Mut,
        "panic" => Token::Panic,
        "pub" => Token::Pub,
        "return" => Token::Return,
        "self" => Token::ClassSelf,
        "using" => Token::Using,
        "if" => Token::If,
        "else" => Token::Else,
        "todo" => Token::Todo,
        "fn" => Token::Fn,
        "for" => Token::For,
        "in" => Token::In,
        "loop" => Token::Loop,

        _ => Token::Unknown,
    };

    match may_be_keyword {
        Token::Unknown => None,
        token => Some(token),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_numbers {
        use super::*;

        #[allow(unused)]
        fn print_mock_test_data(code: &str) {
            let lexer = Lexer::new(code.chars());

            let tokens: Vec<String> = lexer
                .map(|res| res.unwrap())
                .filter(|res| res.1 != Token::Unknown)
                .map(|res| {
                    let thing = match res.1 {
                        Token::Float { .. } => "float",
                        Token::Int { .. } => "int",
                        _ => unreachable!(),
                    };
                    match res.1 {
                        Token::Float { value } | Token::Int { value, .. } => {
                            format!("{}(\"{}\", {}, {})", thing, &value.as_str(), res.0, res.2)
                        }
                        _ => unreachable!(),
                    }
                })
                .collect();

            let joined = tokens.join(",\n");
            println!("{}", joined);
        }

        fn create_test_case() -> Vec<SpannedToken> {
            // the + and - should not affect the parsing
            let code = r#"
            100e10
            1e110
            1.112e100
            1e-100
            1E-100
            1_00e100
            -1e10
            -1e-100
            1e+100
            1_100.1e-100
            0e0
            0.0e0
            0e-1
            0e+1
            1e+10
            2.5e+3
            -3.14e+2
            10
            20
            +20
            -30
            -40
        "#
            .trim()
            .to_owned();

            // print_mock_test_data(&code);

            let lexer = Lexer::new(code.chars());
            let tokens: Vec<SpannedToken> = lexer
                .map(|res| res.unwrap())
                .filter(|res| matches!(res.1, Token::Int { .. } | Token::Float { .. }))
                .collect();

            //
            tokens
        }

        fn float(value: &str, start: usize, end: usize) -> SpannedToken {
            let token = Token::Float {
                value: value.into(),
            };

            (start, token, end)
        }

        fn int(value: &str, start: usize, end: usize) -> SpannedToken {
            let int_value =
                crate::parser::parse_int_value(value).expect("should have parsed to an int value");
            let token = Token::Int {
                value: value.into(),
                int_value, //
            };

            (start, token, end)
        }

        #[test]
        fn should_parse_numbers_correcty_with_spans() {
            // TODO add more _ tests
            let tokens = create_test_case();

            let expected: Vec<SpannedToken> = vec![
                float("100e10", 0, 6),
                float("1e110", 19, 24),
                float("1.112e100", 37, 46),
                float("1e-100", 59, 65),
                float("1E-100", 78, 84),
                float("100e100", 97, 105),
                float("1e10", 119, 123),
                float("1e-100", 137, 143),
                float("1e+100", 156, 162),
                float("1100.1e-100", 175, 187),
                float("0e0", 200, 203),
                float("0.0e0", 216, 221),
                float("0e-1", 234, 238),
                float("0e+1", 251, 255),
                float("1e+10", 268, 273),
                float("2.5e+3", 286, 292),
                float("3.14e+2", 306, 313),
                int("10", 326, 328),
                int("20", 341, 343),
                int("20", 357, 359),
                int("30", 373, 375),
                int("40", 389, 391),
            ];

            assert_eq!(tokens, expected, "There is some problems parsing numbers");
        }

        #[test]
        fn should_parse_to_actual_datatype() {
            let expected_floats: Vec<f64> = vec![
                100e10,
                1e110,
                1.112e100,
                1e-100,
                1E-100,
                100e100,
                1e10,
                1e-100,
                1e+100,
                1100.1e-100,
                0e0,
                0.0e0,
                0e-1,
                0e+1,
                1e+10,
                2.5e+3,
                3.14e+2,
            ];

            let expected_ints: Vec<i32> = vec![10, 20, 20, 30, 40];

            let tokens = create_test_case();

            let mut floats: Vec<f64> = vec![];
            let mut ints: Vec<i32> = vec![];

            for item in tokens.iter() {
                let token = &item.1;
                match token {
                    Token::Float { value } => {
                        let float_val: f64 = value.parse().unwrap();
                        floats.push(float_val);
                    }
                    Token::Int { value, .. } => {
                        let int_val: i32 = value.parse().unwrap();
                        ints.push(int_val);
                    }
                    _ => {}
                }
            }

            assert_eq!(expected_floats, floats);
            assert_eq!(expected_ints, ints);
        }

        #[test]
        fn should_parse_numbers_with_radix_correctly() {
            let expected = [
                int("10", 0, 2),
                int("0xa", 19, 22),
                int("0xff", 39, 43),
                int("0o10", 60, 64),
            ];

            let code = r#"
                10
                0xa
                0xff
                0o10
            "#
            .trim();

            let lexer = Lexer::new(code.chars());
            let tokens: Vec<SpannedToken> = lexer
                .map(|res| res.unwrap())
                .filter(|res| matches!(res.1, Token::Int { .. } | Token::Float { .. }))
                .collect();

            assert_eq!(tokens, expected)
        }
    }

    #[test]
    fn number_parsing_should_not_be_greedy_with_dot_access() {
        use Token::*;
        let code = "
            10.0
            10.radians()
            10.degreees()
        ";

        let expected = [
            (0, NewLine, 1),
            (
                13,
                Float {
                    value: "10.0".into(),
                },
                17,
            ),
            (17, NewLine, 18),
            (
                30,
                Float {
                    value: "10.".into(),
                },
                33,
            ),
            (
                33,
                Name {
                    name: "radians".into(),
                },
                40,
            ),
            (40, LParen, 41),
            (41, RParen, 42),
            (42, NewLine, 43),
            (
                55,
                Float {
                    value: "10.".into(),
                },
                58,
            ),
            (
                58,
                Name {
                    name: "degreees".into(),
                },
                66,
            ),
            (66, LParen, 67),
            (67, RParen, 68),
            (68, NewLine, 69),
        ];

        let lexer = Lexer::new(code.chars());
        let tokens: Vec<SpannedToken> = lexer.map(|res| res.unwrap()).collect();

        assert_eq!(tokens, expected)
    }

    fn lex_input(input: &str) -> Vec<SpannedToken> {
        let mut lexer = Lexer::new(input.chars());
        let mut tokens = Vec::new();

        while let Ok(token) = lexer.advance() {
            if matches!(token.1, Token::Eof) {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }

        tokens
    }

    #[test]
    fn test_single_characters() {
        let input = "( ) [ ] { } + - ; :";
        let tokens = lex_input(input);
        assert_eq!(
            tokens,
            vec![
                (0, Token::LParen, 1),
                (2, Token::RParen, 3),
                (4, Token::LBracket, 5),
                (6, Token::RBracket, 7),
                (8, Token::LCurly, 9),
                (10, Token::RCurly, 11),
                (12, Token::Plus, 13),
                (14, Token::Minus, 15),
                (16, Token::SemiColon, 17),
                (18, Token::Colon, 19),
                (19, Token::Eof, 19)
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        let input = "abc _abc _123";
        let tokens = lex_input(input);
        assert_eq!(
            tokens,
            vec![
                (0, Token::Name { name: "abc".into() }, 3),
                (
                    4,
                    Token::DiscardName {
                        name: "_abc".into()
                    },
                    8
                ),
                (
                    9,
                    Token::DiscardName {
                        name: "_123".into()
                    },
                    13
                ),
                (13, Token::Eof, 13)
            ]
        );
    }

    #[test]
    fn test_numbers() {
        let input = "123 0x1A 0o77 0b101";
        let tokens = lex_input(input);
        assert_eq!(
            tokens,
            vec![
                (
                    0,
                    Token::Int {
                        value: "123".into(),
                        int_value: 123
                    },
                    3
                ),
                (
                    4,
                    Token::Int {
                        value: "0x1A".into(),
                        int_value: 26
                    },
                    8
                ),
                (
                    9,
                    Token::Int {
                        value: "0o77".into(),
                        int_value: 63
                    },
                    13
                ),
                (
                    14,
                    Token::Int {
                        value: "0b101".into(),
                        int_value: 5
                    },
                    19
                ),
                (19, Token::Eof, 19)
            ]
        );
    }

    #[test]
    fn test_strings() {
        let input = r#""hello" "escaped\nstring""#;
        let tokens = lex_input(input);
        assert_eq!(
            tokens,
            vec![
                (
                    0,
                    Token::String {
                        value: "hello".into()
                    },
                    7
                ),
                (
                    8,
                    Token::String {
                        value: "escaped\nstring".into()
                    },
                    25
                ),
                (25, Token::Eof, 25)
            ]
        );
    }

    #[test]
    fn test_eof() {
        let input = "";
        let tokens = lex_input(input);
        assert_eq!(tokens, vec![(0, Token::Eof, 0),]);
    }

    #[test]
    fn test_operators_and_symbols() {
        let input = "== . .. ... -> + * - /";
        let tokens = lex_input(input);
        assert_eq!(
            tokens,
            vec![
                (0, Token::EqEq, 2),
                (3, Token::Dot, 4),
                (5, Token::DotDot, 7),
                (8, Token::DotDotDot, 11),
                (12, Token::ArrowRight, 14),
                (15, Token::Plus, 16),
                (17, Token::Star, 18),
                (19, Token::Minus, 20),
                (21, Token::Slash, 22),
                (22, Token::Eof, 22)
            ]
        );
    }

    #[test]
    fn test_newlines_and_whitespace() {
        let input = " \t\n\r\n";
        let tokens = lex_input(input);
        assert_eq!(
            tokens,
            vec![
                (2, Token::NewLine, 3), // LF
                (3, Token::NewLine, 5), // CRLF
                (5, Token::Eof, 5)
            ]
        );
    }

    #[test]
    fn test_invalid_tokens() {
        let input = "#$%";
        let tokens = lex_input(input);
        assert_eq!(
            tokens,
            vec![
                (0, Token::Unknown, 1),
                (1, Token::Unknown, 2),
                (2, Token::Unknown, 3),
                (3, Token::Eof, 3)
            ]
        );
    }

    #[test]
    fn test_error_non_terminated_string() {
        let input = r#""hello"#;
        let mut lexer = Lexer::new(input.chars());
        assert!(matches!(
            lexer.advance(),
            Err(LexerError {
                kind: LexerErrorKind::NonTerminatedStringLiteral,
                location: Span { start: 0, end: 6 }
            })
        ));
    }

    #[test]
    fn test_error_trailing_underscore() {
        let input = "123_";
        let mut lexer = Lexer::new(input.chars());
        assert!(matches!(
            lexer.advance(),
            Err(LexerError {
                kind: LexerErrorKind::NumberTrailingUnderScore,
                location: Span { start: 3, end: 3 }
            })
        ));
    }
}
