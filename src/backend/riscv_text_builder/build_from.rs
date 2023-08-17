use super::super::riscv::*;
use super::RiscvTextBuilder;

const TAB: &str = RiscvTextBuilder::TAB;

macro_rules! push_text {
    ($self:tt, $($arg:tt)*) => {
        $self.text.push_str(&format!($($arg)*))
    };
}

pub trait BuildFrom<T> {
    fn build_from(&mut self, target: &T);
}

impl BuildFrom<Program> for RiscvTextBuilder {
    fn build_from(&mut self, prog: &Program) {
        for func in &prog.funcs {
            self.build_from(func);
        }
    }
}

impl BuildFrom<Func> for RiscvTextBuilder {
    fn build_from(&mut self, func: &Func) {
        push_text!(self, "{TAB}.globl {}\n", func.name);
        for block in &func.blocks {
            self.build_from(block);
        }
    }
}

impl BuildFrom<Block> for RiscvTextBuilder {
    fn build_from(&mut self, block: &Block) {
        push_text!(self, "{}:\n", block.name);
        for inst in &block.insts {
            self.build_from(inst);
        }
    }
}

impl BuildFrom<Inst> for RiscvTextBuilder {
    fn build_from(&mut self, inst: &Inst) {
        use Inst::*;
        let text = match inst {
            Beqz { rs, label } => format!("beqz {rs}, {label}"),
            Bnez { rs, label } => format!("bnez {rs}, {label}"),
            J { label } => format!("j {label}"),
            Call { label } => format!("call {label}"),
            Ret => format!("ret"),
            Lw { rd, imm, rs } => format!("lw {rd}, {imm}({rs})"),
            Sw { rs, rd, imm } => format!("sw {rs}, {imm}({rd})"),
            Add { rd, rs1, rs2 } => format!("add {rd}, {rs1}, {rs2}"),
            Addi { rd, rs, imm12 } => format!("addi {rd}, {rs}, {imm12}"),
            Sub { rd, rs1, rs2 } => format!("sub {rd}, {rs1}, {rs2}"),
            Slt { rd, rs1, rs2 } => format!("slt {rd}, {rs1}, {rs2}"),
            Sgt { rd, rs1, rs2 } => format!("sgt {rd}, {rs1}, {rs2}"),
            Seqz { rd, rs } => format!("seqz {rd}, {rs}"),
            Snez { rd, rs } => format!("snez {rd}, {rs}"),
            Xor { rd, rs1, rs2 } => format!("xor {rd}, {rs1}, {rs2}"),
            Xori { rd, rs, imm12 } => format!("xori {rd}, {rs}, {imm12}"),
            Or { rd, rs1, rs2 } => format!("or {rd}, {rs1}, {rs2}"),
            Ori { rd, rs, imm12 } => format!("ori {rd}, {rs}, {imm12}"),
            And { rd, rs1, rs2 } => format!("and {rd}, {rs1}, {rs2}"),
            Andi { rd, rs, imm12 } => format!("andi {rd}, {rs}, {imm12}"),
            Sll { rd, rs1, rs2 } => format!("sll {rd}, {rs1}, {rs2}"),
            Srl { rd, rs, rs2 } => format!("srl {rd}, {rs}, {rs2}"),
            Sra { rd, rs, rs2 } => format!("sra {rd}, {rs}, {rs2}"),
            Mul { rd, rs1, rs2 } => format!("mul {rd}, {rs1}, {rs2}"),
            Div { rd, rs1, rs2 } => format!("div {rd}, {rs1}, {rs2}"),
            Rem { rd, rs1, rs2 } => format!("rem {rd}, {rs1}, {rs2}"),
            Li { rd, imm } => format!("li {rd}, {imm}"),
            La { rd, label } => format!("la {rd}, {label}"),
            Mv { rd, rs } => format!("mv {rd}, {rs}"),
        };
        push_text!(self, "{TAB}{text}\n");
    }
}
