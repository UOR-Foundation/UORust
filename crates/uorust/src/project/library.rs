//! The native-library route (UR-R29): one library project per library
//! package of the workspace, written beside the workspace project, each
//! holding only the closure of the roots it admits.
//!
//! PrismPM's native-library profile (SPEC 12.6) is one closed
//! `NativeLibrary` value per project, and it cannot coexist with an
//! application or a system declaration, so a workspace with several library
//! packages has several projects. Each holds a closure of its own because
//! `prismpm verify` audits every theorem of the selected snapshot (a theorem
//! with a nonempty axiom audit refuses the whole library) and the exporter
//! names every definition of its roots' closure by its last component (a
//! collision or a Rust keyword refuses the whole library). A root that would
//! refuse its library is refused alone, with its cause, so that the rest of
//! the library is still built.

use std::collections::{BTreeMap, BTreeSet};

use super::{qualified, words, Notes, BEHAVIOR, BOOL, NAT};
use crate::admit::Refusal;
use crate::certify::{higher_order, Definitions};
use crate::facts::{Package, TargetKind, Workspace};
use crate::ingest::summary;
use crate::lex::{pascal, string, Decl, Module};
use crate::lift::{PRELUDE, PRELUDE_SOURCE};

/// The native-library profile's bound on each root list (SPEC 12.6: 1..1024
/// roots), which PrismPM's projection enforces.
pub(super) const LIBRARY_ROOTS: usize = 1024;

/// The SDK module that declares the native-library profile's type.
pub(super) const LIBRARY_MODEL: &str = "Foundation.Library.V1.Model";

/// The directory of the projection the library projects are written under,
/// one directory per library package, named by its Cargo name.
pub(super) const LIBRARIES: &str = "library";

/// The function names the pinned code generator emits literally and so
/// refuses as Rust identifiers (PrismPM `library_build.rs`,
/// `FUNCTION_KEYWORDS`): Rust's strict, reserved and weak keywords.
const FUNCTION_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
];

/// The primitives LexLean lowers to its collections runtime
/// (`LexLeanCollections`, LexLean 0.4 `backend/semantic.rs`, the
/// `Spelling::Collection` rows of `spelling`), by their surface names (the
/// snake-case names of `SemanticPrimitive`): the pinned exporter leaves each
/// such call an unresolved external call and refuses the whole library.
const COLLECTION_PRIMITIVES: &[&str] = &[
    "map_insert",
    "map_remove",
    "map_lookup",
    "map_contains",
    "map_size",
    "map_keys",
    "map_values",
    "map_entries",
    "map_fold",
    "set_insert",
    "set_remove",
    "set_contains",
    "set_size",
    "set_elements",
    "set_union",
    "set_intersection",
    "set_difference",
    "set_fold",
    "list_fold",
    "iterate",
    "iterate_until",
    "graph_successors",
    "graph_reachable",
    "graph_topological",
    "less_than",
];

/// One library package's project: its Cargo name, the closure of its
/// admitted roots (the workspace's Behavior declarations it needs, under the
/// same names, so a root stays `{Prefix}.Behavior.{name}`) and its one
/// `NativeLibrary` value.
pub(super) struct Library {
    pub(super) package: String,
    pub(super) behavior: Module,
    pub(super) value: Module,
}

/// The roots of one library that the exporter can take, the closure they
/// reach, and every other root with the cause that refuses it.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Admission {
    /// Admitted roots, by their Behavior name.
    pub(super) roots: BTreeSet<String>,
    /// Refused roots, by their Behavior name, with their causes.
    pub(super) refused: BTreeMap<String, String>,
    /// Every definition the admitted roots reach, qualified by module
    /// (`Behavior::name`, `Prelude::name`).
    pub(super) closure: BTreeSet<String>,
    /// Every type of Behavior the admitted roots reach, by name.
    pub(super) types: BTreeSet<String>,
}

