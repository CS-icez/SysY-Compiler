use crate::frontend::ast::*;
use super::KoopaTextBuilder;

const TAB: &str = KoopaTextBuilder::TAB;

macro_rules! null {
    () => {
        "null".to_string()
    };
}

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
            fn build_from(&mut self, exp: &$T, used: bool) -> String {
                use $T::*;
                use $O::*;
                match exp {
                    $arm1(bexp) => self.build_from(bexp.as_ref(), used),
                    $arm2(bexps, op, bexp) => {
                        let src1 = self.build_from(bexps.as_ref(), used);
                        let src2 = self.build_from(bexp.as_ref(), used);
                        if !used {
                            return null!();
                        }
                        let dst = self.make_num();
                        let op = match op {
                            $($l => $r,)*
                        };
                        push_binary_op!(self, op, dst, src1, src2);
                        dst
                    }
                }
            }
        }
    };
}

pub trait BuildFrom<T> {
    fn build_from(&mut self, target: &T, used: bool) -> String;
}

impl BuildFrom<Program> for KoopaTextBuilder {
    fn build_from(&mut self, prog: &Program, _: bool) -> String {
        push_text!(self, "\
            decl @getint(): i32\n\
            decl @getch(): i32\n\
            decl @getarray(*i32): i32\n\
            decl @putint(i32)\n\
            decl @putch(i32)\n\
            decl @putarray(i32, *i32)\n\
            decl @starttime()\n\
            decl @stoptime()\n\n"
        );

        for comp_unit in &prog.0 {
            self.build_from(comp_unit, false);
        }
        null!()
    }
}

impl BuildFrom<CompUnit> for KoopaTextBuilder {
    fn build_from(&mut self, comp_unit: &CompUnit, _: bool) -> String {
        use CompUnit::*;
        match comp_unit {
            GlobalDecl(global_decl) => {
                self.build_from(global_decl, false);
            }
            FuncDef(func_def) => {
                self.build_from(func_def, false);
            }
        }
        null!()
    }
}

impl BuildFrom<FuncDef> for KoopaTextBuilder {
    fn build_from(&mut self, func_def: &FuncDef, _: bool) -> String {
        self.reset_labels();
        let params = func_def.2
            .iter()
            .map(|param| self.build_from(param, false))
            .collect::<Vec<_>>()
            .join(", ");
        let ty = self.build_from(&func_def.0, false);
        push_text!(self, "fun @{}({params}){ty} {{\n", &func_def.1);
        push_text!(self, "%entry:\n");
        func_def.2.iter().for_each(|param| {
            let ident = &param.1;
            push_text!(self, "{TAB}{ident} = alloc i32\n");
            push_text!(self, "{TAB}store {ident}_f, {ident}\n");
        });
        self.build_from(&func_def.3, false);
        match func_def.0 {
            BType::Int => push_text!(self, "{TAB}ret 114514\n"),
            BType::Void => push_text!(self, "{TAB}ret\n"),
        }
        push_text!(self, "}}\n");
        push_text!(self, "\n");
        null!()
    }
}

impl BuildFrom<FuncFParam> for KoopaTextBuilder {
    fn build_from(&mut self, param: &FuncFParam, _: bool) -> String {
        format!("{}_f: i32", &param.1)
    }
}

impl BuildFrom<Block> for KoopaTextBuilder {
    fn build_from(&mut self, block: &Block, _: bool) -> String {
        for block_item in &block.0 {
            self.build_from(block_item, false);
        }
        null!()
    }
}

