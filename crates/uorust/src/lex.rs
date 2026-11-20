//! The LexLean authoring notation, printed in the canonical layout `lexlean fmt` keeps.
//!
//! The printer only knows the declaration shapes the projection emits. A shape
//! is printed flat when the flat line fits the formatter's width and broken one
//! member per line otherwise; a `match` always breaks before each arm. The
//! layout is checked against `lexlean fmt --check` by the projection tests, so
//! a drift in the formatter shows up as a red test, never as a silent rewrite.

/// The formatter's line width.
const WIDTH: usize = 100;

#[cfg(test)]
thread_local! { static SYNTAX_PARSES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

/// The same declaration boundaries serve proof slicing and nominal imports.
struct ModuleSyntax<'a> {
    start: usize,
    finish: usize,
    tokens: Vec<(usize, usize, &'a str)>,
    blocks: Vec<(usize, usize, Option<&'a str>)>,
    names: std::collections::BTreeMap<&'a str, usize>,
    groups: std::collections::BTreeMap<&'a str, Vec<usize>>,
}

fn module_syntax(text: &str) -> Result<ModuleSyntax<'_>, crate::UorustError> {
    #[cfg(test)]
    SYNTAX_PARSES.with(|count| count.set(count.get() + 1));
    use std::collections::BTreeMap;
    let bad = |reason: &str| crate::UorustError::Tool(format!("proof closure: {reason}"));
    let begin = "\\begin{semanticmodule}";
    let end = "\\end{semanticmodule}";
    let start = text
        .find(begin)
        .ok_or_else(|| bad("missing semantic module"))?
        + begin.len();
    let finish = text[start..]
        .find(end)
        .ok_or_else(|| bad("missing module end"))?
        + start;
    // Token offsets refer to the original UTF-8 bytes. Quoted content cannot
    // introduce declarations, references, braces or mutual-group fields.
    let mut tokens = Vec::new();
    let bytes = text.as_bytes();
    let mut i = start;
    while i < finish {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let at = i;
        if bytes[i] == b'"' {
            i += 1;
            let mut closed = false;
            while i < finish {
                match bytes[i] {
                    b'\\' => i += 2,
                    b'"' => {
                        i += 1;
                        closed = true;
                        break;
                    }
                    _ => i += 1,
                }
            }
            if !closed {
                return Err(bad("unterminated string"));
            }
            tokens.push((at, i, ""));
        } else if bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' {
            i += 1;
            while i < finish && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            tokens.push((at, i, &text[at..i]));
        } else {
            let ch = text[i..]
                .chars()
                .next()
                .ok_or_else(|| bad("invalid token"))?;
            i += ch.len_utf8();
            tokens.push((at, i, &text[at..i]));
        }
    }
    let mut blocks = Vec::new();
    let mut names = BTreeMap::new();
    let mut groups: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    let mut pos = 0;
    while pos < tokens.len() {
        let first = pos;
        if !matches!(
            tokens[pos].2,
            "structure" | "inductive" | "definition" | "theorem"
        ) || tokens.get(pos + 1).map(|t| t.2) != Some("{")
        {
            return Err(bad(&format!(
                "unsupported trial declaration at byte {}",
                tokens[pos].0
            )));
        }
        pos += 1;
        let mut depth = 0;
        let mut name = None;
        let mut group = None;
        loop {
            let token = tokens
                .get(pos)
                .ok_or_else(|| bad("unclosed declaration"))?
                .2;
            if depth == 1
                && matches!(token, "name" | "mutual")
                && tokens.get(pos + 1).map(|t| t.2) == Some(":")
            {
                let value = tokens
                    .get(pos + 2)
                    .ok_or_else(|| bad("missing field value"))?
                    .2;
                if value.is_empty() {
                    return Err(bad("quoted declaration identity"));
                }
                if token == "name" {
                    name = Some(value);
                } else {
                    group = Some(value);
                }
            }
            match token {
                "{" | "[" | "(" => depth += 1,
                "}" | "]" | ")" => depth -= 1,
                _ => {}
            }
            pos += 1;
            if depth == 0 {
                break;
            }
        }
        let name = name.ok_or_else(|| bad("declaration has no name"))?;
        let index = blocks.len();
        if names.insert(name, index).is_some() {
            return Err(bad(&format!("duplicate declaration {name}")));
        }
        if let Some(group) = group {
            groups.entry(group).or_default().push(index);
        }
        blocks.push((first, pos, group));
    }
    Ok(ModuleSyntax {
        start,
        finish,
        tokens,
        blocks,
        names,
        groups,
    })
}

/// Names of actual type declarations, excluding terms and nested constructors.
pub(crate) fn type_names(
    text: &str,
) -> Result<std::collections::BTreeSet<String>, crate::UorustError> {
    let module = module_syntax(text)?;
    Ok(module
        .names
        .iter()
        .filter(|(_, index)| {
            matches!(
                module.tokens[module.blocks[**index].0].2,
                "structure" | "inductive"
            )
        })
        .map(|(name, _)| (*name).to_string())
        .collect())
}

/// Dependencies between actual emitted declarations, using the same token
/// boundaries and opaque strings as proof slicing. Function values count too.
pub(crate) fn declaration_dependencies(
    text: &str,
) -> Option<std::collections::BTreeMap<String, std::collections::BTreeSet<String>>> {
    let module = module_syntax(text).ok()?;
    Some(
        module
            .names
            .iter()
            .map(|(name, index)| {
                let (first, last, _) = module.blocks[*index];
                let mut dependencies = std::collections::BTreeSet::new();
                for i in first..last {
                    // Declaration and nested field/group labels do not use a value.
                    if i >= first + 2
                        && module.tokens[i - 1].2 == ":"
                        && matches!(module.tokens[i - 2].2, "name" | "mutual")
                    {
                        continue;
                    }
                    let token = module.tokens[i].2;
                    if token != *name && module.names.contains_key(token) {
                        dependencies.insert(token.to_string());
                    }
                }
                ((*name).to_string(), dependencies)
            })
            .collect(),
    )
}

