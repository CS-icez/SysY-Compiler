use std::collections::HashMap;
use Symbol::*;

#[derive(Clone)]
pub enum Symbol {
    // Int { name: String },
    ConstInt { name: String, value: i32 },
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

    pub fn symbol(&self, name: &str) -> &Symbol {
        self.table.get(name).unwrap()
    }

    pub fn value(&self, name: &str) -> i32 {
        match self.symbol(name) {
            ConstInt { value, .. } => *value,
            #[allow(unreachable_patterns)]
            _ => panic!("Get value of a non-const symbol"),
        }
    }

    pub fn insert(&mut self, symbol: Symbol) {
        match symbol {
            ConstInt { ref name, .. } => {
                self.table.insert(name.clone(), symbol);
            }
        }
    }
}