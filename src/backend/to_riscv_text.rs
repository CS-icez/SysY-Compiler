use super::riscv::*;

pub trait ToRiscvText {
    fn to_riscv_text(&self) -> String;
}

fn tab() -> &'static str {
    "    "
}

impl ToRiscvText for Program {
    fn to_riscv_text(&self) -> String {
        let mut res = tab().to_string() + ".text\n";
        for func in &self.funcs {
            res += &func.to_riscv_text();
        }
        res
    }
}

impl ToRiscvText for Func {
    fn to_riscv_text(&self) -> String {
        let mut res = tab().to_string()
            + ".globl " + &self.name + "\n"
            + &self.name + ":\n";
        for block in &self.blocks {
            res += &block.to_riscv_text();
        }
        res
    }
}

impl ToRiscvText for Block {
    fn to_riscv_text(&self) -> String {
        let mut res = String::new();
        if &self.name != "%entry" {
            res += &self.name;
            res += ":\n";
        }
        for inst in &self.insts {
            res += &inst.to_riscv_text();
        }
        res
    }
}

impl ToRiscvText for Inst {
    fn to_riscv_text(&self) -> String {
        use Inst::*;
        tab().to_string() + &match &self {
            Li {rd, imm} => format!("li {rd}, {imm}\n"),
            Ret => format!("ret\n"),
        }
    }
}