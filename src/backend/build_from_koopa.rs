use koopa::ir::entities;
use koopa::ir::layout;
use koopa::ir::dfg::DataFlowGraph;
use lazy_static::lazy_static;
use super::riscv;
use super::reg_manager::RegManager;
use super::riscv_prog;
use crate::backend::riscv::Inst;
use crate::token_generator::TokenGenerator;

mod riscv_helpers;

lazy_static!{
    static ref REG_MGR: RegManager = RegManager::new();
    static ref TOKEN_GEN: TokenGenerator = TokenGenerator::new("label");
}

macro_rules! push_inst {
    (Inst::$T:ident, PushBinary, $rhs:expr, $lhs:expr) => {
        unsafe {
            riscv_prog.push_inst(Inst::$T {
                rd: $rhs.clone(),
                rs1: $lhs.clone(),
                rs2: $rhs.clone(),
            });
        }
    };
    (Inst::$T:ident, PushBinaryImm, $rs:expr, $imm12:literal) => {
        unsafe {
            riscv_prog.push_inst(Inst::$T {
                rd: $rs.clone(),
                rs: $rs.clone(),
                imm12: $imm12,
            });
        }
    };
    (Inst::$T:ident, PushUnary, $rd:expr, $rs:expr) => {
        unsafe {
            riscv_prog.push_inst(Inst::$T {
                rd: $rd.clone(),
                rs: $rs.clone(),
            });
        }
    };
}

impl riscv::Program {
    pub fn build_from_koopa(prog: &entities::Program) {
        for &func in prog.func_layout() {
            let func = prog.func(func);
            unsafe { riscv_prog.push_func(&func.name()[1..]); }
            riscv::Func::build_from_koopa(func);
        }
    }
}

impl riscv::Func {
    fn build_from_koopa(func: &entities::FunctionData) {
        let bbs = func.layout().bbs();
        let mut entry = true;
        for (_, node) in bbs {
            let temp;
            let name = if entry {
                None
            } else {
                temp = TOKEN_GEN.generate();
                Some(&temp[..])
            };
            unsafe { riscv_prog.push_block(name); }
            entry = false;
            riscv::Block::build_from_koopa(node, func.dfg());
        }
    }
}

impl riscv::Block {
    fn build_from_koopa(node: &layout::BasicBlockNode, dfg: &DataFlowGraph) {
        use entities::ValueKind::Return;
        for &inst in node.insts().keys() {
            let value = dfg.value(inst);
            if let Return(_) = value.kind() {
                let res = riscv::Inst::build_from_koopa(value, dfg, None);
                if let Some(res) = res {
                    REG_MGR.free(&res);
                }
            }
        }
    }
}

impl riscv::Inst {
    fn build_from_koopa(
        value: &entities::ValueData,
        dfg: &DataFlowGraph,
        dst: Option<String>,
    ) -> Option<String> {
        use entities::ValueKind::*;
        use koopa::ir::BinaryOp::*;

        match value.kind() {
            Integer(int) => {
                if dst == None && int.value() == 0 {
                    return Some("x0".to_string());
                }
                let rd = REG_MGR.alloc(dst);
                let item = Inst::Li { rd: rd.clone(), imm: int.value() };
                unsafe { riscv_prog.push_inst(item); }
                Some(rd)
            }
            Return(ret) => {
                if let Some(value) = ret.value() {
                    let value = dfg.value(value);
                    let a0 = "a0".to_string();
                    assert!(dst == None || &dst.unwrap() == &a0);
                    riscv::Inst::build_from_koopa(value, dfg, Some(a0.clone()));
                    unsafe { riscv_prog.push_inst(Inst::Ret) };
                    Some(a0)
                } else {
                    assert_eq!(dst, None);
                    unsafe { riscv_prog.push_inst(Inst::Ret) };
                    None
                }
            }
            Binary(binary) => {
                let lhs = riscv::Inst::build_from_koopa(
                    dfg.value(binary.lhs()), dfg, None
                ).unwrap();
                let rhs = riscv::Inst::build_from_koopa(
                    dfg.value(binary.rhs()), dfg, dst
                ).unwrap();
                match binary.op() {
                    NotEq => {
                        if lhs != "x0" {
                            push_inst!(Inst::Xor, PushBinary, rhs, lhs);
                        }
                        push_inst!(Inst::Seqz, PushUnary, rhs, rhs);
                    }
                    Eq => {
                        if lhs != "x0" {
                            push_inst!(Inst::Xor, PushBinary, rhs, lhs);
                        }
                        push_inst!(Inst::Snez, PushUnary, rhs, rhs);
                    }
                    Gt => push_inst!(Inst::Sgt, PushBinary, rhs, lhs),
                    Lt => push_inst!(Inst::Slt, PushBinary, rhs, lhs),
                    Ge => {
                        push_inst!(Inst::Sgt, PushBinary, rhs, lhs);
                        push_inst!(Inst::Xori, PushBinaryImm, rhs, 1);
                    }
                    Le => {
                        push_inst!(Inst::Slt, PushBinary, rhs, lhs);
                        push_inst!(Inst::Xori, PushBinaryImm, rhs, 1);
                    }
                    Add => push_inst!(Inst::Add, PushBinary, rhs, lhs),
                    Sub => push_inst!(Inst::Sub, PushBinary, rhs, lhs),
                    Mul => push_inst!(Inst::Mul, PushBinary, rhs, lhs),
                    Div => push_inst!(Inst::Div, PushBinary, rhs, lhs),
                    Mod => push_inst!(Inst::Rem, PushBinary, rhs, lhs),
                    And => push_inst!(Inst::And, PushBinary, rhs, lhs),
                    Or => push_inst!(Inst::Or, PushBinary, rhs, lhs),
                    Xor => push_inst!(Inst::Xor, PushBinary, rhs, lhs),
                    _ => unreachable!()
                }
                REG_MGR.free(&lhs);
                Some(rhs)
            }
            _ => unreachable!(),
        }
    }


}
