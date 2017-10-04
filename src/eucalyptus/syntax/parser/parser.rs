use std::rc::Rc;

use super::*;

pub struct Parser {
    traveler: Traveler,
}

impl Parser {
    pub fn new(traveler: Traveler) -> Parser {
        Parser {
            traveler,
        }
    }

    pub fn parse(&mut self) -> ParserResult<Vec<Statement>> {
        let mut stack = Vec::new();

        while self.traveler.remaining() > 1 {
            self.skip_whitespace()?;
            stack.push(self.statement()?);
        }

        Ok(stack)
    }
    
    pub fn skip_whitespace(&mut self) -> ParserResult<()> {
        while self.traveler.current_content() == "\n" || self.traveler.current().token_type == TokenType::EOL || self.traveler.current().token_type == TokenType::Indent {
            self.traveler.next();
            
            if self.traveler.remaining() < 2 {
                break
            }
        }

        Ok(())
    }
    
    fn expression(&mut self) -> ParserResult<Expression> {
        self.skip_whitespace()?;

        let expr = self.term()?;

        if expr == Expression::EOF {
            return Ok(expr)
        }

        if self.traveler.remaining() > 1 {
            self.skip_whitespace()?;
            if self.traveler.current().token_type == TokenType::Operator {
                return self.operation(expr)
            }
        }

        Ok(expr)
    }
    
    pub fn term(&mut self) -> ParserResult<Expression> {
        self.skip_whitespace()?;

        if self.traveler.remaining() < 2 {
            return Ok(Expression::EOF)
        }

        match self.traveler.current().token_type {
            TokenType::IntLiteral    => {
                let a = Ok(Expression::Number(self.traveler.current_content().parse::<f64>().unwrap()));
                self.traveler.next();
                a
            }

            TokenType::FloatLiteral  => {
                let a = Ok(Expression::Number(self.traveler.current_content().parse::<f64>().unwrap()));
                self.traveler.next();
                a
            }

            TokenType::BoolLiteral   => {
                let a = Ok(Expression::Bool(self.traveler.current_content() == "true"));
                self.traveler.next();
                a
            }

            TokenType::StringLiteral => {
                let a = Ok(Expression::Str(Rc::new(self.traveler.current_content().clone())));
                self.traveler.next();
                a
            }
            
            TokenType::CharLiteral => {
                let a = Ok(Expression::Char(self.traveler.current_content().clone().remove(0)));
                self.traveler.next();
                a
            }

            TokenType::Identifier => {
                let a = Expression::Identifier(Rc::new(self.traveler.current_content().clone()));
                self.traveler.next();

                if self.traveler.remaining() > 1 {
                    match self.traveler.current_content().as_str() {
                        "}" | "]" | "," | ")" => Ok(a),
                        "["                   => self.index(Rc::new(a)),
                        _                     => self.try_call(a),
                    }
                } else {
                    Ok(a)
                }
            }

            TokenType::Symbol => match self.traveler.current_content().as_str() {
                "(" => {
                    self.traveler.next();
                    if self.traveler.current_content() == ")" {
                        return Err(ParserError::new_pos(self.traveler.current().position, &format!("illegal empty clause '()'")))
                    }
                    
                    let a = self.expression()?;

                    self.skip_whitespace()?;
                    self.traveler.expect_content(")")?;
                    self.traveler.next();

                    if self.traveler.current_content() == "[" {
                        self.index(Rc::new(a))
                    } else if self.traveler.remaining() > 1 {
                        self.try_call(a)
                    } else {
                        Ok(a)
                    }
                }
                "{" => self.array(),
                _ => Err(ParserError::new_pos(self.traveler.current().position, &format!("unexpected symbol: {}", self.traveler.current_content()))),
            },

            TokenType::Keyword => match self.traveler.current_content().as_str() {
                "fun" => self.lambda(),
                _     => Err(ParserError::new_pos(self.traveler.current().position, &format!("unexpected keyword: {}", self.traveler.current_content()))),
            },

            _ => Err(ParserError::new_pos(self.traveler.current().position, &format!("unexpected: {}", self.traveler.current_content()))),
        }
    }

    fn index(&mut self, id: Rc<Expression>) -> ParserResult<Expression> {
        self.traveler.next();

        let index = Rc::new(self.expression()?);

        self.traveler.expect_content("]")?;
        self.traveler.next();
        
        Ok(
            Expression::Index(
                Index {
                    id,
                    index,
                }
            )
        )
    }
    
