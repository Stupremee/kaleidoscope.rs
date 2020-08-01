//! LLVM Codegen
//!
//! This is mostly copied from the [`inkwell`] examples.
//!
//! [`inkwell`]: https://github.com/TheDan64/inkwell

use crate::{
    error::{CompileError, CompileResult},
    parse::ast::{Expr, ExprKind, LetVar},
    source::FileId,
};
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    values::{FloatValue, FunctionValue, PointerValue},
    FloatPredicate,
};
use lasso::{Spur, ThreadedRodeo};
use smol_str::SmolStr;
use std::{collections::HashMap, sync::Arc};

/// The LLVM compiler.
pub struct Compiler<'ctx> {
    ctx: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    vars: HashMap<Spur, PointerValue<'ctx>>,
    rodeo: Arc<ThreadedRodeo>,
    file: FileId,
}

impl<'ctx> Compiler<'ctx> {
    #[inline]
    fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        // TODO: Keep a list of all method defined in the whole file.
        self.module.get_function(name)
    }

    /// Converts the given operator into a name that will be used for the function.
    #[inline]
    fn unary_fn_name(&self, op: char) -> SmolStr {
        format!("unary{}", op).into()
    }

    /// Converts the given operator into a name that will be used for the function.
    #[inline]
    fn binary_fn_name(&self, op: char) -> SmolStr {
        format!("binary{}", op).into()
    }

    fn create_entry_block_alloca(
        &self,
        fun: FunctionValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let builder = self.ctx.create_builder();

        let entry = fun.get_first_basic_block().unwrap();
        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }
        builder.build_alloca(self.ctx.f64_type(), name)
    }

    fn compile_expr(&mut self, expr: &Expr) -> CompileResult<FloatValue<'ctx>> {
        match &expr.kind {
            ExprKind::Number(x) => Ok(self.ctx.f64_type().const_float(x.into_inner())),
            ExprKind::Var(name) => match self.vars.get(&name.spur) {
                Some(var) => Ok(self
                    .builder
                    .build_load(*var, self.rodeo.resolve(&name.spur))
                    .into_float_value()),
                None => Err(expr.span.locate(self.file, CompileError::UnknownVariable)),
            },
            ExprKind::Unary { op, ref val } => {
                let name = self.unary_fn_name(*op);
                match self.get_function(&name) {
                    Some(fun) => {
                        let val = self.compile_expr(val)?;

                        let result = self.builder.build_call(fun, &[val.into()], "temp");
                        match result.try_as_basic_value().left() {
                            Some(val) => Ok(val.into_float_value()),
                            None => Err(expr.span.locate(self.file, CompileError::InvalidCall)),
                        }
                    }
                    None => Err(expr.span.locate(self.file, CompileError::UnknownOperator)),
                }
            }
            ExprKind::Binary {
                ref left,
                op,
                ref right,
            } => {
                let name = self.binary_fn_name(*op);
                match self.get_function(&name) {
                    Some(fun) => {
                        let lhs = self.compile_expr(left)?;
                        let rhs = self.compile_expr(right)?;

                        match op {
                            '+' => return Ok(self.builder.build_float_add(lhs, rhs, "addtemp")),
                            '-' => return Ok(self.builder.build_float_sub(lhs, rhs, "subtemp")),
                            '*' => return Ok(self.builder.build_float_mul(lhs, rhs, "multemp")),
                            '<' => {
                                let result = self.builder.build_float_compare(
                                    FloatPredicate::ULT,
                                    lhs,
                                    rhs,
                                    "cmptemp",
                                );
                                return Ok(self.builder.build_unsigned_int_to_float(
                                    result,
                                    self.ctx.f64_type(),
                                    "booltmp",
                                ));
                            }
                            _ => {}
                        };

                        let result =
                            self.builder
                                .build_call(fun, &[lhs.into(), rhs.into()], "temp");
                        match result.try_as_basic_value().left() {
                            Some(val) => Ok(val.into_float_value()),
                            None => Err(expr.span.locate(self.file, CompileError::InvalidCall)),
                        }
                    }
                    None => Err(expr.span.locate(self.file, CompileError::UnknownOperator)),
                }
            }
            ExprKind::Call { callee, ref args } => {
                let fun = self
                    .get_function(self.rodeo.resolve(&callee.spur))
                    .ok_or(expr.span.locate(self.file, CompileError::UnknownFunction))?;

                let expected = fun.get_params().len();
                if expected != args.len() {
                    return Err(expr.span.locate(
                        self.file,
                        CompileError::InvalidArguments {
                            expected,
                            found: args.len(),
                        },
                    ));
                }

                let args = args
                    .iter()
                    .map(|arg| self.compile_expr(arg).map(Into::into))
                    .collect::<CompileResult<Vec<_>>>()?;

                let result = self.builder.build_call(fun, args.as_slice(), "calltemp");
                match result.try_as_basic_value().left() {
                    Some(val) => Ok(val.into_float_value()),
                    None => Err(expr.span.locate(self.file, CompileError::InvalidCall)),
                }
            }
            ExprKind::If {
                ref cond,
                ref then,
                ref else_,
            } => {
                // Compile condition and convert it to a boolean
                let cond = self.compile_expr(cond)?;
                let cond = self.builder.build_float_compare(
                    FloatPredicate::ONE,
                    cond,
                    self.ctx.f64_type().const_float(0.0),
                    "ifcond",
                );
                // Get the current function
                let fun = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                // Build blocks, that will be used later
                let then_block = self.ctx.append_basic_block(fun, "then");
                let else_block = self.ctx.append_basic_block(fun, "else");
                let merge_block = self.ctx.append_basic_block(fun, "ifcont");

                // Build a conditional branch
                self.builder
                    .build_conditional_branch(cond, then_block, else_block);

                // Build then block
                self.builder.position_at_end(then_block);
                let then = self.compile_expr(then)?;
                self.builder.build_unconditional_branch(merge_block);

                let then_block = self.builder.get_insert_block().unwrap();

                // Build else block
                self.builder.position_at_end(else_block);
                let else_ = self.compile_expr(else_)?;
                self.builder.build_unconditional_branch(merge_block);

                let else_block = self.builder.get_insert_block().unwrap();

                // Build the merge block
                self.builder.position_at_end(merge_block);
                let phi = self.builder.build_phi(self.ctx.f64_type(), "iftemp");

                phi.add_incoming(&[(&then, then_block), (&else_, else_block)]);
                Ok(phi.as_basic_value().into_float_value())
            }
            ExprKind::For { .. } => todo!(),
            ExprKind::Let { ref vars, body } => {
                let mut old = HashMap::new();

                for LetVar { ref name, ref val } in vars {
                    let spur = name.spur;
                    let init = match val {
                        Some(val) => self.compile_expr(val)?,
                        None => self.ctx.f64_type().const_float(0.0),
                    };

                    let name = self.rodeo.resolve(&spur);
                    let fun = self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_parent()
                        .unwrap();
                    let alloca = self.create_entry_block_alloca(fun, name);
                    self.builder.build_store(alloca, init);

                    if let Some(old_var) = self.vars.remove(&spur) {
                        old.insert(spur, old_var);
                    }
                    self.vars.insert(spur, alloca);
                }

                let body = self.compile_expr(body)?;

                for (k, v) in old {
                    self.vars.insert(k, v);
                }

                Ok(body)
            }
        }
    }
}