/// The field types of a type declaration, as one text.
fn fields(decl: &Decl) -> String {
    match decl {
        Decl::Structure { fields, .. } => fields
            .iter()
            .map(|(_, ty)| ty.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        Decl::Data { constructors, .. } => constructors
            .iter()
            .flat_map(|(_, fields)| fields.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    }
}

/// The types of Behavior that `texts` name, and every type their fields
/// name in turn: each word of a text that is the name of a type
/// declaration (over-approximate where a field or binder is spelled like a
/// type, never short of one).
fn types_of<'a>(
    texts: impl IntoIterator<Item = &'a str>,
    types: &BTreeMap<String, (&Decl, String)>,
) -> BTreeSet<String> {
    let mut reached: BTreeSet<String> = BTreeSet::new();
    let mut work: Vec<String> = texts
        .into_iter()
        .flat_map(words)
        .filter(|w| types.contains_key(*w))
        .map(str::to_string)
        .collect();
    while let Some(ty) = work.pop() {
        if !reached.insert(ty.clone()) {
            continue;
        }
        if let Some((_, text)) = types.get(&ty) {
            work.extend(
                words(text)
                    .filter(|w| types.contains_key(*w) && !reached.contains(*w))
                    .map(str::to_string),
            );
        }
    }
    reached
}

/// Admit each of `candidates` (Behavior definitions) as a root of one
/// native library, in order, from what PrismPM's library build and
/// verification refuse a whole library for:
///
/// - a definition that takes a function: an export is an executable
///   definition (SPEC 12.6), and an executable definition takes no function
///   (LexLean SPEC 17.13);
/// - a definition of a mutual recursion group: the pinned exporter compiles
///   its recursor to a local function it does not support and fails the
///   whole build without naming a root (the `[[gap]]` row of
///   `upstream.toml` for UOR-Foundation/PrismPM);
/// - a type of a mutual inductive block: the pinned code generator renders
///   no Rust type for one (PP4102);
/// - a type defined in terms of itself through its fields, or Lean's
///   unbounded `Int`: the pinned code generator renders neither under the
///   library's allocation-free profile (PP4102);
/// - a definition that holds a list as an intermediate value: the pinned
///   code generator's allocation-free lowering stores a list only as a
///   borrowed parameter or in the output buffer (PP4102);
/// - a definition with a parameter whose result is a list of elements that
///   are not `Copy`: the pinned code generator writes them into the output
///   buffer from borrowed elements and the generated crate does not compile
///   (the third `[[gap]]` row for UOR-Foundation/PrismPM);
/// - a definition that calls a primitive LexLean lowers to its collections
///   runtime: the pinned exporter leaves the call unresolved and fails the
///   whole build (the second `[[gap]]` row for UOR-Foundation/PrismPM);
/// - a definition whose termination evidence is a theorem with axioms: the
///   library verification refuses any theorem of the snapshot whose axiom
///   audit is nonempty, and a definition cannot be stated without its
///   evidence;
/// - a definition whose last component is no Rust identifier, is a keyword
///   the code generator emits literally, or is the last component of another
///   definition of the library's closure: the exporter names every
///   definition of the closure by it.
///
/// The last rule is over the whole library, so a root is admitted only if
/// its closure adds no last component already taken by another definition
/// the roots admitted before it reach.
pub(super) fn admit(behavior: &Module, candidates: &[String]) -> Admission {
    let texts: BTreeMap<String, String> = [
        (BEHAVIOR.to_string(), behavior.render()),
        (PRELUDE.to_string(), PRELUDE_SOURCE.to_string()),
    ]
    .into_iter()
    .collect();
    let definitions = Definitions::of(&texts);
    let decls: BTreeMap<String, &Decl> = behavior
        .decls
        .iter()
        .map(|d| (qualified(BEHAVIOR, d.name()), d))
        .collect();
    let types: BTreeMap<String, (&Decl, String)> = behavior
        .decls
        .iter()
        .filter(|d| {
            matches!(
                d,
                Decl::Structure { .. } | Decl::Inductive { .. } | Decl::Data { .. }
            )
        })
        .map(|d| (d.name().to_string(), (d, crate::lex::render_decl(d))))
        .collect();
    let mut out = Admission::default();
    let mut taken: BTreeMap<String, String> = BTreeMap::new();
    for root in candidates {
        let start = qualified(BEHAVIOR, root);
        if !matches!(decls.get(&start), Some(Decl::Definition { .. })) {
            out.refused
                .insert(root.clone(), "it is no definition of Behavior".to_string());
            continue;
        }
        let own = definitions.closure(&start);
        let reached = types_of(own.iter().filter_map(|d| definitions.text(d)), &types);
        // The code generator renders no Rust for a type of a mutual
        // inductive block, so no root may reach one.
        let mutual = reached.iter().find_map(|t| match types.get(t) {
            Some((
                Decl::Data {
                    mutual: Some(group),
                    ..
                },
                _,
            )) => Some((t, group)),
            _ => None,
        });
        // Nor for a type defined in terms of itself, through its fields'
        // types at any depth (the tier-1 memory profile it needs is not the
        // library profile's).
        let recursive = reached.iter().find(|t| {
            types
                .get(*t)
                .is_some_and(|(decl, _)| types_of([fields(decl).as_str()], &types).contains(*t))
        });
        // Nor for Lean's unbounded `Int`, which has no fixed-width Rust type.
        let unbounded = own
            .iter()
            .filter_map(|d| definitions.text(d))
            .chain(
                reached
                    .iter()
                    .filter_map(|t| types.get(t).map(|(_, x)| x.as_str())),
            )
            .any(|text| words(text).any(|w| w == "Int"));
        let cause = match (mutual, recursive) {
            (Some((ty, group)), _) => Some(format!(
                "it reaches the type {ty} of the mutual inductive block {group}, for which \
                 the pinned code generator renders no Rust type"
            )),
            (None, Some(ty)) => Some(format!(
                "it reaches the type {ty}, which is defined in terms of itself, and the \
                 pinned code generator renders a recursive type only under another memory \
                 profile than the library's"
            )),
            (None, None) if unbounded => Some(
                "it reaches the type Int, which has no fixed-width Rust type the pinned \
                 code generator renders"
                    .to_string(),
            ),
            (None, None) => refusal(&own, &definitions, &decls, &types, &taken),
        };
        match cause {
            Some(cause) => {
                out.refused.insert(root.clone(), cause);
            }
            None => {
                for d in &own {
                    taken.insert(leaf(d).to_string(), d.clone());
                }
                out.roots.insert(root.clone());
                out.closure.extend(own);
                out.types.extend(reached);
            }
        }
    }
    out
}

/// A LexLean term read as nested groups: each parenthesised group a node of
/// its parts, every other token (outside string literals) an atom.
#[derive(Debug)]
enum Term {
    Atom(String),
    Group(Vec<Term>),
}

impl Term {
    /// The term `text` reads as, or none where its parentheses do not pair.
    fn read(text: &str) -> Option<Term> {
        let mut stack: Vec<Vec<Term>> = vec![Vec::new()];
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '(' => stack.push(Vec::new()),
                ')' => {
                    let group = stack.pop()?;
                    stack.last_mut()?.push(Term::Group(group));
                }
                '"' => {
                    let mut literal = String::from('"');
                    let mut escaped = false;
                    for c in chars.by_ref() {
                        literal.push(c);
                        match c {
                            _ if escaped => escaped = false,
                            '\\' => escaped = true,
                            '"' => break,
                            _ => {}
                        }
                    }
                    stack.last_mut()?.push(Term::Atom(literal));
                }
                c if c.is_whitespace() => {}
                c => {
                    let mut atom = String::from(c);
                    while let Some(&next) = chars.peek() {
                        if next.is_whitespace() || matches!(next, '(' | ')' | '"') {
                            break;
                        }
                        atom.push(next);
                        chars.next();
                    }
                    // `;` ends a `let`'s value: a part of its own.
                    match atom.strip_suffix(';') {
                        Some(value) if !value.is_empty() => {
                            stack.last_mut()?.push(Term::Atom(value.to_string()));
                            stack.last_mut()?.push(Term::Atom(";".to_string()));
                        }
                        _ => stack.last_mut()?.push(Term::Atom(atom)),
                    }
                }
            }
        }
        let mut top = stack.pop()?;
        (stack.is_empty() && top.len() == 1).then(|| top.remove(0))
    }

    fn atom(&self) -> Option<&str> {
        match self {
            Term::Atom(a) => Some(a),
            Term::Group(_) => None,
        }
    }
}

