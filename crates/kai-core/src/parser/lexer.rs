use crate::ast::span::Span;

use super::error::{LexerError, LexerErrorKind};
use super::token::Token;

// start  token  end
pub type SpannedToken = (usize, Token, usize);

pub struct Lexer<T: Iterator<Item = char>> {
    chars: T,
    c0: Option<char>,
    c1: Option<char>,
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
            c0: None,
            c1: None,
            start: 0,
            end: 0,
            token_queue: Vec::new(),
        };

        lexer.next_char();
        lexer.next_char();

        lexer
    }

    pub fn advance(&mut self) -> LexerResult {
        while self.token_queue.is_empty() {
            self.consume()?;
        }

        Ok(self.token_queue.remove(0))
    }

    fn consume(&mut self) -> Result<(), LexerError> {
        if let Some(c) = self.c0 {
            // println!("char({})", c);
            if self.is_name_start(c) {
                let name = self.name()?;
                self.queue(name)
            } else if self.is_number_start(c, self.c1) {
                let number = self.number()?;
                self.queue(number);
            } else {
                self.queue((0, Token::Unknown, 0));
            }
            // TODO remove
            self.next_char();
        } else {
            self.queue((0, Token::Eof, 0));
        }

        Ok(())
    }

    // name
    fn name(&mut self) -> LexerResult {
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
        self.c0
            .map(|c| matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
            .unwrap_or(false)
    }

    // number
    fn number(&mut self) -> LexerResult {
        let number: SpannedToken = if self.c0 == Some('0') {
            if matches!(self.c1, Some('x') | Some('X')) {
                todo!("Hex radix")
            } else if matches!(self.c1, Some('o') | Some('O')) {
                todo!("ocatal radix")
            } else if matches!(self.c1, Some('b') | Some('B')) {
                todo!("binary radix")
            } else {
                self.eat_decimal_number()?
            }
        } else {
            self.eat_decimal_number()?
        };

        Ok(number)
    }

    fn eat_decimal_number(&mut self) -> LexerResult {
        let start = self.cursor();
        let mut value = String::new();

        // 1e110
        // 1.112e100
        // 1e-100
        // 1e+100
        // 1100.1e-100

        if self.c0 == Some('-') {
            value.push(
                self.next_char()
                    .expect("parse_int_or_decimal: expected a '-' "),
            )
        };

        let mut is_decimal = false;

        // try to parse an integer;
        value.push_str(&self.parse_number_with_radix(10));

        if matches!(self.c0, Some('e') | Some('E') | Some('.')) {
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

                    if !after_decimal_values.is_empty() && matches!(self.c0, Some('e') | Some('E'))
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
            Token::Int {
                value: value.into(),
            }
        };

        let end = self.cursor();

        Ok((start, token, end))
    }

    // the e or E should be eaten before caliing this
    fn parse_exponent(&mut self) -> Result<String, LexerError> {
        let mut value = String::new();
        let start = self.cursor();

        if matches!(self.c0, Some('-') | Some('+')) {
            value.push(
                self.next_char()
                    .expect("eat_decimal_number: expected a + or - "),
            );
        }

        let exp_val = self.parse_number_with_radix(10);

        if exp_val.is_empty() {
            Err(LexerError {
                kind: LexerErrorKind::BadExponent,
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
            } else if self.c0 == Some('_') && Self::match_radix(self.c1, radix) {
                self.next_char();
            } else {
                break;
            }
        }

        number
    }

    fn eat_digit(&mut self, radix: u32) -> Option<char> {
        if Self::match_radix(self.c0, radix) {
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

    fn is_number_start(&self, start: char, next: Option<char>) -> bool {
        match start {
            '0'..='9' => true,
            // negative numbers
            '-' => matches!(next, Some('0'..='9')),
            _ => false,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let cur = self.c0;
        let next = self.chars.next();

        self.c0 = self.c1;
        self.c1 = next;

        if cur.is_some() || self.c0.is_some() {
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
        "let" => Token::Let,
        "mut" => Token::Mut,
        "return" => Token::Return,
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

    fn float(value: &str, start: usize, end: usize) -> SpannedToken {
        let token = Token::Float {
            value: value.into(),
        };

        (start, token, end)
    }

    fn int(value: &str, start: usize, end: usize) -> SpannedToken {
        let token = Token::Int {
            value: value.into(),
        };

        (start, token, end)
    }

    #[test]
    fn parse_numbers() {
        let code = r#"
            100e10
            1e110
            1.112e100
            1e-100
            1E-100
            100e100
            -1e10
            -1e-100
            1e+100
            1100.1e-100
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

        let lexer = Lexer::new(code.chars());
        let tokens: Vec<SpannedToken> = lexer
            .map(|res| res.unwrap())
            .filter(|res| res.1 != Token::Unknown)
            .collect();

        let expected: Vec<SpannedToken> = vec![
            float("100e10", 0, 6),
            float("1e110", 19, 24),
            float("1.112e100", 37, 46),
            float("1e-100", 59, 65),
            float("1E-100", 78, 84),
            float("100e100", 97, 104),
            float("-1e10", 117, 122),
            float("-1e-100", 135, 142),
            float("1e+100", 155, 161),
            float("1100.1e-100", 174, 185),
            float("0e0", 198, 201),
            float("0.0e0", 214, 219),
            float("0e-1", 232, 236),
            float("0e+1", 249, 253),
            float("1e+10", 266, 271),
            float("2.5e+3", 284, 290),
            float("-3.14e+2", 303, 311),
            int("10", 324, 326),
            int("20", 339, 341),
            int("20", 355, 357),
            int("-30", 370, 373),
            int("-40", 386, 389),
        ];

        assert_eq!(tokens, expected, "There is some problems parsing numbers");
    }
}
