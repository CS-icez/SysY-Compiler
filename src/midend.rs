//! This module abstracts the midend of the compiler, i.e.,
//! from AST in memory, to Koopa IR in memory or ASCII text.
//! I implemented the conversion from AST to Koopa text, and
//! used the Koopa library to transform it to in-memory form.
//! The library may rearrange basic blocks, so the input text
//! must include some seemingly redundant jump instructions.

mod koopa_text_builder;

use crate::frontend::ast;
use koopa::front::Driver;
use koopa::ir::entities;
use koopa_text_builder::KoopaTextBuilder;

impl ast::Program {
    /// Converts an AST to Koopa text.
    pub fn to_koopa_text(&self) -> String {
        KoopaTextBuilder::build(self)
    }

    /// Converts an AST to Koopa in-memory program.
    pub fn to_koopa_program(&self) -> entities::Program {
        let text = self.to_koopa_text();
        Driver::from(text)
            .generate_program()
            .expect("Invalid Koopa text")
    }
}
