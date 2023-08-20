//! I adjusted the grammar specified in https://pku-minic.github.io/online-doc/#/misc-app-ref/sysy-spec
//! for convenience, while maintaining equivalence.

/// Program ::= CompUnit {CompUnit}
pub struct Program(pub Vec<CompUnit>);

/// CompUnit ::= GlobalDecl | FuncDef
pub enum CompUnit {
    GlobalDecl(GlobalDecl),
    FuncDef(FuncDef),
}

/// FuncDef ::= BType IDENT "(" [FuncFParams] ")" Block
/// FuncFParams ::= FuncFParam {"," FuncFParam}
pub struct FuncDef(pub BType, pub String, pub Vec<FuncFParam>, pub Block);

/// FuncFParam ::= BType IDENT
pub struct FuncFParam(pub BType, pub String);

/// Block ::= "{" {BlockItem} "}"
pub struct Block(pub Vec<BlockItem>);

/// Stmt ::= LVal "=" Exp ";"
///     | [Exp] ";"
///     | Block
///     | "if" "(" Exp ")" Stmt ["else" Stmt]
///     | "while" "(" Exp ")" Stmt
///     | "break" ";"
///     | "continue" ";"
///     | "return" Exp ";"
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

/// Number ::= INT_CONST
pub struct Number(pub i32);

/// Exp ::= LOrExp
pub struct Exp(pub LOrExp);

/// PrimaryExp ::= "(" Exp ")" | Number | LVal
pub enum PrimaryExp {
    BracketedExp(Box<Exp>),
    Number(Number),
    LVal(LVal),
}

/// UnaryExp ::= PrimaryExp
///     | IDENT "(" [FuncRParams] ")"
///     | UnaryOp UnaryExp;
/// FuncRParams ::= Exp {"," Exp}
pub enum UnaryExp {
    Primary(Box<PrimaryExp>),
    FuncCall(String, Vec<Exp>),
    OpUnary(UnaryOp, Box<UnaryExp>),
}

/// UnaryOp ::= "+" | "-" | "!"
pub enum UnaryOp {
    Plus, Minus, Not,
}

/// MulExp ::= UnaryExp | MulExp MulOp UnaryExp
pub enum MulExp {
    Unary(Box<UnaryExp>),
    MulOpUnary(Box<MulExp>, MulOp, Box<UnaryExp>),
}

/// MulOp ::= "*" | "/" | "%"
pub enum MulOp {
    Mul, Div, Rem,
}

/// AddExp ::= MulExp | AddExp AddOp MulExp
pub enum AddExp {
    Mul(Box<MulExp>),
    AddOpMul(Box<AddExp>, AddOp, Box<MulExp>),
}

/// AddOp ::= "+" | "-"
pub enum AddOp {
    Add, Sub,
}

/// RelExp ::= AddExp | RelExp RelOp AddExp
pub enum RelExp {
    Add(Box<AddExp>),
    RelOpAdd(Box<RelExp>, RelOp, Box<AddExp>),
}

/// RelOp ::= "<=" | "<" | ">=" | ">"
pub enum RelOp {
    Le, Lt, Ge, Gt,
}

/// EqExp ::= RelExp | EqExp EqOp RelExp
pub enum EqExp {
    Rel(Box<RelExp>),
    EqOpRel(Box<EqExp>, EqOp, Box<RelExp>),
}

/// EqOp ::= "==" | "!="
pub enum EqOp {
    Eq, Ne,
}

/// LAndExp ::= EqExp | LAndExp "&&" EqExp
pub enum LAndExp {
    Eq(Box<EqExp>),
    LAndEq(Box<LAndExp>, Box<EqExp>),
}

/// LOrExp ::= LAndExp | LOrExp "||" LAndExp
pub enum LOrExp {
    LAnd(Box<LAndExp>),
    LOrLAnd(Box<LOrExp>, Box<LAndExp>),
}

/// GlobalDecl ::= Decl
pub struct GlobalDecl(pub Decl);

/// Decl ::= ConstDecl | VarDecl
pub enum Decl {
    ConstDecl(ConstDecl),
    VarDecl(VarDecl),
}

/// ConstDecl ::= "const" BType ConstDef {"," ConstDef} ";"
pub struct ConstDecl(pub BType, pub Vec<ConstDef>);

/// BType ::= "int"
pub enum BType {
    Int, Void,
}

/// ConstDef ::= IDENT "=" ConstInitVal
pub struct ConstDef(pub String, pub ConstInitVal);

/// ConstInitVal ::= ConstExp
pub struct ConstInitVal(pub ConstExp);

/// VarDecl ::= BType VarDef {"," VarDef} ";"
pub struct VarDecl(pub BType, pub Vec<VarDef>);

/// VarDef ::= IDENT | IDENT "=" InitVal
pub enum VarDef {
    NoInit(String),
    Init(String, InitVal),
}

/// InitVal ::= Exp
pub enum InitVal {
    Exp(Exp),
    Number(Number),
}


/// BlockItem ::= Decl | Stmt
pub enum BlockItem {
    Decl(Decl),
    Stmt(Stmt),
}

/// LVal ::= IDENT
pub struct LVal(pub String);

/// ConstExp ::= Exp
pub struct ConstExp(pub Exp);