/// Whether a definition's text holds a list the pinned code generator's
/// allocation-free lowering has no storage for. A list there is a borrowed
/// parameter, or flows into the output buffer of a definition whose result
/// is a list (prod-codegen `Error::UnsupportedList`: a list "used as an
/// intermediate value rather than flowing into the output buffer"), so a
/// list a definition constructs (`cons`, `nil`, a primitive or a call whose
/// result is a list, `lists` naming the definitions whose result is one) is
/// admitted only where it is the definition's result: its body, a branch or
/// arm of it, a `let`'s body, the tail of a `cons` or an operand of an
/// `append` there. A let-bound list is one; so is a computed list passed to a
/// call. A body that does not read as a term is one, unread.
fn intermediate_list(text: &str, lists: &BTreeSet<&str>) -> bool {
    let Some((head, body)) = text.split_once("body: ") else {
        return false;
    };
    // The body is the first whole term of what follows `body: `.
    let (mut depth, mut quoted, mut escaped, mut end) = (0usize, false, false, body.len());
    for (at, c) in body.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end = at + 1;
                    break;
                }
            }
            ',' | '}' | '\n' if !quoted && depth == 0 => {
                end = at;
                break;
            }
            _ => {}
        }
    }
    let body = &body[..end];
    let Some(term) = Term::read(body) else {
        return true;
    };
    let result = head.contains("result: (List ");
    // A list with no parameter is a static slice, whose elements are closed
    // constants: numerals, Booleans and constructors of them.
    if result && head.contains("parameters: []") {
        return !constant_list(&term);
    }
    !stored(&term, result, lists)
}

/// Whether a term is a list of closed allocation-free constants (prod-codegen
/// `static_constant`: a numeral, a Boolean, a constructor of such).
fn constant_list(term: &Term) -> bool {
    fn constant(term: &Term) -> bool {
        match term {
            Term::Atom(a) => a == "true" || a == "false" || a.chars().all(|c| c.is_ascii_digit()),
            Term::Group(parts) => match parts.first().and_then(Term::atom) {
                Some("ctor") => parts[1..]
                    .iter()
                    .skip(1)
                    .filter(|p| !matches!(p, Term::Atom(a) if a.starts_with('[')))
                    .all(constant),
                _ => false,
            },
        }
    }
    match term {
        Term::Group(parts) => match parts.first().and_then(Term::atom) {
            Some("nil") => true,
            Some("cons") => parts.len() == 3 && constant(&parts[1]) && constant_list(&parts[2]),
            _ => false,
        },
        Term::Atom(_) => false,
    }
}

/// Whether every list `term` constructs has storage, `result` saying
/// whether `term` is in the result's position of a definition whose result
/// is a list.
fn stored(term: &Term, result: bool, lists: &BTreeSet<&str>) -> bool {
    let Term::Group(parts) = term else {
        return true;
    };
    let all = |parts: &[Term], result: bool| parts.iter().all(|p| stored(p, result, lists));
    let listed = |parts: &[Term]| {
        parts.windows(2).any(|w| {
            w[0].atom() == Some(":")
                && matches!(&w[1], Term::Group(g) if g.first().and_then(Term::atom) == Some("List"))
        })
    };
    match parts.first().and_then(Term::atom) {
        Some("match") => {
            // `(match s | p => e | ...)`: the scrutinee is read, the arms
            // are the term's value.
            let mut arms = parts
                .get(1..)
                .unwrap_or_default()
                .split(|p| p.atom() == Some("|"));
            let scrutinee = arms.next().unwrap_or_default();
            all(scrutinee, false)
                && arms.all(|arm| {
                    let at = arm.iter().position(|p| p.atom() == Some("=>"));
                    at.is_some_and(|at| all(&arm[at + 1..], result))
                })
        }
        Some("if") => {
            let then = parts.iter().position(|p| p.atom() == Some("then"));
            let otherwise = parts.iter().position(|p| p.atom() == Some("else"));
            match (then, otherwise) {
                (Some(t), Some(e)) if t < e => {
                    all(&parts[1..t], false)
                        && all(&parts[t + 1..e], result)
                        && all(&parts[e + 1..], result)
                }
                _ => false,
            }
        }
        Some("let") => {
            // `(let x : T := v; b)`: a list-typed binding has no storage.
            let bind = parts.iter().position(|p| p.atom() == Some(":="));
            let end = parts.iter().position(|p| p.atom() == Some(";"));
            match (bind, end) {
                (Some(b), Some(e)) if b < e => {
                    !listed(&parts[..b])
                        && all(&parts[b + 1..e], false)
                        && all(&parts[e + 1..], result)
                }
                _ => false,
            }
        }
        Some("nil") => result,
        Some("cons") => {
            result
                && parts.len() == 3
                && stored(&parts[1], false, lists)
                && stored(&parts[2], true, lists)
        }
        Some("prim") if listed(parts) => {
            let append = parts.get(1).and_then(Term::atom) == Some("append");
            result && all(parts.get(2..).unwrap_or_default(), append)
        }
        Some(callee) if lists.contains(callee.rsplit("::").next().unwrap_or(callee)) => {
            // A computed list argument of a call is a jump argument with
            // no storage; a parameter passed on is borrowed.
            result
                && parts.get(1..).unwrap_or_default().iter().all(|arg| {
                    !matches!(arg, Term::Group(g) if is_list(g, lists)) && stored(arg, false, lists)
                })
        }
        _ => all(parts.get(1..).unwrap_or_default(), false),
    }
}

/// Whether a group constructs a list.
fn is_list(parts: &[Term], lists: &BTreeSet<&str>) -> bool {
    match parts.first().and_then(Term::atom) {
        Some("cons" | "nil") => true,
        Some(callee) => {
            lists.contains(callee.rsplit("::").next().unwrap_or(callee))
                || parts.windows(2).any(|w| {
                    w[0].atom() == Some(":")
                        && matches!(&w[1], Term::Group(g) if g.first().and_then(Term::atom) == Some("List"))
                })
        }
        None => false,
    }
}

/// Whether values of the LexLean type `ty` are `Copy` in the generated
/// crate (prod-codegen `copy_type`): text, bytes and lists are not; a
/// declared type is when every field's type is; an option or a tuple is
/// when its parts are; a number or a Boolean is. A type this cannot read,
/// or one declared outside Behavior, is taken as not `Copy`.
fn copy_type(
    ty: &Term,
    types: &BTreeMap<String, (&Decl, String)>,
    seen: &mut BTreeSet<String>,
) -> bool {
    match ty {
        Term::Atom(name) => match name.as_str() {
            "String" | "Bytes" => false,
            "Nat" | "Bool" | "Int8" | "Int16" | "Int32" | "Int64" | "UInt8" | "UInt16"
            | "UInt32" | "UInt64" => true,
            other => {
                if !seen.insert(other.to_string()) {
                    return false;
                }
                let copy = types.get(other).is_some_and(|(decl, _)| {
                    let fields = fields(decl);
                    Term::read(&format!("({fields})")).is_some_and(|t| match t {
                        Term::Group(parts) => parts.iter().all(|p| copy_type(p, types, seen)),
                        Term::Atom(_) => false,
                    })
                });
                seen.remove(other);
                copy
            }
        },
        Term::Group(parts) if parts.len() == 1 => copy_type(&parts[0], types, seen),
        Term::Group(parts) => match parts.first().and_then(Term::atom) {
            Some("List") => false,
            Some("Option" | "Result" | "Prod") => {
                parts[1..].iter().all(|p| copy_type(p, types, seen))
            }
            _ => false,
        },
    }
}

