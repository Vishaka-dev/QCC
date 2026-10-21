use crate::ast::{Ref, Stmt, StmtKind};
use crate::ir::{Gate, Instruction};
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
                            message: format!("unknown gate {}", name),
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

pub fn lower_circuit(def: &CircuitDef) -> Result<Circuit, LowerError> {
    let clbit_count = def.clbit_count.unwrap_or(def.qubit_count);

    let mut instructions = Vec::new();

    for stmt in &def.body {
        let lowered = lower_stmt(
            stmt,
            def.qubit_count,
            clbit_count,
        )?;

        instructions.extend(lowered);
    }

    Ok(Circuit {
        name: def.name.clone(),
        qubit_count: def.qubit_count,
        clbit_count,
        instructions,
    })
}
pub fn lower(program: &Program) -> Result<Vec<Circuit>, LowerError> {
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

    #[test]
    fn gate_signatures() {
        assert_eq!(gate_signature("H"), Some((1, false)));
        assert_eq!(gate_signature("RX"), Some((1, true)));
        assert_eq!(gate_signature("CNOT"), Some((2, false)));

        assert_eq!(gate_signature("CNOTT"), None);
        assert_eq!(gate_signature("h"), None);
    }
}