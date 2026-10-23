use crate::ast::*;
use crate::ir::*;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct LowerError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

fn gate_signature(name: &str) -> Option<( usize, bool )> {
    match name {
        "H" | "X" | "Y" | "Z" | "S" | "T" => Some((1, false)),
        "RX" | "RY" | "RZ" => Some((1, true)),
        "CNOT" | "CZ" | "SWAP" => Some((2, false)),
        _ => None,
    }
}

fn check_range( r: &Ref, limit: usize, what: &str ) -> Result<usize, LowerError> {
    let prefix = match what {
        "qubit" => "q",
        "classical bit" => "c",
        _ => "",
    };

    if r.index < limit {
        Ok(r.index)
    } else {
        Err(LowerError {
            message: format!(
                "{} {}{} out of range (circuit has {} {}s)",
                what,
                prefix,
                r.index,
                limit,
                what
            ),
            line: r.line,
            col: r.col,
        })
    }
}

fn lower_stmt(stmt: &Stmt, qubit_count: usize, clbit_count: usize) -> Result< Vec<Instruction>, LowerError> {
    match &stmt.kind {
        StmtKind::Gate {
            name,
            angle,
            qubits,
        } => {
            let (expected_qubits, take_angle) = 
                match gate_signature(name) {
                    Some(signature) => signature,
                    None => {
                        return Err(LowerError{
                            message: format!("unknown gate '{}'", name),
                            line: stmt.line,
                            col: stmt.col,
                        });
                    }
                };
            // check the angle
            if take_angle && angle.is_none() {
                return Err(LowerError{
                    message: format!("{} needs an angle, e.g. {}(pi/2)", name, name),
                    line: stmt.line,
                    col: stmt.col,
                })
            }
            if !take_angle && angle.is_some() {
                return Err(LowerError{
                    message: format!("{} does not take an angle", name),
                    line: stmt.line,
                    col: stmt.col,
                })
            }
            // check qubit count
            if qubits.len() != expected_qubits {
                return Err(LowerError{
                    message: format!("{} takes {} qubits, found {}", name, expected_qubits, qubits.len()),
                    line: stmt.line,
                    col: stmt.col,
                });
            }
            // range check the qubits
            let mut indices = Vec::new();

            for qubit in qubits {
                indices.push(check_range(qubit, qubit_count, "qubit")?);
            }

            // distinct qubit checking
            if expected_qubits == 2 && indices[0] == indices[1] {
                return Err(LowerError {
                    message: format!("{} needs two different qubits", name),
                    line: stmt.line,
                    col: stmt.col,
                });
            }

            let gate = match name.as_str() {
                "H" => Gate::H {
                    qubit: indices[0],
                },
                "X" => Gate::X {
                    qubit: indices[0],
                },
                "Y" => Gate::Y {
                    qubit: indices[0],
                },
                "Z" => Gate::Z {
                    qubit: indices[0],
                },
                "S" => Gate::S {
                    qubit: indices[0],
                },
                "T" => Gate::T {
                    qubit: indices[0],
                },

                "RX" => Gate::RX {
                    qubit: indices[0],
                    angle: angle.as_ref().unwrap().eval(),
                },
                "RY" => Gate::RY {
                    qubit: indices[0],
                    angle: angle.as_ref().unwrap().eval(),
                },
                "RZ" => Gate::RZ {
                    qubit: indices[0],
                    angle: angle.as_ref().unwrap().eval(),
                },

                "CNOT" => Gate::CNOT {
                    control: indices[0],
                    target: indices[1],
                },
                "CZ" => Gate::CZ {
                    control: indices[0],
                    target: indices[1],
                },
                "SWAP" => Gate::SWAP {
                    first: indices[0],
                    second: indices[1],
                },

                _ => unreachable!(),
            };
            Ok(vec![Instruction::Gate(gate)])

        },
        StmtKind::Measure { qubit, clbit } => {
            let qubit_index =
                check_range(qubit, qubit_count, "qubit")?;

            let clbit_index =
                check_range(clbit, clbit_count, "classical bit")?;

            Ok(vec![
                Instruction::Measure {
                    qubit: qubit_index,
                    clbit: clbit_index,
                }
            ])
        },  
        StmtKind::Barrier { qubits } => {
            let indices = if qubits.is_empty() {
                (0..qubit_count).collect()
            } else {
                let mut indices = Vec::new();

                for qubit in qubits {
                    indices.push(
                        check_range(qubit, qubit_count, "qubit")?
                    );
                }

                indices
            };

            Ok(vec![
                Instruction::Barrier {
                    qubits: indices,
                }
            ])
        },
    }
}

