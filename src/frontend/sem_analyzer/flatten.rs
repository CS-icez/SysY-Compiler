//! Flatten initializer lists and fill in omitted values with zeros.
//! This may be space-consuming if you create a large array with only zeros.

use super::SemAnalyzer;
use crate::frontend::ast::{self, Exp, InitList, VarDef};
use std::iter::repeat;

impl SemAnalyzer {
    pub fn flatten(def: &mut VarDef) {
        let VarDef::Array(_, sizes, opt_init) = def else {
            panic!("Unexpected arm");
        };
        let Some(init) = opt_init else {
            panic!("Unexpected arm");
        };

        let dims = sizes
            .iter()
            .rev()
            .scan(1, |acc, size| {
                *acc *= size.value() as usize;
                Some(*acc)
            })
            .collect::<Vec<_>>();

        let mut flat = Vec::with_capacity(dims.last().unwrap().clone());
        Self::flatten_helper(&mut flat, dims, init);
        *init = InitList::Flat(flat);
    }

    fn flatten_helper(dst: &mut Vec<Exp>, dims: Vec<usize>, init: &InitList) {
        use InitList::*;
        let begin_size = dst.len();
        match init {
            Exp(exp) => {
                // TODO: avoid cloning.
                dst.push(exp.clone());
                return;
            }
            List(list) => {
                list.iter().for_each(|init| match init {
                    Exp(exp) => {
                        dst.push(exp.clone());
                    }
                    List(_) => {
                        let cur_len = dst.len();
                        let sub_dims = dims
                            .iter()
                            .take(dims.len() - 1)
                            .map_while(|dim| if cur_len % dim == 0 { Some(*dim) } else { None })
                            .collect::<Vec<_>>();
                        Self::flatten_helper(dst, sub_dims, init);

                    }
                    Flat(_) => {
                        panic!("Unexpected arm");
                    }
                });
                let rest = dims.last().unwrap() + begin_size - dst.len();
                repeat(0).take(rest).for_each(|_| {
                    dst.push(ast::Exp::from_number(0));
                });
            }
            Flat(_) => {
                panic!("Unexpected arm");
            }
        }
    }
}
