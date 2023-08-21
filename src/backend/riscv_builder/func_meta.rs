//! Function metadata.

use koopa::ir::{ValueKind::*, *};
use std::collections::HashMap;

pub struct FuncMeta {
    frame_size: u32,
    offset: HashMap<Value, u32>,
    is_leaf: bool,
    arg_size: u32,
}

impl FuncMeta {
    /// Create an empty instance.
    pub fn new() -> Self {
        Self {
            frame_size: 0,
            offset: HashMap::new(),
            is_leaf: false,
            arg_size: 0,
        }
    }

    /// Returns the frame size of the function, in terms of bytes.
    pub fn frame_size(&self) -> u32 {
        self.frame_size
    }

    /// Returns the offset of the given value, in terms of bytes.
    pub fn offset(&self, value: Value) -> u32 {
        *self.offset.get(&value).unwrap()
    }

    /// Returns whether the function is a leaf node, i.e.,
    /// no function call from within.
    pub fn is_leaf(&self) -> bool {
        self.is_leaf
    }

    /// Returns the frame size reserved for function call arguments,
    /// in terms of bytes.
    pub fn arg_size(&self) -> u32 {
        self.arg_size
    }

    /// Returns all the values in the given functions, collected in a Vec.
    fn func_values(func: &FunctionData) -> Vec<Value> {
        func.layout()
            .bbs()
            .nodes()
            .flat_map(|block| block.insts().keys().map(|&handle| handle))
            .collect()
    }
}

impl From<&FunctionData> for FuncMeta {
    fn from(func: &FunctionData) -> Self {
        const MAX_ARG_NUM_IN_REG: u32 = 8;
        let mut res = Self::new();
        let values = Self::func_values(func);
        let kind = |handle| func.dfg().value(handle).kind();

        // Reserve frame for variables.
        values.iter().for_each(|&handle| {
            if let Alloc(_) = kind(handle) {
                res.offset.insert(handle, res.frame_size);
                res.frame_size += 4;
            }
        });

        // Reserve frame for temporary location of function arguments.
        // TODO: This can be optimized.
        values.iter().for_each(|&handle| {
            let used_by = func.dfg().value(handle).used_by();
            if used_by.len() != 1 {
                return;
            }
            let &user = used_by.iter().next().unwrap();
            if let Call(_) = kind(user) {
                res.offset.insert(handle, res.frame_size);
                res.frame_size += 4;
            }
        });

        // Determine whether the function is a leaf node.
        res.is_leaf = !values
            .iter()
            .any(|&handle| matches!(kind(handle), Call { .. }));

        // 4B for reserving return address.
        if !res.is_leaf {
            res.frame_size += 4;
        }

        // Reserve frame for calling convention.
        let arg_num = values
            .iter()
            .map(|&handle| match kind(handle) {
                Call(call) => call.args().len() as u32,
                _ => 0,
            })
            .max()
            .unwrap_or(0);
        if arg_num > MAX_ARG_NUM_IN_REG {
            res.arg_size = 4 * (arg_num - MAX_ARG_NUM_IN_REG);
            res.frame_size += res.arg_size;
            res.offset.values_mut().for_each(|offset| {
                *offset += res.arg_size;
            });
        }

        // Align frame size to 16B.
        res.frame_size = (res.frame_size + 15) / 16 * 16;

        res
    }
}
