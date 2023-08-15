use std::collections::LinkedList;

pub type Reg = String;

pub struct Program {
    pub global_decls: LinkedList<GlobalDecl>,
    pub funcs: LinkedList<Func>,
}

pub struct GlobalDecl;

pub struct Func {
    pub name: String,
    pub blocks: LinkedList<Block>,
}

pub struct Block {
    pub name: String,
    pub insts: LinkedList<Inst>,
}

pub enum Inst {
    Li { rd: Reg, imm: i32 },
    Ret,
}