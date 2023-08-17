pub use ast::Program;
use parser::ProgramParser;
use sem_analyzer::SemAnalyzer;

pub mod ast;
mod sem_analyzer;

lalrpop_util::lalrpop_mod!(parser, "/frontend/sysy.rs");

impl Program {
    pub fn from_c_text(prog: &str) -> Self {
        ProgramParser::new()
            .parse(&prog)
            .expect("Parse error")
            .analyze_sem()
    }

    fn analyze_sem(mut self) -> Self {
        SemAnalyzer::new().run(&mut self);
        self
    }
}