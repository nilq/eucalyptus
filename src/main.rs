use std::rc::Rc;

mod eucalyptus;
use eucalyptus::*;

fn main() {
    let test = r#"
let a = fun b -> b
a 10
    "#;

    let lexer = lexer(&mut test.chars());

    let traveler   = Traveler::new(lexer.collect());
    let mut parser = Parser::new(traveler);

    match parser.parse() {
        Err(why)  => println!("error: {}", why),
        Ok(stuff) => {
            let symtab  = Rc::new(SymTab::new_global());
            let typetab = Rc::new(TypeTab::new_global());

            let valtab = Rc::new(ValTab::new_global());
            
            for s in stuff.iter() {
                match s.visit(&symtab, &typetab, &valtab) {
                    Ok(()) => (),
                    Err(e) => {
                        println!("{}", e);
                        break
                    },
                }

                match s.get_type(&symtab, &typetab, &valtab) {
                    Ok(v) => println!("type: {:#?}", v),
                    Err(e) => {
                        println!("{}", e);
                        break
                    },
                }
                
                match s.eval(&symtab, &valtab) {
                    Ok(v) => println!("value: {:#?}", v),
                    Err(e) => {
                        println!("{}", e);
                        break
                    },
                }
            }
        },
    }
}
