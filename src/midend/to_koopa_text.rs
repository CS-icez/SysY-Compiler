use crate::frontend::ast::*;

pub trait ToKoopaText {
    fn to_koopa_text(&self) -> String;
}

// impl ToKoopaText for CompUnit {
//     fn to_koopa_text(&self) -> String {
//         FuncDef::to_koopa_text(&self)
//     }
// }

impl ToKoopaText for FuncDef {
    fn to_koopa_text(&self) -> String {
        "fun @".to_string()
            + &self.ident
            + "(): "
            + &self.func_type.to_koopa_text()
            + " "
            + &self.block.to_koopa_text()
    }
}

impl ToKoopaText for FuncType {
    fn to_koopa_text(&self) -> String {
        use FuncType::*;
        match self {
            Int => "i32",
        }.to_string()
    }
}

impl ToKoopaText for Block {
    fn to_koopa_text(&self) -> String {
        "{\n".to_string()
            + &self.stmt.to_koopa_text()
            + &"}\n"
    }
}

impl ToKoopaText for Stmt {
    fn to_koopa_text(&self) -> String {
        "%entry:".to_string()
            + "\n   ret "
            + &self.number.to_string()
            + "\n"
    }
}