impl BuildFrom<Stmt> for KoopaTextBuilder {
    fn build_from(&mut self, stmt: &Stmt, _: bool) -> String {
        use Stmt::*;
        match stmt {
            Assign(lval, exp) => {
                let src = self.build_from(exp, true);
                let ident = &lval.0;
                push_text!(self, "{TAB}store {src}, {ident}\n");
            }
            Empty => {}
            Exp(exp) => {
                self.build_from(exp, false);
            }
            Block(block) => {
                self.build_from(block, false);
            }
            If(exp, stmt, opt_stmt) => {
                let src = self.build_from(exp, true);
                let then_label = self.make_token("%then_");
                let else_label = self.make_token("%else_");
                let endif_label = self.make_token("%endif_");
                if let Some(else_stmt) = opt_stmt {
                    push_text!(self, "{TAB}br {src}, {then_label}, {else_label}\n");
                    push_text!(self, "{}:\n", then_label);
                    self.build_from(stmt.as_ref(), false);
                    push_text!(self, "{TAB}jump {endif_label}\n");
                    push_text!(self, "{}:\n", else_label);
                    self.build_from(else_stmt.as_ref(), false);
                    push_text!(self, "{TAB}jump {endif_label}\n");
                    push_text!(self, "{endif_label}:\n");
                } else {
                    push_text!(self, "{TAB}br {src}, {then_label}, {endif_label}\n");
                    push_text!(self, "{}:\n", then_label);
                    self.build_from(stmt.as_ref(), false);
                    push_text!(self, "{TAB}jump {endif_label}\n");
                    push_text!(self, "{endif_label}:\n");
                }
            }
            While(exp, stmt) => {
                self.enter_loop();
                let entry = self.make_token("%cond_");
                let body = self.make_token("%body_");
                let end = self.make_token("%endwhile_");
                push_text!(self, "{TAB}jump {entry}\n");
                push_text!(self, "{}:\n", entry);
                let cond = self.build_from(exp, true);
                push_text!(self, "{TAB}br {cond}, {body}, {end}\n");
                push_text!(self, "{}:\n", body);
                self.build_from(stmt.as_ref(), false);
                push_text!(self, "{TAB}jump {entry}\n");
                push_text!(self, "{end}:\n");
                self.exit_loop();
            }
            Break => {
                let label = self.cur_end_label();
                push_text!(self, "{TAB}jump {label}\n");
                let label = self.make_koopa();
                push_text!(self, "{label}:\n");
            }
            Continue => {
                let label = self.cur_cond_label();
                push_text!(self, "{TAB}jump {label}\n");
                let label = self.make_koopa();
                push_text!(self, "{label}:\n");
            }
            Return(exp) => {
                let dst = self.build_from(exp, true);
                push_text!(self, "{TAB}ret {dst}\n");
                let label = self.make_koopa();
                push_text!(self, "{label}:\n");
            }
        }
        null!()
    }
}

impl BuildFrom<Number> for KoopaTextBuilder {
    fn build_from(&mut self, number: &Number, _: bool) -> String {
        number.0.to_string()
    }
}

impl BuildFrom<Exp> for KoopaTextBuilder {
    fn build_from(&mut self, exp: &Exp, used: bool) -> String {
        self.build_from(&exp.0, used)
    }
}

impl BuildFrom<PrimaryExp> for KoopaTextBuilder {
    fn build_from(&mut self, primary_exp: &PrimaryExp, used: bool) -> String {
        use PrimaryExp::*;
        match primary_exp {
            BracketedExp(bexp) => self.build_from(bexp.as_ref(), used),
            Number(number) => self.build_from(number, used),
            LVal(lval) => self.build_from(lval, used),
        }
    }
}

