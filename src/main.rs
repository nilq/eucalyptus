mod eucalyptus;

use eucalyptus::*;

fn main() {
    let test = r#"
let a b = b + 10
    "#;
    
    let lexer = lexer(&mut test.chars());

    for token in lexer {
        println!("{:#?}", token)
    }
}