    fn try_call(&mut self, callee: Expression) -> ParserResult<Expression> {
        match self.traveler.current().token_type {
            TokenType::IntLiteral    |
            TokenType::FloatLiteral  |
            TokenType::BoolLiteral   |
            TokenType::StringLiteral |
            TokenType::CharLiteral   |
            TokenType::Identifier => self.call(callee),
            TokenType::Symbol     => match self.traveler.current_content().as_str() {
                "(" | "{"         => self.call(callee),
                _                 => Err(ParserError::new_pos(self.traveler.current().position, &format!("unexpected symbol: {}", self.traveler.current_content()))),
            },
            
            _ => Ok(callee),
        }
    }

    fn array(&mut self) -> ParserResult<Expression> {
        self.traveler.next();
        
        let mut content = Vec::new();
        
        let mut acc = 0;

        while self.traveler.current_content() != "}" {
            if self.traveler.current_content() == "," {
                self.traveler.next();
                content.push(Rc::new(self.expression()?));

            } else if acc == 0 {
                content.push(Rc::new(self.expression()?));

            } else {
                self.traveler.prev();
                if self.traveler.current_content() != "," {
                    self.traveler.next();
                }
                break
            }

            acc += 1
        }

        self.traveler.next();

        if self.traveler.current_content() == "[" {
            self.index(Rc::new(Expression::Array(content)))
        } else {
            Ok(Expression::Array(content))
        }
    }
    
    fn lambda(&mut self) -> ParserResult<Expression> {
        self.traveler.next();

        let mut params = Vec::new();

        while self.traveler.current_content() != "->" {         
            let param = Rc::new(self.traveler.expect(TokenType::Identifier)?);
            self.traveler.next();
            
            params.push(param);
            
            if self.traveler.current().token_type == TokenType::EOL || self.traveler.current().token_type == TokenType::EOF {
                return Err(ParserError::new_pos(self.traveler.current().position, "expected '=' found eol/eof"))
            }
        }
        
        self.traveler.next();
        
        let body;
        
        if self.traveler.current_content() == "\n" {
            body = Rc::new(self.block()?)
        } else {
            body = Rc::new(self.expression()?)
        }

        Ok(
            Expression::Lambda(
                Lambda {
                    params,
                    body,
                }
            )
        )
    }

    fn call(&mut self, caller: Expression) -> ParserResult<Expression> {
        let mut args = Vec::new();

        let mut acc = 0;

        while self.traveler.current_content() != "\n" {
            if self.traveler.current_content() == "," {
                self.traveler.next();
                
                let expr = Rc::new(self.expression()?);

                if *expr == Expression::EOF {
                    break
                }

                args.push(expr);

            } else if acc == 0 {
                let expr = Rc::new(self.expression()?);
                
                if *expr == Expression::EOF {
                    break
                }

                args.push(expr);

            } else {
                self.traveler.prev();
                if self.traveler.current_content() != "!" || self.traveler.current_content() != "," {
                    self.traveler.next();
                }
                break
            }
            
            acc += 1
        }
        
        Ok(
            Expression::Call(
                Call {
                    callee: Rc::new(caller),
                    args,
                }
            )
        )
    }

    fn block(&mut self) -> ParserResult<Expression> {
        let mut stack = Vec::new();
        loop {
            if self.traveler.current().token_type == TokenType::Indent {
                self.traveler.next();
                if self.traveler.current_content() == "\n" {
                    self.traveler.next();
                    break
                }
            } else if self.traveler.current_content() == "\n" {
                stack.push(self.traveler.current().clone());
                self.traveler.next();

                if self.traveler.current().token_type == TokenType::Indent {
                    self.traveler.next();
                } else {
                    break
                }
            }
            
            if self.traveler.remaining() < 2 {
                break
            }

            stack.push(self.traveler.current().clone());
            self.traveler.next();
        }        

        let mut parser = Parser::new(Traveler::new(stack));

        match parser.parse() {
            Ok(s)    => Ok(Expression::Block(s)),
            Err(why) => Err(ParserError::new(&format!("{}", why))),
        }
    }