impl BuildFrom<UnaryExp> for KoopaTextBuilder {
    fn build_from(&mut self, unary_exp: &UnaryExp, used: bool) -> String {
        use UnaryExp::*;
        use UnaryOp::*;
        match unary_exp {
            Primary(bexp) => self.build_from(bexp.as_ref(), used),
            FuncCall(ident, exps) => {
                let args = exps
                    .iter()
                    .map(|exp| self.build_from(exp, true))
                    .collect::<Vec<_>>();
                push_text!(self, "{TAB}");
                let mut dst = null!();
                if used {
                    dst = self.make_num();
                    push_text!(self, "{dst} = ");
                }
                let text = args.join(", ");
                push_text!(self, "call @{ident}({text})\n");
                dst
            }
            OpUnary(op, bexp) => {
                let src = self.build_from(bexp.as_ref(), used);
                if !used {
                    return null!();
                }
                let dst;
                match op {
                    Plus => dst = src,
                    Minus => {
                        dst = self.make_num();
                        push_binary_op!(self, "sub", dst, 0, src);
                    }
                    Not => {
                        dst = self.make_num();
                        push_binary_op!(self, "eq", dst, 0, src);
                    }
                }
                dst
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
    
impl BuildFrom<LAndExp> for KoopaTextBuilder {
    fn build_from(&mut self, land_exp: &LAndExp, used: bool) -> String {
        use LAndExp::*;
        match land_exp {
            Eq(bexp) => self.build_from(bexp.as_ref(), used),
            LAndEq(bexps, bexp) => {
                let then_label = self.make_token("%then_");
                let _ = self.make_token("%else_");
                let endif_label = self.make_token("%endif_");
                if !used {
                    let src1 = self.build_from(bexps.as_ref(), true);
                    push_text!(self, "{TAB}br {src1}, {then_label}, {endif_label}\n");
                    push_text!(self, "{}:\n", then_label);
                    self.build_from(bexp.as_ref(), false);
                    push_text!(self, "{TAB}jump {endif_label}\n");
                    push_text!(self, "{endif_label}:\n");
                    return null!();
                }
                let var = self.make_tmp();
                push_text!(self, "{TAB}{var} = alloc i32\n");
                push_text!(self, "{TAB}store 0, {var}\n");
                let src1 = self.build_from(bexps.as_ref(), true);
                push_text!(self, "{TAB}br {src1}, {then_label}, {endif_label}\n");
                push_text!(self, "{}:\n", then_label);
                let src2 = self.build_from(bexp.as_ref(), true);
                let temp = self.make_num();
                push_text!(self, "{TAB}{temp} = ne 0, {src2}\n");
                push_text!(self, "{TAB}store {temp}, {var}\n");
                push_text!(self, "{TAB}jump {endif_label}\n");
                push_text!(self, "{endif_label}:\n");
                let dst = self.make_num();
                push_text!(self, "{TAB}{dst} = load {var}\n");
                dst
                // let src1 = self.build_from(bexps.as_ref());
                // let src2 = self.build_from(bexp.as_ref());
                // let temp1 = self.make_ident();
                // let temp2 = self.make_ident();
                // let dst = self.make_ident();
                // push_binary_op!(self, "ne", temp1, 0, src1);
                // push_binary_op!(self, "ne", temp2, 0, src2);
                // push_binary_op!(self, "and", dst, temp1, temp2);
                // dst
            }
        }
    }
}

impl BuildFrom<LOrExp> for KoopaTextBuilder {
    fn build_from(&mut self, lor_exp: &LOrExp, used: bool) -> String {
        use LOrExp::*;
        match lor_exp {
            LAnd(bexp) => self.build_from(bexp.as_ref(), used),
            LOrLAnd(bexps, bexp) => {
                let then_label = self.make_token("%then_");
                let _ = self.make_token("%else_");
                let endif_label = self.make_token("%endif_");
                if !used {
                    let src1 = self.build_from(bexps.as_ref(), true);
                    push_text!(self, "{TAB}br {src1}, {endif_label}, {then_label}\n");
                    push_text!(self, "{}:\n", then_label);
                    self.build_from(bexp.as_ref(), false);
                    push_text!(self, "{TAB}jump {endif_label}\n");
                    push_text!(self, "{endif_label}:\n");
                    return null!();
                }
                let var = self.make_tmp();
                push_text!(self, "{TAB}{var} = alloc i32\n");
                push_text!(self, "{TAB}store 1, {var}\n");
                let src1 = self.build_from(bexps.as_ref(), true);
                push_text!(self, "{TAB}br {src1}, {endif_label}, {then_label}\n");
                push_text!(self, "{}:\n", then_label);
                let src2 = self.build_from(bexp.as_ref(), true);
                let temp = self.make_num();
                push_text!(self, "{TAB}{temp} = ne 0, {src2}\n");
                push_text!(self, "{TAB}store {temp}, {var}\n");
                push_text!(self, "{TAB}jump {endif_label}\n");
                push_text!(self, "{endif_label}:\n");
                let dst = self.make_num();
                push_text!(self, "{TAB}{dst} = load {var}\n");
                dst
                // let src1 = self.build_from(bexps.as_ref());
                // let src2 = self.build_from(bexp.as_ref());
                // let temp = self.make_ident();
                // let dst = self.make_ident();
                // push_binary_op!(self, "or", temp, src1, src2);
                // push_binary_op!(self, "ne", dst, 0, temp);
                // dst
            }
        }
    }
}

impl BuildFrom<GlobalDecl> for KoopaTextBuilder {
    fn build_from(&mut self, global_decl: &GlobalDecl, _: bool) -> String {
        use Decl::*;
        use VarDef::*;
        let decl = &global_decl.0;
        match decl {
            ConstDecl(_) => {}
            VarDecl(var_decl) => {
                for var_def in &var_decl.1 {
                    match var_def {
                        Init(ident, init_val) => {
                            let val = self.build_from(init_val, true);
                            push_text!(self, "global {ident} = alloc i32, {val}\n\n");
                        }
                        NoInit(ident) => {
                            push_text!(self, "global {ident} = alloc i32, zeroinit\n\n");
                        }
                    }
                }
            }
        }
        null!()
    }
}

impl BuildFrom<Decl> for KoopaTextBuilder {
    fn build_from(&mut self, decl: &Decl, _: bool) -> String {
        use Decl::*;
        match decl {
            ConstDecl(const_decl) => self.build_from(const_decl, false),
            VarDecl(var_decl) => self.build_from(var_decl, false),
        }
    }
}

impl BuildFrom<ConstDecl> for KoopaTextBuilder {
    fn build_from(&mut self, _: &ConstDecl, _: bool) -> String {
        null!()
    }
}

impl BuildFrom<BType> for KoopaTextBuilder {
    fn build_from(&mut self, btype: &BType, _: bool) -> String {
        use BType::*;
        match btype {
            Int => ": i32",
            Void => "",
        }.to_string()
    }
}

impl BuildFrom<VarDecl> for KoopaTextBuilder {
    fn build_from(&mut self, var_decl: &VarDecl, _: bool) -> String {
        for var_def in &var_decl.1 {
            self.build_from(var_def, false);
        }
        null!()
    }
}

impl BuildFrom<VarDef> for KoopaTextBuilder {
    fn build_from(&mut self, var_def: &VarDef, _: bool) -> String {
        match var_def {
            VarDef::Init(ident, init_val) => {
                let src = self.build_from(init_val, true);
                push_text!(self, "{TAB}{ident} = alloc i32\n");
                push_text!(self, "{TAB}store {src}, {ident}\n");
            }
            VarDef::NoInit(ident) => {
                push_text!(self, "{TAB}{ident} = alloc i32\n");
            }
        }
        null!()
    }
}

impl BuildFrom<InitVal> for KoopaTextBuilder {
    fn build_from(&mut self, init_val: &InitVal, used: bool) -> String {
        match init_val {
            InitVal::Exp(exp) => self.build_from(exp, used),
            InitVal::Number(number) => self.build_from(number, used),
        }
    }
}

impl BuildFrom<BlockItem> for KoopaTextBuilder {
    fn build_from(&mut self, block_item: &BlockItem, _: bool) -> String {
        use BlockItem::*;
        match block_item {
            Stmt(stmt) => self.build_from(stmt, false),
            Decl(decl) => self.build_from(decl, false),
        }
    }
}

impl BuildFrom<LVal> for KoopaTextBuilder {
    fn build_from(&mut self, lval: &LVal, used: bool) -> String {
        if !used {
            return null!();
        }
        let ident = &lval.0;
        let dst = self.make_num();
        push_text!(self, "{TAB}{dst} = load {ident}\n");
        dst
    }
}