//! Build RISCV instructions from Koopa IR values.

use super::RiscvBuilder;
use crate::backend::riscv::{Inst, Reg};
use koopa::ir::entities::*;

macro_rules! push_inst {
    ($self:tt, Inst::$T:ident, Binary, $rd:expr, $rhs:expr, $lhs:expr) => {
        $self.push_inst(Inst::$T {
            rd: $rd,
            rs1: $lhs,
            rs2: $rhs,
        })
    };
    ($self:tt, Inst::$T:ident, BinaryImm, $rd:expr, $rs:expr, $imm12:literal) => {
        $self.push_inst(Inst::$T {
            rd: $rd,
            rs: $rs,
            imm12: $imm12,
        })
    };
    ($self:tt, Inst::$T:ident, Unary, $rd:expr, $rs:expr) => {
        $self.push_inst(Inst::$T { rd: $rd, rs: $rs })
    };
}

// Used when we already know which arm the value falls in.
macro_rules! to_arm {
    ($self:tt, $value:expr, $arm:tt) => {{
        if let ValueKind::$arm(res) = $self.value_kind($value) {
            res
        } else {
            panic!("Unexpected value kind");
        }}
    };
}

impl RiscvBuilder<'_> {
    pub fn build_integer(&mut self, value: Value, dst: Option<Reg>) -> Option<Reg> {
        let int = to_arm!(self, value, Integer);
        let imm = int.value();
        if dst == None && imm == 0 { // Optimization.
            return Some("x0");
        }
        let rd = self.alloc_reg(value, dst);
        self.push_inst(Inst::Li { rd, imm });
        Some(rd)
    }

    // NOTE: Specifying destination register is not supported.
    pub fn build_func_arg_ref(&mut self, value: Value) -> Option<Reg> {
        let arg_ref = to_arm!(self, value, FuncArgRef);
        let idx = arg_ref.index() as u32;
        if idx < 8 {
            Some(RiscvBuilder::ARG_REGS[idx as usize])
        } else {
            let imm = (idx - 8) * 4 + self.frame_size();
            let dst = self.alloc_reg(value, None);
            self.build_lw(dst, imm as i32, "sp");
            Some(dst)
        }
    }

    pub fn build_load(&mut self, value: Value, dst: Option<Reg>) -> Option<Reg> {
        let load = to_arm!(self, value, Load);
        let src = load.src();
        let rd = self.alloc_reg(value, dst);

        if self.is_global_var(src) {
            let label = self.global_var_name(src).to_string();
            self.push_inst(Inst::La { rd, label });
            self.build_lw(rd, 0, rd);
        } else {
            let imm = self.offset(src) as i32;
            self.build_lw(rd, imm, "sp");
        }

        Some(rd)
    }

    pub fn build_global_alloc(&mut self, value: Value) {
        let value_data = self.koopa_prog().borrow_value(value);
        let ValueKind::GlobalAlloc(alloc) = value_data.kind() else {
            panic!("Unexpected value kind");
        };

        // HACK: Have to nest `data` and `kind` in a block, or else won't compile.
        let init = {
            let data = self.global_value_data(alloc.init());
            let kind = data.kind();

            match kind {
                ValueKind::Integer(int) => int.value(),
                ValueKind::ZeroInit(..) => 0,
                _ => panic!("Invalid init value"),
            }
        };
        self.push_global_def(value, init);
    }

    pub fn build_store(&mut self, value: Value) -> Option<Reg> {
        let store = to_arm!(self, value, Store);
        let src = store.value();
        let dst = store.dest();
        let rs = self.move_inst(src, None);

        if self.is_global_var(dst) {
            self.push_inst(Inst::La {
                rd: "t0",
                label: self.global_var_name(dst).to_string(),
            });
            self.build_sw(rs, 0, "t0");
            self.free_reg(src, rs);
        } else {
            let imm = self.offset(dst) as i32;
            self.build_sw(rs, imm, "sp");
            self.free_reg(src, rs);
        }
        None
    }

    pub fn build_binary(&mut self, value: Value, dst: Option<Reg>) -> Option<Reg> {
        use koopa::ir::BinaryOp::*;

        let binary = to_arm!(self, value, Binary);
        let lhs = self.move_inst(binary.lhs(), None);
        let rhs = self.move_inst(binary.rhs(), None);
        let rd = self.alloc_reg(value, dst);
        // println!("lhs: {lhs:?}, rhs: {rhs:?}, rd: {rd:?}\n");

        match binary.op() {
            NotEq => {
                if lhs != "x0" {
                    push_inst!(self, Inst::Xor, Binary, rd, rhs, lhs);
                    push_inst!(self, Inst::Snez, Unary, rd, rd);
                } else { // Optimization.
                    push_inst!(self, Inst::Snez, Unary, rd, rhs);
                }
            }
            Eq => {
                if lhs != "x0" {
                    push_inst!(self, Inst::Xor, Binary, rd, rhs, lhs);
                    push_inst!(self, Inst::Seqz, Unary, rd, rd);
                } else { // Optimization.
                    push_inst!(self, Inst::Seqz, Unary, rd, rhs);
                }
            }
            Gt => push_inst!(self, Inst::Sgt, Binary, rd, rhs, lhs),
            Lt => push_inst!(self, Inst::Slt, Binary, rd, rhs, lhs),
            Ge => {
                push_inst!(self, Inst::Slt, Binary, rd, rhs, lhs);
                push_inst!(self, Inst::Xori, BinaryImm, rd, rd, 1);
            }
            Le => {
                push_inst!(self, Inst::Sgt, Binary, rd, rhs, lhs);
                push_inst!(self, Inst::Xori, BinaryImm, rd, rd, 1);
            }
            Add => push_inst!(self, Inst::Add, Binary, rd, rhs, lhs),
            Sub => push_inst!(self, Inst::Sub, Binary, rd, rhs, lhs),
            Mul => push_inst!(self, Inst::Mul, Binary, rd, rhs, lhs),
            Div => push_inst!(self, Inst::Div, Binary, rd, rhs, lhs),
            Mod => push_inst!(self, Inst::Rem, Binary, rd, rhs, lhs),
            And => push_inst!(self, Inst::And, Binary, rd, rhs, lhs),
            Or => push_inst!(self, Inst::Or, Binary, rd, rhs, lhs),
            Xor => push_inst!(self, Inst::Xor, Binary, rd, rhs, lhs),
            _ => unreachable!(),
        }
        self.free_reg(binary.lhs(), lhs);
        self.free_reg(binary.rhs(), rhs);
        Some(rd)
    }

    pub fn build_branch(&mut self, value: Value) -> Option<Reg> {
        let branch = to_arm!(self, value, Branch);
        let cond = self.move_inst(branch.cond(), None);
        self.push_inst(Inst::Beqz {
            rs: cond,
            label: self.block_name(branch.false_bb()).to_string(),
        });
        self.push_inst(Inst::J {
            label: self.block_name(branch.true_bb()).to_string(),
        });
        self.free_reg(branch.cond(), cond);
        None
    }

    pub fn build_jump(&mut self, value: Value) -> Option<Reg> {
        let jump = to_arm!(self, value, Jump);
        self.push_inst(Inst::J {
            label: self.block_name(jump.target()).to_string(),
        });
        None
    }

    pub fn build_call(&mut self, value: Value, dst: Option<Reg>) -> Option<Reg> {
        let call = to_arm!(self, value, Call);
        self.pass_args(call.args());
        self.save_regs();
        self.push_inst(Inst::Call {
            label: self.func_name(call.callee()).to_string(),
        });
        self.restore_regs();
        if self.is_used(value) {
            let rd = self.alloc_reg(value, dst);
            self.push_inst(Inst::Mv { rd, rs: "a0" });
            Some(rd)
        } else {
            None
        }
    }

    pub fn build_return(&mut self, value: Value) -> Option<Reg> {
        let ret = to_arm!(self, value, Return);
        if let Some(value) = ret.value() {
            self.move_inst(value, Some("a0"));
        }
        if !self.is_leaf_func() {
            self.build_lw("ra", self.frame_size() as i32 - 4, "sp");
        }
        self.build_addi("sp", "sp", self.frame_size() as i32);
        if let Some(value) = ret.value() {
            self.free_reg(value, "a0");
        }
        self.push_inst(Inst::Ret);
        None
    }
}
