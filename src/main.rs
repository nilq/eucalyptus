use std::rc::Rc;

mod eucalyptus;
use eucalyptus::*;

fn main() {
    let test = r#"
let _ = 10
_
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

            let mut acc = 1;
            
            for s in stuff.iter() {
                match s.visit(&symtab, &typetab, &valtab) {
                    Ok(_)  => (),
                    Err(e) => {
                        println!("{}", e);
                        break
                    },
                }

                match s.get_type(&symtab, &typetab, &valtab) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{}", e);
                        break
                    },
                }
                
                match s.eval(&symtab, &valtab) {
                    Ok(v) => if acc == stuff.len() {
                        println!("{:#?}", v)
                    },

                    Err(e) => {
                        println!("{}", e);
                        break
                    },
                }
                acc += 1
            }
        },
    }
}
