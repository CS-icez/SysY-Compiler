use crate::frontend::ast::*;
use super::KoopaTextBuilder;

const TAB: &str = KoopaTextBuilder::TAB;

macro_rules! push_text {
    ($self:tt, $($arg:tt)*) => {
        $self.text.push_str(&format!($($arg)*))
    };
}

macro_rules! push_binary_op {
    ($self:tt, $op:tt, $dst:tt, $src1:tt, $src2:tt) => {
        push_text!($self, "{TAB}{} = {} {}, {}\n",
            $dst, $op, $src1, $src2)
    };
}

macro_rules! impl_build_from_binary_op {
    ($self:tt, $T:ty, $arm1:tt, $arm2:tt, $O:ty,
        op_rule: $($l:tt => $r:tt),*) => {
        impl BuildFrom<$T> for KoopaTextBuilder {
            fn build_from(&mut self, exp: &$T) -> Option<String> {
                use $T::*;
                use $O::*;
                match exp {
                    $arm1(bexp) => self.build_from(bexp.as_ref()),
                    $arm2(bexps, op, bexp) => {
                        let src1 = self.build_from(bexps.as_ref()).unwrap();
                        let src2 = self.build_from(bexp.as_ref()).unwrap();
                        let dst = self.make_token();
                        let op = match op {
                            $($l => $r,)*
                        };
                        push_binary_op!(self, op, dst, src1, src2);
                        Some(dst)
                    }
                }
            }
        }
    };
    ($self:tt, $T:ty, $arm1:tt, $arm2:tt, $op:tt) => {
        impl BuildFrom<$T> for KoopaTextBuilder {
            fn build_from(&mut self, exp: &$T) -> Option<String> {
                use $T::*;
                match exp {
                    $arm1(bexp) => self.build_from(bexp.as_ref()),
                    $arm2(bexps, bexp) => {
                        let src1 = self.build_from(bexps.as_ref()).unwrap();
                        let src2 = self.build_from(bexp.as_ref()).unwrap();
                        let dst = self.make_token();
                        push_binary_op!(self, $op, dst, src1, src2);
                        Some(dst)
                    }
                }
            }
        }
    };
}

pub trait BuildFrom<T> {
    fn build_from(&mut self, target: &T) -> Option<String>;
}

impl BuildFrom<Program> for KoopaTextBuilder {
    fn build_from(&mut self, prog: &Program) -> Option<String> {
        for comp_unit in &prog.0 {
            self.build_from(comp_unit);
        }
        None
    }
}

impl BuildFrom<CompUnit> for KoopaTextBuilder {
    fn build_from(&mut self, comp_unit: &CompUnit) -> Option<String> {
        use CompUnit::*;
        match comp_unit {
            FuncDef(func_def) => {
                self.build_from(func_def);
            }
        }
        None
    }
}

impl BuildFrom<FuncDef> for KoopaTextBuilder {
    fn build_from(&mut self, func_def: &FuncDef) -> Option<String> {
        push_text!(self, "fun @{}(): ", &func_def.1);
        self.build_from(&func_def.0);
        push_text!(self, " ");
        self.build_from(&func_def.2);
        None
    }
}

impl BuildFrom<FuncType> for KoopaTextBuilder {
    fn build_from(&mut self, func_type: &FuncType) -> Option<String> {
        use FuncType::*;
        match func_type {
            Int => push_text!(self, "i32"),
        }
        None
    }
}

impl BuildFrom<Block> for KoopaTextBuilder {
    fn build_from(&mut self, block: &Block) -> Option<String> {
        push_text!(self, "{{\n");
        for block_item in &block.0 {
            self.build_from(block_item);
        }
        push_text!(self, "}}\n");
        None
    }
}

impl BuildFrom<Stmt> for KoopaTextBuilder {
    fn build_from(&mut self, stmt: &Stmt) -> Option<String> {
        push_text!(self, "%entry:\n");
        match stmt {
            Stmt::Return(exp) => {
                let dst = self.build_from(exp).unwrap();
                push_text!(self, "{TAB}ret {dst}\n");
            }
        }
        None
    }
}

impl BuildFrom<Number> for KoopaTextBuilder {
    fn build_from(&mut self, number: &Number) -> Option<String> {
        Some(number.0.to_string())
    }
}

impl BuildFrom<Exp> for KoopaTextBuilder {
    fn build_from(&mut self, exp: &Exp) -> Option<String> {
        self.build_from(&exp.0)
    }
}

impl BuildFrom<PrimaryExp> for KoopaTextBuilder {
    fn build_from(&mut self, primary_exp: &PrimaryExp) -> Option<String> {
        use PrimaryExp::*;
        match primary_exp {
            BracketedExp(bexp) => self.build_from(bexp.as_ref()),
            Number(number) => self.build_from(number),
            LVal(_lval) => panic!("LVal is not supported"),
        }
    }
}

impl BuildFrom<UnaryExp> for KoopaTextBuilder {
    fn build_from(&mut self, unary_exp: &UnaryExp) -> Option<String> {
        use UnaryExp::*;
        use UnaryOp::*;
        match unary_exp {
            Primary(bexp) => self.build_from(bexp.as_ref()),
            OpUnary(op, bexp) => {
                let src = self.build_from(bexp.as_ref()).unwrap();
                let dst;
                match op {
                    Plus => dst = src,
                    Minus => {
                        dst = self.make_token();
                        push_binary_op!(self, "sub", dst, 0, src);
                    }
                    Not => {
                        dst = self.make_token();
                        push_binary_op!(self, "eq", dst, 0, src);
                    }
                }
                Some(dst)
            }
        }
    }
}

impl_build_from_binary_op!(self, MulExp, Unary, MulOpUnary, MulOp,
    op_rule: Mul => "mul", Div => "div", Rem => "mod");

impl_build_from_binary_op!(self, AddExp, Mul, AddOpMul, AddOp,
    op_rule: Add => "add", Sub => "sub");

impl_build_from_binary_op!(self, RelExp, Add, RelOpAdd, RelOp,
    op_rule: Le => "le", Lt => "lt", Ge => "ge", Gt => "gt");

impl_build_from_binary_op!(self, EqExp, Rel, EqOpRel, EqOp,
    op_rule: Eq => "eq", Ne => "ne");
    
impl_build_from_binary_op!(self, LAndExp, Eq, LAndEq, "and");
impl_build_from_binary_op!(self, LOrExp, LAnd, LOrLAnd, "or");

impl BuildFrom<Decl> for KoopaTextBuilder {
    fn build_from(&mut self, decl: &Decl) -> Option<String> {
        use Decl::*;
        match decl {
            ConstDecl(const_decl) => self.build_from(const_decl),
        }
    }
}

impl BuildFrom<ConstDecl> for KoopaTextBuilder {
    fn build_from(&mut self, _: &ConstDecl) -> Option<String> {
        None
    }
}

impl BuildFrom<BlockItem> for KoopaTextBuilder {
    fn build_from(&mut self, block_item: &BlockItem) -> Option<String> {
        use BlockItem::*;
        match block_item {
            Stmt(stmt) => self.build_from(stmt),
            Decl(decl) => self.build_from(decl),
        }
    }
}