/// Change only observed policy values in an existing projected package.
/// Declaration boundaries and quoted content use the same lexer as slicing.
pub(crate) fn with_axiom_policies(
    text: &str,
    policies: &std::collections::BTreeMap<String, Vec<String>>,
) -> Result<String, crate::UorustError> {
    let bad = |why: &str| crate::UorustError::Tool(format!("proof policy: {why}"));
    let module = module_syntax(text)?;
    let mut edits = Vec::new();
    for (name, axioms) in policies {
        let index = module
            .names
            .get(name.as_str())
            .ok_or_else(|| bad("missing declaration"))?;
        let (first, last, _) = module.blocks[*index];
        if !matches!(module.tokens[first].2, "definition" | "theorem") {
            return Err(bad("a policy observation does not name a term declaration"));
        }
        if axioms.iter().any(|axiom| {
            axiom.split('.').any(|part| {
                let mut chars = part.chars();
                !chars
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                    || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
            })
        }) {
            return Err(bad("invalid observed axiom identity"));
        }
        let policy = format!("[{}]", axioms.join(", "));
        let mut depth = 0;
        let mut found = None;
        for position in first + 1..last {
            let token = module.tokens[position].2;
            if depth == 1
                && token == "axioms"
                && module.tokens.get(position + 1).map(|t| t.2) == Some(":")
            {
                if found.is_some() {
                    return Err(bad("duplicate policy field"));
                }
                let start = position + 2;
                if module.tokens.get(start).map(|t| t.2) != Some("[") {
                    return Err(bad("policy field is not a list"));
                }
                let end = (start + 1..last)
                    .find(|i| module.tokens[*i].2 == "]")
                    .ok_or_else(|| bad("unclosed policy field"))?;
                if module.tokens[start + 1..end]
                    .iter()
                    .any(|t| matches!(t.2, "[" | "{" | "("))
                {
                    return Err(bad("nested policy value"));
                }
                found = Some((module.tokens[start].0, module.tokens[end].1));
            }
            match token {
                "{" | "[" | "(" => depth += 1,
                "}" | "]" | ")" => depth -= 1,
                _ => {}
            }
        }
        if let Some((start, end)) = found {
            edits.push((start, end, policy));
        } else {
            let at = module.tokens[last - 1].0;
            let comma = if module.tokens[last - 2].2 == "," {
                ""
            } else {
                ","
            };
            edits.push((at, at, format!("{comma} axioms: {policy}")));
        }
    }
    edits.sort_by_key(|edit| edit.0);
    let mut result = text.to_string();
    for (start, end, replacement) in edits.into_iter().rev() {
        result.replace_range(start..end, &replacement);
    }
    Ok(result)
}

/// One immutable lexical graph for all proof roots of the same model.
pub(crate) struct ProofClosureGraph<'a> {
    text: &'a str,
    start: usize,
    finish: usize,
    names: std::collections::BTreeMap<&'a str, usize>,
    spans: Vec<(usize, usize)>,
    dependencies: Vec<Vec<usize>>,
}

impl<'a> ProofClosureGraph<'a> {
    pub(crate) fn contains(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }

    pub(crate) fn new(text: &'a str) -> Result<Self, crate::UorustError> {
        use std::collections::BTreeSet;
        let ModuleSyntax {
            start,
            finish,
            tokens,
            blocks,
            names,
            groups,
        } = module_syntax(text)?;
        let dependencies = blocks
            .iter()
            .map(|(first, last, group)| {
                let mut reached: BTreeSet<usize> = tokens[*first..*last]
                    .iter()
                    .filter_map(|(_, _, token)| names.get(token).copied())
                    .collect();
                if let Some(group) = group {
                    reached.extend(&groups[group]);
                }
                reached.into_iter().collect()
            })
            .collect();
        let spans = blocks
            .iter()
            .map(|(first, last, _)| (tokens[*first].0, tokens[*last - 1].1))
            .collect();
        Ok(Self {
            text,
            start,
            finish,
            names,
            spans,
            dependencies,
        })
    }

    pub(crate) fn closure(&self, roots: &[String]) -> Result<String, crate::UorustError> {
        use std::collections::BTreeSet;
        let bad = |reason: &str| crate::UorustError::Tool(format!("proof closure: {reason}"));
        let mut pending = Vec::new();
        for root in roots {
            pending.push(
                *self
                    .names
                    .get(root.as_str())
                    .ok_or_else(|| bad(&format!("missing proof obligation {root}")))?,
            );
        }
        if pending.is_empty() {
            return Err(bad("empty proof obligations"));
        }
        let mut kept = BTreeSet::new();
        while let Some(index) = pending.pop() {
            if kept.insert(index) {
                pending.extend(&self.dependencies[index]);
            }
        }
        let mut out = self.text[..self.start].to_string();
        out.push('\n');
        for index in kept {
            let (first, last) = self.spans[index];
            out.push_str(&self.text[first..last]);
            out.push_str("\n\n");
        }
        out.push_str(&self.text[self.finish..]);
        Ok(out)
    }
}

/// Keep complete emitted declarations reachable from every proof obligation.
/// Strings are opaque and mutual groups are indivisible. The retained bytes
/// are unchanged; this only scopes native trials, never full-project output.
pub(crate) fn proof_closure(text: &str, roots: &[String]) -> Result<String, crate::UorustError> {
    ProofClosureGraph::new(text)?.closure(roots)
}

/// An expression: a flat term, or a form whose `match` may break its arms
/// onto lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Expr {
    /// A term printed on one line.
    Flat(String),
    /// `(match scrutinee | pattern => expr ...)`.
    Match { scrutinee: String, arms: Vec<Arm> },
    /// `(if condition then a else b)` where a branch may hold a match.
    If {
        condition: Box<Expr>,
        then: Box<Expr>,
        otherwise: Box<Expr>,
    },
    /// `(let name : type := value; body)`.
    Let {
        name: String,
        ty: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
}

