use::std::collections::{VecDeque, HashMap};
use koopa::front::Driver;
use koopa::ir::entities;
use crate::frontend::ast;
use crate::utils::token_generator::TokenGenerator;
use build_from::BuildFrom;
// use post_process::*;

mod build_from;
mod post_process;

struct LoopMeta {
    cond_label: String,
    #[allow(dead_code)]
    body_label: String,
    end_label: String,
}

#[derive(Default)]
struct KoopaTextBuilder {
    text: String,
    loop_meta: VecDeque<LoopMeta>,
    token_gen: HashMap<&'static str, TokenGenerator>,
}

impl KoopaTextBuilder {
    const TAB: & str = "    ";

    pub fn new() -> Self {
        Self {
            text: String::new(),
            loop_meta: VecDeque::new(),
            token_gen: HashMap::new(),
        }
    }

    pub fn build(&mut self, prog: &ast::Program) -> String {
        self.build_from(prog, false);
        // Huge overhead here, yet don't know which syntax to use to avoid it.
        let res = self.text.clone();
        *self = Default::default();
        res
    }

    fn enter_loop(&mut self) {
        let cond_label = self.peek_token("%cond_");
        let body_label = self.peek_token("%body_");
        let end_label = self.peek_token("%endwhile_");
        self.loop_meta.push_back(LoopMeta {
            cond_label, body_label, end_label,
        });
    }

    fn exit_loop(&mut self) {
        self.loop_meta.pop_back();
    }

    fn cur_cond_label(&self) -> &String {
        &self.loop_meta.back().unwrap().cond_label
    }

    fn cur_end_label(&self) -> &String {
        &self.loop_meta.back().unwrap().end_label
    }

    fn make_token(&mut self, name: &'static str) -> String {
        let gen = self.token_gen.entry(name)
            .or_insert_with(|| TokenGenerator::new(name));
        gen.generate()
    }

    fn peek_token(&mut self, name: &'static str) -> String {
        let gen = self.token_gen.entry(name)
            .or_insert_with(|| TokenGenerator::new(name));
        gen.peek()
    }

    fn make_num(&mut self) -> String {
        self.make_token("%")
    }

    fn make_tmp(&mut self) -> String {
        self.make_token("%tmp")
    }

    fn make_koopa(&mut self) -> String {
        self.make_token("%koopa_")
    }

    fn reset_labels(&mut self) {
        self.token_gen.clear();
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
