use std::env::args;
use std::fs::read_to_string;
use std::io::Result;

pub mod ast;
mod frontend;
mod midend;

fn main() -> Result<()> {
  let mut args = args();
  args.next();
  let mode = args.next().unwrap();
  let input = args.next().unwrap();
  args.next();
  let output = args.next().unwrap();
  let input = read_to_string(input)?;
  let ast = frontend::to_ast(&input);
  let koopa_text = midend::to_koopa_text(&ast);
  println!("{}", koopa_text);
  Ok(())
}
