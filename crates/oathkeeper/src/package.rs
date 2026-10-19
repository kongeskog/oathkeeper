use oath_core::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub language: String,
    pub entry: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub path: String,
    pub version: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: BTreeMap<String, Dependency>,
}
#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub digest: String,
    pub dependencies: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackageLock {
    pub version: String,
    pub root_package: String,
    pub root_digest: String,
    pub packages: Vec<LockedPackage>,
}
#[derive(Clone, Debug)]
pub struct ResolvedPackage {
    pub root: PathBuf,
    pub name: String,
    pub entry: String,
    pub sources: BTreeMap<String, String>,
    pub digest: String,
    pub lock: PackageLock,
    pub roots: BTreeMap<String, PathBuf>,
}
fn safe_relative(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);
    if path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!("expected a relative package path, got {value}"));
    }
    Ok(path)
}
fn normalized(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
fn text_file(path: &Path) -> Result<String, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if metadata.len() > 64 * 1024 * 1024 {
        return Err("package file exceeds byte limit".into());
    }
    fs::read_to_string(path)
        .map(|s| s.replace("\r\n", "\n"))
        .map_err(|e| format!("{}: {e}", path.display()))
}
fn walk(root: &Path, dir: &Path, extension: &str, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            return Err(format!(
                "package symlinks are not supported: {}",
                entry.path().display()
            ));
        }
        if kind.is_dir() {
            walk(root, &entry.path(), extension, files)?
        } else if entry.path().extension().is_some_and(|x| x == extension) {
            files.push(
                entry
                    .path()
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_path_buf(),
            )
        }
        if files.len() > 100_000 {
            return Err("package file count limit exceeded".into());
        }
    }
    Ok(())
}
struct Resolver {
    active: BTreeSet<PathBuf>,
    done: BTreeMap<PathBuf, LockedPackage>,
    packages: BTreeMap<String, LockedPackage>,
    sources: BTreeMap<String, String>,
    roots: BTreeMap<String, PathBuf>,
}
impl Resolver {
    fn visit(&mut self, root: &Path, depth: usize) -> Result<(LockedPackage, Manifest), String> {
        if depth > 128 {
            return Err("package dependency depth exceeded".into());
        }
        let root = root
            .canonicalize()
            .map_err(|e| format!("{}: {e}", root.display()))?;
        let manifest: Manifest =
            toml::from_str(&text_file(&root.join("oath.toml"))?).map_err(|e| e.to_string())?;
        if manifest.package.language != format!("={VERSION}") {
            return Err(format!(
                "{} requires unsupported language {}",
                manifest.package.name, manifest.package.language
            ));
        }
        if manifest.package.name.is_empty()
            || !manifest
                .package
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_')
        {
            return Err("invalid package identity".into());
        }
        safe_relative(&manifest.package.entry)?;
        if let Some(locked) = self.done.get(&root) {
            return Ok((locked.clone(), manifest));
        }
        if !self.active.insert(root.clone()) {
            return Err("package dependency cycle".into());
        }
        let mut dependencies = BTreeMap::new();
        let mut normalized_dependencies = BTreeMap::new();
        for (alias, dep) in &manifest.dependencies {
            let (locked, _) = self.visit(&root.join(&dep.path), depth + 1)?;
            if dep.version != format!("={}", locked.version) {
                return Err(format!("dependency {alias} version mismatch"));
            }
            dependencies.insert(alias.clone(), locked.digest.clone());
            normalized_dependencies.insert(alias.clone(), (locked.name, dep.version.clone()));
        }
        let mut files = Vec::new();
        walk(&root, &root.join("src"), "oath", &mut files)?;
        if files.is_empty() {
            return Err(format!(
                "{} contains no src/*.oath files",
                manifest.package.name
            ));
        }
        let mut file_hashes = BTreeMap::new();
        let mut total = 0;
        for relative in files {
            let source = text_file(&root.join(&relative))?;
            total += source.len();
            if total > 64 * 1024 * 1024 {
                return Err("package source byte limit exceeded".into());
            }
            file_hashes.insert(normalized(&relative), sha256(source.as_bytes()));
            self.sources.insert(
                format!("{}/{}", manifest.package.name, normalized(&relative)),
                source,
            );
        }
        if !file_hashes.contains_key(&manifest.package.entry.replace('\\', "/")) {
            return Err("entry is not an included source file under src/".into());
        }
        let hash = digest(
            "package",
            &(
                &manifest.package,
                normalized_dependencies,
                file_hashes,
                &dependencies,
            ),
        )?;
        let locked = LockedPackage {
            name: manifest.package.name.clone(),
            version: manifest.package.version.clone(),
            digest: hash,
            dependencies,
        };
        if let Some(existing) = self.packages.get(&locked.name)
            && existing != &locked
        {
            return Err(format!("conflicting snapshots of package {}", locked.name));
        }
        self.packages.insert(locked.name.clone(), locked.clone());
        self.roots.insert(locked.name.clone(), root.clone());
        self.done.insert(root.clone(), locked.clone());
        self.active.remove(&root);
        Ok((locked, manifest))
    }
}
fn resolve(root: &Path) -> Result<ResolvedPackage, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut r = Resolver {
        active: BTreeSet::new(),
        done: BTreeMap::new(),
        packages: BTreeMap::new(),
        sources: BTreeMap::new(),
        roots: BTreeMap::new(),
    };
    let (locked, manifest) = r.visit(&root, 0)?;
    let lock = PackageLock {
        version: VERSION.into(),
        root_package: locked.name.clone(),
        root_digest: locked.digest.clone(),
        packages: r.packages.into_values().collect(),
    };
    Ok(ResolvedPackage {
        root,
        name: locked.name.clone(),
        entry: format!(
            "{}/{}",
            locked.name,
            manifest.package.entry.replace('\\', "/")
        ),
        sources: r.sources,
        digest: locked.digest,
        lock,
        roots: r.roots,
    })
}
pub fn lock_package(root: &Path) -> Result<PackageLock, String> {
    let p = resolve(root)?;
    let bytes = serde_json::to_vec_pretty(&p.lock).map_err(|e| e.to_string())?;
    fs::write(p.root.join("oath.lock"), bytes).map_err(|e| e.to_string())?;
    Ok(p.lock)
}
pub fn load_package(root: &Path) -> Result<ResolvedPackage, String> {
    let p = resolve(root)?;
    let bytes = fs::read(p.root.join("oath.lock"))
        .map_err(|e| format!("cannot read oath.lock ({e}); run oath lock explicitly"))?;
    let lock: PackageLock = read_json(&bytes, &ExecutionLimits::default())?;
    if lock != p.lock {
        return Err(
            "package snapshot differs from oath.lock; review changes and run oath lock explicitly"
                .into(),
        );
    }
    Ok(p)
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceDescriptor {
    pub stable_id: String,
    pub citation: String,
    pub edition_or_version: String,
    pub locations: Vec<String>,
    pub expected_sha256: String,
    pub package: String,
}
fn field_text(fields: &BTreeMap<String, Observation>, name: &str) -> Result<String, String> {
    match fields.get(name).and_then(|x| x.value()) {
        Some(Value::Text { value }) => Ok(value),
        _ => Err(format!("source metadata requires text field {name}")),
    }
}
fn field_texts(fields: &BTreeMap<String, Observation>, name: &str) -> Result<Vec<String>, String> {
    match fields.get(name).and_then(|x| x.value()) {
        Some(Value::List { items }) => items
            .iter()
            .map(|i| match i.value() {
                Some(Value::Text { value }) => Ok(value),
                _ => Err("expected source location text".into()),
            })
            .collect(),
        _ => Err(format!("source metadata requires list field {name}")),
    }
}
pub fn source_references(program: &Program) -> Result<Vec<SourceDescriptor>, String> {
    let mut refs = BTreeMap::new();
    let mut anchors = Vec::new();
    fn scan(
        value: &Value,
        package: &str,
        refs: &mut BTreeMap<String, SourceDescriptor>,
        anchors: &mut Vec<String>,
    ) -> Result<(), String> {
        match value {
            Value::Record { name, fields, .. } => {
                if name == "std.provenance.SourceRef" {
                    let source = SourceDescriptor {
                        stable_id: field_text(fields, "stable_id")?,
                        citation: field_text(fields, "citation")?,
                        edition_or_version: field_text(fields, "edition_or_version")?,
                        locations: field_texts(fields, "locations")?,
                        expected_sha256: field_text(fields, "expected_sha256")?,
                        package: package.into(),
                    };
                    if source.stable_id.is_empty()
                        || source.expected_sha256.len() != 64
                        || !source
                            .expected_sha256
                            .bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    {
                        return Err(format!(
                            "{}: expected a stable ID and lowercase SHA-256",
                            source.stable_id
                        ));
                    }
                    if let Some(old) = refs.get(&source.stable_id) {
                        if old.expected_sha256 != source.expected_sha256
                            || old.edition_or_version != source.edition_or_version
                        {
                            return Err("conflicting source IDs".into());
                        }
                    } else {
                        refs.insert(source.stable_id.clone(), source);
                    }
                } else if name == "std.provenance.SourceAnchor" {
                    anchors.push(field_text(fields, "source_id")?);
                    if field_text(fields, "locator")?.is_empty() {
                        return Err("empty source locator".into());
                    }
                }
                for v in fields.values() {
                    if let Some(value) = v.value() {
                        scan(&value, package, refs, anchors)?
                    }
                }
            }
            Value::List { items } | Value::Variant { fields: items, .. } => {
                for v in items {
                    if let Some(value) = v.value() {
                        scan(&value, package, refs, anchors)?
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn contains_source(p: &Program, ty: &Ty, seen: &BTreeSet<String>) -> bool {
        match ty {
            Ty::List(t) => contains_source(p, t, seen),
            Ty::Named { name, args } => {
                if name == "std.provenance.SourceRef" || name == "std.provenance.SourceAnchor" {
                    return true;
                }
                if seen.contains(name) {
                    return false;
                }
                let Some(def) = p.types.get(name) else {
                    return false;
                };
                let mut seen = seen.clone();
                seen.insert(name.clone());
                let bindings = def
                    .parameters
                    .iter()
                    .cloned()
                    .zip(args.iter().cloned())
                    .collect();
                let types: Vec<&Ty> = match &def.body {
                    TypeBody::Record { fields } => fields.values().collect(),
                    TypeBody::Enum { variants } => variants.values().flatten().collect(),
                    TypeBody::Alias { ty } => vec![ty],
                };
                types
                    .iter()
                    .any(|t| contains_source(p, &t.subst(&bindings), &seen))
            }
            _ => false,
        }
    }
    for (name, c) in &program.constants {
        if !contains_source(program, &c.ty, &BTreeSet::new()) {
            continue;
        }
        let value = crate::runtime::constant_value(program, name)
            .map_err(|e| e.to_string())?
            .value()
            .ok_or("metadata constant is unresolved")?;
        let package = c.span.file.split('/').next().unwrap_or("");
        scan(&value, package, &mut refs, &mut anchors)?;
    }
    for id in anchors {
        if !refs.contains_key(&id) {
            return Err(format!("source anchor references missing source {id}"));
        }
    }
    Ok(refs.into_values().collect())
}
pub fn coverage(program: &Program) -> Result<Vec<Value>, String> {
    let mut records = Vec::new();
    for (name, c) in &program.constants {
        if matches!(&c.ty,Ty::Named{name,..} if name=="std.provenance.CoverageRecord") {
            let value = crate::runtime::constant_value(program, name)
                .map_err(|e| e.to_string())?
                .value()
                .ok_or("unresolved coverage record")?;
            records.push(value);
        }
    }
    Ok(records)
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceAudit {
    Matched {
        source: String,
        sha256: String,
    },
    Mismatched {
        source: String,
        expected: String,
        actual: String,
    },
    Unavailable {
        source: String,
        reason: String,
    },
    NotChecked {
        source: String,
    },
}
pub fn audit_sources(
    package: &ResolvedPackage,
    program: &Program,
    cache: &Path,
    fetch: bool,
) -> Result<Vec<SourceAudit>, String> {
    let mut reports = Vec::new();
    for source in source_references(program)? {
        let cached = cache.join(&source.expected_sha256);
        let mut bytes = None;
        let mut reasons = Vec::new();
        if cached.is_file() {
            let metadata = fs::metadata(&cached).map_err(|e| e.to_string())?;
            if metadata.len() > 64 * 1024 * 1024 {
                return Err("source cache object exceeds byte limit".into());
            }
            bytes = Some(fs::read(&cached).map_err(|e| e.to_string())?)
        }
        if bytes.is_none() {
            for location in &source.locations {
                if let Some(relative) = location.strip_prefix("fixture:") {
                    let relative = safe_relative(relative)?;
                    if let Some(root) = package.roots.get(&source.package) {
                        let path = root.join(relative);
                        if path.is_file() {
                            let path = path.canonicalize().map_err(|e| e.to_string())?;
                            if !path.starts_with(root) {
                                return Err("source fixture escapes its package".into());
                            }
                            if fs::metadata(&path).map_err(|e| e.to_string())?.len()
                                > 64 * 1024 * 1024
                            {
                                return Err("source fixture exceeds byte limit".into());
                            }
                            bytes = Some(fs::read(path).map_err(|e| e.to_string())?);
                            break;
                        }
                    }
                } else if fetch && location.starts_with("https://") {
                    let tls = ureq::native_tls::TlsConnector::new().map_err(|e| e.to_string())?;
                    let agent = ureq::AgentBuilder::new()
                        .tls_connector(std::sync::Arc::new(tls))
                        .build();
                    match agent
                        .get(location)
                        .timeout(std::time::Duration::from_secs(30))
                        .call()
                    {
                        Ok(response) => {
                            let mut body = Vec::new();
                            match response
                                .into_reader()
                                .take(64 * 1024 * 1024 + 1)
                                .read_to_end(&mut body)
                            {
                                Ok(_) => {
                                    if body.len() > 64 * 1024 * 1024 {
                                        return Err("source download exceeds byte limit".into());
                                    }
                                    bytes = Some(body);
                                    break;
                                }
                                Err(e) => reasons.push(e.to_string()),
                            }
                        }
                        Err(e) => reasons.push(e.to_string()),
                    }
                }
            }
        }
        if let Some(bytes) = bytes {
            let actual = sha256(&bytes);
            if actual == source.expected_sha256 {
                if fetch && !cached.exists() {
                    fs::create_dir_all(cache).map_err(|e| e.to_string())?;
                    use std::io::Write;
                    match fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&cached)
                    {
                        Ok(mut f) => f.write_all(&bytes).map_err(|e| e.to_string())?,
                        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                        Err(e) => return Err(e.to_string()),
                    }
                }
                reports.push(SourceAudit::Matched {
                    source: source.stable_id,
                    sha256: actual,
                });
            } else {
                reports.push(SourceAudit::Mismatched {
                    source: source.stable_id,
                    expected: source.expected_sha256,
                    actual,
                })
            }
        } else {
            reports.push(SourceAudit::Unavailable{source:source.stable_id,reason:if reasons.is_empty(){"no matching cached bytes or local fixture; use --fetch for declared HTTPS locations".into()}else{reasons.join("; ")}})
        }
    }
    Ok(reports)
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expectation {
    Outcome {
        status: String,
        #[serde(default)]
        value: Option<Value>,
        #[serde(default)]
        stage: Option<String>,
        #[serde(default)]
        issue_ids: Vec<String>,
        #[serde(default)]
        support: Option<Vec<String>>,
    },
    Error {
        code: String,
        #[serde(default)]
        message_contains: Option<String>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub name: String,
    pub request: RunRequest,
    pub expected: Expectation,
    #[serde(default)]
    pub limits: Option<ExecutionLimits>,
}
#[derive(Clone, Debug, Serialize)]
pub struct TestReport {
    pub name: String,
    pub passed: bool,
    pub message: String,
}
pub fn outcome_status(outcome: &QueryOutcome) -> &'static str {
    match outcome {
        QueryOutcome::Resolved { .. } => "resolved",
        QueryOutcome::Inapplicable { .. } => "inapplicable",
        QueryOutcome::OutsideCoverage { .. } => "outside_coverage",
        QueryOutcome::Undetermined { .. } => "undetermined",
        QueryOutcome::Conflicted { .. } => "conflicted",
    }
}
pub fn test_package(
    package: &ResolvedPackage,
    program: &Program,
) -> Result<Vec<TestReport>, String> {
    source_references(program)?;
    let mut files = Vec::new();
    walk(
        &package.root,
        &package.root.join("tests"),
        "json",
        &mut files,
    )?;
    let mut reports = Vec::new();
    for file in files {
        let fixture: Fixture = read_json(
            text_file(&package.root.join(file))?.as_bytes(),
            &ExecutionLimits::default(),
        )?;
        let limits = fixture.limits.unwrap_or_default();
        let run = crate::run(program, &fixture.request, &limits);
        let result = match (&fixture.expected, run) {
            (
                Expectation::Error {
                    code,
                    message_contains,
                },
                Err(error),
            ) => {
                if &error.code == code
                    && message_contains
                        .as_ref()
                        .is_none_or(|s| error.message.contains(s))
                {
                    Ok(())
                } else {
                    Err(format!("unexpected error: {error}"))
                }
            }
            (Expectation::Error { .. }, Ok(_)) => {
                Err("expected execution error, got legal result".into())
            }
            (_, Err(error)) => Err(error.to_string()),
            (
                Expectation::Outcome {
                    status,
                    value,
                    stage,
                    issue_ids,
                    support,
                },
                Ok(bundle),
            ) => {
                let outcome = &bundle.result.outcome;
                if outcome_status(outcome) != status {
                    Err(format!(
                        "expected {status}, got {}",
                        outcome_status(outcome)
                    ))
                } else if value.as_ref().is_some_and(
                    |v| !matches!(outcome,QueryOutcome::Resolved{value,..} if value==v),
                ) {
                    Err("resolved value differs from authored expectation".into())
                } else if support.as_ref().is_some_and(
                    |s| !matches!(outcome,QueryOutcome::Resolved{support,..} if support==s),
                ) {
                    Err("support differs from expectation".into())
                } else if stage.as_ref().is_some_and(
                    |s| !matches!(outcome,QueryOutcome::Undetermined{stage,..} if stage==s),
                ) {
                    Err("unresolved stage differs from expectation".into())
                } else if !issue_ids.is_empty()
                    && !matches!(outcome,QueryOutcome::Undetermined{issues,..} if issue_ids.iter().all(|id|issues.iter().any(|i|&i.id==id)))
                {
                    Err("expected issues are missing".into())
                } else {
                    crate::verify(program, &fixture.request, &bundle.certificate, &limits)
                        .map(|_| ())
                        .map_err(|e| e.to_string())
                }
            }
        };
        reports.push(TestReport {
            name: fixture.name,
            passed: result.is_ok(),
            message: result
                .err()
                .unwrap_or_else(|| "expectation and independent certificate check passed".into()),
        });
    }
    if reports.is_empty() {
        return Err("no scenario fixtures in tests/".into());
    }
    Ok(reports)
}
