//! This module defines the abstract syntax tree (AST) for the SysY language.
//! I adjusted the grammars specified in
//! https://pku-minic.github.io/online-doc/#/misc-app-ref/sysy-spec
//! for convenience, while maintaining equivalence.
//! See /frontend/sysy.lalrpop for the adjusted grammars.

pub struct Program(pub Vec<CompUnit>);

pub enum CompUnit {
    VarDecl(VarDecl),
    FuncDef(FuncDef),
}

// Variable declaration.

pub struct VarDecl {
    pub is_global: bool,
    pub is_const: bool,
    pub btype: BType,
    pub var_defs: Vec<VarDef>,
}

pub enum BType {
    Int, Void,
}

pub enum VarDef {
    Scalar(String, Option<Exp>),
    Array(String, Vec<Exp>, Option<InitList>),
}

pub enum InitList {
    Exp(Exp),
    List(Vec<InitList>),
    Flat(Vec<Exp>), // This arm not used in parsing.
}

// Function definition.

pub struct FuncDef(pub BType, pub String, pub Vec<FuncFParam>, pub Block);

pub enum FuncFParam {
    Scalar(BType, String),
    Array(BType, String, Vec<Exp>),
}

// Block.

pub struct Block(pub Vec<BlockItem>);

pub enum BlockItem {
    VarDecl(VarDecl),
    Stmt(Stmt),
}

// Statement.

pub enum Stmt {
    Assign(LVal, Exp),
    Empty,
    Exp(Exp),
    Block(Block),
    If(Exp, Box<Stmt>, Option<Box<Stmt>>),
    While(Exp, Box<Stmt>),
    Break,
    Continue,
    Return(Option<Exp>),
}

// Expression.

#[derive(Clone)]
pub enum Exp {
    LOrExp(LOrExp),
    Number(Number), // This arm not used in parsing.
}

impl Exp {
    pub fn from_number(value: i32) -> Self {
        Exp::Number(Number(value))
    }

    pub fn set_value(&mut self, value: i32) {
        *self = Exp::Number(Number(value));
    }

    pub fn value(&self) -> i32 {
        if let Exp::Number(Number(value)) = self {
            *value
        } else {
            panic!("Unexpected arm");
        }
    }
}

#[derive(Clone)]
pub enum LVal {
    Ident(String),
    ArrayElem(String, Vec<Exp>),
}

#[derive(Clone)]
pub enum PrimaryExp {
    BracketedExp(Box<Exp>),
    Number(Number),
    LVal(LVal),
}

#[derive(Clone)]
pub struct Number(pub i32);

#[derive(Clone)]
pub enum UnaryExp {
    Primary(Box<PrimaryExp>),
    FuncCall(String, Vec<Exp>),
    OpUnary(UnaryOp, Box<UnaryExp>),
}

#[derive(Clone)]
pub enum UnaryOp {
    Plus, Minus, Not,
}

#[derive(Clone)]
pub enum MulExp {
    Unary(Box<UnaryExp>),
    MulOpUnary(Box<MulExp>, MulOp, Box<UnaryExp>),
}

#[derive(Clone)]
pub enum MulOp {
    Mul, Div, Rem,
}

#[derive(Clone)]
pub enum AddExp {
    Mul(Box<MulExp>),
    AddOpMul(Box<AddExp>, AddOp, Box<MulExp>),
}

#[derive(Clone)]
pub enum AddOp {
    Add, Sub,
}

#[derive(Clone)]
pub enum RelExp {
    Add(Box<AddExp>),
    RelOpAdd(Box<RelExp>, RelOp, Box<AddExp>),
}

#[derive(Clone)]
pub enum RelOp {
    Le, Lt, Ge, Gt,
}

#[derive(Clone)]
pub enum EqExp {
    Rel(Box<RelExp>),
    EqOpRel(Box<EqExp>, EqOp, Box<RelExp>),
}

#[derive(Clone)]
pub enum EqOp {
    Eq, Ne,
}

#[derive(Clone)]
pub enum LAndExp {
    Eq(Box<EqExp>),
    LAndEq(Box<LAndExp>, Box<EqExp>),
}

#[derive(Clone)]
pub enum LOrExp {
    LAnd(Box<LAndExp>),
    LOrLAnd(Box<LOrExp>, Box<LAndExp>),
}