    fn binding(&mut self) -> ParserResult<Statement> {
        self.traveler.next();
        
        let left = Rc::new(self.traveler.expect(TokenType::Identifier)?);
        self.traveler.next();
        
        if self.traveler.current().token_type == TokenType::Identifier {
            let mut params = Vec::new();
            while self.traveler.current_content() != "=" {
                let param = Rc::new(self.traveler.expect(TokenType::Identifier)?);
                self.traveler.next();
                
                params.push(param);
                
                if self.traveler.current().token_type == TokenType::EOL || self.traveler.current().token_type == TokenType::EOF {
                    return Err(ParserError::new_pos(self.traveler.current().position, "expected '=' found eol/eof"))
                }
            }

            self.traveler.next();
            
            let body;

            match self.traveler.current().token_type {
                TokenType::EOL => {
                    self.traveler.next();

                    body = Rc::new(self.block()?)
                },
                _ => body = Rc::new(self.expression()?)
            }
            
            Ok(
                Statement::Function(
                    Function {
                        name: left,
                        params,
                        body,
                    }
                )
            )
        } else {
            self.skip_whitespace()?;
            self.traveler.expect_content("=")?;
            self.traveler.next();

            let right = Rc::new(self.expression()?);
            let left  = Rc::new(Expression::Identifier(left));
            
            Ok(
                Statement::Binding(
                    Binding {
                        left,
                        right,
                    }
                )
            )
        }
    }

    fn assignment(&mut self, left: Rc<Expression>) -> ParserResult<Statement> {
        self.traveler.next();

        let right = Rc::new(self.expression()?);

        Ok(
            Statement::Assignment(
                Assignment {
                    left,
                    right,
                }
            )
        )
    }

    fn statement(&mut self) -> ParserResult<Statement> {
        self.skip_whitespace()?;
        match self.traveler.current().token_type {
            TokenType::Symbol => match self.traveler.current_content().as_str() {
                "\n" => {
                    self.traveler.next();
                    self.statement()
                },
                _ => Ok(Statement::Expression(Rc::new(self.expression()?))),
            },
            
            TokenType::Identifier => {
                let a = Expression::Identifier(Rc::new(self.traveler.current_content().clone()));
                self.traveler.next();
                
                if self.traveler.current_content() == "=" {
                    self.assignment(Rc::new(a))
                } else {
                    self.traveler.prev();
                    Ok(Statement::Expression(Rc::new(self.expression()?)))
                }
            },
            
            TokenType::Keyword => match self.traveler.current_content().as_str() {
                "let" => Ok(self.binding()?),
                _     => Ok(Statement::Expression(Rc::new(self.expression()?))),
            },
            
            _ => Ok(Statement::Expression(Rc::new(self.expression()?))),
        }
    }
    
    fn operation(&mut self, expression: Expression) -> ParserResult<Expression> {
        let mut ex_stack = vec![expression];
        let mut op_stack: Vec<(Operand, u8)> = Vec::new();
        
        op_stack.push(Operand::from_str(&self.traveler.current_content()).unwrap());
        self.traveler.next();

        if self.traveler.current_content() == "\n" {
            self.traveler.next();
        }
        
        let term = self.term()?;

        ex_stack.push(term);
        
        let mut done = false;
        
        while ex_stack.len() > 1 {
            if !done {
                if self.traveler.current().token_type != TokenType::Operator {
                    done = true;
                    continue
                }
                
                let (op, precedence) = Operand::from_str(&self.traveler.current_content()).unwrap();
                self.traveler.next();

                if precedence >= op_stack.last().unwrap().1 {
                    let left  = ex_stack.pop().unwrap();
                    let right = ex_stack.pop().unwrap();

                    ex_stack.push(
                        Expression::Operation(
                            Operation {
                                right: Rc::new(left),
                                op:    op_stack.pop().unwrap().0,
                                left:  Rc::new(right)
                            }
                        )
                    );

                    let term = self.term()?;

                    ex_stack.push(term);
                    op_stack.push((op, precedence));

                    continue
                }

                let term = self.term()?;

                ex_stack.push(term);
                op_stack.push((op, precedence));
            }

            let left  = ex_stack.pop().unwrap();
            let right = ex_stack.pop().unwrap();

            ex_stack.push(
                Expression::Operation(
                    Operation {
                        right: Rc::new(left),
                        op:    op_stack.pop().unwrap().0,
                        left:  Rc::new(right)
                    }
                )
            );
        }
                
        Ok(ex_stack.pop().unwrap())
    }
}
