//! Update AST nodes with semantic information.
//! For now, all it does is replacing identifiers
//! with their mangled names or constant values.

use super::SemAnalyzer;
use crate::frontend::ast::{self, *};

pub trait Update<T> {
    /// Updates the given expression.
    fn update(&self, target: &mut T);
}

impl Update<InitList> for SemAnalyzer {
    fn update(&self, init: &mut InitList) {
        use InitList::*;
        match init {
            Exp(exp) => self.update(exp),
            List(list) => list.iter_mut().for_each(|list| self.update(list)),
            Flat(..) => panic!("Unexpected arm"),
        }
    }
}

impl Update<Exp> for SemAnalyzer {
    fn update(&self, exp: &mut Exp) {
        use Exp::*;
        match exp {
            LOrExp(lor) => self.update(lor),
            Number(..) => panic!("Unexpected arm"),
        }
    }
}

impl Update<LVal> for SemAnalyzer {
    fn update(&self, lval: &mut LVal) {
        use LVal::*;
        match lval {
            Ident(ident) => {
                self.mangle(ident);
            }
            ArrayElem(ident, indices) => {
                self.mangle(ident);
                indices.iter_mut().for_each(|exp| self.update(exp));
            }
        }
    }
}

impl Update<PrimaryExp> for SemAnalyzer {
    fn update(&self, exp: &mut PrimaryExp) {
        use PrimaryExp::*;
        use ast::LVal::*;
        match exp {
            BracketedExp(bexp) => self.update(bexp.as_mut()),
            Number(..) => {}
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
                    ArrayElem(..) => self.update(lval),
                }
            }
        }
    }
}

impl Update<UnaryExp> for SemAnalyzer {
    fn update(&self, exp: &mut UnaryExp) {
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
            fn update(&self, exp: &mut $T) {
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
            fn update(&self, exp: &mut $T) {
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
