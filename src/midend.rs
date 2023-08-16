use crate::frontend::ast::CompUnit;
pub use generate_koopa_text::GenerateKoopaText;
use koopa::front::Driver;
use koopa::ir::entities::Program;

mod generate_koopa_text;

#[allow(non_upper_case_globals)]
static mut koopa_text: String = String::new();

impl CompUnit {
    pub fn to_koopa_text(&self) -> String {
        self.generate_koopa_text();
        let mut res = String::new();
        unsafe { std::mem::swap(&mut koopa_text, &mut res); }
        res
    }

    pub fn to_koopa_program(&self) -> Program {
        self.generate_koopa_text();
        let mut res = String::new();
        unsafe { std::mem::swap(&mut koopa_text, &mut res); }
        Driver::from(res).generate_program().unwrap()
    }
}
