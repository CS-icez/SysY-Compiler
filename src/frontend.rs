//! This module abstracts the frontend of the compiler, i.e.,
//! from SysY program in ASCII text, to AST in memory.
//! The AST may not be strictly identical to the original SysY program
//! because sematic analysis is performed automatically.

pub mod ast;
mod sem_analyzer;

pub use ast::Program;
use parser::ProgramParser;
use sem_analyzer::SemAnalyzer;

lalrpop_util::lalrpop_mod!(parser, "/frontend/sysy.rs");

impl Program {
    /// Creates an AST from SysY program in ASCII text.
    pub fn from_sysy_text(prog: &str) -> Self {
        ProgramParser::new()
            .parse(&prog)
            .expect("Parse error")
            .analyze_sem()
    }

    /// Analyzes the semantics of input AST and returns the transformed AST.
    fn analyze_sem(mut self) -> Self {
        SemAnalyzer::run_on(&mut self);
        self
    }
}
