use super::{riscv::*, riscv_text};

pub trait GenerateText {
    fn generate_text(&self);
}

const TAB: &str = "    ";

impl GenerateText for Program {
    fn generate_text(&self) {
        let text = format!("{TAB}.text\n");
        unsafe { riscv_text.push_str(&text); }
        for func in &self.funcs {
            func.generate_text();
        }
    }
}

impl GenerateText for Func {
    fn generate_text(&self) {
        let text = format!("{TAB}.globl {0}\n{}:\n", &self.name);
        unsafe { riscv_text.push_str(&text); }
        for block in &self.blocks {
            block.generate_text();
        }
    }
}

impl GenerateText for Block {
    fn generate_text(&self) {
        if let Some(ref name) = self.name {
            let text = format!("{name}:\n");
            unsafe { riscv_text.push_str(&text); }
        }
        for inst in &self.insts {
            inst.generate_text();
        }
    }
}

impl GenerateText for Inst {
    fn generate_text(&self) {
        use Inst::*;
        let text = match &self {
            Beqz { rs, label } => format!("{TAB}beqz {rs}, {label}\n"),
            Bnez { rs, label } => format!("{TAB}bnez {rs}, {label}\n"),
            J { label } => format!("{TAB}j {label}\n"),
            Call { label } => format!("{TAB}call {label}\n"),
            Ret => format!("{TAB}ret\n"),
            Lw { rd, imm, rs } => format!("{TAB}lw {rd}, {imm}({rs})\n"),
            Sw { rs, rd, imm } => format!("{TAB}sw {rs}, {imm}({rd})\n"),
            Add { rd, rs1, rs2 } => format!("{TAB}add {rd}, {rs1}, {rs2}\n"),
            Addi { rd, rs, imm12 } => format!("{TAB}addi {rd}, {rs}, {imm12}\n"),
            Sub { rd, rs1, rs2 } => format!("{TAB}sub {rd}, {rs1}, {rs2}\n"),
            Slt { rd, rs1, rs2 } => format!("{TAB}slt {rd}, {rs1}, {rs2}\n"),
            Sgt { rd, rs1, rs2 } => format!("{TAB}sgt {rd}, {rs1}, {rs2}\n"),
            Seqz { rd, rs } => format!("{TAB}seqz {rd}, {rs}\n"),
            Snez { rd, rs } => format!("{TAB}snez {rd}, {rs}\n"),
            Xor { rd, rs1, rs2 } => format!("{TAB}xor {rd}, {rs1}, {rs2}\n"),
            Xori { rd, rs, imm12 } => format!("{TAB}xori {rd}, {rs}, {imm12}\n"),
            Or { rd, rs1, rs2 } => format!("{TAB}or {rd}, {rs1}, {rs2}\n"),
            Ori { rd, rs, imm12 } => format!("{TAB}ori {rd}, {rs}, {imm12}\n"),
            And { rd, rs1, rs2 } => format!("{TAB}and {rd}, {rs1}, {rs2}\n"),
            Andi { rd, rs, imm12 } => format!("{TAB}andi {rd}, {rs}, {imm12}\n"),
            Sll { rd, rs1, rs2 } => format!("{TAB}sll {rd}, {rs1}, {rs2}\n"),
            Srl { rd, rs, rs2 } => format!("{TAB}srl {rd}, {rs}, {rs2}\n"),
            Sra { rd, rs, rs2 } => format!("{TAB}sra {rd}, {rs}, {rs2}\n"),
            Mul { rd, rs1, rs2 } => format!("{TAB}mul {rd}, {rs1}, {rs2}\n"),
            Div { rd, rs1, rs2 } => format!("{TAB}div {rd}, {rs1}, {rs2}\n"),
            Rem { rd, rs1, rs2 } => format!("{TAB}rem {rd}, {rs1}, {rs2}\n"),
            Li {rd, imm} => format!("{TAB}li {rd}, {imm}\n"),
            La {rd, label} => format!("{TAB}la {rd}, {label}\n"),
            Mv {rd, rs} => format!("{TAB}mv {rd}, {rs}\n"),
        };
        unsafe { riscv_text.push_str(&text); }
    }
}