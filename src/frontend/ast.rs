/// CompUnit ::= FuncDef
pub type CompUnit = FuncDef;

/// FuncDef ::= FuncType IDENT "(" ")" Block
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}

/// FuncType ::= "int"
pub enum FuncType {
    Int,
}

/// Block ::= "{" Stmt "}"
pub struct Block {
    pub stmt: Stmt,
}

/// Stmt ::= "return" Exp ";"
pub struct Stmt {
    pub exp: Exp,
}

/// Number ::= INT_CONST
pub struct Number {
    pub int_const: i32,
}

/// Exp ::= UnaryExp
pub struct Exp {
    pub unary_exp: UnaryExp,
}

/// PrimaryExp ::= "(" Exp ")" | Number
pub enum PrimaryExp {
    BracketedExp {bexp: Box<Exp> },
    Num { number: Number },
}

/// UnaryExp ::= PrimaryExp | UnaryOp UnaryExp;
pub enum UnaryExp {
    Primary { primary_bexp: Box<PrimaryExp> },
    OpExp { op: UnaryOp, bexp: Box<UnaryExp> },
}

/// UnaryOp ::= "+" | "-" | "!"
pub enum UnaryOp {
    Plus, Minus, Not,
}