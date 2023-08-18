use koopa::ir::entities;
use koopa::ir::dfg::DataFlowGraph;
use super::RiscvBuilder;
use super::riscv::{Inst::*, Reg};

fn is_imm12(imm: i32) -> bool {
    imm >= -2048 && imm <= 2047
}

#[allow(non_upper_case_globals)]
const t0: Reg = "t0";

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

    pub fn query_or_build(
        &mut self,
        value: entities::Value,
        dfg: &DataFlowGraph,
    ) -> Reg {
        use entities::ValueKind::*;
        let value_data = dfg.value(value);
        // println!("call query_or_build: {value_data:#?}");
        if let Integer(_) = value_data.kind() {
            self.build_inst(value, dfg, None).unwrap()
        } else {
            self.query_inst(value)
        }
    }
}