/// One arm of a `match`: its pattern and its value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Arm {
    pub(crate) pattern: String,
    pub(crate) body: Expr,
}

impl Expr {
    /// The structured form of a term held as text: a top-level `(match
    /// scrutinee | pattern => body ...)` as a [`Expr::Match`] (its arms'
    /// bodies read the same way), so the printer lays it out as `lexlean
    /// fmt` does; any other term flat.
    pub(crate) fn read(text: &str) -> Self {
        let flat = || Self::Flat(text.to_string());
        let Some(inner) = text
            .strip_prefix("(match ")
            .and_then(|t| t.strip_suffix(')'))
        else {
            return flat();
        };
        // The pieces of `inner` between its ` | ` at depth 0, outside strings;
        // `None` when its parentheses do not close inside it.
        let mut pieces = Vec::new();
        let (mut depth, mut start, mut quoted, mut escaped) = (0i32, 0, false, false);
        let bytes = inner.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            if quoted {
                match b {
                    _ if escaped => escaped = false,
                    b'\\' => escaped = true,
                    b'"' => quoted = false,
                    _ => {}
                }
                continue;
            }
            match b {
                b'"' => quoted = true,
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => {
                    depth -= 1;
                    if depth < 0 {
                        return flat();
                    }
                }
                b'|' if depth == 0
                    && inner[..i].ends_with(' ')
                    && inner[i + 1..].starts_with(' ') =>
                {
                    pieces.push(&inner[start..i - 1]);
                    start = i + 2;
                }
                _ => {}
            }
        }
        if depth != 0 || quoted || pieces.is_empty() {
            return flat();
        }
        pieces.push(&inner[start..]);
        let scrutinee = pieces[0].to_string();
        let mut arms = Vec::new();
        for piece in &pieces[1..] {
            let Some((pattern, body)) = piece.split_once(" => ") else {
                return flat();
            };
            arms.push(Arm {
                pattern: pattern.to_string(),
                body: Self::read(body),
            });
        }
        Self::Match { scrutinee, arms }
    }
}

/// A piece of a compound expression: fixed text or a sub-expression.
enum Piece<'a> {
    Text(String),
    Sub(&'a Expr),
}

impl Expr {
    /// No `match` or `let` anywhere: a definition holding it may print flat.
    fn is_flat(&self) -> bool {
        match self {
            Self::Flat(_) => true,
            Self::Match { .. } | Self::Let { .. } => false,
            Self::If {
                condition,
                then,
                otherwise,
            } => condition.is_flat() && then.is_flat() && otherwise.is_flat(),
        }
    }

    /// The whole expression on one line.
    pub(crate) fn flat(&self) -> String {
        match self {
            Self::Flat(text) => text.clone(),
            Self::Match { scrutinee, arms } => {
                let mut out = format!("(match {scrutinee}");
                for arm in arms {
                    out.push_str(&format!(" | {} => {}", arm.pattern, arm.body.flat()));
                }
                out.push(')');
                out
            }
            _ => self
                .pieces()
                .iter()
                .map(|p| match p {
                    Piece::Text(t) => t.clone(),
                    Piece::Sub(e) => e.flat(),
                })
                .collect(),
        }
    }

    fn pieces(&self) -> Vec<Piece<'_>> {
        match self {
            Self::If {
                condition,
                then,
                otherwise,
            } => vec![
                Piece::Text("(if ".into()),
                Piece::Sub(condition),
                Piece::Text(" then ".into()),
                Piece::Sub(then),
                Piece::Text(" else ".into()),
                Piece::Sub(otherwise),
                Piece::Text(")".into()),
            ],
            Self::Let {
                name,
                ty,
                value,
                body,
            } => vec![
                Piece::Text(format!("(let {name} : {ty} := ")),
                Piece::Sub(value),
                Piece::Text("; ".into()),
                Piece::Sub(body),
                Piece::Text(")".into()),
            ],
            _ => Vec::new(),
        }
    }

    /// The text of the expression at nesting `depth`, starting at column
    /// `col` with `trail` characters after it on its last line. A `match`
    /// stays on one line when that line fits the width; otherwise each arm
    /// starts a line indented by `4 + 2 * depth`.
    pub(crate) fn render_at(&self, depth: usize, col: usize, trail: usize) -> String {
        let flat = self.flat();
        if col + flat.len() + trail <= WIDTH {
            return flat;
        }
        match self {
            Self::Flat(text) => text.clone(),
            Self::Match { scrutinee, arms } => {
                let indent = 4 + 2 * depth;
                let mut out = format!("(match {scrutinee}");
                for (i, arm) in arms.iter().enumerate() {
                    let head = format!("{}| {} => ", " ".repeat(indent), arm.pattern);
                    let last = if i + 1 == arms.len() { trail + 1 } else { 0 };
                    out.push('\n');
                    out.push_str(&head);
                    out.push_str(&arm.body.render_at(depth + 1, head.len(), last));
                }
                out.push(')');
                out
            }
            _ => {
                let pieces = self.pieces();
                let mut out = String::new();
                for (i, piece) in pieces.iter().enumerate() {
                    match piece {
                        Piece::Text(t) => out.push_str(t),
                        Piece::Sub(e) => {
                            let after: usize = pieces[i + 1..]
                                .iter()
                                .map(|p| match p {
                                    Piece::Text(t) => t.len(),
                                    Piece::Sub(e) => e.flat().len(),
                                })
                                .sum();
                            let at = out.rsplit('\n').next().map_or(0, str::len);
                            let column = if out.contains('\n') { at } else { col + at };
                            out.push_str(&e.render_at(depth, column, after + trail));
                        }
                    }
                }
                out
            }
        }
    }
}