/// Whether a definition whose result is a list of elements that are not
/// `Copy` writes its output buffer from borrowed elements: the pinned code
/// generator assigns such an element without a clone and the generated crate
/// does not compile (E0308), so no such definition with a parameter is taken.
fn borrowed_output(text: &str, types: &BTreeMap<String, (&Decl, String)>) -> bool {
    let Some((head, _)) = text.split_once("body: ") else {
        return false;
    };
    if head.contains("parameters: []") {
        return false;
    }
    let Some(element) = head
        .split_once("result: (List ")
        .map(|(_, rest)| rest.split_once(",\n").map_or(rest, |(e, _)| e))
        .map(|e| e.trim_end().trim_end_matches(',').trim_end())
        .and_then(|e| e.strip_suffix(')'))
    else {
        return false;
    };
    Term::read(&format!("({element})")).is_none_or(|t| !copy_type(&t, types, &mut BTreeSet::new()))
}

/// The last component of a qualified definition name.
fn leaf(qualified: &str) -> &str {
    qualified.rsplit("::").next().unwrap_or(qualified)
}

/// Why a root whose closure is `own` cannot be a root of the library whose
/// admitted closure names its definitions by `taken`, if it cannot.
fn refusal(
    own: &BTreeSet<String>,
    definitions: &Definitions,
    decls: &BTreeMap<String, &Decl>,
    types: &BTreeMap<String, (&Decl, String)>,
    taken: &BTreeMap<String, String>,
) -> Option<String> {
    let mut leaves: BTreeMap<&str, &str> = BTreeMap::new();
    let lists: BTreeSet<&str> = own
        .iter()
        .filter(|d| {
            definitions.text(d).is_some_and(|text| {
                text.split_once("body: ")
                    .is_some_and(|(head, _)| head.contains("result: (List "))
            })
        })
        .map(|d| leaf(d))
        .collect();
    for d in own {
        if definitions
            .text(d)
            .is_some_and(|text| intermediate_list(text, &lists))
        {
            return Some(format!(
                "it reaches {d}, which holds a list as an intermediate value, and the pinned \
                 code generator stores a list only in a parameter or the output buffer"
            ));
        }
        if definitions
            .text(d)
            .is_some_and(|text| borrowed_output(text, types))
        {
            return Some(format!(
                "it reaches {d}, whose result is a list of elements that are not Copy, which \
                 the pinned code generator writes from borrowed elements without a clone"
            ));
        }
        if definitions.text(d).is_some_and(higher_order) {
            return Some(format!(
                "it reaches {d}, a definition with a function parameter, which no \
                 executable definition takes"
            ));
        }
        if let Some(primitive) = definitions.text(d).and_then(|text| {
            COLLECTION_PRIMITIVES
                .iter()
                .find(|p| text.contains(&format!("(prim {p} ")))
        }) {
            return Some(format!(
                "it reaches {d}, which calls the primitive {primitive}, a call of LexLean's \
                 collections runtime the pinned exporter leaves unresolved"
            ));
        }
        if let Some(Decl::Definition {
            mutual,
            termination,
            ..
        }) = decls.get(d)
        {
            if let Some(group) = mutual {
                return Some(format!(
                    "it reaches {d}, a definition of the mutual recursion group {group}, \
                     whose recursor the pinned exporter does not compile"
                ));
            }
            for evidence in termination.iter().flat_map(|(_, e)| e) {
                if let Some(Decl::Theorem { axioms, .. }) =
                    decls.get(&qualified(BEHAVIOR, evidence))
                {
                    if !axioms.is_empty() {
                        return Some(format!(
                            "it reaches {d}, whose termination evidence {evidence} depends \
                             on the axioms {}, and a library's theorems must depend on none",
                            axioms.join(", ")
                        ));
                    }
                }
            }
        }
        let name = leaf(d);
        let identifier = name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            && name != "_";
        if !identifier || FUNCTION_KEYWORDS.contains(&name) {
            return Some(format!(
                "it reaches {d}, whose last component {name} is no Rust function name"
            ));
        }
        let other = leaves.insert(name, d).or_else(|| {
            taken
                .get(name)
                .map(String::as_str)
                .filter(|o| *o != d.as_str())
        });
        if let Some(other) = other {
            return Some(format!(
                "it reaches {d}, whose last component {name} is also that of {other} \
                 in the library's closure"
            ));
        }
    }
    None
}

