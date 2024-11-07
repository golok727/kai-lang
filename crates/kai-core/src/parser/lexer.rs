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
                let name = self.eat_name()?;
                self.queue(name)
            } else if self.is_number_start(c, self.c1) {
                let number = self.eat_number()?;
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

    fn eat_number(&mut self) -> LexerResult {
        dbg!(self.c0, self.c1);
        todo!("number is not implemented yet")
    }

    fn eat_name(&mut self) -> LexerResult {
        let mut name = String::from("");

        let name_start = self.cursor();

        while self.is_name_continuation() {
            name.push(self.next_char().expect("eat_name is not getting food"))
        }

        let name_end = self.cursor();

        let token = match get_keyword_from_str(&name) {
            Some(token) => token,
            None => {
                if name.starts_with('_') {
                    Token::DiscardName(name.into())
                } else {
                    Token::Name(name.into())
                }
            }
        };

        Ok((name_start, token, name_end))
    }

    fn is_number_start(&self, start: char, next: Option<char>) -> bool {
        match start {
            '0'..='9' => true,
            '_' => matches!(next, Some('0'..='9')),
            _ => false,
        }
    }

    fn is_name_start(&self, c: char) -> bool {
        matches!(c, '_' | 'a'..='z')
    }

    fn is_name_continuation(&self) -> bool {
        self.c0
            .map(|c| matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
            .unwrap_or(false)
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
