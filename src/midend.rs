use koopa::front::Driver;
use koopa::ir::entities;
use crate::frontend::ast;
use crate::utils::token_generator::TokenGenerator;
use build_from::BuildFrom;

mod build_from;

#[derive(Default)]
struct KoopaTextBuilder {
    text: String,
    token_gen: TokenGenerator,
}

impl KoopaTextBuilder {
    const TAB: & str = "    ";

    pub fn new() -> Self {
        Self {
            text: String::new(),
            token_gen: TokenGenerator::new("%"),
        }
    }

    pub fn build(&mut self, prog: &ast::Program) -> String {
        self.build_from(prog);
        // Huge overhead here, yet don't know which syntax to use to avoid it.
        let res = self.text.clone();
        *self = Default::default();
        res
    }

    fn make_token(&self) -> String {
        self.token_gen.generate()
    }
}

impl ast::Program {
    pub fn to_koopa_text(&self) -> String {
        KoopaTextBuilder::new().build(self)
    }

    pub fn to_koopa_program(&self) -> entities::Program {
        let text = self.to_koopa_text();
        Driver::from(text)
            .generate_program()
            .expect("Invalid Koopa text")
    }
}
