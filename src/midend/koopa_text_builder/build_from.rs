//! This module defines and implements the `BuildFrom` trait for `KoopaTextBuilder`.
//! Koopa text generating is done by traversing the AST.

use super::KoopaTextBuilder;
use crate::frontend::ast::*;

// A shortcut. I don't want to type `KoopaTextBuilder::TAB` every time.
// Besides, `format!` cannot capture such expression.
// Hope this feature will be enhanced in Rust 2024.
const TAB: &str = KoopaTextBuilder::TAB;

// Also a shortcut. Just for interface consistency.
macro_rules! null {
    () => {
        "null".to_string()
    };
}

macro_rules! push_text {
    ($self:tt, $($arg:tt)*) => {
        $self.text.push_str(format!($($arg)*).as_ref())
    };
}

// The returned string may be used for different purposes, or not used at all.
pub trait BuildFrom<T> {
    fn build_from(&mut self, target: &T, used: bool) -> String;
}

impl BuildFrom<Program> for KoopaTextBuilder {
    fn build_from(&mut self, prog: &Program, _: bool) -> String {
        // Declare library functions.
        push_text!(
            self,
            "\
            decl @getint(): i32\n\
            decl @getch(): i32\n\
            decl @getarray(*i32): i32\n\
            decl @putint(i32)\n\
            decl @putch(i32)\n\
            decl @putarray(i32, *i32)\n\
            decl @starttime()\n\
            decl @stoptime()\n\n"
        );

        prog.0.iter().for_each(|def| { self.build_from(def, false); });

        null!()
    }
}

impl BuildFrom<CompUnit> for KoopaTextBuilder {
    fn build_from(&mut self, comp_unit: &CompUnit, _: bool) -> String {
        use CompUnit::*;
        match comp_unit {
            VarDecl(var_decl) => {
                self.build_from(var_decl, false);
            }
            FuncDef(func_def) => {
                self.build_from(func_def, false);
            }
        }
        null!()
    }
}

// Variable declaration.

impl BuildFrom<VarDecl> for KoopaTextBuilder {
    fn build_from(&mut self, decl: &VarDecl, _: bool) -> String {
        use VarDef::*;
        let is_global = decl.is_global;
        let is_const = decl.is_const;

        decl.var_defs.iter().for_each(|decl| {
            match decl {
                Scalar(ident, opt_init) => {
                    if is_const {
                        return;
                    }
                    if is_global {
                        let value = if let Some(init) = opt_init {
                            self.build_from(init, true)
                        } else {
                            "zeroinit".to_string()
                        };
                        push_text!(self, "global {ident} = alloc i32, {value}\n");
                        return;
                    }
                    push_text!(self, "{ident} = alloc i32\n");
                    if let Some(init) = opt_init {
                        let value = self.build_from(init, true);
                        push_text!(self, "store {value}, {ident}\n");
                    }
                }
                Array(ident, sizes, opt_inits) => {
                    let size = self.build_from(&sizes[0], true);
                    let arr_size = usize::from_str_radix(&size, 10).unwrap();

                    if is_global {
                        let text = if let Some(inits) = opt_inits {
                            let fill_num = arr_size - inits.len();
                            let mut fill = vec!["0".to_string(); fill_num];
                            let mut init_vec = inits
                                .iter()
                                .map(|init| self.build_from(init, true))
                                .collect::<Vec<_>>();
                            init_vec.append(&mut fill);
                            "{".to_string()
                                + init_vec.join(", ").as_ref()
                                + "}"
                        } else {
                            "zeroinit".to_string()
                        };
                        push_text!(self, "global {ident} = alloc [i32, {size}], {text}\n");
                        return;
                    }

                    push_text!(self, "{ident} = alloc [i32, {size}]\n");
                    if let Some(inits) = opt_inits {
                        (0..arr_size).for_each(|i| {
                            let ptr = self.make_token("%ptr_");
                            let value = if i < inits.len() {
                                self.build_from(&inits[i], true)
                            } else {
                                "0".to_string()
                            };
                            push_text!(self, "{ptr} = getelemptr {ident}, {i}\n");
                            push_text!(self, "store {value}, {ptr}\n");
                        });
                    }
                }
            }
        });

        null!()
    }
}

impl BuildFrom<BType> for KoopaTextBuilder {
    fn build_from(&mut self, btype: &BType, _: bool) -> String {
        use BType::*;
        match btype {
            Int => ": i32",
            Void => "",
        }
        .to_string()
    }
}

// Function definition.

