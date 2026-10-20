use crate::ast::{ BinOp, Expr };
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
    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.peek().token == Token::Minus {
            self.advance();

            let expr = self.parse_unary()?;

            Ok(Expr::Neg(Box::new(expr)))
        } else {
            self.parse_primary()
        }
    }
    fn parse_term(&mut self) ->Result<Expr, ParseError> {
        let mut lhs = self.parse_unary()?;

        loop {
            let op = match self.peek().token {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => break,
            };
            self.advance();

            let rhs = self.parse_unary()?;

            lhs = Expr::Binary {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            };
        }
        Ok(lhs)
    }
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_term()?;

        loop {
            let op = match self.peek().token {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();

            let rhs = self.parse_term()?;

            lhs = Expr::Binary {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            };
        }
        Ok(lhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::lex;

    fn parse_str(src: &str) -> Result<Expr, ParseError> {
        let tokens = crate::lexer::lex(src).unwrap();

        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expr()?;

        parser.expect(Token::Eof, "end of expresion")?;
        Ok(expr)
    }

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
    #[test]
    fn parse_negative_number() {
        let tokens = lex("-3").unwrap();
        let mut parser = Parser::new(tokens);

        let expr = parser.parse_expr().unwrap();

        assert_eq!(
            expr,
            Expr::Neg(Box::new(Expr::Number(3.0)))
        );
    }
    fn mul_has_higher_prsident_than_addition() {
        let expr = parse_str("1+2*3").unwrap();

        assert_eq!(
            expr,
            Expr::Binary {
                left: Box::new(Expr::Number(1.0)),
                op: BinOp::Add,
                right: Box::new(
                    Expr::Binary {
                        left: Box::new(Expr::Number(2.0)),
                        op: BinOp::Mul,
                        right: Box::new(Expr::Number(3.0)),
                    }
                )
            }
        );
    }
}