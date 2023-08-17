use koopa::ir::entities;
use reg_manager::RegManager;
use super::riscv;
use crate::utils::token_generator::TokenGenerator;

mod build;
mod reg_manager;
mod riscv_helpers;

#[derive(Default)]
pub struct RiscvBuilder {
    prog: riscv::Program,
    reg_mgr: RegManager,
    token_gen: TokenGenerator,
}

impl RiscvBuilder {
    pub fn new() -> Self {
        Self {
            prog: riscv::Program::new(),
            reg_mgr: RegManager::new(),
            token_gen: TokenGenerator::new("l"),
        }
    }

    pub fn build(&mut self, prog: &entities::Program) -> riscv::Program {
        self.build_prog(prog);
        // Huge overhead here, yet don't know which syntax to use to avoid it.
        let res = self.prog.clone();
        *self = Default::default();
        res
    }

    fn make_token(&self) -> String {
        self.token_gen.generate()
    }

    fn alloc_reg(&self, reg: Option<String>) -> String {
        self.reg_mgr.alloc(reg)
    }

    fn free_reg(&self, reg: &str) {
        self.reg_mgr.free(reg);
    }

    fn back_func_mut(&mut self) -> &mut riscv::Func {
        self.prog.funcs.back_mut().unwrap()
    }

    fn back_block_mut(&mut self) -> &mut riscv::Block {
        self.back_func_mut().blocks.back_mut().unwrap()
    }

    pub fn push_func(&mut self, name: &str) {
        self.prog.funcs.push_back(riscv::Func {
            name: name.to_string(),
            ..Default::default()
        });
    }

    pub fn push_block(&mut self, name: &str) {
        self.back_func_mut().blocks.push_back(riscv::Block {
            name: name.to_string(),
            ..Default::default()
        });
    }

    pub fn push_inst(&mut self, inst: riscv::Inst) {
        self.back_block_mut().insts.push_back(inst);
    }
}