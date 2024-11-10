pub mod ast;
pub mod parser;
use parser::lexer::Lexer;

use crate::{ast::span::span, parser::lexer::SpannedToken};

#[derive(Default, Debug)]
pub struct Vm {}

impl Vm {
    pub fn run(&mut self) {
        let code = r#"
            let a = 10; 
            let b = 1.2; 
            0x10 == 0x10; 
        "#
        .trim()
        .to_owned();

        let s = span(59, 63);
        dbg!(s.src_text(&code));

        let lexer = Lexer::new(code.chars());

        let tokens: Vec<SpannedToken> = lexer
            .map(|res| res.unwrap())
            // .filter(|res| matches!(res.1, Token::Int { .. } | Token::Float { .. }))
            .collect();

        // println!("{:#?}", &tokens);

        let thing: Vec<String> = tokens
            .iter()
            .map(|res| {
                format!(
                    "Token = {}, start = {}, end = {}",
                    res.1,
                    &res.0.to_string(),
                    res.2
                )
            })
            .collect();

        dbg!(thing);
    }
}
