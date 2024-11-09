use super::error::LexerError;
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
            println!("char({})", c);
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
                self.eat_decomal_digits()
            }
        } else {
            self.eat_decomal_digits()
        };

        Ok(number)
    }

    fn eat_decomal_digits(&mut self) -> SpannedToken {
        self.parse_int_or_decimal(true)
    }

    fn parse_int_or_decimal(&mut self, expect_decimal: bool) -> SpannedToken {
        let start = self.cursor();
        let mut value = String::new();

        if self.c0 == Some('-') {
            value.push(
                self.next_char()
                    .expect("parse_int_or_decimal: expected a '-' "),
            )
        };

        value.push_str(&self.try_parse_number_with_radix(10));

        if expect_decimal && self.c0 == Some('.') {
            panic!("Float not implemeted")
        }

        let end = self.cursor();

        (
            start,
            Token::Int {
                value: value.into(),
            },
            end,
        )
    }

    fn try_parse_number_with_radix(&mut self, radix: u32) -> String {
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
