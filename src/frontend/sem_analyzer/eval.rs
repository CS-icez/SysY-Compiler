//! Evaluate constant expressions.

use core::panic;

use super::SemAnalyzer;
use crate::frontend::ast::*;

macro_rules! panic_arm {
    () => {
        panic!("Unexpected arm")
    };
}

pub trait Eval<T> {
    /// Evaluates the given expression.
    fn eval(&self, target: &T) -> i32;
}

impl Eval<Exp> for SemAnalyzer {
    fn eval(&self, exp: &Exp) -> i32 {
        use Exp::*;
        match exp {
            LOrExp(lor) => self.eval(lor),
            Number(..) => panic_arm!(),
        }
    }
}

impl Eval<LVal> for SemAnalyzer {
    fn eval(&self, lval: &LVal) -> i32 {
        use LVal::*;
        match lval {
            Ident(ident) => self.value(ident),
            ArrayElem(..) => panic_arm!(),
        }
    }
}

impl Eval<PrimaryExp> for SemAnalyzer {
    fn eval(&self, exp: &PrimaryExp) -> i32 {
        use PrimaryExp::*;
        match exp {
            BracketedExp(bexp) => self.eval(bexp.as_ref()),
            Number(num) => self.eval(num),
            LVal(lval) => self.eval(lval),
        }
    }
}

impl Eval<Number> for SemAnalyzer {
    fn eval(&self, num: &Number) -> i32 {
        num.0
    }
}

impl Eval<UnaryExp> for SemAnalyzer {
    fn eval(&self, exp: &UnaryExp) -> i32 {
        use UnaryExp::*;
        use UnaryOp::*;
        match exp {
            Primary(bexp) => self.eval(bexp.as_ref()),
            FuncCall(..) => panic_arm!(),
            OpUnary(op, bexp) => {
                let value = self.eval(bexp.as_ref());
                match op {
                    Plus => value,
                    Minus => -value,
                    Not => (value == 0) as i32,
                }
            }
        }
    }
}

// This saves me from writing a lot of similar code.
// I really hope lalrpop could support defining rule precedence.
macro_rules! impl_eval_binary_op {
    ($T:ty, $arm1:tt, $arm2:tt, $clo:tt) => {
        impl Eval<$T> for SemAnalyzer {
            fn eval(&self, exp: &$T) -> i32 {
                use $T::*;
                match exp {
                    $arm1(bexp) => self.eval(bexp.as_ref()),
                    $arm2(bexps, bexp) => {
                        let lhs = self.eval(bexps.as_ref());
                        let rhs = self.eval(bexp.as_ref());
                        $clo(lhs, rhs)
                    }
                }
            }
        }
    };
    ($T:ty, $arm1:tt, $arm2:tt, $O:ty,
        op_rule: $($arm:tt => $clo:tt,)*) => {
        impl Eval<$T> for SemAnalyzer {
            fn eval(&self, exp: &$T) -> i32 {
                use $T::*;
                use $O::*;
                match exp {
                    $arm1(bexp) => self.eval(bexp.as_ref()),
                    $arm2(bexps, op, bexp) => {
                        let lhs = self.eval(bexps.as_ref());
                        let rhs = self.eval(bexp.as_ref());
                        match op {
                            $($arm => $clo(lhs, rhs),)*
                        }
                    }
                }
            }
        }
    };
}

impl_eval_binary_op!(MulExp, Unary, MulOpUnary, MulOp,
    op_rule:
        Mul => (|x, y| x * y),
        Div => (|x, y| x / y),
        Rem => (|x, y| x % y),
);

impl_eval_binary_op!(AddExp, Mul, AddOpMul, AddOp,
    op_rule:
        Add => (|x, y| x + y),
        Sub => (|x, y| x - y),
);

impl_eval_binary_op!(RelExp, Add, RelOpAdd, RelOp,
    op_rule:
        Le => (|x, y| (x <= y) as i32),
        Lt => (|x, y| (x < y) as i32),
        Ge => (|x, y| (x >= y) as i32),
        Gt => (|x, y| (x > y) as i32),
);

impl_eval_binary_op!(EqExp, Rel, EqOpRel, EqOp,
    op_rule:
        Eq => (|x, y| (x == y) as i32),
        Ne => (|x, y| (x != y) as i32),
);

impl_eval_binary_op!(LAndExp, Eq, LAndEq, (|x, y| (x != 0 && y != 0) as i32));

impl_eval_binary_op!(LOrExp, LAnd, LOrLAnd, (|x, y| (x != 0 || y != 0) as i32));
