//! Structural safety checks for caller-supplied IR. These do not choose rules.
use crate::*;
use std::collections::BTreeMap;

/// Reserve a bounded worker stack instead of depending on the embedding thread's
/// stack size. The closure has no ambient language inputs; failure is operational.
pub fn with_bounded_stack<T: Send>(f: impl FnOnce() -> T + Send) -> Result<T, String> {
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("oath-bounded".into())
            .stack_size(32 * 1024 * 1024)
            .spawn_scoped(scope, f)
            .map_err(|e| format!("cannot reserve execution stack: {e}"))?;
        worker.join().map_err(|_| "internal worker panic".into())
    })
}

pub fn validate_limits(l: &ExecutionLimits) -> Result<(), String> {
    if l.call_depth > 128 || l.nesting > 128 {
        return Err("v0.1 structural ceiling is 128 for call depth and nesting".into());
    }
    if l.trace_nodes > u32::MAX as usize {
        return Err("trace node limit exceeds identifier space".into());
    }
    Ok(())
}

pub fn expression_children(kind: &ExprKind) -> Vec<ExprId> {
    match kind {
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

pub fn validate_program(p: &Program) -> Result<(), String> {
    if p.semantics != VERSION {
        return Err("unsupported IR semantics".into());
    }
    fn ty(p: &Program, t: &Ty) -> Result<(), String> {
        let mut pending = vec![(t, 0)];
        while let Some((t, depth)) = pending.pop() {
            if depth > 128 {
                return Err("IR type nesting limit".into());
            }
            match t {
                Ty::Unknown => return Err("untyped IR".into()),
                Ty::List(t) => pending.push((t, depth + 1)),
                Ty::Named { name, args } => {
                    let d = p.types.get(name).ok_or("unknown IR type")?;
                    if d.parameters.len() != args.len() {
                        return Err("IR type arity".into());
                    }
                    pending.extend(args.iter().map(|t| (t, depth + 1)));
                }
                _ => {}
            }
        }
        Ok(())
    }
    let meta = |m: &Option<String>| -> Result<(), String> {
        if m.as_ref().is_some_and(|m| !p.constants.contains_key(m)) {
            Err("unknown IR metadata".into())
        } else {
            Ok(())
        }
    };
    for (name, d) in &p.types {
        if name != &d.name {
            return Err("IR type identity mismatch".into());
        }
        match &d.body {
            TypeBody::Record { fields } => {
                for t in fields.values() {
                    ty(p, t)?;
                }
            }
            TypeBody::Enum { variants } => {
                for t in variants.values().flatten() {
                    ty(p, t)?;
                }
            }
            TypeBody::Alias { ty: t } => ty(p, t)?,
        }
    }
    for (n, c) in &p.constants {
        if n != &c.name {
            return Err("IR constant identity mismatch".into());
        }
        p.expr(c.value)?;
        ty(p, &c.ty)?;
    }
    for (n, f) in &p.functions {
        if n != &f.name {
            return Err("IR function identity mismatch".into());
        }
        p.expr(f.body)?;
        ty(p, &f.returns)?;
        for a in &f.args {
            ty(p, &a.ty)?;
        }
    }
    for (n, q) in &p.queries {
        if n != &q.name {
            return Err("IR query identity mismatch".into());
        }
        p.expr(q.scope)?;
        p.expr(q.applies)?;
        meta(&q.meta)?;
        ty(p, &q.returns)?;
        for a in &q.args {
            ty(p, &a.ty)?;
        }
    }
    for (n, r) in &p.rules {
        if n != &r.name {
            return Err("IR rule identity mismatch".into());
        }
        let q = p.queries.get(&r.query).ok_or("unknown IR query")?;
        if q.args.len() != r.bindings.len() {
            return Err("IR rule binding arity".into());
        }
        p.expr(r.when)?;
        p.expr(r.value)?;
        meta(&r.meta)?;
    }
    for (n, r) in &p.priorities {
        if n != &r.name {
            return Err("IR priority identity mismatch".into());
        }
        let q = p.queries.get(&r.query).ok_or("unknown priority query")?;
        if q.args.len() != r.bindings.len() {
            return Err("IR priority binding arity".into());
        }
        for endpoint in [&r.higher, &r.lower] {
            if p.rules
                .get(endpoint)
                .is_none_or(|rule| rule.query != r.query)
            {
                return Err("invalid IR priority endpoint".into());
            }
        }
        p.expr(r.when)?;
        meta(&r.meta)?;
    }
    let mut incoming = vec![0usize; p.expressions.len()];
    let mut edges = BTreeMap::new();
    for (i, e) in p.expressions.iter().enumerate() {
        if e.id as usize != i {
            return Err("IR expression identity mismatch".into());
        }
        ty(p, &e.ty)?;
        let children = expression_children(&e.kind);
        for id in &children {
            p.expr(*id)?;
            incoming[*id as usize] += 1;
        }
        edges.insert(i, children);
        match &e.kind {
            ExprKind::Call {
                target,
                args,
                type_args,
            } if !BUILTINS.contains(&target.as_str()) => {
                if let Some(f) = p.functions.get(target) {
                    if args.len() != f.args.len() || type_args.len() != f.parameters.len() {
                        return Err("IR function call arity".into());
                    }
                } else if let Some(q) = p.queries.get(target) {
                    if args.len() != q.args.len() || !type_args.is_empty() {
                        return Err("IR query call arity".into());
                    }
                } else {
                    return Err("unknown IR call".into());
                }
            }
            ExprKind::Collection { function, .. } if !p.functions.contains_key(function) => {
                return Err("unknown IR callback".into());
            }
            _ => {}
        }
    }
    let mut ready: Vec<_> = incoming
        .iter()
        .enumerate()
        .filter(|(_, n)| **n == 0)
        .map(|(i, _)| i)
        .collect();
    let mut visited = 0;
    while let Some(i) = ready.pop() {
        visited += 1;
        for child in &edges[&i] {
            let n = &mut incoming[*child as usize];
            *n -= 1;
            if *n == 0 {
                ready.push(*child as usize);
            }
        }
    }
    if visited != p.expressions.len() {
        return Err("cyclic IR expression graph".into());
    }
    Ok(())
}
