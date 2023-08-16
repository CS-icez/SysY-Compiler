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

/// Exp ::= AddExp
pub struct Exp {
    pub add_exp: LOrExp,
}

/// PrimaryExp ::= "(" Exp ")" | Number
pub enum PrimaryExp {
    BracketedExp { bexp: Box<Exp> },
    Num { number: Number },
}

/// UnaryExp ::= PrimaryExp | UnaryOp UnaryExp;
pub enum UnaryExp {
    Primary { primary_bexp: Box<PrimaryExp> },
    OpUnary { op: UnaryOp, unary_bexp: Box<UnaryExp> },
}

/// UnaryOp ::= "+" | "-" | "!"
pub enum UnaryOp {
    Plus, Minus, Not,
}

/// MulExp ::= UnaryExp | MulExp MulOp UnaryExp
pub enum MulExp {
    Unary { unary_bexp: Box<UnaryExp> },
    MulOpUnary { bexp: Box<MulExp>, op: MulOp, unary_bexp: Box<UnaryExp> },
}

/// MulOp ::= "*" | "/" | "%"
pub enum MulOp {
    Mul, Div, Rem,
}

/// AddExp ::= MulExp | AddExp AddOp MulExp
pub enum AddExp {
    Mul { mul_bexp: Box<MulExp> },
    AddOpMul { bexp: Box<AddExp>, op: AddOp, mul_bexp: Box<MulExp> },
}

/// AddOp ::= "+" | "-"
pub enum AddOp {
    Add, Sub,
}

/// RelExp ::= AddExp | RelExp RelOp AddExp
pub enum RelExp {
    Add { add_bexp: Box<AddExp> },
    RelOpAdd { bexp: Box<RelExp>, op: RelOp, add_bexp: Box<AddExp> },
}

/// RelOp ::= "<=" | "<" | ">=" | ">"
pub enum RelOp {
    Le, Lt, Ge, Gt,
}

/// EqExp ::= RelExp | EqExp EqOp RelExp
pub enum EqExp {
    Rel { rel_bexp: Box<RelExp> },
    EqOpRel { bexp: Box<EqExp>, op: EqOp, rel_bexp: Box<RelExp> },
}

/// EqOp ::= "==" | "!="
pub enum EqOp {
    Eq, Ne,
}

/// LAndExp ::= EqExp | LAndExp "&&" EqExp
pub enum LAndExp {
    Eq { eq_bexp: Box<EqExp> },
    LAndEq { bexp: Box<LAndExp>, eq_bexp: Box<EqExp> },
}

/// LOrExp ::= LAndExp | LOrExp "||" LAndExp
pub enum LOrExp {
    LAnd { land_bexp: Box<LAndExp> },
    LOrLAnd { bexp: Box<LOrExp>, land_bexp: Box<LAndExp> },
}
