use koopa::ir::entities;
use koopa::ir::dfg::DataFlowGraph;
use super::RiscvBuilder;
use super::riscv::Inst::*;

fn is_imm12(imm: i32) -> bool {
    imm >= -2048 && imm <= 2047
}

impl RiscvBuilder {
    pub fn build_lw(&mut self, rd: &str, imm: i32, rs: &str) {
        if is_imm12(imm) {
            self.push_inst(Lw {
                rd: rd.to_string(),
                imm12: imm,
                rs: rs.to_string(),
            });
        } else {
            let temp = "t0".to_string();
            self.push_inst(Addi {
                rd: temp.clone(),
                rs: rs.to_string(),
                imm12: imm,
            });
            self.push_inst(Lw {
                rd: rd.to_string(),
                imm12: 0,
                rs: temp,
            });
        }
    }

    pub fn build_sw(&mut self, rs: &str, imm: i32, rd: &str) {
        if is_imm12(imm) {
            self.push_inst(Sw {
                rs: rs.to_string(),
                imm12: imm,
                rd: rd.to_string(),
            });
        } else {
            let temp = "t0".to_string();
            self.push_inst(Addi {
                rd: temp.clone(),
                rs: rd.to_string(),
                imm12: imm,
            });
            self.push_inst(Sw {
                rs: rs.to_string(),
                imm12: 0,
                rd: temp,
            });
        }
    }

    pub fn build_addi(&mut self, rd: &str, rs: &str, imm: i32) {
        if imm == 0 {
            if rd != rs {
                self.push_inst(Mv {
                    rd: rd.to_string(),
                    rs: rs.to_string(),
                });
            }
            return;
        }
        
        if is_imm12(imm) {
            self.push_inst(Addi {
                rd: rd.to_string(),
                rs: rs.to_string(),
                imm12: imm,
            });
        } else {
            let temp = "t0".to_string();
            self.push_inst(Li {
                rd: temp.clone(),
                imm,
            });
            self.push_inst(Add {
                rd: rd.to_string(),
                rs1: rs.to_string(),
                rs2: temp,
            });
        }
    }

    pub fn query_or_build(
        &mut self,
        value: entities::Value,
        dfg: &DataFlowGraph,
    ) -> String {
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
