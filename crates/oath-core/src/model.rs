use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const VERSION: &str = "0.1.0";
pub type ExprId = u32;
pub type NodeId = u32;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Span {
    pub file: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub span: Span,
}
impl Diagnostic {
    pub fn new(code: &str, message: impl Into<String>, span: &Span) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            span: span.clone(),
        }
    }
}
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}: {}",
            self.span.file, self.span.line, self.span.column, self.code, self.message
        )
    }
}

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Ty {
    Unit,
    Bool,
    Text,
    Int,
    Rational,
    List(Box<Ty>),
    Named { name: String, args: Vec<Ty> },
    Param(String),
    Unknown,
}
impl Ty {
    pub fn subst(&self, bindings: &BTreeMap<String, Ty>) -> Ty {
        match self {
            Ty::Param(p) => bindings.get(p).cloned().unwrap_or_else(|| self.clone()),
            Ty::List(t) => Ty::List(Box::new(t.subst(bindings))),
            Ty::Named { name, args } => Ty::Named {
                name: name.clone(),
                args: args.iter().map(|t| t.subst(bindings)).collect(),
            },
            _ => self.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Value {
    Unit,
    Bool {
        value: bool,
    },
    Text {
        value: String,
    },
    Int {
        value: String,
    },
    Rational {
        numerator: String,
        denominator: String,
    },
    List {
        items: Vec<Observation>,
    },
    Record {
        name: String,
        #[serde(default)]
        type_args: Vec<Ty>,
        fields: BTreeMap<String, Observation>,
    },
    Variant {
        name: String,
        #[serde(default)]
        type_args: Vec<Ty>,
        variant: String,
        #[serde(default)]
        fields: Vec<Observation>,
    },
}

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Issue {
    pub id: String,
    pub kind: String,
    pub message: String,
}
impl Issue {
    pub fn new(id: impl Into<String>, kind: &str, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: kind.into(),
            message: message.into(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub value: Value,
    #[serde(default)]
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum Observation {
    Known {
        value: Value,
        #[serde(default)]
        evidence: Vec<String>,
    },
    Missing {
        issues: Vec<Issue>,
    },
    Disputed {
        issue: String,
        claims: Vec<Claim>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Param {
    pub name: String,
    pub ty: Ty,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TypeBody {
    Record { fields: BTreeMap<String, Ty> },
    Enum { variants: BTreeMap<String, Vec<Ty>> },
    Alias { ty: Ty },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TypeDef {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: TypeBody,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Constant {
    pub name: String,
    pub ty: Ty,
    pub value: ExprId,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub args: Vec<Param>,
    pub returns: Ty,
    pub body: ExprId,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub name: String,
    pub args: Vec<Param>,
    pub returns: Ty,
    pub scope: ExprId,
    pub applies: ExprId,
    pub meta: Option<String>,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub name: String,
    pub query: String,
    pub bindings: Vec<String>,
    pub when: ExprId,
    pub value: ExprId,
    pub meta: Option<String>,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Priority {
    pub name: String,
    pub query: String,
    pub bindings: Vec<String>,
    pub higher: String,
    pub lower: String,
    pub when: ExprId,
    pub meta: Option<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Pattern {
    Wildcard,
    Bind {
        name: String,
    },
    Variant {
        name: String,
        variant: String,
        bindings: Vec<String>,
    },
    Bool {
        value: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Arm {
    pub pattern: Pattern,
    pub body: ExprId,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExprKind {
    Literal {
        value: Value,
    },
    Name {
        name: String,
    },
    Unary {
        op: String,
        value: ExprId,
    },
    Binary {
        op: String,
        left: ExprId,
        right: ExprId,
    },
    Field {
        value: ExprId,
        field: String,
    },
    List {
        items: Vec<ExprId>,
    },
    Record {
        name: String,
        type_args: Vec<Ty>,
        fields: BTreeMap<String, ExprId>,
    },
    Variant {
        name: String,
        type_args: Vec<Ty>,
        variant: String,
        fields: Vec<ExprId>,
    },
    Call {
        target: String,
        type_args: Vec<Ty>,
        args: Vec<ExprId>,
    },
    If {
        condition: ExprId,
        yes: ExprId,
        no: ExprId,
    },
    Let {
        name: String,
        value: ExprId,
        body: ExprId,
    },
    Match {
        value: ExprId,
        arms: Vec<Arm>,
    },
    Collection {
        op: String,
        input: ExprId,
        initial: Option<ExprId>,
        function: String,
        type_args: Vec<Ty>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Expr {
    pub id: ExprId,
    pub ty: Ty,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub semantics: String,
    pub package_digest: String,
    pub modules: Vec<String>,
    pub types: BTreeMap<String, TypeDef>,
    pub constants: BTreeMap<String, Constant>,
    pub functions: BTreeMap<String, Function>,
    pub queries: BTreeMap<String, Query>,
    pub rules: BTreeMap<String, Rule>,
    pub priorities: BTreeMap<String, Priority>,
    pub expressions: Vec<Expr>,
}
pub type CompiledProgram = Program;
impl Program {
    pub fn expr(&self, id: ExprId) -> Result<&Expr, String> {
        self.expressions
            .get(id as usize)
            .filter(|e| e.id == id)
            .ok_or_else(|| format!("invalid expression {id}"))
    }
    pub fn digest(&self) -> Result<String, String> {
        crate::digest("program", self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunRequest {
    pub version: String,
    pub query: String,
    pub arguments: BTreeMap<String, Observation>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecutionLimits {
    pub steps: u64,
    pub trace_nodes: usize,
    pub collection_entries: usize,
    pub call_depth: usize,
    pub integer_bits: u64,
    pub input_bytes: usize,
    pub nesting: usize,
}
impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            steps: 1_000_000,
            trace_nodes: 250_000,
            collection_entries: 100_000,
            call_depth: 128,
            integer_bits: 65_536,
            input_bytes: 64 * 1024 * 1024,
            nesting: 128,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Supported {
    pub value: Value,
    pub support: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum QueryOutcome {
    Resolved { value: Value, support: Vec<String> },
    Inapplicable { reasons: Vec<String> },
    OutsideCoverage { reasons: Vec<String> },
    Undetermined { stage: String, issues: Vec<Issue> },
    Conflicted { alternatives: Vec<Supported> },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryResult {
    pub query: String,
    pub outcome: QueryOutcome,
    pub used_premises: Vec<String>,
    pub metadata: Vec<String>,
    pub coverage: Vec<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Env {
    pub values: BTreeMap<String, Observation>,
    pub types: BTreeMap<String, Ty>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryTrace {
    pub query: String,
    pub args: Vec<Observation>,
    pub scope: NodeId,
    pub applies: Option<NodeId>,
    pub guards: BTreeMap<String, NodeId>,
    pub priorities: BTreeMap<String, NodeId>,
    pub yields: BTreeMap<String, NodeId>,
    pub outcome: QueryOutcome,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TraceNode {
    Expression {
        expr: ExprId,
        env: Env,
        children: Vec<NodeId>,
        value: Observation,
    },
    Query {
        trace: QueryTrace,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Certificate {
    pub version: String,
    pub semantics: String,
    pub program_digest: String,
    pub package_digest: String,
    pub request_digest: String,
    pub limits: ExecutionLimits,
    pub result_digest: String,
    pub result: QueryResult,
    pub nodes: Vec<TraceNode>,
    pub root: NodeId,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunBundle {
    pub result: QueryResult,
    pub certificate: Certificate,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VerifiedRun {
    pub result: QueryResult,
    pub certificate_digest: String,
}

#[derive(Clone, Debug, thiserror::Error, Serialize, Deserialize, schemars::JsonSchema)]
#[error("{code}: {message} at {span:?}")]
#[serde(deny_unknown_fields)]
pub struct ExecutionError {
    pub code: String,
    pub message: String,
    pub span: Option<Span>,
}
impl ExecutionError {
    pub fn new(code: &str, message: impl Into<String>, span: Option<&Span>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            span: span.cloned(),
        }
    }
}
#[derive(Clone, Debug, thiserror::Error)]
#[error("certificate rejected: {0}")]
pub struct VerificationError(pub String);
