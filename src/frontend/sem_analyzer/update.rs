use crate::frontend::ast::{self, *};
use super::SemAnalyzer;

macro_rules! impl_update_binary_op {
    ($T:ty, $arm1:tt, $arm2:tt, var) => {
        impl Update<$T> for SemAnalyzer {
            fn update(&mut self, exp: &mut $T) {
                use $T::*;
                match exp {
                    $arm1(bexp) => self.update(bexp.as_mut()),
                    $arm2(bexps, _, bexp) => {
                        self.update(bexp.as_mut());
                        self.update(bexps.as_mut());
                    }
                }
            }
        }
    };
    ($T:ty, $arm1:tt, $arm2:tt, fixed) => {
        impl Update<$T> for SemAnalyzer {
            fn update(&mut self, exp: &mut $T) {
                use $T::*;
                match exp {
                    $arm1(bexp) => self.update(bexp.as_mut()),
                    $arm2(bexps, bexp) => {
                        self.update(bexp.as_mut());
                        self.update(bexps.as_mut());
                    }
                }
            }
        }
    };
}

pub trait Update<T> {
    fn update(&mut self, target: &mut T);
}

impl Update<Exp> for SemAnalyzer {
    fn update(&mut self, exp: &mut Exp) {
        self.update(&mut exp.0);
    }
}

impl Update<PrimaryExp> for SemAnalyzer {
    fn update(&mut self, exp: &mut PrimaryExp) {
        use PrimaryExp::*;
        match exp {
            BracketedExp(bexp) => self.update(bexp.as_mut()),
            Number(_) => {}
            LVal(lval) => {
                let ident = &mut lval.0;
                if self.is_const(ident) {
                    let value = self.value(ident);
                    *exp = Number(ast::Number(value));
                } else {
                    self.update(lval);
                }
            }
        }
    }
}

impl Update<UnaryExp> for SemAnalyzer {
    fn update(&mut self, exp: &mut UnaryExp) {
        use UnaryExp::*;
        match exp {
            Primary(bexp) => self.update(bexp.as_mut()),
            OpUnary(_, bexp) => self.update(bexp.as_mut()),
        }
    }
}

impl_update_binary_op!(MulExp, Unary, MulOpUnary, var);
impl_update_binary_op!(AddExp, Mul, AddOpMul, var);
impl_update_binary_op!(RelExp, Add, RelOpAdd, var);
impl_update_binary_op!(EqExp, Rel, EqOpRel, var);
impl_update_binary_op!(LAndExp, Eq, LAndEq, fixed);
impl_update_binary_op!(LOrExp, LAnd, LOrLAnd, fixed);

impl Update<InitVal> for SemAnalyzer {
    fn update(&mut self, init_val: &mut InitVal) {
        self.update(&mut init_val.0);
    }
}

impl Update<LVal> for SemAnalyzer {
    fn update(&mut self, lval: &mut LVal) {
        let ident = &mut lval.0;
        *ident = self.name(&ident[..]).to_string();
    }
}