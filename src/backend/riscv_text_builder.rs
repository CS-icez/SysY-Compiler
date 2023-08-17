mod build_from;

use super::riscv;
use build_from::BuildFrom;

#[derive(Default)]
pub struct RiscvTextBuilder {
    text: String,
}

impl RiscvTextBuilder {
    const TAB: &str = "    ";

    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    pub fn build(&mut self, prog: &riscv::Program) -> String {
        self.build_from(prog);
        // Huge overhead here, yet don't know which syntax to use to avoid it.
        let res = self.text.clone();
        *self = Default::default();
        res
    }
}

impl riscv::Program {
    pub fn to_text(&self) -> String {
        RiscvTextBuilder::new().build(self)
    }
}