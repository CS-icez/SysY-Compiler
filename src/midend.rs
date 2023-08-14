use crate::ast::CompUnit;
use to_koopa_text::ToKoopaText;

mod to_koopa_text;

pub fn to_koopa_text(ast: &CompUnit) -> String {
    ast.to_koopa_text()
}