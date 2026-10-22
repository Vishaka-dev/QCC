use crate::diag::{Diagnostic, Severity};
use crate::ir::{Circuit, Instruction};
use std::collections::HashSet;

pub fn validate(circuit: &Circuit, spans: &[(usize, usize)]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut measured = HashSet::new();

    for (index, instruction) in circuit.instructions.iter().enumerate() {
        let (line, col) = spans[index];

        match instruction {
            Instruction::Measure { qubit, ..} => {
                measured.insert(*qubit);
            }
            Instruction::Gate(gate) => {
                for qubit in gate.qubits() {
                    if measured.contains(&qubit) {
                        diagnostics.push( Diagnostic {
                            severity: Severity::Warning,
                            message: format!("gate applied to q{} after it was measured", qubit),
                            line,
                            col,
                        });
                    }
                }
            }
            Instruction::Barrier {..} => {}
        }
    }

    diagnostics
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::lower::lower;
    use crate::parser::parse;

    fn validate_source(
        source: &str,
    ) -> Vec<Diagnostic> {
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();
        let circuits = lower(&program).unwrap();

        let (circuit, spans) = &circuits[0];

        validate(circuit, spans)
    }

    #[test]
    fn warns_when_gate_is_applied_after_measurement() {
        let source = r#"circuit a(1) {
    measure q0 -> c0
    H q0
}
"#;

        let diagnostics = validate_source(source);

        assert_eq!(diagnostics.len(), 1);

        assert_eq!(
            diagnostics[0],
            Diagnostic {
                severity: Severity::Warning,
                message: "gate applied to q0 after it was measured".to_string(),
                line: 3,
                col: 5,
            }
        );
    }
}