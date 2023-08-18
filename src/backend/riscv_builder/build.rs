use koopa::ir::entities;
use koopa::ir::layout;
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
    pub fn build_prog(&mut self, prog: &entities::Program) {
        for &handle in prog.func_layout() {
            let func = prog.func(handle);
            self.build_func(func);
        }
    }

    pub fn build_func(&mut self, func: &entities::FunctionData) {
        self.build_func_meta(func);

        let func_name = &func.name()[1..];
        self.push_func(func_name);
        
        let size = self.frame_size() as i32;

        let bbs = func.layout().bbs();
        let mut entry = true;
        for (&bb, node) in bbs {
            if entry {
                let name = func.name();
                self.push_block(&name[1..]);
                self.build_addi("sp", "sp", -size);
                entry = false;
            } else {
                let name = func.dfg().bb(bb).name().as_ref().unwrap();
                self.push_block(&name[1..]);
            }
            self.build_block(node, func.dfg());
        }
    }

    pub fn build_block(
        &mut self,
        node: &layout::BasicBlockNode,
        dfg: &DataFlowGraph
    ) {
        for &value in node.insts().keys() {
            // println!("build_block call: {value:?}");
            self.build_inst(value, dfg, None);
        }
    }

    pub fn build_inst(
        &mut self,
        value: entities::Value,
        dfg: &DataFlowGraph,
        dst: Option<Reg>,
    ) -> Option<Reg> {
        use entities::ValueKind::*;
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
                let ident = dfg.value(load.src()).name().as_ref().unwrap();
                let imm = self.offset(ident) as i32;
                self.build_lw(&rd, imm, "sp");
                Some(rd)
            }

            Store(store) => {
                let rs = self.query_or_build(store.value(), dfg);
                let ident = dfg.value(store.dest()).name().as_ref().unwrap();
                let imm = self.offset(ident) as i32;
                self.build_sw(&rs, imm, "sp");
                self.free_reg(store.value());
                None
            }

            Binary(binary) => {
                // println!("call build_inst: {binary:#?}");
                let lhs = self.query_or_build(binary.lhs(), dfg);
                let rhs = self.query_or_build(binary.rhs(), dfg);
                let rd = self.alloc_reg(value, dst);
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
                self.free_reg(binary.lhs());
                self.free_reg(binary.rhs());
                Some(rhs)
            }

            Branch(branch) => {
                let cond = self.query_or_build(branch.cond(), dfg);
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
                self.free_reg(branch.cond());
                None
            }

            Jump(jump) => {
                let name = dfg.bb(jump.target()).name().as_ref().unwrap();
                self.push_inst(Inst::J {
                    label: name[1..].to_string(),
                });
                None
            }

            Return(ret) => {
                if let Some(value) = ret.value() {
                    let a0 = "a0";
                    // assert!(dst == None || dst.as_ref().unwrap() == &a0);
                    let rs = self.query_or_build(value, dfg);
                    if rs != a0 {
                        self.push_inst(Inst::Mv { rd: a0, rs  });
                    }
                }
                self.build_addi("sp", "sp", self.frame_size() as i32);
                self.push_inst(Inst::Ret);
                None
            }
            _ => unreachable!(),
        }
    }
}
