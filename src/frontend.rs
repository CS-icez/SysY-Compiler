use crate::ast::CompUnit;

lalrpop_util::lalrpop_mod!(parser, "/frontend/sysy.rs");

pub fn to_ast(program: &str) -> CompUnit {
    parser::CompUnitParser::new().parse(&program).unwrap()
}
