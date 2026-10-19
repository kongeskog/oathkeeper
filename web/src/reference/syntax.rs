use super::SourceSpan;
use crate::*;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
struct Token {
    text: String,
    string: bool,
    line: usize,
    column: usize,
}

fn lex(input: &str, file: &str) -> Result<Vec<Token>> {
    let bytes = input.as_bytes();
    let (mut i, mut line, mut column) = (0, 1, 1);
    let mut tokens = Vec::new();
    while i < bytes.len() {
        let start = i;
        if bytes[i].is_ascii_whitespace() {
            if bytes[i] == b'\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
            i += 1;
            continue;
        }
        if bytes[i..].starts_with(b"//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
                column += 1;
            }
            continue;
        }
        let string = bytes[i] == b'"';
        if string {
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            if i >= bytes.len() {
                return Err(Error(format!(
                    "{file}:{line}:{column}: [E_PARSE] unterminated string"
                )));
            }
            i += 1;
        } else if b"{}()[],:;@$=".contains(&bytes[i]) {
            i += 1;
        } else if bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-' {
            i += 1;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || b"._/-".contains(&bytes[i]))
            {
                i += 1;
            }
        } else {
            return Err(Error(format!(
                "{file}:{line}:{column}: [E_PARSE] unexpected character"
            )));
        }
        let raw = &input[start..i];
        let text = if string {
            serde_json::from_str::<String>(raw)
                .map_err(|e| Error(format!("{file}:{line}:{column}: [E_PARSE] {e}")))?
        } else {
            raw.to_owned()
        };
        tokens.push(Token {
            text,
            string,
            line,
            column,
        });
        // JSON strings cannot contain unescaped physical newlines.
        column += i - start;
        if tokens.len() > 250_000 {
            return Err(Error("[E_LIMIT] too many tokens".into()));
        }
    }
    tokens.push(Token {
        text: "<eof>".into(),
        string: false,
        line,
        column,
    });
    Ok(tokens)
}

