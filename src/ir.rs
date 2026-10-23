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

impl Instruction {
    pub fn qubits(&self) -> Vec<usize> {
        match self {
            Instruction::Gate(gate) => gate.qubits(),
            Instruction::Measure { qubit, .. } => vec![*qubit],
            Instruction::Barrier { qubits } => qubits.clone(),
        }
    }
}

pub fn cancels(a: &Gate, b: &Gate) -> bool {
    match (a, b) {
        (Gate::H { qubit: q1 }, Gate::H { qubit: q2 }) => q1 == q2,

        (Gate::X { qubit: q1 }, Gate::X { qubit: q2 }) => q1 == q2,

        (Gate::Y { qubit: q1 }, Gate::Y { qubit: q2 }) => q1 == q2,

        (Gate::Z { qubit: q1 }, Gate::Z { qubit: q2 }) => q1 == q2,

        (Gate::CNOT { control: c1, target: t1 },
         Gate::CNOT { control: c2, target: t2 }) => {
            c1 == c2 && t1 == t2
        }

        (Gate::CZ { control: c1, target: t1 },
         Gate::CZ { control: c2, target: t2 }) => {
            (c1 == c2 && t1 == t2) || (c1 == t2 && t1 == c2)
        }

        (Gate::SWAP { first: f1, second: s1 },
         Gate::SWAP { first: f2, second: s2 }) => {
            (f1 == f2 && s1 == s2) || (f1 == s2 && s1 == f2)
        }

        _ => false,
    }
}

fn merge(a: &Gate, b: &Gate) -> Option<Gate> {
    match (a, b) {
        (
            Gate::RX {
                qubit: q1,
                angle: a1,
            },
            Gate::RX {
                qubit: q2,
                angle: a2,
            },
        ) if q1 == q2 => {
            Some(Gate::RX {
                qubit: *q1,
                angle: *a1 + *a2,
            })
        }

        (
            Gate::RY {
                qubit: q1,
                angle: a1,
            },
            Gate::RY {
                qubit: q2,
                angle: a2,
            },
        ) if q1 == q2 => {
            Some(Gate::RY {
                qubit: *q1,
                angle: *a1 + *a2,
            })
        }

        (
            Gate::RZ {
                qubit: q1,
                angle: a1,
            },
            Gate::RZ {
                qubit: q2,
                angle: a2,
            },
        ) if q1 == q2 => {
            Some(Gate::RZ {
                qubit: *q1,
                angle: *a1 + *a2,
            })
        }

        _ => None,
    }
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
    #[test]
    fn instruction_qubits_for_gate() {
        let instruction = Instruction::Gate(Gate::H { qubit: 2 });

        assert_eq!(instruction.qubits(), vec![2]);
    }

    #[test]
    fn instruction_qubits_for_measure() {
        let instruction = Instruction::Measure {
            qubit: 1,
            clbit: 0,
        };

        assert_eq!(instruction.qubits(), vec![1]);
    }

    #[test]
    fn instruction_qubits_for_barrier() {
        let instruction = Instruction::Barrier {
            qubits: vec![0, 1, 2],
        };

        assert_eq!(instruction.qubits(), vec![0, 1, 2]);
    }

    #[test]
    fn cancels_h_same_qubit() {
        let a = Gate::H { qubit: 0 };
        let b = Gate::H { qubit: 0 };

        assert!(cancels(&a, &b));
    }

    #[test]
    fn does_not_cancel_h_different_qubits() {
        let a = Gate::H { qubit: 0 };
        let b = Gate::H { qubit: 1 };

        assert!(!cancels(&a, &b));
    }

    #[test]
    fn cancels_cnot_same_control_and_target() {
        let a = Gate::CNOT {
            control: 0,
            target: 1,
        };

        let b = Gate::CNOT {
            control: 0,
            target: 1,
        };

        assert!(cancels(&a, &b));
    }

    #[test]
    fn does_not_cancel_cnot_reversed() {
        let a = Gate::CNOT {
            control: 0,
            target: 1,
        };

        let b = Gate::CNOT {
            control: 1,
            target: 0,
        };

        assert!(!cancels(&a, &b));
    }

    #[test]
    fn cancels_cz_in_both_orders() {
        let a = Gate::CZ {
            control: 0,
            target: 1,
        };

        let b = Gate::CZ {
            control: 1,
            target: 0,
        };

        assert!(cancels(&a, &b));
    }

    #[test]
    fn does_not_cancel_s_or_t() {
        let s1 = Gate::S { qubit: 0 };
        let s2 = Gate::S { qubit: 0 };

        let t1 = Gate::T { qubit: 0 };
        let t2 = Gate::T { qubit: 0 };

        assert!(!cancels(&s1, &s2));
        assert!(!cancels(&t1, &t2));
    }

    #[test]
    fn merges_rx_same_qubit() {
        let a = Gate::RX {
            qubit: 0,
            angle: 0.25,
        };

        let b = Gate::RX {
            qubit: 0,
            angle: 0.50,
        };

        assert_eq!(
            merge(&a, &b),
            Some(Gate::RX {
                qubit: 0,
                angle: 0.75,
            })
        );
    }

    #[test]
    fn merges_ry_same_qubit() {
        let a = Gate::RY {
            qubit: 1,
            angle: 0.25,
        };

        let b = Gate::RY {
            qubit: 1,
            angle: 0.50,
        };

        assert_eq!(
            merge(&a, &b),
            Some(Gate::RY {
                qubit: 1,
                angle: 0.75,
            })
        );
    }

    #[test]
    fn merges_rz_same_qubit() {
        let a = Gate::RZ {
            qubit: 0,
            angle: 1.0,
        };

        let b = Gate::RZ {
            qubit: 0,
            angle: 2.0,
        };

        assert_eq!(
            merge(&a, &b),
            Some(Gate::RZ {
                qubit: 0,
                angle: 3.0,
            })
        );
    }

    #[test]
    fn does_not_merge_different_rotation_types() {
        let a = Gate::RZ {
            qubit: 0,
            angle: 1.0,
        };

        let b = Gate::RX {
            qubit: 0,
            angle: 2.0,
        };

        assert_eq!(merge(&a, &b), None);
    }

    #[test]
    fn does_not_merge_different_qubits() {
        let a = Gate::RZ {
            qubit: 0,
            angle: 1.0,
        };

        let b = Gate::RZ {
            qubit: 1,
            angle: 2.0,
        };

        assert_eq!(merge(&a, &b), None);
    }
}
