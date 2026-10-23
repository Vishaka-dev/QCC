use crate::ast::*;
use crate::lexer::{ Spanned, Token } ;
use crate::ir::{Circuit, Instruction};

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

pub fn optimize(circuit: &mut Circuit) {
    let mut changed = true;

    while changed {
        changed = false;

        let mut i = 0;

        while i + 1 < circuit.instructions.len() {
            // transformation here.
            i += 1;
        }
    }
}

fn touches_any(instruction: &Instruction, qubits: &[usize]) -> bool {
    instruction
        .qubits()
        .iter()
        .any(|qubit| qubits.contains(qubit))
}

impl Parser {
    pub fn new(tokens: Vec<Spanned>) -> Self {
        Self {
            tokens,
            pos: 0,
        }
    }
    fn skip_newlines(&mut self) {
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
    fn parse_gate(&mut self) -> Result<Stmt, ParseError> {
        let line = self.peek().line;
        let col = self.peek().col;

        let name = match self.peek().token.clone() {
            Token::Ident(name) => {
                self.advance();
                name
            },
            _ => {
                return Err(ParseError {
                    message: "expected gate name".to_string(),
                    line,
                    col,
                });
            },
        };
        let angle = if self.peek().token == Token::LParen {
            self.advance();

            let expr = self.parse_expr()?;

            self.expect(Token::RParen, "')'")?;

            Some(expr)
        } else {
            None
        };
        let first_qubit = self.parse_ref('q', "qubit")?;
        let mut qubits = vec![first_qubit];

        while matches!(self.peek().token, Token::Ident(_)) {
            qubits.push(self.parse_ref('q', "qubit")?);
        }
        Ok(Stmt {
            kind: StmtKind::Gate {
                name,
                angle,
                qubits,
            },
            line,
            col,
        })
    }
    fn parse_measure(&mut self) -> Result<Stmt, ParseError> {
        let line = self.peek().line;
        let col = self.peek().col;

        self.expect(Token::Measure, "'measure'")?;

        let qubit = self.parse_ref('q', "'qubit'")?;

        self.expect(Token::Arrow, "->")?;

        let clbit = self.parse_ref('c', "'classical bit'")?;

        Ok (Stmt {
            kind: StmtKind::Measure {
                qubit,
                clbit,
            },
            line,
            col,
        })
    }
    fn parse_barrier(&mut self) -> Result<Stmt, ParseError> {
        let line = self.peek().line;
        let col = self.peek().col;

        self.expect(Token::Barrier, "'barrier'");

        let mut qubits = Vec::new();

        while matches!(self.peek().token, Token::Ident(_)) {
            qubits.push(self.parse_ref('q', "quibit")?);
        }
        Ok (Stmt {
            kind: StmtKind::Barrier { qubits },
            line,
            col,
        })
    }
    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek().token {
            Token::Measure => self.parse_measure(),
            Token::Barrier => self.parse_barrier(),
            Token::Ident(_) => self.parse_gate(),
            _ => Err(ParseError {
                message: "expected a gate, measure, or barrier".to_string(),
                line: self.peek().line,
                col: self.peek().col,
            }),
        }
    }
    fn parse_circuit(&mut self) -> Result<CircuitDef, ParseError> {
        let line = self.peek().line;
        let col = self.peek().col;

        self.expect(Token::Circuit, "'circuit'")?;

        let name = match self.peek().token.clone() {
            Token::Ident(name) => {
                self.advance();
                name
            },
            _ => {
                return Err(ParseError {
                    message: "expected circuit name".to_string(),
                    line: self.peek().line,
                    col: self.peek().col,
                })
            },
        };

        self.expect(Token::LParen, "'('")?;

        let qubit_count = match self.peek().token.clone() {
            Token::Int(count) => {
                if count == 0 {
                    return Err(ParseError {
                        message: "qubit count must be at least 1".to_string(),
                        line: self.peek().line,
                        col: self.peek().col,
                    });
                }
                self.advance();
                count
            }
            _ => {
                return Err(ParseError {
                    message: "qubit count must be a whole number".to_string(),
                    line: self.peek().line,
                    col: self.peek().col,
                });
            }
        };

        let clbit_count = if self.peek().token == Token::Comma {
            self.advance();
            
            match self.peek().token.clone() {
                Token::Int(count) => {
                    self.advance();
                    Some(count)
                }
                _ => {
                    return Err(ParseError {
                        message: "classical bit count must be a whole number".to_string(),
                        line: self.peek().line,
                        col: self.peek().col,
                    });
                }
            }
        } else {
            None
        };

        self.expect(Token::RParen, "')'")?;
        self.expect(Token::LBrace, "'{'")?;
        
        self.skip_newlines();

        let mut body = Vec::new();

        loop {
            self.skip_newlines();
            
            match self.peek().token {
                Token::RBrace => {
                    self.advance();
                    break;
                },
                Token::Eof => {
                    return Err(ParseError {
                        message: "unclosed circuit, missing '}'".to_string(),
                        line: self.peek().line,
                        col: self.peek().col,
                    });
                },
                Token::Circuit => {
                    return Err(ParseError {
                        message: "unclosed circuit, missing '}' before next circuit".to_string(),
                        line: self.peek().line,
                        col: self.peek().col,
                    });
                },
                _ => {
                    body.push(self.parse_statement()?);
                }
            }
        }
        Ok(CircuitDef {
            name,
            qubit_count,
            clbit_count,
            body,
            line,
            col,
        })
    }
    fn parse_program(&mut self) -> Result<Program, ParseError> {
        self.skip_newlines();

        let mut circuits = Vec::new();

        while self.peek().token != Token::Eof {
            circuits.push(self.parse_circuit()?);
            self.skip_newlines();
        }
        Ok(Program { circuits })
    }
}

