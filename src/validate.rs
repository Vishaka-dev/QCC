use crate::diag::{Diagnostic, Severity};
use crate::ir::{Circuit, Instruction};
//use std::collections::HashSet;

fn validate_gate_after_measure(
    circuit: &Circuit,
    spans: &[(usize, usize)],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut measured = vec![false; circuit.qubit_count];

    for (index, instruction) in circuit.instructions.iter().enumerate() {
        let (line, col) = spans[index];

        match instruction {
            Instruction::Measure { qubit, .. } => {
                measured[*qubit] = true;
            }

            Instruction::Gate(gate) => {
                for qubit in gate.qubits() {
                    if measured[qubit] {
                        diagnostics.push(Diagnostic {
                            severity: Severity::Warning,
                            message: format!(
                                "qubit q{} is used after being measured",
                                qubit
                            ),
                            line,
                            col,
                        });

                        // Warn only once for this qubit.
                        measured[qubit] = false;
                    }
                }
            }

            Instruction::Barrier { .. } => {}
        }
    }

    diagnostics
}

pub fn validate_over_written_clbits(circuit: &Circuit, spans: &[(usize, usize)]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let mut last_write: Vec<Option<usize>> = vec![None; circuit.clbit_count];

    for (index, instruction) in circuit.instructions.iter().enumerate() {
        if let Instruction::Measure { clbit, .. } = instruction {
            if let Some(previous_index) = last_write[*clbit] {
                let (previous_line, _) = spans[previous_index];
                let (line, col) = spans[index];

                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!(
                        "classical bit c{} is overwritten (previous measurement at line {})",
                        clbit,
                        previous_line
                    ),
                    line,
                    col,
                });
            }
            last_write[*clbit] = Some(index);
        }
    }
    diagnostics
}

fn validate_unused_qubits(
    circuit: &Circuit,
    circuit_line: usize,
    circuit_col: usize,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut used = vec![false; circuit.qubit_count];

    for instruction in &circuit.instructions {
        match instruction {
            Instruction::Gate(gate) => {
                for qubit in gate.qubits() {
                    used[qubit] = true;
                }
            }

            Instruction::Measure { qubit, .. } => {
                used[*qubit] = true;
            }

            Instruction::Barrier { qubits } => {
                for qubit in qubits {
                    used[*qubit] = true;
                }
            }
        }
    }

    for (qubit, was_used) in used.iter().enumerate() {
        if !was_used {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                message: format!("qubit q{} is never used", qubit),
                line: circuit_line,
                col: circuit_col,
            });
        }
    }

    diagnostics
}

pub fn validate(circuit: &Circuit, spans: &[(usize, usize)], circuit_line: usize, circuit_col: usize) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    //let mut measured = HashSet::new();

    diagnostics.extend( validate_gate_after_measure(circuit, spans) );
    diagnostics.extend( validate_over_written_clbits(circuit, spans) );
    diagnostics.extend( validate_unused_qubits(circuit, circuit_line, circuit_col) );

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

        let (circuit, spans, circuit_pos) = &circuits[0];

        validate(
            circuit,
            spans,
            circuit_pos.0,
            circuit_pos.1,
        )
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
                message: "qubit q0 is used after being measured".to_string(),
                line: 3,
                col: 5,
            }
        );
    }
}