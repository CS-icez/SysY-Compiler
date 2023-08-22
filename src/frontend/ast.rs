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
    Array(String, Vec<Exp>, Option<Vec<Exp>>),
}

// Function definition.

pub struct FuncDef(pub BType, pub String, pub Vec<FuncFParam>, pub Block);

pub struct FuncFParam(pub BType, pub String);

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
    Return(Exp),
}

// Expression.

pub enum Exp {
    LOrExp(LOrExp),
    Number(Number), // This arm not used in parsing.
}

pub enum LVal {
    Ident(String),
    ArrayElem(String, Exp),
}

pub enum PrimaryExp {
    BracketedExp(Box<Exp>),
    Number(Number),
    LVal(LVal),
}

pub struct Number(pub i32);

pub enum UnaryExp {
    Primary(Box<PrimaryExp>),
    FuncCall(String, Vec<Exp>),
    OpUnary(UnaryOp, Box<UnaryExp>),
}

pub enum UnaryOp {
    Plus, Minus, Not,
}

pub enum MulExp {
    Unary(Box<UnaryExp>),
    MulOpUnary(Box<MulExp>, MulOp, Box<UnaryExp>),
}

pub enum MulOp {
    Mul, Div, Rem,
}

pub enum AddExp {
    Mul(Box<MulExp>),
    AddOpMul(Box<AddExp>, AddOp, Box<MulExp>),
}

pub enum AddOp {
    Add, Sub,
}

pub enum RelExp {
    Add(Box<AddExp>),
    RelOpAdd(Box<RelExp>, RelOp, Box<AddExp>),
}

pub enum RelOp {
    Le, Lt, Ge, Gt,
}

pub enum EqExp {
    Rel(Box<RelExp>),
    EqOpRel(Box<EqExp>, EqOp, Box<RelExp>),
}

pub enum EqOp {
    Eq, Ne,
}

pub enum LAndExp {
    Eq(Box<EqExp>),
    LAndEq(Box<LAndExp>, Box<EqExp>),
}

pub enum LOrExp {
    LAnd(Box<LAndExp>),
    LOrLAnd(Box<LOrExp>, Box<LAndExp>),
}
