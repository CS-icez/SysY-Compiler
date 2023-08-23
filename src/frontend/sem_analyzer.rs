//! Semantic analyzer.

mod analyze_sem;
mod eval;
mod fold;
mod flatten;
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
        self.symtabs.iter().find_map(|table| table.get(ident)).unwrap()
    }

    /// Returns the mangled name of the given identifier.
    fn mangled_name(&self, ident: &str) -> &str {
        match self.symbol(ident) {
            Int { token } => token,
            ConstInt { token, .. } => token,
            IntArray { token } => token,
        }
    }

    /// Converts the identifier to its mangled name.
    fn mangle(&self, ident: &mut String) {
        *ident = self.mangled_name(&ident).to_string();
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

    /// Returns the next token of the given identifier.
    fn next_token(&mut self, ident: &str) -> String {
        let cnt = self.ident_cnt.entry(ident.to_string()).or_default();
        let token = format!("@{ident}_{cnt}");
        *cnt += 1;
        token
    }

    /// Inserts an integer symbol into the symbol table of the current scope.
    fn insert_int(&mut self, ident: String) {
        let token = self.next_token(&ident);
        self.symtabs
            .front_mut()
            .unwrap()
            .insert(ident, Int { token });
    }

    /// Inserts a constant integer symbol into the symbol table
    /// of the current scope.
    fn insert_const_int(&mut self, ident: String, value: i32) {
        let token = self.next_token(&ident);
        self.symtabs
            .front_mut()
            .unwrap()
            .insert(ident, ConstInt { token, value });
    }

    /// Inserts an integer array symbol into the symbol table
    /// of the current scope.
    fn insert_int_array(&mut self, ident: String) {
        let token = self.next_token(&ident);
        self.symtabs
            .front_mut()
            .unwrap()
            .insert(ident, IntArray { token });
    }
}
