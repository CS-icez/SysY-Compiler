/// CompUnit ::= FuncDef
pub type CompUnit = FuncDef;

/// FuncDef ::= FuncType IDENT "(" ")" Block
#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}

/// FuncType ::= "int"
#[derive(Debug)]
pub enum FuncType {
    Int,
}

/// Block ::= "{" Stmt "}"
#[derive(Debug)]
pub struct Block {
    pub stmt: Stmt,
}

/// Stmt ::= "return" Number ";"
/// Number ::= INT_CONST
#[derive(Debug)]
pub struct Stmt {
    pub number: i32,
}