/// One top-level declaration.
#[derive(Clone, Debug)]
pub(crate) enum Decl {
    Structure {
        name: String,
        type_parameters: Vec<String>,
        fields: Vec<(String, String)>,
    },
    Inductive {
        name: String,
        type_parameters: Vec<String>,
        constructors: Vec<String>,
    },
    /// An inductive whose constructors carry fields: `(constructor, field
    /// types)`.
    Data {
        name: String,
        type_parameters: Vec<String>,
        constructors: Vec<(String, Vec<String>)>,
        /// The label of the mutual group the inductive is declared in.
        mutual: Option<String>,
    },
    Definition {
        name: String,
        /// Its type parameters, each a LexLean name its types write `'T`.
        type_parameters: Vec<String>,
        parameters: Vec<(String, String)>,
        result: String,
        body: Expr,
        /// The Lean axioms the definition's elaboration depends on.
        axioms: Vec<String>,
        /// The parameter a structurally recursive definition descends on.
        recursive: Option<String>,
        /// A well-founded definition's measure and, for each recursive call
        /// in pre-order, the theorem stating that the call decreases it.
        termination: Option<(String, Vec<String>)>,
        /// The label of the mutual group the definition is declared in.
        mutual: Option<String>,
    },
    Theorem {
        name: String,
        type_parameters: Vec<String>,
        parameters: Vec<(String, String)>,
        statement: String,
        proof: String,
        axioms: Vec<String>,
    },
}

impl Decl {
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Structure { name, .. }
            | Self::Inductive { name, .. }
            | Self::Data { name, .. }
            | Self::Definition { name, .. }
            | Self::Theorem { name, .. } => name,
        }
    }

    /// A record definition with no parameters.
    pub(crate) fn value(name: impl Into<String>, result: impl Into<String>, body: String) -> Self {
        Self::Definition {
            type_parameters: Vec::new(),
            name: name.into(),
            parameters: Vec::new(),
            result: result.into(),
            body: Expr::Flat(body),
            axioms: Vec::new(),
            recursive: None,
            termination: None,
            mutual: None,
        }
    }
}

/// Tokens of the emitter's own notation. Quoted content is not a reference or
/// a type variable; offsets permit binding without touching literal bytes.
fn binding_words(text: &str) -> Vec<(usize, usize, &str)> {
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < bytes.len() {
        if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() {
                match bytes[i] {
                    b'\\' => i += 2,
                    b'"' => {
                        i += 1;
                        break;
                    }
                    _ => i += 1,
                }
            }
        } else if bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'_' | b'\'') {
            let start = i;
            i += 1;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'_' | b'.' | b':'))
            {
                i += 1;
            }
            out.push((start, i, &text[start..i]));
        } else {
            i += 1;
        }
    }
    out
}

/// Bind generated compiler declarations by the type variables they and their
/// dependencies actually use. The SDK still checks every resulting type,
/// recursive call and proof; this pass introduces no Rust operation model.
pub(crate) fn bind_parameters(
    decls: &mut [Decl],
    aux: &mut std::collections::BTreeMap<String, Decl>,
    outer: &[String],
    main: &str,
) {
    use std::collections::{BTreeMap, BTreeSet};
    if outer.is_empty() {
        return;
    }
    let all: Vec<&Decl> = aux.values().chain(decls.iter()).collect();
    let allowed: BTreeSet<_> = outer.iter().cloned().collect();
    let texts: BTreeMap<_, _> = all
        .iter()
        .map(|d| (d.name().to_string(), render_decl(d)))
        .collect();
    let mut used: BTreeMap<String, BTreeSet<String>> = texts
        .iter()
        .map(|(n, text)| {
            let mut vars: BTreeSet<_> = binding_words(text)
                .iter()
                .filter_map(|(_, _, word)| {
                    word.strip_prefix('\'')
                        .filter(|v| allowed.contains(*v))
                        .map(str::to_string)
                })
                .collect();
            if n == main {
                vars.extend(outer.iter().cloned());
            }
            (n.clone(), vars)
        })
        .collect();
    let groups: BTreeMap<_, _> = all
        .iter()
        .filter_map(|d| match d {
            Decl::Data {
                name,
                mutual: Some(g),
                ..
            }
            | Decl::Definition {
                name,
                mutual: Some(g),
                ..
            } => Some((name.clone(), g.clone())),
            _ => None,
        })
        .collect();
    loop {
        let old = used.clone();
        for (n, text) in &texts {
            for (_, _, word) in binding_words(text) {
                let owner = word.split_once('.').map_or(word, |(n, _)| n);
                if let Some(vars) = old.get(owner) {
                    used.get_mut(n)
                        .expect("emitted declaration")
                        .extend(vars.iter().cloned());
                }
            }
            if let Some(group) = groups.get(n) {
                for (member, other) in &groups {
                    if other == group {
                        used.get_mut(n)
                            .expect("emitted declaration")
                            .extend(old[member].iter().cloned());
                    }
                }
            }
        }
        if used == old {
            break;
        }
    }
    let bindings: BTreeMap<String, Vec<String>> = used
        .into_iter()
        .map(|(n, vars)| {
            (
                n,
                outer
                    .iter()
                    .filter(|p| vars.contains(*p))
                    .cloned()
                    .collect(),
            )
        })
        .collect();
    let data: BTreeSet<_> = all
        .iter()
        .filter(|d| {
            matches!(
                d,
                Decl::Structure { .. } | Decl::Inductive { .. } | Decl::Data { .. }
            )
        })
        .map(|d| d.name().to_string())
        .collect();
    let rewrite = |text: &str, ty: bool| {
        let words = binding_words(text);
        let mut out = String::new();
        let mut at = 0;
        for (index, (start, end, word)) in words.iter().enumerate() {
            out.push_str(&text[at..*start]);
            let owner = word.split_once('.').map_or(*word, |(n, _)| n);
            let params = bindings.get(owner).filter(|p| !p.is_empty());
            let previous = index.checked_sub(1).map(|i| words[i].2);
            let after = text[*end..].trim_start();
            let head = text[..*start].trim_end().ends_with('(');
            let field = after.starts_with(":=");
            if let Some(params) = params.filter(|_| !field && !after.starts_with('[')) {
                let arguments = params.iter().map(|p| format!("'{p}")).collect::<Vec<_>>();
                if data.contains(owner)
                    && owner == *word
                    && (ty || !matches!(previous, Some("record")))
                {
                    out.push_str(&format!("({word} {})", arguments.join(" ")));
                } else if !ty
                    && (matches!(previous, Some("ctor" | "record" | "ref"))
                        || (head && !data.contains(owner)))
                {
                    out.push_str(&format!("{word} [{}]", arguments.join(", ")));
                } else {
                    out.push_str(word);
                }
            } else {
                out.push_str(word);
            }
            at = *end;
        }
        out.push_str(&text[at..]);
        out
    };
    for decl in aux.values_mut().chain(decls.iter_mut()) {
        let name = decl.name().to_string();
        let (parameters, type_parameters) = match decl {
            Decl::Structure {
                type_parameters,
                fields,
                ..
            } => {
                for (_, ty) in fields {
                    *ty = rewrite(ty, true);
                }
                (None, type_parameters)
            }
            Decl::Inductive {
                type_parameters, ..
            } => (None, type_parameters),
            Decl::Data {
                type_parameters,
                constructors,
                ..
            } => {
                for (_, fields) in constructors {
                    for ty in fields {
                        *ty = rewrite(ty, true);
                    }
                }
                (None, type_parameters)
            }
            Decl::Definition {
                type_parameters,
                parameters,
                result,
                body,
                termination,
                ..
            } => {
                *result = rewrite(result, true);
                *body = Expr::read(&rewrite(&body.flat(), false));
                if let Some((measure, _)) = termination {
                    *measure = rewrite(measure, false);
                }
                (Some(parameters), type_parameters)
            }
            Decl::Theorem {
                type_parameters,
                parameters,
                statement,
                ..
            } => {
                *statement = rewrite(statement, false);
                (Some(parameters), type_parameters)
            }
        };
        if let Some(parameters) = parameters {
            for (_, ty) in parameters {
                *ty = rewrite(ty, true);
            }
        }
        for param in &bindings[&name] {
            if !type_parameters.contains(param) {
                type_parameters.push(param.clone());
            }
        }
    }
}

