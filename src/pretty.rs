use crate::parse::ast::{Expr, ExprKind, Item, ItemKind, LetVar};
use lasso::ThreadedRodeo;
use pretty::{DocAllocator, DocBuilder};

/// Trait representing anything that can be turned into a `Doc`.
pub trait Pretty {
    /// Turns `&self` into a `DocBuilder`.
    fn pretty<'alloc, D>(
        &'alloc self,
        alloc: &'alloc D,
        rodeo: &ThreadedRodeo,
    ) -> DocBuilder<'alloc, D>
    where
        D: DocAllocator<'alloc>,
        D::Doc: Clone;
}

impl Pretty for Expr {
    fn pretty<'alloc, D>(
        &'alloc self,
        alloc: &'alloc D,
        rodeo: &ThreadedRodeo,
    ) -> DocBuilder<'alloc, D>
    where
        D: DocAllocator<'alloc>,
        D::Doc: Clone,
    {
        self.kind.pretty(alloc, rodeo)
    }
}

impl Pretty for ExprKind {
    fn pretty<'alloc, D>(
        &'alloc self,
        alloc: &'alloc D,
        rodeo: &ThreadedRodeo,
    ) -> DocBuilder<'alloc, D>
    where
        D: DocAllocator<'alloc>,
        D::Doc: Clone,
    {
        match self {
            ExprKind::Number(x) => alloc.as_string(x),
            ExprKind::Var(name) => alloc.as_string(rodeo.resolve(&name.spur)),
            ExprKind::Unary { op, val } => {
                alloc.as_string(op).append(val.pretty(alloc, rodeo)).group()
            }
            ExprKind::Binary { left, op, right } => left
                .pretty(alloc, rodeo)
                .append(alloc.space())
                .append(alloc.as_string(op))
                .append(alloc.space())
                .append(right.pretty(alloc, rodeo))
                .group(),
            ExprKind::Call { callee, args } => {
                let separator = alloc.text(",").append(alloc.space());
                alloc
                    .as_string(rodeo.resolve(&callee.spur))
                    .append(alloc.text("("))
                    .append(alloc.intersperse(
                        args.into_iter().map(|expr| expr.pretty(alloc, rodeo)),
                        separator,
                    ))
                    .append(alloc.text(")"))
                    .group()
            }
            ExprKind::If { cond, then, else_ } => alloc
                .text("if")
                .append(alloc.space())
                .append(cond.pretty(alloc, rodeo))
                .append(alloc.space())
                .append(alloc.text("then"))
                .append(alloc.hardline().append(then.pretty(alloc, rodeo)).nest(2))
                .append(alloc.hardline().append(alloc.text("else")))
                .append(alloc.hardline().append(else_.pretty(alloc, rodeo)).nest(2))
                .group(),
            ExprKind::For { .. } => todo!(),
            ExprKind::Let { vars, body } => {
                let vars = vars.into_iter().map(|LetVar { name, val }| {
                    let doc = alloc.as_string(rodeo.resolve(&name.spur));
                    if let Some(val) = val {
                        doc.append(alloc.space())
                            .append(alloc.text("="))
                            .append(alloc.space())
                            .append(val.pretty(alloc, rodeo))
                            .group()
                    } else {
                        doc.group()
                    }
                });
                let separator = alloc.text(",").append(alloc.space());
                alloc
                    .text("var")
                    .append(alloc.space())
                    .append(alloc.intersperse(vars, separator))
                    .append(alloc.space())
                    .append(alloc.text("in"))
                    .append(alloc.hardline())
                    .append(body.pretty(alloc, rodeo).nest(2))
                    .group()
            }
        }
    }
}

impl Pretty for Item {
    fn pretty<'alloc, D>(
        &'alloc self,
        alloc: &'alloc D,
        rodeo: &ThreadedRodeo,
    ) -> DocBuilder<'alloc, D>
    where
        D: DocAllocator<'alloc>,
        D::Doc: Clone,
    {
        self.kind.pretty(alloc, rodeo)
    }
}

impl Pretty for ItemKind {
    fn pretty<'alloc, D>(
        &'alloc self,
        alloc: &'alloc D,
        rodeo: &ThreadedRodeo,
    ) -> DocBuilder<'alloc, D>
    where
        D: DocAllocator<'alloc>,
        D::Doc: Clone,
    {
        match self {
            ItemKind::Function { name, args, body } => {
                let separator = alloc.space();
                alloc
                    .text("def")
                    .append(alloc.space())
                    .append(alloc.as_string(rodeo.resolve(&name.spur)))
                    .append(alloc.text("("))
                    .append(
                        alloc.intersperse(
                            args.into_iter()
                                .map(|name| alloc.as_string(rodeo.resolve(&name.spur))),
                            separator,
                        ),
                    )
                    .append(alloc.text(")"))
                    .group()
                    .append(
                        alloc
                            .hardline()
                            .append(body.pretty(alloc, rodeo))
                            .append(alloc.text(";"))
                            .nest(2),
                    )
            }
            ItemKind::Extern { name, args } => {
                let separator = alloc.space();
                alloc
                    .text("extern")
                    .append(alloc.space())
                    .append(alloc.as_string(rodeo.resolve(&name.spur)))
                    .append(alloc.text("("))
                    .append(
                        alloc.intersperse(
                            args.into_iter()
                                .map(|name| alloc.as_string(rodeo.resolve(&name.spur))),
                            separator,
                        ),
                    )
                    .append(alloc.text(")"))
                    .append(alloc.text(";"))
                    .group()
            }
            ItemKind::Operator {
                op,
                prec,
                is_binary,
                body,
                args,
            } => {
                let separator = alloc.space();
                alloc
                    .text("def")
                    .append(alloc.space())
                    .append(alloc.text(if *is_binary { "binary" } else { "unary" }))
                    .append(alloc.as_string(op))
                    .append(if *is_binary {
                        alloc
                            .space()
                            .append(alloc.as_string(prec))
                            .append(alloc.space())
                    } else {
                        alloc.nil()
                    })
                    .append(alloc.text("("))
                    .append(
                        alloc.intersperse(
                            args.into_iter()
                                .map(|name| alloc.as_string(rodeo.resolve(&name.spur))),
                            separator,
                        ),
                    )
                    .append(alloc.text(")"))
                    .group()
                    .append(
                        alloc
                            .hardline()
                            .append(body.pretty(alloc, rodeo))
                            .append(alloc.text(";"))
                            .nest(2),
                    )
            }
        }
    }
}
