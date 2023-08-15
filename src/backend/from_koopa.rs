use std::collections::LinkedList;
use koopa::ir::entities;
use koopa::ir::layout;
use koopa::ir::dfg::DataFlowGraph;
use super::riscv;

impl riscv::Program {
    pub fn from_koopa(prog: &entities::Program) -> Self {
        let mut funcs = LinkedList::new();
        for &func in prog.func_layout() {
            let item = riscv::Func::from_koopa(prog.func(func));
            funcs.push_back(item);
        }
        Self {
            global_decls: LinkedList::new(),
            funcs,
        }
    }
}

impl riscv::Func {
    fn from_koopa(func: &entities::FunctionData) -> Self {
        let mut blocks = LinkedList::new();
        let bbs = func.layout().bbs();
        for (&bb, node) in bbs {
            let item = riscv::Block::from_koopa(func.dfg().bb(bb), node, func.dfg());
            blocks.push_back(item);
        }
        Self {
            name: func.name()[1..].to_string(),
            blocks,
        }
    }
}

impl riscv::Block {
    fn from_koopa(
        data: &entities::BasicBlockData,
        node: &layout::BasicBlockNode,
        dfg: &DataFlowGraph,
    ) -> Self {
        let mut insts = LinkedList::new();
        for &inst in node.insts().keys() {
            let mut item = riscv::Inst::from_koopa(dfg.value(inst), dfg);
            insts.append(&mut item);
        }
        Self {
            name: data.name().as_ref().unwrap().to_string(),
            insts,
        }
    }
}

impl riscv::Inst {
    fn from_koopa(value: &entities::ValueData, dfg: &DataFlowGraph) -> LinkedList<Self> {
        use entities::ValueKind::*;
        use riscv::Inst::*;
        let mut res = LinkedList::new();
        match value.kind() {
            Integer(int) => {
                let item = Li { rd: "a0".to_string(), imm: int.value() };
                res.push_back(item);
            }
            Return(ret) => {
                if ret.value().is_some() {
                    let value_data = dfg.value(ret.value().unwrap());
                    let mut item = Self::from_koopa(value_data, dfg);
                    res.append(&mut item);
                }
                res.push_back(Ret);
            }
            _ => unreachable!(),
        }
        res
    }
}