pub fn parse(tokens: Vec<Spanned>) -> Result<Program, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::ir::Gate;

        #[test]
        fn instruction_on_different_qubit_does_not_block() {
            let instruction = Instruction::Gate(Gate::X { qubit: 1 });

            assert!(!touches_any(&instruction, &[0]));
        }

        #[test]
        fn instruction_on_same_qubit_blocks() {
            let instruction = Instruction::Gate(Gate::X { qubit: 0 });

            assert!(touches_any(&instruction, &[0]));
        }

        #[test]
        fn two_qubit_instruction_blocks_if_it_touches_target() {
            let instruction = Instruction::Gate(Gate::CNOT {
                control: 0,
                target: 1,
            });

            assert!(touches_any(&instruction, &[0]));
            assert!(touches_any(&instruction, &[1]));
        }
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
    #[test]
    fn parse_simple_gate() {
        let tokens = lex("H q0").unwrap();
        let mut parser = Parser::new(tokens);

        let stmt = parser.parse_gate().unwrap();

        match stmt.kind {
            StmtKind::Gate {
                name,
                angle,
                qubits,
            } => {
                assert_eq!(name, "H");
                assert_eq!(angle, None);
                assert_eq!(qubits.len(), 1);
                assert_eq!(qubits[0].index, 0);
            }
            _ => panic!("expected gate"),
        }
    }
    #[test]
    fn gate_requires_qubit() {
        let tokens = lex("H").unwrap();
        let mut parser = Parser::new(tokens);

        assert!(parser.parse_gate().is_err());
    }
    #[test]
    fn error_float_qubit_count() {
        let tokens = lex("circuit bell(2.5) {}").unwrap();

        let err = parse(tokens).unwrap_err();

        assert_eq!(err.message, "qubit count must be a whole number");
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 14);
    }
    #[test]
    fn error_gate_without_qubit() {
        let tokens = lex("circuit bell(2) { H }").unwrap();

        let err = parse(tokens).unwrap_err();

        assert_eq!(err.message, "expected a qubit");
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 21);
    }
    #[test]
    fn error_invalid_extra_identifier_after_qubits() {
        let tokens = lex("circuit bell(2) { H q0 q1 foo }").unwrap();

        let err = parse(tokens).unwrap_err();

        assert_eq!(
            err.message,
            "expected a qubit like q0, found foo"
        );
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 27);
    }
    #[test]
    fn parse_multiple_circuits_with_comments_and_blank_lines() {
        let source = r#"
    # First circuit

    circuit first(1) {
        H q0
    }

    # Between circuits

    circuit second(2) {

        CNOT q0 q1
    }
    "#;

        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.circuits.len(), 2);

        assert_eq!(program.circuits[0].name, "first");
        assert_eq!(program.circuits[0].qubit_count, 1);
        assert_eq!(program.circuits[0].body.len(), 1);

        assert_eq!(program.circuits[1].name, "second");
        assert_eq!(program.circuits[1].qubit_count, 2);
        assert_eq!(program.circuits[1].body.len(), 1);
    }
}