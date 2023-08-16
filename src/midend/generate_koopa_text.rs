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
            koopa_text.push_str(" ");
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
        self.add_exp.generate_koopa_text()
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
            OpUnary { ref op, ref unary_bexp } => {
                let src = unary_bexp.generate_koopa_text().unwrap();
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

impl GenerateKoopaText for MulExp {
    fn generate_koopa_text(&self) -> Option<String> {
        use MulExp::*;
        use MulOp::*;
        match &self {
            Unary { ref unary_bexp } => {
                unary_bexp.generate_koopa_text()
            }
            MulOpUnary { ref bexp, ref op, ref unary_bexp } => {
                let src1 = bexp.generate_koopa_text().unwrap();
                let src2 = unary_bexp.generate_koopa_text().unwrap();
                let dst = TOKEN_GEN.generate();
                let text = match op {
                    Mul => format!("{TAB}{dst} = mul {src1}, {src2}\n"),
                    Div => format!("{TAB}{dst} = div {src1}, {src2}\n"),
                    Rem => format!("{TAB}{dst} = mod {src1}, {src2}\n"),
                };
                unsafe { koopa_text.push_str(&text); }
                Some(dst)
            }
        }
    }
}

impl GenerateKoopaText for AddExp {
    fn generate_koopa_text(&self) -> Option<String> {
        use AddExp::*;
        use AddOp::*;
        match &self {
            Mul { ref mul_bexp } => {
                mul_bexp.generate_koopa_text()
            }
            AddOpMul { ref bexp, ref op, ref mul_bexp } => {
                let src1 = bexp.generate_koopa_text().unwrap();
                let src2 = mul_bexp.generate_koopa_text().unwrap();
                let dst = TOKEN_GEN.generate();
                let text = match op {
                    Add => format!("{TAB}{dst} = add {src1}, {src2}\n"),
                    Sub => format!("{TAB}{dst} = sub {src1}, {src2}\n"),
                };
                unsafe { koopa_text.push_str(&text); }
                Some(dst)
            }
        }
    }
}

impl GenerateKoopaText for RelExp {
    fn generate_koopa_text(&self) -> Option<String> {
        use RelExp::*;
        use RelOp::*;
        match &self {
            Add { ref add_bexp } => {
                add_bexp.generate_koopa_text()
            }
            RelOpAdd { ref bexp, ref op, ref add_bexp } => {
                let src1 = bexp.generate_koopa_text().unwrap();
                let src2 = add_bexp.generate_koopa_text().unwrap();
                let dst = TOKEN_GEN.generate();
                let text = match op {
                    Le => format!("{TAB}{dst} = le {src1}, {src2}\n"),
                    Lt => format!("{TAB}{dst} = lt {src1}, {src2}\n"),
                    Ge => format!("{TAB}{dst} = ge {src1}, {src2}\n"),
                    Gt => format!("{TAB}{dst} = gt {src1}, {src2}\n"),
                };
                unsafe { koopa_text.push_str(&text); }
                Some(dst)
            }
        }
    }
}

impl GenerateKoopaText for EqExp {
    fn generate_koopa_text(&self) -> Option<String> {
        use EqExp::*;
        use EqOp::*;
        match &self {
            Rel { ref rel_bexp } => {
                rel_bexp.generate_koopa_text()
            }
            EqOpRel { ref bexp, ref op, ref rel_bexp } => {
                let src1 = bexp.generate_koopa_text().unwrap();
                let src2 = rel_bexp.generate_koopa_text().unwrap();
                let dst = TOKEN_GEN.generate();
                let text = match op {
                    Eq => format!("{TAB}{dst} = eq {src1}, {src2}\n"),
                    Ne => format!("{TAB}{dst} = ne {src1}, {src2}\n"),
                };
                unsafe { koopa_text.push_str(&text); }
                Some(dst)
            }
        }
    }
}

impl GenerateKoopaText for LAndExp {
    fn generate_koopa_text(&self) -> Option<String> {
        use LAndExp::*;
        match &self {
            Eq { ref eq_bexp } => {
                eq_bexp.generate_koopa_text()
            }
            LAndEq { ref bexp, ref eq_bexp } => {
                let src1 = bexp.generate_koopa_text().unwrap();
                let src2 = eq_bexp.generate_koopa_text().unwrap();
                let temp1 = TOKEN_GEN.generate();
                let temp2 = TOKEN_GEN.generate();
                let dst = TOKEN_GEN.generate();
                unsafe {
                    koopa_text.push_str(&format!("{TAB}{temp1} = ne 0, {src1}\n"));
                    koopa_text.push_str(&format!("{TAB}{temp2} = ne 0, {src2}\n"));
                    koopa_text.push_str(&format!("{TAB}{dst} = and {temp1}, {temp2}\n"));
                }
                Some(dst)
            }
        }
    }
}

impl GenerateKoopaText for LOrExp {
    fn generate_koopa_text(&self) -> Option<String> {
        use LOrExp::*;
        match &self {
            LAnd { ref land_bexp } => {
                land_bexp.generate_koopa_text()
            }
            LOrLAnd { ref bexp, ref land_bexp } => {
                let src1 = bexp.generate_koopa_text().unwrap();
                let src2 = land_bexp.generate_koopa_text().unwrap();
                let temp = TOKEN_GEN.generate();
                let dst = TOKEN_GEN.generate();
                unsafe {
                    koopa_text.push_str(&format!("{TAB}{temp} = or {src1}, {src2}\n"));
                    koopa_text.push_str(&format!("{TAB}{dst} = ne 0, {temp}\n"));
                }
                Some(dst)
            }
        }
    }
}
