use koopa::ir::entities;
use generate_text::GenerateText;

mod riscv;
mod build_from_koopa;
mod generate_text;
mod reg_manager;

#[allow(non_upper_case_globals)]
static mut riscv_prog: riscv::Program = riscv::Program::new();
#[allow(non_upper_case_globals)]
static mut riscv_text: String = String::new();

pub fn riscv_text_from(prog: &entities::Program) -> String {
    riscv::Program::build_from_koopa(prog);
    unsafe { riscv_prog.generate_text(); }
    unsafe { riscv_prog = Default::default(); }
    let mut res = String::new();
    unsafe { std::mem::swap(&mut riscv_text, &mut res) };
    res
}