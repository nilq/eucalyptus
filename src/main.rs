use std::rc::Rc;

mod eucalyptus;
use eucalyptus::*;

fn main() {
    let test = r#"
let a = {1, 2, 3} - 1
a
    "#;

    let lexer = lexer(&mut test.chars());

    let traveler   = Traveler::new(lexer.collect());
    let mut parser = Parser::new(traveler);

    match parser.parse() {
        Err(why)  => println!("error: {}", why),
        Ok(stuff) => {
            let symtab  = Rc::new(SymTab::new_global());
            let typetab = Rc::new(TypeTab::new_global());

            let mut valtab = Rc::new(ValTab::new_global());
            
            for s in stuff.iter() {
                match s.visit(&symtab, &typetab) {
                    Ok(()) => (),
                    Err(e) => {
                        println!("{}", e);
                        break
                    },
                }
                
                match s.eval(&symtab, &mut valtab) {
                    Ok(v) => println!("{:#?}", v),
                    Err(e) => {
                        println!("{}", e);
                        break
                    },
                }
            }
        },
    }
}
