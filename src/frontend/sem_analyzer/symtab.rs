//! Symbol table for semantic analysis.
//! A symbol table records symbol information within a scope in SysY program.
//! It is actually a hash table from name to symbol.
//! Note that this module only provides basic interfaces.
//! Advanced functionalities (name mangling, scope management, etc.)
//! are implemented in SemAnalyzer.

use std::collections::HashMap;

#[derive(Clone)]
pub enum Symbol {
    // Token is used as mangled name.
    Int { token: String },
    ConstInt { token: String, value: i32 },
}

#[derive(Default)]
pub struct SymTab {
    table: HashMap<String, Symbol>,
}

impl SymTab {
    /// Create an empty symbol table.
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    /// Returns a reference to the symbol corresponding to the given identifier.
    pub fn get(&self, ident: &str) -> Option<&Symbol> {
        self.table.get(ident)
    }

    /// Inserts a identifier-symbol pair into the symbol table.
    pub fn insert(&mut self, ident: String, symbol: Symbol) {
        self.table.insert(ident, symbol);
    }
}
