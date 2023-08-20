use koopa::ir::entities::*;
use koopa::ir::dfg::DataFlowGraph;
use super::RiscvBuilder;
use super::riscv::{Inst::*, Reg};

fn is_imm12(imm: i32) -> bool {
    imm >= -2048 && imm <= 2047
}

#[allow(non_upper_case_globals)]
const t0: Reg = "t0";
const REGS: [&str; 8] = ["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"];

impl RiscvBuilder {
    pub fn build_lw(&mut self, rd: Reg, imm: i32, rs: Reg) {
        if is_imm12(imm) {
            self.push_inst(Lw { rd, imm12: imm, rs});
        } else {
            self.push_inst(Addi { rd: t0, rs, imm12: imm });
            self.push_inst(Lw { rd, imm12: 0, rs: t0 });
        }
    }

    pub fn build_sw(&mut self, rs: Reg, imm: i32, rd: Reg) {
        if is_imm12(imm) {
            self.push_inst(Sw { rs, imm12: imm, rd });
        } else {
            self.push_inst(Addi { rd: t0, rs: rd, imm12: imm });
            self.push_inst(Sw { rs, imm12: 0, rd: t0 });
        }
    }

    pub fn build_addi(&mut self, rd: Reg, rs: Reg, imm: i32) {
        if imm == 0 {
            if rd != rs {
                self.push_inst(Mv { rd, rs });
            }
            return;
        }
        
        if is_imm12(imm) {
            self.push_inst(Addi { rd, rs, imm12: imm });
        } else {
            self.push_inst(Li { rd: t0, imm });
            self.push_inst(Add { rd, rs1: rs, rs2: t0 });
        }
    }

    fn query_inst(&mut self, inst: Value, dst: Option<Reg>) -> Reg {
        let res = self.reg_mgr.reg(inst);
        if dst == None || dst == Some(res) {
            return res;
        } else {
            let dst = dst.unwrap();
            let need_swap = self.reg_mgr.move_to(res, dst);
            if need_swap {
                self.push_inst(Mv { rd: t0, rs: dst });
                self.push_inst(Mv { rd: dst, rs: res });
                self.push_inst(Mv { rd: res, rs: t0 });
            } else {
                self.push_inst(Mv { rd: dst, rs: res });
            }
            return dst;
        }
    }

    pub fn query_or_build(
        &mut self,
        value: Value,
        dfg: &DataFlowGraph,
        prog: &Program,
        dst: Option<Reg>,
    ) -> Reg {
        use ValueKind::*;
        let value_data = dfg.value(value);
        // println!("call query_or_build: {value_data:#?}");
        if let Integer(_) = value_data.kind() {
            self.build_inst(value, dfg, prog, dst).unwrap()
        } else if let FuncArgRef(arg_ref) = value_data.kind() {
            let idx = arg_ref.index() as u32;
            if idx < 8 {
                REGS[idx as usize]
            } else {
                let imm = (idx - 8) * 4 + self.frame_size();
                let dst = self.alloc_reg(value, None);
                self.build_lw(dst, imm as i32, "sp");
                dst
            }
        } else {
            self.query_inst(value, dst)
        }
    }

    fn lw_or_build(&mut self, value: Value, dfg: &DataFlowGraph, dst: Reg) {
        use ValueKind::*;
        let value_data = dfg.value(value);
        if let Integer(int) = value_data.kind() {
            self.push_inst(Li { rd: dst, imm: int.value() });
        } else {
            let imm = self.offset(value);
            self.build_lw(dst, imm as i32, "sp");
        }
    }

    pub fn save_regs(&mut self) {
        let regs = self.reg_mgr.regs();
        let size = 4 * regs.len() as i32;
        let off = self.arg_offset() as i32;
        self.build_addi("sp", "sp", -size);
        for (i, reg) in regs.iter().enumerate() {
            self.build_sw(reg, (i as i32) * 4 + off, "sp");
        }
    }

    pub fn restore_regs(&mut self) {
        let regs = self.reg_mgr.regs();
        let size = 4 * regs.len() as i32;
        let off = self.arg_offset() as i32;
        for (i, reg) in regs.iter().enumerate() {
            self.build_lw(reg, (i as i32) * 4 + off, "sp");
        }
        self.build_addi("sp", "sp", size);
    }

    pub fn pass_args(&mut self, args: &[Value], dfg: &DataFlowGraph) {
        for (i, &arg) in args.iter().enumerate() {
            if i < 8 {
                // First 8 arguments go to a0-a7.
                self.lw_or_build(arg, dfg, REGS[i]);
            } else {
                // The rest go to stack.
                self.lw_or_build(arg, dfg, t0);
                self.build_sw(t0, (i - 8) as i32 * 4, "sp");
            }
        }
    }

    #[allow(dead_code)]
    pub fn choose_dst(&self, value: Value, dfg: &DataFlowGraph) -> Option<Reg> {
        use ValueKind::*;
        let value_data = dfg.value(value);
        let used_by = value_data.used_by();
        if used_by.len() > 1 {
            assert!(matches!(value_data.kind(), Alloc(_)));
            return None;
        } else if used_by.len() == 0 {
            return None;
        } else {
            let &user = used_by.iter().next().unwrap();
            let user_data = dfg.value(user);
            match user_data.kind() {
                // `load` only uses result of `alloc`.
                Load(_) => None,
                // These instructions don't have dedicated registers.
                Store(_) | Binary(_) | Branch(_) => None,
                // `call` requires the first 8 args stored in `a0` to `a7`.
                Call(call) => {
                    let idx = call.args()
                        .iter()
                        .position(|&arg| arg == value)
                        .unwrap();
                    if idx < 8 {
                        Some(REGS[idx])
                    } else {
                        None
                    }
                }
                // `return` requires return value stored in `a0`.
                Return(_) => Some("a0"),
                // The rest instructions don't use results of others.
                _ => panic!("Unexpected user: {user_data:#?}"),
            }
        }
    }
}
