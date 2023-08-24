//! Flatten initializer lists and fill in omitted values with zeros.
//! This can be space-consuming if you create a large array with only zeros.

// This module, along with every piece of code in midend that uses it,
// is written very awfully.
// Maybe I should come up with a new representation of initializer lists
// which allows for more graceful handling of C style aggregate initializer.
// Or possibly those C syntax rules are simply a mess.

use crate::frontend::ast::{self, Exp, InitList};

pub fn flatten(list: &[InitList], sizes: &[Exp]) -> InitList {
    // e.g. [1, 2, 3] => [3, 3 * 2, 3 * 2 * 1]
    let dims = sizes
        .iter()
        .rev()
        .scan(1, |acc, size| {
            *acc *= size.value() as usize;
            Some(*acc)
        })
        .collect::<Vec<_>>();

    let mut flat = Vec::with_capacity(dims.last().unwrap().clone());
    flatten_helper(&mut flat, dims, list);
    InitList::Flat(flat)
}

fn flatten_helper(dst: &mut Vec<Exp>, dims: Vec<usize>, list: &[InitList]) {
    use InitList::*;
    let begin_size = dst.len();
    list.iter().for_each(|init| match init {
        Exp(exp) => {
            dst.push(exp.clone()); // NOTE: Overhead.
        }
        List(sub_list) => {
            let len = dst.len();
            let sub_dims = dims
                .iter()
                .take(dims.len() - 1)
                .map_while(|dim| if len % dim == 0 { Some(*dim) } else { None })
                .collect::<Vec<_>>();
            flatten_helper(dst, sub_dims, sub_list);
        }
        Flat(..) => {
            panic!("Unexpected arm");
        }
    });

    let rest = dims.last().unwrap() + begin_size - dst.len();
    (0..rest).for_each(|_| {
        dst.push(ast::Exp::from_number(0));
    });
}
