use symtab::{Symbol::{self, *}, SymTab};
use super::ast::*;
use analyze_sem::Analyze;

mod symtab;
mod analyze_sem;
mod eval;
mod update;

#[derive(Default)]
pub struct SemAnalyzer {
    symtab: SymTab,
}

impl SemAnalyzer {
    pub fn new() -> Self {
        Self {
            symtab: SymTab::new(),
        }
    }

    pub fn run(&mut self, prog: &mut Program) {
        self.analyze(prog);
        *self = Default::default();
    }
    
    pub fn get(&self, name: &str) -> &Symbol {
        self.symtab.get(name)
    }

    #[allow(dead_code)]
    pub fn name(&self, name: &str) -> &str {
        match self.get(name) {
            Int { token } => token,
            ConstInt { token, .. } => token,
        }
    }

    pub fn value(&self, name: &str) -> i32 {
        match self.get(name) {
            ConstInt { value, .. } => *value,
            _ => panic!("Get value of non-const symbol {name}"),
        }
    }

    pub fn is_const(&self, name: &str) -> bool {
        match self.get(name) {
            ConstInt { .. } => true,
            _ => false,
        }
    }
    
    pub fn insert_int(&mut self, name: String) {
        self.symtab.insert_int(name);
    }
    
    pub fn insert_const_int(&mut self, name: String, value: i32) {
        self.symtab.insert_const_int(name, value);
    }

}