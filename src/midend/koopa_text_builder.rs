mod build_from;

use crate::frontend::ast;
use crate::utils::token_generator::TokenGenerator;
use build_from::BuildFrom;
use std::collections::{HashMap, VecDeque};

struct LoopMeta {
    cond_label: String,
    #[allow(dead_code)]
    body_label: String, // Not used, but added for completeness.
    end_label: String,
}

#[derive(Default)]
pub struct KoopaTextBuilder {
    text: String,
    loop_meta: VecDeque<LoopMeta>, // Actually a stack.
    token_gen: HashMap<&'static str, TokenGenerator>,
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
        let gen = self
            .token_gen
            .entry(name)
            .or_insert_with(|| TokenGenerator::new(name));
        gen.generate()
    }

    /// Peeks the next token with the given prefix, i.e.,
    /// what will be returns if `make_token` is called with the same argument.
    fn peek_token(&mut self, name: &'static str) -> String {
        let gen = self
            .token_gen
            .entry(name)
            .or_insert_with(|| TokenGenerator::new(name));
        gen.peek()
    }

    /// Equivalent to `make_token("%")`, serving as a shortcut.
    fn make_num(&mut self) -> String {
        self.make_token("%")
    }

    /// Equivalent to `make_token("%tmp")`, serving as a shortcut.
    fn make_tmp(&mut self) -> String {
        self.make_token("%tmp")
    }

    /// Equivalent to `make_token("%koopa_")`, serving as a shortcut.
    fn make_koopa(&mut self) -> String {
        self.make_token("%koopa_")
    }

    /// Resets the state of token generator.
    fn reset_tokens(&mut self) {
        self.token_gen.clear();
    }
}
