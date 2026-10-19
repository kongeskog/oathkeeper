//! A deliberately small, non-executing authoring frontend. The kernel also accepts
//! the emitted IR directly and repeats validity checks independently of this module.
mod syntax;
use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use syntax::{Declaration, Unit};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpan {
    pub kind: String,
    pub id: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedSource {
    pub kind: String,
    pub package: String,
    pub version: String,
    pub sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compilation {
    pub format: String,
    pub compiler_version: String,
    pub snapshot_sha256: String,
    pub module: LawModule,
    pub case: Option<CaseFile>,
    pub lock: Vec<LockedSource>,
    pub source_map: Vec<SourceSpan>,
}
impl Compilation {
    pub fn bundle(&self) -> Result<Bundle> {
        Ok(self.module.clone().with_case(
            self.case
                .clone()
                .ok_or_else(|| Error("a module needs a case before verification".into()))?,
        ))
    }
    pub fn verify(&self) -> Result<Report> {
        verify(&self.bundle()?).map_err(|error| self.locate(error))
    }
    fn locate(&self, error: Error) -> Error {
        let location = self.source_map.iter().find(|s| {
            (s.kind == "step" && error.0.starts_with(&format!("step {}:", s.id)))
                || (s.kind == "rule" && error.0.starts_with(&format!("rule {}@", s.id)))
        });
        if let Some(s) = location {
            Error(format!("{}:{}:{}: {error}", s.file, s.line, s.column))
        } else {
            error
        }
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn located(d: &Declaration, error: impl std::fmt::Display) -> Error {
    Error(format!(
        "{}:{}:{}: {error}",
        d.span.file, d.span.line, d.span.column
    ))
}
fn decode<T: serde::de::DeserializeOwned>(value: Value, d: &Declaration) -> Result<T> {
    serde_json::from_value(value).map_err(|e| located(d, e))
}

#[derive(Default)]
struct Loader {
    visited: BTreeSet<PathBuf>,
    visiting: BTreeSet<PathBuf>,
    packages: BTreeSet<(String, String)>,
    units: Vec<Unit>,
    lock: Vec<LockedSource>,
}
impl Loader {
    fn visit(&mut self, path: &Path, expected: Option<&str>, depth: usize) -> Result<()> {
        if depth > 64 || self.visited.len() + self.visiting.len() >= 128 {
            return Err(Error("[E_LIMIT] import graph limit exceeded".into()));
        }
        let path = fs::canonicalize(path)
            .map_err(|e| Error(format!("{}: [E_IMPORT] {e}", path.display())))?;
        if self.visiting.contains(&path) {
            return Err(Error(format!(
                "{}: [E_IMPORT_CYCLE] cyclic import",
                path.display()
            )));
        }
        let metadata = fs::metadata(&path).map_err(|e| Error(e.to_string()))?;
        if metadata.len() > 2 * 1024 * 1024 {
            return Err(Error("[E_LIMIT] source exceeds 2 MiB".into()));
        }
        let bytes = fs::read(&path).map_err(|e| Error(e.to_string()))?;
        let sha256 = digest(&bytes);
        if expected.is_some_and(|value| value != sha256) {
            return Err(Error(format!(
                "{}: [E_IMPORT_HASH] source bytes do not match pinned SHA-256",
                path.display()
            )));
        }
        // Verify every incoming edge before deduplicating a diamond dependency.
        if self.visited.contains(&path) {
            return Ok(());
        }
        self.visiting.insert(path.clone());
        let source = String::from_utf8(bytes).map_err(|e| Error(e.to_string()))?;
        let unit = syntax::parse(&source, &path.to_string_lossy())?;
        if depth > 0 && unit.kind != "module" {
            return Err(Error("[E_IMPORT] only law modules may be imported".into()));
        }
        if !self
            .packages
            .insert((unit.package.clone(), unit.version.clone()))
        {
            return Err(Error(format!(
                "[E_NAMESPACE] duplicate package identity: {}@{}",
                unit.package, unit.version
            )));
        }
        for import in &unit.imports {
            if Path::new(&import.path).is_absolute() || import.path.contains("://") {
                return Err(Error(
                    "[E_IMPORT] imports must be relative local paths".into(),
                ));
            }
            if import.sha256.len() != 64
                || !import
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(Error(
                    "[E_IMPORT_HASH] expected lowercase SHA-256 hex".into(),
                ));
            }
            self.visit(
                &path.parent().unwrap().join(&import.path),
                Some(&import.sha256),
                depth + 1,
            )?;
        }
        self.lock.push(LockedSource {
            kind: unit.kind.clone(),
            package: unit.package.clone(),
            version: unit.version.clone(),
            sha256,
        });
        self.units.push(unit);
        self.visiting.remove(&path);
        self.visited.insert(path);
        Ok(())
    }
}

fn resolve_sources(value: &mut Value, sources: &BTreeMap<String, Source>) -> Result<()> {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(name)) = map.get("source") {
                let source = sources
                    .get(name)
                    .ok_or_else(|| Error(format!("[E_NAME] unknown source: {name}")))?;
                map.insert("source".into(), json!(source));
            }
            if let Some(Value::String(name)) = map.get("evidence") {
                let source = sources
                    .get(name)
                    .ok_or_else(|| Error(format!("[E_NAME] unknown evidence source: {name}")))?;
                if map.contains_key("evidence_sha256") {
                    return Err(Error(
                        "evidence and evidence_sha256 cannot both be supplied".into(),
                    ));
                }
                map.insert("evidence_sha256".into(), json!(source.sha256));
                map.remove("evidence");
            }
            for child in map.values_mut() {
                resolve_sources(child, sources)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                resolve_sources(child, sources)?;
            }
        }
        _ => (),
    }
    Ok(())
}

