use oath_core::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
struct Token {
    text: String,
    kind: u8,
    span: Span,
}
#[derive(Clone, Debug)]
pub(crate) enum Decl {
    Type(TypeDef),
    Const(Constant),
    Function(Function),
    Query(Query),
    Rule(Rule),
    Priority(Priority),
}
#[derive(Clone, Debug)]
pub(crate) struct Module {
    pub name: String,
    pub imports: BTreeMap<String, String>,
    pub declarations: Vec<Decl>,
}

fn lex(file: &str, text: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut tokens = Vec::new();
    let bytes = text.as_bytes();
    let (mut i, mut line, mut col) = (0, 1, 1);
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            if bytes[i] == b'\n' {
                line += 1;
                col = 1
            } else {
                col += 1
            }
            i += 1;
            continue;
        }
        if bytes[i..].starts_with(b"//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
                col += 1
            }
            continue;
        }
        let span = Span {
            file: file.into(),
            start: i,
            end: i + 1,
            line,
            column: col,
        };
        if bytes[i..].starts_with(b"/*") {
            let mut depth = 1;
            i += 2;
            col += 2;
            while i < bytes.len() && depth > 0 {
                if bytes[i..].starts_with(b"/*") {
                    depth += 1;
                    i += 2;
                    col += 2
                } else if bytes[i..].starts_with(b"*/") {
                    depth -= 1;
                    i += 2;
                    col += 2
                } else {
                    if bytes[i] == b'\n' {
                        line += 1;
                        col = 1
                    } else {
                        col += 1
                    }
                    i += 1
                }
            }
            if depth != 0 {
                return Err(Diagnostic::new("Lex", "unterminated comment", &span));
            }
            continue;
        }
        let start = i;
        let kind;
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            kind = 0;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1
            }
        } else if bytes[i].is_ascii_digit() {
            kind = 1;
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1
            }
            if i + 1 < bytes.len() && bytes[i] == b'.' && bytes[i + 1].is_ascii_digit() {
                i += 1;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1
                }
            }
        } else if bytes[i] == b'"' {
            kind = 2;
            i += 1;
            let mut closed = false;
            while i < bytes.len() {
                if bytes[i] == b'"' {
                    i += 1;
                    closed = true;
                    break;
                }
                if bytes[i] == b'\\' {
                    i += 1
                }
                if i >= bytes.len() {
                    break;
                }
                if bytes[i] == b'\n' {
                    return Err(Diagnostic::new("Lex", "newline in string", &span));
                }
                i += 1
            }
            if !closed {
                return Err(Diagnostic::new("Lex", "unterminated string", &span));
            }
        } else {
            kind = 3;
            let pair = bytes.get(i..i + 2);
            if pair
                .is_some_and(|p| [b"->".as_slice(), b"==", b"!=", b"<=", b">=", b"::"].contains(&p))
            {
                i += 2
            } else if b"{}()[];,:.=<>+-*/%!".contains(&bytes[i]) {
                i += 1
            } else {
                return Err(Diagnostic::new("Lex", "unexpected character", &span));
            }
        }
        col += i - start;
        tokens.push(Token {
            text: text[start..i].into(),
            kind,
            span: Span { end: i, ..span },
        });
        if tokens.len() > 1_000_000 {
            return Err(Diagnostic::new(
                "Limit",
                "token limit exceeded",
                &tokens.last().unwrap().span,
            ));
        }
    }
    tokens.push(Token {
        text: "<eof>".into(),
        kind: 4,
        span: Span {
            file: file.into(),
            start: i,
            end: i,
            line,
            column: col,
        },
    });
    Ok(tokens)
}
pub(crate) fn parse(
    file: &str,
    source: &str,
    expressions: &mut Vec<Expr>,
) -> Result<Module, Diagnostic> {
    if source.len() > 64 * 1024 * 1024 {
        return Err(Diagnostic::new(
            "Limit",
            "source byte limit exceeded",
            &Span {
                file: file.into(),
                ..Span::default()
            },
        ));
    }
    let tokens = lex(file, source)?;
    Parser {
        tokens,
        pos: 0,
        expressions,
        depth: 0,
    }
    .module()
}
struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    expressions: &'a mut Vec<Expr>,
    depth: usize,
}
impl Parser<'_> {
    fn token(&self) -> &Token {
        &self.tokens[self.pos]
    }
    fn at(&self, text: &str) -> bool {
        self.token().text == text
    }
    fn take(&mut self, text: &str) -> bool {
        // In type context, a closing angle may be adjacent to an assignment.
        if text == ">" && self.at(">=") {
            let token = &mut self.tokens[self.pos];
            token.text = "=".into();
            token.span.start += 1;
            token.span.column += 1;
            return true;
        }
        if self.at(text) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn err(&self, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new("Parse", message, &self.token().span)
    }
    fn expect(&mut self, text: &str) -> Result<(), Diagnostic> {
        if self.take(text) {
            Ok(())
        } else {
            Err(self.err(format!("expected {text}, found {}", self.token().text)))
        }
    }
    fn ident(&mut self) -> Result<String, Diagnostic> {
        if self.token().kind == 0 {
            let s = self.token().text.clone();
            self.pos += 1;
            Ok(s)
        } else {
            Err(self.err("expected identifier"))
        }
    }
    fn path(&mut self) -> Result<String, Diagnostic> {
        let mut p = self.ident()?;
        while self.at(".") && self.tokens.get(self.pos + 1).is_some_and(|t| t.kind == 0) {
            self.pos += 1;
            p.push('.');
            p.push_str(&self.ident()?)
        }
        Ok(p)
    }
    fn add(&mut self, kind: ExprKind, span: Span) -> ExprId {
        let id = self.expressions.len() as u32;
        self.expressions.push(Expr {
            id,
            ty: Ty::Unknown,
            kind,
            span,
        });
        id
    }
    fn generic_params(&mut self) -> Result<Vec<String>, Diagnostic> {
        let mut p = Vec::new();
        if self.take("<") {
            loop {
                p.push(self.ident()?);
                if self.take(">") {
                    break;
                }
                self.expect(",")?
            }
        }
        Ok(p)
    }
    fn types(&mut self) -> Result<Vec<Ty>, Diagnostic> {
        self.expect("<")?;
        let mut args = Vec::new();
        loop {
            args.push(self.ty()?);
            if self.take(">") {
                break;
            }
            self.expect(",")?
        }
        Ok(args)
    }
    fn ty(&mut self) -> Result<Ty, Diagnostic> {
        self.depth += 1;
        if self.depth > 128 {
            return Err(self.err("type nesting limit exceeded"));
        }
        let name = self.path()?;
        let args = if self.at("<") { self.types()? } else { vec![] };
        let ty = match (name.as_str(), args.as_slice()) {
            ("Unit", []) => Ty::Unit,
            ("Bool", []) => Ty::Bool,
            ("Text", []) => Ty::Text,
            ("Int", []) => Ty::Int,
            ("Rational", []) => Ty::Rational,
            ("List", [t]) => Ty::List(Box::new(t.clone())),
            _ => Ty::Named { name, args },
        };
        self.depth -= 1;
        Ok(ty)
    }
    fn params(&mut self) -> Result<Vec<Param>, Diagnostic> {
        self.expect("(")?;
        let mut args = Vec::new();
        if !self.take(")") {
            loop {
                let name = self.ident()?;
                self.expect(":")?;
                args.push(Param {
                    name,
                    ty: self.ty()?,
                });
                if self.take(")") {
                    break;
                }
                self.expect(",")?
            }
        }
        Ok(args)
    }
    fn bindings(&mut self) -> Result<Vec<String>, Diagnostic> {
        self.expect("(")?;
        let mut args = Vec::new();
        if !self.take(")") {
            loop {
                args.push(self.ident()?);
                if self.take(")") {
                    break;
                }
                self.expect(",")?
            }
        }
        Ok(args)
    }
    fn module(&mut self) -> Result<Module, Diagnostic> {
        self.expect("module")?;
        let name = self.path()?;
        self.expect(";")?;
        let mut imports = BTreeMap::new();
        let mut declarations = Vec::new();
        while !self.at("<eof>") {
            let span = self.token().span.clone();
            if self.take("use") {
                let path = self.path()?;
                if self.take(".") {
                    self.expect("{")?;
                    loop {
                        let item = self.ident()?;
                        let alias = if self.take("as") {
                            self.ident()?
                        } else {
                            item.clone()
                        };
                        if imports.insert(alias, format!("{path}.{item}")).is_some() {
                            return Err(self.err("duplicate import alias"));
                        }
                        if self.take("}") {
                            break;
                        }
                        self.expect(",")?;
                        if self.take("}") {
                            break;
                        }
                    }
                } else {
                    let alias = if self.take("as") {
                        self.ident()?
                    } else {
                        path.rsplit('.').next().unwrap().into()
                    };
                    if imports.insert(alias, path).is_some() {
                        return Err(self.err("duplicate import alias"));
                    }
                }
                self.expect(";")?;
                continue;
            }
            if self.take("type") {
                let n = self.ident()?;
                let parameters = self.generic_params()?;
                self.expect("=")?;
                let body = if self.take("{") {
                    let mut fields = BTreeMap::new();
                    while !self.take("}") {
                        let field = self.ident()?;
                        self.expect(":")?;
                        if fields.insert(field, self.ty()?).is_some() {
                            return Err(self.err("duplicate field"));
                        }
                        if !self.take(",") {
                            self.expect("}")?;
                            break;
                        }
                    }
                    TypeBody::Record { fields }
                } else {
                    TypeBody::Alias { ty: self.ty()? }
                };
                self.expect(";")?;
                declarations.push(Decl::Type(TypeDef {
                    name: format!("{name}.{n}"),
                    parameters,
                    body,
                    span,
                }));
            } else if self.take("enum") {
                let n = self.ident()?;
                let parameters = self.generic_params()?;
                self.expect("{")?;
                let mut variants = BTreeMap::new();
                while !self.take("}") {
                    let variant = self.ident()?;
                    let mut fields = Vec::new();
                    if self.take("(") && !self.take(")") {
                        loop {
                            fields.push(self.ty()?);
                            if self.take(")") {
                                break;
                            }
                            self.expect(",")?
                        }
                    }
                    if variants.insert(variant, fields).is_some() {
                        return Err(self.err("duplicate variant"));
                    }
                    if !self.take(",") {
                        self.expect("}")?;
                        break;
                    }
                }
                self.take(";");
                declarations.push(Decl::Type(TypeDef {
                    name: format!("{name}.{n}"),
                    parameters,
                    body: TypeBody::Enum { variants },
                    span,
                }));
            } else if self.take("const") {
                let n = self.ident()?;
                let ty = if self.take(":") {
                    self.ty()?
                } else {
                    Ty::Unknown
                };
                self.expect("=")?;
                let value = self.expr(0)?;
                self.expect(";")?;
                declarations.push(Decl::Const(Constant {
                    name: format!("{name}.{n}"),
                    ty,
                    value,
                    span,
                }));
            } else if self.take("fn") {
                let n = self.ident()?;
                let parameters = self.generic_params()?;
                let args = self.params()?;
                self.expect("->")?;
                let returns = self.ty()?;
                let body = self.block()?;
                declarations.push(Decl::Function(Function {
                    name: format!("{name}.{n}"),
                    parameters,
                    args,
                    returns,
                    body,
                    span,
                }));
            } else if self.take("query") {
                let n = self.ident()?;
                let args = self.params()?;
                self.expect("->")?;
                let returns = self.ty()?;
                self.expect("{")?;
                let (mut scope, mut applies, mut meta) = (None, None, None);
                while !self.take("}") {
                    let key = self.ident()?;
                    self.expect("=")?;
                    match key.as_str() {
                        "scope" => {
                            if scope.is_some() {
                                return Err(self.err("duplicate scope"));
                            }
                            scope = Some(self.expr(0)?)
                        }
                        "applies" => {
                            if applies.is_some() {
                                return Err(self.err("duplicate applies"));
                            }
                            applies = Some(self.expr(0)?)
                        }
                        "meta" => {
                            if meta.is_some() {
                                return Err(self.err("duplicate meta"));
                            }
                            meta = Some(self.path()?)
                        }
                        _ => return Err(self.err("unknown query field")),
                    }
                    self.expect(";")?
                }
                declarations.push(Decl::Query(Query {
                    name: format!("{name}.{n}"),
                    args,
                    returns,
                    scope: scope.ok_or_else(|| self.err("query requires scope"))?,
                    applies: applies.ok_or_else(|| self.err("query requires applies"))?,
                    meta,
                    span,
                }));
            } else if self.at("rule") || self.at("priority") {
                let priority = self.take("priority");
                if !priority {
                    self.expect("rule")?
                }
                let n = self.ident()?;
                self.expect("for")?;
                let query = self.path()?;
                let bindings = self.bindings()?;
                self.expect("{")?;
                let (mut when, mut value, mut meta, mut higher, mut lower) =
                    (None, None, None, None, None);
                let mut seen = std::collections::BTreeSet::new();
                while !self.take("}") {
                    let key = self.ident()?;
                    if !seen.insert(key.clone()) {
                        return Err(self.err("duplicate declaration field"));
                    }
                    self.expect("=")?;
                    match key.as_str() {
                        "when" => when = Some(self.expr(0)?),
                        "yield" if !priority => value = Some(self.expr(0)?),
                        "meta" => meta = Some(self.path()?),
                        "higher" if priority => higher = Some(self.path()?),
                        "lower" if priority => lower = Some(self.path()?),
                        _ => return Err(self.err("unknown rule/priority field")),
                    }
                    self.expect(";")?
                }
                let when = when.ok_or_else(|| self.err("rule/priority requires when"))?;
                if priority {
                    declarations.push(Decl::Priority(Priority {
                        name: format!("{name}.{n}"),
                        query,
                        bindings,
                        higher: higher.ok_or_else(|| self.err("priority requires higher"))?,
                        lower: lower.ok_or_else(|| self.err("priority requires lower"))?,
                        when,
                        meta,
                        span,
                    }))
                } else {
                    declarations.push(Decl::Rule(Rule {
                        name: format!("{name}.{n}"),
                        query,
                        bindings,
                        when,
                        value: value.ok_or_else(|| self.err("rule requires yield"))?,
                        meta,
                        span,
                    }))
                }
            } else {
                return Err(self.err("expected declaration"));
            }
        }
        Ok(Module {
            name,
            imports,
            declarations,
        })
    }
    fn block(&mut self) -> Result<ExprId, Diagnostic> {
        self.expect("{")?;
        let mut bindings = Vec::new();
        while self.take("let") {
            let span = self.tokens[self.pos - 1].span.clone();
            let name = self.ident()?;
            self.expect("=")?;
            let value = self.expr(0)?;
            self.expect(";")?;
            bindings.push((name, value, span))
        }
        let mut body = if self.at("}") {
            self.add(
                ExprKind::Literal { value: Value::Unit },
                self.token().span.clone(),
            )
        } else {
            self.expr(0)?
        };
        self.take(";");
        self.expect("}")?;
        for (name, value, span) in bindings.into_iter().rev() {
            body = self.add(ExprKind::Let { name, value, body }, span)
        }
        Ok(body)
    }
    fn expr(&mut self, min: u8) -> Result<ExprId, Diagnostic> {
        self.depth += 1;
        if self.depth > 128 {
            return Err(self.err("expression nesting limit exceeded"));
        }
        let span = self.token().span.clone();
        let mut left;
        if self.take("if") {
            let condition = self.expr(0)?;
            let yes = self.block()?;
            self.expect("else")?;
            let no = if self.at("if") {
                self.expr(0)?
            } else {
                self.block()?
            };
            left = self.add(ExprKind::If { condition, yes, no }, span.clone());
        } else if self.take("match") {
            let value = self.expr(0)?;
            self.expect("{")?;
            let mut arms = Vec::new();
            while !self.take("}") {
                let pattern = if self.take("_") {
                    Pattern::Wildcard
                } else if self.take("true") {
                    Pattern::Bool { value: true }
                } else if self.take("false") {
                    Pattern::Bool { value: false }
                } else {
                    let p = self.path()?;
                    if let Some((name, variant)) = p.rsplit_once('.') {
                        let bindings = if self.at("(") {
                            self.bindings()?
                        } else {
                            vec![]
                        };
                        Pattern::Variant {
                            name: name.into(),
                            variant: variant.into(),
                            bindings,
                        }
                    } else {
                        Pattern::Bind { name: p }
                    }
                };
                self.expect("=")?;
                self.expect(">")?;
                let body = if self.at("{") {
                    self.block()?
                } else {
                    self.expr(0)?
                };
                arms.push(Arm { pattern, body });
                if !self.take(",") {
                    self.expect("}")?;
                    break;
                }
            }
            left = self.add(ExprKind::Match { value, arms }, span.clone());
        } else if self.at("-") || self.at("not") || self.at("!") {
            let op = if self.take("-") {
                "-"
            } else {
                self.pos += 1;
                "not"
            };
            let value = self.expr(7)?;
            left = self.add(
                ExprKind::Unary {
                    op: op.into(),
                    value,
                },
                span.clone(),
            );
        } else if self.take("(") {
            if self.take(")") {
                left = self.add(ExprKind::Literal { value: Value::Unit }, span.clone())
            } else {
                left = self.expr(0)?;
                self.expect(")")?
            }
        } else if self.take("[") {
            let mut items = Vec::new();
            while !self.take("]") {
                items.push(self.expr(0)?);
                if !self.take(",") {
                    self.expect("]")?;
                    break;
                }
            }
            left = self.add(ExprKind::List { items }, span.clone());
        } else if self.token().kind == 1 {
            let token = self.token().text.clone();
            self.pos += 1;
            let value = if token.contains('.') {
                decimal(&token).map_err(|e| Diagnostic::new("Number", e, &span))?
            } else {
                integer(&token, &ExecutionLimits::default())
                    .map_err(|e| Diagnostic::new("Number", e, &span))?;
                Value::Int { value: token }
            };
            left = self.add(ExprKind::Literal { value }, span.clone());
        } else if self.token().kind == 2 {
            let value = serde_json::from_str::<String>(&self.token().text)
                .map_err(|e| Diagnostic::new("String", e.to_string(), &span))?;
            self.pos += 1;
            left = self.add(
                ExprKind::Literal {
                    value: Value::Text { value },
                },
                span.clone(),
            );
        } else if self.at("true") || self.at("false") {
            let value = self.take("true");
            if !value {
                self.expect("false")?
            }
            left = self.add(
                ExprKind::Literal {
                    value: Value::Bool { value },
                },
                span.clone(),
            );
        } else if self.token().kind == 0 {
            let name = self.path()?;
            let type_args = if self.take("::") {
                self.types()?
            } else {
                vec![]
            };
            if self.take("(") {
                let mut args = Vec::new();
                while !self.take(")") {
                    args.push(self.expr(0)?);
                    if !self.take(",") {
                        self.expect(")")?;
                        break;
                    }
                }
                left = self.add(
                    ExprKind::Call {
                        target: name,
                        type_args,
                        args,
                    },
                    span.clone(),
                );
            } else if self.at("{")
                && (self.tokens.get(self.pos + 1).is_some_and(|t| t.text == "}")
                    || self.tokens.get(self.pos + 2).is_some_and(|t| t.text == ":"))
            {
                self.pos += 1;
                let mut fields = BTreeMap::new();
                while !self.take("}") {
                    let key = self.ident()?;
                    self.expect(":")?;
                    if fields.insert(key, self.expr(0)?).is_some() {
                        return Err(self.err("duplicate record field"));
                    }
                    if !self.take(",") {
                        self.expect("}")?;
                        break;
                    }
                }
                left = self.add(
                    ExprKind::Record {
                        name,
                        type_args,
                        fields,
                    },
                    span.clone(),
                );
            } else {
                if !type_args.is_empty() {
                    return Err(self.err("type arguments require a constructor or call"));
                }
                left = self.add(ExprKind::Name { name }, span.clone())
            }
        } else if self.at("{") {
            left = self.block()?
        } else {
            return Err(self.err("expected expression"));
        }
        loop {
            if self.take(".") {
                let field = self.ident()?;
                left = self.add(ExprKind::Field { value: left, field }, span.clone());
                continue;
            }
            let op = self.token().text.clone();
            let power = match op.as_str() {
                "or" => 1,
                "and" => 2,
                "==" | "!=" => 3,
                "<" | "<=" | ">" | ">=" => 4,
                "+" | "-" => 5,
                "*" | "/" | "%" => 6,
                _ => 0,
            };
            if power == 0 || power < min {
                break;
            }
            self.pos += 1;
            let right = self.expr(power + 1)?;
            left = self.add(ExprKind::Binary { op, left, right }, span.clone());
        }
        self.depth -= 1;
        Ok(left)
    }
}
