use koopa::ir::entities;
use koopa::ir::layout;
use koopa::ir::dfg::DataFlowGraph;
use crate::backend::riscv::Inst;
use super::RiscvBuilder;

macro_rules! push_inst {
    ($self:tt, Inst::$T:ident, Binary, $rhs:expr, $lhs:expr) => {
        $self.push_inst(Inst::$T {
            rd: $rhs.clone(),
            rs1: $lhs.clone(),
            rs2: $rhs.clone(),
        })
    };
    ($self:tt, Inst::$T:ident, BinaryImm, $rs:expr, $imm12:literal) => {
        $self.push_inst(Inst::$T {
            rd: $rs.clone(),
            rs: $rs.clone(),
            imm12: $imm12,
        })
    };
    ($self:tt, Inst::$T:ident, Unary, $rd:expr, $rs:expr) => {
        $self.push_inst(Inst::$T {
            rd: $rd.clone(),
            rs: $rs.clone(),
        })
    };
}

impl RiscvBuilder {
    pub fn build_prog(&mut self, prog: &entities::Program) {
        for &func in prog.func_layout() {
            let func = prog.func(func);
            self.build_func(func);
        }
    }

    pub fn build_func(&mut self, func: &entities::FunctionData) {
        let func_name = &func.name()[1..];
        self.push_func(func_name);
        let bbs = func.layout().bbs();
        let mut entry = true;
        for (_, node) in bbs {
            let temp;
            let name = if entry {
                func_name
            } else {
                temp = self.make_token();
                &temp
            };
            self.push_block(name);
            entry = false;
            self.build_block(node, func.dfg());
        }
    }

    pub fn build_block(
        &mut self,
        node: &layout::BasicBlockNode,
        dfg: &DataFlowGraph
    ) {
        use entities::ValueKind::*;
        for &inst in node.insts().keys() {
            let value = dfg.value(inst);
            if let Return(_) = value.kind() {
                let res = self.build_inst(value, dfg, None);
                if let Some(res) = res {
                    self.free_reg(&res);
                }
            }
        }
    }

    pub fn build_inst(
        &mut self,
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
                let rd = self.alloc_reg(dst.clone());
                let item = Inst::Li { rd: rd.clone(), imm: int.value() };
                self.push_inst(item);
                Some(rd)
            }
            Return(ret) => {
                if let Some(value) = ret.value() {
                    let value = dfg.value(value);
                    let a0 = "a0".to_string();
                    assert!(dst == None || dst.as_ref().unwrap() == &a0);
                    self.build_inst(value, dfg, Some(a0.clone()));
                    self.push_inst(Inst::Ret);
                    Some(a0)
                } else {
                    assert_eq!(dst, None);
                    self.push_inst(Inst::Ret);
                    None
                }
            }
            Binary(binary) => {
                let lhs = self.build_inst(dfg.value(binary.lhs()), dfg, None).unwrap();
                let rhs = self.build_inst(dfg.value(binary.rhs()), dfg, dst).unwrap();
                match binary.op() {
                    NotEq => {
                        if lhs != "x0" {
                            push_inst!(self, Inst::Xor, Binary, rhs, lhs);
                        }
                        push_inst!(self, Inst::Seqz, Unary, rhs, rhs);
                    }
                    Eq => {
                        if lhs != "x0" {
                            push_inst!(self, Inst::Xor, Binary, rhs, lhs);
                        }
                        push_inst!(self, Inst::Snez, Unary, rhs, rhs);
                    }
                    Gt => push_inst!(self, Inst::Sgt, Binary, rhs, lhs),
                    Lt => push_inst!(self, Inst::Slt, Binary, rhs, lhs),
                    Ge => {
                        push_inst!(self, Inst::Sgt, Binary, rhs, lhs);
                        push_inst!(self, Inst::Xori, BinaryImm, rhs, 1);
                    }
                    Le => {
                        push_inst!(self, Inst::Slt, Binary, rhs, lhs);
                        push_inst!(self, Inst::Xori, BinaryImm, rhs, 1);
                    }
                    Add => push_inst!(self, Inst::Add, Binary, rhs, lhs),
                    Sub => push_inst!(self, Inst::Sub, Binary, rhs, lhs),
                    Mul => push_inst!(self, Inst::Mul, Binary, rhs, lhs),
                    Div => push_inst!(self, Inst::Div, Binary, rhs, lhs),
                    Mod => push_inst!(self, Inst::Rem, Binary, rhs, lhs),
                    And => push_inst!(self, Inst::And, Binary, rhs, lhs),
                    Or => push_inst!(self, Inst::Or, Binary, rhs, lhs),
                    Xor => push_inst!(self, Inst::Xor, Binary, rhs, lhs),
                    _ => unreachable!()
                }
                self.free_reg(&lhs);
                Some(rhs)
            }
            _ => unreachable!(),
        }
    }
}