/// `[a, b]` flat, or one item per line under `indent`.
fn list(key: &str, items: &[String], indent: usize, comma: bool) -> String {
    let flat = format!("{}{key}: [{}]", " ".repeat(indent), items.join(", "));
    if flat.len() + usize::from(comma) <= WIDTH || items.is_empty() {
        return flat;
    }
    let inner = " ".repeat(indent + 2);
    let mut out = format!("{}{key}: [\n", " ".repeat(indent));
    out.push_str(
        &items
            .iter()
            .map(|item| format!("{inner}{item}"))
            .collect::<Vec<_>>()
            .join(",\n"),
    );
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push(']');
    out
}

/// The members of a declaration, flat when the whole line fits.
fn members(
    kind: &str,
    flat_members: &[String],
    broken_members: &[String],
    flat_ok: bool,
) -> String {
    let flat = format!("{kind}{{{}}}", flat_members.join(", "));
    if flat_ok && flat.len() <= WIDTH {
        return flat;
    }
    format!("{kind}{{\n{}\n}}", broken_members.join(",\n"))
}

fn params(parameters: &[(String, String)]) -> Vec<String> {
    parameters
        .iter()
        .map(|(name, ty)| format!("{name} : {ty}"))
        .collect()
}

pub(crate) fn render_decl(decl: &Decl) -> String {
    match decl {
        Decl::Structure {
            name,
            type_parameters,
            fields,
        } => {
            let items: Vec<String> = fields
                .iter()
                .map(|(field, ty)| format!("{{name: {field}, type: {ty}}}"))
                .collect();
            let lines: Vec<String> = fields
                .iter()
                .zip(&items)
                .enumerate()
                .map(|(i, ((field, ty), item))| {
                    if 4 + item.len() + usize::from(i + 1 < items.len()) <= WIDTH {
                        item.clone()
                    } else {
                        format!("{{\n      name: {field},\n      type: {ty}\n    }}")
                    }
                })
                .collect();
            let head = [
                format!("name: {name}"),
                format!("type_parameters: [{}]", type_parameters.join(", ")),
                "parameters: []".to_string(),
            ];
            let mut flat = head.to_vec();
            flat.push(format!("fields: [{}]", items.join(", ")));
            let mut broken: Vec<String> = head.iter().map(|m| format!("  {m}")).collect();
            broken.push(list("fields", &lines, 2, false));
            members("structure", &flat, &broken, true)
        }
        Decl::Inductive {
            name,
            type_parameters,
            constructors,
        } => {
            let items: Vec<String> = constructors
                .iter()
                .map(|ctor| format!("{{name: {ctor}, fields: []}}"))
                .collect();
            let head = [
                format!("name: {name}"),
                format!("type_parameters: [{}]", type_parameters.join(", ")),
                "parameters: []".to_string(),
            ];
            let mut flat = head.to_vec();
            flat.push(format!("constructors: [{}]", items.join(", ")));
            let mut broken: Vec<String> = head.iter().map(|m| format!("  {m}")).collect();
            broken.push(list("constructors", &items, 2, false));
            members("inductive", &flat, &broken, true)
        }
        Decl::Data {
            name,
            type_parameters,
            constructors,
            mutual,
        } => {
            let items: Vec<String> = constructors
                .iter()
                .map(|(ctor, fields)| format!("{{name: {ctor}, fields: [{}]}}", fields.join(", ")))
                .collect();
            // A constructor too long for its line is broken one member per
            // line, its fields one per line.
            let lines: Vec<String> = constructors
                .iter()
                .zip(&items)
                .enumerate()
                .map(|(i, ((ctor, fields), item))| {
                    let comma = i + 1 < items.len();
                    if 4 + item.len() + usize::from(comma) <= WIDTH {
                        item.clone()
                    } else {
                        format!(
                            "{{\n      name: {ctor},\n{}\n    }}",
                            list("fields", fields, 6, false)
                        )
                    }
                })
                .collect();
            let head = [
                format!("name: {name}"),
                format!("type_parameters: [{}]", type_parameters.join(", ")),
                "parameters: []".to_string(),
            ];
            let mut flat = head.to_vec();
            flat.push(format!("constructors: [{}]", items.join(", ")));
            let mut broken: Vec<String> = head.iter().map(|m| format!("  {m}")).collect();
            broken.push(list("constructors", &lines, 2, mutual.is_some()));
            if let Some(label) = mutual {
                flat.push(format!("mutual: {label}"));
                broken.push(format!("  mutual: {label}"));
            }
            members("inductive", &flat, &broken, true)
        }
        Decl::Definition {
            name,
            type_parameters,
            parameters,
            result,
            body,
            axioms,
            recursive,
            termination,
            mutual,
        } => {
            // A body held as text whose term is a match is laid out as one,
            // as `lexlean fmt` lays it out, whichever writer held it as text.
            let read;
            let body = match body {
                Expr::Flat(text) => {
                    read = Expr::read(text);
                    &read
                }
                other => other,
            };
            let ps = params(parameters);
            // A definition of no type parameters writes none, as `lexlean
            // fmt` leaves it.
            let generic = (!type_parameters.is_empty())
                .then(|| format!("type_parameters: [{}]", type_parameters.join(", ")));
            let mut flat = vec![format!("name: {name}")];
            flat.extend(generic.clone());
            flat.extend([
                format!("parameters: [{}]", ps.join(", ")),
                format!("result: {result}"),
                format!("body: {}", body.flat()),
            ]);
            let more = recursive.is_some() || termination.is_some() || mutual.is_some();
            let trail = usize::from(more || !axioms.is_empty());
            let mut broken = vec![format!("  name: {name}")];
            broken.extend(generic.map(|g| format!("  {g}")));
            broken.extend([
                list("parameters", &ps, 2, true),
                format!("  result: {result}"),
                format!("  body: {}", body.render_at(0, 8, trail)),
            ]);
            if !axioms.is_empty() {
                flat.push(format!("axioms: [{}]", axioms.join(", ")));
                broken.push(list("axioms", axioms, 2, more));
            }
            if let Some(label) = mutual {
                flat.push(format!("mutual: {label}"));
                broken.push(format!("  mutual: {label}"));
            }
            if let Some(arg) = recursive {
                flat.push(format!("recursive_argument: {arg}"));
                broken.push(format!("  recursive_argument: {arg}"));
            }
            if let Some((measure, evidence)) = termination {
                let clause = format!(
                    "termination: {{evidence: [{}], measure: {measure}}}",
                    evidence.join(", ")
                );
                flat.push(clause.clone());
                if 2 + clause.len() <= WIDTH {
                    broken.push(format!("  {clause}"));
                } else {
                    broken.push(format!(
                        "  termination: {{\n{},\n    measure: {measure}\n  }}",
                        list("evidence", evidence, 4, true)
                    ));
                }
            }
            members("definition", &flat, &broken, body.is_flat() && !more)
        }
        Decl::Theorem {
            name,
            type_parameters,
            parameters,
            statement,
            proof,
            axioms,
        } => {
            let ps = params(parameters);
            let mut broken_proof = format!("  proof: {proof}");
            if broken_proof.len() + usize::from(!axioms.is_empty()) > WIDTH {
                if let Some(definitions) = proof
                    .strip_prefix("linear_arithmetic{definitions: [")
                    .and_then(|text| text.strip_suffix("]}"))
                {
                    let definitions: Vec<String> =
                        definitions.split(", ").map(str::to_string).collect();
                    broken_proof = format!(
                        "  proof: linear_arithmetic{{\n{}\n  }}",
                        list("definitions", &definitions, 4, false)
                    );
                }
            }
            let mut flat = vec![
                format!("name: {name}"),
                format!("parameters: [{}]", ps.join(", ")),
                format!("statement: {statement}"),
                format!("proof: {proof}"),
            ];
            let mut broken = vec![
                format!("  name: {name}"),
                list("parameters", &ps, 2, true),
                format!("  statement: {statement}"),
                broken_proof,
            ];
            if !type_parameters.is_empty() {
                flat.insert(
                    1,
                    format!("type_parameters: [{}]", type_parameters.join(", ")),
                );
                broken.insert(
                    1,
                    format!("  type_parameters: [{}]", type_parameters.join(", ")),
                );
            }
            if !axioms.is_empty() {
                flat.push(format!("axioms: [{}]", axioms.join(", ")));
                broken.push(list("axioms", axioms, 2, false));
            }
            members("theorem", &flat, &broken, true)
        }
    }
}

