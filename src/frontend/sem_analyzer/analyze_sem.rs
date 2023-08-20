use crate::frontend::ast::*;
use super::SemAnalyzer;

pub trait Analyze<T> {
    fn analyze(&mut self, target: &mut T);
}

impl Analyze<Program> for SemAnalyzer {
    fn analyze(&mut self, prog: &mut Program) {
        self.enter_scope();
        for comp_unit in &mut prog.0 {
            self.analyze(comp_unit);
        }
        self.exit_scope();
    }
}

impl Analyze<CompUnit> for SemAnalyzer {
    fn analyze(&mut self, comp_unit: &mut CompUnit) {
        use CompUnit::*;
        match comp_unit {
            GlobalDecl(global_decl) => self.analyze(global_decl),
            FuncDef(func_def) => self.analyze(func_def),
        }
    }
}

impl Analyze<FuncDef> for SemAnalyzer {
    fn analyze(&mut self, func_def: &mut FuncDef) {
        for param in &mut func_def.2 {
            self.insert_int(param.1.clone());
            param.1 = self.name(&param.1[..]).to_string();
        }
        self.analyze(&mut func_def.3);
    }
}

impl Analyze<Block> for SemAnalyzer {
    fn analyze(&mut self, block: &mut Block) {
        self.enter_scope();
        for block_item in &mut block.0 {
            self.analyze(block_item);
        }
        self.exit_scope();
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
            Empty => {}
            Exp(exp) => self.update(exp),
            Block(block) => self.analyze(block),
            If(exp, stmt, opt_stmt) => {
                self.update(exp);
                self.analyze(stmt.as_mut());
                if let Some(stmt) = opt_stmt {
                    self.analyze(stmt.as_mut());
                }
            }
            While(exp, stmt) => {
                self.update(exp);
                self.analyze(stmt.as_mut());
            }
            Break => {}
            Continue => {}
            Return(exp) => self.update(exp),
        }
    }
}

impl Analyze<GlobalDecl> for SemAnalyzer {
    fn analyze(&mut self, global_decl: &mut GlobalDecl) {
        use Decl::*;
        use VarDef::*;
        use super::eval::Eval;
        match &mut global_decl.0 {
            ConstDecl(const_decl) => self.analyze(const_decl),
            VarDecl(var_decl) => {
                for var_def in &mut var_decl.1 {
                    match var_def {
                        NoInit(name) => {
                            self.insert_int(name.to_string());
                            *name = self.name(&name[..]).to_string();
                        }
                        Init(name, init_val) => {
                            self.insert_int(name.to_string());
                            *name = self.name(&name[..]).to_string();
                            let value = self.eval(init_val);
                            *init_val = InitVal::Number(Number(value));
                        }
                    }
                }
            }
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