use koopa::ir::entities::*;
use koopa::ir::layout::*;
use koopa::ir::dfg::DataFlowGraph;
use crate::backend::riscv::{Inst, Reg};
use super::RiscvBuilder;

macro_rules! push_inst {
    ($self:tt, Inst::$T:ident, Binary, $rd:expr, $rhs:expr, $lhs:expr) => {
        $self.push_inst(Inst::$T {
            rd: $rd, rs1: $lhs, rs2: $rhs,
        })
    };
    ($self:tt, Inst::$T:ident, BinaryImm, $rd:expr, $rs:expr, $imm12:literal) => {
        $self.push_inst(Inst::$T {
            rd: $rd, rs: $rs, imm12: $imm12,
        })
    };
    ($self:tt, Inst::$T:ident, Unary, $rd:expr, $rs:expr) => {
        $self.push_inst(Inst::$T {
            rd: $rd, rs: $rs,
        })
    };
}

impl RiscvBuilder {
    pub fn build_prog(&mut self, prog: &Program) {
        for &handle in prog.inst_layout() {
            let data = prog.borrow_value(handle);
            self.build_data(&data, prog);
        }
        for &handle in prog.func_layout() {
            let func = prog.func(handle);
            let name = &func.name()[1..];
            self.record_func_name(handle, name);
            self.build_func(func, prog);
        }
    }

    pub fn build_data(&mut self, data: &ValueData, prog: &Program) {
        use ValueKind::*;
        let name = data.name().as_ref().unwrap();
        let name = &name[1..];

        let GlobalAlloc(alloc) = data.kind() else {
            panic!("Unexpected data kind");
        };

        let init = match prog.borrow_value(alloc.init()).kind() {
            Integer(int) => int.value(),
            ZeroInit(..) => 0,
            _ => panic!("Invalid init value"),
        };

        self.push_global_def(name, init);
    }

    pub fn build_func(&mut self, func: &FunctionData, prog: &Program) {
        // Ignore function declaration.
        if func.layout().entry_bb() == None {
            return;
        }

        self.reset_reg();
        self.build_func_meta(func);

        let func_name = &func.name()[1..];
        self.push_func(func_name);

        const REGS: [Reg; 8] = ["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"];
        for (i, &value) in func.params().iter().take(8).enumerate() {
            self.alloc_reg(value, Some(REGS[i]));
        }
        
        let size = self.frame_size() as i32;

        let bbs = func.layout().bbs();
        let mut entry = true;
        for (&bb, node) in bbs {
            if entry {
                let name = func.name();
                self.push_block(&name[1..]);
                self.build_addi("sp", "sp", -size);
                if !self.is_leaf() {
                    self.build_sw("ra", size - 4, "sp");
                }
                entry = false;
                self.build_block(node, func.dfg(), prog);
            } else {
                let name = func.dfg().bb(bb).name().as_ref().unwrap();
                self.push_block(&name[1..]);
                self.build_block(node, func.dfg(), prog);
            }
        }
    }

    fn is_func_arg(&self, value: Value, dfg: &DataFlowGraph) -> bool {
        let used_by = dfg.value(value).used_by();
        if used_by.len() != 1 {
            return false;
        }
        let &user = used_by.iter().next().unwrap();
        let user_kind = dfg.value(user).kind();
        matches!(user_kind, ValueKind::Call(_))
    }

    pub fn build_block(&mut self, node: &BasicBlockNode, dfg: &DataFlowGraph, prog: &Program) {
        for &value in node.insts().keys() {
            // let dst = self.choose_dst(value, dfg);
            let reg = self.build_inst(value, dfg, prog, None);
            if self.is_func_arg(value, dfg) {
                let reg = reg.unwrap();
                let imm = self.offset(value);
                self.build_sw(reg, imm as i32, "sp");
                self.free_reg(value, reg);
            }
        }
    }

