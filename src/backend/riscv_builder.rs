//! RISCV in-memory program builder.

mod build;
mod build_helpers;
mod build_value;
mod func_meta;
mod reg_manager;

use super::riscv::{self, Reg};
use func_meta::FuncMeta;
use koopa::ir::entities::*;
use reg_manager::RegManager;
use std::cell::Ref;
use std::collections::{HashSet, LinkedList};
use std::ops::Not;

pub struct RiscvBuilder<'a> {
    prog: riscv::Program,
    reg_mgr: RegManager,
    func_meta: FuncMeta,
    koopa_prog: Option<&'a Program>,
    koopa_func: Option<&'a FunctionData>,
}

impl<'a:'r, 'r> RiscvBuilder<'a> {
    /// RISCV registers used for passing arguments.
    const ARG_REGS: [Reg; 8] = ["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"];

    /// Builds RISCV program from given Koopa IR program.
    pub fn build(prog: &Program) -> riscv::Program {
        let mut builder = Self::new();
        builder.build_prog(prog);
        builder.prog
    }

    /// Creates a new RISCV builder.
    fn new() -> Self {
        Self {
            prog: riscv::Program::new(),
            reg_mgr: RegManager::new(),
            func_meta: FuncMeta::new(),
            koopa_prog: None,
            koopa_func: None,
        }
    }

    /// Returns internal `koopa_prog` field.
    fn koopa_prog(&self) -> &'a Program {
        self.koopa_prog.unwrap()
    }

    /// Returns internal `koopa_func` field.
    fn koopa_func(&self) -> &'a FunctionData {
        self.koopa_func.unwrap()
    }

    /// Enters the given program.
    fn enter_prog(&mut self, prog: &'a Program) {
        self.koopa_prog = Some(prog);
    }

    /// Enters the given function.
    fn enter_func(&mut self, func: Function) {
        let data = self.koopa_prog().func(func);
        self.koopa_func = Some(data);
        self.func_meta = FuncMeta::from(data);
        self.reg_mgr.reset();
        self.prog.funcs.push_back(riscv::Func {
            name: self.func_name(func).to_string(),
            blocks: LinkedList::new(),
        });
    }

    /// Enters the given block.
    fn enter_block(&mut self, block: BasicBlock, is_entry: bool) {
        let name = if is_entry {
            self.koopa_func().name()[1..].to_string()
        } else {
            self.block_name(block).to_string()
        };

        self.back_func_mut().blocks.push_back(riscv::Block {
            name,
            insts: LinkedList::new(),
        });
    }

    // The followings are all wrapper functions.

    // Wrappers of `RegManager`.

    /// Allocates the given register for the given value and returns it.
    fn alloc_reg(&mut self, inst: Value, reg: Option<Reg>) -> Reg {
        self.reg_mgr.alloc(inst, reg)
    }

    /// Frees the given register from the given value.
    fn free_reg(&mut self, inst: Value, reg: Reg) {
        // println!("free_reg: {inst:?}");
        self.reg_mgr.free(inst, reg);
    }

    // Wrappers of `FuncMeta`.

    /// Returns the frame size of the current function.
    fn frame_size(&self) -> usize {
        self.func_meta.frame_size()
    }

    /// Returns the stack offset of the given local variable
    /// in the current function.
    fn offset(&self, value: Value) -> usize {
        self.func_meta.offset(value).unwrap()
    }

    /// Returns whether the current function is a leaf node.
    fn is_leaf_func(&self) -> bool {
        self.func_meta.is_leaf()
    }

    // Wrappers of Koopa library.

    /// Returns whether the given function is a declaration.
    fn is_func_decl(&self, func: Function) -> bool {
        self.koopa_prog().func(func).layout().entry_bb() == None
    }

    /// Returns the data of the given function.
    fn func_data(&self, func: Function) -> &'r FunctionData {
        self.koopa_prog().func(func)
    }

    /// Returns the name of the given function.
    fn func_name(&self, func: Function) -> &str {
        self.koopa_prog().func(func).name()[1..].as_ref()
    }

    fn block_name(&self, block: BasicBlock) -> &str {
        self.koopa_func().dfg().bb(block).name().as_ref().unwrap()[1..].as_ref()
    }

    /// Returns the data of the given global value.
    /// DO NOT CALL THIS FUNCTION FROM OUTSIDE. WIELD THINGS HAPPEN.
    /// I DON'T UNDERSTAND WHY.
    fn global_value_data(&self, value: Value) -> Ref<ValueData> {
        self.koopa_prog().borrow_value(value)
    }

    /// Returns the name of the given global variable.
    fn global_var_name(&self, var: Value) -> String {
        self.global_value_data(var).name().as_ref().unwrap()[1..].to_string()
        //? I tried to return `&str` or `&String`, but it didn't work.
        //? Too little I know about `Ref<T>`. 
        //? self.global_value_data(var).name().as_ref().unwrap().as_str()
    }

    //? Failed to compile this.
    //? fn global_value_kind(&'s self, var: Value) -> &'r ValueKind {
    //?     self.global_value_data(var).kind()
    //? }

    /// Returns the data of the given local value in the current function.
    fn value_data(&self, value: Value) -> &'r ValueData {
        self.koopa_func().dfg().value(value)
    }

    /// Returns the kind of the given local value in the current function.
    fn value_kind(&self, value: Value) -> &'r ValueKind {
        self.value_data(value).kind()
    }

    /// Returns a reference to the values that the given local value is used by.
    /// The given value must fall in the current function.
    fn used_by(&self, value: Value) -> &HashSet<Value> {
        self.value_data(value).used_by()
    }

    /// Returns whether the given local value is used by
    /// at least one other value.
    /// The given value must fall in the current function.
    fn is_used(&self, value: Value) -> bool {
        self.used_by(value).is_empty().not()
    }

    /// Returns whether the given value is a global variable.
    fn is_global_var(&self, value: Value) -> bool {
        self.koopa_prog().inst_layout().contains(&value)
    }

    /// Returns whether the given value is a local variable
    /// of the current function.
    fn is_local_var(&self, value: Value) -> bool {
        self.func_meta.offset(value) != None
    }

    /// Returns whether the given local value is used as a function argument.
    /// The given value must fall in the current function.
    fn is_arg(&self, value: Value) -> bool {
        let used_by = self.used_by(value);
        if used_by.len() != 1 {
            return false;
        }
        let &user = used_by.iter().next().unwrap();
        matches!(self.value_kind(user), ValueKind::Call(_))
    }

    // Wrappers of RISCV data types.

    /// Provides a mutable reference to the current function being built.
    fn back_func_mut(&mut self) -> &mut riscv::Func {
        self.prog.funcs.back_mut().unwrap()
    }

    /// Provides a mutable reference to the current block being built.
    fn back_block_mut(&mut self) -> &mut riscv::Block {
        self.back_func_mut().blocks.back_mut().unwrap()
    }

    /// Appends a new global variable definition to the program.
    fn push_global_def(&mut self, var: Value, init: LinkedList<riscv::MemFill>) {
        self.prog.global_defs.push_back(riscv::GlobalDef {
            name: self.global_var_name(var).to_string(),
            init,
        });
    }

    /// Appends a new instruction to the current block.
    fn push_inst(&mut self, inst: riscv::Inst) {
        self.back_block_mut().insts.push_back(inst);
    }
}
