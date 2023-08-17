use std::collections::HashMap;
use Symbol::*;

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

    #[allow(dead_code)]
    pub fn contains(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }

    pub fn get(&self, name: &str) -> &Symbol {
        self.table.get(name)
            .expect(&format!("Symbol {name} not found"))
    }

    pub fn insert_int(&mut self, name: String) {
        let token = format!("@{name}");
        self.table.insert(
            name,
            Int { token },
        );
    }

    pub fn insert_const_int(&mut self, name: String, value: i32) {
        self.table.insert(
            name.clone(),
            ConstInt { token: name, value }
        );
    }
}