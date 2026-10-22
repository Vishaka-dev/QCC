mod emit;
mod ir;
mod lexer;
mod parser;
mod passes;
mod ast;
mod lower;
mod diag;

use diag::{ Diagnostic, render};
use lexer::lex;
use parser::parse;
use lower::lower;
use ir::Circuit;

fn run(src: &str) -> Result< Vec<Circuit>, Diagnostic> {
    let tokens = lex(src).map_err(Diagnostic::from)?;
    let program = parse(tokens).map_err(Diagnostic::from)?;
    let circuit = lower(&program).map_err(Diagnostic::from)?;

    Ok(circuit)
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
            println!("{:#?}", circuits);
        }

        Err(d) => {
            eprint!("{}", render(&d, src, path));
            std::process::exit(1);
        }
    }
}
