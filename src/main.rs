use backend::riscv_text_from;
use frontend::Program;
use std::env;
use std::fs;
use std::io::Write;

mod backend;
mod frontend;
mod midend;
pub mod utils;

fn main() -> std::io::Result<()> {
    let mut args = env::args();
    let _cmd = args.next().unwrap();
    let mode = args.next().unwrap();
    let input = args.next().unwrap();
    assert_eq!(args.next().unwrap(), "-o");
    let output = args.next().unwrap();

    let input = fs::read_to_string(input)?;
    let mut output = fs::File::create(output).unwrap();

    let res = match &mode[..] {
        "-koopa" => {
            Program::from_c_text(&input[..])
                .to_koopa_text()
        }
        "-riscv" => {
            riscv_text_from(&Program::from_c_text(&input[..])
                .to_koopa_program())
        }
        _ => unreachable!(),
    };

    output.write_all(res.as_bytes())?;
    Ok(())
}
