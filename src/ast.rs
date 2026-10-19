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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_hand_built_tree() {
        // 1 + 2
        let e = Expr::Binary {
            left: Box::new(Expr::Number(1.0)),
            op: BinOp::Add,
            right: Box::new(Expr::Number(2.0)),
        };
        //assert_eq!(e.eval(), 3.0);
    }
}

