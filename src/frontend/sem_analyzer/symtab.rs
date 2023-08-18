use std::collections::HashMap;

#[derive(Clone)]
pub enum Symbol {
    Int { token: String },
    ConstInt { token: String, value: i32 },
}

#[derive(Default)]
pub struct SymTab {
    table: HashMap<String, Symbol>,
}

impl SymTab {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.table.get(name)
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) {
        self.table.insert(name, symbol);
    }
}