//! Function metadata.

use koopa::ir::{ValueKind::*, *};
use core::panic;
use std::collections::HashMap;

pub struct FuncMeta {
    frame_size: usize,
    offset: HashMap<Value, usize>,
    is_leaf: bool,
    arg_size: usize,
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
    pub fn frame_size(&self) -> usize {
        self.frame_size
    }

    /// Returns the offset of the given value, in terms of bytes.
    pub fn offset(&self, value: Value) -> Option<usize> {
        self.offset.get(&value).copied()
    }

    /// Returns whether the function is a leaf node, i.e.,
    /// no function call from within.
    pub fn is_leaf(&self) -> bool {
        self.is_leaf
    }

    /// Returns the frame size reserved for function call arguments,
    /// in terms of bytes.
    pub fn arg_size(&self) -> usize {
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
        const MAX_ARG_NUM_IN_REG: usize = 8;
        let mut res = Self::new();
        let values = Self::func_values(func);

        let data = |handle| func.dfg().value(handle);
        let kind = |handle| data(handle).kind();
        let base_size = |handle| {
            let ty_kind = data(handle).ty().kind();
            if let TypeKind::Pointer(base) = ty_kind {
                base.size()
            } else {
                panic!("Unexpected type kind");
            }
        };

        // Reserve frame for variables.
        values.iter().for_each(|&handle| {
            if let Alloc(..) = kind(handle) {
                res.offset.insert(handle, res.frame_size);
                res.frame_size += base_size(handle);
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
            if let Call(..) = kind(user) {
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
                Call(call) => call.args().len() as usize,
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
