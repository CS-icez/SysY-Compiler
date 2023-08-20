//! This module defines the abstract syntax tree (AST) for the SysY language.
//! I adjusted the grammars specified in
//! https://pku-minic.github.io/online-doc/#/misc-app-ref/sysy-spec
//! for convenience, while maintaining equivalence.
//! See /frontend/sysy.lalrpop for the adjusted grammars.

pub struct Program(pub Vec<CompUnit>);

pub enum CompUnit {
    GlobalDecl(GlobalDecl),
    FuncDef(FuncDef),
}

// Variable declaration.

pub struct GlobalDecl(pub Decl);

pub enum Decl {
    ConstDecl(ConstDecl),
    VarDecl(VarDecl),
}

pub struct ConstDecl(pub BType, pub Vec<ConstDef>);

pub enum BType {
    Int, Void,
}

pub struct ConstDef(pub String, pub ConstInitVal);

pub struct ConstInitVal(pub ConstExp);

pub struct VarDecl(pub BType, pub Vec<VarDef>);

pub enum VarDef {
    NoInit(String),
    Init(String, InitVal),
}

pub enum InitVal {
    Exp(Exp),
    Number(Number), // This arm not used in parsing.
}

// Function definition.

pub struct FuncDef(pub BType, pub String, pub Vec<FuncFParam>, pub Block);

pub struct FuncFParam(pub BType, pub String);

// Block.

pub struct Block(pub Vec<BlockItem>);

pub enum BlockItem {
    Decl(Decl),
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

pub struct Exp(pub LOrExp);

pub struct LVal(pub String);

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

pub struct ConstExp(pub Exp);
