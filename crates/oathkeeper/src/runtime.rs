use oath_core::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn run(
    program: &CompiledProgram,
    request: &RunRequest,
    limits: &ExecutionLimits,
) -> Result<RunBundle, ExecutionError> {
    with_bounded_stack(|| run_on_stack(program, request, limits))
        .map_err(|e| ExecutionError::new("Resource", e, None))?
}
fn run_on_stack(
    program: &CompiledProgram,
    request: &RunRequest,
    limits: &ExecutionLimits,
) -> Result<RunBundle, ExecutionError> {
    validate_request(program, request, limits)
        .map_err(|e| ExecutionError::new("Input", e, None))?;
    let q = &program.queries[&request.query];
    let args = q
        .args
        .iter()
        .map(|p| request.arguments[&p.name].clone())
        .collect();
    let mut machine = Machine {
        program,
        limits,
        nodes: vec![],
        steps: 0,
    };
    let (outcome, root) = machine.query(&request.query, args, 0)?;
    let used_premises = sorted(
        machine
            .nodes
            .iter()
            .flat_map(|n| match n {
                TraceNode::Expression { value, .. } => value.evidence(),
                _ => vec![],
            })
            .collect(),
    );
    let mut metadata = Vec::new();
    for n in &machine.nodes {
        if let TraceNode::Query { trace } = n {
            metadata.extend(program.queries[&trace.query].meta.clone());
            for name in trace.guards.keys() {
                metadata.extend(program.rules[name].meta.clone())
            }
            for name in trace.priorities.keys() {
                metadata.extend(program.priorities[name].meta.clone())
            }
        }
    }
    let result = QueryResult {
        query: request.query.clone(),
        outcome,
        used_premises,
        metadata: sorted(metadata),
        coverage: program.constants.iter().filter(|(_,c)|matches!(&c.ty,Ty::Named{name,..} if name=="std.provenance.CoverageRecord")).map(|(name,_)|name.clone()).collect(),
    };
    let hash_err = |e| ExecutionError::new("Hash", e, None);
    let certificate = Certificate {
        version: VERSION.into(),
        semantics: program.semantics.clone(),
        program_digest: program.digest().map_err(hash_err)?,
        package_digest: program.package_digest.clone(),
        request_digest: digest("request", request).map_err(hash_err)?,
        limits: limits.clone(),
        result_digest: digest("result", &result).map_err(hash_err)?,
        result: result.clone(),
        nodes: machine.nodes,
        root,
    };
    if canonical_json(&certificate).map_err(hash_err)?.len() > limits.input_bytes {
        return Err(ExecutionError::new(
            "Limit",
            "certificate byte limit exceeded",
            None,
        ));
    }
    Ok(RunBundle {
        result,
        certificate,
    })
}
pub(crate) fn constant_value(program: &Program, name: &str) -> Result<Observation, ExecutionError> {
    with_bounded_stack(|| constant_on_stack(program, name))
        .map_err(|e| ExecutionError::new("Resource", e, None))?
}
fn constant_on_stack(program: &Program, name: &str) -> Result<Observation, ExecutionError> {
    let limits = ExecutionLimits::default();
    let mut m = Machine {
        program,
        limits: &limits,
        nodes: vec![],
        steps: 0,
    };
    let c = program
        .constants
        .get(name)
        .ok_or_else(|| ExecutionError::new("Constant", "unknown constant", None))?;
    m.expression(c.value, &Env::default(), 0).map(|x| x.0)
}
struct Machine<'a> {
    program: &'a Program,
    limits: &'a ExecutionLimits,
    nodes: Vec<TraceNode>,
    steps: u64,
}
impl Machine<'_> {
    fn tick(&mut self, depth: usize, span: Option<&Span>) -> Result<(), ExecutionError> {
        self.steps += 1;
        if self.steps > self.limits.steps {
            return Err(ExecutionError::new(
                "Limit",
                "evaluation step limit exceeded",
                span,
            ));
        }
        if depth > self.limits.call_depth {
            return Err(ExecutionError::new("Limit", "call depth exceeded", span));
        }
        Ok(())
    }
    fn emit(&mut self, node: TraceNode) -> Result<NodeId, ExecutionError> {
        if self.nodes.len() >= self.limits.trace_nodes {
            return Err(ExecutionError::new(
                "Limit",
                "trace node limit exceeded",
                None,
            ));
        }
        let id = self.nodes.len() as u32;
        self.nodes.push(node);
        Ok(id)
    }
    fn expression(
        &mut self,
        id: ExprId,
        env: &Env,
        depth: usize,
    ) -> Result<(Observation, NodeId), ExecutionError> {
        let e = self
            .program
            .expr(id)
            .map_err(|e| ExecutionError::new("IR", e, None))?
            .clone();
        self.tick(depth, Some(&e.span))?;
        let mut children = Vec::new();
        let err = |s| ExecutionError::new("Operation", s, Some(&e.span));
        let value = match &e.kind {
            ExprKind::Literal { value } => Observation::known(value.clone()),
            ExprKind::Name { name } => {
                if let Some(v) = env.values.get(name) {
                    v.normalize()
                } else {
                    let c = self
                        .program
                        .constants
                        .get(name)
                        .ok_or_else(|| err(format!("unbound value {name}")))?;
                    let (v, n) = self.expression(c.value, &Env::default(), depth + 1)?;
                    children.push(n);
                    v
                }
            }
            ExprKind::Unary { op, value } => {
                let (v, n) = self.expression(*value, env, depth + 1)?;
                children.push(n);
                unary(op, &v, self.limits).map_err(err)?
            }
            ExprKind::Binary { op, left, right } => {
                let (a, n) = self.expression(*left, env, depth + 1)?;
                children.push(n);
                if (op == "and" && a.bool() == Some(false))
                    || (op == "or" && a.bool() == Some(true))
                {
                    a
                } else {
                    let (b, n) = self.expression(*right, env, depth + 1)?;
                    children.push(n);
                    binary(op, &a, &b, self.limits).map_err(err)?
                }
            }
            ExprKind::Field { value, field } => {
                let (v, n) = self.expression(*value, env, depth + 1)?;
                children.push(n);
                project(&v, field).map_err(err)?
            }
            ExprKind::List { items } => {
                if items.len() > self.limits.collection_entries {
                    return Err(err("collection limit exceeded".into()));
                }
                let mut values = Vec::new();
                for item in items {
                    let (v, n) = self.expression(*item, env, depth + 1)?;
                    children.push(n);
                    values.push(v)
                }
                Observation::known(Value::List { items: values })
            }
            ExprKind::Record {
                name,
                type_args,
                fields,
            } => {
                let mut values = BTreeMap::new();
                for (k, i) in fields {
                    let (v, n) = self.expression(*i, env, depth + 1)?;
                    children.push(n);
                    values.insert(k.clone(), v);
                }
                Observation::known(Value::Record {
                    name: name.clone(),
                    type_args: type_args.iter().map(|t| t.subst(&env.types)).collect(),
                    fields: values,
                })
            }
            ExprKind::Variant {
                name,
                type_args,
                variant,
                fields,
            } => {
                let mut values = Vec::new();
                for i in fields {
                    let (v, n) = self.expression(*i, env, depth + 1)?;
                    children.push(n);
                    values.push(v)
                }
                Observation::known(Value::Variant {
                    name: name.clone(),
                    type_args: type_args.iter().map(|t| t.subst(&env.types)).collect(),
                    variant: variant.clone(),
                    fields: values,
                })
            }
            ExprKind::Call {
                target,
                type_args,
                args,
            } => {
                let mut values = Vec::new();
                for arg in args {
                    let (v, n) = self.expression(*arg, env, depth + 1)?;
                    children.push(n);
                    values.push(v)
                }
                if BUILTINS.contains(&target.as_str()) {
                    primitive(target, &values, self.limits).map_err(err)?
                } else if let Some(f) = self.program.functions.get(target) {
                    let f = f.clone();
                    let call_env = Env {
                        values: f.args.iter().map(|a| a.name.clone()).zip(values).collect(),
                        types: f
                            .parameters
                            .iter()
                            .cloned()
                            .zip(type_args.iter().map(|t| t.subst(&env.types)))
                            .collect(),
                    };
                    let (v, n) = self.expression(f.body, &call_env, depth + 1)?;
                    children.push(n);
                    v
                } else {
                    let (out, n) = self.query(target, values, depth + 1)?;
                    children.push(n);
                    query_value(target, &out)
                }
            }
            ExprKind::If { condition, yes, no } => {
                let (v, n) = self.expression(*condition, env, depth + 1)?;
                children.push(n);
                if let Some(b) = v.bool() {
                    let (out, n) = self.expression(if b { *yes } else { *no }, env, depth + 1)?;
                    children.push(n);
                    out
                } else {
                    blocked(&[v])
                }
            }
            ExprKind::Let { name, value, body } => {
                let (v, n) = self.expression(*value, env, depth + 1)?;
                children.push(n);
                let mut nested = env.clone();
                nested.values.insert(name.clone(), v);
                let (v, n) = self.expression(*body, &nested, depth + 1)?;
                children.push(n);
                v
            }
            ExprKind::Match { value, arms } => {
                let (v, n) = self.expression(*value, env, depth + 1)?;
                children.push(n);
                if v.value().is_none() {
                    blocked(&[v])
                } else {
                    let (arm, bindings) = arms
                        .iter()
                        .find_map(|a| pattern_bind(&a.pattern, &v).map(|b| (a, b)))
                        .ok_or_else(|| err("non-exhaustive match in IR".into()))?;
                    let mut nested = env.clone();
                    nested
                        .values
                        .extend(bindings.into_iter().filter(|(n, _)| n != "_"));
                    let (out, n) = self.expression(arm.body, &nested, depth + 1)?;
                    children.push(n);
                    out
                }
            }
            ExprKind::Collection {
                op,
                input,
                initial,
                function,
                type_args,
            } => {
                let (input, n) = self.expression(*input, env, depth + 1)?;
                children.push(n);
                let mut accumulator = if let Some(initial) = initial {
                    let (v, n) = self.expression(*initial, env, depth + 1)?;
                    children.push(n);
                    v
                } else {
                    boolean(op == "all")
                };
                if let Some(Value::List { items }) = input.value() {
                    let f = self
                        .program
                        .functions
                        .get(function)
                        .ok_or_else(|| err("unknown callback".into()))?
                        .clone();
                    let types: BTreeMap<String, Ty> = f
                        .parameters
                        .iter()
                        .cloned()
                        .zip(type_args.iter().map(|t| t.subst(&env.types)))
                        .collect();
                    let mut output = Vec::new();
                    let mut predicates = Vec::new();
                    let mut issues = Vec::new();
                    for item in items {
                        let actual = if op == "fold" {
                            vec![accumulator.clone(), item.clone()]
                        } else {
                            vec![item.clone()]
                        };
                        let call_env = Env {
                            values: f.args.iter().map(|a| a.name.clone()).zip(actual).collect(),
                            types: types.clone(),
                        };
                        let (v, n) = self.expression(f.body, &call_env, depth + 1)?;
                        children.push(n);
                        match op.as_str() {
                            "map" => output.push(v),
                            "fold" => accumulator = v,
                            "filter" => {
                                if v.bool() == Some(true) {
                                    output.push(item)
                                } else if v.bool().is_none() {
                                    issues.extend(v.issues())
                                }
                            }
                            "all" | "any" => {
                                predicates.push(v.clone());
                                if v.bool() == Some(op == "any") {
                                    break;
                                }
                            }
                            _ => return Err(err("unknown collection operation".into())),
                        }
                    }
                    match op.as_str() {
                        "map" => Observation::known(Value::List { items: output }),
                        "filter" => {
                            if issues.is_empty() {
                                Observation::known(Value::List { items: output })
                            } else {
                                Observation::Missing {
                                    issues: sorted(issues),
                                }
                            }
                        }
                        "fold" => accumulator,
                        "all" | "any" => {
                            let decisive = op == "any";
                            if predicates.iter().any(|p| p.bool() == Some(decisive)) {
                                with_evidence(Value::Bool { value: decisive }, &predicates)
                            } else if predicates.iter().any(|p| p.bool().is_none()) {
                                blocked(&predicates)
                            } else {
                                with_evidence(Value::Bool { value: !decisive }, &predicates)
                            }
                        }
                        _ => unreachable!(),
                    }
                } else {
                    blocked(&[input, accumulator])
                }
            }
        }
        .normalize();
        validate_observation(self.program, &value, &e.ty.subst(&env.types), self.limits)
            .map_err(err)?;
        let n = self.emit(TraceNode::Expression {
            expr: id,
            env: env.clone(),
            children,
            value: value.clone(),
        })?;
        Ok((value, n))
    }
    fn query(
        &mut self,
        name: &str,
        args: Vec<Observation>,
        depth: usize,
    ) -> Result<(QueryOutcome, NodeId), ExecutionError> {
        let q = self
            .program
            .queries
            .get(name)
            .ok_or_else(|| ExecutionError::new("Query", format!("unknown query {name}"), None))?
            .clone();
        self.tick(depth, Some(&q.span))?;
        let env = Env {
            values: q
                .args
                .iter()
                .map(|p| p.name.clone())
                .zip(args.iter().cloned())
                .collect(),
            types: BTreeMap::new(),
        };
        let (scope, scope_node) = self.expression(q.scope, &env, depth + 1)?;
        let mut trace = QueryTrace {
            query: name.into(),
            args: args.clone(),
            scope: scope_node,
            applies: None,
            guards: BTreeMap::new(),
            priorities: BTreeMap::new(),
            yields: BTreeMap::new(),
            outcome: QueryOutcome::Undetermined {
                stage: "scope".into(),
                issues: vec![],
            },
        };
        let outcome = match scope.bool() {
            Some(false) => QueryOutcome::OutsideCoverage {
                reasons: vec![format!("{name}: scope condition is false")],
            },
            None => QueryOutcome::Undetermined {
                stage: "scope".into(),
                issues: scope.issues(),
            },
            Some(true) => {
                let (applies, n) = self.expression(q.applies, &env, depth + 1)?;
                trace.applies = Some(n);
                match applies.bool() {
                    Some(false) => QueryOutcome::Inapplicable {
                        reasons: vec![format!("{name}: applicability condition is false")],
                    },
                    None => QueryOutcome::Undetermined {
                        stage: "applicability".into(),
                        issues: applies.issues(),
                    },
                    Some(true) => self.select(name, &args, &mut trace, depth + 1)?,
                }
            }
        };
        trace.outcome = outcome.clone();
        let node = self.emit(TraceNode::Query { trace })?;
        Ok((outcome, node))
    }
    fn select(
        &mut self,
        name: &str,
        args: &[Observation],
        trace: &mut QueryTrace,
        depth: usize,
    ) -> Result<QueryOutcome, ExecutionError> {
        let rules: Vec<_> = self
            .program
            .rules
            .values()
            .filter(|r| r.query == name)
            .cloned()
            .collect();
        let priorities: Vec<_> = self
            .program
            .priorities
            .values()
            .filter(|p| p.query == name)
            .cloned()
            .collect();
        let mut active = BTreeSet::new();
        let mut issues = Vec::new();
        for r in &rules {
            let env = Env {
                values: r
                    .bindings
                    .iter()
                    .cloned()
                    .zip(args.iter().cloned())
                    .collect(),
                types: BTreeMap::new(),
            };
            let (v, n) = self.expression(r.when, &env, depth + 1)?;
            trace.guards.insert(r.name.clone(), n);
            match v.bool() {
                Some(true) => {
                    active.insert(r.name.clone());
                }
                None => issues.extend(v.issues()),
                _ => {}
            }
        }
        let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut priority_issues = Vec::new();
        for p in priorities {
            let env = Env {
                values: p
                    .bindings
                    .iter()
                    .cloned()
                    .zip(args.iter().cloned())
                    .collect(),
                types: BTreeMap::new(),
            };
            let (v, n) = self.expression(p.when, &env, depth + 1)?;
            trace.priorities.insert(p.name.clone(), n);
            match v.bool() {
                Some(true) => {
                    edges.entry(p.higher).or_default().insert(p.lower);
                }
                None => priority_issues.extend(v.issues()),
                _ => {}
            }
        }
        if !issues.is_empty() || !priority_issues.is_empty() {
            let stage = if issues.is_empty() {
                "priority"
            } else {
                "rule_condition"
            };
            issues.extend(priority_issues);
            return Ok(QueryOutcome::Undetermined {
                stage: stage.into(),
                issues: sorted(issues),
            });
        }
        let mut defeated = BTreeSet::new();
        for higher in &active {
            let mut visited = BTreeSet::new();
            let mut pending: Vec<_> = edges.get(higher).into_iter().flatten().cloned().collect();
            while let Some(lower) = pending.pop() {
                self.tick(depth, None)?;
                if visited.insert(lower.clone()) {
                    if active.contains(&lower) {
                        defeated.insert(lower.clone());
                    }
                    pending.extend(edges.get(&lower).into_iter().flatten().cloned());
                }
            }
        }
        let winners: Vec<_> = active.difference(&defeated).cloned().collect();
        if winners.is_empty() {
            return Ok(QueryOutcome::Undetermined {
                stage: "selection".into(),
                issues: vec![Issue::new(
                    name,
                    "NoSupportingRule",
                    "no applicable supporting rule",
                )],
            });
        }
        let mut answers: Vec<Supported> = Vec::new();
        let mut issues = Vec::new();
        for winner in winners {
            let r = self.program.rules[&winner].clone();
            let env = Env {
                values: r
                    .bindings
                    .iter()
                    .cloned()
                    .zip(args.iter().cloned())
                    .collect(),
                types: BTreeMap::new(),
            };
            let (v, n) = self.expression(r.value, &env, depth + 1)?;
            trace.yields.insert(winner.clone(), n);
            let blockers = v.issues();
            if !blockers.is_empty() {
                issues.extend(blockers);
                continue;
            }
            let value = clean_value(&v.value().ok_or_else(|| {
                ExecutionError::new("IR", "missing resolved rule value", Some(&r.span))
            })?);
            if let Some(answer) = answers.iter_mut().find(|a| a.value == value) {
                answer.support.push(winner)
            } else {
                answers.push(Supported {
                    value,
                    support: vec![winner],
                })
            }
        }
        if !issues.is_empty() {
            return Ok(QueryOutcome::Undetermined {
                stage: "value".into(),
                issues: sorted(issues),
            });
        }
        if answers.len() == 1 {
            let a = answers.remove(0);
            Ok(QueryOutcome::Resolved {
                value: a.value,
                support: a.support,
            })
        } else {
            Ok(QueryOutcome::Conflicted {
                alternatives: answers,
            })
        }
    }
}
