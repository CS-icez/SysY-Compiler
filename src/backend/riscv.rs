use std::collections::LinkedList;

pub type Reg = String;

#[derive(Default)]
pub struct Program {
    pub global_decls: LinkedList<GlobalDecl>,
    pub funcs: LinkedList<Func>,
}

#[derive(Default)]
pub struct GlobalDecl;

#[derive(Default)]
pub struct Func {
    pub name: String,
    pub blocks: LinkedList<Block>,
}

#[derive(Default)]
pub struct Block {
    pub name: Option<String>,
    pub insts: LinkedList<Inst>,
}

#[allow(dead_code)]
pub enum Inst {
    Beqz { rs: Reg, label: String },
    Bnez { rs: Reg, label: String },
    J { label: String },
    Call { label: String },
    Ret,
    Lw { rd: Reg, imm: i32, rs: Reg },
    Sw { rs: Reg, rd: Reg, imm: i32 },
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

    fn back_func_mut(&mut self) -> &mut Func {
        self.funcs.back_mut().unwrap()
    }

    fn back_block_mut(&mut self) -> &mut Block {
        self.back_func_mut().blocks.back_mut().unwrap()
    }

    pub fn push_func(&mut self, name: &str) {
        self.funcs.push_back(Func {
            name: name.to_string(),
            ..Default::default()
        });
    }

    pub fn push_block(&mut self, name: Option<&str>) {
        let name = if let Some(name) = name {
            Some(name.to_string())
        } else {
            None
        };
        self.back_func_mut().blocks.push_back(Block {
            name,
            ..Default::default()
        });
    }

    pub fn push_inst(&mut self, inst: Inst) {
        self.back_block_mut().insts.push_back(inst);
    }
}