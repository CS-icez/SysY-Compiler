//! Fold AST nodes to constant values.

use super::eval::Eval;
use super::SemAnalyzer;
use crate::frontend::ast::*;

pub trait Fold<T> {
    fn fold(&self, target: &mut T);
}

impl Fold<InitList> for SemAnalyzer {
    fn fold(&self, list: &mut InitList) {
        use InitList::*;
        match list {
            Exp(exp) => self.fold(exp),
            List(list) => list.iter_mut().for_each(|list| self.fold(list)),
            Flat(_) => panic!("Unexpected arm"),
        }
    }
}

impl Fold<Exp> for SemAnalyzer {
    fn fold(&self, exp: &mut Exp) {
        if let Exp::LOrExp(lor_exp) = exp {
            let value = self.eval(lor_exp);
            exp.set_value(value);
        } else {
            panic!("Unexpected arm");
        }
    }
}
