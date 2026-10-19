use crate::*;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

pub fn integer(s: &str, limits: &ExecutionLimits) -> Result<BigInt, String> {
    let digits = s.strip_prefix('-').unwrap_or(s);
    if digits.is_empty()
        || !digits.bytes().all(|b| b.is_ascii_digit())
        || (digits.len() > 1 && digits.starts_with('0'))
        || s == "-0"
    {
        return Err(format!("noncanonical integer {s:?}"));
    }
    if digits.len() as u64 > limits.integer_bits / 3 + 2 {
        return Err("integer bit limit exceeded".into());
    }
    let n = s.parse::<BigInt>().map_err(|e| e.to_string())?;
    if n.bits() > limits.integer_bits {
        return Err("integer bit limit exceeded".into());
    }
    Ok(n)
}
pub fn number(v: &Value, limits: &ExecutionLimits) -> Result<BigRational, String> {
    match v {
        Value::Int { value } => Ok(BigRational::from_integer(integer(value, limits)?)),
        Value::Rational {
            numerator,
            denominator,
        } => {
            let a = integer(numerator, limits)?;
            let b = integer(denominator, limits)?;
            if b <= BigInt::zero() {
                return Err("rational denominator must be positive".into());
            }
            let r = BigRational::new(a.clone(), b.clone());
            if r.numer() != &a || r.denom() != &b {
                return Err("rational must be reduced".into());
            }
            Ok(r)
        }
        _ => Err("expected numeric value".into()),
    }
}
pub fn rational(r: BigRational, limits: &ExecutionLimits) -> Result<Value, String> {
    if r.numer().bits() > limits.integer_bits || r.denom().bits() > limits.integer_bits {
        return Err("integer bit limit exceeded".into());
    }
    Ok(Value::Rational {
        numerator: r.numer().to_string(),
        denominator: r.denom().to_string(),
    })
}
pub fn int_value(n: BigInt, limits: &ExecutionLimits) -> Result<Value, String> {
    if n.bits() > limits.integer_bits {
        return Err("integer bit limit exceeded".into());
    }
    Ok(Value::Int {
        value: n.to_string(),
    })
}
pub fn int(n: i64) -> Observation {
    Observation::known(Value::Int {
        value: n.to_string(),
    })
}
pub fn boolean(b: bool) -> Observation {
    Observation::known(Value::Bool { value: b })
}
pub fn text(s: impl Into<String>) -> Observation {
    Observation::known(Value::Text { value: s.into() })
}
pub fn decimal(s: &str) -> Result<Value, String> {
    let (a, b) = s.split_once('.').ok_or("expected decimal literal")?;
    if a.len().saturating_add(b.len()) > (ExecutionLimits::default().integer_bits / 3 + 2) as usize
    {
        return Err("decimal literal exceeds integer bit budget".into());
    }
    if a.is_empty()
        || b.is_empty()
        || !a.bytes().chain(b.bytes()).all(|x| x.is_ascii_digit())
        || (a.len() > 1 && a.starts_with('0'))
    {
        return Err("invalid decimal literal".into());
    }
    let n = format!("{a}{b}")
        .parse::<BigInt>()
        .map_err(|e| e.to_string())?;
    let d = BigInt::from(10u8).pow(b.len().try_into().map_err(|_| "decimal too long")?);
    rational(BigRational::new(n, d), &ExecutionLimits::default())
}
impl Observation {
    pub fn known(value: Value) -> Self {
        Self::Known {
            value,
            evidence: Vec::new(),
        }
    }
    pub fn missing(id: impl Into<String>, kind: &str, message: impl Into<String>) -> Self {
        Self::Missing {
            issues: vec![Issue::new(id, kind, message)],
        }
    }
    pub fn value(&self) -> Option<Value> {
        match self.normalize() {
            Self::Known { value, .. } => Some(value),
            _ => None,
        }
    }
    pub fn bool(&self) -> Option<bool> {
        match self.value() {
            Some(Value::Bool { value }) => Some(value),
            _ => None,
        }
    }
    pub fn normalize(&self) -> Self {
        match self {
            Self::Known { value, evidence } => Self::Known {
                value: normalize_value(value),
                evidence: sorted(evidence.clone()),
            },
            Self::Missing { issues } => Self::Missing {
                issues: sorted(issues.clone()),
            },
            Self::Disputed { issue, claims } => {
                if let Some(first) = claims.first()
                    && claims
                        .iter()
                        .all(|c| clean_value(&c.value) == clean_value(&first.value))
                {
                    return Self::Known {
                        value: normalize_value(&first.value),
                        evidence: sorted(claims.iter().flat_map(|c| c.evidence.clone()).collect()),
                    };
                }
                Self::Disputed {
                    issue: issue.clone(),
                    claims: claims.clone(),
                }
            }
        }
    }
    pub fn issues(&self) -> Vec<Issue> {
        match self.normalize() {
            Self::Missing { issues } => issues,
            Self::Disputed { issue, .. } => vec![Issue::new(
                &issue,
                "Disputed",
                format!("competing claims for {issue}"),
            )],
            Self::Known { value, .. } => sorted(
                children(&value)
                    .into_iter()
                    .flat_map(|x| x.issues())
                    .collect(),
            ),
        }
    }
    pub fn evidence(&self) -> Vec<String> {
        match self {
            Self::Known { value, evidence } => sorted(
                evidence
                    .iter()
                    .cloned()
                    .chain(children(value).into_iter().flat_map(|x| x.evidence()))
                    .collect(),
            ),
            Self::Disputed { claims, .. } => {
                sorted(claims.iter().flat_map(|c| c.evidence.clone()).collect())
            }
            _ => Vec::new(),
        }
    }
}
pub fn sorted<T: Ord>(mut values: Vec<T>) -> Vec<T> {
    values.sort();
    values.dedup();
    values
}
fn children(value: &Value) -> Vec<&Observation> {
    match value {
        Value::List { items } | Value::Variant { fields: items, .. } => items.iter().collect(),
        Value::Record { fields, .. } => fields.values().collect(),
        _ => vec![],
    }
}
pub fn normalize_value(v: &Value) -> Value {
    match v {
        Value::List { items } => Value::List {
            items: items.iter().map(|v| v.normalize()).collect(),
        },
        Value::Record {
            name,
            type_args,
            fields,
        } => Value::Record {
            name: name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(k, v)| (k.clone(), v.normalize()))
                .collect(),
        },
        Value::Variant {
            name,
            type_args,
            variant,
            fields,
        } => Value::Variant {
            name: name.clone(),
            type_args: type_args.clone(),
            variant: variant.clone(),
            fields: fields.iter().map(|v| v.normalize()).collect(),
        },
        _ => v.clone(),
    }
}
pub fn clean_value(v: &Value) -> Value {
    fn clean(o: &Observation) -> Observation {
        match o.normalize() {
            Observation::Known { value, .. } => Observation::known(clean_value(&value)),
            other => other,
        }
    }
    match v {
        Value::List { items } => Value::List {
            items: items.iter().map(clean).collect(),
        },
        Value::Record {
            name,
            type_args,
            fields,
        } => Value::Record {
            name: name.clone(),
            type_args: type_args.clone(),
            fields: fields.iter().map(|(k, v)| (k.clone(), clean(v))).collect(),
        },
        Value::Variant {
            name,
            type_args,
            variant,
            fields,
        } => Value::Variant {
            name: name.clone(),
            type_args: type_args.clone(),
            variant: variant.clone(),
            fields: fields.iter().map(clean).collect(),
        },
        _ => v.clone(),
    }
}
pub fn blocked(values: &[Observation]) -> Observation {
    Observation::Missing {
        issues: sorted(values.iter().flat_map(|x| x.issues()).collect()),
    }
}
pub fn with_evidence(value: Value, values: &[Observation]) -> Observation {
    Observation::Known {
        value,
        evidence: sorted(values.iter().flat_map(|v| v.evidence()).collect()),
    }
}
pub fn project(value: &Observation, field: &str) -> Result<Observation, String> {
    match value.normalize() {
        Observation::Known {
            value: Value::Record { fields, .. },
            evidence,
        } => {
            let mut found = fields
                .get(field)
                .ok_or_else(|| format!("missing record field {field}"))?
                .normalize();
            if let Observation::Known {
                evidence: ref mut e,
                ..
            } = found
            {
                e.extend(evidence);
                *e = sorted(e.clone())
            }
            Ok(found)
        }
        Observation::Known { .. } => Err("field projection requires a record".into()),
        other => Ok(blocked(&[other])),
    }
}
pub fn unary(op: &str, v: &Observation, limits: &ExecutionLimits) -> Result<Observation, String> {
    if !v.issues().is_empty() {
        return Ok(blocked(std::slice::from_ref(v)));
    }
    let value = v.value().ok_or("expected known value")?;
    let out = match (op, value) {
        ("not", Value::Bool { value }) => Value::Bool { value: !value },
        ("-", Value::Int { value }) => int_value(-integer(&value, limits)?, limits)?,
        ("-", v @ Value::Rational { .. }) => rational(-number(&v, limits)?, limits)?,
        _ => return Err("invalid unary operation".into()),
    };
    Ok(with_evidence(out, std::slice::from_ref(v)))
}
pub fn compare(a: &Value, b: &Value, limits: &ExecutionLimits) -> Result<Ordering, String> {
    match (a, b) {
        (Value::Int { .. }, Value::Int { .. })
        | (Value::Rational { .. }, Value::Rational { .. }) => {
            Ok(number(a, limits)?.cmp(&number(b, limits)?))
        }
        (Value::Bool { value: a }, Value::Bool { value: b }) => Ok(a.cmp(b)),
        (Value::Text { value: a }, Value::Text { value: b }) => Ok(a.cmp(b)),
        (Value::Unit, Value::Unit) => Ok(Ordering::Equal),
        (
            Value::Record {
                name: a,
                type_args: ta,
                fields: fa,
            },
            Value::Record {
                name: b,
                type_args: tb,
                fields: fb,
            },
        ) if a == b && ta == tb && fa.keys().eq(fb.keys()) => {
            for (k, v) in fa {
                let c = compare(
                    &v.value().ok_or("unknown comparison")?,
                    &fb[k].value().ok_or("unknown comparison")?,
                    limits,
                )?;
                if c != Ordering::Equal {
                    return Ok(c);
                }
            }
            Ok(Ordering::Equal)
        }
        (Value::List { items: a }, Value::List { items: b }) => compare_sequence(a, b, limits),
        (
            Value::Variant {
                name: a,
                type_args: ta,
                variant: va,
                fields: fa,
            },
            Value::Variant {
                name: b,
                type_args: tb,
                variant: vb,
                fields: fb,
            },
        ) if a == b && ta == tb => {
            let c = va.cmp(vb);
            if c == Ordering::Equal {
                compare_sequence(fa, fb, limits)
            } else {
                Ok(c)
            }
        }
        _ => Err("values cannot be compared".into()),
    }
}
fn compare_sequence(
    a: &[Observation],
    b: &[Observation],
    limits: &ExecutionLimits,
) -> Result<Ordering, String> {
    for (x, y) in a.iter().zip(b) {
        let c = compare(
            &x.value().ok_or("unknown comparison")?,
            &y.value().ok_or("unknown comparison")?,
            limits,
        )?;
        if c != Ordering::Equal {
            return Ok(c);
        }
    }
    Ok(a.len().cmp(&b.len()))
}
pub fn binary(
    op: &str,
    a: &Observation,
    b: &Observation,
    limits: &ExecutionLimits,
) -> Result<Observation, String> {
    if op == "and" || op == "or" {
        let decisive = op == "or";
        if a.bool() == Some(decisive) || b.bool() == Some(decisive) {
            return Ok(with_evidence(
                Value::Bool { value: decisive },
                &[a.clone(), b.clone()],
            ));
        }
        if let (Some(x), Some(y)) = (a.bool(), b.bool()) {
            return Ok(with_evidence(
                Value::Bool {
                    value: if op == "and" { x && y } else { x || y },
                },
                &[a.clone(), b.clone()],
            ));
        }
        return Ok(blocked(&[a.clone(), b.clone()]));
    }
    if !a.issues().is_empty() || !b.issues().is_empty() {
        return Ok(blocked(&[a.clone(), b.clone()]));
    }
    let av = a.value().ok_or("unknown operand")?;
    let bv = b.value().ok_or("unknown operand")?;
    let out = match op {
        "==" | "!=" | "<" | "<=" | ">" | ">=" => {
            let c = compare(&av, &bv, limits)?;
            Value::Bool {
                value: match op {
                    "==" => c == Ordering::Equal,
                    "!=" => c != Ordering::Equal,
                    "<" => c == Ordering::Less,
                    "<=" => c != Ordering::Greater,
                    ">" => c == Ordering::Greater,
                    _ => c != Ordering::Less,
                },
            }
        }
        "+" if matches!(av, Value::Text { .. }) => {
            if let (Value::Text { value: x }, Value::Text { value: y }) = (&av, &bv) {
                let s = format!("{x}{y}");
                if s.len() > limits.input_bytes {
                    return Err("text byte limit exceeded".into());
                }
                Value::Text { value: s }
            } else {
                return Err("text addition requires text".into());
            }
        }
        "+" | "-" | "*" | "/" | "%" => {
            let x = number(&av, limits)?;
            let y = number(&bv, limits)?;
            if (op == "/" || op == "%") && y.is_zero() {
                return Err("division by zero".into());
            }
            let n = match op {
                "+" => x + y,
                "-" => x - y,
                "*" => x * y,
                "/" => x / y,
                _ => {
                    if !x.is_integer() || !y.is_integer() {
                        return Err("remainder requires integers".into());
                    }
                    BigRational::from_integer(x.to_integer() % y.to_integer())
                }
            };
            if op != "/" && matches!(av, Value::Int { .. }) && matches!(bv, Value::Int { .. }) {
                int_value(n.to_integer(), limits)?
            } else {
                rational(n, limits)?
            }
        }
        _ => return Err(format!("unknown binary operation {op}")),
    };
    Ok(with_evidence(out, &[a.clone(), b.clone()]))
}

