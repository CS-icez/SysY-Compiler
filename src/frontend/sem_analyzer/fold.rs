//! Fold AST nodes to constant values.

use core::panic;

use super::eval::Eval;
use super::SemAnalyzer;
use crate::frontend::ast::*;

pub trait Fold<T> {
    /// Folds the given constant expression into its result.
    fn fold(&self, target: &mut T);
}

impl Fold<InitList> for SemAnalyzer {
    fn fold(&self, init: &mut InitList) {
        use InitList::*;
        match init {
            Exp(exp) => self.fold(exp),
            List(list) => list.iter_mut().for_each(|list| self.fold(list)),
            Flat(..) => panic!("Unexpected arm"),
        }
    }
}

impl Fold<Exp> for SemAnalyzer {
    fn fold(&self, exp: &mut Exp) {
        use Exp::*;
        match exp {
            LOrExp(lor) => {
                let value = self.eval(lor);
                exp.set_value(value);
            }
            Number(..) => panic!("Unexpected arm"),
        }
    }
}