fn link(units: Vec<Unit>, mut lock: Vec<LockedSource>) -> Result<Compilation> {
    let entry = units
        .last()
        .ok_or_else(|| Error("empty compilation".into()))?;
    if entry.kind == "case" && entry.imports.len() > 1 {
        return Err(Error(
            "a case imports one root law module; put additional imports in that module".into(),
        ));
    }
    let root = units
        .iter()
        .rev()
        .find(|u| u.kind == "module")
        .unwrap_or(entry);
    let mut module = LawModule {
        language: format!("oathkeeper/{}", root.language),
        package: root.package.clone(),
        version: root.version.clone(),
        jurisdictions: Vec::new(),
        rules: Vec::new(),
        authorities: Vec::new(),
        authority_assessments: Vec::new(),
        types: Vec::new(),
        predicates: Vec::new(),
        constants: Vec::new(),
    };
    crate::kernel::identifier(&module.package)?;
    crate::kernel::identifier(&module.version)?;
    let mut sources = BTreeMap::new();
    let mut names = BTreeSet::new();
    for unit in &units {
        for d in &unit.declarations {
            let name = if d.kind == "rule" {
                format!("{}@{}", d.id, d.value["version"])
            } else {
                d.id.clone()
            };
            if !names.insert((d.kind.clone(), name)) {
                return Err(located(
                    d,
                    format!("[E_NAMESPACE] duplicate {}: {}", d.kind, d.id),
                ));
            }
            if d.kind == "source" {
                sources.insert(d.id.clone(), decode::<Source>(d.value.clone(), d)?);
            }
        }
    }
    let mut facts = Vec::new();
    let mut retractions = Vec::new();
    let mut attestations = Vec::new();
    let mut oracle_policy = Vec::new();
    let mut constants = Vec::new();
    let mut query = None;
    let mut proof = None;
    let mut source_map = Vec::new();
    for unit in &units {
        if unit.language != root.language {
            return Err(Error(
                "[E_VERSION] imported sources must use the same language version".into(),
            ));
        }
        source_map.extend(unit.spans.clone());
        for d in &unit.declarations {
            let mut value = d.value.clone();
            resolve_sources(&mut value, &sources).map_err(|e| located(d, e))?;
            match d.kind.as_str() {
                "source" => (),
                "type" => module.types.push(d.id.clone()),
                "predicate" => module.predicates.push(decode(value, d)?),
                "constant" => {
                    if unit.kind == "case" {
                        constants.push(decode(value, d)?);
                    } else {
                        module.constants.push(decode(value, d)?);
                    }
                }
                "jurisdiction" => module.jurisdictions.push(decode(value, d)?),
                "authority" => module.authorities.push(decode(value, d)?),
                "assessment" => module.authority_assessments.push(decode(value, d)?),
                "rule" => module.rules.push(decode(value, d)?),
                "fact" => {
                    let map = value.as_object_mut().unwrap();
                    let mut basis = serde_json::Map::new();
                    for key in ["kind", "source", "pinpoint", "explanation"] {
                        basis.insert(
                            key.into(),
                            map.remove(key)
                                .ok_or_else(|| located(d, format!("fact requires {key}")))?,
                        );
                    }
                    if map.contains_key("basis") || map.contains_key("evidence_sha256") {
                        return Err(located(
                            d,
                            "fact provenance is supplied by source/kind/pinpoint/explanation",
                        ));
                    }
                    map.insert("evidence_sha256".into(), basis["source"]["sha256"].clone());
                    map.insert("basis".into(), Value::Object(basis));
                    facts.push(decode(value, d)?);
                }
                "attestation" => attestations.push(decode(value, d)?),
                "retraction" => retractions.push(decode(value, d)?),
                "policy" => {
                    let map = value.as_object_mut().unwrap();
                    let claim = map.remove("claim").unwrap();
                    if claim["negative"] == true {
                        return Err(located(d, "policy requests bind a predicate and arguments, not an expected answer polarity"));
                    }
                    if map.contains_key("oracle")
                        || map.contains_key("atom")
                        || map.contains_key("arguments")
                    {
                        return Err(located(
                            d,
                            "policy request is already supplied by declaration",
                        ));
                    }
                    let id = map.remove("id").unwrap();
                    map.insert("oracle".into(), id);
                    map.insert("atom".into(), claim["atom"].clone());
                    map.insert("arguments".into(), claim["arguments"].clone());
                    oracle_policy.push(decode(value, d)?);
                }
                "query" => query = Some(decode(value, d)?),
                "proof" => proof = Some(decode(value, d)?),
                _ => unreachable!(),
            }
        }
    }
    let case = if entry.kind == "case" {
        Some(CaseFile {
            retractions,
            constants,
            facts,
            attestations,
            oracle_policy,
            query: query.ok_or_else(|| Error("[E_PARSE] case requires a query".into()))?,
            proof: proof.unwrap_or_default(),
        })
    } else {
        None
    };
    let mut compilation = Compilation {
        format: "oathkeeper-compilation/1".into(),
        compiler_version: env!("CARGO_PKG_VERSION").into(),
        module,
        case,
        lock: Vec::new(),
        source_map,
        snapshot_sha256: String::new(),
    };
    validate_module_schema(&compilation.module).map_err(|e| compilation.locate(e))?;
    if compilation.module.language == "oathkeeper/0.4" {
        validate_module_metadata(&compilation.module)?;
    }
    if compilation.case.is_some() {
        let bundle = compilation.bundle()?;
        let schema =
            crate::schema::Schema::new(&bundle.types, &bundle.predicates, &bundle.constants)?;
        let empty = BTreeMap::new();
        schema.claim(&bundle.query.goal, &empty)?;
        for fact in &bundle.facts {
            schema.claim(&fact.claim, &empty)?;
        }
        for a in &bundle.attestations {
            schema.claim(&a.claim, &empty)?;
        }
        for policy in &bundle.oracle_policy {
            schema.claim(
                &Claim {
                    atom: policy.atom.clone(),
                    negative: false,
                    arguments: policy.arguments.clone(),
                },
                &empty,
            )?;
        }
        for step in &bundle.proof {
            schema
                .claim(&step.claim, &empty)
                .map_err(|e| compilation.locate(Error(format!("step {}: {e}", step.id))))?;
        }
        if bundle.language == "oathkeeper/0.4" {
            validate_context(&bundle)?;
        }
    }
    lock.sort_by(|a, b| (&a.package, &a.version).cmp(&(&b.package, &b.version)));
    compilation.snapshot_sha256 = digest(
        &serde_json::to_vec(
            &json!({"module":compilation.module,"case":compilation.case,"lock":lock}),
        )
        .map_err(|e| Error(e.to_string()))?,
    );
    compilation.lock = lock;
    Ok(compilation)
}

