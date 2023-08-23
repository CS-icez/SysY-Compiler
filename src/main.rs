//! We assume the input program is both syntactically and semantically correct.
//! Otherwise, anything could happen, probably just panic somewhere.
//! This applies to all modules in this crate.

mod backend;
mod frontend;
mod midend;

use frontend::Program;
use std::env;
use std::fs;
use std::io::Write;

fn main() {
    let [_, ref mode, ref input, _, ref output] = env::args().collect::<Vec<_>>()[..] else {
        panic!("Incorrect command line arguments");
    };

    // let mut args = env::args();
    // let _cmd = args.next().unwrap();
    // let mode = args.next().unwrap();
    // let input = args.next().unwrap();
    // assert_eq!(args.next().unwrap(), "-o");
    // let output = args.next().unwrap();

    let input = fs::read_to_string(input).unwrap();
    let mut output = fs::File::create(output).unwrap();

    let res = match &mode[..] {
        "-koopa" => Program::from_sysy_text(&input).to_koopa_text(),
        "-riscv" | "-perf" => {
            let koopa = Program::from_sysy_text(&input).to_koopa_program();
            backend::riscv_text_from(&koopa)
        }
        _ => panic!("Unknown mode: {mode}"),
    };

    output.write_all(res.as_bytes()).unwrap();
}
