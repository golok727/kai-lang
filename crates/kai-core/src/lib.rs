pub mod ast;
pub mod parser;
use parser::lexer::Lexer;

#[derive(Default, Debug)]
pub struct Vm {}

impl Vm {
    pub fn run(&mut self) {
        let code = r#"
            let a = 10; 
            let b = 1.2; 
        "#
        .trim()
        .to_owned();

        let lexer = Lexer::new(code.chars());

        let tokens: Vec<String> = lexer
            .map(|res| res.unwrap())
            // .filter(|res| res.1 != Token::Unknown)
            .map(|res| {
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

// fn create_test_data(code: &str) {
// let lexer = Lexer::new(code.char());

// let tokens: Vec<String> = lexer
//     .map(|res| res.unwrap())
//     .filter(|res| res.1 != Token::Unknown)
//     .map(|res| {
//         let thing = match res.1 {
//             Token::Float { .. } => "float",
//             Token::Int { .. } => "int",
//             _ => unreachable!(),
//         };
//         match res.1 {
//             Token::Float { value } | Token::Int { value } => {
//                 format!("{}(\"{}\", {}, {})", thing, &value.as_str(), res.0, res.2)
//             }
//             _ => unreachable!(),
//         }
//     })
//     .collect();

// let joined = tokens.join(",\n");
// println!("{}", joined);
// }
