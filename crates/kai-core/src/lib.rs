pub mod parser;
use parser::lexer::Lexer;

#[derive(Default, Debug)]
pub struct Vm {}

impl Vm {
    pub fn run(&mut self) {
        let code = r#"
            let thing = a;
        "#
        .trim()
        .to_owned();

        let lexer = Lexer::new(code.chars());
        let tokens: Vec<String> = lexer.map(|e| e.unwrap().1.to_string()).collect();

        dbg!(tokens);
    }
}
