use std::collections::LinkedList;

pub type Reg = &'static str;

#[derive(Default, Clone)]
pub struct Program {
    pub global_decls: LinkedList<GlobalDecl>,
    pub funcs: LinkedList<Func>,
}

#[derive(Default, Clone)]
pub struct GlobalDecl;

#[derive(Default, Clone)]
pub struct Func {
    pub name: String,
    pub blocks: LinkedList<Block>,
}

#[derive(Default, Clone)]
pub struct Block {
    pub name: String,
    pub insts: LinkedList<Inst>,
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Inst {
    Beqz { rs: Reg, label: String },
    Bnez { rs: Reg, label: String },
    J { label: String },
    Call { label: String },
    Ret,
    Lw { rd: Reg, imm12: i32, rs: Reg },
    Sw { rs: Reg, imm12: i32, rd: Reg },
    Add { rd: Reg, rs1: Reg, rs2: Reg },
    Addi { rd: Reg, rs: Reg, imm12: i32 },
    Sub { rd: Reg, rs1: Reg, rs2: Reg },
    Slt { rd: Reg, rs1: Reg, rs2: Reg },
    Sgt { rd: Reg, rs1: Reg, rs2: Reg },
    Seqz { rd: Reg, rs: Reg },
    Snez { rd: Reg, rs: Reg },
    Xor { rd: Reg, rs1: Reg, rs2: Reg },
    Xori { rd: Reg, rs: Reg, imm12: i32 },
    Or { rd: Reg, rs1: Reg, rs2: Reg },
    Ori { rd: Reg, rs: Reg, imm12: i32 },
    And { rd: Reg, rs1: Reg, rs2: Reg },
    Andi { rd: Reg, rs: Reg, imm12: i32 },
    Sll { rd: Reg, rs1: Reg, rs2: Reg },
    Srl { rd: Reg, rs: Reg, rs2: Reg },
    Sra { rd: Reg, rs: Reg, rs2: Reg },
    Mul { rd: Reg, rs1: Reg, rs2: Reg },
    Div { rd: Reg, rs1: Reg, rs2: Reg },
    Rem { rd: Reg, rs1: Reg, rs2: Reg },
    Li { rd: Reg, imm: i32 },
    La { rd: Reg, label: String },
    Mv { rd: Reg, rs: Reg },
}

impl Program {
    pub const fn new() -> Self {
        Self {
            global_decls: LinkedList::new(),
            funcs: LinkedList::new(),
        }
    }
}