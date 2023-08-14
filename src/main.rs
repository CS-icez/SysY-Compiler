use std::env::args;
use std::fs::read_to_string;
use std::io::Result;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(sysy);

mod ast;

fn main() -> Result<()> {
  let mut args = args();
  args.next();
  let mode = args.next().unwrap();
  let input = args.next().unwrap();
  args.next();
  let output = args.next().unwrap();
  let input = read_to_string(input)?;
  let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
  println!("{:#?}", ast);
  Ok(())
}