pub const BUILTINS: &[&str] = &[
    "len",
    "at",
    "append",
    "concat",
    "range",
    "to_rational",
    "floor",
    "floor_div",
    "text_len",
    "text_at",
    "parse_int",
    "int_text",
    "fail",
];
pub fn primitive(
    name: &str,
    args: &[Observation],
    limits: &ExecutionLimits,
) -> Result<Observation, String> {
    if name == "fail" {
        return Err(match args.first().and_then(|x| x.value()) {
            Some(Value::Text { value }) => value,
            _ => "explicit failure".into(),
        });
    }
    if name == "len"
        && let Some(Value::List { items }) = args.first().and_then(|x| x.value())
    {
        return Ok(with_evidence(
            Value::Int {
                value: items.len().to_string(),
            },
            args,
        ));
    }
    if name == "at"
        && args.len() == 2
        && let (Some(Value::List { items }), Some(Value::Int { value })) =
            (args[0].value(), args[1].value())
    {
        let i = integer(&value, limits)?
            .to_usize()
            .ok_or("invalid list index")?;
        return items
            .get(i)
            .cloned()
            .ok_or_else(|| "list index out of bounds".into());
    }
    if name == "append"
        && args.len() == 2
        && let Some(Value::List { mut items }) = args[0].value()
    {
        if items.len() >= limits.collection_entries {
            return Err("collection limit exceeded".into());
        }
        items.push(args[1].clone());
        return Ok(Observation::known(Value::List { items }));
    }
    if name == "concat"
        && args.len() == 2
        && let (Some(Value::List { mut items }), Some(Value::List { items: other })) =
            (args[0].value(), args[1].value())
    {
        if items.len().saturating_add(other.len()) > limits.collection_entries {
            return Err("collection limit exceeded".into());
        }
        items.extend(other);
        return Ok(Observation::known(Value::List { items }));
    }
    if args.iter().any(|v| !v.issues().is_empty()) {
        return Ok(blocked(args));
    }
    let values: Vec<_> = args
        .iter()
        .map(|v| v.value().ok_or("unknown primitive argument"))
        .collect::<Result<_, _>>()?;
    let out = match (name, values.as_slice()) {
        ("to_rational", [v @ Value::Int { .. }]) => rational(number(v, limits)?, limits)?,
        ("floor", [v @ Value::Rational { .. }]) => {
            int_value(number(v, limits)?.floor().to_integer(), limits)?
        }
        ("floor_div", [a @ Value::Int { .. }, b @ Value::Int { .. }]) => {
            let b = number(b, limits)?;
            if b.is_zero() {
                return Err("division by zero".into());
            }
            int_value((number(a, limits)? / b).floor().to_integer(), limits)?
        }
        ("range", [Value::Int { value: a }, Value::Int { value: b }]) => {
            let a = integer(a, limits)?;
            let b = integer(b, limits)?;
            let count = (&b - &a)
                .max(BigInt::zero())
                .to_usize()
                .ok_or("collection limit exceeded")?;
            if count > limits.collection_entries {
                return Err("collection limit exceeded".into());
            }
            Value::List {
                items: (0..count)
                    .map(|i| {
                        Observation::known(Value::Int {
                            value: (&a + BigInt::from(i)).to_string(),
                        })
                    })
                    .collect(),
            }
        }
        ("text_len", [Value::Text { value }]) => Value::Int {
            value: value.chars().count().to_string(),
        },
        ("text_at", [Value::Text { value: s }, Value::Int { value: i }]) => {
            let i = integer(i, limits)?.to_usize().ok_or("invalid text index")?;
            Value::Text {
                value: s
                    .chars()
                    .nth(i)
                    .ok_or("text index out of bounds")?
                    .to_string(),
            }
        }
        ("parse_int", [Value::Text { value }]) => int_value(integer(value, limits)?, limits)?,
        ("int_text", [Value::Int { value }]) => Value::Text {
            value: value.clone(),
        },
        _ => return Err(format!("invalid arguments to {name}")),
    };
    Ok(with_evidence(out, args))
}

