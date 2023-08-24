//! Helper functions for RISCV Builder.

use super::riscv::{Inst::*, Reg};
use super::RiscvBuilder;
use koopa::ir::dfg::DataFlowGraph;
use koopa::ir::entities::*;

fn is_imm12(imm: i32) -> bool {
    imm >= -2048 && imm <= 2047
}

#[allow(non_upper_case_globals)]
const t0: Reg = "t0";
const REGS: [&str; 8] = ["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"];

impl RiscvBuilder<'_> {
    pub fn build_lw(&mut self, rd: Reg, imm: i32, rs: Reg) {
        if is_imm12(imm) {
            self.push_inst(Lw { rd, imm12: imm, rs });
        } else {
            self.push_inst(Li { rd: t0, imm });
            self.push_inst(Add { rd: t0, rs1: rs, rs2: t0 });
            self.push_inst(Lw { rd, imm12: 0, rs: t0 });
        }
    }

    pub fn build_sw(&mut self, rs: Reg, imm: i32, rd: Reg) {
        if is_imm12(imm) {
            self.push_inst(Sw { rs, imm12: imm, rd });
        } else {
            self.push_inst(Li { rd: t0, imm });
            self.push_inst(Add { rd: t0, rs1: rd, rs2: t0 });
            self.push_inst(Sw { rs, imm12: 0, rd: t0 });
        }
    }

    pub fn build_addi(&mut self, rd: Reg, rs: Reg, imm: i32) {
        if imm == 0 { // Optimization.
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

    pub fn build_muli(&mut self, rd: Reg, rs: Reg, imm: i32) {
        self.push_inst(Li { rd: t0, imm });
        self.push_inst(Mul { rd, rs1: rs, rs2: t0 });
    }

    // TODO: coming up with a more suitable name.
    fn move_inst_to(&mut self, inst: Value, dst: Option<Reg>) -> Reg {
        let rs = self.reg_mgr.reg(inst);
        if dst == None || dst == Some(rs) { // Optimization.
            return rs;
        } else {
            let rd = dst.unwrap();
            let need_swap = self.reg_mgr.move_to(rs, rd);
            if need_swap {
                self.push_inst(Mv { rd: t0, rs: rd });
                self.push_inst(Mv { rd, rs });
                self.push_inst(Mv { rd: rs, rs: t0 });
            } else {
                self.push_inst(Mv { rd, rs });
            }
            return rd;
        }
    }

    pub fn move_inst(&mut self, value: Value, dst: Option<Reg>) -> Reg {
        use ValueKind::*;
        match self.value_kind(value) {
            Integer(..) => self.build_integer(value, dst).unwrap(),
            FuncArgRef(..) => {
                // NOTE: In current implementation, `dst` must be `None` here.
                assert_eq!(dst, None);
                self.build_func_arg_ref(value).unwrap()
            }
            // Expect the above two cases, `value` is guaranteed
            // to be built already when calling.
            _ => self.move_inst_to(value, dst),
        }
    }

    pub fn save_regs(&mut self) {
        let regs = self.reg_mgr.regs();
        let size = 4 * regs.len() as i32;

        if size ==0 {
            return;
        }

        let off = self.func_meta.arg_size() as i32;

        self.build_addi("sp", "sp", -size);
        let arg_num = off / 4;
        (0..arg_num).for_each(|i| {
            self.build_lw(t0, i as i32 * 4 + size, "sp");
            self.build_sw(t0, i as i32 * 4, "sp");
        });
        regs.iter().enumerate().for_each(|(i, reg)| {
            self.build_sw(reg, (i as i32) * 4 + off, "sp");
        });
    }

    pub fn restore_regs(&mut self) {
        let regs = self.reg_mgr.regs();
        let size = 4 * regs.len() as i32;
        let off = self.func_meta.arg_size() as i32;

        regs.iter().enumerate().for_each(|(i, reg)| {
            self.build_lw(reg, (i as i32) * 4 + off, "sp");
        });
        self.build_addi("sp", "sp", size);
    }

    pub fn pass_args(&mut self, args: &[Value]) {
        // I have to pass `self` as an argument because of E0501.
        // I don't very well understand it.
        let f = |builder: &mut RiscvBuilder, value, rd| {
            if let ValueKind::Integer(int) = builder.value_kind(value) {
                builder.push_inst(Li { rd, imm: int.value() });
            } else {
                builder.build_lw(rd, builder.offset(value) as i32, "sp");
            }
        };

        args.iter().enumerate().for_each(|(i, &arg)| {
            if i < 8 { // First 8 arguments go to a0-a7.
                f(self, arg, REGS[i]);
            } else { // The rest go to stack.
                f(self, arg, t0);
                self.build_sw(t0, (i - 8) as i32 * 4, "sp");
            }
        });
    }

    #[allow(dead_code)]
    // Maybe useful in the future.
    pub fn choose_dst(&self, value: Value, dfg: &DataFlowGraph) -> Option<Reg> {
        use ValueKind::*;
        let value_data = dfg.value(value);
        let used_by = value_data.used_by();
        if used_by.len() > 1 {
            assert!(matches!(value_data.kind(), Alloc(..)));
            return None;
        } else if used_by.len() == 0 {
            return None;
        } else {
            let &user = used_by.iter().next().unwrap();
            let user_data = dfg.value(user);
            match user_data.kind() {
                // `load` only uses result of `alloc`.
                Load(..) => None,
                // These instructions don't have dedicated registers.
                Store(..) | Binary(..) | Branch(..) => None,
                // `call` requires the first 8 args stored in `a0` to `a7`.
                Call(call) => {
                    let idx = call.args().iter().position(|&arg| arg == value).unwrap();
                    if idx < 8 {
                        Some(REGS[idx])
                    } else {
                        None
                    }
                }
                // `return` requires return value stored in `a0`.
                Return(..) => Some("a0"),
                // The rest instructions don't use results of others.
                _ => panic!("Unexpected user: {user_data:#?}"),
            }
        }
    }
}
