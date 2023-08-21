//! Semantic analyzer.

mod analyze_sem;
mod eval;
mod symtab;
mod update;

use super::ast::*;
use analyze_sem::Analyze;
use std::collections::{HashMap, VecDeque};
use symtab::{
    SymTab,
    Symbol::{self, *},
};

pub struct SemAnalyzer {
    symtabs: VecDeque<SymTab>, // Actually a stack, Rust std didn't provide it.
    ident_cnt: HashMap<String, u32>,
}

impl SemAnalyzer {
    /// Run semantic analysis on the given program.
    pub fn run_on(prog: &mut Program) {
        Self::new().analyze(prog);
    }

    /// Creates a new semantic analyzer.
    fn new() -> Self {
        Self {
            symtabs: VecDeque::new(),
            ident_cnt: HashMap::new(),
        }
    }

    /// Enters a new scope.
    fn enter_scope(&mut self) {
        self.symtabs.push_front(SymTab::new());
    }

    /// Exits the current scope.
    fn exit_scope(&mut self) {
        self.symtabs.pop_front();
    }

    /// Traverses the symbol table stack and returns the symbol
    /// corresponding to the given identifier.
    fn symbol(&self, ident: &str) -> &Symbol {
        let res = self.symtabs.iter().find_map(|table| table.get(ident));

        if let Some(symbol) = res {
            return symbol;
        } else {
            panic!("Symbol {ident} not found");
        }
    }

    /// Returns the mangled name of the given identifier.
    fn mangled_name(&self, ident: &str) -> &str {
        match self.symbol(ident) {
            Int { token } => token,
            ConstInt { token, .. } => token,
        }
    }

    /// Convert the identifier to its mangled name.
    /// `to_mangled(ident)` is equivalent to `mangled_name(ident).to_string()`.
    fn to_mangled(&self, ident: &str) -> String {
        self.mangled_name(&ident).to_string()
    }

    /// Returns the value of the given identifier,
    /// which must be a constant integer.
    fn value(&self, ident: &str) -> i32 {
        if let ConstInt { value, .. } = self.symbol(ident) {
            return *value;
        } else {
            panic!("Get value of non-const symbol {ident}");
        }
    }

    /// Returns whether the given identifier is a constant integer.
    fn is_const(&self, ident: &str) -> bool {
        matches!(self.symbol(ident), ConstInt { .. })
    }

    /// Inserts an integer symbol into the symbol table of the current scope.
    fn insert_int(&mut self, ident: String) {
        let cnt = self.ident_cnt.entry(ident.clone()).or_default();
        let token = format!("@{ident}_{cnt}");
        *cnt += 1;
        self.symtabs
            .front_mut()
            .unwrap()
            .insert(ident, Int { token });
    }

    /// Inserts a constant integer symbol into the symbol table
    /// of the current scope.
    fn insert_const_int(&mut self, ident: String, value: i32) {
        let cnt = self.ident_cnt.entry(ident.clone()).or_default();
        let token = format!("@{ident}_{cnt}");
        *cnt += 1;
        self.symtabs
            .front_mut()
            .unwrap()
            .insert(ident, ConstInt { token, value });
    }
}