pub fn pattern_bind(
    pattern: &Pattern,
    value: &Observation,
) -> Option<BTreeMap<String, Observation>> {
    match pattern {
        Pattern::Wildcard => Some(BTreeMap::new()),
        Pattern::Bind { name } => Some(BTreeMap::from([(name.clone(), value.clone())])),
        Pattern::Bool { value: b } if value.bool() == Some(*b) => Some(BTreeMap::new()),
        Pattern::Variant {
            name,
            variant,
            bindings,
        } => match value.value() {
            Some(Value::Variant {
                name: n,
                variant: v,
                fields,
                ..
            }) if &n == name && &v == variant && fields.len() == bindings.len() => {
                Some(bindings.iter().cloned().zip(fields).collect())
            }
            _ => None,
        },
        _ => None,
    }
}
pub fn query_value(name: &str, outcome: &QueryOutcome) -> Observation {
    match outcome {
        QueryOutcome::Resolved { value, .. } => Observation::known(value.clone()),
        QueryOutcome::Undetermined { issues, .. } => Observation::Missing {
            issues: issues.clone(),
        },
        QueryOutcome::Inapplicable { .. } => Observation::missing(
            name,
            "DependencyInapplicable",
            "dependent query is inapplicable",
        ),
        QueryOutcome::OutsideCoverage { .. } => Observation::missing(
            name,
            "DependencyOutsideCoverage",
            "dependent query is outside coverage",
        ),
        QueryOutcome::Conflicted { .. } => Observation::missing(
            name,
            "DependencyConflict",
            "dependent query has conflicting conclusions",
        ),
    }
}

