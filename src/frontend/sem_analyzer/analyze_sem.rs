//! This module defines the trait `Analyze` and its implementations.
//! Semantic analysis is done by traversing the AST
//! and performing AST transformations.

use super::eval::Eval;
use super::update::Update;
use super::SemAnalyzer;
use crate::frontend::ast::*;

pub trait Analyze<T> {
    fn analyze(&mut self, target: &mut T);
}

impl Analyze<Program> for SemAnalyzer {
    fn analyze(&mut self, prog: &mut Program) {
        self.enter_scope(); // Global scope.
        prog.0.iter_mut().for_each(|unit| self.analyze(unit));
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
        // Record and update parameters.
        func_def.2.iter_mut().for_each(|param| {
            self.insert_int(param.1.clone());
            param.1 = self.to_mangled(&param.1);
        });
        // Analyze function body.
        self.analyze(&mut func_def.3);
    }
}

impl Analyze<Block> for SemAnalyzer {
    fn analyze(&mut self, block: &mut Block) {
        self.enter_scope();
        block.0.iter_mut().for_each(|item| self.analyze(item));
        self.exit_scope();
    }
}

impl Analyze<Stmt> for SemAnalyzer {
    fn analyze(&mut self, stmt: &mut Stmt) {
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
        match &mut global_decl.0 {
            ConstDecl(const_decl) => self.analyze(const_decl),
            VarDecl(var_decl) => {
                for var_def in &mut var_decl.1 {
                    match var_def {
                        NoInit(name) => {
                            self.insert_int(name.to_string());
                            *name = self.to_mangled(name);
                        }
                        Init(name, init_val) => {
                            self.insert_int(name.to_string());
                            *name = self.to_mangled(name);
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
        const_decl.1.iter_mut().for_each(|def| self.analyze(def));
    }
}

impl Analyze<ConstDef> for SemAnalyzer {
    fn analyze(&mut self, const_def: &mut ConstDef) {
        let name = const_def.0.clone();
        let value = self.eval(&const_def.1);
        self.insert_const_int(name, value);
    }
}

impl Analyze<VarDecl> for SemAnalyzer {
    // Local variable declaration.
    fn analyze(&mut self, var_decl: &mut VarDecl) {
        var_decl.1.iter_mut().for_each(|def| self.analyze(def));
    }
}

impl Analyze<VarDef> for SemAnalyzer {
    // Local variable definition.
    fn analyze(&mut self, var_def: &mut VarDef) {
        use VarDef::*;
        match var_def {
            NoInit(name) => {
                self.insert_int(name.to_string());
                *name = self.to_mangled(name);
            }
            Init(name, init_val) => {
                self.insert_int(name.to_string());
                self.update(init_val);
                *name = self.to_mangled(name);
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
