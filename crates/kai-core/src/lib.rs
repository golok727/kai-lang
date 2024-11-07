pub mod parser;
use parser::lexer::Lexer;

#[derive(Default, Debug)]
pub struct Vm {}

impl Vm {
    pub fn run(&mut self) {
        let code = r#"
            let thing = a;
            let a = thing;
        "#
        .trim()
        .to_owned();

        let lexer = Lexer::new(code.chars());
        let tokens: Vec<String> = lexer
            .map(|res| {
                let res = res.unwrap();
                format!(
                    "Token = {}, start = {}, end = {}",
                    res.1,
                    &res.0.to_string(),
                    res.2
                )
            })
            .collect();

        dbg!(tokens);
    }
}