pub fn lower_circuit(def: &CircuitDef) -> Result<(Circuit, Vec<( usize, usize )>, (usize, usize)), LowerError> {
    let clbit_count = def.clbit_count.unwrap_or(def.qubit_count);

    let mut instructions = Vec::new();
    let mut spans = Vec::new();

    for stmt in &def.body {
        let lowered = lower_stmt(
            stmt,
            def.qubit_count,
            clbit_count,
        )?;

        for instruction in lowered {
            instructions.push(instruction);
            spans.push((stmt.line, stmt.col));
        }
    }

    let circuit = Circuit {
        name: def.name.clone(),
        qubit_count: def.qubit_count,
        clbit_count,
        instructions,
    };
    Ok((circuit, spans, (def.line, def.col), ))
}
pub fn lower(program: &Program) -> Result<Vec<(Circuit, Vec<(usize, usize)>, (usize, usize))>, LowerError> {
    let mut names = HashSet::new();
    let mut circuits = Vec::new();

    for def in &program.circuits {
        if !names.insert(def.name.clone()) {
            return Err(LowerError {
                message: format!(
                    "duplicate circuit name '{}'",
                    def.name
                ),
                line: def.line,
                col: def.col,
            });
        }

        circuits.push(lower_circuit(def)?);
    }

    Ok(circuits)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;

    fn lower_source(source: &str) -> Result<Vec<(Circuit, Vec<(usize, usize)>, (usize, usize))>, LowerError> {
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();
        lower(&program)
    }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;

    fn lower_source(
        source: &str,
    ) -> Result<Vec<(Circuit, Vec<(usize, usize)>, (usize, usize))>, LowerError>{
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();
        lower(&program)
    }

    #[test]
    fn gate_signatures() {
        assert_eq!(gate_signature("H"), Some((1, false)));
        assert_eq!(gate_signature("RX"), Some((1, true)));
        assert_eq!(gate_signature("CNOT"), Some((2, false)));

        assert_eq!(gate_signature("CNOTT"), None);
        assert_eq!(gate_signature("h"), None);
    }

    #[test]
    fn lower_bell_circuit() {
        let source = r#"circuit bell(2) {
    H q0
    CNOT q0 q1
    measure q0 -> c0
    measure q1 -> c1
}
"#;

        let circuits = lower_source(source).unwrap();

        assert_eq!(circuits.len(), 1);

        let (circuit, spans, _circuit_pos) = &circuits[0];

        assert_eq!(circuit.name, "bell");
        assert_eq!(circuit.qubit_count, 2);
        assert_eq!(circuit.clbit_count, 2);
        assert_eq!(circuit.instructions.len(), 4);

        assert_eq!(
            circuit.instructions[1],
            Instruction::Gate(Gate::CNOT {
                control: 0,
                target: 1,
            })
        );

        assert_eq!(spans.len(), 4);
    }

    #[test]
    fn lower_rz_angle() {
        let source = "circuit r(1, 3) { RZ(pi/2) q0 }";

        let circuits = lower_source(source).unwrap();
        let (circuit, _spans, _circuit_pos) = &circuits[0];

        let instruction = &circuit.instructions[0];

        match instruction {
            Instruction::Gate(Gate::RZ { qubit, angle }) => {
                assert_eq!(*qubit, 0);

                let expected = std::f64::consts::PI / 2.0;

                assert!((*angle - expected).abs() < 1e-12);
            }

            _ => panic!("expected RZ gate"),
        }
    }

    #[test]
    fn lower_empty_barrier_to_all_qubits() {
        let source = "circuit a(2) { barrier }";

        let circuits = lower_source(source).unwrap();
        let (circuit, _spans, _circuit_pos) = &circuits[0];
        assert_eq!(
            circuit.instructions[0],
            Instruction::Barrier {
                qubits: vec![0, 1],
            }
        );
    }

    #[test]
    fn error_unknown_gate() {
        let source = "circuit a(2) { CNOTT q0 q1 }";

        let err = lower_source(source).unwrap_err();

        assert_eq!(err.message, "unknown gate 'CNOTT'");
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 16);
    }

    #[test]
    fn error_missing_angle() {
        let source = "circuit a(1) { RZ q0 }";

        let err = lower_source(source).unwrap_err();

        assert_eq!(
            err.message,
            "RZ needs an angle, e.g. RZ(pi/2)"
        );
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 16);
    }

    #[test]
    fn error_unexpected_angle() {
        let source = "circuit a(1) { H(pi) q0 }";

        let err = lower_source(source).unwrap_err();

        assert_eq!(err.message, "H does not take an angle");
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 16);
    }

    #[test]
    fn error_wrong_gate_arity() {
        let source = "circuit a(2) { CNOT q0 }";

        let err = lower_source(source).unwrap_err();

        assert_eq!(
            err.message,
            "CNOT takes 2 qubits, found 1"
        );
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 16);
    }

    #[test]
    fn error_qubit_out_of_range() {
        let source = "circuit a(2) { H q5 }";

        let err = lower_source(source).unwrap_err();

        assert_eq!(
            err.message,
            "qubit q5 out of range (circuit has 2 qubits)"
        );
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 18);
    }

    #[test]
    fn error_two_qubit_gate_same_qubit() {
        let source = "circuit a(2) { CNOT q1 q1 }";

        let err = lower_source(source).unwrap_err();

        assert_eq!(
            err.message,
            "CNOT needs two different qubits"
        );
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 16);
    }

    #[test]
    fn error_clbit_out_of_range() {
        let source = "circuit a(2) { measure q0 -> c9 }";

        let err = lower_source(source).unwrap_err();

        assert_eq!(
            err.message,
            "classical bit c9 out of range (circuit has 2 classical bits)"
        );
        assert_eq!(err.line, 1);
        assert_eq!(err.col, 30);
    }

    #[test]
    fn error_duplicate_circuit_name() {
        let source = r#"circuit a(1) {
}

circuit a(2) {
}
"#;

        let err = lower_source(source).unwrap_err();

        assert_eq!(
            err.message,
            "duplicate circuit name 'a'"
        );
        assert_eq!(err.line, 4);
        assert_eq!(err.col, 1);
    }

    #[test]
    fn lower_preserves_instruction_spans() {
        let source = r#"circuit bell(2) {
    H q0
    CNOT q0 q1
    measure q0 -> c0
}
"#;

        let circuits = lower_source(source).unwrap();

        let (circuit, spans, _circuit_pos) = &circuits[0];

        assert_eq!(circuit.instructions.len(), 3);
        assert_eq!(spans.len(), 3);

        assert_eq!(spans[0], (2, 5));
        assert_eq!(spans[1], (3, 5));
        assert_eq!(spans[2], (4, 5));
    }
}
}