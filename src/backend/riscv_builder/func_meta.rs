use koopa::ir::{ValueKind::*, *};
use std::collections::HashMap;
// use super::riscv;
// use super::reg_manager;

#[derive(Default)]
pub struct FuncMeta {
    frame_size: u32,
    offset: HashMap<Value, u32>,
    is_leaf: bool,
    arg_offset: u32,
}

impl FuncMeta {
    pub const MAX_ARG_NUM_IN_REG: u32 = 8;

    pub fn new() -> Self {
        Self {
            frame_size: 0,
            offset: HashMap::new(),
            is_leaf: false,
            arg_offset: 0,
        }
    }

    pub fn frame_size(&self) -> u32 {
        self.frame_size
    }

    pub fn offset(&self, value: Value) -> u32 {
        *self.offset.get(&value).unwrap()
    }

    pub fn is_leaf(&self) -> bool {
        self.is_leaf
    }

    pub fn arg_offset(&self) -> u32 {
        self.arg_offset
    }

    fn for_each_value(func: &FunctionData, mut f: impl FnMut(Value)) {
        func.layout().bbs().nodes()
            .for_each(|block| {
                block.insts().keys()
                    .for_each(|&handle| {
                        f(handle);
                    })
            })
    }

    fn any_value(func: &FunctionData, f: impl Fn(Value) -> bool) -> bool {
        func.layout().bbs().nodes()
            .any(|block| {
                block.insts().keys()
                    .any(|&handle| {
                        f(handle)
                    })
            })
    }

    pub fn max_arg_num(func: &FunctionData) -> u32 {
        func.layout().bbs().nodes()
            .map(|block| {
                block.insts().keys()
                    .map(|&handle| {
                        let kind = func.dfg().value(handle).kind();
                        match kind {
                            Call(call) => call.args().len() as u32,
                            _ => 0,
                        }
                    })
                    .max()
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0)
    }
}

impl From<&FunctionData> for FuncMeta {
    fn from(func: &FunctionData) -> Self {
        let mut res = Self::new();

        // Reserve frame for variables.
        Self::for_each_value(func, |handle| {
            let value = func.dfg().value(handle);
            if let Alloc(_) = value.kind() {
                res.offset.insert(handle, res.frame_size);
                res.frame_size += 4;
            }
        });

        // Reserve frame for function arguments.
        Self::for_each_value(func, |handle| {
            let used_by = func.dfg().value(handle).used_by();
            if used_by.len() != 1 {
                return;
            }
            let &user = used_by.iter().next().unwrap();
            let user_kind = func.dfg().value(user).kind();
            if let Call(_) = user_kind {
                res.offset.insert(handle, res.frame_size);
                res.frame_size += 4;
            }
        });

        // Determine whether the function is a leaf node.
        res.is_leaf = !Self::any_value(func, |handle| {
            let kind = func.dfg().value(handle).kind();
            matches!(kind, Call { .. })
        });

        // 4B for reserving return address.
        if !res.is_leaf {
            res.frame_size += 4;
        }

        // Reserve frame for calling convention.
        let arg_num = Self::max_arg_num(func);
        if arg_num > Self::MAX_ARG_NUM_IN_REG {
            res.arg_offset = 4 * (arg_num - Self::MAX_ARG_NUM_IN_REG);
            res.frame_size += res.arg_offset;
            res.offset.values_mut().for_each(|offset| {
                *offset += res.arg_offset;
            });
        }

        // Align frame size to 16B.
        res.frame_size = (res.frame_size + 15) / 16 * 16;

        res
    }
}
