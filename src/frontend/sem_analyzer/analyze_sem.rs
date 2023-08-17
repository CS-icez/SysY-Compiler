use crate::frontend::ast::*;
use super::SemAnalyzer;

pub trait Analyze<T> {
    fn analyze(&mut self, target: &mut T);
}

impl Analyze<Program> for SemAnalyzer {
    fn analyze(&mut self, prog: &mut Program) {
        for comp_unit in &mut prog.0 {
            self.analyze(comp_unit);
        }
    }
}

impl Analyze<CompUnit> for SemAnalyzer {
    fn analyze(&mut self, comp_unit: &mut CompUnit) {
        use CompUnit::*;
        match comp_unit {
            FuncDef(func_def) => self.analyze(func_def),
        }
    }
}

impl Analyze<FuncDef> for SemAnalyzer {
    fn analyze(&mut self, func_def: &mut FuncDef) {
        self.analyze(&mut func_def.2);
    }
}

impl Analyze<Block> for SemAnalyzer {
    fn analyze(&mut self, block: &mut Block) {
        for block_item in &mut block.0 {
            self.analyze(block_item);
        }
    }
}

impl Analyze<Stmt> for SemAnalyzer {
    fn analyze(&mut self, stmt: &mut Stmt) {
        use super::update::Update;
        use Stmt::*;
        match stmt {
            Return(exp) => self.update(exp),
        }
    }
}

impl Analyze<Decl> for SemAnalyzer {
    fn analyze(&mut self, decl: &mut Decl) {
        use Decl::*;
        match decl {
            ConstDecl(const_decl) => self.analyze(const_decl),
        }
    }
}

impl Analyze<ConstDecl> for SemAnalyzer {
    fn analyze(&mut self, const_decl: &mut ConstDecl) {
        for const_def in &mut const_decl.1 {
            self.analyze(const_def);
        }
    }
}

impl Analyze<ConstDef> for SemAnalyzer {
    fn analyze(&mut self, const_def: &mut ConstDef) {
        use super::eval::Eval;
        use super::symtab::Symbol::*;
        let name = const_def.0.clone();
        let value = self.eval(&const_def.1);
        let symbol = ConstInt { name, value };
        self.insert_sym(symbol);
    }
}

impl Analyze<BlockItem> for SemAnalyzer {
    fn analyze(&mut self, block_item: &mut BlockItem) {
        use BlockItem::*;
        match block_item {
            Stmt(stmt) => self.analyze(stmt),
            Decl(decl) => self.analyze(decl),
        }
    }
}