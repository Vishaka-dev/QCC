use crate::ast::Expr;
use crate::lexer::{ Spanned, Token } ;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

pub struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned>) -> Self {
        Self {
            tokens,
            pos: 0,
        }
    }
    fn peek(&self) -> &Spanned {
        &self.tokens[self.pos]
    }
    fn advance(&mut self) {
        self.pos += 1;
    }
    fn expect(&mut self, expected: Token, description: &str) -> Result<(), ParseError> {
        let actual = self.peek().token.clone();

        if actual == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("expected: {}", description),
                line: self.peek().line,
                col: self.peek().col,
            })
        }
    }
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let tok = self.peek().token.clone();

        match tok {
            Token::Int(n) => {
                self.advance();
                Ok(Expr::Number(n as f64))
            }
            Token::Float(x) => {
                self.advance();
                Ok(Expr::Number(x))
            }
            Token::Ident(name) => {
                self.advance();
                
                if name == "pi" {
                    Ok(Expr::Pi)
                } else {
                    Err(ParseError {
                        message: format!("unknown constant {}", name),
                        line: self.peek().line,
                        col: self.peek().col,
                    })
                }
            }
            Token::LParen => {
                self.advance();

                let expr = self.parse_expr()?;

                self.expect(Token::RParen, "')'")?;
                Ok(expr)
            }
            _ => Err(ParseError {
                message: "Ecpected a number, pi or (".to_string(),
                line: self.peek().line,
                col: self.peek().col,
            })
        }
    }
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_primary()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::lex;

    #[test]
    fn parse_integer_primary() {
        let tokens = lex("42").unwrap();
        let mut parser = Parser::new(tokens);

        let expr = parser.parse_primary().unwrap();

        assert_eq!(expr, Expr::Number(42.0));
    }

    #[test]
    fn parse_pi_primary() {
        let tokens = lex("pi").unwrap();
        let mut parser = Parser::new(tokens);

        let expr = parser.parse_primary().unwrap();

        assert_eq!(expr, Expr::Pi);
    }
}