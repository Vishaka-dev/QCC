use crate::ast::{ BinOp, Expr, Ref};
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
    fn skip_newline(&mut self) {
        while self.peek().token == Token::Newline {
            self.advance();
        }
    }
    fn parse_ref(&mut self, prefix: char, what: &str) -> Result<crate::ast::Ref, ParseError> {
        let line = self.peek().line;
        let col = self.peek().col;

        let token = self.peek().token.clone();

        let name = match token {
            Token::Ident(name) => name,
            _ => {
                return Err(ParseError{
                    message: format!("expected a {}", what),
                    line,
                    col,
                })
            },
        };
        let digits = match name.strip_prefix(prefix) {
            Some(digits) => digits,
            None => {
                return Err(ParseError{
                    message: format!("expected a {} like {}0, found {}", what , prefix, name),
                    line,
                    col,
                });
            },
        };
        let index = match digits.parse::<usize>() {
            Ok(index) => index,
            Err(_) =>{
                return Err(ParseError {
                    message: format!("expected a {} like {}0, found '{}'", what, prefix, name),
                    line,
                    col,
                });
            },
        };
        self.advance();

        Ok(crate::ast::Ref {
            index,
            line,
            col,
        })  
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
                
                if name == "pi" {
                    self.advance();
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
                message: "Ecpected a number, 'pi' or '('".to_string(),
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

        parser.expect(Token::Eof, "end of expression")?;
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
    #[test]
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
    #[test]
    fn primary_number() {
        assert_eq!(parse_str("3").unwrap(), Expr::Number(3.0));
        assert_eq!(parse_str("2.5").unwrap(), Expr::Number(2.5));
    }

    #[test]
    fn primary_pi() {
        assert_eq!(parse_str("pi").unwrap(), Expr::Pi);
    }

    #[test]
    fn primary_parens() {
        assert_eq!(parse_str("(4)").unwrap(), Expr::Number(4.0));
    }

    #[test]
    fn primary_errors() {
        assert!(parse_str("foo").is_err());
        assert!(parse_str("(").is_err());
        assert!(parse_str("(3").is_err());
        assert!(parse_str("3 4").is_err());
    }
    #[test]
    fn parse_qubit_refs() {
        let tokens = lex("q0 q12").unwrap();
        let mut parser = Parser::new(tokens);

        let q0 = parser.parse_ref('q', "qubit").unwrap();
        let q12 = parser.parse_ref('q', "qubit").unwrap();

        assert_eq!(q0.index, 0);
        assert_eq!(q12.index, 12);
    }
    #[test]
    fn parse_qubit_ref_errors() {
        for source in ["x0", "q", "qa", "5"] {
            let tokens = lex(source).unwrap();
            let mut parser = Parser::new(tokens);

            assert!(
                parser.parse_ref('q', "qubit").is_err(),
                "{} should be rejected",
                source
            );
        }
    }
    #[test]
    fn parse_classical_refs() {
        let tokens = lex("c0 c12").unwrap();
        let mut parser = Parser::new(tokens);

        let c0 = parser.parse_ref('c', "classical bit").unwrap();
        let c12 = parser.parse_ref('c', "classical bit").unwrap();

        assert_eq!(c0.index, 0);
        assert_eq!(c12.index, 12);
    }
}