#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // keywords
    Circuit,
    Measure,
    Barrier,
    // names and numbers
    Ident(String),
    Int(usize),
    Float(f64),
    // puncuations
    LBrace,
    RBrace,
    LParen,
    RParen,
    Comma,
    Arrow, // ->
    // operators
    Plus,
    Minus,
    Star,
    Hash,
    // structure
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Lexer {
            chars: src.chars().peekable(),
            line: 1,
            col: 1,
        }
    }

    // look at the next chrater without consuming it
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    // consume one char and keep the line/col updated
    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

pub fn lex(src: &str) -> Result<Vec<Spanned>, LexError> {
    let mut lx = Lexer::new(src);
    let mut tokens = Vec::new();

    // -------------------------------------------main lexing loop -------------------------------------------------------
    loop {
        let c = match lx.peek() {
            Some(c) => c,
            None => break,
        };
        if c == ' ' || c == '\t' || c == '\r' {
            lx.bump();
            continue;
        }
        let (line, col) = (lx.line, lx.col);

        let token = match c {
            '{' => Some(Token::LBrace),
            '}' => Some(Token::RBrace),
            '(' => Some(Token::LParen),
            ')' => Some(Token::RParen),
            ',' => Some(Token::Comma),
            '+' => Some(Token::Plus),
            '-' => {
                lx.bump();
                let token = if lx.peek() == Some('>') {
                    lx.bump();
                    Token::Arrow
                } else {
                    Token::Minus
                };
                tokens.push(Spanned { token, line, col });
                continue;
            }
            '*' => Some(Token::Star),
            '#' => {
                lx.bump();

                // consume everything untill the newline
                while let Some(ch) = lx.peek() {
                    if ch == '\n' {
                        break;
                    }
                    lx.bump();
                }
                continue;
            }
            '\n' => Some(Token::Newline),
            _ => None,
        };

        if let Some(token) = token {
            lx.bump();
            tokens.push(Spanned { token, line, col });
            continue;
        };

        // -------------------- Identifier token catcher -----------------
        if c.is_alphabetic() || c == '_' {
            let mut ident = String::new();

            while let Some(ch) = lx.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    ident.push(ch);
                    lx.bump();
                } else {
                    break;
                }
            }
            let token = match ident.as_str() {
                "circuit" => Token::Circuit,
                "measure" => Token::Measure,
                "barrier" => Token::Barrier,
                _ => Token::Ident(ident),
            };
            tokens.push(Spanned { token, line, col });
            continue;
        };

        // ---------------------Int and Float token catcher---------------------
        if c.is_ascii_digit() {
            let mut number = String::new();
            let mut is_float = false;

            // consume the integer part
            while let Some(ch) = lx.peek() {
                if ch.is_ascii_digit() {
                    number.push(ch);
                    lx.bump();
                } else {
                    break;
                }
            }

            // cheak for decimal part
            if lx.peek() == Some('.') {
                is_float = true;
                number.push('.');
                lx.bump();

                match lx.peek() {
                    Some(ch) if ch.is_ascii_digit() => {}
                    _ => {
                        return Err(LexError {
                            message: format!(
                                "Expected digit after decimal point in {} at line: {}, collumn: {}.",
                                number, line, col
                            ),
                            line,
                            col,
                        });
                    }
                }

                // consume the decimal digits after the '.'
                while let Some(ch) = lx.peek() {
                    if ch.is_ascii_digit() {
                        number.push(ch);
                        lx.bump();
                    } else {
                        break;
                    }
                }
            }

            // parse the number
            let token = if is_float {
                match number.parse::<f64>() {
                    Ok(value) => Token::Float(value),
                    Err(_) => {
                        return Err(LexError {
                            message: format!(
                                "Invalid float litteral: {} in line: {}, collumn: {}.",
                                number, line, col
                            ),
                            line,
                            col,
                        });
                    }
                }
            } else {
                match number.parse::<usize>() {
                    Ok(value) => Token::Int(value),
                    Err(_) => {
                        return Err(LexError {
                            message: format!(
                                "Invalid integer litteral: {} in line: {}, collumn: {}.",
                                number, line, col
                            ),
                            line,
                            col,
                        });
                    }
                }
            };

            tokens.push(Spanned { token, line, col });
            continue;
        }

        return Err(LexError {
            message: format!(
                "Unexpected character: '{}' at line: {}, collumn: {}",
                c, line, col
            ),
            line,
            col,
        });
    }
    //------------------------------------------------------------------------------------------------------------------------------

    tokens.push(Spanned {
        token: Token::Eof,
        line: lx.line,
        col: lx.col,
    });
    Ok(tokens)
}

// --------------------------------Tests for the lexer----------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn skip_whitespace() {
        let src = "    \n\t  \r\n";
        let tokens = lex(src).unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Newline);
        assert_eq!(tokens[1].token, Token::Newline);
        assert_eq!(tokens[2].token, Token::Eof);
    }

    #[test]
    fn single_char_position() {
        let src = "{}";
        let toks = lex(src).unwrap();
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[0].token, Token::LBrace);
        assert_eq!(toks[0].line, 1);
        assert_eq!(toks[0].col, 1);
        assert_eq!(toks[1].token, Token::RBrace);
        assert_eq!(toks[1].line, 1);
        assert_eq!(toks[1].col, 2);
        assert_eq!(toks[2].token, Token::Eof);
        assert_eq!(toks[2].line, 1);
        assert_eq!(toks[2].col, 3);
    }

    #[test]
    fn identifiers() {
        let src = "circuit CNOT q0";
        let toks = lex(src).unwrap();

        assert_eq!(toks.len(), 4);
        assert_eq!(toks[0].token, Token::Circuit);
        assert_eq!(toks[1].token, Token::Ident("CNOT".to_string()));
        assert_eq!(toks[2].token, Token::Ident("q0".to_string()));
        assert_eq!(toks[1].col, 9);
    }

    #[test]
    fn numbers() {
        let toks = lex("2 3.14 ").unwrap();

        assert_eq!(toks[0].token, Token::Int(2));
        assert_eq!(toks[1].token, Token::Float(3.14));
    }

    #[test]
    fn rejects_float_without_decimal_digits() {
        assert!(lex("3.").is_err());
        assert!(lex("3.a").is_err());

        assert!(lex("3.0").is_ok());
        assert!(lex("3.14").is_ok());
    }

    #[test]
    fn minus_and_arrow() {
        let toks = lex("- ->").unwrap();

        assert_eq!(toks[0].token, Token::Minus);
        assert_eq!(toks[1].token, Token::Arrow);
    }
    #[test]
    fn comments() {
        let src = "measure q0 # this is a comment\nbarrier";

        let toks = lex(src).unwrap();

        assert_eq!(toks[0].token, Token::Measure);
        assert_eq!(toks[1].token, Token::Ident("q0".to_string()));
        assert_eq!(toks[2].token, Token::Newline);
        assert_eq!(toks[3].token, Token::Barrier);
        assert_eq!(toks[4].token, Token::Eof);
    }

    #[test]
    fn bad_char_errors_with_position() {
        let err = lex("H q0\n  @").unwrap_err();
        assert_eq!(err.line, 2);
        assert_eq!(err.col, 3);
    }
    
    #[test]
    fn lexes_bell_circuit() {
        let src = "circuit bell(2) {\n    H q0\n    CNOT q0 q1\n    measure q0 -> c0\n}\n";
        let toks = lex(src).unwrap();
        assert_eq!(toks.first().unwrap().token, Token::Circuit);
        assert_eq!(toks.last().unwrap().token, Token::Eof);
    }
}
