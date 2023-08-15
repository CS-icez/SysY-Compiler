pub use ast::CompUnit;

pub mod ast;

lalrpop_util::lalrpop_mod!(parser, "/frontend/sysy.rs");

impl From<&str> for CompUnit {
    fn from(prog: &str) -> CompUnit {
        parser::CompUnitParser::new().parse(&prog).unwrap()
    }
}