    pub fn build_inst(
        &mut self,
        value: Value,
        dfg: &DataFlowGraph,
        prog: &Program,
        dst: Option<Reg>,
    ) -> Option<Reg> {
        use ValueKind::*;
        use koopa::ir::BinaryOp::*;

        // println!("build_inst: {value:?}");
        let value_data = dfg.value(value);
        // println!("{value_data:#?}");
        match value_data.kind() {
            Integer(int) => {
                if dst == None && int.value() == 0 {
                    return Some("x0");
                }
                let rd = self.alloc_reg(value, dst);
                let item = Inst::Li { rd, imm: int.value() };
                self.push_inst(item);
                Some(rd)
            }

            Alloc(_) => {
                None
            }

            Load(load) => {
                let rd = self.alloc_reg(value, dst);
                let src = load.src();
                if dfg.values().contains_key(&src) {
                    let imm = self.offset(src) as i32;
                    self.build_lw(rd, imm, "sp");
                } else {
                    let src = prog.borrow_value(src);
                    let name = src.name().as_ref().unwrap();
                    let name = &name[1..];
                    self.push_inst(Inst::La { rd, label: name.to_string() });
                    self.build_lw(rd, 0, rd);
                }
                Some(rd)
            }

            Store(store) => {
                let rs = self.query_or_build(store.value(), dfg, prog, None);
                let dst = store.dest();
                if dfg.values().contains_key(&dst) {
                    let imm = self.offset(store.dest()) as i32;
                    self.build_sw(&rs, imm, "sp");
                    self.free_reg(store.value(), rs);
                } else {
                    let dest = prog.borrow_value(store.dest());
                    let name = dest.name().as_ref().unwrap();
                    let name = &name[1..];
                    self.push_inst(Inst::La { rd: "t0", label: name.to_string() });
                    self.build_sw(rs, 0, "t0");
                    self.free_reg(store.value(), rs);
                }
                None
            }

            Binary(binary) => {
                let lhs = self.query_or_build(binary.lhs(), dfg, prog, None);
                let rhs = self.query_or_build(binary.rhs(), dfg, prog, None);
                let rd = self.alloc_reg(value, dst);
                // println!("lhs: {lhs:?}, rhs: {rhs:?}, rd: {rd:?}\n");
                match binary.op() {
                    NotEq => {
                        if lhs != "x0" {
                            push_inst!(self, Inst::Xor, Binary, rd, rhs, lhs);
                            push_inst!(self, Inst::Snez, Unary, rd, rd);
                        } else {
                            push_inst!(self, Inst::Snez, Unary, rd, rhs);
                        }
                    }
                    Eq => {
                        if lhs != "x0" {
                            push_inst!(self, Inst::Xor, Binary, rd, rhs, lhs);
                            push_inst!(self, Inst::Seqz, Unary, rd, rd);
                        } else {
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
                    _ => unreachable!()
                }
                self.free_reg(binary.lhs(), lhs);
                self.free_reg(binary.rhs(), rhs);
                Some(rd)
            }

            Branch(branch) => {
                let cond = self.query_or_build(branch.cond(), dfg, prog, None);
                let get_name = |bb| {
                    let name = dfg.bb(bb).name().as_ref().unwrap();
                    name[1..].to_string()
                };
                let true_bb_name = get_name(branch.true_bb());
                let false_bb_name = get_name(branch.false_bb());
                self.push_inst(Inst::Beqz {
                    rs: cond,
                    label: false_bb_name,
                });
                self.push_inst(Inst::J {
                    label: true_bb_name,
                });
                self.free_reg(branch.cond(), cond);
                None
            }

            Jump(jump) => {
                let name = dfg.bb(jump.target()).name().as_ref().unwrap();
                self.push_inst(Inst::J {
                    label: name[1..].to_string(),
                });
                None
            }

            Call(call) => {
                self.pass_args(call.args(), dfg);
                self.save_regs();
                let label = self.func_name(call.callee());
                self.push_inst(Inst::Call { label: label.to_string() });
                self.restore_regs();
                if !value_data.used_by().is_empty() {
                    let rd = self.alloc_reg(value, None);
                    self.push_inst(Inst::Mv { rd, rs: "a0" });
                    Some(rd)
                    // let a0 = self.alloc_reg(value, Some("a0"));
                    // let rd = self.choose_dst(value, dfg);
                    // if rd != Some("a0") {
                    //     self.free_reg(value, a0);
                    //     let rd = self.alloc_reg(value, rd);
                    //     self.push_inst(Inst::Mv { rd, rs: "a0" });
                    //     Some(rd)
                    // } else {
                    //     rd
                    // }
                } else {
                    None
                }
            }

            Return(ret) => {
                if let Some(value) = ret.value() {
                    let rs = self.query_or_build(value, dfg, prog, None);
                    self.push_inst(Inst::Mv { rd: "a0", rs });
                }
                if !self.is_leaf() {
                    self.build_lw("ra", self.frame_size() as i32 - 4, "sp");
                }
                self.build_addi("sp", "sp", self.frame_size() as i32);
                self.push_inst(Inst::Ret);
                // if let Some(value) = ret.value() {
                //     self.free_reg(value, "a0");
                // }
                None
            }
            _ => panic!("Unexpected value kind {:?}", value_data.kind()),
        }
    }
}
