use std::collections::HashMap;
use koopa::ir::entities;
use reg_manager::RegManager;
use func_meta::FuncMeta;
use super::riscv::{self, Reg};
use crate::utils::token_generator::TokenGenerator;

mod build;
mod reg_manager;
mod func_meta;
mod build_helpers;

#[derive(Default)]
pub struct RiscvBuilder {
    prog: riscv::Program,
    reg_mgr: RegManager,
    token_gen: TokenGenerator,
    func_meta: FuncMeta,
    inst2reg: HashMap<entities::Value, Reg>,
}

impl RiscvBuilder {
    pub fn new() -> Self {
        Self {
            prog: riscv::Program::new(),
            reg_mgr: RegManager::new(),
            token_gen: TokenGenerator::new("l"),
            func_meta: FuncMeta::new(),
            inst2reg: HashMap::new(),
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

    fn alloc_reg(&mut self, inst: entities::Value, reg: Option<Reg>) -> Reg {
        let reg = self.reg_mgr.alloc(reg);
        self.inst2reg.insert(inst, reg);
        reg
    }

    fn query_inst(&self, inst: entities::Value) -> Reg {
        self.inst2reg.get(&inst)
            .expect("Instruction not allocated to register")
    }

    #[allow(dead_code)]
    fn replace_reg_owner(&mut self, old_inst: entities::Value, new_inst: entities::Value) {
        let reg = *self.inst2reg.get(&old_inst).unwrap();
        self.inst2reg.remove(&old_inst);
        self.inst2reg.insert(new_inst, reg);
    }

    fn free_reg(&mut self, inst: entities::Value, reg: Reg) {
        self.reg_mgr.free(reg);
        self.inst2reg.remove(&inst);
    }

    fn build_func_meta(&mut self, func: &entities::FunctionData) {
        self.func_meta = FuncMeta::from(func);
    }

    fn frame_size(&self) -> u32 {
        self.func_meta.frame_size()
    }

    fn offset(&self, name: &str) -> u32 {
        self.func_meta.offset(name)
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