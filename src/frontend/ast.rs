/// Program ::= CompUnit {CompUnit}
pub struct Program(pub Vec<CompUnit>);

/// CompUnit ::= FuncDef
pub enum CompUnit {
    FuncDef(FuncDef),
}

/// FuncDef ::= FuncType IDENT "(" ")" Block
pub struct FuncDef(pub FuncType, pub String, pub Block);

/// FuncType ::= "int"
pub enum FuncType {
    Int,
}

/// Block ::= "{" {BlockItem} "}"
pub struct Block(pub Vec<BlockItem>);

/// Stmt ::= LVal "=" Exp ";"
///     | "return" Exp ";"
pub enum Stmt {
    Assign(LVal, Exp),
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

/// UnaryExp ::= PrimaryExp | UnaryOp UnaryExp;
pub enum UnaryExp {
    Primary(Box<PrimaryExp>),
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

/// Decl ::= ConstDecl | VarDecl
pub enum Decl {
    ConstDecl(ConstDecl),
    VarDecl(VarDecl),
}

/// ConstDecl ::= "const" BType ConstDef {"," ConstDef} ";"
pub struct ConstDecl(pub BType, pub Vec<ConstDef>);

/// BType ::= "int"
pub enum BType {
    Int,
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
pub struct InitVal(pub Exp);


/// BlockItem ::= Decl | Stmt
pub enum BlockItem {
    Decl(Decl),
    Stmt(Stmt),
}

/// LVal ::= IDENT
pub struct LVal(pub String);

/// ConstExp ::= Exp
pub struct ConstExp(pub Exp);
