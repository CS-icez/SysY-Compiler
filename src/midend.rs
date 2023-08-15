use crate::frontend::ast::CompUnit;
pub use to_koopa_text::ToKoopaText;
use koopa::front::Driver;
use koopa::ir::entities::Program;

mod to_koopa_text;

impl CompUnit {
    pub fn to_koopa_program(&self) -> Program {
        Driver::from(self.to_koopa_text()).generate_program().unwrap()
    }
}