/// Preflight all declaration links and metadata. No legal eligibility claim is made.
fn validate_module_metadata(module: &LawModule) -> Result<()> {
    if module.jurisdictions.is_empty() {
        return crate::kernel::require(
            module.rules.is_empty()
                && module.authorities.is_empty()
                && module.authority_assessments.is_empty(),
            "legal declarations require a jurisdiction",
        );
    }
    let at = "1970-01-01T00:00:00Z".to_owned();
    let case = CaseFile {
        constants: Vec::new(),
        facts: Vec::new(),
        attestations: Vec::new(),
        oracle_policy: Vec::new(),
        proof: Vec::new(),
        retractions: Vec::new(),
        query: Query {
            jurisdiction: module.jurisdictions[0].id.clone(),
            as_of: at.clone(),
            known_at: at.clone(),
            goal: Claim {
                atom: "oathkeeper.integer.eq".into(),
                negative: false,
                arguments: vec![Term::Integer { value: 0 }, Term::Integer { value: 0 }],
            },
            root: "preflight".into(),
            forum: Some("oathkeeper.module-preflight".into()),
            event_at: Some(at.clone()),
            authority_at: Some(at),
            purpose: Some(CasePurpose::SyntheticTest),
            temporal_rationale: Some(
                "Structural module preflight only; no query or legal result is certified.".into(),
            ),
        },
    };
    validate_context(&module.clone().with_case(case))
}

/// Compile a local source graph. Every import edge must pin exact bytes. No network or code execution.
pub fn compile_file(path: impl AsRef<Path>) -> Result<Compilation> {
    let mut loader = Loader::default();
    loader.visit(path.as_ref(), None, 0)?;
    link(loader.units, loader.lock)
}

/// Compile a standalone source. For local imports, use `compile_file`.
pub fn compile_text(source: &str) -> Result<Compilation> {
    if source.len() > 2 * 1024 * 1024 {
        return Err(Error("[E_LIMIT] source exceeds 2 MiB".into()));
    }
    let unit = syntax::parse(source, "<input>")?;
    if !unit.imports.is_empty() {
        return Err(Error(
            "[E_IMPORT] text compilation cannot read imports; use compile_file".into(),
        ));
    }
    let lock = vec![LockedSource {
        kind: unit.kind.clone(),
        package: unit.package.clone(),
        version: unit.version.clone(),
        sha256: digest(source.as_bytes()),
    }];
    link(vec![unit], lock)
}
