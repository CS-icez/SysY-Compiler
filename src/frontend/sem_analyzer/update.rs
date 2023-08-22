//! Update AST nodes with semantic information.
//! For now, all it does is replacing identifiers
//! with their mangled names or constant value.

use super::SemAnalyzer;
use crate::frontend::ast::{self, *};

pub trait Update<T> {
    fn update(&mut self, target: &mut T);
}

impl Update<Exp> for SemAnalyzer {
    fn update(&mut self, exp: &mut Exp) {
        if let Exp::LOrExp(exp) = exp {
            self.update(exp);
        } else {
            panic!("Unexpected arm");
        }
    }
}

impl Update<LVal> for SemAnalyzer {
    fn update(&mut self, lval: &mut LVal) {
        use LVal::*;
        match lval {
            Ident(ident) => {
                self.mangle(ident);
            }
            ArrayElem(ident, index) => {
                self.mangle(ident);
                self.update(index);
            }
        }
    }
}

impl Update<PrimaryExp> for SemAnalyzer {
    fn update(&mut self, exp: &mut PrimaryExp) {
        use PrimaryExp::*;
        use crate::frontend::ast::LVal::*;
        match exp {
            BracketedExp(bexp) => self.update(bexp.as_mut()),
            Number(_) => {}
            LVal(lval) => {
                match lval {
                    Ident(ident) => {
                        if self.is_const(ident) {
                            let value = self.value(ident);
                            *exp = Number(ast::Number(value));
                        } else {
                            self.update(lval);
                        }
                    }
                    ArrayElem(_, _) => self.update(lval),
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
            FuncCall(_, exps) => {
                exps.iter_mut().for_each(|exp| self.update(exp));
            }
            OpUnary(_, bexp) => self.update(bexp.as_mut()),
        }
    }
}

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

impl_update_binary_op!(MulExp, Unary, MulOpUnary, var);
impl_update_binary_op!(AddExp, Mul, AddOpMul, var);
impl_update_binary_op!(RelExp, Add, RelOpAdd, var);
impl_update_binary_op!(EqExp, Rel, EqOpRel, var);
impl_update_binary_op!(LAndExp, Eq, LAndEq, fixed);
impl_update_binary_op!(LOrExp, LAnd, LOrLAnd, fixed);
