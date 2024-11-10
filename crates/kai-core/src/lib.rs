pub mod ast;
pub mod parser;

use parser::lexer::Lexer;

use crate::{ast::span::span, parser::lexer::SpannedToken};

#[derive(Default, Debug)]
pub struct Vm {}

impl Vm {
    pub fn run(&mut self) {
        self.lexer();
    }

    pub fn lexer(&mut self) {
        let code = r#"
            let a = 10; 
            let b = 1.2; 
            0x10 == 0x10; 
            let thing= "hello \n\tworld"
        "#
        .trim()
        .to_owned();

        let s = span(89, 106);
        dbg!(s.src_text(&code));

        let lexer = Lexer::new(code.chars());

        let tokens: Vec<SpannedToken> = lexer.map(|res| res.unwrap()).collect();

        // println!("{:#?}", &tokens);

        let thing: Vec<String> = tokens
            .iter()
            .map(|res| {
                // println!("{:#?}", &res.1);

                format!("Token({}) => span({}, {})", res.1, res.0, res.2)
            })
            .collect();

        dbg!(thing);
    }
}
