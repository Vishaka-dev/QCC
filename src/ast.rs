#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Pi,
    Neg(Box<Expr>),
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    }
}

impl Expr {
    pub fn eval(&self) -> f64 {
        match self {
            Expr::Number(x) => *x,
            Expr::Pi => std::f64::consts::PI,
            Expr::Neg(expr) => -expr.eval(),
            Expr::Binary { left, op, right } => {
                let lhs = left.eval();
                let rhs = right.eval();

                match op {
                    BinOp::Add => lhs + rhs,
                    BinOp::Sub => lhs - rhs,
                    BinOp::Mul => lhs * rhs,
                    BinOp::Div => lhs / rhs,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_hand_built_tree() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Number(2.0)),
            op: BinOp::Add,
            right: Box::new(Expr::Number(3.0)),
        };
        assert_eq!(expr.eval(), 5.0);
    }
    #[test]
    fn eval_nested_tree() {
        let e = Expr::Neg(Box::new(
            Expr::Binary {
                left: Box::new(Expr::Pi),
                op: BinOp::Div,
                right: Box::new(Expr::Number(2.0)),
            }
        ));

        let expected = -std::f64::consts::PI / 2.0;

        assert!((e.eval() - expected).abs() < 1e-12);
    }
}

