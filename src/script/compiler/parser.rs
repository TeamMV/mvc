use super::lexer::*;
use super::ast::*;
use std::error::Error;
use std::fmt::Display;

pub struct Parser {
    lexer: Lexer,
    program: Program,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl From<&str> for ParseError {
    fn from(s: &str) -> Self {
        ParseError {
            message: s.to_string(),
        }
    }
}

impl From<String> for ParseError {
    fn from(s: String) -> Self {
        ParseError {
            message: s,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for ParseError {}

impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        Parser {
            lexer,
            program: Program::new(),
        }
    }

    pub fn parse(mut self) -> Result<Program, Program> {
        while let Some(token) = self.lexer.next() {
            let element = self.parse_element(token);
            if element.is_err() {
                return Ok(self.program);
            }
            self.program.push(element.unwrap());
        }
        Ok(self.program)
    }

    pub fn parse_element(&mut self, token: Token) -> Result<Element, ParseError> {
        if let Token::Keyword(keyword) = token {
            match keyword {
                Keyword::Include => {
                    let token = self.lexer.next_token();
                    return if let Token::Identifier(name) = token {
                        let token = self.lexer.next_token();
                        if let Token::Semicolon = token {
                            Ok(Element::Statement(TopLevelStatement::Include(name)))
                        }
                        else {
                            Err(format!("Include: Unexpected token, expected ';', got {}", token).into())
                        }
                    }
                    else {
                        Err(format!("Include: Unexpected token, expected identifier, got {}", token).into())
                    };
                },
                Keyword::Use => Ok(Element::Statement(TopLevelStatement::Use(self.parse_use()?))),
                Keyword::Const => {
                    let declaration = self.parse_declaration()?;
                    if declaration.value.is_none() {
                        Err("Const: Must have a value".into())
                    }
                    else {
                        //do stuff later
                        Ok(Element::Empty)
                    }
                },
                Keyword::Let => Ok(Element::Statement(TopLevelStatement::Declaration(self.parse_declaration()?))),
                Keyword::Fn => Ok(Element::Function(self.parse_fn()?)),
                _ => Err(format!("File: Unexpected keyword, expected 'include' | 'use' | 'const' | 'let' | 'fn', found {}", keyword).into())
            }
        }
        else {
            Err(format!("File: Unexpected token, expected Keyword, found {}", token).into())
        }
    }

    pub fn parse_use(&mut self) -> Result<Vec<String>, ParseError> {
        let mut res = Vec::new();
        let token = self.lexer.next_token();
        if let Token::Identifier(usage) = token {
            res.push(usage);
        }
        else {
            return Err(format!("Use: Unexpected token, expected Identifier, found {}", token).into());
        }
        while let Some(token) = self.lexer.next() {
            match token {
                Token::Semicolon => break,
                Token::Comma => {},
                _ => {
                    return Err(format!("Use: Unexpected token, expected ';' or ',', found {}", token).into())
                }
            }
        }
        let token = self.lexer.next_token();
        if let Token::Identifier(usage) = token {
            res.push(usage);
        }
        else {
            return Err(format!("Use: Unexpected token, expected Identifier, found {}", token).into());
        }
        Ok(res)
    }

    pub fn parse_declaration(&mut self) -> Result<Declaration, ParseError> {
        let token = self.lexer.next_token();
        if let Token::Identifier(name) = token {
            let mut ty = None;
            let mut token = self.lexer.next_token();
            if let Token::Colon = token {
                token = self.lexer.next_token();
                if let Token::Keyword(word) = token {
                    ty = Some(Type::try_from(word)?);
                }
                else {
                    return Err(format!("Let/Const: Unexpected token, expected Type, found {}", token).into());
                }
                token = self.lexer.next_token();
            }
            match token {
                Token::Operator(Operator::Assign) => {
                    let value = self.parse_expression()?;
                    if let Some(ty) = ty {
                        Ok(Declaration {
                            name,
                            ty,
                            value: Some(value),
                        })
                    }
                    else {
                        let ty = value.infer_type();
                        if let Some(ty) = ty {
                            Ok(Declaration {
                                name,
                                ty,
                                value: Some(value),
                            })
                        }
                        else {
                            Err(format!("Let/Const: Cannot infer type for variable {}, please add a type annotation", name).into())
                        }
                    }
                }
                Token::Semicolon => {
                    if let Some(ty) = ty {
                        Ok(Declaration {
                            name,
                            ty,
                            value: None
                        })
                    }
                    else {
                        Err("Let/Const: Variable without initial value must be given a type annotation".into())
                    }
                }
                _ => Err(format!("Let/Const: Unexpected token, expected '=' or ';', found {}", token).into())
            }
        }
        else {
            Err(format!("Let/Const: Unexpected token, expected Identifier, found {}", token).into())
        }
    }

    pub fn parse_fn(&mut self) -> Result<Function, ParseError> {
        let token = self.lexer.next_token();
        if let Token::Identifier(name) = token {
            let token = self.lexer.next_token();
            if token != Token::LParen {
                return Err(format!("Fn: Unexpected token, expected '(', found {}", token).into());
            }
            let mut parameters = Vec::new();
            let mut token = self.lexer.next_token();
            while token != Token::RParen {
                if token == Token::Comma {
                    token = self.lexer.next_token();
                }
                if let Token::Identifier(name) = token {
                    let token = self.lexer.next_token();
                    if token != Token::Colon {
                        return Err(format!("Fn: Unexpected token, expected ':', found {}", token).into());
                    }
                    let token = self.lexer.next_token();
                    if let Token::Keyword(word) = token {
                        let ty = Type::try_from(word)?;
                        parameters.push((name, ty));
                    }
                    else {
                        return Err(format!("Let/Const: Unexpected token, expected Type, found {}", token).into());
                    }
                }
                else {
                    return Err(format!("Fn: Unexpected token, expected Identifier, found {}", token).into());
                }
                token = self.lexer.next_token();
            }
            let mut token = self.lexer.next_token();
            let mut ty = Type::Void;
            if let Token::Arrow = token {
                token = self.lexer.next_token();
                if let Token::Keyword(word) = token {
                    ty = Type::try_from(word)?;
                }
                else if let Token::LParen = token {
                    token = self.lexer.next_token();
                    if token != Token::RParen {
                        return Err(format!("Fn: Unexpected token, tuples are not supported, expected ')', found {}", token).into());
                    }
                }
                else {
                    return Err(format!("Fn: Unexpected token, expected Type, found {}", token).into());
                }
                token = self.lexer.next_token();
            }
            if token != Token::LCurly {
                return Err(format!("Fn: Unexpected token, expected '{{', found {}", token).into());
            }
            let mut body = Vec::new();
            token = self.lexer.next_token();
            while token != Token::RCurly {
                body.push(self.parse_statement(token)?);
                token = self.lexer.next_token();
            }
            Ok(Function {
                name,
                parameters,
                return_type: ty,
                body: Block {
                    statements: body
                },
            })
        }
        else {
            return Err(format!("Fn: Unexpected token, expected Identifier, found {}", token).into());
        }
    }

    pub fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        Err("".into())
    }

    pub fn parse_statement(&mut self, token: Token) -> Result<Statement, ParseError> {
        Err("".into())
    }
}