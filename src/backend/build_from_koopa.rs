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
                        unsafe {
                            riscv_prog.push_inst(Inst::Xor {
                                rd: rhs.clone(),
                                rs1: lhs.clone(),
                                rs2: rhs.clone()
                            });
                            riscv_prog.push_inst(Inst::Snez {
                                rd: rhs.clone(),
                                rs: rhs.clone(),
                            });
                        }
                    }
                    Eq => {
                        unsafe {
                            riscv_prog.push_inst(Inst::Xor {
                                rd: rhs.clone(),
                                rs1: lhs.clone(),
                                rs2: rhs.clone(),
                            });
                            riscv_prog.push_inst(Inst::Seqz {
                                rd: rhs.clone(),
                                rs: rhs.clone(),
                            });
                        }
                    }
                    Sub => {
                        unsafe { riscv_prog.push_inst(Inst::Sub {
                            rd: rhs.clone(),
                            rs1: lhs.clone(),
                            rs2: rhs.clone(),
                        })};
                    }
                    Xor => {
                        unsafe { riscv_prog.push_inst(Inst::Xor {
                            rd: rhs.clone(),
                            rs1: lhs.clone(),
                            rs2: rhs.clone(),
                        })};
                    }
                    _ => unreachable!()
                }
                REG_MGR.free(&lhs);
                Some(rhs)
            }
            _ => unreachable!(),
        }
    }


}
