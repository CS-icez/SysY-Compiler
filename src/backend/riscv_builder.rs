use std::collections::HashMap;
use koopa::ir::entities::{self, Value};
use reg_manager::RegManager;
use func_meta::FuncMeta;
use super::riscv::{self, Reg};

mod build;
mod reg_manager;
mod func_meta;
mod build_helpers;

#[derive(Default)]
pub struct RiscvBuilder {
    prog: riscv::Program,
    reg_mgr: RegManager,
    func_meta: FuncMeta,
    func_name: HashMap<entities::Function, String>,
}

impl RiscvBuilder {
    pub fn new() -> Self {
        Self {
            prog: riscv::Program::new(),
            reg_mgr: RegManager::new(),
            func_meta: FuncMeta::new(),
            func_name: HashMap::new(),
        }
    }

    pub fn build(&mut self, prog: &entities::Program) -> riscv::Program {
        self.build_prog(prog);
        // Huge overhead here, yet don't know which syntax to use to avoid it.
        let res = self.prog.clone();
        *self = Default::default();
        res
    }

    fn alloc_reg(&mut self, inst: Value, reg: Option<Reg>) -> Reg {
        let mut res = self.reg_mgr.alloc(inst, reg);
        match reg {
            Some(r) if r != res => {
                self.push_inst(riscv::Inst::Mv { rd: res, rs: r });
                res = r;
            }
            _ => {}
        }
        res
    }

    #[allow(dead_code)]
    fn replace_reg_owner(&mut self, old: Value, new: Value) {
        let reg = self.reg_mgr.reg(old);
        self.reg_mgr.free(old, reg);
        self.reg_mgr.alloc(new, Some(reg));
    }

    fn free_reg(&mut self, inst: Value, reg: Reg) {
        // println!("free_reg: {inst:?}");
        self.reg_mgr.free(inst, reg);
    }

    fn reset_reg(&mut self) {
        self.reg_mgr.reset();
    }

    fn build_func_meta(&mut self, func: &entities::FunctionData) {
        self.func_meta = FuncMeta::from(func);
    }

    fn frame_size(&self) -> u32 {
        self.func_meta.frame_size()
    }

    fn offset(&self, value: Value) -> u32 {
        self.func_meta.offset(value)
    }

    fn is_leaf(&self) -> bool {
        self.func_meta.is_leaf()
    }

    fn arg_offset(&self) -> u32 {
        self.func_meta.arg_offset()
    }

    fn record_func_name(&mut self, func: entities::Function, name: &str) {
        self.func_name.insert(func, name.to_string());
    }

    fn func_name(&self, func: entities::Function) -> &str {
        self.func_name.get(&func).unwrap()
    }

    fn back_func_mut(&mut self) -> &mut riscv::Func {
        self.prog.funcs.back_mut().unwrap()
    }

    fn back_block_mut(&mut self) -> &mut riscv::Block {
        self.back_func_mut().blocks.back_mut().unwrap()
    }

    pub fn push_global_def(&mut self, name: &str, init: i32) {
        self.prog.global_defs.push_back(riscv::GlobalDef {
            name: name.to_string(),
            init,
        });
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