#[derive(Debug)]
pub(super) struct Import {
    pub path: String,
    pub sha256: String,
}
#[derive(Debug)]
pub(super) struct Declaration {
    pub kind: String,
    pub id: String,
    pub value: Value,
    pub span: SourceSpan,
}
#[derive(Debug)]
pub(super) struct Unit {
    pub language: String,
    pub kind: String,
    pub package: String,
    pub version: String,
    pub imports: Vec<Import>,
    pub declarations: Vec<Declaration>,
    pub spans: Vec<SourceSpan>,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    file: String,
    spans: Vec<SourceSpan>,
}
impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
    fn at(&self, text: &str) -> bool {
        !self.peek().string && self.peek().text == text
    }
    fn error(&self, message: impl AsRef<str>) -> Error {
        Error(format!(
            "{}:{}:{}: [E_PARSE] {}",
            self.file,
            self.peek().line,
            self.peek().column,
            message.as_ref()
        ))
    }
    fn take(&mut self) -> Result<Token> {
        if self.at("<eof>") {
            return Err(self.error("unexpected end of source"));
        }
        let value = self.peek().clone();
        self.pos += 1;
        Ok(value)
    }
    fn eat(&mut self, text: &str) -> bool {
        if self.at(text) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, text: &str) -> Result<()> {
        if self.eat(text) {
            Ok(())
        } else {
            Err(self.error(format!(
                "expected {text:?}, received {:?}",
                self.peek().text
            )))
        }
    }
    fn name(&mut self) -> Result<String> {
        let token = self.take()?;
        if token.string {
            return Err(self.error("expected an identifier"));
        }
        crate::kernel::identifier(&token.text).map_err(|e| self.error(e.to_string()))?;
        Ok(token.text)
    }
    fn string(&mut self) -> Result<String> {
        let token = self.take()?;
        if !token.string {
            return Err(self.error("expected a quoted string"));
        }
        Ok(token.text)
    }
    fn sort(&mut self) -> Result<Value> {
        let name = self.name()?;
        if name == "quantity" {
            self.expect("(")?;
            let unit = self.name()?;
            self.expect(")")?;
            return Ok(json!({"quantity":{"unit":unit}}));
        }
        Ok(if matches!(name.as_str(), "integer" | "symbol") {
            json!(name)
        } else {
            json!({"domain":name})
        })
    }
    fn term(&mut self) -> Result<Value> {
        if self.eat("$") {
            return Ok(json!({"kind":"variable","name":self.name()?}));
        }
        let value = self.name()?;
        if let Ok(integer) = value.parse::<i64>() {
            if self.eat("[") {
                let unit = self.name()?;
                self.expect("]")?;
                return Ok(json!({"kind":"quantity","value":integer,"unit":unit}));
            }
            return Ok(json!({"kind":"integer","value":integer}));
        }
        if value.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
            return Err(self.error("integer literal is outside signed 64-bit range"));
        }
        Ok(json!({"kind":"symbol","value":value}))
    }
    fn claim(&mut self) -> Result<Value> {
        let negative = self.eat("not");
        let atom = self.name()?;
        self.expect("(")?;
        let mut arguments = Vec::new();
        if !self.eat(")") {
            loop {
                arguments.push(self.term()?);
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        Ok(json!({"atom":atom,"negative":negative,"arguments":arguments}))
    }
    fn list(&mut self, depth: usize) -> Result<Value> {
        let mut values = Vec::new();
        if !self.eat("]") {
            loop {
                values.push(self.value(depth + 1)?);
                if self.eat("]") {
                    break;
                }
                self.expect(",")?;
            }
        }
        Ok(Value::Array(values))
    }
    fn value(&mut self, depth: usize) -> Result<Value> {
        if depth > 32 {
            return Err(self.error("metadata nesting limit exceeded"));
        }
        if self.eat("{") {
            return self.fields(depth + 1);
        }
        if self.eat("[") {
            return self.list(depth + 1);
        }
        let token = self.take()?;
        if token.string {
            return Ok(Value::String(token.text));
        }
        match token.text.as_str() {
            "null" => Ok(Value::Null),
            "true" => Ok(json!(true)),
            "false" => Ok(json!(false)),
            text => {
                if let Ok(value) = serde_json::from_str::<serde_json::Number>(text) {
                    Ok(Value::Number(value))
                } else {
                    crate::kernel::identifier(text).map_err(|e| self.error(e.to_string()))?;
                    Ok(json!(text))
                }
            }
        }
    }
    fn fields(&mut self, depth: usize) -> Result<Value> {
        let mut map = Map::new();
        while !self.eat("}") {
            let key = self.name()?;
            let value = if matches!(key.as_str(), "scope" | "goal") {
                self.claim()?
            } else {
                self.value(depth + 1)?
            };
            self.expect(";")?;
            if map.insert(key.clone(), value).is_some() {
                return Err(self.error(format!("duplicate field: {key}")));
            }
        }
        Ok(Value::Object(map))
    }
    fn block(&mut self) -> Result<Value> {
        self.expect("{")?;
        self.fields(0)
    }
    fn span(&self, kind: &str, id: &str, start: &Token) -> SourceSpan {
        SourceSpan {
            kind: kind.into(),
            id: id.into(),
            file: self.file.clone(),
            line: start.line,
            column: start.column,
            end_line: self.tokens[self.pos.saturating_sub(1)].line,
        }
    }
    fn parameters(&mut self) -> Result<Value> {
        self.expect("(")?;
        let mut parameters = Vec::new();
        if !self.eat(")") {
            loop {
                let name = self.name()?;
                self.expect(":")?;
                parameters.push(json!({"name":name,"sort":self.sort()?}));
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        Ok(json!(parameters))
    }
    fn rule(&mut self, id: &str) -> Result<Value> {
        let parameters = self.parameters()?;
        self.expect("@")?;
        let version = self.string()?;
        self.expect("{")?;
        let mut map = Map::new();
        let mut premises = Vec::new();
        let mut exceptions = Vec::new();
        while !self.eat("}") {
            let start = self.peek().clone();
            let key = self.name()?;
            if key == "requires" {
                premises.push(self.claim()?);
                self.expect(";")?;
                self.spans.push(self.span(
                    "premise",
                    &format!("{id}@{version}/{}", premises.len() - 1),
                    &start,
                ));
            } else if key == "unless" {
                exceptions.push(self.claim()?);
                self.expect(";")?;
            } else {
                let value = if key == "ensures" {
                    self.claim()?
                } else {
                    self.value(0)?
                };
                self.expect(";")?;
                let field = if key == "ensures" {
                    "conclusion".to_owned()
                } else {
                    key
                };
                if matches!(
                    field.as_str(),
                    "parameters" | "id" | "version" | "premises" | "exceptions"
                ) {
                    return Err(self.error("reserved rule field"));
                }
                if map.insert(field.clone(), value).is_some() {
                    return Err(self.error(format!("duplicate rule field: {field}")));
                }
            }
        }
        map.insert("id".into(), json!(id));
        map.insert("version".into(), json!(version));
        map.insert("parameters".into(), parameters);
        map.insert("premises".into(), json!(premises));
        if !exceptions.is_empty() {
            map.insert("exceptions".into(), json!(exceptions));
        }
        Ok(Value::Object(map))
    }
    fn proof(&mut self) -> Result<Value> {
        self.expect("{")?;
        let mut steps = Vec::new();
        while !self.eat("}") {
            let start = self.peek().clone();
            self.expect("step")?;
            let id = self.name()?;
            self.expect(":")?;
            let claim = self.claim()?;
            self.expect("by")?;
            let reason = match self.name()?.as_str() {
                "fact" => json!({"kind":"fact","fact":self.name()?}),
                "oracle" => json!({"kind":"oracle","attestation":self.name()?}),
                "compute" => json!({"kind":"compute","operator":self.name()?}),
                "compare" => json!({"kind":"compare","operator":self.name()?}),
                "apply" => {
                    let rule = self.name()?;
                    self.expect("@")?;
                    let version = self.string()?;
                    self.expect("(")?;
                    let mut substitution = BTreeMap::new();
                    if !self.eat(")") {
                        loop {
                            let name = self.name()?;
                            self.expect("=")?;
                            if substitution.insert(name.clone(), self.term()?).is_some() {
                                return Err(self.error(format!("duplicate substitution: {name}")));
                            }
                            if self.eat(")") {
                                break;
                            }
                            self.expect(",")?;
                        }
                    }
                    self.expect("using")?;
                    self.expect("[")?;
                    let premises = self.list(0)?;
                    json!({"kind":"apply","rule":rule,"version":version,"premises":premises,"substitution":substitution})
                }
                other => return Err(self.error(format!("unknown proof reason: {other}"))),
            };
            self.expect(";")?;
            self.spans.push(self.span("step", &id, &start));
            steps.push(json!({"id":id,"claim":claim,"reason":reason}));
        }
        Ok(json!(steps))
    }
    fn parse(mut self) -> Result<Unit> {
        self.expect("oathkeeper")?;
        let language = self.string()?;
        if !matches!(language.as_str(), "0.3" | "0.4" | "0.5") {
            return Err(self.error("expected oathkeeper \"0.3\", \"0.4\" or \"0.5\""));
        }
        self.expect(";")?;
        let kind = self.name()?;
        if !matches!(kind.as_str(), "module" | "case") {
            return Err(self.error("expected module or case"));
        }
        let package = self.name()?;
        self.expect("@")?;
        let version = self.string()?;
        self.expect(";")?;
        let mut imports = Vec::new();
        let mut declarations = Vec::new();
        while !self.at("<eof>") {
            let start = self.peek().clone();
            let declaration = self.name()?;
            if declaration == "import" {
                let path = self.string()?;
                self.expect("sha256")?;
                let sha256 = self.string()?;
                self.expect(";")?;
                imports.push(Import { path, sha256 });
                continue;
            }
            if kind == "case"
                && !matches!(
                    declaration.as_str(),
                    "source"
                        | "constant"
                        | "fact"
                        | "attestation"
                        | "policy"
                        | "query"
                        | "proof"
                        | "retraction"
                )
            {
                return Err(self.error("a case cannot redefine module declarations"));
            }
            if kind == "module"
                && matches!(
                    declaration.as_str(),
                    "fact" | "attestation" | "policy" | "query" | "proof" | "retraction"
                )
            {
                return Err(
                    self.error("case evidence and proofs cannot be declared in a law module")
                );
            }
            let id = if matches!(declaration.as_str(), "query" | "proof") {
                declaration.clone()
            } else {
                self.name()?
            };
            let value = match declaration.as_str() {
                "type" => {
                    self.expect(";")?;
                    json!(id)
                }
                "predicate" => {
                    self.expect("(")?;
                    let mut sorts = Vec::new();
                    if !self.eat(")") {
                        loop {
                            sorts.push(self.sort()?);
                            if self.eat(")") {
                                break;
                            }
                            self.expect(",")?;
                        }
                    }
                    self.expect(";")?;
                    json!({"id":id,"arguments":sorts})
                }
                "constant" => {
                    self.expect(":")?;
                    let sort = self.sort()?;
                    self.expect(";")?;
                    json!({"id":id,"sort":sort})
                }
                "rule" => self.rule(&id)?,
                "source" | "jurisdiction" | "authority" | "assessment" | "retraction"
                | "coverage" | "interpretation" => {
                    let mut value = self.block()?;
                    if declaration != "source"
                        && value
                            .as_object_mut()
                            .unwrap()
                            .insert("id".into(), json!(id))
                            .is_some()
                    {
                        return Err(self.error("id is already supplied by the declaration"));
                    }
                    value
                }
                "fact" | "attestation" | "policy" => {
                    self.expect(":")?;
                    let claim = self.claim()?;
                    let mut value = self.block()?;
                    let map = value.as_object_mut().unwrap();
                    if map.contains_key("id") || map.contains_key("claim") {
                        return Err(self.error("id/claim is already supplied by the declaration"));
                    }
                    map.insert("id".into(), json!(id));
                    map.insert("claim".into(), claim);
                    value
                }
                "query" => self.block()?,
                "proof" => self.proof()?,
                other => return Err(self.error(format!("unsupported declaration: {other}"))),
            };
            let span = self.span(&declaration, &id, &start);
            self.spans.push(span.clone());
            declarations.push(Declaration {
                kind: declaration,
                id,
                value,
                span,
            });
        }
        Ok(Unit {
            language,
            kind,
            package,
            version,
            imports,
            declarations,
            spans: self.spans,
        })
    }
}

pub(super) fn parse(input: &str, file: &str) -> Result<Unit> {
    Parser {
        tokens: lex(input, file)?,
        pos: 0,
        file: file.into(),
        spans: Vec::new(),
    }
    .parse()
}
