//! Build RISCV instructions from Koopa IR values.

use std::collections::LinkedList;

use super::RiscvBuilder;
use crate::backend::riscv::{Inst, MemFill, Reg};
use koopa::ir::{entities::*, TypeKind};
use koopa::ir::values::Aggregate;

#[allow(non_upper_case_globals)]
const t0: Reg = "t0";

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

    pub fn build_aggregate(&mut self, agg: &Aggregate) -> LinkedList<MemFill> {
        use MemFill::*;
        use ValueKind::*;
        let mut res = LinkedList::new();

        agg.elems().iter().for_each(|&value| {
            let data = self.koopa_prog().borrow_value(value);
            let kind = data.kind();

            match kind {
                Integer(int) => res.push_back(Word(int.value())),
                Aggregate(agg) => {
                    let mut list = self.build_aggregate(agg);
                    res.append(&mut list);
                }
                _ => panic!("Invalid init value"),
            }
        });

        res
    }

    // NOTE: Specifying destination register is not supported.
    pub fn build_func_arg_ref(&mut self, value: Value) -> Option<Reg> {
        let arg_ref = to_arm!(self, value, FuncArgRef);
        let idx = arg_ref.index() as usize;
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
        } else if self.is_local_var(src) {
            let imm = self.offset(src) as i32;
            self.build_lw(rd, imm, "sp");
        } else {
            let rs = self.move_inst(src, None);
            self.build_lw(rd, 0, rs);
            self.free_reg(src, rs);
        }

        Some(rd)
    }

    pub fn build_global_alloc(&mut self, value: Value) {
        use MemFill::*;
        use ValueKind::*;
        let value_data = self.koopa_prog().borrow_value(value);
        let GlobalAlloc(alloc) = value_data.kind() else {
            panic!("Unexpected value kind");
        };

        // HACK: Have to nest `data` and `kind` in a block, or else won't compile.
        let init = {
            let data = self.koopa_prog().borrow_value(alloc.init());
            let kind = data.kind();
            let mut res = LinkedList::new();

            match kind {
                Integer(int) => res.push_back(Word(int.value())),
                ZeroInit(..) => {
                    let size = {
                        if let TypeKind::Pointer(base) = value_data.ty().kind() {
                            base.size()
                        } else {
                            panic!("Unexpected type kind");
                        }
                    };
                    res.push_back(Zero(size));
                }
                Aggregate(agg) => {
                    let mut list = self.build_aggregate(agg);
                    res.append(&mut list);
                }
                _ => panic!("Invalid init value"),
            }

            res
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
        } else if self.is_local_var(dst){
            let imm = self.offset(dst) as i32;
            self.build_sw(rs, imm, "sp");
            self.free_reg(src, rs);
        } else { // A temporary pointer.
            let rd = self.move_inst(dst, None);
            self.build_sw(rs, 0, rd);
            self.free_reg(src, rs);
            self.free_reg(dst, rd);
        }
        None
    }

    pub fn build_get_ptr(&mut self, value: Value, dst: Option<Reg>) -> Option<Reg> {
        let gp = to_arm!(self, value, GetPtr);
        let src = gp.src();
        let src_ty_kind = {
            if src.is_global() {
                let data = self.koopa_prog().borrow_value(src);
                data.ty().kind().clone()
            } else {
                self.value_data(src).ty().kind().clone()
            }
        };
        let base_size = match src_ty_kind {
            TypeKind::Array(base, _) => base.size(),
            TypeKind::Pointer(base) => base.size(),
            _ => panic!("Unexpected type"),
        };
        let index = gp.index();
        let rd;

        if self.is_global_var(src) {
            let idx = self.move_inst(index, None);
            let off = self.alloc_reg(value, None);
            self.build_muli(off, idx, base_size as i32);
            self.free_reg(index, idx);
            self.free_reg(value, off);
            self.push_inst(Inst::La {
                rd: t0,
                label: self.global_var_name(src).to_string(),
            });
            rd = self.alloc_reg(value, dst);
            push_inst!(self, Inst::Add, Binary, rd, t0, off);
        } else if self.is_local_var(src) {
            let idx = self.move_inst(index, None);
            let off = self.alloc_reg(value, None);
            self.build_muli(off, idx, base_size as i32);
            self.free_reg(index, idx);
            self.free_reg(value, off);
            let imm = self.offset(src) as i32;
            self.build_addi(t0, "sp", imm);
            rd = self.alloc_reg(value, dst);
            push_inst!(self, Inst::Add, Binary, rd, t0, off);
        } else {
            let rs = self.move_inst(src, None);
            let idx = self.move_inst(index, None);
            let off = self.alloc_reg(value, None);
            self.build_muli(off, idx, base_size as i32);
            self.free_reg(index, idx);
            self.free_reg(value, off);
            rd = self.alloc_reg(value, dst);
            push_inst!(self, Inst::Add, Binary, rd, off, rs);
            self.free_reg(src, rs);
        }

        Some(rd)
    }

    pub fn build_get_elem_ptr(&mut self, value: Value, dst: Option<Reg>) -> Option<Reg> {
        let gep = to_arm!(self, value, GetElemPtr);
        let src = gep.src();
        let src_ty_kind = {
            if src.is_global() {
                let data = self.koopa_prog().borrow_value(src);
                data.ty().kind().clone()
            } else {
                self.value_data(src).ty().kind().clone()
            }
        };
        let base_size = if let TypeKind::Pointer(base) = src_ty_kind {
            if let TypeKind::Array(base, _) = base.kind() {
                base.size()
            } else {
                panic!("Unexpected type");
            }
        } else {
            panic!("Unexpected type");
        };
        let index = gep.index();
        let rd;

        if self.is_global_var(src) {
            let idx = self.move_inst(index, None);
            let off = self.alloc_reg(value, None);
            self.build_muli(off, idx, base_size as i32);
            self.free_reg(index, idx);
            self.free_reg(value, off);
            self.push_inst(Inst::La {
                rd: t0,
                label: self.global_var_name(src).to_string(),
            });
            rd = self.alloc_reg(value, dst);
            push_inst!(self, Inst::Add, Binary, rd, t0, off);
        } else if self.is_local_var(src) {
            let idx = self.move_inst(index, None);
            let off = self.alloc_reg(value, None);
            self.build_muli(off, idx, base_size as i32);
            self.free_reg(index, idx);
            self.free_reg(value, off);
            let imm = self.offset(src) as i32;
            self.build_addi(t0, "sp", imm);
            rd = self.alloc_reg(value, dst);
            push_inst!(self, Inst::Add, Binary, rd, t0, off);
        } else {
            let rs = self.move_inst(src, None);
            let idx = self.move_inst(index, None);
            let off = self.alloc_reg(value, None);
            self.build_muli(off, idx, base_size as i32);
            self.free_reg(index, idx);
            self.free_reg(value, off);
            rd = self.alloc_reg(value, dst);
            push_inst!(self, Inst::Add, Binary, rd, off, rs);
            self.free_reg(src, rs);
        }

        Some(rd)
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
