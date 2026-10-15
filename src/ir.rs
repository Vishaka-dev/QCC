#[derive(Debug, Clone, PartialEq)]
pub enum Gate {
    H { qubit: usize },
    X { qubit: usize },
    Y { qubit: usize },
    Z { qubit: usize },
    CNOT { control: usize, target: usize },
}

impl Gate {
    pun fn qubits(&self) -> Vec<usize> {
        match self {
            Gate::H { qubit }
            | Gate::X { qubit }
            | Gate::Y { qubit }
            | Gate::Z { qubit } => vec![*qubit],
            Gate::CNOT { control, target } => vec![ *control, *target ],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Gate(Gate),
    Measure { qubits: Vec<usize> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Circuit {
    pub name: String,
    pub qubit_count: usize,
    pub instructions: Vec<Instruction>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_gate() {
        let h = Gate::H { qubit: 0 };
        let cnot = Gate::CNOT {
            control: 0,
            target: 1,
        };

        assert_eq!(h, Gate::H { qubit: 0 });
        assert_eq!(
            cnot,
            Gate::CNOT {
                control: 0,
                target: 1
            }
        );

        let h = Instruction::Gate(Gate::H { qubit: 0 });
        let measurement = Instruction::Measure { qubits: vec![0, 1] };

        assert_eq!(h, Instruction::Gate(Gate::H { qubit: 0 }));
        assert_eq!(measurement, Instruction::Measure { qubits: vec![0, 1] });

        let bell_state = Circuit {
            name: "bell".to_string(),
            qubit_count: 2,
            instructions: vec![
                Instruction::Gate(Gate::H { qubit: 0 }),
                Instruction::Gate(Gate::CNOT {
                    control: 0,
                    target: 1,
                }),
                Instruction::Measure { qubits: vec![0, 1] },
            ],
        };
        assert_eq!(bell_state.name, "bell");
        assert_eq!(bell_state.qubit_count, 2);
        assert_eq!(bell_state.instructions.len(), 3);
        assert_eq!(
            bell_state.instructions[1],
            Instruction::Gate(Gate::CNOT {
                control: 0,
                target: 1
            })
        );
    }
}
