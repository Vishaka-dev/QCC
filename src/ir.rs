#[derive(Debug, Clone, PartialEq)]
pub enum Gate {
    H { qubit: usize },
    X { qubit: usize },
    Y { qubit: usize },
    Z { qubit: usize },

    S { qubit: usize },
    T { qubit: usize },

    // to store angles
    RX { qubit: usize, angle: f64 },
    RY { qubit: usize, angle: f64 },
    RZ { qubit: usize, angle: f64 },

    CNOT { control: usize, target: usize },
    CZ { control: usize, target: usize },
    SWAP { first: usize, second: usize },
}

impl Gate {
    pub fn qubits(&self) -> Vec<usize> {
        match self {
            Gate::H { qubit }
            | Gate::X { qubit }
            | Gate::Y { qubit }
            | Gate::Z { qubit }
            | Gate::S { qubit }
            | Gate::T { qubit }
            | Gate::RX { qubit, .. }
            | Gate::RY { qubit, .. }
            | Gate::RZ { qubit, .. } => {
                vec![*qubit]
            }

            Gate::CNOT { control, target }
            | Gate::CZ { control, target } => {
                vec![*control, *target]
            }

            Gate::SWAP { first, second } => {
                vec![*first, *second]
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Gate(Gate),
    Measure { qubit: usize, clbit: usize },
    Barrier { qubits: Vec<usize> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Circuit {
    pub name: String,
    pub qubit_count: usize,
    pub clbit_count: usize,
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
        let measurement = Instruction::Measure { qubit: 0, clbit: 0 };

        assert_eq!(h, Instruction::Gate(Gate::H { qubit: 0 }));
        assert_eq!(measurement, Instruction::Measure { qubit: 0, clbit: 0 });

        let bell_state = Circuit {
            name: "bell".to_string(),
            qubit_count: 2,
            clbit_count: 2,
            instructions: vec![
                Instruction::Gate(Gate::H { qubit: 0 }),
                Instruction::Gate(Gate::CNOT {
                    control: 0,
                    target: 1,
                }),
                Instruction::Measure { qubit: 0, clbit: 0 },
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
