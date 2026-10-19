use crate::parser::{self, Decl, Module};
use oath_core::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn compile(package: &crate::ResolvedPackage) -> Result<CompiledProgram, Vec<Diagnostic>> {
    let program = compile_inner(&package.entry, &package.sources, package.digest.clone())
        .map_err(|e| vec![e])?;
    crate::source_references(&program)
        .map_err(|e| vec![Diagnostic::new("Provenance", e, &Span::default())])?;
    Ok(program)
}
pub fn compile_sources(
    entry: &str,
    sources: &BTreeMap<String, String>,
) -> Result<CompiledProgram, Vec<Diagnostic>> {
    let digest = digest("source-package", sources)
        .map_err(|e| vec![Diagnostic::new("Hash", e, &Span::default())])?;
    compile_inner(entry, sources, digest).map_err(|e| vec![e])
}
fn compile_inner(
    entry: &str,
    sources: &BTreeMap<String, String>,
    package_digest: String,
) -> Result<Program, Diagnostic> {
    with_bounded_stack(|| compile_on_stack(entry, sources, package_digest))
        .map_err(|e| Diagnostic::new("Internal", e, &Span::default()))?
}
fn compile_on_stack(
    entry: &str,
    sources: &BTreeMap<String, String>,
    package_digest: String,
) -> Result<Program, Diagnostic> {
    let mut program = Program {
        semantics: VERSION.into(),
        package_digest,
        ..Program::default()
    };
    let mut modules = BTreeMap::new();
    let mut entry_name = entry.to_string();
    for (file, source) in sources {
        let m = parser::parse(file, source, &mut program.expressions)?;
        if file == entry {
            entry_name = m.name.clone()
        }
        if modules.insert(m.name.clone(), m).is_some() {
            return Err(Diagnostic::new(
                "Module",
                "duplicate module",
                &Span {
                    file: file.clone(),
                    ..Span::default()
                },
            ));
        }
    }
    let mut included = BTreeSet::new();
    let mut pending = vec![entry_name];
    while let Some(name) = pending.pop() {
        if !included.insert(name.clone()) {
            continue;
        }
        let m = modules.get(&name).ok_or_else(|| {
            Diagnostic::new(
                "Import",
                format!("module {name} is not in the locked source snapshot"),
                &Span::default(),
            )
        })?;
        for import in m.imports.values() {
            let target = module_prefix(import, &modules).ok_or_else(|| {
                Diagnostic::new(
                    "Import",
                    format!("unresolved import {import}"),
                    &Span::default(),
                )
            })?;
            pending.push(target);
        }
    }
    modules.retain(|n, _| included.contains(n));
    program.modules = included.into_iter().collect();
    let mut names = BTreeSet::new();
    for module in modules.values() {
        for decl in &module.declarations {
            let (name, span) = match decl {
                Decl::Type(d) => (&d.name, &d.span),
                Decl::Const(d) => (&d.name, &d.span),
                Decl::Function(d) => (&d.name, &d.span),
                Decl::Query(d) => (&d.name, &d.span),
                Decl::Rule(d) => (&d.name, &d.span),
                Decl::Priority(d) => (&d.name, &d.span),
            };
            if !names.insert(name.clone()) {
                return Err(Diagnostic::new(
                    "Name",
                    format!("duplicate declaration {name}"),
                    span,
                ));
            }
            match decl {
                Decl::Type(d) => {
                    program.types.insert(name.clone(), d.clone());
                }
                Decl::Const(d) => {
                    program.constants.insert(name.clone(), d.clone());
                }
                Decl::Function(d) => {
                    program.functions.insert(name.clone(), d.clone());
                }
                Decl::Query(d) => {
                    program.queries.insert(name.clone(), d.clone());
                }
                Decl::Rule(d) => {
                    program.rules.insert(name.clone(), d.clone());
                }
                Decl::Priority(d) => {
                    program.priorities.insert(name.clone(), d.clone());
                }
            }
        }
    }
    let mut c = Checker {
        program,
        modules,
        names,
        constants: BTreeMap::new(),
        depth: 0,
        canonical_types: false,
    };
    c.signatures()?;
    c.normalize_signatures()?;
    c.canonical_types = true;
    c.bodies()?;
    c.graphs()?;
    c.prune()?;
    Ok(c.program)
}
fn module_prefix(name: &str, modules: &BTreeMap<String, Module>) -> Option<String> {
    let mut p = name;
    loop {
        if modules.contains_key(p) {
            return Some(p.into());
        }
        p = p.rsplit_once('.')?.0
    }
}
fn owner(name: &str) -> &str {
    name.rsplit_once('.').map(|x| x.0).unwrap_or("")
}
struct Checker {
    program: Program,
    modules: BTreeMap<String, Module>,
    names: BTreeSet<String>,
    constants: BTreeMap<String, u8>,
    depth: usize,
    canonical_types: bool,
}
impl Checker {
    fn resolve(&self, module: &str, name: &str) -> Option<String> {
        let local = format!("{module}.{name}");
        if self.names.contains(&local) {
            return Some(local);
        }
        let (first, suffix) = name.split_once('.').map_or((name, ""), |(a, b)| (a, b));
        if let Some(base) = self.modules.get(module)?.imports.get(first) {
            let p = if suffix.is_empty() {
                base.clone()
            } else {
                format!("{base}.{suffix}")
            };
            if self.names.contains(&p) {
                return Some(p);
            }
        }
        if self.names.contains(name) {
            return Some(name.into());
        }
        None
    }
    fn ty(&self, module: &str, t: &Ty, params: &[String], span: &Span) -> Result<Ty, Diagnostic> {
        let result = match t {
            Ty::List(t) => Ty::List(Box::new(self.ty(module, t, params, span)?)),
            Ty::Named { name, args } if params.contains(name) && args.is_empty() => {
                Ty::Param(name.clone())
            }
            Ty::Named { name, args } => {
                let n = self
                    .resolve(module, name)
                    .ok_or_else(|| Diagnostic::new("Type", format!("unknown type {name}"), span))?;
                let d =
                    self.program.types.get(&n).ok_or_else(|| {
                        Diagnostic::new("Type", format!("{n} is not a type"), span)
                    })?;
                if d.parameters.len() != args.len() {
                    return Err(Diagnostic::new(
                        "Type",
                        format!("{n} expects {} type arguments", d.parameters.len()),
                        span,
                    ));
                }
                Ty::Named {
                    name: n,
                    args: args
                        .iter()
                        .map(|t| self.ty(module, t, params, span))
                        .collect::<Result<_, _>>()?,
                }
            }
            _ => t.clone(),
        };
        if self.canonical_types {
            self.expand(&result, span)
        } else {
            Ok(result)
        }
    }
    fn unique(&self, items: impl Iterator<Item = String>, span: &Span) -> Result<(), Diagnostic> {
        let mut seen = BTreeSet::new();
        for n in items {
            if !seen.insert(n) {
                return Err(Diagnostic::new("Name", "duplicate parameter/binding", span));
            }
        }
        Ok(())
    }
    fn signatures(&mut self) -> Result<(), Diagnostic> {
        for (mut n, mut d) in self.program.types.clone() {
            let m = owner(&n);
            self.unique(d.parameters.iter().cloned(), &d.span)?;
            d.body = match d.body {
                TypeBody::Record { fields } => TypeBody::Record {
                    fields: fields
                        .into_iter()
                        .map(|(k, t)| Ok((k, self.ty(m, &t, &d.parameters, &d.span)?)))
                        .collect::<Result<_, Diagnostic>>()?,
                },
                TypeBody::Enum { variants } => {
                    if variants.is_empty() {
                        return Err(Diagnostic::new("Type", "enum must have a variant", &d.span));
                    }
                    TypeBody::Enum {
                        variants: variants
                            .into_iter()
                            .map(|(k, ts)| {
                                Ok((
                                    k,
                                    ts.iter()
                                        .map(|t| self.ty(m, t, &d.parameters, &d.span))
                                        .collect::<Result<_, _>>()?,
                                ))
                            })
                            .collect::<Result<_, Diagnostic>>()?,
                    }
                }
                TypeBody::Alias { ty } => TypeBody::Alias {
                    ty: self.ty(m, &ty, &d.parameters, &d.span)?,
                },
            };
            self.program.types.insert(std::mem::take(&mut n), d);
        }
        for (n, mut f) in self.program.functions.clone() {
            let m = owner(&n);
            self.unique(f.parameters.iter().cloned(), &f.span)?;
            self.unique(f.args.iter().map(|a| a.name.clone()), &f.span)?;
            for a in &mut f.args {
                a.ty = self.ty(m, &a.ty, &f.parameters, &f.span)?
            }
            f.returns = self.ty(m, &f.returns, &f.parameters, &f.span)?;
            self.program.functions.insert(n, f);
        }
        for (n, mut q) in self.program.queries.clone() {
            let m = owner(&n);
            self.unique(q.args.iter().map(|a| a.name.clone()), &q.span)?;
            for a in &mut q.args {
                a.ty = self.ty(m, &a.ty, &[], &q.span)?
            }
            q.returns = self.ty(m, &q.returns, &[], &q.span)?;
            q.meta = self.meta(m, q.meta, &q.span)?;
            self.program.queries.insert(n, q);
        }
        for (n, mut r) in self.program.rules.clone() {
            let m = owner(&n);
            r.query = self
                .resolve(m, &r.query)
                .ok_or_else(|| Diagnostic::new("Rule", "unknown target query", &r.span))?;
            self.rule_bindings(&r.query, &r.bindings, &r.span)?;
            r.meta = self.meta(m, r.meta, &r.span)?;
            self.program.rules.insert(n, r);
        }
        for (n, mut p) in self.program.priorities.clone() {
            let m = owner(&n);
            p.query = self
                .resolve(m, &p.query)
                .ok_or_else(|| Diagnostic::new("Priority", "unknown target query", &p.span))?;
            self.rule_bindings(&p.query, &p.bindings, &p.span)?;
            p.higher = self
                .resolve(m, &p.higher)
                .ok_or_else(|| Diagnostic::new("Priority", "unknown higher rule", &p.span))?;
            p.lower = self
                .resolve(m, &p.lower)
                .ok_or_else(|| Diagnostic::new("Priority", "unknown lower rule", &p.span))?;
            for end in [&p.higher, &p.lower] {
                if self
                    .program
                    .rules
                    .get(end)
                    .is_none_or(|r| r.query != p.query)
                {
                    return Err(Diagnostic::new(
                        "Priority",
                        "endpoints must be rules for the same query",
                        &p.span,
                    ));
                }
            }
            p.meta = self.meta(m, p.meta, &p.span)?;
            self.program.priorities.insert(n, p);
        }
        for (n, mut c) in self.program.constants.clone() {
            c.ty = self.ty(owner(&n), &c.ty, &[], &c.span)?;
            self.program.constants.insert(n, c);
        }
        Ok(())
    }
    fn meta(
        &self,
        module: &str,
        meta: Option<String>,
        span: &Span,
    ) -> Result<Option<String>, Diagnostic> {
        meta.map(|m| {
            let n = self
                .resolve(module, &m)
                .ok_or_else(|| Diagnostic::new("Metadata", "unknown metadata constant", span))?;
            if !self.program.constants.contains_key(&n) {
                return Err(Diagnostic::new(
                    "Metadata",
                    "metadata must reference a typed constant",
                    span,
                ));
            }
            Ok(n)
        })
        .transpose()
    }
    fn rule_bindings(
        &self,
        query: &str,
        names: &[String],
        span: &Span,
    ) -> Result<BTreeMap<String, Ty>, Diagnostic> {
        let q = self
            .program
            .queries
            .get(query)
            .ok_or_else(|| Diagnostic::new("Rule", "target is not a query", span))?;
        if q.args.len() != names.len() {
            return Err(Diagnostic::new(
                "Rule",
                "query binding arity mismatch",
                span,
            ));
        }
        self.unique(names.iter().cloned(), span)?;
        Ok(names
            .iter()
            .cloned()
            .zip(q.args.iter().map(|a| a.ty.clone()))
            .collect())
    }
    fn expand(&self, t: &Ty, span: &Span) -> Result<Ty, Diagnostic> {
        fn visit(
            c: &Checker,
            t: &Ty,
            span: &Span,
            active: &mut BTreeSet<String>,
            depth: usize,
        ) -> Result<Ty, Diagnostic> {
            if depth > 128 {
                return Err(Diagnostic::new(
                    "Limit",
                    "expanded type nesting limit exceeded",
                    span,
                ));
            }
            match t {
                Ty::List(t) => Ok(Ty::List(Box::new(visit(c, t, span, active, depth + 1)?))),
                Ty::Named { name, args } => {
                    let args = args
                        .iter()
                        .map(|t| visit(c, t, span, active, depth + 1))
                        .collect::<Result<Vec<_>, _>>()?;
                    let d = c
                        .program
                        .types
                        .get(name)
                        .ok_or_else(|| Diagnostic::new("Type", "unknown nominal type", span))?;
                    if let TypeBody::Alias { ty } = &d.body {
                        if !active.insert(name.clone()) {
                            return Err(Diagnostic::new("Cycle", "type alias cycle", span));
                        }
                        let expanded = visit(
                            c,
                            &ty.subst(&d.parameters.iter().cloned().zip(args).collect()),
                            span,
                            active,
                            depth + 1,
                        )?;
                        active.remove(name);
                        Ok(expanded)
                    } else {
                        Ok(Ty::Named {
                            name: name.clone(),
                            args,
                        })
                    }
                }
                _ => Ok(t.clone()),
            }
        }
        visit(self, t, span, &mut BTreeSet::new(), 0)
    }
    fn normalize_signatures(&mut self) -> Result<(), Diagnostic> {
        let mut types = self.program.types.clone();
        for definition in types.values_mut() {
            match &mut definition.body {
                TypeBody::Record { fields } => {
                    for ty in fields.values_mut() {
                        *ty = self.expand(ty, &definition.span)?;
                    }
                }
                TypeBody::Enum { variants } => {
                    for ty in variants.values_mut().flatten() {
                        *ty = self.expand(ty, &definition.span)?;
                    }
                }
                TypeBody::Alias { ty } => *ty = self.expand(ty, &definition.span)?,
            }
        }
        self.program.types = types;
        let mut functions = self.program.functions.clone();
        for f in functions.values_mut() {
            for a in &mut f.args {
                a.ty = self.expand(&a.ty, &f.span)?;
            }
            f.returns = self.expand(&f.returns, &f.span)?;
        }
        self.program.functions = functions;
        let mut queries = self.program.queries.clone();
        for q in queries.values_mut() {
            for a in &mut q.args {
                a.ty = self.expand(&a.ty, &q.span)?;
            }
            q.returns = self.expand(&q.returns, &q.span)?;
        }
        self.program.queries = queries;
        let mut constants = self.program.constants.clone();
        for c in constants.values_mut() {
            c.ty = self.expand(&c.ty, &c.span)?;
        }
        self.program.constants = constants;
        Ok(())
    }
    fn same(&self, a: &Ty, b: &Ty, span: &Span) -> Result<(), Diagnostic> {
        if self.expand(a, span)? != self.expand(b, span)? {
            Err(Diagnostic::new(
                "Type",
                format!("expected {b:?}, found {a:?}"),
                span,
            ))
        } else {
            Ok(())
        }
    }
    fn constant(&mut self, name: &str) -> Result<Ty, Diagnostic> {
        let c = self.program.constants[name].clone();
        match self.constants.get(name) {
            Some(1) => {
                return Err(Diagnostic::new(
                    "Cycle",
                    "constant dependency cycle",
                    &c.span,
                ));
            }
            Some(2) => return Ok(c.ty),
            _ => {}
        }
        self.constants.insert(name.into(), 1);
        let expected = if c.ty == Ty::Unknown {
            None
        } else {
            Some(c.ty)
        };
        let ty = self.expr(
            c.value,
            owner(name),
            &BTreeMap::new(),
            &[],
            expected.as_ref(),
            false,
        )?;
        self.program.constants.get_mut(name).unwrap().ty = ty.clone();
        self.constants.insert(name.into(), 2);
        Ok(ty)
    }
    fn bodies(&mut self) -> Result<(), Diagnostic> {
        for n in self.program.constants.keys().cloned().collect::<Vec<_>>() {
            self.constant(&n)?;
        }
        for (n, f) in self.program.functions.clone() {
            let env = f
                .args
                .iter()
                .map(|a| (a.name.clone(), a.ty.clone()))
                .collect();
            self.expr(
                f.body,
                owner(&n),
                &env,
                &f.parameters,
                Some(&f.returns),
                false,
            )?;
        }
        for (n, q) in self.program.queries.clone() {
            let env = q
                .args
                .iter()
                .map(|a| (a.name.clone(), a.ty.clone()))
                .collect();
            self.expr(q.scope, owner(&n), &env, &[], Some(&Ty::Bool), true)?;
            self.expr(q.applies, owner(&n), &env, &[], Some(&Ty::Bool), true)?;
        }
        for (n, r) in self.program.rules.clone() {
            let env = self.rule_bindings(&r.query, &r.bindings, &r.span)?;
            let returns = self.program.queries[&r.query].returns.clone();
            self.expr(r.when, owner(&n), &env, &[], Some(&Ty::Bool), true)?;
            self.expr(r.value, owner(&n), &env, &[], Some(&returns), true)?;
        }
        for (n, p) in self.program.priorities.clone() {
            let env = self.rule_bindings(&p.query, &p.bindings, &p.span)?;
            self.expr(p.when, owner(&n), &env, &[], Some(&Ty::Bool), true)?;
        }
        for t in self.program.types.values() {
            if let TypeBody::Alias { ty } = &t.body {
                self.expand(ty, &t.span)?;
            }
        }
        Ok(())
    }
    fn expr(
        &mut self,
        id: ExprId,
        module: &str,
        env: &BTreeMap<String, Ty>,
        params: &[String],
        expected: Option<&Ty>,
        queries: bool,
    ) -> Result<Ty, Diagnostic> {
        if self.depth >= 128 {
            return Err(Diagnostic::new(
                "Limit",
                "typechecking expression depth exceeded",
                &self.program.expressions[id as usize].span,
            ));
        }
        self.depth += 1;
        let result = self.expr_inner(id, module, env, params, expected, queries);
        self.depth -= 1;
        result
    }
    fn expr_inner(
        &mut self,
        id: ExprId,
        module: &str,
        env: &BTreeMap<String, Ty>,
        params: &[String],
        expected: Option<&Ty>,
        queries: bool,
    ) -> Result<Ty, Diagnostic> {
        let mut e = self.program.expressions[id as usize].clone();
        let span = e.span.clone();
        let fail = |s: &str| Diagnostic::new("Type", s, &span);
        let ty = match e.kind.clone() {
            ExprKind::Literal { value } => match value {
                Value::Unit => Ty::Unit,
                Value::Bool { .. } => Ty::Bool,
                Value::Text { .. } => Ty::Text,
                Value::Int { value } => {
                    if expected == Some(&Ty::Rational) {
                        e.kind = ExprKind::Literal {
                            value: Value::Rational {
                                numerator: value,
                                denominator: "1".into(),
                            },
                        };
                        Ty::Rational
                    } else {
                        Ty::Int
                    }
                }
                Value::Rational { .. } => Ty::Rational,
                _ => return Err(fail("composite literals require constructors")),
            },
            ExprKind::Name { name } => {
                let first = name.split('.').next().unwrap();
                if let Some(t) = env.get(first) {
                    if first == name {
                        t.clone()
                    } else {
                        let mut current = self.program.expressions.len() as u32;
                        self.program.expressions.push(Expr {
                            id: current,
                            ty: t.clone(),
                            kind: ExprKind::Name { name: first.into() },
                            span: span.clone(),
                        });
                        let parts: Vec<_> = name.split('.').skip(1).collect();
                        for part in &parts[..parts.len() - 1] {
                            let next = self.program.expressions.len() as u32;
                            self.program.expressions.push(Expr {
                                id: next,
                                ty: Ty::Unknown,
                                kind: ExprKind::Field {
                                    value: current,
                                    field: (*part).into(),
                                },
                                span: span.clone(),
                            });
                            current = next
                        }
                        e.kind = ExprKind::Field {
                            value: current,
                            field: parts.last().unwrap().to_string(),
                        };
                        self.program.expressions[id as usize] = e;
                        return self.expr(id, module, env, params, expected, queries);
                    }
                } else if let Some(n) = self.resolve(module, &name) {
                    if !self.program.constants.contains_key(&n) {
                        return Err(fail(
                            "only constants and local bindings can be used as values",
                        ));
                    }
                    let ty = self.constant(&n)?;
                    e.kind = ExprKind::Name { name: n };
                    ty
                } else {
                    for (split, _) in name.match_indices('.').rev() {
                        if let Some(base) = self.resolve(module, &name[..split])
                            && self.program.constants.contains_key(&base)
                        {
                            let t = self.constant(&base)?;
                            let mut current = self.program.expressions.len() as u32;
                            self.program.expressions.push(Expr {
                                id: current,
                                ty: t,
                                kind: ExprKind::Name { name: base },
                                span: span.clone(),
                            });
                            let fields: Vec<_> = name[split + 1..].split('.').collect();
                            for field in &fields[..fields.len() - 1] {
                                let next = self.program.expressions.len() as u32;
                                self.program.expressions.push(Expr {
                                    id: next,
                                    ty: Ty::Unknown,
                                    kind: ExprKind::Field {
                                        value: current,
                                        field: (*field).into(),
                                    },
                                    span: span.clone(),
                                });
                                current = next
                            }
                            e.kind = ExprKind::Field {
                                value: current,
                                field: fields.last().unwrap().to_string(),
                            };
                            self.program.expressions[id as usize] = e;
                            return self.expr(id, module, env, params, expected, queries);
                        }
                    }
                    if let Some((base, variant)) = name.rsplit_once('.')
                        && let Some(n) = self.resolve(module, base)
                        && self.program.types.contains_key(&n)
                    {
                        e.kind = ExprKind::Variant {
                            name: n,
                            type_args: vec![],
                            variant: variant.into(),
                            fields: vec![],
                        };
                        self.program.expressions[id as usize] = e;
                        return self.expr(id, module, env, params, expected, queries);
                    }
                    return Err(fail(&format!("unknown value {name}")));
                }
            }
            ExprKind::Unary { op, value } => {
                let t = self.expr(
                    value,
                    module,
                    env,
                    params,
                    if op == "not" {
                        Some(&Ty::Bool)
                    } else {
                        expected
                    },
                    queries,
                )?;
                if op != "not" && !matches!(t, Ty::Int | Ty::Rational) {
                    return Err(fail("negation requires a number"));
                }
                t
            }
            ExprKind::Binary { op, left, right } => {
                if op == "and" || op == "or" {
                    self.expr(left, module, env, params, Some(&Ty::Bool), queries)?;
                    self.expr(right, module, env, params, Some(&Ty::Bool), queries)?;
                    Ty::Bool
                } else {
                    let hint = if ["+", "-", "*"].contains(&op.as_str()) {
                        expected.filter(|t| matches!(t, Ty::Int | Ty::Rational | Ty::Text))
                    } else {
                        None
                    };
                    let a = self.expr(left, module, env, params, hint, queries)?;
                    let b = self.expr(right, module, env, params, Some(&a), queries)?;
                    self.same(&a, &b, &span)?;
                    if ["==", "!=", "<", "<=", ">", ">="].contains(&op.as_str()) {
                        Ty::Bool
                    } else {
                        if !matches!(a, Ty::Int | Ty::Rational) && !(op == "+" && a == Ty::Text) {
                            return Err(fail("arithmetic requires matching numeric types"));
                        }
                        if op == "%" && a != Ty::Int {
                            return Err(fail("remainder requires Int"));
                        }
                        if op == "/" { Ty::Rational } else { a }
                    }
                }
            }
            ExprKind::Field { value, field } => {
                let t = self.expr(value, module, env, params, None, queries)?;
                let t = self.expand(&t, &span)?;
                if let Ty::Named { name, args } = t {
                    let d = &self.program.types[&name];
                    if let TypeBody::Record { fields } = &d.body {
                        fields
                            .get(&field)
                            .ok_or_else(|| fail("unknown record field"))?
                            .subst(&d.parameters.iter().cloned().zip(args).collect())
                    } else {
                        return Err(fail("field access requires record"));
                    }
                } else {
                    return Err(fail("field access requires nominal record"));
                }
            }
            ExprKind::List { items } => {
                let mut item = match expected {
                    Some(Ty::List(t)) => Some(*t.clone()),
                    _ => None,
                };
                for i in items {
                    let t = self.expr(i, module, env, params, item.as_ref(), queries)?;
                    item = Some(t)
                }
                Ty::List(Box::new(
                    item.ok_or_else(|| fail("empty list needs a contextual type"))?,
                ))
            }
            ExprKind::Record {
                name,
                type_args,
                fields,
            } => {
                let name = self
                    .resolve(module, &name)
                    .ok_or_else(|| fail("unknown record type"))?;
                let d = self
                    .program
                    .types
                    .get(&name)
                    .ok_or_else(|| fail("constructor requires type"))?
                    .clone();
                let TypeBody::Record { fields: types } = &d.body else {
                    return Err(fail("expected record type"));
                };
                if !fields.keys().eq(types.keys()) {
                    return Err(fail(
                        "record constructor requires exactly its declared fields",
                    ));
                }
                let mut bindings = self.seed(
                    &name,
                    &d.parameters,
                    &type_args,
                    expected,
                    module,
                    params,
                    &span,
                )?;
                for (k, i) in &fields {
                    let pattern = &types[k];
                    let hint = pattern.subst(&bindings);
                    let t = self.expr(
                        *i,
                        module,
                        env,
                        params,
                        if fully_bound(pattern, &bindings) {
                            Some(&hint)
                        } else {
                            None
                        },
                        queries,
                    )?;
                    unify(pattern, &t, &mut bindings, &span)?;
                }
                let args = self.finish(&d.parameters, &bindings, &span)?;
                e.kind = ExprKind::Record {
                    name: name.clone(),
                    type_args: args.clone(),
                    fields,
                };
                Ty::Named { name, args }
            }
            ExprKind::Variant {
                name,
                type_args,
                variant,
                fields,
            } => {
                let name = self
                    .resolve(module, &name)
                    .ok_or_else(|| fail("unknown enum type"))?;
                let d = self.program.types[&name].clone();
                let TypeBody::Enum { variants } = &d.body else {
                    return Err(fail("expected enum type"));
                };
                let types = variants
                    .get(&variant)
                    .ok_or_else(|| fail("unknown enum variant"))?;
                if fields.len() != types.len() {
                    return Err(fail("variant constructor arity mismatch"));
                }
                let mut bindings = self.seed(
                    &name,
                    &d.parameters,
                    &type_args,
                    expected,
                    module,
                    params,
                    &span,
                )?;
                for (i, pattern) in fields.iter().zip(types) {
                    let hint = pattern.subst(&bindings);
                    let t = self.expr(
                        *i,
                        module,
                        env,
                        params,
                        if fully_bound(pattern, &bindings) {
                            Some(&hint)
                        } else {
                            None
                        },
                        queries,
                    )?;
                    unify(pattern, &t, &mut bindings, &span)?;
                }
                let args = self.finish(&d.parameters, &bindings, &span)?;
                e.kind = ExprKind::Variant {
                    name: name.clone(),
                    type_args: args.clone(),
                    variant,
                    fields,
                };
                Ty::Named { name, args }
            }
            ExprKind::Call {
                target,
                type_args,
                args,
            } => {
                if ["map", "filter", "fold", "all", "any"].contains(&target.as_str()) {
                    let n = if target == "fold" { 3 } else { 2 };
                    if args.len() != n {
                        return Err(fail("collection combinator arity mismatch"));
                    }
                    let ExprKind::Name { name: function } = self.program.expressions
                        [*args.last().unwrap() as usize]
                        .kind
                        .clone()
                    else {
                        return Err(fail("collection callback must be a named function"));
                    };
                    e.kind = ExprKind::Collection {
                        op: target,
                        input: args[0],
                        initial: if n == 3 { Some(args[1]) } else { None },
                        function,
                        type_args,
                    };
                    self.program.expressions[id as usize] = e;
                    return self.expr(id, module, env, params, expected, queries);
                }
                if BUILTINS.contains(&target.as_str()) {
                    self.builtin(
                        &target, &args, module, env, params, expected, queries, &span,
                    )?
                } else if let Some(n) = self.resolve(module, &target) {
                    let (generics, formal, returns) =
                        if let Some(f) = self.program.functions.get(&n) {
                            (f.parameters.clone(), f.args.clone(), f.returns.clone())
                        } else if let Some(q) = self.program.queries.get(&n) {
                            if !queries {
                                return Err(fail("functions and constants cannot call queries"));
                            }
                            (vec![], q.args.clone(), q.returns.clone())
                        } else {
                            return Err(fail("target is not callable"));
                        };
                    if args.len() != formal.len() {
                        return Err(fail("call arity mismatch"));
                    }
                    let mut bindings = BTreeMap::new();
                    if !type_args.is_empty() {
                        if type_args.len() != generics.len() {
                            return Err(fail("type argument arity mismatch"));
                        }
                        for (p, t) in generics.iter().zip(&type_args) {
                            bindings.insert(p.clone(), self.ty(module, t, params, &span)?);
                        }
                    }
                    if let Some(t) = expected {
                        unify(&returns, t, &mut bindings, &span)?
                    }
                    for (i, param) in args.iter().zip(&formal) {
                        let hint = param.ty.subst(&bindings);
                        let t = self.expr(
                            *i,
                            module,
                            env,
                            params,
                            if fully_bound(&param.ty, &bindings) {
                                Some(&hint)
                            } else {
                                None
                            },
                            queries,
                        )?;
                        unify(&param.ty, &t, &mut bindings, &span)?;
                    }
                    let type_args = self.finish(&generics, &bindings, &span)?;
                    e.kind = ExprKind::Call {
                        target: n,
                        type_args,
                        args,
                    };
                    returns.subst(&bindings)
                } else if let Some((base, variant)) = target.rsplit_once('.') {
                    if let Some(n) = self.resolve(module, base)
                        && self.program.types.contains_key(&n)
                    {
                        e.kind = ExprKind::Variant {
                            name: n,
                            type_args,
                            variant: variant.into(),
                            fields: args,
                        };
                        self.program.expressions[id as usize] = e;
                        return self.expr(id, module, env, params, expected, queries);
                    }
                    return Err(fail(&format!("unknown callable {target}")));
                } else {
                    return Err(fail(&format!("unknown callable {target}")));
                }
            }
            ExprKind::If { condition, yes, no } => {
                self.expr(condition, module, env, params, Some(&Ty::Bool), queries)?;
                let t = self.expr(yes, module, env, params, expected, queries)?;
                self.expr(no, module, env, params, Some(&t), queries)?;
                t
            }
            ExprKind::Let { name, value, body } => {
                let t = self.expr(value, module, env, params, None, queries)?;
                let mut env = env.clone();
                env.insert(name, t);
                self.expr(body, module, &env, params, expected, queries)?
            }
            ExprKind::Match { value, mut arms } => {
                let t = self.expr(value, module, env, params, None, queries)?;
                let mut result = expected.cloned();
                let mut covered = BTreeSet::new();
                let mut catchall = false;
                for arm in &mut arms {
                    if catchall {
                        return Err(fail("unreachable match arm"));
                    }
                    let mut inner = env.clone();
                    match &mut arm.pattern {
                        Pattern::Wildcard => catchall = true,
                        Pattern::Bind { name } => {
                            inner.insert(name.clone(), t.clone());
                            catchall = true
                        }
                        Pattern::Bool { value } => {
                            self.same(&t, &Ty::Bool, &span)?;
                            if !covered.insert(value.to_string()) {
                                return Err(fail("duplicate match arm"));
                            }
                        }
                        Pattern::Variant {
                            name,
                            variant,
                            bindings,
                        } => {
                            *name = self
                                .resolve(module, name)
                                .ok_or_else(|| fail("unknown pattern type"))?;
                            let Ty::Named { name: n, args } = &t else {
                                return Err(fail("variant pattern requires enum"));
                            };
                            if n != name {
                                return Err(fail("pattern enum mismatch"));
                            }
                            let d = &self.program.types[name];
                            let TypeBody::Enum { variants } = &d.body else {
                                return Err(fail("pattern requires enum"));
                            };
                            let fields = variants
                                .get(variant)
                                .ok_or_else(|| fail("unknown pattern variant"))?;
                            if fields.len() != bindings.len() {
                                return Err(fail("pattern binding arity mismatch"));
                            }
                            self.unique(
                                bindings.iter().filter(|n| n.as_str() != "_").cloned(),
                                &span,
                            )?;
                            let substitutions = d
                                .parameters
                                .iter()
                                .cloned()
                                .zip(args.iter().cloned())
                                .collect();
                            for (n, t) in bindings.iter().zip(fields) {
                                if n != "_" {
                                    inner.insert(n.clone(), t.subst(&substitutions));
                                }
                            }
                            if !covered.insert(variant.clone()) {
                                return Err(fail("duplicate match arm"));
                            }
                        }
                    }
                    result = Some(self.expr(
                        arm.body,
                        module,
                        &inner,
                        params,
                        result.as_ref(),
                        queries,
                    )?);
                }
                if !catchall {
                    let count = match &t {
                        Ty::Bool => 2,
                        Ty::Named { name, .. } => match &self.program.types[name].body {
                            TypeBody::Enum { variants } => variants.len(),
                            _ => 0,
                        },
                        _ => 0,
                    };
                    if count == 0 || covered.len() != count {
                        return Err(fail("non-exhaustive match"));
                    }
                }
                e.kind = ExprKind::Match { value, arms };
                result.ok_or_else(|| fail("empty match"))?
            }
            ExprKind::Collection {
                op,
                input,
                initial,
                function,
                type_args,
            } => {
                let list = self.expr(input, module, env, params, None, queries)?;
                let Ty::List(item) = list else {
                    return Err(fail("collection operation needs List"));
                };
                let function = self
                    .resolve(module, &function)
                    .ok_or_else(|| fail("unknown collection function"))?;
                let f = self
                    .program
                    .functions
                    .get(&function)
                    .ok_or_else(|| fail("collection callback must be a function"))?
                    .clone();
                let mut bindings = BTreeMap::new();
                if !type_args.is_empty() {
                    if type_args.len() != f.parameters.len() {
                        return Err(fail("callback type arity"));
                    }
                    for (n, t) in f.parameters.iter().zip(&type_args) {
                        bindings.insert(n.clone(), self.ty(module, t, params, &span)?);
                    }
                }
                let actual = if let Some(initial) = initial {
                    vec![
                        self.expr(initial, module, env, params, expected, queries)?,
                        *item.clone(),
                    ]
                } else {
                    vec![*item.clone()]
                };
                if actual.len() != f.args.len() {
                    return Err(fail("callback arity mismatch"));
                }
                for (p, t) in f.args.iter().zip(&actual) {
                    unify(&p.ty, t, &mut bindings, &span)?
                }
                let returns = f.returns.subst(&bindings);
                let t = match op.as_str() {
                    "map" => Ty::List(Box::new(returns)),
                    "filter" => {
                        self.same(&returns, &Ty::Bool, &span)?;
                        Ty::List(item)
                    }
                    "all" | "any" => {
                        self.same(&returns, &Ty::Bool, &span)?;
                        Ty::Bool
                    }
                    "fold" => {
                        self.same(&returns, &actual[0], &span)?;
                        returns
                    }
                    _ => return Err(fail("unknown collection operation")),
                };
                e.kind = ExprKind::Collection {
                    op,
                    input,
                    initial,
                    function,
                    type_args: self.finish(&f.parameters, &bindings, &span)?,
                };
                t
            }
        };
        if let Some(want) = expected {
            self.same(&ty, want, &span)?
        }
        e.ty = ty.clone();
        self.program.expressions[id as usize] = e;
        Ok(ty)
    }
    #[allow(clippy::too_many_arguments)]
    fn seed(
        &self,
        name: &str,
        generics: &[String],
        args: &[Ty],
        expected: Option<&Ty>,
        module: &str,
        params: &[String],
        span: &Span,
    ) -> Result<BTreeMap<String, Ty>, Diagnostic> {
        let mut bindings = BTreeMap::new();
        if !args.is_empty() {
            if args.len() != generics.len() {
                return Err(Diagnostic::new("Type", "constructor type arity", span));
            }
            for (p, t) in generics.iter().zip(args) {
                bindings.insert(p.clone(), self.ty(module, t, params, span)?);
            }
        }
        if let Some(Ty::Named { name: n, args }) = expected
            && n == name
        {
            for (p, t) in generics.iter().zip(args) {
                if let Some(old) = bindings.insert(p.clone(), t.clone()) {
                    self.same(&old, t, span)?
                }
            }
        }
        Ok(bindings)
    }
    fn finish(
        &self,
        generics: &[String],
        bindings: &BTreeMap<String, Ty>,
        span: &Span,
    ) -> Result<Vec<Ty>, Diagnostic> {
        generics
            .iter()
            .map(|p| {
                bindings.get(p).cloned().ok_or_else(|| {
                    Diagnostic::new(
                        "Type",
                        format!("cannot infer {p}; supply explicit ::<...> type arguments"),
                        span,
                    )
                })
            })
            .collect()
    }
    #[allow(clippy::too_many_arguments)]
    fn builtin(
        &mut self,
        name: &str,
        args: &[ExprId],
        module: &str,
        env: &BTreeMap<String, Ty>,
        params: &[String],
        expected: Option<&Ty>,
        queries: bool,
        span: &Span,
    ) -> Result<Ty, Diagnostic> {
        let fail = |s: &str| Diagnostic::new("Type", s, span);
        let arity = match name {
            "at" | "append" | "concat" | "range" | "floor_div" | "text_at" => 2,
            _ => 1,
        };
        if args.len() != arity {
            return Err(fail("primitive arity mismatch"));
        }
        let first_hint = match name {
            "to_rational" | "int_text" | "floor_div" | "range" => Some(Ty::Int),
            "floor" => Some(Ty::Rational),
            "text_len" | "text_at" | "parse_int" | "fail" => Some(Ty::Text),
            "append" | "concat" => expected.cloned(),
            _ => None,
        };
        let a = self.expr(args[0], module, env, params, first_hint.as_ref(), queries)?;
        Ok(match name {
            "len" => {
                if !matches!(a, Ty::List(_)) {
                    return Err(fail("len requires List"));
                }
                Ty::Int
            }
            "at" => {
                self.expr(args[1], module, env, params, Some(&Ty::Int), queries)?;
                let Ty::List(t) = a else {
                    return Err(fail("at requires List"));
                };
                *t
            }
            "append" => {
                let Ty::List(t) = &a else {
                    return Err(fail("append requires List"));
                };
                self.expr(args[1], module, env, params, Some(t), queries)?;
                a
            }
            "concat" => {
                if !matches!(a, Ty::List(_)) {
                    return Err(fail("concat requires List"));
                }
                self.expr(args[1], module, env, params, Some(&a), queries)?;
                a
            }
            "range" => {
                self.expr(args[1], module, env, params, Some(&Ty::Int), queries)?;
                Ty::List(Box::new(Ty::Int))
            }
            "floor_div" => {
                self.expr(args[1], module, env, params, Some(&Ty::Int), queries)?;
                Ty::Int
            }
            "text_at" => {
                self.expr(args[1], module, env, params, Some(&Ty::Int), queries)?;
                Ty::Text
            }
            "to_rational" => Ty::Rational,
            "floor" | "text_len" | "parse_int" => Ty::Int,
            "int_text" => Ty::Text,
            "fail" => expected
                .cloned()
                .ok_or_else(|| fail("fail requires a contextual return type"))?,
            _ => return Err(fail("unknown primitive")),
        })
    }
    fn dependencies(&self, root: ExprId, out: &mut BTreeSet<String>) {
        let mut pending = vec![root];
        let mut seen = BTreeSet::new();
        while let Some(root) = pending.pop() {
            if !seen.insert(root) {
                continue;
            }
            let e = &self.program.expressions[root as usize];
            match &e.kind {
                ExprKind::Name { name } if self.program.constants.contains_key(name) => {
                    out.insert(name.clone());
                }
                ExprKind::Call { target, .. } if !BUILTINS.contains(&target.as_str()) => {
                    out.insert(target.clone());
                }
                ExprKind::Collection { function, .. } => {
                    out.insert(function.clone());
                }
                _ => {}
            }
            pending.extend(expr_children(&e.kind));
        }
    }
    fn graphs(&self) -> Result<(), Diagnostic> {
        let mut aliases: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (name, definition) in &self.program.types {
            if let TypeBody::Alias { ty } = &definition.body {
                let mut pending = vec![ty];
                let edges = aliases.entry(name.clone()).or_default();
                while let Some(ty) = pending.pop() {
                    match ty {
                        Ty::Named { name, args } => {
                            if self
                                .program
                                .types
                                .get(name)
                                .is_some_and(|d| matches!(d.body, TypeBody::Alias { .. }))
                            {
                                edges.insert(name.clone());
                            }
                            pending.extend(args);
                        }
                        Ty::List(t) => pending.push(t),
                        _ => {}
                    }
                }
            }
        }
        cycle(&aliases).map_err(|s| {
            Diagnostic::new("Cycle", format!("type alias cycle: {s}"), &Span::default())
        })?;
        let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (n, c) in &self.program.constants {
            self.dependencies(c.value, graph.entry(n.clone()).or_default())
        }
        for (n, f) in &self.program.functions {
            self.dependencies(f.body, graph.entry(n.clone()).or_default())
        }
        for (n, q) in &self.program.queries {
            let set = graph.entry(n.clone()).or_default();
            self.dependencies(q.scope, set);
            self.dependencies(q.applies, set);
        }
        for r in self.program.rules.values() {
            let set = graph.entry(r.query.clone()).or_default();
            self.dependencies(r.when, set);
            self.dependencies(r.value, set);
        }
        for p in self.program.priorities.values() {
            self.dependencies(p.when, graph.entry(p.query.clone()).or_default());
        }
        cycle(&graph).map_err(|s| {
            Diagnostic::new(
                "Cycle",
                format!("executable dependency cycle: {s}"),
                &Span::default(),
            )
        })?;
        let mut priorities = BTreeMap::new();
        for p in self.program.priorities.values() {
            priorities
                .entry(p.higher.clone())
                .or_insert_with(BTreeSet::new)
                .insert(p.lower.clone());
        }
        cycle(&priorities).map_err(|s| {
            Diagnostic::new(
                "Cycle",
                format!("potential priority cycle: {s}"),
                &Span::default(),
            )
        })
    }
    fn prune(&mut self) -> Result<(), Diagnostic> {
        let mut reachable = BTreeSet::new();
        let mut pending = Vec::new();
        pending.extend(self.program.constants.values().map(|c| c.value));
        pending.extend(self.program.functions.values().map(|f| f.body));
        for q in self.program.queries.values() {
            pending.extend([q.scope, q.applies])
        }
        for r in self.program.rules.values() {
            pending.extend([r.when, r.value])
        }
        for p in self.program.priorities.values() {
            pending.push(p.when)
        }
        while let Some(id) = pending.pop() {
            if reachable.insert(id) {
                pending.extend(expr_children(&self.program.expressions[id as usize].kind));
            }
        }
        let map: BTreeMap<_, _> = reachable
            .iter()
            .enumerate()
            .map(|(i, id)| (*id, i as u32))
            .collect();
        let mut expressions = Vec::new();
        for old in reachable {
            let mut e = self.program.expressions[old as usize].clone();
            if e.ty == Ty::Unknown {
                return Err(Diagnostic::new(
                    "Internal",
                    "untyped IR expression",
                    &e.span,
                ));
            }
            e.id = map[&old];
            remap(&mut e.kind, &map);
            expressions.push(e)
        }
        self.program.expressions = expressions;
        for c in self.program.constants.values_mut() {
            c.value = map[&c.value]
        }
        for f in self.program.functions.values_mut() {
            f.body = map[&f.body]
        }
        for q in self.program.queries.values_mut() {
            q.scope = map[&q.scope];
            q.applies = map[&q.applies]
        }
        for r in self.program.rules.values_mut() {
            r.when = map[&r.when];
            r.value = map[&r.value]
        }
        for p in self.program.priorities.values_mut() {
            p.when = map[&p.when]
        }
        Ok(())
    }
}
fn fully_bound(t: &Ty, bindings: &BTreeMap<String, Ty>) -> bool {
    match t {
        Ty::Param(p) => bindings.contains_key(p),
        Ty::Unknown => false,
        Ty::List(t) => fully_bound(t, bindings),
        Ty::Named { args, .. } => args.iter().all(|t| fully_bound(t, bindings)),
        _ => true,
    }
}
fn unify(
    pattern: &Ty,
    actual: &Ty,
    bindings: &mut BTreeMap<String, Ty>,
    span: &Span,
) -> Result<(), Diagnostic> {
    match (pattern, actual) {
        (Ty::Param(p), a) => {
            if let Some(old) = bindings.get(p) {
                if old != a {
                    return Err(Diagnostic::new(
                        "Type",
                        format!("inconsistent inference for {p}: {old:?} vs {a:?}"),
                        span,
                    ));
                }
            } else {
                bindings.insert(p.clone(), a.clone());
            }
            Ok(())
        }
        (Ty::List(p), Ty::List(a)) => unify(p, a, bindings, span),
        (Ty::Named { name: p, args: ps }, Ty::Named { name: a, args: as_ })
            if p == a && ps.len() == as_.len() =>
        {
            for (p, a) in ps.iter().zip(as_) {
                unify(p, a, bindings, span)?
            }
            Ok(())
        }
        (p, a) if p == a => Ok(()),
        _ => Err(Diagnostic::new(
            "Type",
            format!("expected {pattern:?}, found {actual:?}"),
            span,
        )),
    }
}
pub(crate) fn expr_children(e: &ExprKind) -> Vec<ExprId> {
    match e {
        ExprKind::Unary { value, .. } | ExprKind::Field { value, .. } => vec![*value],
        ExprKind::Binary { left, right, .. } => vec![*left, *right],
        ExprKind::List { items }
        | ExprKind::Variant { fields: items, .. }
        | ExprKind::Call { args: items, .. } => items.clone(),
        ExprKind::Record { fields, .. } => fields.values().copied().collect(),
        ExprKind::If { condition, yes, no } => vec![*condition, *yes, *no],
        ExprKind::Let { value, body, .. } => vec![*value, *body],
        ExprKind::Match { value, arms } => std::iter::once(*value)
            .chain(arms.iter().map(|a| a.body))
            .collect(),
        ExprKind::Collection { input, initial, .. } => std::iter::once(*input)
            .chain(initial.iter().copied())
            .collect(),
        _ => vec![],
    }
}
fn remap(e: &mut ExprKind, map: &BTreeMap<ExprId, ExprId>) {
    match e {
        ExprKind::Unary { value, .. } | ExprKind::Field { value, .. } => *value = map[value],
        ExprKind::Binary { left, right, .. } => {
            *left = map[left];
            *right = map[right]
        }
        ExprKind::List { items }
        | ExprKind::Variant { fields: items, .. }
        | ExprKind::Call { args: items, .. } => {
            for i in items {
                *i = map[i]
            }
        }
        ExprKind::Record { fields, .. } => {
            for i in fields.values_mut() {
                *i = map[i]
            }
        }
        ExprKind::If { condition, yes, no } => {
            *condition = map[condition];
            *yes = map[yes];
            *no = map[no]
        }
        ExprKind::Let { value, body, .. } => {
            *value = map[value];
            *body = map[body]
        }
        ExprKind::Match { value, arms } => {
            *value = map[value];
            for a in arms {
                a.body = map[&a.body]
            }
        }
        ExprKind::Collection { input, initial, .. } => {
            *input = map[input];
            if let Some(i) = initial {
                *i = map[i]
            }
        }
        _ => {}
    }
}
fn cycle(graph: &BTreeMap<String, BTreeSet<String>>) -> Result<(), String> {
    let mut incoming: BTreeMap<String, usize> = BTreeMap::new();
    for (name, edges) in graph {
        incoming.entry(name.clone()).or_default();
        for edge in edges {
            *incoming.entry(edge.clone()).or_default() += 1;
        }
    }
    let mut ready: Vec<String> = incoming
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(s, _)| s.clone())
        .collect();
    let mut visited = 0;
    while let Some(name) = ready.pop() {
        visited += 1;
        if let Some(edges) = graph.get(&name) {
            for edge in edges {
                let n = incoming.get_mut(edge).unwrap();
                *n -= 1;
                if *n == 0 {
                    ready.push(edge.clone())
                }
            }
        }
    }
    if visited == incoming.len() {
        Ok(())
    } else {
        Err(incoming.into_iter().find(|(_, n)| *n > 0).unwrap().0)
    }
}
