use kai_core::{
    ast::span::span,
    parser::lexer::{Lexer, Token},
    runtime::Engine,
};

#[allow(unused)]
pub fn sandbox_lexer() {
    let code = r#"
        using kai.io;

        fn square(a: int) -> int {
            a * a
        }

        fn main() {
            for i in 1..=10 {
                io.print(square(i));
            }
        }

        "#
    .trim()
    .to_owned();

    let s = span(89, 106);

    dbg!(s.src_text(&code));

    let lexer = Lexer::new(code.chars());

    let tokens: Vec<Token> = lexer.map(|res| res.unwrap()).collect();

    // println!("{:#?}", &tokens);

    let thing: Vec<String> = tokens
        .iter()
        .map(|res| {
            // println!("{:#?}", &res.1);

            format!("Token(`{}`) => span({}, {})", res.1, res.0, res.2)
        })
        .collect();

    dbg!(thing);
}

pub fn main() {
    println!("Welcome to kai");

    sandbox_lexer();

    let mut engine = Engine::default();
    engine.run(false);
}