/// A whole module.
#[derive(Clone, Debug)]
pub(crate) struct Module {
    pub(crate) name: String,
    pub(crate) glossaries: Vec<&'static str>,
    pub(crate) imports: Vec<String>,
    pub(crate) title: &'static str,
    pub(crate) decls: Vec<Decl>,
}

impl Module {
    pub(crate) fn render(&self) -> String {
        let mut out = format!("\\begin{{lexlean}}{{{}}}\n", self.name);
        for glossary in &self.glossaries {
            out.push_str(&format!("\\useglossary{{{glossary}}}\n"));
        }
        for import in &self.imports {
            out.push_str(&format!("\\importmodule{{{import}}}\n"));
        }
        out.push_str(&format!("\\title{{{}}}\n", self.title));
        out.push_str("\\begin{semanticmodule}\n");
        let decls: Vec<String> = self.decls.iter().map(render_decl).collect();
        out.push_str(&decls.join("\n\n"));
        if !decls.is_empty() {
            out.push('\n');
        }
        out.push_str("\\end{semanticmodule}\n\\end{lexlean}\n");
        out
    }

    /// The path of the module's source under `src/`.
    pub(crate) fn path(&self) -> String {
        format!("src/{}.lex.tex", self.name.replace('.', "/"))
    }
}

