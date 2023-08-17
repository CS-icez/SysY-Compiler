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
            Assign(lval, exp) => {
                self.update(lval);
                self.update(exp);
            }
            Return(exp) => self.update(exp),
        }
    }
}

impl Analyze<Decl> for SemAnalyzer {
    fn analyze(&mut self, decl: &mut Decl) {
        use Decl::*;
        match decl {
            ConstDecl(const_decl) => self.analyze(const_decl),
            VarDecl(var_decl) => self.analyze(var_decl),
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
        let name = const_def.0.clone();
        let value = self.eval(&const_def.1);
        self.insert_const_int(name, value);
    }
}

impl Analyze<VarDecl> for SemAnalyzer {
    fn analyze(&mut self, var_decl: &mut VarDecl) {
        for var_def in &mut var_decl.1 {
            self.analyze(var_def);
        }
    }
}

impl Analyze<VarDef> for SemAnalyzer {
    fn analyze(&mut self, var_def: &mut VarDef) {
        use super::update::Update;
        use VarDef::*;
        match var_def {
            NoInit(name) => {
                self.insert_int(name.to_string());
                *name = self.name(&name[..]).to_string();
            }
            Init(name, init_val) => {
                self.insert_int(name.to_string());
                self.update(init_val);
                *name = self.name(&name[..]).to_string();
            }
        }
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