pub fn validate_observation(
    program: &Program,
    obs: &Observation,
    ty: &Ty,
    limits: &ExecutionLimits,
) -> Result<(), String> {
    fn check(
        p: &Program,
        o: &Observation,
        t: &Ty,
        l: &ExecutionLimits,
        depth: usize,
    ) -> Result<(), String> {
        if depth > l.nesting {
            return Err("value nesting limit exceeded".into());
        }
        match o {
            Observation::Missing { issues } => {
                if issues.is_empty() || issues.iter().any(|i| i.id.is_empty()) {
                    return Err("missing observations require named issues".into());
                }
                Ok(())
            }
            Observation::Disputed { issue, claims } => {
                if issue.is_empty() || claims.len() < 2 || claims.len() > l.collection_entries {
                    return Err(
                        "a dispute requires an issue and at least two bounded claims".into(),
                    );
                }
                for c in claims {
                    check(p, &Observation::known(c.value.clone()), t, l, depth + 1)?
                }
                Ok(())
            }
            Observation::Known { value, evidence } => {
                if evidence.len() > l.collection_entries {
                    return Err("evidence limit exceeded".into());
                }
                match (t, value) {
                    (Ty::Unit, Value::Unit) | (Ty::Bool, Value::Bool { .. }) => Ok(()),
                    (Ty::Text, Value::Text { value }) => {
                        if value.len() <= l.input_bytes {
                            Ok(())
                        } else {
                            Err("text limit exceeded".into())
                        }
                    }
                    (Ty::Int, Value::Int { value }) => integer(value, l).map(|_| ()),
                    (Ty::Rational, Value::Rational { .. }) => number(value, l).map(|_| ()),
                    (Ty::List(t), Value::List { items }) => {
                        if items.len() > l.collection_entries {
                            return Err("collection limit exceeded".into());
                        }
                        for x in items {
                            check(p, x, t, l, depth + 1)?
                        }
                        Ok(())
                    }
                    (Ty::Named { name, args }, _) => {
                        let def = p
                            .types
                            .get(name)
                            .ok_or_else(|| format!("unknown type {name}"))?;
                        if def.parameters.len() != args.len() {
                            return Err("type argument arity mismatch".into());
                        }
                        let bindings = def
                            .parameters
                            .iter()
                            .cloned()
                            .zip(args.iter().cloned())
                            .collect();
                        match (&def.body, value) {
                            (TypeBody::Alias { ty }, _) => {
                                check(p, o, &ty.subst(&bindings), l, depth + 1)
                            }
                            (
                                TypeBody::Record { fields: types },
                                Value::Record {
                                    name: n,
                                    type_args,
                                    fields,
                                },
                            ) if n == name
                                && type_args == args
                                && fields.keys().eq(types.keys()) =>
                            {
                                for (k, t) in types {
                                    check(p, &fields[k], &t.subst(&bindings), l, depth + 1)?
                                }
                                Ok(())
                            }
                            (
                                TypeBody::Enum { variants },
                                Value::Variant {
                                    name: n,
                                    type_args,
                                    variant,
                                    fields,
                                },
                            ) if n == name && type_args == args => {
                                let types = variants.get(variant).ok_or("unknown variant")?;
                                if types.len() != fields.len() {
                                    return Err("variant arity mismatch".into());
                                }
                                for (x, t) in fields.iter().zip(types) {
                                    check(p, x, &t.subst(&bindings), l, depth + 1)?
                                }
                                Ok(())
                            }
                            _ => Err(format!("value does not inhabit {name}")),
                        }
                    }
                    _ => Err(format!("value does not inhabit {t:?}")),
                }
            }
        }
    }
    check(program, obs, ty, limits, 0)
}
pub fn validate_request(
    program: &Program,
    request: &RunRequest,
    limits: &ExecutionLimits,
) -> Result<(), String> {
    validate_limits(limits)?;
    validate_program(program)?;
    if request.version != VERSION || program.semantics != VERSION {
        return Err("unsupported semantics/request version".into());
    }
    let q = program
        .queries
        .get(&request.query)
        .ok_or_else(|| format!("unknown query {}", request.query))?;
    let expected: BTreeSet<_> = q.args.iter().map(|a| a.name.as_str()).collect();
    let got: BTreeSet<_> = request.arguments.keys().map(|a| a.as_str()).collect();
    if expected != got {
        return Err("request must contain exactly the query's named arguments; use missing observations for unknowns".into());
    }
    for arg in &q.args {
        validate_observation(program, &request.arguments[&arg.name], &arg.ty, limits)?
    }
    if crate::canonical_json(request)?.len() > limits.input_bytes {
        return Err("request byte limit exceeded".into());
    }
    Ok(())
}