/// A LexLean string literal. Control characters become spaces and runs of
/// whitespace collapse, because a record name or text is one bounded line.
pub(crate) fn string(text: &str) -> String {
    format!("\"{}\"", escape(text))
}

pub(crate) fn escape(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = String::with_capacity(collapsed.len());
    for c in collapsed.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

/// The list of the strings `items`: `(cons "a" (cons "b" (nil String)))`,
/// and a long one as balanced halves ([`crate::embed::term_list`]), since a
/// feature lists every call and acceptance row of its module and the linker
/// reads a term to a bounded depth.
pub(crate) fn string_list(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|item| string(item)).collect();
    crate::embed::term_list(&quoted, "String")
}

/// `lowerCamel` from any Rust or Cargo name: `thermo-core::should_heat` ->
/// `thermoCoreShouldHeat`.
pub(crate) fn camel(parts: &[&str]) -> String {
    pascal(parts)
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_lowercase() } else { c })
        .collect::<String>()
}

/// `PascalCase` from any Rust or Cargo name.
pub(crate) fn pascal(parts: &[&str]) -> String {
    let mut out = String::new();
    for part in parts {
        let mut upper = true;
        let mut prev_lower = false;
        for c in part.chars() {
            if !c.is_ascii_alphanumeric() {
                upper = true;
                prev_lower = false;
                continue;
            }
            if upper {
                out.push(c.to_ascii_uppercase());
                upper = false;
            } else if c.is_ascii_uppercase() && !prev_lower {
                out.push(c.to_ascii_lowercase());
            } else {
                out.push(c);
            }
            prev_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
        }
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        format!("N{out}")
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_binding_follows_dependencies_and_ignores_literals_ue_67() {
        use std::collections::BTreeMap;
        let helper = Decl::Definition {
            name: "helper".into(),
            type_parameters: Vec::new(),
            parameters: vec![("x".into(), "'T".into())],
            result: "'T".into(),
            body: Expr::Flat("x".into()),
            axioms: Vec::new(),
            recursive: None,
            termination: None,
            mutual: None,
        };
        let mut aux = BTreeMap::from([("helper".into(), helper)]);
        let literal = string("'T (helper x)");
        let mut main = aux["helper"].clone();
        if let Decl::Definition { name, body, .. } = &mut main {
            *name = "main".into();
            *body = Expr::Flat("(helper x)".into());
        }
        let mut decls = vec![
            main,
            Decl::value("literal", "String", literal.clone()),
            Decl::value("independent", "Bool", "true".into()),
        ];
        bind_parameters(&mut decls, &mut aux, &["T".into()], "main");
        let Decl::Definition {
            type_parameters, ..
        } = &aux["helper"]
        else {
            panic!("helper")
        };
        assert_eq!(type_parameters, &["T"]);
        let Decl::Definition {
            type_parameters,
            body,
            ..
        } = &decls[0]
        else {
            panic!("main")
        };
        assert_eq!(type_parameters, &["T"]);
        assert_eq!(body.flat(), "(helper ['T] x)");
        let Decl::Definition {
            type_parameters,
            body,
            ..
        } = &decls[1]
        else {
            panic!("literal")
        };
        assert!(type_parameters.is_empty());
        assert_eq!(body.flat(), literal);
        let Decl::Definition {
            type_parameters, ..
        } = &decls[2]
        else {
            panic!("independent")
        };
        assert!(type_parameters.is_empty());
    }

    #[test]
    fn observed_policy_edits_preserve_statements_proofs_and_literals_ue_84() {
        let text = r#"\begin{semanticmodule}
inductive{name: Payload, constructors: [{name: Item, fields: []}]}
definition{name: textValue, parameters: [], result: String, body: "axioms: [Quoted]", axioms: [Old]}
definition{name: flag, parameters: [], result: Bool, body: true}
theorem{name: valid, parameters: [], statement: (true = true), proof: reflexivity{}, axioms: []}
\end{semanticmodule}"#;
        let policies = std::collections::BTreeMap::from([
            (
                "textValue".into(),
                vec!["Quot.sound".into(), "propext".into()],
            ),
            ("flag".into(), vec![]),
            ("valid".into(), vec!["Classical.choice".into()]),
        ]);
        let expected = text
            .replace("axioms: [Old]", "axioms: [Quot.sound, propext]")
            .replace("body: true}", "body: true, axioms: []}")
            .replace(
                "reflexivity{}, axioms: []",
                "reflexivity{}, axioms: [Classical.choice]",
            );
        let patched = with_axiom_policies(text, &policies).unwrap();
        assert_eq!(patched, expected);
        assert_eq!(with_axiom_policies(&patched, &policies).unwrap(), patched);
        for (name, axiom) in [
            ("missing", "propext"),
            ("Payload", "propext"),
            ("flag", "x], body: false"),
        ] {
            assert!(with_axiom_policies(
                text,
                &std::collections::BTreeMap::from([(name.into(), vec![axiom.into()])])
            )
            .is_err());
        }
        assert!(with_axiom_policies(
            &text.replace("axioms: [Old]", "axioms: [], axioms: [Old]"),
            &policies
        )
        .is_err());
    }

    #[test]
    fn type_authority_excludes_terms_nested_names_and_literals_ue_82() {
        let text = r#"\begin{semanticmodule}
inductive{name: Payload, constructors: [{name: Nested, fields: []}]}
structure{name: Frame, fields: [value : Payload]}
definition{name: Term, parameters: [], result: String, body: "inductive{name: Forged}"}
theorem{name: Claim, parameters: [], statement: true, proof: decide{}}
\end{semanticmodule}"#;
        assert_eq!(
            type_names(text).unwrap(),
            ["Frame".into(), "Payload".into()].into_iter().collect()
        );
        assert!(type_names(&text.replace("name: Term,", "name: Payload,")).is_err());
        assert!(type_names(&text.replace("structure{", "opaque{")).is_err());
    }

    const PROOF_MODEL: &str = r#"\begin{semanticmodule}
inductive{name: Shape, constructors: [{name: Item, fields: []}]}
definition{name: leaf, result: Shape, body: Shape.Item, mutual: pair}
definition{name: sibling, result: Nat, body: 0, mutual: pair}
definition{name: quoted, result: String, body: "é🦀 } unrelated \" { mutual: fake"}
definition{name: root, result: Shape, body: (let note := quoted; leaf)}
definition{name: unrelated, result: Bool, body: false}
theorem{name: obligation, statement: (root = root), proof: reflexivity}
\end{semanticmodule}"#;

    #[test]
    fn native_roots_share_one_immutable_dependency_index_ue_109() {
        SYNTAX_PARSES.with(|count| count.set(0));
        let graph = ProofClosureGraph::new(PROOF_MODEL).unwrap();
        let first = graph.closure(&["obligation".into()]).unwrap();
        let second = graph.closure(&["root".into()]).unwrap();
        assert_eq!(
            SYNTAX_PARSES.with(std::cell::Cell::get),
            1,
            "root queries reparse the complete model"
        );
        assert!(first.contains("name: obligation,"));
        assert!(!second.contains("name: obligation,"));
        for name in ["Shape", "leaf", "sibling", "quoted", "root"] {
            assert!(first.contains(&format!("name: {name},")));
            assert!(second.contains(&format!("name: {name},")));
        }
        assert!(graph.closure(&[]).is_err());
        assert!(graph.closure(&["missing".into()]).is_err());
        assert!(ProofClosureGraph::new(&PROOF_MODEL.replace("inductive{", "opaque{")).is_err());
        let changed = PROOF_MODEL.replace("body: 0,", "body: 1,");
        let other = ProofClosureGraph::new(&changed)
            .unwrap()
            .closure(&["root".into()])
            .unwrap();
        assert_ne!(second, other);
    }

    #[test]
    fn proof_closure_keeps_transitive_types_and_mutual_groups_ue_53() {
        let text = PROOF_MODEL;
        let kept = proof_closure(text, &["obligation".into()]).expect("closure");
        for name in ["Shape", "leaf", "sibling", "quoted", "root", "obligation"] {
            assert!(kept.contains(&format!("name: {name},")), "{name}");
        }
        assert!(!kept.contains("definition{name: unrelated,"));
        assert!(proof_closure(text, &["missing".into()]).is_err());
        assert!(proof_closure(&text.replace("inductive{", "opaque{"), &["root".into()]).is_err());
        assert!(proof_closure(text, &[]).is_err());
    }

    #[test]
    fn names_are_camel_cased_from_rust_and_cargo_names() {
        assert_eq!(
            camel(&["thermo-core", "should_heat"]),
            "thermoCoreShouldHeat"
        );
        assert_eq!(pascal(&["UorustError"]), "UorustError");
        assert_eq!(pascal(&["HTTPServer"]), "Httpserver");
        assert_eq!(pascal(&["2d"]), "N2d");
    }

    #[test]
    fn strings_are_escaped_and_collapsed() {
        assert_eq!(string("a \"b\"\n\tc\\"), "\"a \\\"b\\\" c\\\\\"");
    }

    #[test]
    fn a_match_held_as_text_is_read_into_its_arms() {
        let text = "(match (prim index xs i : (Option T)) | Option.none => (record T {a := \"|\"}) | Option.some x => (match x | T.mk y => y))";
        let Expr::Match { scrutinee, arms } = Expr::read(text) else {
            panic!("not read as a match");
        };
        assert_eq!(scrutinee, "(prim index xs i : (Option T))");
        assert_eq!(arms.len(), 2);
        assert_eq!(arms[0].pattern, "Option.none");
        assert_eq!(arms[0].body, Expr::Flat("(record T {a := \"|\"})".into()));
        assert!(matches!(arms[1].body, Expr::Match { .. }));
        assert_eq!(Expr::read(text).flat(), text);
        assert_eq!(Expr::read("(f x)"), Expr::Flat("(f x)".into()));
    }

    #[test]
    fn a_short_definition_is_flat_and_a_long_one_breaks() {
        let short = Decl::value("callA", "String", string("a"));
        assert_eq!(
            render_decl(&short),
            "definition{name: callA, parameters: [], result: String, body: \"a\"}"
        );
        let long = Decl::value("callA", "String", string(&"x".repeat(80)));
        assert!(render_decl(&long).starts_with("definition{\n  name: callA,\n"));
    }

    // residue: `camel` and `pascal` rewritten inside the fragment the tool
    // lifts from itself; this pins what the old bodies did.

    #[test]
    fn residue_names_keep_their_case_rules_at_the_edges() {
        assert_eq!(camel(&[]), "");
        assert_eq!(camel(&["--", "_"]), "");
        assert_eq!(pascal(&[]), "");
        assert_eq!(pascal(&["--"]), "");
        // The first character only is lowered, after the digit's prefix.
        assert_eq!(camel(&["2d"]), "n2d");
        assert_eq!(camel(&["x"]), "x");
        assert_eq!(camel(&["Ab", "Cd"]), "abCd");
        assert_eq!(camel(&["r#type"]), "rType");
        // A digit after the first part is no prefix.
        assert_eq!(pascal(&["a", "2"]), "A2");
        assert_eq!(pascal(&["9", "a"]), "N9A");
        assert_eq!(camel(&["9", "a"]), "n9A");
        // Non-ASCII characters are separators.
        assert_eq!(pascal(&["é9"]), "N9");
    }
}
