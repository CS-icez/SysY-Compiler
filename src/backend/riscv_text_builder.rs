//! RISC-V text builder.

mod build_from;

use super::riscv::Program;
use build_from::BuildFrom;

pub struct RiscvTextBuilder {
    text: String,
}

impl RiscvTextBuilder {
    const TAB: &str = "    ";

    /// Builds RISCV text from given RISCV program.
    pub fn build(prog: &Program) -> String {
        let mut builder = Self::new();
        builder.build_from(prog);
        builder.text
    }

    /// Creates a new RISCV text builder.
    fn new() -> Self {
        Self {
            text: String::new(),
        }
    }
}
