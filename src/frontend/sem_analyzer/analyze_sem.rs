//! This module defines the trait `Analyze` and its implementations.
//! Semantic analysis is done by traversing the AST
//! and performing AST transformations.

use super::eval::Eval;
use super::fold::Fold;
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
            VarDecl(var_decl) => self.analyze(var_decl),
            FuncDef(func_def) => self.analyze(func_def),
        }
    }
}


impl Analyze<VarDecl> for SemAnalyzer {
    fn analyze(&mut self, decl: &mut VarDecl) {
        use VarDef::*;
        let is_global = decl.is_global;
        let is_const = decl.is_const;

        decl.var_defs.iter_mut().for_each(|def| {
            match def {
                Scalar(ident, opt_exp) => {
                    if is_const {
                        let value = self.eval(opt_exp.as_ref().unwrap());
                        self.insert_const_int(ident.clone(), value);
                        return;
                    }
                    self.insert_int(ident.clone());
                    self.mangle(ident);
                    if let Some(exp) = opt_exp {
                        if is_global {
                            self.fold(exp);
                        } else {
                            self.update(exp);
                        }
                    }
                }
                Array(ident, sizes, opt_init) => {
                    self.insert_int_array(ident.clone());
                    self.mangle(ident);
                    sizes.iter_mut().for_each(|size| {
                        self.fold(size);
                    });
                    if let Some(init) = opt_init {
                        if is_global || is_const {
                            self.fold(init);
                        } else {
                            self.update(init);
                        }
                        Self::flatten(def);
                    }
                }
            }
        });
    }
}

impl Analyze<FuncDef> for SemAnalyzer {
    fn analyze(&mut self, func_def: &mut FuncDef) {
        // Record and update parameters.
        func_def.2.iter_mut().for_each(|param| {
            self.insert_int(param.1.clone());
            self.mangle(&mut param.1);
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

impl Analyze<BlockItem> for SemAnalyzer {
    fn analyze(&mut self, block_item: &mut BlockItem) {
        use BlockItem::*;
        match block_item {
            Stmt(stmt) => self.analyze(stmt),
            VarDecl(decl) => self.analyze(decl),
        }
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
