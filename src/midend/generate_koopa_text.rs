use crate::{frontend::ast::*, token_generator::TokenGenerator};
use super::koopa_text;
use lazy_static::lazy_static;

lazy_static!{
    static ref TOKEN_GEN: TokenGenerator = TokenGenerator::new("%");
}

const TAB: &str = "    ";

pub trait GenerateKoopaText {
    fn generate_koopa_text(&self) -> Option<String>;
}

// impl GenerateKoopaText for CompUnit {
//     fn generate_koopa_text(&self) -> Option<String> {
//         FuncDef::to_koopa_text(&self)
//     }
// }

impl GenerateKoopaText for FuncDef {
    fn generate_koopa_text(&self) -> Option<String> {
        unsafe {
            koopa_text.push_str(
                &format!("fun @{}(): ", self.ident)
            );
            self.func_type.generate_koopa_text();
            self.block.generate_koopa_text();
            None
        }
    }
}

impl GenerateKoopaText for FuncType {
    fn generate_koopa_text(&self) -> Option<String> {
        use FuncType::*;
        unsafe {
            koopa_text.push_str(match self {
                Int => "i32",
            });
            None
        }
    }
}

impl GenerateKoopaText for Block {
    fn generate_koopa_text(&self) -> Option<String> {
        unsafe {
            koopa_text.push_str("{\n");
            self.stmt.generate_koopa_text();
            koopa_text.push_str("}\n");
            None
        }
    }
}

impl GenerateKoopaText for Stmt {
    fn generate_koopa_text(&self) -> Option<String> {
        unsafe {
            koopa_text.push_str("%entry:\n");
            let dst = self.exp.generate_koopa_text().unwrap();
            koopa_text.push_str(
                &format!("{TAB}ret {dst}\n")
            );
            None
        }
    }
}

impl GenerateKoopaText for Number {
    fn generate_koopa_text(&self) -> Option<String> {
        Some(self.int_const.to_string())
    }
}

impl GenerateKoopaText for Exp {
    fn generate_koopa_text(&self) -> Option<String> {
        self.unary_exp.generate_koopa_text()
    }
}

impl GenerateKoopaText for PrimaryExp {
    fn generate_koopa_text(&self) -> Option<String> {
        use PrimaryExp::*;
        match &self {
            BracketedExp { ref bexp } => {
                bexp.generate_koopa_text()
            }
            Num { ref number } => {
                number.generate_koopa_text()
            }
        }
    }
}

impl GenerateKoopaText for UnaryExp {
    fn generate_koopa_text(&self) -> Option<String> {
        use UnaryExp::*;
        use UnaryOp::*;
        match &self {
            Primary { ref primary_bexp } => {
                primary_bexp.generate_koopa_text()
            }
            OpExp { ref op, ref bexp } => {
                let src = bexp.generate_koopa_text().unwrap();
                let dst = TOKEN_GEN.generate();
                match op {
                    Plus => {
                        TOKEN_GEN.roll_back();
                        Some(src)
                    }
                    Minus => {
                        let text = format!("{TAB}{dst} = sub 0, {src}\n");
                        unsafe { koopa_text.push_str(&text); }
                        Some(dst)
                    }
                    Not => {
                        let text = format!("{TAB}{dst} = eq 0, {src}\n");
                        unsafe { koopa_text.push_str(&text); }
                        Some(dst)
                    }
                }
            }
        }
    }
}
