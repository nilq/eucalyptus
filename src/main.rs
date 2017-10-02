mod eucalyptus;

use eucalyptus::*;

fn main() {
    let test = r#"
let a = {1, 2, 3}

print a[1]

let add a b =
  let b   = a + b
  let idk = fun a b -> a + b
  b

idk 1, 2, (fun a b -> a + b)
    "#;

    let lexer = lexer(&mut test.chars());

    let traveler   = Traveler::new(lexer.collect());
    let mut parser = Parser::new(traveler);

    match parser.parse() {
        Err(why)  => println!("error: {}", why),
        Ok(stuff) => println!("{:#?}", stuff),
    }
}
