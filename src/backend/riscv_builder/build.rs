//! Build RISCV program from Koopa IR.
use super::RiscvBuilder;
use crate::backend::riscv::Reg;
use koopa::ir::entities::*;
use koopa::ir::layout::*;

impl<'a> RiscvBuilder<'a> {
    pub fn build_prog(&mut self, prog: &'a Program) {
        self.enter_prog(prog);
        prog.inst_layout().iter().for_each(|&inst| self.build_global_alloc(inst));
        prog.func_layout().iter().for_each(|&func| self.build_func(func));
    }

    pub fn build_func(&mut self, func: Function) {
        // Ignore function declaration.
        if self.is_func_decl(func) {
            return;
        }

        self.enter_func(func);

        let func = self.func_data(func);

        // When entering a function, some registers are already
        // allocated to parameters.
        func.params().iter().take(8).enumerate().for_each(|(i, &value)| {
            self.alloc_reg(value, Some(RiscvBuilder::ARG_REGS[i]));
        });

        let mut iter = func.layout().bbs().iter();
        // Entry block needs special care.
        let entry_block = iter.next().unwrap();
        self.build_block(entry_block, true);
        iter.for_each(|block| self.build_block(block, false));
    }

    pub fn build_block(
        &mut self,
        (&bb, node): (&BasicBlock, &BasicBlockNode),
        is_entry: bool,
    ) {
        self.enter_block(bb, is_entry);

        if is_entry {
            let size = self.frame_size() as i32;
            self.build_addi("sp", "sp", -size);
            if !self.is_leaf_func() {
                self.build_sw("ra", size - 4, "sp");
            }
        }

        node.insts().keys().for_each(|&inst| { self.build_inst(inst, None); });
    }

    pub fn build_inst(&mut self, value: Value, dst: Option<Reg>) -> Option<Reg> {
        use ValueKind::*;

        // println!("build_inst: {value:?}");
        // println!("{value_data:#?}");
        let res = match self.value_kind(value) {
            Integer(_) => self.build_integer(value, dst),
            Alloc(_) => None, // Alloc has been translated to stack offset.
            Load(_) => self.build_load(value, dst),
            Store(_) => self.build_store(value),
            GetElemPtr(_) => self.build_get_elem_ptr(value, dst),
            Binary(_) => self.build_binary(value, dst),
            Branch(_) => self.build_branch(value),
            Jump(_) => self.build_jump(value),
            Call(_) => self.build_call(value, dst),
            Return(_) => self.build_return(value),
            _ => panic!("Unexpected value kind"),
        };

        // TODO: Optimize on argument passing.
        // Arguments are temporarily stored in stack.
        if self.is_arg(value) {
            let reg = res.unwrap();
            let imm = self.offset(value);
            self.build_sw(reg, imm as i32, "sp");
            self.free_reg(value, reg);
        }

        res
    }
}
