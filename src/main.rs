mod emit;
mod ir;
mod lexer;
mod parser;
mod passes;
mod ast;
mod lower;
mod diag;
mod validate;

use diag::{ Diagnostic, render};
use lexer::lex;
use parser::parse;
use lower::lower;
use ir::Circuit;
use validate::validate;

fn run(src: &str) -> Result< Vec<(Circuit, Vec<(usize, usize)>, (usize, usize))>, Diagnostic> {
    let tokens = lex(src).map_err(Diagnostic::from)?;
    let program = parse(tokens).map_err(Diagnostic::from)?;
    let circuits = lower(&program).map_err(Diagnostic::from)?;

    Ok(circuits)
}

fn main() {
    let path = "example.qc";

    let src = r#"circuit bell(2) {
        H q0
        CNOT q0 q1
        measure q0 -> c0
        measure q1 -> c1
    }"#;

    match run(src) {
        Ok(circuits) => {
            for (circuit, spans, circuit_pos) in &circuits {
                let diagnostics = validate(
                    circuit,
                    spans,
                    circuit_pos.0,
                    circuit_pos.1,
                );

                for diagnostic in diagnostics {
                    eprint!("{}", render(&diagnostic, src, path));
                }
            }

            println!("{:#?}", circuits);
        }

        Err(d) => {
            eprint!("{}", render(&d, src, path));
            std::process::exit(1);
        }
    }
}
