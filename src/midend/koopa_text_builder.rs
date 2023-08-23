//! Koopa text builder.

mod build_from;
mod token_generator;

use crate::frontend::ast;
use build_from::BuildFrom;
use std::collections::{HashMap, VecDeque};
use token_generator::TokenGenerator;

struct LoopMeta {
    cond_label: String,
    #[allow(dead_code)]
    body_label: String, // Not used, but added for completeness.
    end_label: String,
}

pub struct KoopaTextBuilder {
    text: String,
    loop_meta: VecDeque<LoopMeta>, // Actually a stack.
    token_gen: HashMap<&'static str, TokenGenerator>,
    arrays: HashMap<String, usize>,
    pointers: HashMap<String, usize>,
}

impl KoopaTextBuilder {
    const TAB: &str = "    ";

    /// Builds Koopa text from the given AST.
    pub fn build(prog: &ast::Program) -> String {
        let mut builder = Self::new();
        builder.build_from(prog, false);
        builder.text
    }

    /// Creates a new builder.
    fn new() -> Self {
        Self {
            text: String::new(),
            loop_meta: VecDeque::new(),
            token_gen: HashMap::new(),
            arrays: HashMap::new(),
            pointers: HashMap::new(),
        }
    }

    /// Enters a while loop.
    fn enter_loop(&mut self) {
        let cond_label = self.peek_token("%cond_");
        let body_label = self.peek_token("%body_");
        let end_label = self.peek_token("%endwhile_");
        self.loop_meta.push_back(LoopMeta {
            cond_label,
            body_label,
            end_label,
        });
    }

    /// Exits the current while loop.
    fn exit_loop(&mut self) {
        self.loop_meta.pop_back();
    }

    /// Returns the condition label of the current while loop.
    fn cur_cond_label(&self) -> &String {
        &self.loop_meta.back().unwrap().cond_label
    }

    /// Returns the end label of the current while loop.
    fn cur_end_label(&self) -> &String {
        &self.loop_meta.back().unwrap().end_label
    }

    /// Makes a new token with the given prefix.
    /// Tokens are guaranteed to be unique among calls.
    fn make_token(&mut self, name: &'static str) -> String {
        self.token_gen
            .entry(name)
            .or_insert_with(|| TokenGenerator::new(name))
            .generate()
    }

    /// Peeks the next token with the given prefix, i.e.,
    /// what will be returned if `make_token` is called with the same argument.
    fn peek_token(&mut self, name: &'static str) -> String {
        self.token_gen
            .entry(name)
            .or_insert_with(|| TokenGenerator::new(name))
            .peek()
    }

    /// Returns the previously generated token with the given prefix.
    fn prev_token(&mut self, name: &'static str) -> String {
        self.token_gen.get(name).unwrap().prev()
    }

    /// Equivalent to `make_token("%")`, serving as a shortcut.
    fn make_num(&mut self) -> String {
        self.make_token("%")
    }

    /// Equivalent to `make_token("%tmp")`, serving as a shortcut.
    /// This is used for logical and/or expressions.
    fn make_tmp(&mut self) -> String {
        self.make_token("%tmp_")
    }

    /// Equivalent to `make_token("%koopa_")`, serving as a shortcut.
    /// This is used to satisfy Koopa's rule that `br` `jump` `ret` must be
    /// the last instruction in a basic block.
    fn make_koopa(&mut self) -> String {
        self.make_token("%koopa_")
    }

    /// Resets the state of token generator.
    fn reset_tokens(&mut self) {
        let f = |gen: &mut TokenGenerator| {gen.reset(); Some(0) };
        self.token_gen.get_mut("%").and_then(f);
        self.token_gen.get_mut("%tmp_").and_then(f);
        self.token_gen.get_mut("%ptr_").and_then(f);
    }

    fn is_array(&self, name: &str) -> bool {
        self.arrays.contains_key(name)
    }

    fn is_pointer(&self, name: &str) -> bool {
        self.pointers.contains_key(name)
    }
}
