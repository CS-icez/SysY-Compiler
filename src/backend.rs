use koopa::ir::entities;
use to_riscv_text::ToRiscvText;

mod riscv;
mod from_koopa;
mod to_riscv_text;

pub fn to_riscv_text(prog: &entities::Program) -> String {
    riscv::Program::from_koopa(prog).to_riscv_text()
}