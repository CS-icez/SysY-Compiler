mod riscv;
mod riscv_builder;
mod riscv_text_builder;

use koopa::ir::entities;
use riscv_builder::RiscvBuilder;

pub fn riscv_text_from(prog: &entities::Program) -> String {
    RiscvBuilder::new().build(prog).to_text()
}