/// The library projects of the workspace: one per package with a library
/// target, its metadata from Cargo, its exports the admitted roots among
/// its public lifted functions and the closed Boolean claims the
/// workspace's tests make over its lifted functions, and its acceptance
/// roots those claims (UR-R29).
///
/// A slot Cargo leaves empty is inferred where Cargo itself defines the
/// default: the homepage is the repository when only that is declared
/// (Cargo's manifest reference: a homepage distinct from the source
/// repository), and the description is the crate doc's summary line, the
/// text rustdoc and crates.io show for the crate. A slot no rule defines is
/// an undefined slot in the register, and that package gets no project.
///
/// `public` holds the crate paths of the functions declared `pub`: Rust's
/// own notion of what a library offers, so private functions are in the
/// library only as its roots reach them.
pub(super) fn libraries(
    ws: &Workspace,
    prefix: &str,
    behavior: &Module,
    public: &BTreeSet<String>,
    notes: &mut Notes,
) -> Vec<Library> {
    let defined: BTreeSet<&str> = behavior
        .decls
        .iter()
        .filter_map(|decl| match decl {
            Decl::Definition { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    let texts: BTreeMap<String, String> = [(BEHAVIOR.to_string(), behavior.render())]
        .into_iter()
        .collect();
    let definitions = Definitions::of(&texts);
    let claims: BTreeSet<String> = notes
        .roots
        .values()
        .flatten()
        .filter(|root| defined.contains(root.as_str()))
        .cloned()
        .collect();
    let mut out = Vec::new();
    for package in ws
        .packages
        .iter()
        .filter(|p| p.targets.iter().any(|t| t.kind == TargetKind::Lib))
    {
        // The library's own lifted functions, by definition, and whether
        // each is public: those defined in a file of this package's library
        // target. A crate name does not say it: Cargo lets two packages give
        // their libraries one name, and a binary target (of this package, or
        // another's `[[bin]]`) take the library's name, and neither puts a
        // function into the library.
        let files = lib_files(package);
        let own: BTreeMap<String, bool> = notes
            .lifted
            .iter()
            .filter(|(_, (span, _, _))| files.contains(span.file.as_str()))
            .filter_map(|(path, (_, q, _))| {
                q.strip_prefix(&qualified(BEHAVIOR, ""))
                    .filter(|name| defined.contains(name))
                    .map(|name| (name.to_string(), public.contains(path)))
            })
            .collect();
        // A claim of any package's tests that runs one of them.
        let acceptance: BTreeSet<String> = claims
            .iter()
            .filter(|claim| {
                definitions
                    .closure(&qualified(BEHAVIOR, claim))
                    .iter()
                    .any(|d| own.contains_key(leaf(d)))
            })
            .cloned()
            .collect();
        let candidates: Vec<String> = acceptance
            .iter()
            .cloned()
            .chain(
                own.iter()
                    .filter(|(_, public)| **public)
                    .map(|(name, _)| name.clone()),
            )
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let admission = admit(behavior, &candidates);
        for (root, cause) in &admission.refused {
            notes.refusals.insert(Refusal::LibraryRoot {
                package: package.name.clone(),
                root: root.clone(),
                cause: cause.clone(),
            });
        }
        let accepted: Vec<&String> = acceptance
            .iter()
            .filter(|r| admission.roots.contains(*r))
            .collect();
        let qualify = |roots: &[&String]| -> Vec<String> {
            roots
                .iter()
                .map(|r| format!("{prefix}.{BEHAVIOR}.{r}"))
                .collect()
        };
        let exports = qualify(&admission.roots.iter().collect::<Vec<_>>());
        let accepted = qualify(&accepted);
        match project(
            package,
            &exports,
            &accepted,
            &candidates,
            &acceptance,
            notes,
        ) {
            Some(value) => out.push(Library {
                package: package.name.clone(),
                behavior: closure(behavior, &admission),
                value,
            }),
            None => continue,
        }
    }
    out
}

/// The files of the package's library target, by their path relative to the
/// workspace root: those ingest assigns to it (`ingest::assign`, the target
/// whose root's directory tree holds the file, the library before a binary
/// in a shared `src/`).
fn lib_files(package: &Package) -> BTreeSet<&str> {
    package
        .files
        .iter()
        .filter(|f| {
            package
                .targets
                .get(f.target)
                .is_some_and(|t| t.kind == TargetKind::Lib)
        })
        .map(|f| f.rel.as_str())
        .collect()
}

/// The native-library profile's bounds on the Cargo metadata it holds
/// (SPEC 12.6, enforced by PrismPM's `holo/library.rs` `validate`): a
/// description of at most 1024 UTF-8 bytes, a Cargo name of at most 64, and
/// repository and homepage URIs of at most 2048.
const DESCRIPTION_BYTES: usize = 1024;
const CARGO_NAME_BYTES: usize = 64;
const URI_BYTES: usize = 2048;

/// What the native-library profile admits for the metadata `slot`, when
/// `value` is not that (SPEC 12.6): a control-free description of at most
/// [`DESCRIPTION_BYTES`]; a Cargo name of 1 to [`CARGO_NAME_BYTES`]
/// lowercase ASCII letters, digits, `_` or `-`, beginning with a letter; a
/// canonical stable SemVer version (no pre-release, no build metadata); and
/// for the repository and the homepage an ASCII `https` URI of at most
/// [`URI_BYTES`] by RFC 3986's grammar, whose host is a registered name or
/// an IPv4 address. Cargo admits more in each (an `http` URL, an upper-case
/// name, a pre-release), and a project holding such a value is one
/// `prismpm check` refuses, so the package gets a row instead.
///
/// A value holding a percent sign is refused whatever the slot: a LexLean
/// source admits no raw percent character (LexLean SPEC 12.1), in a string
/// literal too, so no project can state it.
fn refused(slot: &str, value: &str) -> Option<String> {
    if value.contains('%') {
        return Some("no percent sign in a LexLean source (LexLean SPEC 12.1)".to_string());
    }
    let admitted = match slot {
        "description" => value.len() <= DESCRIPTION_BYTES && !value.chars().any(char::is_control),
        "name" => {
            value.len() <= CARGO_NAME_BYTES
                && value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                && value.bytes().all(|b| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-')
                })
        }
        "version" => semver::Version::parse(value)
            .is_ok_and(|v| v.pre.is_empty() && v.build.is_empty() && v.to_string() == value),
        _ => value.len() <= URI_BYTES && https_uri(value),
    };
    if admitted {
        return None;
    }
    Some(match slot {
        "description" => format!("control-free text of at most {DESCRIPTION_BYTES} bytes"),
        "name" => format!(
            "1 to {CARGO_NAME_BYTES} lowercase ASCII letters, digits, underscores or \
             hyphens, beginning with a letter"
        ),
        "version" => "a canonical stable SemVer version".to_string(),
        _ => format!(
            "an ASCII https URI of at most {URI_BYTES} bytes whose host is a name or an \
             IPv4 address"
        ),
    })
}

/// Whether `value` is an `https` URI by RFC 3986's grammar (`"https://"
/// authority path-abempty ["?" query] ["#" fragment]`), with the host a
/// registered name or an IPv4 address (both read by the `reg-name` rule) and
/// no IP literal, which this check does not read and so refuses.
fn https_uri(value: &str) -> bool {
    const SUB_DELIMS: &[u8] = b"!$&'()*+,;=";
    let unreserved = |b: u8| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~');
    // Each byte is in `extra`, unreserved or a sub-delimiter, or begins a
    // percent-encoded octet.
    let chars = |text: &str, extra: &[u8]| {
        let bytes = text.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if b == b'%' {
                let hex = |at: usize| bytes.get(at).is_some_and(u8::is_ascii_hexdigit);
                if !(hex(i + 1) && hex(i + 2)) {
                    return false;
                }
                i += 3;
                continue;
            }
            if !(unreserved(b) || SUB_DELIMS.contains(&b) || extra.contains(&b)) {
                return false;
            }
            i += 1;
        }
        true
    };
    let Some(rest) = value.strip_prefix("https://") else {
        return false;
    };
    let (rest, fragment) = rest.split_once('#').unwrap_or((rest, ""));
    let (rest, query) = rest.split_once('?').unwrap_or((rest, ""));
    let (authority, path) = match rest.find('/') {
        Some(at) => rest.split_at(at),
        None => (rest, ""),
    };
    let (userinfo, host_port) = authority.split_once('@').unwrap_or(("", authority));
    let (host, port) = host_port.rsplit_once(':').unwrap_or((host_port, ""));
    chars(fragment, b":@/?")
        && chars(query, b":@/?")
        && chars(path, b":@/")
        && chars(userinfo, b":")
        && chars(host, b"")
        && port.bytes().all(|b| b.is_ascii_digit())
}

/// The package's `NativeLibrary` value over `exports` and `acceptance`, or
/// none, with each slot no rule defines an undefined slot of the register.
fn project(
    package: &Package,
    exports: &[String],
    acceptance: &[String],
    candidates: &[String],
    claims: &BTreeSet<String>,
    notes: &mut Notes,
) -> Option<Module> {
    let crate_doc = package
        .targets
        .iter()
        .find(|t| t.kind == TargetKind::Lib)
        .and_then(|t| package.files.iter().find(|f| f.rel == t.src))
        .map(|f| summary(&f.doc))
        .unwrap_or_default();
    let description = if package.description.trim().is_empty() {
        crate_doc
    } else {
        package.description.clone()
    };
    let homepage = if package.homepage.is_empty() {
        package.repository.clone()
    } else {
        package.homepage.clone()
    };
    // Each slot names the manifest field that would fill it: the SDK's
    // native-library profile requires all three (`cargoDescription`, and
    // `cargoRepository` and `cargoHomepage` as HTTPS URIs), and Cargo's
    // manifest is the only statement of them the projection reads.
    let mut undefined = Vec::new();
    for (slot, value, fields) in [
        (
            "description",
            &description,
            "`package.description`, and its crate doc has no summary line",
        ),
        ("repository", &package.repository, "`package.repository`"),
        (
            "homepage",
            &homepage,
            "`package.homepage` or `package.repository`",
        ),
    ] {
        if value.trim().is_empty() {
            undefined.push((
                slot,
                format!(
                    "its Cargo.toml states no {fields}; the SDK's native-library profile \
                     requires cargo{} and no rule infers one",
                    pascal(&[slot])
                ),
            ));
        } else if let Some(want) = refused(slot, value) {
            undefined.push((
                slot,
                format!(
                    "its Cargo.toml states {fields} as a value no library project can \
                     hold as cargo{}: the SDK admits {want}",
                    pascal(&[slot])
                ),
            ));
        }
    }
    for (slot, value, field) in [
        ("name", &package.name, "`package.name`"),
        ("version", &package.version, "`package.version`"),
    ] {
        if let Some(want) = refused(slot, value) {
            undefined.push((
                slot,
                format!(
                    "its Cargo.toml states {field} as a value no library project can \
                     hold as cargo{}: the SDK admits {want}",
                    pascal(&[slot])
                ),
            ));
        }
    }
    let why_empty = |slot: &str, proposed: usize| {
        if proposed == 0 {
            match slot {
                "exportRoots" => "the library has no lifted public function and its tests \
                                  state no closed claim over its lifted functions"
                    .to_string(),
                _ => "its tests state no closed claim over its lifted functions".to_string(),
            }
        } else {
            format!("each of the {proposed} roots proposed for it is a refused library-root row")
        }
    };
    for (slot, roots, proposed) in [
        ("exportRoots", exports, candidates.len()),
        ("acceptanceRoots", acceptance, claims.len()),
    ] {
        if roots.is_empty() {
            undefined.push((slot, why_empty(slot, proposed)));
        } else if roots.len() > LIBRARY_ROOTS {
            undefined.push((
                slot,
                format!(
                    "the crate yields {} {slot}, past the profile's {LIBRARY_ROOTS}",
                    roots.len()
                ),
            ));
        }
    }
    if !undefined.is_empty() {
        for (slot, reason) in undefined {
            notes.refusals.insert(Refusal::UndefinedSlot {
                package: package.name.clone(),
                slot,
                reason,
            });
        }
        return None;
    }
    let list = |roots: &[String]| {
        crate::embed::term_list(
            &roots.iter().map(|r| string(r)).collect::<Vec<_>>(),
            "String",
        )
    };
    let body = format!(
        "(record {LIBRARY_MODEL}::NativeLibrary {{profile := \"prismpm/native-library/1\", \
         name := {}, cargoName := {}, cargoVersion := {}, cargoDescription := {}, \
         cargoRepository := {}, cargoHomepage := {}, exportRoots := {}, acceptanceRoots := {}}})",
        string(&package.name),
        string(&package.name),
        string(&package.version),
        string(&description),
        string(&package.repository),
        string(&homepage),
        list(exports),
        list(acceptance),
    );
    Some(Module {
        name: "Library".into(),
        glossaries: vec![BOOL, NAT],
        imports: vec![LIBRARY_MODEL.into()],
        title: "Boolean",
        decls: vec![Decl::value(
            "library",
            format!("{LIBRARY_MODEL}::NativeLibrary"),
            body,
        )],
    })
}

/// The Behavior module of a library project: the types and definitions
/// the admitted roots reach, and the theorems those definitions cite as
/// termination evidence, in the workspace module's order and under its
/// names.
fn closure(behavior: &Module, admission: &Admission) -> Module {
    let closure = &admission.closure;
    let kept: BTreeSet<String> = closure
        .iter()
        .filter_map(|d| d.strip_prefix(&qualified(BEHAVIOR, "")))
        .map(str::to_string)
        .collect();
    let evidence: BTreeSet<&str> = behavior
        .decls
        .iter()
        .filter_map(|d| match d {
            Decl::Definition {
                name, termination, ..
            } if kept.contains(name) => termination.as_ref(),
            _ => None,
        })
        .flat_map(|(_, e)| e.iter().map(String::as_str))
        .collect();
    let decls = behavior
        .decls
        .iter()
        .filter(|d| match d {
            Decl::Structure { name, .. }
            | Decl::Inductive { name, .. }
            | Decl::Data { name, .. } => admission.types.contains(name),
            Decl::Definition { name, .. } => kept.contains(name),
            Decl::Theorem { name, .. } => evidence.contains(name.as_str()),
        })
        .cloned()
        .collect();
    Module {
        decls,
        ..behavior.clone()
    }
}

#[cfg(test)]
mod tests {
    // Class: the library route (per-root admission of a library project).
    use super::*;
    use crate::lex::Expr;

    fn library_def(name: &str, body: &str) -> Decl {
        Decl::Definition {
            name: name.into(),
            parameters: vec![("n".into(), "Nat".into())],
            result: "Nat".into(),
            body: Expr::Flat(body.into()),
            axioms: Vec::new(),
            recursive: None,
            termination: None,
            mutual: None,
        }
    }

    fn library_module(decls: Vec<Decl>) -> Module {
        Module {
            name: BEHAVIOR.into(),
            glossaries: vec![BOOL, NAT],
            imports: vec![PRELUDE.into()],
            title: "Lifted Rust behavior",
            decls,
        }
    }

    fn library_names(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| (*n).to_string()).collect()
    }

    #[test]
    fn library_a_root_reaching_a_mutual_group_is_refused_alone() {
        let mut inner = library_def("sizeOfTree", "(sizeOfTree n)");
        if let Decl::Definition { mutual, .. } = &mut inner {
            *mutual = Some("SizeOfTree".into());
        }
        let behavior = library_module(vec![
            inner,
            library_def("treeSize", "(sizeOfTree n)"),
            library_def("plain", "(n + 1)"),
        ]);
        let admission = admit(&behavior, &library_names(&["plain", "treeSize"]));
        assert_eq!(admission.roots, ["plain".to_string()].into_iter().collect());
        let cause = &admission.refused["treeSize"];
        assert!(cause.contains("Behavior::sizeOfTree"), "{cause}");
        assert!(
            cause.contains("mutual recursion group SizeOfTree"),
            "{cause}"
        );
        assert!(!admission.closure.contains("Behavior::sizeOfTree"));
    }

    #[test]
    fn library_termination_evidence_with_axioms_refuses_its_root() {
        let evidence = |name: &str, axioms: Vec<String>| Decl::Theorem {
            name: name.into(),
            parameters: Vec::new(),
            statement: "True".into(),
            proof: "trivial{}".into(),
            axioms,
        };
        let looping = |name: &str, theorem: &str| {
            let mut d = library_def(name, &format!("({name} n)"));
            if let Decl::Definition { termination, .. } = &mut d {
                *termination = Some(("n".into(), vec![theorem.into()]));
            }
            d
        };
        let behavior = library_module(vec![
            evidence("decreases_a", vec!["propext".into()]),
            evidence("decreases_b", Vec::new()),
            looping("loopA", "decreases_a"),
            looping("loopB", "decreases_b"),
        ]);
        let admission = admit(&behavior, &library_names(&["loopA", "loopB"]));
        assert!(admission.roots.contains("loopB"));
        let cause = &admission.refused["loopA"];
        assert!(
            cause.contains("decreases_a depends on the axioms propext"),
            "{cause}"
        );
        // The library keeps the evidence of what it admitted, and only that.
        let kept = closure(&behavior, &admission);
        let names: Vec<&str> = kept.decls.iter().map(Decl::name).collect();
        assert_eq!(names, ["decreases_b", "loopB"]);
    }

    #[test]
    fn library_a_function_parameter_or_a_keyword_name_is_refused() {
        let mut hof = library_def("applied", "(f n)");
        if let Decl::Definition { parameters, .. } = &mut hof {
            parameters.push(("f".into(), "(Fn Nat Nat)".into()));
        }
        let behavior = library_module(vec![
            hof,
            library_def("try", "n"),
            library_def("viaTry", "(try n)"),
        ]);
        let admission = admit(&behavior, &library_names(&["applied", "viaTry"]));
        assert!(admission.roots.is_empty(), "{admission:?}");
        assert!(admission.refused["applied"].contains("function parameter"));
        assert!(admission.refused["viaTry"].contains("last component try is no Rust"));
    }

    #[test]
    fn library_a_type_of_a_mutual_block_is_refused_and_unreached_types_stay_out() {
        let data = |name: &str, field: &str| Decl::Data {
            name: name.into(),
            constructors: vec![("leaf".into(), vec![field.into()])],
            mutual: Some("TreeGroup".into()),
        };
        let plain = |name: &str, field: &str| Decl::Structure {
            name: name.into(),
            fields: vec![("value".into(), field.into())],
        };
        let mut reads = library_def("depth", "n");
        if let Decl::Definition { parameters, .. } = &mut reads {
            parameters.push(("t".into(), "Tree".into()));
        }
        let mut boxed = library_def("unbox", "n");
        if let Decl::Definition { parameters, .. } = &mut boxed {
            parameters.push(("b".into(), "Box".into()));
        }
        let behavior = library_module(vec![
            data("Tree", "Forest"),
            data("Forest", "Tree"),
            plain("Inner", "Nat"),
            plain("Box", "Inner"),
            plain("Unused", "Nat"),
            reads,
            boxed,
        ]);
        let admission = admit(&behavior, &library_names(&["depth", "unbox"]));
        let cause = &admission.refused["depth"];
        assert!(
            cause.contains("mutual inductive block TreeGroup"),
            "{cause}"
        );
        let kept = closure(&behavior, &admission);
        let names: Vec<&str> = kept.decls.iter().map(Decl::name).collect();
        assert_eq!(names, ["Inner", "Box", "unbox"]);
    }

    #[test]
    fn library_a_recursive_type_or_an_unbounded_int_is_refused() {
        let zone = Decl::Data {
            name: "Zone".into(),
            constructors: vec![
                ("Room".into(), vec!["String".into()]),
                ("Group".into(), vec!["(List Zone)".into()]),
            ],
            mutual: None,
        };
        let typed = |name: &str, ty: &str| {
            let mut d = library_def(name, "n");
            if let Decl::Definition { parameters, .. } = &mut d {
                parameters.push(("x".into(), ty.into()));
            }
            d
        };
        let behavior = library_module(vec![
            zone,
            typed("rooms", "Zone"),
            typed("signed", "Int"),
            typed("count", "Nat"),
        ]);
        let admission = admit(&behavior, &library_names(&["count", "rooms", "signed"]));
        assert_eq!(admission.roots, ["count".to_string()].into_iter().collect());
        assert!(
            admission.refused["rooms"].contains("type Zone, which is defined in terms of itself")
        );
        assert!(admission.refused["signed"].contains("the type Int"));
    }

    #[test]
    fn library_an_intermediate_list_is_refused_and_a_built_result_is_not() {
        let listed = |name: &str, result: &str, body: &str| {
            let mut d = library_def(name, body);
            if let Decl::Definition {
                parameters,
                result: r,
                ..
            } = &mut d
            {
                parameters.push(("xs".into(), "(List Nat)".into()));
                *r = result.into();
            }
            d
        };
        let behavior = library_module(vec![
            listed("pushed", "(List Nat)", "(cons n xs)"),
            listed("counted", "Nat", "(prim length (pushed n xs) : Nat)"),
            listed(
                "kept",
                "(List Nat)",
                "(let ys : (List Nat) := (cons n xs); (pushed n ys))",
            ),
            listed("first", "Nat", "(prim length xs : Nat)"),
        ]);
        let admission = admit(
            &behavior,
            &library_names(&["counted", "first", "kept", "pushed"]),
        );
        let roots: Vec<&str> = admission.roots.iter().map(String::as_str).collect();
        assert_eq!(roots, ["first", "pushed"], "{admission:?}");
        assert!(admission.refused["counted"].contains("Behavior::counted, which holds a list"));
        assert!(admission.refused["kept"].contains("Behavior::kept, which holds a list"));
    }

    #[test]
    fn library_a_list_with_no_parameter_is_admitted_only_as_constants() {
        let constant = |name: &str, body: &str| Decl::Definition {
            name: name.into(),
            parameters: Vec::new(),
            result: "(List Nat)".into(),
            body: Expr::Flat(body.into()),
            axioms: Vec::new(),
            recursive: None,
            termination: None,
            mutual: None,
        };
        let behavior = library_module(vec![
            constant("digits", "(cons 1 (cons 2 (nil Nat)))"),
            constant("labels", "(cons \"a\" (nil String))"),
        ]);
        let admission = admit(&behavior, &library_names(&["digits", "labels"]));
        assert!(admission.roots.contains("digits"), "{admission:?}");
        assert!(admission.refused["labels"].contains("holds a list"));
    }

    #[test]
    fn library_a_list_of_elements_that_are_not_copy_is_refused() {
        let listed = |name: &str, element: &str| {
            let mut d = library_def(name, &format!("(cons x (nil {element}))"));
            if let Decl::Definition {
                parameters, result, ..
            } = &mut d
            {
                *parameters = vec![("x".into(), element.into())];
                *result = format!("(List {element})");
            }
            d
        };
        let point = Decl::Structure {
            name: "Point".into(),
            fields: vec![("x".into(), "Nat".into()), ("on".into(), "Bool".into())],
        };
        let named = Decl::Structure {
            name: "Named".into(),
            fields: vec![("label".into(), "String".into())],
        };
        let behavior = library_module(vec![
            point,
            named,
            listed("points", "Point"),
            listed("nameds", "Named"),
            listed("texts", "String"),
            listed("counts", "(Option Nat)"),
        ]);
        let admission = admit(
            &behavior,
            &library_names(&["counts", "nameds", "points", "texts"]),
        );
        let roots: Vec<&str> = admission.roots.iter().map(String::as_str).collect();
        assert_eq!(roots, ["counts", "points"], "{admission:?}");
        assert!(admission.refused["texts"].contains("elements that are not Copy"));
        assert!(admission.refused["nameds"].contains("elements that are not Copy"));
    }

    #[test]
    fn library_a_collections_runtime_call_is_refused() {
        let behavior = library_module(vec![
            library_def("before", "(prim less_than n n : Bool)"),
            library_def("viaBefore", "(before n)"),
            library_def("same", "(prim equal n n : Bool)"),
        ]);
        let admission = admit(&behavior, &library_names(&["same", "viaBefore"]));
        assert!(admission.roots.contains("same"), "{admission:?}");
        let cause = &admission.refused["viaBefore"];
        assert!(
            cause.contains("Behavior::before, which calls the primitive less_than"),
            "{cause}"
        );
    }

    #[test]
    fn library_a_last_component_two_definitions_share_refuses_the_later_root() {
        // A Behavior definition named like a Prelude definition: the
        // exporter names both `textLength`, so only one may be in the library.
        let behavior = library_module(vec![
            library_def("decimal", "(Prelude::textLength n)"),
            library_def("textLength", "n"),
            library_def("own", "(textLength n)"),
        ]);
        let admission = admit(&behavior, &library_names(&["decimal", "own"]));
        assert!(admission.roots.contains("decimal"), "{admission:?}");
        let cause = &admission.refused["own"];
        assert!(
            cause.contains("last component textLength is also that of Prelude::textLength"),
            "{cause}"
        );
    }

    /// The profile's metadata rules (SPEC 12.6), both ways: what PrismPM
    /// admits is admitted, what Cargo admits beyond it is refused.
    #[test]
    fn library_profile_admits_only_what_the_sdk_admits() {
        for uri in [
            "https://example.org/zeta",
            "https://example.org",
            "https://user:pw@example.org:8443/a/b?q=1&r=/x?#frag/ment",
            "https://127.0.0.1/",
            "https://example.org/a(b)!$*,;=~",
        ] {
            assert_eq!(refused("repository", uri), None, "{uri}");
            assert_eq!(refused("homepage", uri), None, "{uri}");
        }
        for uri in [
            "http://example.org/zeta",
            "HTTPS://example.org",
            "ftp://example.org",
            "https://exa mple.org",
            "https://example.org/caf\u{e9}",
            "https://example.org/%zz",
            "https://example.org/%4",
            "https://example.org/a%2Fb",
            "https://example.org:80a/",
            "https://a:b:c/",
            "https://a@b@c/",
            "https://[::1]/",
            "https://example.org/a^b",
            "https://example.org/#a#b",
            "https://example.org/\u{7f}",
        ] {
            assert!(refused("repository", uri).is_some(), "{uri}");
        }
        let long = format!("https://example.org/{}", "a".repeat(URI_BYTES));
        assert!(refused("repository", &long).is_some());
        assert_eq!(refused("repository", &long[..URI_BYTES]), None);
        for name in ["zeta", "z", "zeta-core_2"] {
            assert_eq!(refused("name", name), None, "{name}");
        }
        for name in ["Zeta", "2zeta", "_zeta", "ze.ta", ""] {
            assert!(refused("name", name).is_some(), "{name}");
        }
        assert!(refused("name", &"z".repeat(CARGO_NAME_BYTES + 1)).is_some());
        assert_eq!(refused("name", &"z".repeat(CARGO_NAME_BYTES)), None);
        assert_eq!(refused("version", "0.1.0"), None);
        for version in ["0.1.0-alpha.1", "1.0.0+build", "1.0", "01.0.0"] {
            assert!(refused("version", version).is_some(), "{version}");
        }
        assert_eq!(refused("description", "The zeta part."), None);
        assert!(refused("description", "Two\nlines").is_some());
        assert!(refused("description", "The 100% part.").is_some());
        assert!(refused("description", &"d".repeat(DESCRIPTION_BYTES + 1)).is_some());
        assert_eq!(refused("description", &"d".repeat(DESCRIPTION_BYTES)), None);
    }
}
