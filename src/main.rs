mod eucalyptus;

use eucalyptus::*;

fn main() {
    let test = r#"
let a = 123
let add a b = a + b
    "#;
    
    let lexer = lexer(&mut test.chars());

    let traveler   = Traveler::new(lexer.collect());
    let mut parser = Parser::new(traveler);
    
    match parser.parse() {
        Err(why)  => println!("error: {}", why),
        Ok(stuff) => println!("{:#?}", stuff),
    }
}
