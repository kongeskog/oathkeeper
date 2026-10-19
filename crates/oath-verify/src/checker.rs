use oath_core::*;
use std::collections::{BTreeMap, BTreeSet};

/// Checks a supplied derivation. Does not execute the evaluator or search for a proof.
pub fn verify(
    program: &CompiledProgram,
    request: &RunRequest,
    certificate: &Certificate,
    limits: &ExecutionLimits,
) -> Result<VerifiedRun, VerificationError> {
    fn check(
        program: &Program,
        request: &RunRequest,
        c: &Certificate,
        l: &ExecutionLimits,
    ) -> Result<VerifiedRun, String> {
        validate_request(program, request, l)?;
        if c.version != VERSION || c.semantics != program.semantics {
            return Err("unsupported certificate version".into());
        }
        if c.program_digest != program.digest()? || c.package_digest != program.package_digest {
            return Err("program/package digest mismatch".into());
        }
        if c.request_digest != digest("request", request)? || &c.limits != l {
            return Err("request/limits mismatch".into());
        }
        if c.nodes.is_empty()
            || c.nodes.len() > l.trace_nodes
            || c.nodes.len() as u64 > l.steps
            || c.root as usize != c.nodes.len() - 1
        {
            return Err("invalid trace root, step budget or node count".into());
        }
        let q = &program.queries[&request.query];
        let args = q
            .args
            .iter()
            .map(|p| request.arguments[&p.name].clone())
            .collect::<Vec<_>>();
        let mut checker = Proof {
            p: program,
            c,
            l,
            used: BTreeSet::new(),
            work: 0,
        };
        let outcome = checker.query(c.root, &request.query, &args, 0)?;
        if checker.used.len() != c.nodes.len() {
            return Err("unreferenced trace nodes".into());
        }
        let mut premises = BTreeSet::new();
        let mut metadata = BTreeSet::new();
        for node in &c.nodes {
            match node {
                TraceNode::Expression { value, .. } => premises.extend(value.evidence()),
                TraceNode::Query { trace } => {
                    metadata.extend(program.queries[&trace.query].meta.clone());
                    for rule in trace.guards.keys() {
                        metadata.extend(program.rules[rule].meta.clone())
                    }
                    for priority in trace.priorities.keys() {
                        metadata.extend(program.priorities[priority].meta.clone())
                    }
                }
            }
        }
        let result = QueryResult {
            query: request.query.clone(),
            outcome,
            used_premises: premises.into_iter().collect(),
            metadata: metadata.into_iter().collect(),
            coverage: program.constants.iter().filter(|(_,c)|matches!(&c.ty,Ty::Named{name,..} if name=="std.provenance.CoverageRecord")).map(|(name,_)|name.clone()).collect(),
        };
        if result != c.result || digest("result", &result)? != c.result_digest {
            return Err("reported result does not follow from the checked derivation".into());
        }
        let encoded = canonical_json(c)?;
        if encoded.len() > l.input_bytes {
            return Err("certificate byte limit exceeded".into());
        }
        Ok(VerifiedRun {
            result,
            certificate_digest: digest("certificate", c)?,
        })
    }
    with_bounded_stack(|| check(program, request, certificate, limits))
        .map_err(VerificationError)?
        .map_err(VerificationError)
}
struct Proof<'a> {
    p: &'a Program,
    c: &'a Certificate,
    l: &'a ExecutionLimits,
    used: BTreeSet<NodeId>,
    work: u64,
}
impl Proof<'_> {
    fn charge(&mut self, depth: usize) -> Result<(), String> {
        self.work += 1;
        if self.work
            > self
                .l
                .steps
                .saturating_mul(16)
                .saturating_add(self.l.trace_nodes as u64 * 8)
        {
            return Err("checker work limit exceeded".into());
        }
        if depth > self.l.call_depth {
            return Err("proof call depth exceeded".into());
        }
        Ok(())
    }
    fn node(&mut self, id: NodeId, depth: usize) -> Result<TraceNode, String> {
        self.charge(depth)?;
        if !self.used.insert(id) {
            return Err(format!("reused or cyclic proof node {id}"));
        }
        self.c
            .nodes
            .get(id as usize)
            .cloned()
            .ok_or_else(|| format!("invalid proof node {id}"))
    }
    fn child(
        &self,
        parent: NodeId,
        children: &[NodeId],
        index: &mut usize,
    ) -> Result<NodeId, String> {
        let child = *children.get(*index).ok_or("omitted expression premise")?;
        *index += 1;
        if child >= parent {
            return Err("proof edges must reference earlier nodes".into());
        }
        Ok(child)
    }
    fn expression_child(
        &mut self,
        parent: NodeId,
        children: &[NodeId],
        index: &mut usize,
        expr: ExprId,
        env: &Env,
        depth: usize,
    ) -> Result<Observation, String> {
        let id = self.child(parent, children, index)?;
        self.expression(id, expr, env, depth)
    }
    fn expression(
        &mut self,
        node: NodeId,
        expr: ExprId,
        env: &Env,
        depth: usize,
    ) -> Result<Observation, String> {
        let TraceNode::Expression {
            expr: claimed_expr,
            env: claimed_env,
            children,
            value: claim,
        } = self.node(node, depth)?
        else {
            return Err("expected expression proof".into());
        };
        if claimed_expr != expr || &claimed_env != env {
            return Err(format!("expression or environment mismatch at node {node}"));
        }
        let e = self.p.expr(expr)?.clone();
        let mut cursor = 0;
        let computed = match e.kind {
            ExprKind::Literal { value } => Observation::known(value),
            ExprKind::Name { name } => {
                if let Some(value) = env.values.get(&name) {
                    value.normalize()
                } else {
                    let c = self.p.constants.get(&name).ok_or("unbound proof name")?;
                    self.expression_child(
                        node,
                        &children,
                        &mut cursor,
                        c.value,
                        &Env::default(),
                        depth + 1,
                    )?
                }
            }
            ExprKind::Unary { op, value } => {
                let a =
                    self.expression_child(node, &children, &mut cursor, value, env, depth + 1)?;
                unary(&op, &a, self.l)?
            }
            ExprKind::Binary { op, left, right } => {
                let a =
                    self.expression_child(node, &children, &mut cursor, left, env, depth + 1)?;
                match (op.as_str(), a.bool()) {
                    ("and", Some(false)) | ("or", Some(true)) => a,
                    _ => {
                        let b = self.expression_child(
                            node,
                            &children,
                            &mut cursor,
                            right,
                            env,
                            depth + 1,
                        )?;
                        binary(&op, &a, &b, self.l)?
                    }
                }
            }
            ExprKind::Field { value, field } => {
                let a =
                    self.expression_child(node, &children, &mut cursor, value, env, depth + 1)?;
                project(&a, &field)?
            }
            ExprKind::List { items } => {
                if items.len() > self.l.collection_entries {
                    return Err("collection proof exceeds limits".into());
                }
                let mut values = Vec::new();
                for expr in items {
                    values.push(self.expression_child(
                        node,
                        &children,
                        &mut cursor,
                        expr,
                        env,
                        depth + 1,
                    )?)
                }
                Observation::known(Value::List { items: values })
            }
            ExprKind::Record {
                name,
                type_args,
                fields,
            } => {
                let mut output = BTreeMap::new();
                for (key, expr) in fields {
                    output.insert(
                        key,
                        self.expression_child(node, &children, &mut cursor, expr, env, depth + 1)?,
                    );
                }
                Observation::known(Value::Record {
                    name,
                    type_args: type_args.iter().map(|t| t.subst(&env.types)).collect(),
                    fields: output,
                })
            }
            ExprKind::Variant {
                name,
                type_args,
                variant,
                fields,
            } => {
                let mut output = Vec::new();
                for expr in fields {
                    output.push(self.expression_child(
                        node,
                        &children,
                        &mut cursor,
                        expr,
                        env,
                        depth + 1,
                    )?)
                }
                Observation::known(Value::Variant {
                    name,
                    type_args: type_args.iter().map(|t| t.subst(&env.types)).collect(),
                    variant,
                    fields: output,
                })
            }
            ExprKind::Call {
                target,
                type_args,
                args,
            } => {
                let mut actual = Vec::new();
                for arg in args {
                    actual.push(self.expression_child(
                        node,
                        &children,
                        &mut cursor,
                        arg,
                        env,
                        depth + 1,
                    )?)
                }
                if BUILTINS.contains(&target.as_str()) {
                    primitive(&target, &actual, self.l)?
                } else if let Some(f) = self.p.functions.get(&target) {
                    if actual.len() != f.args.len() || type_args.len() != f.parameters.len() {
                        return Err("function argument mismatch".into());
                    }
                    let nested = Env {
                        values: f.args.iter().map(|p| p.name.clone()).zip(actual).collect(),
                        types: f
                            .parameters
                            .iter()
                            .cloned()
                            .zip(type_args.iter().map(|t| t.subst(&env.types)))
                            .collect(),
                    };
                    self.expression_child(node, &children, &mut cursor, f.body, &nested, depth + 1)?
                } else {
                    let child = self.child(node, &children, &mut cursor)?;
                    let outcome = self.query(child, &target, &actual, depth + 1)?;
                    query_value(&target, &outcome)
                }
            }
            ExprKind::If { condition, yes, no } => {
                let test =
                    self.expression_child(node, &children, &mut cursor, condition, env, depth + 1)?;
                if let Some(condition) = test.bool() {
                    self.expression_child(
                        node,
                        &children,
                        &mut cursor,
                        if condition { yes } else { no },
                        env,
                        depth + 1,
                    )?
                } else {
                    blocked(&[test])
                }
            }
            ExprKind::Let { name, value, body } => {
                let value =
                    self.expression_child(node, &children, &mut cursor, value, env, depth + 1)?;
                let mut local = env.clone();
                local.values.insert(name, value);
                self.expression_child(node, &children, &mut cursor, body, &local, depth + 1)?
            }
            ExprKind::Match { value, arms } => {
                let scrutinee =
                    self.expression_child(node, &children, &mut cursor, value, env, depth + 1)?;
                if scrutinee.value().is_none() {
                    blocked(&[scrutinee])
                } else {
                    let mut selected = None;
                    for arm in arms {
                        if let Some(bindings) = pattern_bind(&arm.pattern, &scrutinee) {
                            selected = Some((arm.body, bindings));
                            break;
                        }
                    }
                    let (body, bindings) = selected.ok_or("non-exhaustive proof match")?;
                    let mut local = env.clone();
                    for (n, v) in bindings {
                        if n != "_" {
                            local.values.insert(n, v);
                        }
                    }
                    self.expression_child(node, &children, &mut cursor, body, &local, depth + 1)?
                }
            }
            ExprKind::Collection {
                op,
                input,
                initial,
                function,
                type_args,
            } => {
                let input =
                    self.expression_child(node, &children, &mut cursor, input, env, depth + 1)?;
                let mut acc = if let Some(i) = initial {
                    self.expression_child(node, &children, &mut cursor, i, env, depth + 1)?
                } else {
                    boolean(op == "all")
                };
                match input.value() {
                    Some(Value::List { items }) => {
                        let f = self
                            .p
                            .functions
                            .get(&function)
                            .ok_or("unknown proof callback")?
                            .clone();
                        let types: BTreeMap<_, _> = f
                            .parameters
                            .iter()
                            .cloned()
                            .zip(type_args.iter().map(|t| t.subst(&env.types)))
                            .collect();
                        let mut selected = Vec::new();
                        let mut predicates = Vec::new();
                        let mut undecided = Vec::new();
                        for item in items {
                            let actual = if op == "fold" {
                                vec![acc.clone(), item.clone()]
                            } else {
                                vec![item.clone()]
                            };
                            let local = Env {
                                values: f.args.iter().map(|p| p.name.clone()).zip(actual).collect(),
                                types: types.clone(),
                            };
                            let result = self.expression_child(
                                node,
                                &children,
                                &mut cursor,
                                f.body,
                                &local,
                                depth + 1,
                            )?;
                            if op == "map" {
                                selected.push(result)
                            } else if op == "fold" {
                                acc = result
                            } else if op == "filter" {
                                match result.bool() {
                                    Some(true) => selected.push(item),
                                    Some(false) => {}
                                    None => undecided.extend(result.issues()),
                                }
                            } else if op == "all" || op == "any" {
                                let stop = result.bool() == Some(op == "any");
                                predicates.push(result);
                                if stop {
                                    break;
                                }
                            } else {
                                return Err("invalid collection proof".into());
                            }
                        }
                        if op == "map" {
                            Observation::known(Value::List { items: selected })
                        } else if op == "filter" {
                            if undecided.is_empty() {
                                Observation::known(Value::List { items: selected })
                            } else {
                                Observation::Missing {
                                    issues: sorted(undecided),
                                }
                            }
                        } else if op == "fold" {
                            acc
                        } else {
                            let decisive = op == "any";
                            let decisive_result =
                                predicates.iter().any(|p| p.bool() == Some(decisive));
                            if decisive_result {
                                with_evidence(Value::Bool { value: decisive }, &predicates)
                            } else if predicates.iter().any(|p| p.bool().is_none()) {
                                blocked(&predicates)
                            } else {
                                with_evidence(Value::Bool { value: !decisive }, &predicates)
                            }
                        }
                    }
                    _ => blocked(&[input, acc]),
                }
            }
        }
        .normalize();
        if cursor != children.len() {
            return Err(format!("extra expression premises at node {node}"));
        }
        validate_observation(self.p, &computed, &e.ty.subst(&env.types), self.l)?;
        if computed != claim {
            return Err(format!("incorrect expression result at node {node}"));
        }
        Ok(computed)
    }
    fn premise(
        &mut self,
        parent: NodeId,
        child: NodeId,
        expr: ExprId,
        env: &Env,
        depth: usize,
    ) -> Result<Observation, String> {
        if child >= parent {
            return Err("query proof must refer to earlier nodes".into());
        }
        self.expression(child, expr, env, depth)
    }
    fn query(
        &mut self,
        node: NodeId,
        name: &str,
        args: &[Observation],
        depth: usize,
    ) -> Result<QueryOutcome, String> {
        let TraceNode::Query { trace } = self.node(node, depth)? else {
            return Err("expected query proof".into());
        };
        if trace.query != name || trace.args != args {
            return Err("query proof arguments mismatch".into());
        }
        let q = self
            .p
            .queries
            .get(name)
            .ok_or("unknown proof query")?
            .clone();
        if q.args.len() != args.len() {
            return Err("query argument arity mismatch".into());
        }
        let env = Env {
            values: q
                .args
                .iter()
                .map(|p| p.name.clone())
                .zip(args.iter().cloned())
                .collect(),
            types: BTreeMap::new(),
        };
        let scope = self.premise(node, trace.scope, q.scope, &env, depth + 1)?;
        let outcome = if scope.bool() != Some(true) {
            if trace.applies.is_some()
                || !trace.guards.is_empty()
                || !trace.priorities.is_empty()
                || !trace.yields.is_empty()
            {
                return Err("unexpected premises after failed scope".into());
            }
            if scope.bool() == Some(false) {
                QueryOutcome::OutsideCoverage {
                    reasons: vec![format!("{name}: scope condition is false")],
                }
            } else {
                QueryOutcome::Undetermined {
                    stage: "scope".into(),
                    issues: scope.issues(),
                }
            }
        } else {
            let applies = self.premise(
                node,
                trace.applies.ok_or("omitted applicability premise")?,
                q.applies,
                &env,
                depth + 1,
            )?;
            if applies.bool() != Some(true) {
                if !trace.guards.is_empty()
                    || !trace.priorities.is_empty()
                    || !trace.yields.is_empty()
                {
                    return Err("unexpected premises after failed applicability".into());
                }
                if applies.bool() == Some(false) {
                    QueryOutcome::Inapplicable {
                        reasons: vec![format!("{name}: applicability condition is false")],
                    }
                } else {
                    QueryOutcome::Undetermined {
                        stage: "applicability".into(),
                        issues: applies.issues(),
                    }
                }
            } else {
                self.selection(node, &trace, args, depth)?
            }
        };
        if outcome != trace.outcome {
            return Err("incorrect query conclusion".into());
        }
        Ok(outcome)
    }
    fn selection(
        &mut self,
        node: NodeId,
        trace: &QueryTrace,
        args: &[Observation],
        depth: usize,
    ) -> Result<QueryOutcome, String> {
        let rules: BTreeMap<_, _> = self
            .p
            .rules
            .iter()
            .filter(|(_, r)| r.query == trace.query)
            .map(|(n, r)| (n.clone(), r.clone()))
            .collect();
        let priorities: BTreeMap<_, _> = self
            .p
            .priorities
            .iter()
            .filter(|(_, p)| p.query == trace.query)
            .map(|(n, p)| (n.clone(), p.clone()))
            .collect();
        if !trace.guards.keys().eq(rules.keys()) {
            return Err("omitted or invented candidate rules".into());
        }
        if !trace.priorities.keys().eq(priorities.keys()) {
            return Err("omitted or invented priorities".into());
        }
        let mut eligible = BTreeSet::new();
        let mut unresolved = Vec::new();
        for (name, rule) in &rules {
            let env = Env {
                values: rule
                    .bindings
                    .iter()
                    .cloned()
                    .zip(args.iter().cloned())
                    .collect(),
                types: BTreeMap::new(),
            };
            let evidence = self.premise(node, trace.guards[name], rule.when, &env, depth + 2)?;
            if evidence.bool() == Some(true) {
                eligible.insert(name.clone());
            } else if evidence.bool().is_none() {
                unresolved.extend(evidence.issues())
            }
        }
        let mut relation: BTreeMap<String, BTreeSet<String>> =
            rules.keys().map(|n| (n.clone(), BTreeSet::new())).collect();
        let mut priority_issues = Vec::new();
        for (name, priority) in priorities {
            let env = Env {
                values: priority
                    .bindings
                    .iter()
                    .cloned()
                    .zip(args.iter().cloned())
                    .collect(),
                types: BTreeMap::new(),
            };
            let evidence = self.premise(
                node,
                trace.priorities[&name],
                priority.when,
                &env,
                depth + 2,
            )?;
            if evidence.bool() == Some(true) {
                relation
                    .get_mut(&priority.higher)
                    .ok_or("invalid higher endpoint")?
                    .insert(priority.lower);
            } else if evidence.bool().is_none() {
                priority_issues.extend(evidence.issues())
            }
        }
        if !unresolved.is_empty() || !priority_issues.is_empty() {
            if !trace.yields.is_empty() {
                return Err("rule yields supplied despite unresolved conditions".into());
            }
            let stage = if unresolved.is_empty() {
                "priority"
            } else {
                "rule_condition"
            };
            unresolved.extend(priority_issues);
            return Ok(QueryOutcome::Undetermined {
                stage: stage.into(),
                issues: sorted(unresolved),
            });
        }
        // Independent ordering check: topological order and reverse closure, rather than the evaluator's active-root DFS.
        let mut indegree: BTreeMap<String, usize> = rules.keys().map(|n| (n.clone(), 0)).collect();
        for successors in relation.values() {
            for lower in successors {
                *indegree.get_mut(lower).ok_or("invalid lower endpoint")? += 1
            }
        }
        let mut ready: BTreeSet<_> = indegree
            .iter()
            .filter(|(_, d)| **d == 0)
            .map(|(n, _)| n.clone())
            .collect();
        let mut order = Vec::new();
        while let Some(n) = ready.pop_first() {
            self.charge(depth)?;
            order.push(n.clone());
            for lower in &relation[&n] {
                let d = indegree.get_mut(lower).unwrap();
                *d -= 1;
                if *d == 0 {
                    ready.insert(lower.clone());
                }
            }
        }
        if order.len() != rules.len() {
            return Err("cyclic priority relation".into());
        }
        for n in order.into_iter().rev() {
            let next = relation[&n].clone();
            for child in next {
                let descendants = relation[&child].clone();
                for descendant in descendants {
                    self.charge(depth)?;
                    relation.get_mut(&n).unwrap().insert(descendant);
                }
            }
        }
        let maximal: BTreeSet<_> = eligible
            .iter()
            .filter(|candidate| {
                !eligible
                    .iter()
                    .any(|other| relation[other].contains(*candidate))
            })
            .cloned()
            .collect();
        if !trace.yields.keys().eq(maximal.iter()) {
            return Err("selected rule set is incomplete or incorrect".into());
        }
        if maximal.is_empty() {
            return Ok(QueryOutcome::Undetermined {
                stage: "selection".into(),
                issues: vec![Issue::new(
                    &trace.query,
                    "NoSupportingRule",
                    "no applicable supporting rule",
                )],
            });
        }
        let mut alternatives: Vec<Supported> = Vec::new();
        let mut blockers = Vec::new();
        for name in maximal {
            let rule = &rules[&name];
            let env = Env {
                values: rule
                    .bindings
                    .iter()
                    .cloned()
                    .zip(args.iter().cloned())
                    .collect(),
                types: BTreeMap::new(),
            };
            let evidence = self.premise(node, trace.yields[&name], rule.value, &env, depth + 2)?;
            let issues = evidence.issues();
            if !issues.is_empty() {
                blockers.extend(issues);
                continue;
            }
            let value = clean_value(&evidence.value().ok_or("unresolved winner value")?);
            if let Some(existing) = alternatives.iter_mut().find(|a| a.value == value) {
                existing.support.push(name)
            } else {
                alternatives.push(Supported {
                    value,
                    support: vec![name],
                })
            }
        }
        if !blockers.is_empty() {
            return Ok(QueryOutcome::Undetermined {
                stage: "value".into(),
                issues: sorted(blockers),
            });
        }
        if alternatives.len() == 1 {
            let a = alternatives.remove(0);
            Ok(QueryOutcome::Resolved {
                value: a.value,
                support: a.support,
            })
        } else {
            Ok(QueryOutcome::Conflicted { alternatives })
        }
    }
}
