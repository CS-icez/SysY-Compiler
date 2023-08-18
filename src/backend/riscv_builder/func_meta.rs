use std::collections::HashMap;
use koopa::ir::entities;
// use super::riscv;
// use super::reg_manager;

#[derive(Default)]
pub struct FuncMeta {
    frame_size: u32,
    var_offset: HashMap<String, u32>,
}

impl FuncMeta {
    pub fn new() -> Self {
        Self {
            frame_size: 0,
            var_offset: HashMap::new(),
        }
    }

    pub fn frame_size(&self) -> u32 {
        self.frame_size
    }

    pub fn offset(&self, name: &str) -> u32 {
        *self.var_offset.get(name).unwrap()
    }
}

impl From<&entities::FunctionData> for FuncMeta {
    fn from(func: &entities::FunctionData) -> Self {
        use entities::ValueKind::*;
        let mut res = Self::new();
        for block in func.layout().bbs().nodes() {
            for &handle in block.insts().keys() {
                let value = func.dfg().value(handle);
                match value.kind() {
                    Alloc(_) => {
                        let name = value.name().as_ref().unwrap();
                        res.var_offset.insert(name.clone(), res.frame_size);
                        res.frame_size += 4;
                    }
                    _ => {}
                }    
            }
        }
        res
    }
}
