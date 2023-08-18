use std::collections::{VecDeque, HashMap};
use symtab::{Symbol::{self, *}, SymTab};
use super::ast::*;
use analyze_sem::Analyze;

mod symtab;
mod analyze_sem;
mod eval;
mod update;

#[derive(Default)]
pub struct SemAnalyzer {
    symtabs: VecDeque<SymTab>,
    ident_cnt: HashMap<String, u32>,
}

impl SemAnalyzer {
    pub fn new() -> Self {
        Self {
            symtabs: VecDeque::new(),
            ident_cnt: HashMap::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.symtabs.push_front(SymTab::new());
    }

    pub fn exit_scope(&mut self) {
        self.symtabs.pop_front();
    }

    fn symbol(&self, name: &str) -> &Symbol {
        for table in &self.symtabs {
            if let Some(symbol) = table.get(name) {
                return symbol;
            }
        }
        panic!("Symbol {name} not found");
    }

    pub fn run(&mut self, prog: &mut Program) {
        self.analyze(prog);
        *self = Default::default();
    }

    pub fn name(&self, name: &str) -> &str {
        match self.symbol(name) {
            Int { token } => token,
            ConstInt { token, .. } => token,
        }
    }

    pub fn value(&self, name: &str) -> i32 {
        match self.symbol(name) {
            ConstInt { value, .. } => *value,
            _ => panic!("Get value of non-const symbol {name}"),
        }
    }

    pub fn is_const(&self, name: &str) -> bool {
        match self.symbol(name) {
            ConstInt { .. } => true,
            _ => false,
        }
    }
    
    pub fn insert_int(&mut self, name: String) {
        let cnt = self.ident_cnt.entry(name.clone()).or_default();
        let token = format!("@{name}_{cnt}");
        *cnt += 1;
        self.symtabs.front_mut().unwrap().insert(
            name,
            Int { token },
        );
    }

    pub fn insert_const_int(&mut self, name: String, value: i32) {
        let cnt = self.ident_cnt.entry(name.clone()).or_default();
        let token = format!("@{name}_{cnt}");
        *cnt += 1;
        self.symtabs.front_mut().unwrap().insert(
            name,
            ConstInt { token, value }
        );
    }
}