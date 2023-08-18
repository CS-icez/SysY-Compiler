use koopa::front::Driver;
use koopa::ir::entities;
use crate::frontend::ast;
use crate::utils::token_generator::TokenGenerator;
use build_from::BuildFrom;
// use post_process::*;

mod build_from;
mod post_process;

#[derive(Default)]
struct KoopaTextBuilder {
    text: String,
    temp_gen: TokenGenerator,
    name_gen: TokenGenerator,
    then_label_gen: TokenGenerator,
    else_label_gen: TokenGenerator,
    endif_label_gen: TokenGenerator,
    ret_label_gen: TokenGenerator,
}

impl KoopaTextBuilder {
    const TAB: & str = "    ";

    pub fn new() -> Self {
        Self {
            text: String::new(),
            temp_gen: TokenGenerator::new("%"),
            name_gen: TokenGenerator::new("@if_"),
            then_label_gen: TokenGenerator::new("%then_"),
            else_label_gen: TokenGenerator::new("%else_"),
            endif_label_gen: TokenGenerator::new("%endif_"),
            ret_label_gen: TokenGenerator::new("%ret_"),
        }
    }

    pub fn build(&mut self, prog: &ast::Program) -> String {
        self.build_from(prog);
        // Huge overhead here, yet don't know which syntax to use to avoid it.
        let res = self.text.clone();
        *self = Default::default();
        res
    }

    fn make_temp(&self) -> String {
        self.temp_gen.generate()
    }

    fn make_name(&self) -> String {
        self.name_gen.generate()
    }

    fn make_then_label(&self) -> String {
        self.then_label_gen.generate()
    }

    fn make_else_label(&self) -> String {
        self.else_label_gen.generate()
    }

    fn make_endif_label(&self) -> String {
        self.endif_label_gen.generate()
    }

    fn make_ret_label(&self) -> String {
        self.ret_label_gen.generate()
    }

    fn reset_labels(&mut self) {
        self.temp_gen.reset();
        self.then_label_gen.reset();
        self.else_label_gen.reset();
        self.endif_label_gen.reset();
    }
}

impl ast::Program {
    pub fn to_koopa_text(&self) -> String {
        KoopaTextBuilder::new().build(self)
        // post_process(text)
    }

    pub fn to_koopa_program(&self) -> entities::Program {
        let text = self.to_koopa_text();
        Driver::from(text)
            .generate_program()
            .expect("Invalid Koopa text")
    }
}