impl BuildFrom<FuncDef> for KoopaTextBuilder {
    fn build_from(&mut self, func_def: &FuncDef, _: bool) -> String {
        self.reset_tokens();

        let ty = self.build_from(&func_def.0, false);
        let name = &func_def.1;
        let params = func_def.2
            .iter()
            .map(|param| self.build_from(param, false))
            .collect::<Vec<_>>()
            .join(", ");

        // fun @name(...) [-> i32] {
        // %entry:
        push_text!(self, "fun @{name}({params}){ty} {{\n");
        push_text!(self, "%entry:\n");

        // Localize parameters.
        func_def.2.iter().for_each(|param| {
            let ident = &param.1;
            push_text!(self, "{TAB}{ident} = alloc i32\n");
            push_text!(self, "{TAB}store {ident}_f, {ident}\n");
        });

        // Build function body.
        self.build_from(&func_def.3, false);

        // HACK:
        // In my implementation, the last return in a function always leaves
        // a dangling label. As I generate Koopa text in one pass, it is hard
        // to address this problem normally. Here is the workaround:
        // just add an extra, unreachable instruction.
        // Koopa library will emit a warning and ignore unreachable blocks.
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

// Block.

impl BuildFrom<Block> for KoopaTextBuilder {
    fn build_from(&mut self, block: &Block, _: bool) -> String {
        block.0.iter().for_each(|item| { self.build_from(item, false); });
        null!()
    }
}

impl BuildFrom<BlockItem> for KoopaTextBuilder {
    fn build_from(&mut self, block_item: &BlockItem, _: bool) -> String {
        use BlockItem::*;
        match block_item {
            Stmt(stmt) => self.build_from(stmt, false),
            VarDecl(decl) => self.build_from(decl, false),
        }
    }
}

// Statement.

impl BuildFrom<Stmt> for KoopaTextBuilder {
    fn build_from(&mut self, stmt: &Stmt, _: bool) -> String {
        use Stmt::*;
        use LVal::*;
        match stmt {
            Assign(lval, exp) => {
                let src = self.build_from(exp, true);
                match lval {
                    Ident(ident) => {
                        push_text!(self, "{TAB}store {src}, {ident}\n");
                    }
                    ArrayElem(ident, exp) => {
                        let idx = self.build_from(exp, true);
                        let ptr = self.make_num();
                        push_text!(self, "{TAB}{ptr} = getelemptr {ident}, {idx}\n");
                        push_text!(self, "{TAB}store {src}, {ptr}\n");
                    }
                }
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
                let then = self.make_token("%then_");
                let else_ = self.make_token("%else_");
                let endif = self.make_token("%endif_");
                if let Some(else_stmt) = opt_stmt {
                    //     br ... then else
                    // then:
                    //     ...
                    //     jump endif
                    // else:
                    //     ...
                    //     jump endif
                    // endif:
                    push_text!(self, "{TAB}br {src}, {then}, {else_}\n");
                    push_text!(self, "{then}:\n");
                    self.build_from(stmt.as_ref(), false);
                    push_text!(self, "{TAB}jump {endif}\n");
                    push_text!(self, "{else_}:\n");
                    self.build_from(else_stmt.as_ref(), false);
                    push_text!(self, "{TAB}jump {endif}\n");
                    push_text!(self, "{endif}:\n");
                } else {
                    //    br ... then endif
                    // then:
                    //     ...
                    //     jump endif
                    // endif:
                    push_text!(self, "{TAB}br {src}, {then}, {endif}\n");
                    push_text!(self, "{then}:\n");
                    self.build_from(stmt.as_ref(), false);
                    push_text!(self, "{TAB}jump {endif}\n");
                    push_text!(self, "{endif}:\n");
                }
            }

            While(exp, stmt) => {
                //     jump entry
                // entry:
                //     (calc cond)
                //     br cond body end
                // body:
                //     ...
                //     jump entry
                self.enter_loop();
                let entry = self.make_token("%cond_");
                let body = self.make_token("%body_");
                let end = self.make_token("%endwhile_");
                push_text!(self, "{TAB}jump {entry}\n");
                push_text!(self, "{entry}:\n");
                let cond = self.build_from(exp, true);
                push_text!(self, "{TAB}br {cond}, {body}, {end}\n");
                push_text!(self, "{body}:\n");
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

// Expression.

impl BuildFrom<Exp> for KoopaTextBuilder {
    fn build_from(&mut self, exp: &Exp, used: bool) -> String {
        use Exp::*;
        match exp {
            LOrExp(exp) => self.build_from(exp, used),
            Number(number) => self.build_from(number, used),
        }
    }
}

impl BuildFrom<LVal> for KoopaTextBuilder {
    fn build_from(&mut self, lval: &LVal, used: bool) -> String {
        use LVal::*;
        if !used {
            return null!();
        }
        match lval {
            Ident(ident) => {
                let dst = self.make_num();
                push_text!(self, "{TAB}{dst} = load {ident}\n");
                dst
            }
            ArrayElem(ident, exp) => {
                let idx = self.build_from(exp, true);
                let ptr = self.make_token("%ptr_");
                push_text!(self, "{TAB}{ptr} = getelemptr {ident}, {idx}\n");
                let dst = self.make_num();
                push_text!(self, "{TAB}{dst} = load {ptr}\n");
                dst
            }
        }
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

impl BuildFrom<Number> for KoopaTextBuilder {
    fn build_from(&mut self, number: &Number, _: bool) -> String {
        number.0.to_string()
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
                    .collect::<Vec<_>>()
                    .join(", ");
                if used {
                    let dst = self.make_num();
                    push_text!(self, "{TAB}{dst} = call @{ident}({args})\n");
                    dst
                } else {
                    push_text!(self, "{TAB}call @{ident}({args})\n");
                    null!()
                }
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
                        push_text!(self, "{TAB}{dst} = sub 0, {src}\n");
                    }
                    Not => {
                        dst = self.make_num();
                        push_text!(self, "{TAB}{dst} = eq 0, {src}\n");
                    }
                }
                dst
            }
        }
    }
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
                        push_text!(self, "{TAB}{dst} = {op} {src1}, {src2}\n");
                        dst
                    }
                }
            }
        }
    };
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
                let then = self.make_token("%then_");
                let _ = self.make_token("%else_");
                let endif = self.make_token("%endif_");
                if !used {
                    //     (calc lhs)
                    //     br lhs then endif
                    // then:
                    //     (calc rhs)
                    //     jump endif
                    // endif:
                    let src1 = self.build_from(bexps.as_ref(), true);
                    push_text!(self, "{TAB}br {src1}, {then}, {endif}\n");
                    push_text!(self, "{then}:\n");
                    self.build_from(bexp.as_ref(), false);
                    push_text!(self, "{TAB}jump {endif}\n");
                    push_text!(self, "{endif}:\n");
                    return null!();
                }
                //     var = alloc i32
                //     store 0, var
                //     (calc lhs)
                //     br lhs then endif
                // then:
                //     (calc rhs)
                //     tmp = ne 0, rhs
                //     store tmp, var
                //     jump endif
                // endif:
                //     dst = load var
                let var = self.make_tmp();
                push_text!(self, "{TAB}{var} = alloc i32\n");
                push_text!(self, "{TAB}store 0, {var}\n");
                let src1 = self.build_from(bexps.as_ref(), true);
                push_text!(self, "{TAB}br {src1}, {then}, {endif}\n");
                push_text!(self, "{then}:\n");
                let src2 = self.build_from(bexp.as_ref(), true);
                let temp = self.make_num();
                push_text!(self, "{TAB}{temp} = ne 0, {src2}\n");
                push_text!(self, "{TAB}store {temp}, {var}\n");
                push_text!(self, "{TAB}jump {endif}\n");
                push_text!(self, "{endif}:\n");
                let dst = self.make_num();
                push_text!(self, "{TAB}{dst} = load {var}\n");
                dst
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
                let then = self.make_token("%then_");
                let _ = self.make_token("%else_");
                let endif = self.make_token("%endif_");
                if !used {
                    //     (calc lhs)
                    //     br lhs endif then
                    // then:
                    //     (calc rhs)
                    //     jump endif
                    // endif:
                    let src1 = self.build_from(bexps.as_ref(), true);
                    push_text!(self, "{TAB}br {src1}, {endif}, {then}\n");
                    push_text!(self, "{then}:\n");
                    self.build_from(bexp.as_ref(), false);
                    push_text!(self, "{TAB}jump {endif}\n");
                    push_text!(self, "{endif}:\n");
                    return null!();
                }
                //     var = alloc i32
                //     store 1, var
                //     (calc lhs)
                //     br lhs endif then
                // then:
                //     (calc rhs)
                //     tmp = ne 0, rhs
                //     store tmp, var
                //     jump endif
                // endif:
                //     dst = load var
                let var = self.make_tmp();
                push_text!(self, "{TAB}{var} = alloc i32\n");
                push_text!(self, "{TAB}store 1, {var}\n");
                let src1 = self.build_from(bexps.as_ref(), true);
                push_text!(self, "{TAB}br {src1}, {endif}, {then}\n");
                push_text!(self, "{then}:\n");
                let src2 = self.build_from(bexp.as_ref(), true);
                let temp = self.make_num();
                push_text!(self, "{TAB}{temp} = ne 0, {src2}\n");
                push_text!(self, "{TAB}store {temp}, {var}\n");
                push_text!(self, "{TAB}jump {endif}\n");
                push_text!(self, "{endif}:\n");
                let dst = self.make_num();
                push_text!(self, "{TAB}{dst} = load {var}\n");
                dst
            }
        }
    }
}
