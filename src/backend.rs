//! This module abstracts the backend of the compiler, i.e.,
//! from Koopa IR in memory, to RISCV assembly in ASCII text.

mod riscv;
mod riscv_builder;
mod riscv_text_builder;

use koopa::ir::entities::Program;
use riscv_builder::RiscvBuilder;
use riscv_text_builder::RiscvTextBuilder;

/// Converts a Koopa program to RISCV-32IM assembly.
pub fn riscv_text_from(prog: &Program) -> String {
    let prog = RiscvBuilder::build(prog);
    RiscvTextBuilder::build(&prog)
}
