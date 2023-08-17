use symtab::{Symbol, SymTab};
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
    
    #[allow(dead_code)]
    pub fn symbol(&self, name: &str) -> &Symbol {
        self.symtab.symbol(name)
    }

    pub fn sym_value(&self, name: &str) -> i32 {
        self.symtab.value(name)
    }

    pub fn insert_sym(&mut self, symbol: Symbol) {
        self.symtab.insert(symbol);
    }
}