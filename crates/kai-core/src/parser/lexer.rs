use crate::ast::span::Span;

use super::error::{LexerError, LexerErrorKind};
use super::token::TokenKind;

// start  token  end
pub type Token = (usize, TokenKind, usize);

impl From<Token> for Span {
    fn from(value: Token) -> Self {
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
    token_queue: Vec<Token>,
}

pub type LexerResult = Result<Token, LexerError>;

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
            self.queue((pos, TokenKind::Eof, pos));
        }

        Ok(())
    }

    #[inline]
    fn eat_single_token(&mut self, tk: TokenKind) -> TokenKind {
        self.next_char();
        tk
    }

    fn eat_block_comment(&mut self) -> LexerResult {
        let start = self.cursor();
        debug_assert_eq!(self.next_char(), Some('/'));
        debug_assert_eq!(self.next_char(), Some('*'));

        let mut doc_str: Option<String> = if self.ch0 == Some('*') {
            self.next_char();
            Some(String::new())
        } else {
            None
        };

        loop {
            match self.ch0 {
                Some('*') => {
                    if self.ch1 == Some('/') {
                        self.next_char();
                        self.next_char();
                        break;
                    } else {
                        let c = self.next_char().expect("expected a character");
                        if let Some(ref mut doc) = doc_str {
                            doc.push(c);
                        }
                    }
                }
                _ => {
                    let c = self.next_char().expect("expected a character");
                    if let Some(ref mut doc) = doc_str {
                        doc.push(c);
                    }
                }
            }
        }

        let end = self.cursor();
        let spanned = match doc_str {
            Some(doc) => (
                start,
                TokenKind::DocComment {
                    comment: doc.into(),
                },
                end,
            ),
            _ => (start, TokenKind::Comment, end),
        };

        Ok(spanned)
    }

    fn eat_comment(&mut self) -> Token {
        let start = self.cursor();
        debug_assert_eq!(self.next_char(), Some('/'));
        debug_assert_eq!(self.next_char(), Some('/'));
        // FIXME will source be normalized ?
        loop {
            match self.ch0 {
                Some('\n') | None => {
                    self.next_char();
                    break;
                }
                _ => {
                    self.next_char();
                }
            }
        }

        let end = self.cursor();
        (start, TokenKind::Comment, end)
    }

    fn eat_single_character(&mut self) -> Result<(), LexerError> {
        if let Some(c) = self.ch0 {
            let start = self.cursor();

            let token: Option<TokenKind> = match c {
                '\r' => {
                    self.next_char();
                    if self.ch0 == Some('\n') {
                        self.next_char(); // consume '\n' after '\r'
                    }
                    Some(TokenKind::NewLine) // return NewLine token for CR or CRLF
                }
                '\n' => {
                    Some(self.eat_single_token(TokenKind::NewLine)) // return NewLine token for LF
                }
                ' ' | '\t' | '\x0C' => {
                    self.next_char(); // eat whitespace
                    None
                }
                '(' => Some(self.eat_single_token(TokenKind::LParen)),
                ')' => Some(self.eat_single_token(TokenKind::RParen)),

                '[' => Some(self.eat_single_token(TokenKind::LBracket)),
                ']' => Some(self.eat_single_token(TokenKind::RBracket)),

                '{' => Some(self.eat_single_token(TokenKind::LCurly)),
                '}' => Some(self.eat_single_token(TokenKind::RCurly)),

                '/' => match self.ch1 {
                    Some('/') => {
                        let comment = self.eat_comment();
                        self.queue(comment);
                        None
                    }
                    Some('*') => {
                        let comment = self.eat_block_comment()?;
                        self.queue(comment);
                        None
                    }

                    Some('=') => {
                        self.next_char();
                        Some(self.eat_single_token(TokenKind::DivEq))
                    }
                    _ => Some(self.eat_single_token(TokenKind::Slash)),
                },

                '-' => match self.ch1 {
                    Some('=') => {
                        self.next_char();
                        self.next_char();
                        Some(TokenKind::MinusEq)
                    }
                    Some('>') => {
                        self.next_char();
                        self.next_char();
                        Some(TokenKind::ArrowRight)
                    }
                    _ => Some(self.eat_single_token(TokenKind::Minus)),
                },

                '+' | '*' | '!' | '=' | '>' | '<' => {
                    let tok = match c {
                        '+' => (TokenKind::Plus, TokenKind::PlusEq),
                        '*' => (TokenKind::Star, TokenKind::MulEq),
                        '!' => (TokenKind::Bang, TokenKind::NotEq),
                        '=' => (TokenKind::Eq, TokenKind::EqEq),
                        '>' => (TokenKind::Gt, TokenKind::GtEq),
                        '<' => (TokenKind::Lt, TokenKind::LtEq),
                        _ => unreachable!(),
                    };

                    if self.ch1 == Some('=') {
                        self.next_char();
                        Some(self.eat_single_token(tok.1))
                    } else {
                        Some(self.eat_single_token(tok.0))
                    }
                }

                '.' => {
                    self.next_char();

                    let mut n_dots = 1;

                    if self.ch0 == Some('.') {
                        n_dots += 1;
                        self.next_char();
                    }

                    if self.ch0 == Some('=') {
                        self.next_char();
                        Some(TokenKind::DotDotEq)
                    } else {
                        if self.ch0 == Some('.') {
                            n_dots += 1;
                            self.next_char();
                        }

                        let token = match n_dots {
                            1 => TokenKind::Dot,
                            2 => TokenKind::DotDot,
                            3 => TokenKind::DotDotDot,
                            _ => unreachable!(),
                        };

                        Some(token)
                    }
                }

                ';' => Some(self.eat_single_token(TokenKind::SemiColon)),
                ':' => Some(self.eat_single_token(TokenKind::Colon)),
                '"' => {
                    let spanned = self.eat_double_quoted_string()?;
                    self.queue(spanned);
                    None
                }
                _ => {
                    self.next_char();
                    Some(TokenKind::Unknown)
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
                    TokenKind::DiscardName { name: name.into() }
                } else {
                    TokenKind::Name { name: name.into() }
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
            TokenKind::String {
                value: string.into(),
            },
            self.cursor(),
        ))
    }

    // parse number
    fn eat_number(&mut self) -> LexerResult {
        let number: Token = match self.ch0 {
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
                TokenKind::Int {
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
            TokenKind::Float {
                value: value.into(),
            }
        } else {
            let int_value =
                super::parse_int_value(&value).expect("should have parsed to an int value");

            TokenKind::Int {
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
    fn queue(&mut self, value: Token) {
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
            Ok((_, TokenKind::Eof, _)) => None,
            t => Some(t),
        }
    }
}

#[allow(unused)]
fn get_keyword_from_str(string: &str) -> Option<TokenKind> {
    let may_be_keyword = match string {
        "as" => TokenKind::As,
        "class" => TokenKind::Class,
        "let" => TokenKind::Let,
        "mut" => TokenKind::Mut,
        "panic" => TokenKind::Panic,
        "pub" => TokenKind::Pub,
        "return" => TokenKind::Return,
        "self" => TokenKind::ClassSelf,
        "using" => TokenKind::Using,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "todo" => TokenKind::Todo,
        "fn" => TokenKind::Fn,
        "for" => TokenKind::For,
        "in" => TokenKind::In,
        "loop" => TokenKind::Loop,

        _ => TokenKind::Unknown,
    };

    match may_be_keyword {
        TokenKind::Unknown => None,
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
                .filter(|res| res.1 != TokenKind::Unknown)
                .map(|res| {
                    let thing = match res.1 {
                        TokenKind::Float { .. } => "float",
                        TokenKind::Int { .. } => "int",
                        _ => unreachable!(),
                    };
                    match res.1 {
                        TokenKind::Float { value } | TokenKind::Int { value, .. } => {
                            format!("{}(\"{}\", {}, {})", thing, &value.as_str(), res.0, res.2)
                        }
                        _ => unreachable!(),
                    }
                })
                .collect();

            let joined = tokens.join(",\n");
            println!("{}", joined);
        }

        fn create_test_case() -> Vec<Token> {
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
            let tokens: Vec<Token> = lexer
                .map(|res| res.unwrap())
                .filter(|res| matches!(res.1, TokenKind::Int { .. } | TokenKind::Float { .. }))
                .collect();

            //
            tokens
        }

        fn float(value: &str, start: usize, end: usize) -> Token {
            let token = TokenKind::Float {
                value: value.into(),
            };

            (start, token, end)
        }

        fn int(value: &str, start: usize, end: usize) -> Token {
            let int_value =
                crate::parser::parse_int_value(value).expect("should have parsed to an int value");
            let token = TokenKind::Int {
                value: value.into(),
                int_value, //
            };

            (start, token, end)
        }

        #[test]
        fn should_parse_numbers_correcty_with_spans() {
            // TODO add more _ tests
            let tokens = create_test_case();

            let expected: Vec<Token> = vec![
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
                    TokenKind::Float { value } => {
                        let float_val: f64 = value.parse().unwrap();
                        floats.push(float_val);
                    }
                    TokenKind::Int { value, .. } => {
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
            let tokens: Vec<Token> = lexer
                .map(|res| res.unwrap())
                .filter(|res| matches!(res.1, TokenKind::Int { .. } | TokenKind::Float { .. }))
                .collect();

            assert_eq!(tokens, expected)
        }
    }

    #[test]
    fn number_parsing_should_not_be_greedy_with_dot_access() {
        use TokenKind::*;
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
        let tokens: Vec<Token> = lexer.map(|res| res.unwrap()).collect();

        assert_eq!(tokens, expected)
    }

    fn lex_input(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input.chars());
        let mut tokens = Vec::new();

        while let Ok(token) = lexer.advance() {
            if matches!(token.1, TokenKind::Eof) {
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
                (0, TokenKind::LParen, 1),
                (2, TokenKind::RParen, 3),
                (4, TokenKind::LBracket, 5),
                (6, TokenKind::RBracket, 7),
                (8, TokenKind::LCurly, 9),
                (10, TokenKind::RCurly, 11),
                (12, TokenKind::Plus, 13),
                (14, TokenKind::Minus, 15),
                (16, TokenKind::SemiColon, 17),
                (18, TokenKind::Colon, 19),
                (19, TokenKind::Eof, 19)
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
                (0, TokenKind::Name { name: "abc".into() }, 3),
                (
                    4,
                    TokenKind::DiscardName {
                        name: "_abc".into()
                    },
                    8
                ),
                (
                    9,
                    TokenKind::DiscardName {
                        name: "_123".into()
                    },
                    13
                ),
                (13, TokenKind::Eof, 13)
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
                    TokenKind::Int {
                        value: "123".into(),
                        int_value: 123
                    },
                    3
                ),
                (
                    4,
                    TokenKind::Int {
                        value: "0x1A".into(),
                        int_value: 26
                    },
                    8
                ),
                (
                    9,
                    TokenKind::Int {
                        value: "0o77".into(),
                        int_value: 63
                    },
                    13
                ),
                (
                    14,
                    TokenKind::Int {
                        value: "0b101".into(),
                        int_value: 5
                    },
                    19
                ),
                (19, TokenKind::Eof, 19)
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
                    TokenKind::String {
                        value: "hello".into()
                    },
                    7
                ),
                (
                    8,
                    TokenKind::String {
                        value: "escaped\nstring".into()
                    },
                    25
                ),
                (25, TokenKind::Eof, 25)
            ]
        );
    }

    #[test]
    fn test_eof() {
        let input = "";
        let tokens = lex_input(input);
        assert_eq!(tokens, vec![(0, TokenKind::Eof, 0),]);
    }

    #[test]
    fn test_operators_and_symbols() {
        let input = "== . .. ... -> + * - / += -= *= /= = ! != ..= > < >= <=";
        let tokens = lex_input(input);
        assert_eq!(
            tokens,
            vec![
                (0, TokenKind::EqEq, 2),
                (3, TokenKind::Dot, 4),
                (5, TokenKind::DotDot, 7),
                (8, TokenKind::DotDotDot, 11),
                (12, TokenKind::ArrowRight, 14),
                (15, TokenKind::Plus, 16),
                (17, TokenKind::Star, 18),
                (19, TokenKind::Minus, 20),
                (21, TokenKind::Slash, 22),
                (23, TokenKind::PlusEq, 25),
                (26, TokenKind::MinusEq, 28),
                (29, TokenKind::MulEq, 31),
                (32, TokenKind::DivEq, 34),
                (35, TokenKind::Eq, 36),
                (37, TokenKind::Bang, 38),
                (39, TokenKind::NotEq, 41),
                (42, TokenKind::DotDotEq, 45),
                (46, TokenKind::Gt, 47),
                (48, TokenKind::Lt, 49),
                (50, TokenKind::GtEq, 52),
                (53, TokenKind::LtEq, 55),
                (55, TokenKind::Eof, 55),
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
                (2, TokenKind::NewLine, 3), // LF
                (3, TokenKind::NewLine, 5), // CRLF
                (5, TokenKind::Eof, 5)
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
                (0, TokenKind::Unknown, 1),
                (1, TokenKind::Unknown, 2),
                (2, TokenKind::Unknown, 3),
                (3, TokenKind::Eof, 3)
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

    #[test]
    fn test_comments() {
        use TokenKind::*; 
        let input = "
            // hello code
            /*
                Thing
            */

            /**
             * Welcome to Kai lang
             * thing is a thing
            */
            
        ";

        let tokens = lex_input(input)
            .into_iter()
            .filter(|t| !matches!(t.1, TokenKind::NewLine))
            .collect::<Vec<Token>>();


        assert_eq!(tokens,
            [
                ( 13, Comment, 27, ),
                ( 39, Comment, 78, ),
                (
                    92,
                    DocComment {
                        comment: "\n             * Welcome to Kai lang\n             * thing is a thing\n            ".into(),
                    },
                    177,
                ),
                ( 199, Eof, 199, ),
            ]        
        );
    }
}
