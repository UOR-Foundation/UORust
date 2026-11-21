//! Names: Rust's name resolution over a workspace, as far as the written
//! code decides it.
//!
//! Admission may call a call ambiguous only after it is resolved the way the
//! language resolves it: a path through the `use` bindings, the globs and
//! the modules of the module it stands in, `Self` and `self` to the type of
//! the `impl` around it, a method by the type its receiver is written with,
//! and one of several functions of a name by the `cfg` predicates they stand
//! under and by the types of the arguments. What the written code does not
//! decide stays open, and nothing here guesses: a rule either names the one
//! function the language would, says the call leaves the workspace, or
//! leaves the call to admission.
//!
//! The facts are read from the source once more, beside ingest's: each
//! module's bindings (an inline `mod` is a module of its own) and each
//! function body's calls with what the code writes of their arguments' types.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::visit::Visit;

use crate::admit::Refusal;
use crate::ast::{FnArg, GenericArgument, PathArguments, RustType, TypeKind};
use crate::error::{io, UorustError};
use crate::facts::{FnFact, SourceFile, Span, Syntax, TargetKind, Workspace};

/// A `cfg` predicate, as its attribute writes it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Cfg {
    /// One option: `test`, or `feature = "x"` with its value.
    Option(String, Option<String>),
    Not(Box<Cfg>),
    All(Vec<Cfg>),
    Any(Vec<Cfg>),
}

/// One `cfg` option: its name and, for `name = "value"`, its value.
type CfgOption = (String, Option<String>);

/// What a build decides of `cfg` options.
#[derive(Clone, Debug, Default)]
pub(crate) struct Config {
    /// The options it decides, each with whether it holds.
    decided: BTreeMap<CfgOption, bool>,
    /// The features it enables, when it says which: every other is off.
    features: Option<BTreeSet<String>>,
}

/// rustc's codegen switches that are `cfg` options too, each with the option
/// it sets (the rustc book, "Codegen options").
const CODEGEN_OPTIONS: [(&str, &str); 2] = [
    ("-Cdebug-assertions=", "debug_assertions"),
    ("-Coverflow-checks=", "overflow_checks"),
];

impl Config {
    /// The configuration the lift's MIR dump compiles, which a projection
    /// states: the package's default features, no `test`, and what
    /// [`crate::mir::FLAGS`] pass to rustc. The target is the host's, which
    /// no file of the workspace declares: no `target_*` option is decided.
    fn dump(features: BTreeSet<String>) -> Config {
        let mut decided = BTreeMap::new();
        decided.insert(("test".to_string(), None), false);
        for flag in crate::mir::FLAGS {
            if let Some(option) = flag.strip_prefix("--cfg=") {
                let (name, value) = match option.split_once('=') {
                    Some((n, v)) => (n, Some(v.trim_matches('"').to_string())),
                    None => (option, None),
                };
                decided.insert((name.to_string(), value), true);
            }
            for (switch, option) in CODEGEN_OPTIONS {
                let state = match flag.strip_prefix(switch) {
                    Some("y" | "yes" | "on" | "true") => true,
                    Some("n" | "no" | "off" | "false") => false,
                    _ => continue,
                };
                decided.insert((option.to_string(), None), state);
            }
        }
        Config {
            decided,
            features: Some(features),
        }
    }

    fn says(&self, name: &str, value: Option<&str>) -> Option<bool> {
        let key = (name.to_string(), value.map(str::to_string));
        if let Some(held) = self.decided.get(&key) {
            return Some(*held);
        }
        match (name, value, &self.features) {
            ("feature", Some(v), Some(on)) => Some(on.contains(v)),
            _ => None,
        }
    }
}

impl Cfg {
    /// The predicates of the `#[cfg(..)]` attributes among `attrs`.
    fn of(attrs: &[syn::Attribute]) -> Vec<Cfg> {
        attrs
            .iter()
            .filter(|a| a.path().is_ident("cfg"))
            .map(|a| match a.parse_args::<syn::Meta>() {
                Ok(meta) => Cfg::read(&meta),
                // A predicate this reader does not parse is an option of its
                // own, which no configuration decides.
                Err(_) => Cfg::Option(written(a), None),
            })
            .collect()
    }

    fn read(meta: &syn::Meta) -> Cfg {
        match meta {
            syn::Meta::Path(p) => Cfg::Option(written(p), None),
            syn::Meta::NameValue(nv) => {
                let value = match &nv.value {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) => s.value(),
                    other => written(other),
                };
                Cfg::Option(written(&nv.path), Some(value))
            }
            syn::Meta::List(l) => {
                let inner = l
                    .parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
                    .map(|list| list.iter().map(Cfg::read).collect::<Vec<Cfg>>());
                match (written(&l.path).as_str(), inner) {
                    ("not", Ok(inner)) if inner.len() == 1 => Cfg::Not(Box::new(inner[0].clone())),
                    ("all", Ok(inner)) => Cfg::All(inner),
                    ("any", Ok(inner)) => Cfg::Any(inner),
                    _ => Cfg::Option(written(meta), None),
                }
            }
        }
    }

    /// The predicate as Rust writes it inside `cfg( )`.
    pub(crate) fn text(&self) -> String {
        let list = |cs: &[Cfg]| cs.iter().map(Cfg::text).collect::<Vec<_>>().join(", ");
        match self {
            Cfg::Option(name, None) => name.clone(),
            Cfg::Option(name, Some(value)) => format!("{name} = \"{value}\""),
            Cfg::Not(c) => format!("not({})", c.text()),
            Cfg::All(cs) => format!("all({})", list(cs)),
            Cfg::Any(cs) => format!("any({})", list(cs)),
        }
    }

    /// Whether the predicate holds under `config`: `None` where the
    /// configuration does not decide it.
    fn holds(&self, config: &Config) -> Option<bool> {
        match self {
            Cfg::Option(name, value) => config.says(name, value.as_deref()),
            Cfg::Not(c) => c.holds(config).map(|held| !held),
            Cfg::All(cs) => {
                let each: Vec<Option<bool>> = cs.iter().map(|c| c.holds(config)).collect();
                if each.contains(&Some(false)) {
                    Some(false)
                } else if each.iter().all(|held| *held == Some(true)) {
                    Some(true)
                } else {
                    None
                }
            }
            Cfg::Any(cs) => {
                let each: Vec<Option<bool>> = cs.iter().map(|c| c.holds(config)).collect();
                if each.contains(&Some(true)) {
                    Some(true)
                } else if each.iter().all(|held| *held == Some(false)) {
                    Some(false)
                } else {
                    None
                }
            }
        }
    }

    fn options(&self, out: &mut BTreeSet<CfgOption>) {
        match self {
            Cfg::Option(name, value) => {
                out.insert((name.clone(), value.clone()));
            }
            Cfg::Not(c) => c.options(out),
            Cfg::All(cs) | Cfg::Any(cs) => cs.iter().for_each(|c| c.options(out)),
        }
    }
}

/// Whether every predicate of `all` holds under `config`.
fn hold(all: &[Cfg], config: &Config) -> Option<bool> {
    Cfg::All(all.to_vec()).holds(config)
}

/// Whether some build holds every predicate of `all`: some assignment of
/// their options makes each true. Options are taken as independent, so two
/// values of one single-valued option (`target_os`) count as a build; the
/// answer errs towards "some build does".
fn satisfiable(all: &[Cfg]) -> bool {
    fn search(all: &[Cfg], options: &[CfgOption], config: &mut Config) -> bool {
        match hold(all, config) {
            Some(decided) => decided,
            None => {
                let Some((first, rest)) = options.split_first() else {
                    return false;
                };
                for held in [true, false] {
                    config.decided.insert(first.clone(), held);
                    if search(all, rest, config) {
                        config.decided.remove(first);
                        return true;
                    }
                }
                config.decided.remove(first);
                false
            }
        }
    }
    let mut options = BTreeSet::new();
    all.iter().for_each(|c| c.options(&mut options));
    let options: Vec<CfgOption> = options.into_iter().collect();
    search(all, &options, &mut Config::default())
}

/// Whether no build holds both `a` and `b`: two items under them never
/// stand in one crate together.
fn exclusive(a: &[Cfg], b: &[Cfg]) -> bool {
    let both: Vec<Cfg> = a.iter().chain(b).cloned().collect();
    !satisfiable(&both)
}

/// The tokens as Rust writes them.
fn written(tokens: &dyn ToTokens) -> String {
    crate::ingest::tidy(&tokens.to_token_stream().to_string())
}

/// Who may name an item, as its declaration writes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Vis {
    Public,
    /// `pub(crate)`.
    Crate,
    /// `pub(super)`.
    Super,
    /// No `pub`, or `pub(self)`.
    Private,
    /// `pub(in path)`, the path as written.
    In(Vec<String>),
}

impl Vis {
    fn read(vis: &syn::Visibility) -> Vis {
        match vis {
            syn::Visibility::Public(_) => Vis::Public,
            syn::Visibility::Inherited => Vis::Private,
            syn::Visibility::Restricted(r) => {
                let path: Vec<String> = r
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                match path.as_slice() {
                    [one] if one == "crate" => Vis::Crate,
                    [one] if one == "self" => Vis::Private,
                    [one] if one == "super" => Vis::Super,
                    _ => Vis::In(path),
                }
            }
        }
    }
}

/// One name a `use` binds: the name, and the path it names as written. A
/// path with a leading `::` starts with an empty segment.
#[derive(Clone, Debug)]
pub(crate) struct UseName {
    name: String,
    path: Vec<String>,
    vis: Vis,
    cfg: Vec<Cfg>,
}

/// A glob import, `a::b` of `use a::b::*`.
#[derive(Clone, Debug)]
pub(crate) struct Glob {
    path: Vec<String>,
    vis: Vis,
}

/// What kind of type a declaration declares.
#[derive(Clone, Debug)]
pub(crate) enum Kind {
    Struct,
    /// An enum, with its variants.
    Enum(Vec<String>),
    Union,
    Trait,
    /// `type Name<P, ..> = To;`.
    Alias(Vec<String>, RustType),
}

/// A type, trait or alias a module declares.
#[derive(Clone, Debug)]
pub(crate) struct TypeDecl {
    kind: Kind,
    /// How many type parameters it takes.
    generics: usize,
    vis: Vis,
}

/// An `impl` block: its self type, the trait it implements by its last
/// segment, and its type parameters.
#[derive(Clone, Debug)]
pub(crate) struct ImplDecl {
    self_ty: RustType,
    trait_name: Option<String>,
    generics: Vec<String>,
}

/// What one module binds: the file's own module, or an inline `mod`.
#[derive(Clone, Debug, Default)]
pub(crate) struct Scope {
    uses: Vec<UseName>,
    globs: Vec<Glob>,
    types: BTreeMap<String, Vec<TypeDecl>>,
    /// The free functions, constants and statics it declares (the items
    /// of the value namespace a path reads), each with who may name it and
    /// the `cfg` predicates it stands under.
    values: BTreeMap<String, Vec<(Vis, Vec<Cfg>)>>,
    /// It invokes a macro at item level (`make!(go, 4);`), whose expansion
    /// may declare any name: the items it declares are not read here.
    opaque: bool,
    /// The `cfg` predicates each declaration of the module stands under
    /// (one, or one per configuration for a module written twice).
    cfgs: Vec<Vec<Cfg>>,
    /// The modules it declares, inline or in a file of their own.
    mods: BTreeSet<String>,
    impls: Vec<ImplDecl>,
}

/// What a function is a method of.
#[derive(Clone, Debug)]
pub(crate) enum Owner {
    /// A free function.
    None,
    /// A method of an `impl` block with this self type.
    Impl(RustType),
    /// A method a trait declares.
    Trait(String),
}

/// What the written code says of an expression's type, before any name in
/// it is resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Hint {
    /// Nothing written decides it.
    Open,
    /// A type as written: a parameter's, a typed `let`'s, a cast's.
    Written(RustType),
    /// A primitive type a literal has by the language: `str`, `bool`, or a
    /// suffix's.
    Prim(String),
    /// An integer literal without a suffix.
    Integer,
    /// A float literal without a suffix.
    Float,
    /// `&e` or `&mut e`.
    Ref(bool, Box<Hint>),
    /// `*e`.
    Deref(Box<Hint>),
    /// `Self`, the type of the `impl` around the function.
    This,
    /// The value of the function's call of this index.
    Result(usize),
    /// `e?`.
    Try(Box<Hint>),
    /// A struct literal, or a path that is a value: a unit struct or a
    /// variant.
    Ctor(Vec<String>),
    Tuple(Vec<Hint>),
}

/// What a call calls.
#[derive(Clone, Debug)]
pub(crate) enum Callee {
    /// `a::b::f(..)`: the path's segments with any `use` of the body
    /// written out, its text so, and its text as ingest records it.
    Path {
        segments: Vec<String>,
        text: String,
        recorded: String,
        /// `<T as Trait>::f`: the type, and the trait's last segment.
        qself: Option<(RustType, Option<String>)>,
        /// A glob import of an enclosing block may bind the path's head.
        opaque: bool,
    },
    /// `receiver.name(..)`.
    Method { name: String, receiver: Hint },
    /// A call of a name the body binds: a closure or a function a variable
    /// holds, or a function the body declares, which no fact holds.
    Local(String),
}

/// One call of a body, with what the code writes of its arguments.
#[derive(Clone, Debug)]
pub(crate) struct Call {
    callee: Callee,
    args: Vec<Hint>,
}

impl Call {
    /// The call as ingest records it: a path, or `.name` for a method.
    fn recorded(&self) -> String {
        match &self.callee {
            Callee::Path { recorded, .. } => recorded.clone(),
            Callee::Method { name, .. } => format!(".{name}"),
            Callee::Local(name) => name.clone(),
        }
    }

    /// The call with any `use` of the body written out.
    fn text(&self) -> String {
        match &self.callee {
            Callee::Path { text, .. } => text.clone(),
            _ => self.recorded(),
        }
    }
}

/// What name resolution reads of one function.
#[derive(Clone, Debug)]
pub(crate) struct FnNames {
    /// The inline module it stands in, `""` for the file's own.
    module: String,
    /// The `cfg` predicates it stands under: its own, and those of the
    /// `impl` and the inline modules around it.
    cfg: Vec<Cfg>,
    owner: Owner,
    /// The type parameters in scope: its own and its `impl`'s or trait's.
    generics: Vec<String>,
    /// The names its body binds anew: by a `use` or a type it declares.
    rebound: BTreeSet<String>,
    calls: Vec<Call>,
}

/// Where a type stands, and under which `cfg` predicates.
#[derive(Clone, Debug)]
pub(crate) struct Site {
    module: String,
    cfg: Vec<Cfg>,
}

/// What name resolution reads of one source file.
#[derive(Clone, Debug, Default)]
pub(crate) struct FileNames {
    /// The file's own module (`""`) and each inline module in it, by its
    /// path inside the file (`a::b`).
    modules: BTreeMap<String, Scope>,
    /// Each function, by the line and the name of its fact.
    fns: BTreeMap<(usize, String), FnNames>,
    /// Each enum and struct, by the line and the name of its fact.
    types: BTreeMap<(usize, String), Site>,
    /// Each constant, by the line and the name of its identifier.
    consts: BTreeMap<(usize, String), Site>,
}

/// What name resolution reads of a workspace.
#[derive(Clone, Debug, Default)]
pub(crate) struct Names {
    /// Per source file, by its path relative to the workspace root.
    files: BTreeMap<String, FileNames>,
    /// Per package, by its directory: the features its default set enables.
    features: BTreeMap<String, BTreeSet<String>>,
}

/// `syntax` with the names of the workspace at `root` read into it.
pub(crate) fn read(root: &Path, ws: &Workspace, mut syntax: Syntax) -> Result<Syntax, UorustError> {
    let mut names = Names::default();
    for package in &ws.packages {
        let manifest = root.join(&package.dir).join("Cargo.toml");
        names
            .features
            .insert(package.dir.clone(), default_features(&manifest)?);
        for file in &package.files {
            if file.parse_error.is_some() {
                continue;
            }
            let path = root.join(&file.rel);
            let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
            // Ingest parsed the same text: a file that does not parse is
            // already an `unparsable-source` refusal.
            let Ok(parsed) = syn::parse_file(&text) else {
                continue;
            };
            names.files.insert(file.rel.clone(), FileNames::of(&parsed));
        }
    }
    syntax.names = names;
    Ok(syntax)
}

/// The features a package's `default` feature enables, by Cargo's rules
/// (the Cargo book, "Features"): a feature enables the features it lists; an
/// optional dependency is a feature of its own name unless some feature
/// names it `dep:name`; `dep/feature` enables the dependency and `dep?/feature`
/// does not.
fn default_features(manifest: &Path) -> Result<BTreeSet<String>, UorustError> {
    let text = std::fs::read_to_string(manifest).map_err(|e| io(manifest, e))?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|e: toml::de::Error| UorustError::Malformed {
            path: manifest.to_path_buf(),
            reason: e.to_string(),
        })?;
    Ok(enabled_by_default(&value))
}

fn enabled_by_default(manifest: &toml::Value) -> BTreeSet<String> {
    let table: BTreeMap<String, Vec<String>> = manifest
        .get("features")
        .and_then(toml::Value::as_table)
        .map(|features| {
            features
                .iter()
                .map(|(name, enables)| {
                    let enables = enables
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(toml::Value::as_str)
                        .map(str::to_string)
                        .collect();
                    (name.clone(), enables)
                })
                .collect()
        })
        .unwrap_or_default();
    let explicit: BTreeSet<&str> = table
        .values()
        .flatten()
        .filter_map(|entry| entry.strip_prefix("dep:"))
        .collect();
    let mut on = BTreeSet::new();
    let mut todo: Vec<String> = table.get("default").cloned().unwrap_or_default();
    if table.contains_key("default") {
        on.insert("default".to_string());
    }
    while let Some(entry) = todo.pop() {
        if entry.starts_with("dep:") {
            continue;
        }
        let name = match entry.split_once('/') {
            Some((dep, _)) if dep.ends_with('?') || explicit.contains(dep) => continue,
            Some((dep, _)) => dep.to_string(),
            None => entry,
        };
        if on.insert(name.clone()) {
            todo.extend(table.get(&name).cloned().unwrap_or_default());
        }
    }
    on
}

/// The type parameters `generics` declares.
fn type_params(generics: &syn::Generics) -> Vec<String> {
    generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(t) => Some(t.ident.to_string()),
            _ => None,
        })
        .collect()
}

fn idents(path: &syn::Path) -> Vec<String> {
    let mut out: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    if path.leading_colon.is_some() {
        out.insert(0, String::new());
    }
    out
}

fn join(module: &str, name: &str) -> String {
    match (module.is_empty(), name.is_empty()) {
        (true, _) => name.to_string(),
        (_, true) => module.to_string(),
        _ => format!("{module}::{name}"),
    }
}

/// The names a `use` tree binds and the globs it imports, into `scope`.
fn use_tree(
    tree: &syn::UseTree,
    prefix: &mut Vec<String>,
    vis: &Vis,
    cfg: &[Cfg],
    scope: &mut Scope,
) {
    let mut bind = |name: String, path: Vec<String>| {
        if name != "_" {
            scope.uses.push(UseName {
                name,
                path,
                vis: vis.clone(),
                cfg: cfg.to_vec(),
            });
        }
    };
    match tree {
        syn::UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            use_tree(&p.tree, prefix, vis, cfg, scope);
            prefix.pop();
        }
        // `use a::{self}` binds `a`.
        syn::UseTree::Name(n) if n.ident == "self" => {
            if let Some(last) = prefix.last() {
                bind(last.clone(), prefix.clone());
            }
        }
        syn::UseTree::Name(n) => {
            let mut path = prefix.clone();
            path.push(n.ident.to_string());
            bind(n.ident.to_string(), path);
        }
        syn::UseTree::Rename(r) => {
            let mut path = prefix.clone();
            if r.ident != "self" {
                path.push(r.ident.to_string());
            }
            bind(r.rename.to_string(), path);
        }
        syn::UseTree::Glob(_) => scope.globs.push(Glob {
            path: prefix.clone(),
            vis: vis.clone(),
        }),
        syn::UseTree::Group(g) => {
            for item in &g.items {
                use_tree(item, prefix, vis, cfg, scope);
            }
        }
    }
}

fn use_item(u: &syn::ItemUse, cfg: &[Cfg], scope: &mut Scope) {
    let mut prefix = Vec::new();
    if u.leading_colon.is_some() {
        prefix.push(String::new());
    }
    use_tree(&u.tree, &mut prefix, &Vis::read(&u.vis), cfg, scope);
}

impl FileNames {
    fn of(file: &syn::File) -> FileNames {
        let mut out = FileNames::default();
        out.items(&file.items, "", &[]);
        out
    }

    fn items(&mut self, list: &[syn::Item], module: &str, cfg: &[Cfg]) {
        let under = |attrs: &[syn::Attribute]| -> Vec<Cfg> {
            cfg.iter().cloned().chain(Cfg::of(attrs)).collect()
        };
        let mut scope = Scope {
            cfgs: vec![cfg.to_vec()],
            ..Scope::default()
        };
        let declare = |scope: &mut Scope,
                       name: &syn::Ident,
                       kind: Kind,
                       generics: usize,
                       vis: &syn::Visibility| {
            scope
                .types
                .entry(name.to_string())
                .or_default()
                .push(TypeDecl {
                    kind,
                    generics,
                    vis: Vis::read(vis),
                });
        };
        for item in list {
            match item {
                syn::Item::Fn(f) => {
                    scope
                        .values
                        .entry(f.sig.ident.to_string())
                        .or_default()
                        .push((Vis::read(&f.vis), under(&f.attrs)));
                    self.function(
                        &f.sig,
                        Some(&f.block),
                        module,
                        under(&f.attrs),
                        Owner::None,
                        &[],
                    );
                }
                syn::Item::Impl(imp) => {
                    let generics = type_params(&imp.generics);
                    let self_ty = crate::convert::ty(&imp.self_ty);
                    scope.impls.push(ImplDecl {
                        self_ty: self_ty.clone(),
                        trait_name: imp
                            .trait_
                            .as_ref()
                            .and_then(|(path, _)| path.segments.last())
                            .map(|s| s.ident.to_string()),
                        generics: generics.clone(),
                    });
                    let outer = under(&imp.attrs);
                    for member in &imp.items {
                        if let syn::ImplItem::Fn(f) = member {
                            let cfg = outer.iter().cloned().chain(Cfg::of(&f.attrs)).collect();
                            self.function(
                                &f.sig,
                                Some(&f.block),
                                module,
                                cfg,
                                Owner::Impl(self_ty.clone()),
                                &generics,
                            );
                        }
                    }
                }
                syn::Item::Trait(t) => {
                    let generics = type_params(&t.generics);
                    declare(&mut scope, &t.ident, Kind::Trait, generics.len(), &t.vis);
                    let outer = under(&t.attrs);
                    for member in &t.items {
                        if let syn::TraitItem::Fn(f) = member {
                            let cfg = outer.iter().cloned().chain(Cfg::of(&f.attrs)).collect();
                            self.function(
                                &f.sig,
                                f.default.as_ref(),
                                module,
                                cfg,
                                Owner::Trait(t.ident.to_string()),
                                &generics,
                            );
                        }
                    }
                }
                syn::Item::Enum(e) => {
                    let variants = e.variants.iter().map(|v| v.ident.to_string()).collect();
                    let generics = type_params(&e.generics).len();
                    declare(&mut scope, &e.ident, Kind::Enum(variants), generics, &e.vis);
                    self.site(&e.ident, module, under(&e.attrs));
                }
                syn::Item::Struct(s) => {
                    let generics = type_params(&s.generics).len();
                    declare(&mut scope, &s.ident, Kind::Struct, generics, &s.vis);
                    self.site(&s.ident, module, under(&s.attrs));
                }
                syn::Item::Union(u) => {
                    let generics = type_params(&u.generics).len();
                    declare(&mut scope, &u.ident, Kind::Union, generics, &u.vis);
                }
                syn::Item::Type(t) => {
                    let params = type_params(&t.generics);
                    let kind = Kind::Alias(params.clone(), crate::convert::ty(&t.ty));
                    declare(&mut scope, &t.ident, kind, params.len(), &t.vis);
                }
                syn::Item::Use(u) => use_item(u, &under(&u.attrs), &mut scope),
                syn::Item::Const(c) => {
                    scope
                        .values
                        .entry(c.ident.to_string())
                        .or_default()
                        .push((Vis::read(&c.vis), under(&c.attrs)));
                    self.consts.insert(
                        (c.ident.span().start().line, c.ident.to_string()),
                        Site {
                            module: module.to_string(),
                            cfg: under(&c.attrs),
                        },
                    );
                }
                syn::Item::Static(st) => {
                    scope
                        .values
                        .entry(st.ident.to_string())
                        .or_default()
                        .push((Vis::read(&st.vis), under(&st.attrs)));
                }
                // `macro_rules! name { .. }` declares a macro, no item; any
                // other macro at item level expands to items this does not
                // see.
                syn::Item::Macro(m) if m.ident.is_none() => scope.opaque = true,
                syn::Item::ExternCrate(e) => {
                    let name = match &e.rename {
                        Some((_, rename)) => rename.to_string(),
                        None => e.ident.to_string(),
                    };
                    scope.uses.push(UseName {
                        name,
                        path: vec![String::new(), e.ident.to_string()],
                        vis: Vis::read(&e.vis),
                        cfg: under(&e.attrs),
                    });
                }
                syn::Item::Mod(m) => {
                    scope.mods.insert(m.ident.to_string());
                    if let Some((_, inner)) = &m.content {
                        self.items(inner, &join(module, &m.ident.to_string()), &under(&m.attrs));
                    }
                }
                _ => {}
            }
        }
        // An inline module written twice (once per configuration) is one
        // module with the bindings of both.
        let merged = self.modules.entry(module.to_string()).or_default();
        merged.uses.extend(scope.uses);
        merged.globs.extend(scope.globs);
        for (name, decls) in scope.types {
            merged.types.entry(name).or_default().extend(decls);
        }
        for (name, vis) in scope.values {
            merged.values.entry(name).or_default().extend(vis);
        }
        merged.mods.extend(scope.mods);
        merged.impls.extend(scope.impls);
        merged.opaque |= scope.opaque;
        merged.cfgs.extend(scope.cfgs);
    }

    fn site(&mut self, ident: &syn::Ident, module: &str, cfg: Vec<Cfg>) {
        self.types.insert(
            (ident.span().start().line, ident.to_string()),
            Site {
                module: module.to_string(),
                cfg,
            },
        );
    }

    fn function(
        &mut self,
        sig: &syn::Signature,
        block: Option<&syn::Block>,
        module: &str,
        cfg: Vec<Cfg>,
        owner: Owner,
        outer: &[String],
    ) {
        let mut generics = outer.to_vec();
        generics.extend(type_params(&sig.generics));
        let (calls, rebound) = match block {
            Some(block) => Walker::calls(sig, block),
            None => (Vec::new(), BTreeSet::new()),
        };
        self.fns.insert(
            (sig.ident.span().start().line, sig.ident.to_string()),
            FnNames {
                module: module.to_string(),
                cfg,
                owner,
                generics,
                rebound,
                calls,
            },
        );
    }
}

/// A name a body binds.
#[derive(Clone, Debug)]
enum BodyName {
    /// A variable, with what the code writes of its type.
    Value(Hint),
    /// A function declared inside the body, which no fact holds.
    Item,
    /// A name a `use` inside the body binds, with the path it names.
    Use(Vec<String>),
}

/// The names one block of a body binds.
#[derive(Default)]
struct Bindings {
    names: BTreeMap<String, BodyName>,
    /// The block has a glob import, whose names the block does not say.
    glob: bool,
}

/// Reads a body's calls in the scopes of its blocks.
struct Walker {
    scopes: Vec<Bindings>,
    calls: Vec<Call>,
    /// The names a `use` or a type declaration of the body binds.
    rebound: BTreeSet<String>,
}

/// The identifiers a pattern binds.
struct Bound(Vec<String>);

impl<'ast> Visit<'ast> for Bound {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.0.push(node.ident.to_string());
        syn::visit::visit_pat_ident(self, node);
    }

    fn visit_expr(&mut self, _: &'ast syn::Expr) {}
}

impl<'ast> Visit<'ast> for Walker {
    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        self.expr(node);
    }

    fn visit_block(&mut self, node: &'ast syn::Block) {
        self.block(node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        self.mac(node);
    }

    // An item inside a body is not part of the body's own calls.
    fn visit_item(&mut self, _: &'ast syn::Item) {}
}

impl Walker {
    fn calls(sig: &syn::Signature, block: &syn::Block) -> (Vec<Call>, BTreeSet<String>) {
        let mut walker = Walker {
            scopes: vec![Bindings::default()],
            calls: Vec::new(),
            rebound: BTreeSet::new(),
        };
        for input in &sig.inputs {
            match input {
                syn::FnArg::Receiver(r) => {
                    let hint = match &r.kind {
                        syn::ReceiverKind::Value => Hint::This,
                        syn::ReceiverKind::Reference(_, _, mutable) => {
                            Hint::Ref(mutable.is_some(), Box::new(Hint::This))
                        }
                        syn::ReceiverKind::Typed(_, ty) => Hint::Written(crate::convert::ty(ty)),
                        _ => Hint::Open,
                    };
                    walker.bind("self".to_string(), BodyName::Value(hint));
                }
                syn::FnArg::Typed(t) => walker.typed(&t.pat, &t.ty),
            }
        }
        walker.block(block);
        (walker.calls, walker.rebound)
    }

    fn bind(&mut self, name: String, local: BodyName) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.names.insert(name, local);
        }
    }

    /// `pat: ty`: a plain name has the written type; any other pattern
    /// binds names whose types the code does not write.
    fn typed(&mut self, pat: &syn::Pat, ty: &syn::Type) {
        match pat {
            syn::Pat::Ident(i) if i.subpat.is_none() && i.by_ref.is_none() => {
                let hint = Hint::Written(crate::convert::ty(ty));
                self.bind(i.ident.to_string(), BodyName::Value(hint));
            }
            other => self.pattern(other),
        }
    }

    /// A pattern without a written type: each name it binds is open, and
    /// its guard is read.
    fn pattern(&mut self, pat: &syn::Pat) {
        match pat {
            syn::Pat::Guard(g) => {
                self.pattern(&g.pat);
                self.expr(&g.guard);
            }
            syn::Pat::Type(t) => self.typed(&t.pat, &t.ty),
            other => {
                let mut bound = Bound(Vec::new());
                bound.visit_pat(other);
                for name in bound.0 {
                    self.bind(name, BodyName::Value(Hint::Open));
                }
            }
        }
    }

    /// What the innermost scope that binds `name` binds it to, and whether
    /// a block with a glob import lies between.
    fn local(&self, name: &str) -> (Option<&BodyName>, bool) {
        let mut glob = false;
        for scope in self.scopes.iter().rev() {
            if let Some(local) = scope.names.get(name) {
                return (Some(local), glob);
            }
            glob |= scope.glob;
        }
        (None, glob)
    }

    /// A path with the `use` of the body that binds its head written out,
    /// and whether a glob import of the body may bind the head instead.
    fn written_out(&self, segments: Vec<String>) -> (Vec<String>, bool) {
        let Some(head) = segments.first() else {
            return (segments, false);
        };
        match self.local(head) {
            (Some(BodyName::Use(path)), glob) => {
                let mut out = path.clone();
                out.extend(segments.into_iter().skip(1));
                (out, glob)
            }
            (_, glob) => (segments, glob),
        }
    }

    fn push(&mut self, call: Call) -> Hint {
        self.calls.push(call);
        Hint::Result(self.calls.len() - 1)
    }

    fn block(&mut self, block: &syn::Block) -> Hint {
        self.scopes.push(Bindings::default());
        // The items of a block are in scope throughout it.
        for stmt in &block.stmts {
            match stmt {
                syn::Stmt::Item(syn::Item::Fn(f)) => {
                    self.bind(f.sig.ident.to_string(), BodyName::Item);
                }
                syn::Stmt::Item(syn::Item::Use(u)) => {
                    let mut scope = Scope::default();
                    use_item(u, &[], &mut scope);
                    for bound in scope.uses {
                        self.rebound.insert(bound.name.clone());
                        self.bind(bound.name, BodyName::Use(bound.path));
                    }
                    if let (false, Some(bindings)) =
                        (scope.globs.is_empty(), self.scopes.last_mut())
                    {
                        bindings.glob = true;
                    }
                }
                syn::Stmt::Item(syn::Item::Const(c)) => {
                    let hint = Hint::Written(crate::convert::ty(&c.ty));
                    self.bind(c.ident.to_string(), BodyName::Value(hint));
                }
                syn::Stmt::Item(syn::Item::Static(s)) => {
                    let hint = Hint::Written(crate::convert::ty(&s.ty));
                    self.bind(s.ident.to_string(), BodyName::Value(hint));
                }
                syn::Stmt::Item(syn::Item::Struct(syn::ItemStruct { ident, .. }))
                | syn::Stmt::Item(syn::Item::Enum(syn::ItemEnum { ident, .. }))
                | syn::Stmt::Item(syn::Item::Union(syn::ItemUnion { ident, .. }))
                | syn::Stmt::Item(syn::Item::Type(syn::ItemType { ident, .. }))
                | syn::Stmt::Item(syn::Item::Trait(syn::ItemTrait { ident, .. })) => {
                    self.rebound.insert(ident.to_string());
                }
                _ => {}
            }
        }
        let mut last = Hint::Open;
        for stmt in &block.stmts {
            last = Hint::Open;
            match stmt {
                syn::Stmt::Local(l) => {
                    let hint = match &l.init {
                        Some(init) => {
                            let hint = self.expr(&init.expr);
                            if let Some((_, diverge)) = &init.diverge {
                                self.expr(diverge);
                            }
                            hint
                        }
                        None => Hint::Open,
                    };
                    match &l.pat {
                        syn::Pat::Ident(i) if i.subpat.is_none() && i.by_ref.is_none() => {
                            self.bind(i.ident.to_string(), BodyName::Value(hint));
                        }
                        other => self.pattern(other),
                    }
                }
                syn::Stmt::Expr(e, semi) => {
                    let hint = self.expr(e);
                    if semi.is_none() {
                        last = hint;
                    }
                }
                syn::Stmt::Macro(m) => {
                    self.mac(&m.mac);
                }
                syn::Stmt::Item(_) => {}
            }
        }
        self.scopes.pop();
        last
    }

    /// A macro's arguments, where they read as expressions separated by
    /// commas: what ingest reads of a macro too.
    fn mac(&mut self, m: &syn::Macro) -> Hint {
        let args = m.parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated);
        if let Ok(args) = args {
            for arg in &args {
                self.expr(arg);
            }
        }
        Hint::Open
    }

    /// Reads an expression's calls and returns what the code writes of its
    /// type.
    #[allow(clippy::too_many_lines)]
    fn expr(&mut self, e: &syn::Expr) -> Hint {
        match e {
            syn::Expr::Call(c) => {
                let args: Vec<Hint> = c.args.iter().map(|a| self.expr(a)).collect();
                let syn::Expr::Path(p) = &*c.func else {
                    self.expr(&c.func);
                    return Hint::Open;
                };
                let segments = idents(&p.path);
                // A closure or a function a variable holds, or a function
                // the body declares, is no path to a function fact.
                if let ([one], None) = (segments.as_slice(), &p.qself) {
                    if matches!(self.local(one).0, Some(BodyName::Value(_) | BodyName::Item)) {
                        return self.push(Call {
                            callee: Callee::Local(one.clone()),
                            args,
                        });
                    }
                }
                let recorded = crate::convert::path(&p.path).text;
                let (written, opaque) = self.written_out(segments.clone());
                let text = if written == segments {
                    recorded.clone()
                } else {
                    written.join("::")
                };
                let qself = p.qself.as_ref().map(|q| {
                    let of = q
                        .position
                        .checked_sub(1)
                        .and_then(|last| p.path.segments.iter().nth(last))
                        .map(|s| s.ident.to_string());
                    (crate::convert::ty(&q.ty), of)
                });
                self.push(Call {
                    callee: Callee::Path {
                        segments: written,
                        text,
                        recorded,
                        qself,
                        opaque,
                    },
                    args,
                })
            }
            syn::Expr::MethodCall(m) => {
                let receiver = self.expr(&m.receiver);
                let args = m.args.iter().map(|a| self.expr(a)).collect();
                self.push(Call {
                    callee: Callee::Method {
                        name: m.method.to_string(),
                        receiver,
                    },
                    args,
                })
            }
            syn::Expr::Path(p) => {
                if p.qself.is_some() {
                    return Hint::Open;
                }
                let segments = idents(&p.path);
                if let [one] = segments.as_slice() {
                    match self.local(one).0 {
                        Some(BodyName::Value(hint)) => return hint.clone(),
                        Some(BodyName::Item) => return Hint::Open,
                        Some(BodyName::Use(_)) | None => {}
                    }
                }
                match self.written_out(segments) {
                    (written, false) => Hint::Ctor(written),
                    (_, true) => Hint::Open,
                }
            }
            syn::Expr::Lit(l) => match &l.lit {
                syn::Lit::Str(_) => Hint::Ref(false, Box::new(Hint::Prim("str".to_string()))),
                syn::Lit::Bool(_) => Hint::Prim("bool".to_string()),
                syn::Lit::Char(_) => Hint::Prim("char".to_string()),
                syn::Lit::Byte(_) => Hint::Prim("u8".to_string()),
                syn::Lit::Int(i) if i.suffix().is_empty() => Hint::Integer,
                syn::Lit::Int(i) => Hint::Prim(i.suffix().to_string()),
                syn::Lit::Float(f) if f.suffix().is_empty() => Hint::Float,
                syn::Lit::Float(f) => Hint::Prim(f.suffix().to_string()),
                _ => Hint::Open,
            },
            syn::Expr::Reference(r) => {
                Hint::Ref(r.mutability.is_some(), Box::new(self.expr(&r.expr)))
            }
            syn::Expr::Paren(p) => self.expr(&p.expr),
            syn::Expr::Group(g) => self.expr(&g.expr),
            syn::Expr::Unary(u) => {
                let hint = self.expr(&u.expr);
                match u.op {
                    syn::UnOp::Deref(_) => Hint::Deref(Box::new(hint)),
                    // The negation of a literal is a literal of its type.
                    syn::UnOp::Neg(_) => match hint {
                        Hint::Integer | Hint::Float | Hint::Prim(_) => hint,
                        _ => Hint::Open,
                    },
                    _ => Hint::Open,
                }
            }
            syn::Expr::Cast(c) => {
                self.expr(&c.expr);
                Hint::Written(crate::convert::ty(&c.ty))
            }
            syn::Expr::Struct(s) => {
                for field in &s.fields {
                    self.expr(&field.expr);
                }
                if let Some(rest) = &s.rest {
                    self.expr(rest);
                }
                match (&s.qself, self.written_out(idents(&s.path))) {
                    (None, (written, false)) => Hint::Ctor(written),
                    _ => Hint::Open,
                }
            }
            syn::Expr::Try(t) => Hint::Try(Box::new(self.expr(&t.expr))),
            syn::Expr::Tuple(t) => Hint::Tuple(t.elems.iter().map(|x| self.expr(x)).collect()),
            syn::Expr::Block(b) => self.block(&b.block),
            syn::Expr::Unsafe(u) => self.block(&u.block),
            syn::Expr::If(i) => {
                // A `let` of the condition binds its names in the branch.
                self.scopes.push(Bindings::default());
                self.expr(&i.cond);
                self.block(&i.then_branch);
                self.scopes.pop();
                if let Some((_, otherwise)) = &i.else_branch {
                    self.expr(otherwise);
                }
                Hint::Open
            }
            syn::Expr::While(w) => {
                self.scopes.push(Bindings::default());
                self.expr(&w.cond);
                self.block(&w.body);
                self.scopes.pop();
                Hint::Open
            }
            syn::Expr::Let(l) => {
                self.expr(&l.expr);
                self.pattern(&l.pat);
                Hint::Open
            }
            syn::Expr::Match(m) => {
                self.expr(&m.expr);
                for arm in &m.arms {
                    self.scopes.push(Bindings::default());
                    self.pattern(&arm.pat);
                    self.expr(&arm.body);
                    self.scopes.pop();
                }
                Hint::Open
            }
            syn::Expr::ForLoop(f) => {
                self.expr(&f.expr);
                self.scopes.push(Bindings::default());
                self.pattern(&f.pat);
                self.block(&f.body);
                self.scopes.pop();
                Hint::Open
            }
            syn::Expr::Closure(c) => {
                self.scopes.push(Bindings::default());
                for input in &c.inputs {
                    self.pattern(input);
                }
                self.expr(&c.body);
                self.scopes.pop();
                Hint::Open
            }
            syn::Expr::Macro(m) => self.mac(&m.mac),
            other => {
                syn::visit::visit_expr(self, other);
                Hint::Open
            }
        }
    }
}

/// A function of the workspace: package, file and position in the file.
pub(crate) type FnId = (usize, usize, usize);

/// A module of the workspace: package, target and path inside the target.
type ModKey = (usize, usize, String);

/// What the language's rules say a call calls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// These functions of the workspace: one, or the variants of one under
    /// `cfg` predicates no build holds together.
    Resolved(Vec<FnId>),
    /// Nothing the workspace declares as a function: the standard library,
    /// a registry crate, a constructor, a closure.
    Outside,
    /// One of these, and the written code does not decide which.
    Open(Vec<FnId>),
    /// The rules could not follow the call: admission reads it by name.
    Unknown,
}

/// What a type is named by, once its path is resolved.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TypeId {
    /// A type a module of the workspace declares.
    Local(ModKey, String),
    /// A path through a crate outside the workspace.
    Extern(Vec<String>),
    /// A name no module binds: a primitive type or one of the standard
    /// prelude.
    Prelude(String),
    /// A type parameter.
    Param(String),
}

/// A type as far as the written code decides it.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Ty {
    Named(TypeId, Vec<Ty>),
    Ref(bool, Box<Ty>),
    Tuple(Vec<Ty>),
    Slice(Box<Ty>),
    Array(Box<Ty>, String),
    /// An integer literal's: some integer type.
    Integer,
    /// A float literal's: some float type.
    Float,
    Open,
}

/// Whether two types are one type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Match {
    Same,
    Different,
    /// The written code does not decide it.
    Unknown,
    /// A literal against a type its kind of literal can have.
    Literal,
}

/// The primitive integer types (the Rust reference, "Numeric types").
const INTEGERS: [&str; 12] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
];

/// The primitive float types.
const FLOATS: [&str; 2] = ["f32", "f64"];

/// The types an unsuffixed literal falls back to when nothing else decides
/// it (the Rust reference, "Literal expressions").
const INTEGER_FALLBACK: &str = "i32";
const FLOAT_FALLBACK: &str = "f64";

/// The crates every build has without a manifest naming them.
const SYSROOT: [&str; 5] = ["std", "core", "alloc", "proc_macro", "test"];

/// What a path names.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Found {
    Module(ModKey),
    /// A type a module declares.
    Type(ModKey, String),
    /// A free function, constant or static a module declares: an item of
    /// the value namespace.
    Fn(ModKey, String),
    /// An item of a type: `Type::item`.
    Assoc(Box<Found>, Vec<String>),
    /// A path through a crate outside the workspace.
    Extern(Vec<String>),
    /// A name no module binds, in a module without a glob of an outside
    /// crate: a primitive or a name of the standard prelude.
    Prelude(String),
    /// Something outside the workspace that a glob of an outside crate may
    /// bind: no module of the workspace declares it.
    Outside,
    /// A module of the workspace that binds nothing of the name.
    Absent,
    /// The path cannot be followed.
    Unknown,
}

/// Where a type is read: the module, the type parameters in scope and
/// what `Self` is.
struct Reading<'a> {
    module: ModKey,
    generics: &'a [String],
    this: Option<Ty>,
}

/// Rust's name resolution over a workspace's facts.
pub(crate) struct Resolver<'a> {
    ws: &'a Workspace,
    syntax: &'a Syntax,
    /// Each module: the file that holds it and its path inside the file.
    modules: BTreeMap<ModKey, (usize, String)>,
    /// The free functions by module and name.
    free: BTreeMap<(ModKey, String), Vec<FnId>>,
    /// The methods by name.
    methods: BTreeMap<String, Vec<FnId>>,
    /// The type each method is a method of.
    owners: BTreeMap<FnId, TypeId>,
    /// Each trait a type implements: the type, and the trait's name.
    implemented: BTreeSet<(TypeId, String)>,
    /// The workspace's libraries by crate name: package and target.
    libs: BTreeMap<String, (usize, usize)>,
    /// The crates outside the workspace a path can start with.
    outside: BTreeSet<String>,
    /// The registry crates among them.
    registry: BTreeSet<String>,
    /// Each package's projected configuration.
    configs: Vec<Config>,
    /// How many aliases the workspace declares: no chain of aliases that
    /// does not repeat one is longer.
    aliases: usize,
}

fn parent(module: &str) -> Option<String> {
    if module.is_empty() {
        return None;
    }
    Some(match module.rsplit_once("::") {
        Some((up, _)) => up.to_string(),
        None => String::new(),
    })
}

/// Whether `module` is `ancestor` or lies inside it.
fn within(module: &str, ancestor: &str) -> bool {
    ancestor.is_empty() || module == ancestor || module.starts_with(&format!("{ancestor}::"))
}

/// The type with each parameter of `params` replaced by its argument.
fn substituted(ty: &Ty, params: &[String], args: &[Ty]) -> Ty {
    let inner = |t: &Ty| Box::new(substituted(t, params, args));
    match ty {
        Ty::Named(TypeId::Param(p), own) if own.is_empty() => {
            match params.iter().position(|q| q == p) {
                Some(i) => args.get(i).cloned().unwrap_or(Ty::Open),
                None => ty.clone(),
            }
        }
        Ty::Named(id, own) => Ty::Named(
            id.clone(),
            own.iter().map(|t| substituted(t, params, args)).collect(),
        ),
        Ty::Ref(mutable, t) => Ty::Ref(*mutable, inner(t)),
        Ty::Tuple(ts) => Ty::Tuple(ts.iter().map(|t| substituted(t, params, args)).collect()),
        Ty::Slice(t) => Ty::Slice(inner(t)),
        Ty::Array(t, n) => Ty::Array(inner(t), n.clone()),
        Ty::Integer | Ty::Float | Ty::Open => ty.clone(),
    }
}

/// Whether each pair is one type: the same when all are, different when
/// one is.
fn all_same(pairs: impl Iterator<Item = Match>) -> Match {
    let mut out = Match::Same;
    for one in pairs {
        match one {
            Match::Different => return Match::Different,
            Match::Same => {}
            Match::Unknown | Match::Literal => out = Match::Unknown,
        }
    }
    out
}

/// Whether an argument of type `arg` is a parameter of type `param`.
fn compare(arg: &Ty, param: &Ty) -> Match {
    // A nominal type of the workspace, a primitive or a prelude type is
    // not a reference, a tuple, a slice or an array; a type of an outside
    // crate may be an alias of one.
    let nominal = |id: &TypeId| matches!(id, TypeId::Local(..) | TypeId::Prelude(_));
    match (arg, param) {
        (Ty::Open, _) | (_, Ty::Open) => Match::Unknown,
        (Ty::Named(TypeId::Param(_), _), _) | (_, Ty::Named(TypeId::Param(_), _)) => Match::Unknown,
        (Ty::Integer, Ty::Named(TypeId::Prelude(p), _)) if INTEGERS.contains(&p.as_str()) => {
            Match::Literal
        }
        (Ty::Float, Ty::Named(TypeId::Prelude(p), _)) if FLOATS.contains(&p.as_str()) => {
            Match::Literal
        }
        (Ty::Integer | Ty::Float, Ty::Named(id, _)) if !nominal(id) => Match::Unknown,
        (Ty::Integer | Ty::Float, _) => Match::Different,
        (_, Ty::Integer | Ty::Float) => Match::Unknown,
        (Ty::Named(a, xs), Ty::Named(b, ys)) => {
            let args = || {
                if xs.len() == ys.len() {
                    all_same(xs.iter().zip(ys).map(|(x, y)| compare(x, y)))
                } else {
                    Match::Unknown
                }
            };
            match (a, b) {
                _ if a == b => args(),
                (TypeId::Local(..), _) | (_, TypeId::Local(..)) => Match::Different,
                (TypeId::Prelude(_), TypeId::Prelude(_)) => Match::Different,
                // One type of an outside crate can have two paths.
                _ => Match::Unknown,
            }
        }
        (Ty::Ref(m, x), Ty::Ref(n, y)) => {
            if m == n {
                compare(x, y)
            } else {
                Match::Different
            }
        }
        (Ty::Tuple(xs), Ty::Tuple(ys)) => {
            if xs.len() == ys.len() {
                all_same(xs.iter().zip(ys).map(|(x, y)| compare(x, y)))
            } else {
                Match::Different
            }
        }
        (Ty::Slice(x), Ty::Slice(y)) => compare(x, y),
        (Ty::Array(x, m), Ty::Array(y, n)) => {
            if m == n {
                compare(x, y)
            } else {
                Match::Unknown
            }
        }
        (Ty::Named(id, _), _) | (_, Ty::Named(id, _)) => {
            if nominal(id) {
                Match::Different
            } else {
                Match::Unknown
            }
        }
        _ => Match::Different,
    }
}

impl<'a> Resolver<'a> {
    pub(crate) fn new(ws: &'a Workspace, syntax: &'a Syntax) -> Resolver<'a> {
        let mut resolver = Resolver {
            ws,
            syntax,
            modules: BTreeMap::new(),
            free: BTreeMap::new(),
            methods: BTreeMap::new(),
            owners: BTreeMap::new(),
            implemented: BTreeSet::new(),
            libs: BTreeMap::new(),
            outside: SYSROOT.iter().map(|s| (*s).to_string()).collect(),
            registry: BTreeSet::new(),
            configs: Vec::new(),
            aliases: 0,
        };
        let names = &syntax.names;
        for (pkg, package) in ws.packages.iter().enumerate() {
            let features = names
                .features
                .get(&package.dir)
                .cloned()
                .unwrap_or_default();
            resolver.configs.push(Config::dump(features));
            for (t, target) in package.targets.iter().enumerate() {
                if target.kind == TargetKind::Lib {
                    resolver
                        .libs
                        .insert(target.name.replace('-', "_"), (pkg, t));
                }
            }
            for dep in package.deps.iter().filter(|d| d.path.is_none()) {
                resolver.registry.insert(dep.name.replace('-', "_"));
            }
            for (fi, file) in package.files.iter().enumerate() {
                let Some(read) = names.files.get(&file.rel) else {
                    continue;
                };
                for (inline, scope) in &read.modules {
                    let key = (pkg, file.target, join(&file.module, inline));
                    resolver.modules.entry(key).or_insert((fi, inline.clone()));
                    resolver.aliases += scope
                        .types
                        .values()
                        .flatten()
                        .filter(|d| matches!(d.kind, Kind::Alias(..)))
                        .count();
                }
            }
        }
        // A registry crate named like a library of the workspace is the
        // workspace's: a path dependency, not a registry one.
        let libs: Vec<String> = resolver.libs.keys().cloned().collect();
        for lib in libs {
            resolver.registry.remove(&lib);
        }
        resolver.outside.extend(resolver.registry.iter().cloned());
        for (pkg, package) in ws.packages.iter().enumerate() {
            for (fi, file) in package.files.iter().enumerate() {
                for (idx, fact) in file.fns.iter().enumerate() {
                    let f = (pkg, fi, idx);
                    if fact.owner.is_some() {
                        resolver
                            .methods
                            .entry(fact.name.clone())
                            .or_default()
                            .push(f);
                    } else {
                        let key = (resolver.module(f), fact.name.clone());
                        resolver.free.entry(key).or_default().push(f);
                    }
                }
            }
        }
        let methods: Vec<FnId> = resolver.methods.values().flatten().copied().collect();
        for f in methods {
            if let Some(owner) = resolver.owner(f) {
                resolver.owners.insert(f, owner);
            }
        }
        let modules: Vec<ModKey> = resolver.modules.keys().cloned().collect();
        for module in modules {
            let Some(scope) = resolver.scope(&module) else {
                continue;
            };
            for imp in &scope.impls {
                let Some(of) = &imp.trait_name else {
                    continue;
                };
                let reading = Reading {
                    module: module.clone(),
                    generics: &imp.generics,
                    this: None,
                };
                if let Some(id) = head(&resolver.ty(&reading, &imp.self_ty, resolver.aliases)) {
                    resolver.implemented.insert((id, of.clone()));
                }
            }
        }
        resolver
    }

    fn file(&self, f: FnId) -> &'a SourceFile {
        &self.ws.packages[f.0].files[f.1]
    }

    fn fact(&self, f: FnId) -> &'a FnFact {
        &self.file(f).fns[f.2]
    }

    fn names(&self, f: FnId) -> Option<&'a FnNames> {
        let file = self.file(f);
        let fact = &file.fns[f.2];
        let read = self.syntax.names.files.get(&file.rel)?;
        read.fns.get(&(fact.span.line, fact.name.clone()))
    }

    /// The module a function stands in.
    fn module(&self, f: FnId) -> ModKey {
        let file = self.file(f);
        let inline = self.names(f).map(|n| n.module.as_str()).unwrap_or("");
        (f.0, file.target, join(&file.module, inline))
    }

    fn scope(&self, module: &ModKey) -> Option<&'a Scope> {
        let (fi, inline) = self.modules.get(module)?;
        let file = &self.ws.packages[module.0].files[*fi];
        self.syntax.names.files.get(&file.rel)?.modules.get(inline)
    }

    fn signature(&self, f: FnId) -> &'a crate::ast::Signature {
        &self.syntax.function(&self.file(f).rel, f.2).signature
    }

    fn cfg(&self, f: FnId) -> &'a [Cfg] {
        self.names(f).map(|n| n.cfg.as_slice()).unwrap_or(&[])
    }

    /// The type a method is a method of: the `impl`'s self type without
    /// its references, or the trait that declares it.
    fn owner(&self, f: FnId) -> Option<TypeId> {
        let names = self.names(f)?;
        let module = self.module(f);
        match &names.owner {
            Owner::None => None,
            Owner::Trait(name) => Some(TypeId::Local(module, name.clone())),
            Owner::Impl(self_ty) => {
                let reading = Reading {
                    module,
                    generics: &names.generics,
                    this: None,
                };
                head(&self.ty(&reading, self_ty, self.aliases))
            }
        }
    }

    /// Where a function's types are read.
    fn reading(&self, f: FnId) -> Reading<'a> {
        let module = self.module(f);
        let Some(names) = self.names(f) else {
            return Reading {
                module,
                generics: &[],
                this: None,
            };
        };
        let this = match &names.owner {
            Owner::Impl(self_ty) => {
                let outer = Reading {
                    module: module.clone(),
                    generics: &names.generics,
                    this: None,
                };
                Some(self.ty(&outer, self_ty, self.aliases))
            }
            // In a trait's own methods `Self` is read as the trait: what
            // it declares is what `Self` has.
            Owner::Trait(name) => Some(Ty::Named(
                TypeId::Local(module.clone(), name.clone()),
                Vec::new(),
            )),
            Owner::None => None,
        };
        Reading {
            module,
            generics: &names.generics,
            this,
        }
    }

    /// Whether an item declared in `declared` with `vis` can be named
    /// from `from`.
    fn visible(vis: &Vis, declared: &ModKey, from: &ModKey) -> bool {
        if *vis == Vis::Public {
            return true;
        }
        if (declared.0, declared.1) != (from.0, from.1) {
            return false;
        }
        match vis {
            Vis::Public | Vis::Crate => true,
            Vis::Private => within(&from.2, &declared.2),
            Vis::Super => within(&from.2, &parent(&declared.2).unwrap_or_default()),
            Vis::In(path) => {
                let mut at = declared.2.clone();
                let mut rest = path.as_slice();
                match rest.first().map(String::as_str) {
                    Some("crate") => {
                        at = String::new();
                        rest = &rest[1..];
                    }
                    Some("self") => rest = &rest[1..],
                    _ => {}
                }
                while rest.first().is_some_and(|s| s == "super") {
                    at = parent(&at).unwrap_or_default();
                    rest = &rest[1..];
                }
                within(&from.2, &join(&at, &rest.join("::")))
            }
        }
    }

    /// A crate a path names by its first segment.
    fn krate(&self, name: &str) -> Found {
        if let Some(&(pkg, target)) = self.libs.get(name) {
            return Found::Module((pkg, target, String::new()));
        }
        if self.outside.contains(name) {
            return Found::Extern(vec![name.to_string()]);
        }
        Found::Unknown
    }

    /// What `name` names in the scope of module `x`, read from `from`:
    /// what `x` declares, then what its `use` binds, then what its globs
    /// import. `None` when the module binds nothing of the name.
    fn named(
        &self,
        x: &ModKey,
        from: &ModKey,
        name: &str,
        value: bool,
        seen: &mut Vec<(ModKey, String, bool)>,
    ) -> Option<Found> {
        let key = (x.clone(), name.to_string(), value);
        // An import that leads back to itself names nothing.
        if seen.contains(&key) {
            return None;
        }
        seen.push(key);
        let found = self.named_in(x, from, name, value, seen);
        seen.pop();
        found
    }

    fn named_in(
        &self,
        x: &ModKey,
        from: &ModKey,
        name: &str,
        value: bool,
        seen: &mut Vec<(ModKey, String, bool)>,
    ) -> Option<Found> {
        let Some(scope) = self.scope(x) else {
            return Some(Found::Unknown);
        };
        let types = scope.types.get(name).map(Vec::as_slice).unwrap_or(&[]);
        let config = &self.configs[x.0];
        // A value the projected configuration compiles out is no item of
        // the build: a reader the configuration compiles reads the name as
        // what else binds it there (a glob's item, `r07`), and as that
        // value only where nothing else does. A reader compiled out itself
        // (a `#[cfg(test)]` module) reads it as the build that compiles
        // both does.
        let mut compiled_out = false;
        if value {
            let fns = scope.values.get(name).map(Vec::as_slice).unwrap_or(&[]);
            let visible: Vec<&Vec<Cfg>> = fns
                .iter()
                .filter(|(vis, _)| Self::visible(vis, x, from))
                .map(|(_, cfg)| cfg)
                .collect();
            let reader_built = self.scope(from).is_none_or(|reader| {
                reader.cfgs.is_empty() || reader.cfgs.iter().any(|c| hold(c, config) != Some(false))
            });
            if visible
                .iter()
                .any(|cfg| !reader_built || hold(cfg, config) != Some(false))
            {
                return Some(Found::Fn(x.clone(), name.to_string()));
            }
            compiled_out = !visible.is_empty();
            // A struct's name is a value too: its constructor.
            let ctor =
                |d: &TypeDecl| matches!(d.kind, Kind::Struct) && Self::visible(&d.vis, x, from);
            if types.iter().any(ctor) {
                return Some(Found::Type(x.clone(), name.to_string()));
            }
        } else {
            if scope.mods.contains(name) {
                let child = (x.0, x.1, join(&x.2, name));
                return Some(if self.modules.contains_key(&child) {
                    Found::Module(child)
                } else {
                    Found::Unknown
                });
            }
            if types.iter().any(|d| Self::visible(&d.vis, x, from)) {
                return Some(Found::Type(x.clone(), name.to_string()));
            }
        }
        let mut found: Vec<Found> = Vec::new();
        let mut add = |one: Found| {
            if !found.contains(&one) {
                found.push(one);
            }
        };
        // A binding the projected configuration compiles out is read only
        // when every binding of the name is.
        let bound: Vec<&UseName> = scope
            .uses
            .iter()
            .filter(|u| u.name == name && Self::visible(&u.vis, x, from))
            .collect();
        let live: Vec<&UseName> = bound
            .iter()
            .copied()
            .filter(|u| hold(&u.cfg, config) != Some(false))
            .collect();
        for binding in if live.is_empty() { &bound } else { &live } {
            match self.path(x, &binding.path, value, seen) {
                // The binding names nothing in this namespace.
                Found::Absent => {}
                one => add(one),
            }
        }
        match found.len() {
            0 => {}
            1 => return found.pop(),
            _ => return Some(Found::Unknown),
        }
        // An item a macro invocation of the scope expands to stands before
        // every glob's, and this does not read which names those are.
        if value && scope.opaque {
            return Some(Found::Unknown);
        }
        let mut outside = false;
        let mut unknown = false;
        for glob in scope
            .globs
            .iter()
            .filter(|g| Self::visible(&g.vis, x, from))
        {
            match self.path(x, &glob.path, false, seen) {
                Found::Module(y) => match self.named(&y, x, name, value, seen) {
                    None | Some(Found::Absent) => {}
                    Some(Found::Outside) => outside = true,
                    Some(one) => {
                        if !found.contains(&one) {
                            found.push(one);
                        }
                    }
                },
                // An enum's variants, or an item's: no module, no function.
                Found::Type(..) | Found::Assoc(..) => {}
                Found::Extern(_) | Found::Outside => outside = true,
                _ => unknown = true,
            }
        }
        match found.len() {
            0 if unknown => Some(Found::Unknown),
            0 if outside => Some(Found::Outside),
            0 if compiled_out => Some(Found::Fn(x.clone(), name.to_string())),
            0 => None,
            1 => found.pop(),
            _ => Some(Found::Unknown),
        }
    }

    /// The item of its own crate a bare name of the value namespace
    /// (`go` of `go()`, `N` of `N + 1`) names where module `module` of
    /// target `target` of package `pkg` reads it: the module that declares
    /// the item and the item's name, which differs from `name` through a
    /// renaming `use`. Rust's own order decides it: an item the module
    /// declares (compiled in by the projected configuration), a `use`
    /// binding, an item a macro invocation of the module expands to, a
    /// glob's item, the preludes. `None` where the name reads as anything
    /// else (a crate outside the workspace, a variant a glob imports, a
    /// name of the standard prelude) or where this does not follow it (a
    /// macro's expansion, an unread module): a reader then binds none of
    /// the crate's items to it.
    pub(crate) fn bare_value(
        &self,
        (pkg, target, module): (usize, usize, &str),
        name: &str,
    ) -> Option<(String, String)> {
        let at = (pkg, target, module.to_string());
        match self.path(&at, &[name.to_string()], true, &mut Vec::new()) {
            Found::Fn((p, t, declared), item) if (p, t) == (pkg, target) => Some((declared, item)),
            _ => None,
        }
    }

    /// What a path names, read in module `m`: the last segment as a value
    /// (a function) when `value`, every other as a module or a type.
    fn path(
        &self,
        m: &ModKey,
        segments: &[String],
        value: bool,
        seen: &mut Vec<(ModKey, String, bool)>,
    ) -> Found {
        let Some((first, mut rest)) = segments.split_first() else {
            return Found::Unknown;
        };
        let mut at = match first.as_str() {
            // `::krate::..`.
            "" => {
                let Some((krate, tail)) = rest.split_first() else {
                    return Found::Unknown;
                };
                rest = tail;
                self.krate(krate)
            }
            "crate" => Found::Module((m.0, m.1, String::new())),
            "self" => Found::Module(m.clone()),
            "super" => {
                let Some(mut module) = parent(&m.2) else {
                    return Found::Unknown;
                };
                while let Some((next, tail)) = rest.split_first() {
                    if next != "super" {
                        break;
                    }
                    let Some(up) = parent(&module) else {
                        return Found::Unknown;
                    };
                    module = up;
                    rest = tail;
                }
                Found::Module((m.0, m.1, module))
            }
            "Self" => return Found::Unknown,
            name => match self.named(m, m, name, value && rest.is_empty(), seen) {
                Some(found) => found,
                // Nothing the module binds: a crate, or a name of the
                // prelude.
                None => match self.krate(name) {
                    Found::Unknown => Found::Prelude(name.to_string()),
                    krate => krate,
                },
            },
        };
        while let Some((next, tail)) = rest.split_first() {
            at = match at {
                Found::Module(x) => match self.named(&x, m, next, value && tail.is_empty(), seen) {
                    Some(found) => found,
                    None => Found::Absent,
                },
                Found::Type(..) | Found::Prelude(_) => {
                    return Found::Assoc(Box::new(at), rest.to_vec());
                }
                Found::Extern(mut path) => {
                    path.push(next.clone());
                    Found::Extern(path)
                }
                Found::Outside => Found::Outside,
                Found::Absent => return Found::Absent,
                Found::Fn(..) | Found::Assoc(..) | Found::Unknown => return Found::Unknown,
            };
            rest = tail;
        }
        at
    }

    /// The type a declaration of the workspace declares: itself, or what
    /// it stands for when it is an alias.
    fn declared(&self, module: ModKey, name: String, args: Vec<Ty>, depth: usize) -> Ty {
        let decls = self
            .scope(&module)
            .and_then(|s| s.types.get(&name))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        match decls {
            [TypeDecl {
                kind: Kind::Alias(params, to),
                ..
            }] => {
                if depth == 0 || params.len() != args.len() {
                    return Ty::Open;
                }
                let reading = Reading {
                    module,
                    generics: params,
                    this: None,
                };
                substituted(&self.ty(&reading, to, depth - 1), params, &args)
            }
            // An alias under one configuration and something else under
            // another is not read.
            many if many.iter().any(|d| matches!(d.kind, Kind::Alias(..))) => Ty::Open,
            _ => Ty::Named(TypeId::Local(module, name), args),
        }
    }

    /// A written type, with each name resolved where it is read.
    fn ty(&self, reading: &Reading<'_>, written: &RustType, depth: usize) -> Ty {
        let inner = |t: &RustType| Box::new(self.ty(reading, t, depth));
        match &written.kind {
            TypeKind::Reference { mutable, elem } => Ty::Ref(*mutable, inner(elem)),
            TypeKind::Slice(t) => Ty::Slice(inner(t)),
            TypeKind::Array(t, n) => Ty::Array(inner(t), n.clone()),
            TypeKind::Paren(t) => self.ty(reading, t, depth),
            TypeKind::Tuple(ts) => {
                Ty::Tuple(ts.iter().map(|t| self.ty(reading, t, depth)).collect())
            }
            TypeKind::Path { qself: false, path } => {
                let Some((last, before)) = path.segments.split_last() else {
                    return Ty::Open;
                };
                if before.iter().any(|s| s.arguments != PathArguments::None) {
                    return Ty::Open;
                }
                let mut args = Vec::new();
                match &last.arguments {
                    PathArguments::None => {}
                    PathArguments::AngleBracketed(list) => {
                        for arg in list {
                            match arg {
                                GenericArgument::Type(t) => args.push(self.ty(reading, t, depth)),
                                // A lifetime says how long a borrow lives,
                                // not which type it is.
                                GenericArgument::Other(text) if text.starts_with('\'') => {}
                                GenericArgument::Other(_) => args.push(Ty::Open),
                            }
                        }
                    }
                    PathArguments::Parenthesized => return Ty::Open,
                }
                let segments: Vec<String> = path.segments.iter().map(|s| s.ident.clone()).collect();
                if let [one] = segments.as_slice() {
                    if one == "Self" {
                        return reading.this.clone().unwrap_or(Ty::Open);
                    }
                    if reading.generics.contains(one) {
                        return Ty::Named(TypeId::Param(one.clone()), args);
                    }
                }
                match self.path(&reading.module, &segments, false, &mut Vec::new()) {
                    Found::Type(module, name) => self.declared(module, name, args, depth),
                    Found::Prelude(name) => Ty::Named(TypeId::Prelude(name), args),
                    Found::Extern(path) => Ty::Named(TypeId::Extern(path), args),
                    _ => Ty::Open,
                }
            }
            _ => Ty::Open,
        }
    }

    /// The type a constructor path constructs: a struct without type
    /// parameters, or a variant of an enum without them.
    fn constructed(&self, reading: &Reading<'_>, segments: &[String]) -> Ty {
        let plain = |module: &ModKey, name: &str, variant: Option<&String>| -> bool {
            let decls = self
                .scope(module)
                .and_then(|s| s.types.get(name))
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            match (decls, variant) {
                ([one], None) => one.generics == 0 && matches!(one.kind, Kind::Struct),
                ([one], Some(v)) => {
                    one.generics == 0 && matches!(&one.kind, Kind::Enum(vs) if vs.contains(v))
                }
                _ => false,
            }
        };
        if let Some((first, rest)) = segments.split_first() {
            if first == "Self" {
                let Some(Ty::Named(TypeId::Local(module, name), args)) = &reading.this else {
                    return Ty::Open;
                };
                let known = match rest {
                    [] => args.is_empty(),
                    [variant] => plain(module, name, Some(variant)),
                    _ => false,
                };
                return if known {
                    Ty::Named(TypeId::Local(module.clone(), name.clone()), Vec::new())
                } else {
                    Ty::Open
                };
            }
        }
        match self.path(&reading.module, segments, false, &mut Vec::new()) {
            Found::Type(module, name) if plain(&module, &name, None) => {
                Ty::Named(TypeId::Local(module, name), Vec::new())
            }
            Found::Assoc(of, rest) => match (*of, rest.as_slice()) {
                (Found::Type(module, name), [variant]) if plain(&module, &name, Some(variant)) => {
                    Ty::Named(TypeId::Local(module, name), Vec::new())
                }
                _ => Ty::Open,
            },
            _ => Ty::Open,
        }
    }

    /// The type a function's result has, read where it is declared.
    fn returns(&self, f: FnId) -> Ty {
        let signature = self.signature(f);
        if signature.asyncness {
            return Ty::Open;
        }
        match &signature.output {
            None => Ty::Tuple(Vec::new()),
            Some(out) => self.ty(&self.reading(f), out, self.aliases),
        }
    }

    /// The types of a function's parameters after its receiver.
    fn params(&self, f: FnId) -> Vec<Ty> {
        let reading = self.reading(f);
        self.signature(f)
            .inputs
            .iter()
            .filter_map(|input| match input {
                FnArg::Typed(p) => Some(self.ty(&reading, &p.ty, self.aliases)),
                FnArg::Receiver { .. } => None,
            })
            .collect()
    }

    fn takes_self(&self, f: FnId) -> bool {
        matches!(
            self.signature(f).inputs.first(),
            Some(FnArg::Receiver { .. })
        )
    }

    /// The functions of `candidates` when they are one function: a single
    /// one, or the variants of one under `cfg` predicates no build holds
    /// together. Of variants, those the projected configuration compiles
    /// out are left out when it compiles another in.
    fn one(&self, candidates: &[FnId]) -> Option<Vec<FnId>> {
        match candidates {
            [] => None,
            [only] => Some(vec![*only]),
            many => {
                let apart = many.iter().enumerate().all(|(i, a)| {
                    many[i + 1..]
                        .iter()
                        .all(|b| exclusive(self.cfg(*a), self.cfg(*b)))
                });
                if !apart {
                    return None;
                }
                let live: Vec<FnId> = many
                    .iter()
                    .copied()
                    .filter(|c| hold(self.cfg(*c), &self.configs[c.0]) != Some(false))
                    .collect();
                Some(if live.is_empty() { many.to_vec() } else { live })
            }
        }
    }

    /// Which of several functions of one name on one type a call calls.
    /// An inherent function comes before a trait's; implementations of one
    /// trait at different arguments are told apart by the types of the
    /// call's arguments, of which at most one can be the same by the
    /// trait's coherence.
    fn select(
        &self,
        caller: FnId,
        mut candidates: Vec<FnId>,
        args: &[Ty],
        path_call: bool,
        local: bool,
    ) -> Outcome {
        // A function no build holds together with the caller is not called.
        let own = self.cfg(caller);
        candidates.retain(|c| !exclusive(own, self.cfg(*c)));
        if candidates.is_empty() {
            return Outcome::Outside;
        }
        let inherent: Vec<FnId> = candidates
            .iter()
            .copied()
            .filter(|c| self.fact(*c).implements.is_none() && !self.fact(*c).in_trait)
            .collect();
        if !inherent.is_empty() {
            if let Some(one) = self.one(&inherent) {
                return Outcome::Resolved(one);
            }
            candidates = inherent;
        }
        // How each candidate's parameters take the arguments.
        let fit = |c: FnId| -> Match {
            let params = self.params(c);
            // `Type::method(receiver, ..)` passes the receiver first.
            let skip = usize::from(path_call && self.takes_self(c));
            if args.len() != params.len() + skip {
                return Match::Different;
            }
            let pairs: Vec<Match> = args[skip..]
                .iter()
                .zip(&params)
                .map(|(arg, param)| compare(arg, param))
                .collect();
            if pairs.contains(&Match::Different) {
                Match::Different
            } else if pairs.contains(&Match::Unknown) {
                Match::Unknown
            } else if pairs.contains(&Match::Literal) {
                Match::Literal
            } else {
                Match::Same
            }
        };
        let fits: Vec<(FnId, Match)> = candidates.iter().map(|c| (*c, fit(*c))).collect();
        let of = |m: Match| -> Vec<FnId> {
            fits.iter()
                .filter(|(_, x)| *x == m)
                .map(|(c, _)| *c)
                .collect()
        };
        if let Some(one) = self.one(&candidates) {
            // The only function of the name the workspace gives the type.
            // A trait's implementation whose parameters cannot take the
            // arguments is not the one called: the trait has another
            // implementation outside the workspace. For a type of the
            // workspace every other implementation is the standard
            // library's blanket one; for a type outside it the call is
            // this one only when the arguments' types say so.
            let fitting = |m: Match| one.iter().all(|c| of(m).contains(c));
            let traits = one.iter().all(|c| self.fact(*c).implements.is_some());
            return if traits && fitting(Match::Different) {
                Outcome::Outside
            } else if local || !traits || fitting(Match::Same) {
                Outcome::Resolved(one)
            } else {
                Outcome::Open(one)
            };
        }
        let same = of(Match::Same);
        if let Some(one) = self.one(&same) {
            return Outcome::Resolved(one);
        }
        let open: Vec<FnId> = fits
            .iter()
            .filter(|(_, m)| *m != Match::Different)
            .map(|(c, _)| *c)
            .collect();
        if open.is_empty() {
            return Outcome::Outside;
        }
        // An unsuffixed literal is of the one implementation whose
        // parameter its kind of literal can be, and of the fallback type's
        // when several can. Only a type of the workspace has all of its
        // implementations in the workspace.
        let literal = of(Match::Literal);
        if local && same.is_empty() && literal.len() == open.len() {
            if let Some(one) = self.one(&literal) {
                return Outcome::Resolved(one);
            }
            let fallback: Vec<FnId> = literal
                .iter()
                .copied()
                .filter(|c| {
                    let skip = usize::from(path_call && self.takes_self(*c));
                    args[skip..]
                        .iter()
                        .zip(self.params(*c))
                        .all(|(arg, param)| {
                            let want = match arg {
                                Ty::Integer => INTEGER_FALLBACK,
                                Ty::Float => FLOAT_FALLBACK,
                                _ => return true,
                            };
                            matches!(param, Ty::Named(TypeId::Prelude(p), _) if p == want)
                        })
                })
                .collect();
            if let Some(one) = self.one(&fallback) {
                return Outcome::Resolved(one);
            }
        }
        Outcome::Open(open)
    }

    /// A function `name` of the type `id`: its own, or the one a trait it
    /// implements declares with a body.
    fn assoc(
        &self,
        caller: FnId,
        id: &TypeId,
        name: &str,
        args: &[Ty],
        path_call: bool,
    ) -> Outcome {
        let named = self.methods.get(name).map(Vec::as_slice).unwrap_or(&[]);
        let own: Vec<FnId> = named
            .iter()
            .copied()
            .filter(|c| self.owners.get(c) == Some(id))
            .collect();
        if !own.is_empty() {
            let local = matches!(id, TypeId::Local(..));
            return self.select(caller, own, args, path_call, local);
        }
        let declared: Vec<FnId> = named
            .iter()
            .copied()
            .filter(|c| self.fact(*c).in_trait)
            .filter(|c| match (self.owners.get(c), &self.fact(*c).owner) {
                (Some(_), Some(of)) => self.implemented.contains(&(id.clone(), of.clone())),
                _ => false,
            })
            .collect();
        match self.one(&declared) {
            Some(one) => Outcome::Resolved(one),
            None if declared.is_empty() => Outcome::Outside,
            None => Outcome::Open(declared),
        }
    }

    /// What a path call calls, and the type it constructs when the path is
    /// a constructor.
    fn path_call(
        &self,
        caller: FnId,
        reading: &Reading<'_>,
        segments: &[String],
        qself: &Option<(RustType, Option<String>)>,
        args: &[Ty],
    ) -> (Outcome, Ty) {
        let plain = |outcome: Outcome| (outcome, Ty::Open);
        let Some((name, before)) = segments.split_last() else {
            return plain(Outcome::Unknown);
        };
        if let Some((of, as_trait)) = qself {
            let Some(id) = head(&self.ty(reading, of, self.aliases)) else {
                return plain(Outcome::Unknown);
            };
            let named = self.methods.get(name).map(Vec::as_slice).unwrap_or(&[]);
            let own: Vec<FnId> = named
                .iter()
                .copied()
                .filter(|c| self.owners.get(c) == Some(&id))
                .filter(|c| as_trait.is_none() || self.fact(*c).implements == *as_trait)
                .collect();
            if own.is_empty() {
                return plain(Outcome::Outside);
            }
            let local = matches!(id, TypeId::Local(..));
            return plain(self.select(caller, own, args, true, local));
        }
        match before {
            [one] if one == "Self" => {
                return plain(match reading.this.as_ref().and_then(head) {
                    Some(id) => self.assoc(caller, &id, name, args, true),
                    None => Outcome::Unknown,
                });
            }
            // A type parameter's function is the one its bounds give.
            [first, ..] if reading.generics.contains(first) || first == "Self" => {
                return plain(Outcome::Unknown);
            }
            _ => {}
        }
        match self.path(&reading.module, segments, true, &mut Vec::new()) {
            Found::Fn(module, name) => {
                let declared = self
                    .free
                    .get(&(module, name))
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                let own = self.cfg(caller);
                let callable: Vec<FnId> = declared
                    .iter()
                    .copied()
                    .filter(|c| !exclusive(own, self.cfg(*c)))
                    .collect();
                plain(match self.one(&callable) {
                    Some(one) => Outcome::Resolved(one),
                    None if callable.is_empty() => Outcome::Unknown,
                    None => Outcome::Open(callable),
                })
            }
            Found::Type(..) => (Outcome::Outside, self.constructed(reading, segments)),
            Found::Assoc(of, rest) => {
                let [name] = rest.as_slice() else {
                    return plain(Outcome::Unknown);
                };
                let id = match *of {
                    Found::Type(module, of) => {
                        match head(&self.declared(module, of, Vec::new(), self.aliases)) {
                            Some(id) => id,
                            None => return plain(Outcome::Unknown),
                        }
                    }
                    Found::Prelude(of) => TypeId::Prelude(of),
                    _ => return plain(Outcome::Unknown),
                };
                match self.assoc(caller, &id, name, args, true) {
                    // No function of the name: a variant's constructor.
                    Outcome::Outside => (Outcome::Outside, self.constructed(reading, segments)),
                    outcome => plain(outcome),
                }
            }
            // A function of a type of an outside crate is the workspace's
            // when the workspace implements a trait for that type.
            Found::Extern(path) => match path.split_last() {
                Some((name, of)) if !of.is_empty() => {
                    plain(self.assoc(caller, &TypeId::Extern(of.to_vec()), name, args, true))
                }
                _ => plain(Outcome::Outside),
            },
            Found::Prelude(_) | Found::Outside => plain(Outcome::Outside),
            Found::Module(_) | Found::Absent | Found::Unknown => plain(Outcome::Unknown),
        }
    }

    /// What a method call calls, by the type its receiver is written with.
    fn method_call(&self, caller: FnId, name: &str, receiver: &Ty, args: &[Ty]) -> Outcome {
        let mut of = receiver;
        while let Ty::Ref(_, inner) = of {
            of = inner;
        }
        match of {
            Ty::Named(id @ TypeId::Local(..), _) => self.assoc(caller, id, name, args, false),
            // A type of an outside crate with arguments can hand the call
            // on to one of them (`Box<T>`, `Rc<T>`), and a type parameter
            // to its bounds: the code does not say here.
            Ty::Named(TypeId::Prelude(_) | TypeId::Extern(_), type_args)
                if type_args.is_empty() =>
            {
                Outcome::Outside
            }
            Ty::Tuple(_) | Ty::Slice(_) | Ty::Array(..) | Ty::Integer | Ty::Float => {
                Outcome::Outside
            }
            Ty::Named(..) | Ty::Ref(..) | Ty::Open => Outcome::Unknown,
        }
    }

    /// The type a hint stands for.
    fn hint(&self, caller: FnId, reading: &Reading<'_>, hint: &Hint, done: &[(Outcome, Ty)]) -> Ty {
        let of = |h: &Hint| self.hint(caller, reading, h, done);
        match hint {
            Hint::Open => Ty::Open,
            // A type the body itself declares or imports is not the
            // module's: what it names stays open.
            Hint::Written(written)
                if self.names(caller).is_some_and(|n| {
                    written
                        .text
                        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                        .any(|w| n.rebound.contains(w))
                }) =>
            {
                Ty::Open
            }
            Hint::Written(written) => self.ty(reading, written, self.aliases),
            Hint::Prim(name) => Ty::Named(TypeId::Prelude(name.clone()), Vec::new()),
            Hint::Integer => Ty::Integer,
            Hint::Float => Ty::Float,
            Hint::Ref(mutable, inner) => Ty::Ref(*mutable, Box::new(of(inner))),
            Hint::Deref(inner) => match of(inner) {
                Ty::Ref(_, to) => *to,
                _ => Ty::Open,
            },
            Hint::This => reading.this.clone().unwrap_or(Ty::Open),
            Hint::Result(i) => done.get(*i).map(|(_, ty)| ty.clone()).unwrap_or(Ty::Open),
            // `?` on a `Result<T, E>` or an `Option<T>` is a `T`.
            Hint::Try(inner) => match of(inner) {
                Ty::Named(id, args) if unwraps(&id) => args.first().cloned().unwrap_or(Ty::Open),
                _ => Ty::Open,
            },
            Hint::Ctor(segments) => self.constructed(reading, segments),
            Hint::Tuple(items) => Ty::Tuple(items.iter().map(of).collect()),
        }
    }

    /// What each call of a function calls, with the call as ingest records
    /// it. `None` when the function's names were not read.
    pub(crate) fn calls(&self, f: FnId) -> Option<Vec<(String, Outcome)>> {
        let names = self.names(f)?;
        let reading = self.reading(f);
        // A hint names only calls before its own: arguments and receivers
        // are read before the call they belong to.
        let mut done: Vec<(Outcome, Ty)> = Vec::new();
        for call in &names.calls {
            let args: Vec<Ty> = call
                .args
                .iter()
                .map(|h| self.hint(f, &reading, h, &done))
                .collect();
            let (outcome, constructed) = match &call.callee {
                Callee::Path { opaque: true, .. } => (Outcome::Unknown, Ty::Open),
                Callee::Path {
                    segments, qself, ..
                } => self.path_call(f, &reading, segments, qself, &args),
                Callee::Method { name, receiver } => {
                    let receiver = self.hint(f, &reading, receiver, &done);
                    (self.method_call(f, name, &receiver, &args), Ty::Open)
                }
                // A closure or a body-local function: no fact of the
                // workspace, and nothing admission resolves by name.
                Callee::Local(_) => (Outcome::Outside, Ty::Open),
            };
            let ty = match &outcome {
                // The result's type, where every variant declares the same.
                Outcome::Resolved(fns) => {
                    let mut types = fns.iter().map(|g| self.returns(*g));
                    match types.next() {
                        Some(first) if types.all(|t| t == first) => first,
                        _ => Ty::Open,
                    }
                }
                Outcome::Outside => constructed,
                Outcome::Open(_) | Outcome::Unknown => Ty::Open,
            };
            done.push((outcome, ty));
        }
        Some(
            names
                .calls
                .iter()
                .zip(done)
                .map(|(call, (outcome, _))| (call.text(), outcome))
                .collect(),
        )
    }

    /// The bare type names a function's module binds to a type of a
    /// registry crate, each with the crate: `Value` of `use serde_json::Value`.
    pub(crate) fn registry_types(&self, f: FnId) -> BTreeMap<String, String> {
        let module = self.module(f);
        let Some(scope) = self.scope(&module) else {
            return BTreeMap::new();
        };
        // The names the module's own `use` binds, and those of the modules
        // of the workspace it imports whole.
        let mut names: BTreeSet<&str> = scope.uses.iter().map(|u| u.name.as_str()).collect();
        for glob in &scope.globs {
            if let Found::Module(y) = self.path(&module, &glob.path, false, &mut Vec::new()) {
                if let Some(imported) = self.scope(&y) {
                    names.extend(imported.uses.iter().map(|u| u.name.as_str()));
                }
            }
        }
        let mut out = BTreeMap::new();
        for name in names {
            let path = [name.to_string()];
            if let Found::Extern(to) = self.path(&module, &path, false, &mut Vec::new()) {
                if let Some(krate) = to.first().filter(|k| self.registry.contains(*k)) {
                    out.insert(name.to_string(), krate.clone());
                }
            }
        }
        out
    }
}

/// What a type is named by, without its references.
fn head(ty: &Ty) -> Option<TypeId> {
    match ty {
        Ty::Ref(_, inner) => head(inner),
        Ty::Named(id, _) if !matches!(id, TypeId::Param(_)) => Some(id.clone()),
        _ => None,
    }
}

/// Whether `?` on the type yields its first argument: the standard
/// library's `Result` and `Option`, by the prelude's name or by path.
fn unwraps(id: &TypeId) -> bool {
    match id {
        TypeId::Prelude(name) => name == "Result" || name == "Option",
        TypeId::Extern(path) => match path.as_slice() {
            [krate, module, name] => {
                (krate == "std" || krate == "core")
                    && ((module == "result" && name == "Result")
                        || (module == "option" && name == "Option"))
            }
            _ => false,
        },
        _ => false,
    }
}

/// The workspace as its projected configuration compiles it. Two items of
/// one path under `cfg` predicates no build holds together are one fact
/// with two variants, not two facts: where the configuration compiles one
/// in and every other out, the projection states that one, and each other
/// is an open claim. Where the configuration does not decide (a predicate
/// on the target), both stay, and admission refuses the name they share.
pub(crate) fn stated(ws: &Workspace, syntax: &Syntax) -> (Workspace, Syntax, Vec<Refusal>) {
    let mut ws = ws.clone();
    let mut syntax = syntax.clone();
    let mut refusals = Vec::new();
    for package in &mut ws.packages {
        let features = syntax
            .names
            .features
            .get(&package.dir)
            .cloned()
            .unwrap_or_default();
        let config = Config::dump(features);
        // The items of one path: where each stands, and under what.
        type Variant = (usize, usize, Vec<Cfg>);
        // A function's path: target, module, owner, trait, whether a
        // trait declares it, and its name.
        type FnPath = (usize, String, Option<String>, Option<String>, bool, String);
        let mut fns: BTreeMap<FnPath, Vec<Variant>> = BTreeMap::new();
        let mut enums: BTreeMap<(usize, String, String), Vec<Variant>> = BTreeMap::new();
        let mut structs: BTreeMap<(usize, String, String), Vec<Variant>> = BTreeMap::new();
        // A constant's path: target, module and name, as a function's.
        let mut consts: BTreeMap<(usize, String, String), Vec<Variant>> = BTreeMap::new();
        for (fi, file) in package.files.iter().enumerate() {
            let Some(read) = syntax.names.files.get(&file.rel) else {
                continue;
            };
            let declared = syntax
                .files
                .get(&file.rel)
                .map_or(&[][..], |f| &f.consts[..]);
            for (idx, (name, .., line)) in declared.iter().enumerate() {
                if let Some(site) = read.consts.get(&(*line, name.clone())) {
                    let key = (file.target, join(&file.module, &site.module), name.clone());
                    consts
                        .entry(key)
                        .or_default()
                        .push((fi, idx, site.cfg.clone()));
                }
            }
            for (idx, fact) in file.fns.iter().enumerate() {
                let Some(names) = read.fns.get(&(fact.span.line, fact.name.clone())) else {
                    continue;
                };
                let key = (
                    file.target,
                    join(&file.module, &names.module),
                    fact.owner.clone(),
                    fact.implements.clone(),
                    fact.in_trait,
                    fact.name.clone(),
                );
                fns.entry(key)
                    .or_default()
                    .push((fi, idx, names.cfg.clone()));
            }
            for (idx, fact) in file.enums.iter().enumerate() {
                if let Some(site) = read.types.get(&(fact.span.line, fact.name.clone())) {
                    let key = (
                        file.target,
                        join(&file.module, &site.module),
                        fact.name.clone(),
                    );
                    enums
                        .entry(key)
                        .or_default()
                        .push((fi, idx, site.cfg.clone()));
                }
            }
            for (idx, fact) in file.structs.iter().enumerate() {
                if let Some(site) = read.types.get(&(fact.span.line, fact.name.clone())) {
                    let key = (
                        file.target,
                        join(&file.module, &site.module),
                        fact.name.clone(),
                    );
                    structs
                        .entry(key)
                        .or_default()
                        .push((fi, idx, site.cfg.clone()));
                }
            }
        }
        // Of each path's variants, those the configuration compiles out.
        let unstated = |variants: &[Variant]| -> Vec<Variant> {
            let apart = variants
                .iter()
                .enumerate()
                .all(|(i, a)| variants[i + 1..].iter().all(|b| exclusive(&a.2, &b.2)));
            let held: Vec<Option<bool>> = variants.iter().map(|v| hold(&v.2, &config)).collect();
            let one = held.iter().filter(|h| **h == Some(true)).count() == 1
                && held.iter().all(Option::is_some);
            if variants.len() < 2 || !apart || !one {
                return Vec::new();
            }
            variants
                .iter()
                .zip(held)
                .filter(|(_, h)| *h == Some(false))
                .map(|(v, _)| v.clone())
                .collect()
        };
        let predicate = |cfg: &[Cfg]| match cfg {
            [one] => one.text(),
            many => Cfg::All(many.to_vec()).text(),
        };
        let krate = |file: &SourceFile| package.targets[file.target].name.replace('-', "_");
        let mut drop_fns: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut drop_enums: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut drop_structs: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut drop_consts: BTreeSet<(usize, usize)> = BTreeSet::new();
        for ((_, module, owner, _, _, name), variants) in &fns {
            for (fi, idx, cfg) in unstated(variants) {
                let file = &package.files[fi];
                let mut parts = vec![krate(file)];
                parts.extend(
                    module
                        .split("::")
                        .filter(|s| !s.is_empty())
                        .map(str::to_string),
                );
                parts.extend(owner.clone());
                parts.push(name.clone());
                refusals.push(Refusal::CfgVariant {
                    span: file.fns[idx].span.clone(),
                    name: parts.join("::"),
                    predicate: predicate(&cfg),
                });
                drop_fns.insert((fi, idx));
            }
        }
        // A constant declared once per configuration is one constant, as a
        // function is: the variant the configuration compiles is stated,
        // each other is an open claim (`const K` under `cfg(any())` beside
        // one under `cfg(all())`: the build reads the second).
        for ((_, module, name), variants) in &consts {
            for (fi, idx, cfg) in unstated(variants) {
                let file = &package.files[fi];
                let mut parts = vec![krate(file)];
                parts.extend(
                    module
                        .split("::")
                        .filter(|s| !s.is_empty())
                        .map(str::to_string),
                );
                parts.push(name.clone());
                let line = syntax.files.get(&file.rel).map_or(1, |f| f.consts[idx].5);
                refusals.push(Refusal::CfgVariant {
                    span: Span {
                        file: file.rel.clone(),
                        line,
                    },
                    name: parts.join("::"),
                    predicate: predicate(&cfg),
                });
                drop_consts.insert((fi, idx));
            }
        }
        for (types, dropped, enumeration) in [
            (&enums, &mut drop_enums, true),
            (&structs, &mut drop_structs, false),
        ] {
            for ((_, module, name), variants) in types {
                for (fi, idx, cfg) in unstated(variants) {
                    let file = &package.files[fi];
                    let mut parts = vec![krate(file)];
                    parts.extend(
                        module
                            .split("::")
                            .filter(|s| !s.is_empty())
                            .map(str::to_string),
                    );
                    parts.push(name.clone());
                    let span = if enumeration {
                        file.enums[idx].span.clone()
                    } else {
                        file.structs[idx].span.clone()
                    };
                    refusals.push(Refusal::CfgVariant {
                        span,
                        name: parts.join("::"),
                        predicate: predicate(&cfg),
                    });
                    dropped.insert((fi, idx));
                }
            }
        }
        // A fact and its syntax leave together: the two lists stay in step.
        fn keep<T>(list: &mut Vec<T>, file: usize, dropped: &BTreeSet<(usize, usize)>) {
            let mut idx = 0;
            list.retain(|_| {
                idx += 1;
                !dropped.contains(&(file, idx - 1))
            });
        }
        for (fi, file) in package.files.iter_mut().enumerate() {
            let Some(file_syntax) = syntax.files.get_mut(&file.rel) else {
                continue;
            };
            keep(&mut file.fns, fi, &drop_fns);
            keep(&mut file_syntax.fns, fi, &drop_fns);
            keep(&mut file.enums, fi, &drop_enums);
            keep(&mut file_syntax.enums, fi, &drop_enums);
            keep(&mut file.structs, fi, &drop_structs);
            keep(&mut file_syntax.structs, fi, &drop_structs);
            keep(&mut file_syntax.consts, fi, &drop_consts);
        }
    }
    (ws, syntax, refusals)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(text: &str) -> Vec<Cfg> {
        let item: syn::ItemFn =
            syn::parse_str(&format!("#[cfg({text})] fn f() {{}}")).expect("an item");
        Cfg::of(&item.attrs)
    }

    #[test]
    fn a_predicate_and_its_negation_hold_in_no_build_together() {
        assert!(exclusive(
            &cfg("feature = \"a\""),
            &cfg("not(feature = \"a\")")
        ));
        assert!(exclusive(
            &cfg("all(feature = \"a\", target_arch = \"wasm32\")"),
            &cfg("any(not(feature = \"a\"), not(target_arch = \"wasm32\"))")
        ));
        // Two features can be on together, and an item without a predicate
        // stands in every build.
        assert!(!exclusive(&cfg("feature = \"a\""), &cfg("feature = \"b\"")));
        assert!(!exclusive(&cfg("feature = \"a\""), &[]));
        assert!(!exclusive(
            &cfg("any(feature = \"a\", test)"),
            &cfg("not(feature = \"a\")")
        ));
    }

    #[test]
    fn the_projected_configuration_is_the_dumps() {
        let config = Config::dump(["x".to_string()].into_iter().collect());
        let holds = |text: &str| hold(&cfg(text), &config);
        assert_eq!(holds("feature = \"x\""), Some(true));
        assert_eq!(holds("feature = \"y\""), Some(false));
        assert_eq!(holds("test"), Some(false));
        assert_eq!(holds("not(uorust_lift)"), Some(false));
        assert_eq!(holds("debug_assertions"), Some(false));
        // The target is the host's, which the workspace does not declare.
        assert_eq!(holds("target_arch = \"wasm32\""), None);
        assert_eq!(holds("all(feature = \"x\", target_os = \"linux\")"), None);
        assert_eq!(
            holds("all(feature = \"y\", target_os = \"linux\")"),
            Some(false)
        );
    }

    #[test]
    fn default_features_follow_cargos_rules() {
        let manifest_doc = "[features]\ndefault = [\"a\", \"opt/x\", \"weak?/y\"]\n\
             a = [\"b\", \"dep:hidden\"]\nb = []\nc = []\n";
        let manifest: toml::Value = toml::from_str(manifest_doc).expect("a manifest");
        let on: Vec<String> = enabled_by_default(&manifest).into_iter().collect();
        assert_eq!(on, ["a", "b", "default", "opt"]);
        let none: toml::Value = toml::from_str("[features]\nc = []\n").expect("a manifest");
        assert!(enabled_by_default(&none).is_empty());
    }

    fn body_calls(source: &str) -> Vec<Call> {
        let file: syn::File = syn::parse_str(source).expect("a file");
        let names = FileNames::of(&file);
        names.fns.into_values().next().expect("a function").calls
    }

    #[test]
    fn a_body_is_read_with_what_it_writes_of_types() {
        let calls = body_calls(
            "fn f(a: u8, b: &str) { let c: Foo = make(); let d = 1.5; let (e, _) = pair(); \
             g(a, b, c, d, e, \"s\", -2, &a, x as i64); let a = other(); h(a); }",
        );
        let texts: Vec<String> = calls.iter().map(Call::text).collect();
        assert_eq!(texts, ["make", "pair", "g", "other", "h"]);
        let written = |h: &Hint| match h {
            Hint::Written(t) => t.text.clone(),
            other => format!("{other:?}"),
        };
        let args: Vec<String> = calls[2].args.iter().map(written).collect();
        assert_eq!(
            args,
            [
                "u8",
                "&str",
                "Foo",
                "Float",
                "Open",
                "Ref(false, Prim(\"str\"))",
                "Integer",
                "Ref(false, Written(RustType { text: \"u8\", kind: Path { qself: false, path: Path { segments: [PathSegment { ident: \"u8\", arguments: None }], text: \"u8\" } } }))",
                "i64"
            ]
        );
        // A `let` without a written type has its value's: here a call's.
        assert_eq!(calls[4].args, [Hint::Result(3)]);
    }

    #[test]
    fn a_name_the_body_binds_is_not_a_path_to_a_function() {
        let calls = body_calls(
            "fn f() { let run = |x: u8| x; run(1); fn inner() {} inner(); \
             { use a::b::go; go(); } go(); }",
        );
        let texts: Vec<String> = calls
            .iter()
            .filter(|c| !matches!(c.callee, Callee::Local(_)))
            .map(Call::text)
            .collect();
        assert_eq!(texts, ["a::b::go", "go"]);
        let local: Vec<String> = calls
            .iter()
            .filter(|c| matches!(c.callee, Callee::Local(_)))
            .map(Call::text)
            .collect();
        assert_eq!(local, ["run", "inner"]);
    }

    // Name resolution: a call resolved as the language resolves it.

    /// What each call of each function of `source` (a library `lib.rs`)
    /// resolves to, as `caller: call -> outcome`, with a function written
    /// `Owner::name@line`.
    fn resolutions(tag: &str, manifest_extra: &str, source: &str) -> Vec<String> {
        let dir = std::env::temp_dir().join(format!("uorust-names-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"w\"\nversion = \"0.1.0\"\nedition = \"2021\"\n{manifest_extra}"),
        )
        .unwrap();
        std::fs::write(dir.join("src/lib.rs"), source).unwrap();
        let (ws, syntax) = crate::ingest::workspace(&dir, None).expect("a workspace");
        let syntax = read(&dir, &ws, syntax).expect("its names");
        let _ = std::fs::remove_dir_all(&dir);
        let resolver = Resolver::new(&ws, &syntax);
        let named = |g: &FnId| {
            let f = &ws.packages[g.0].files[g.1].fns[g.2];
            match &f.owner {
                Some(o) => format!("{o}::{}@{}", f.name, f.span.line),
                None => format!("{}@{}", f.name, f.span.line),
            }
        };
        let mut out = Vec::new();
        for (pkg, package) in ws.packages.iter().enumerate() {
            for (file, source) in package.files.iter().enumerate() {
                for (idx, fact) in source.fns.iter().enumerate() {
                    for (call, outcome) in resolver.calls((pkg, file, idx)).unwrap_or_default() {
                        let to = match &outcome {
                            Outcome::Resolved(fns) => {
                                fns.iter().map(named).collect::<Vec<_>>().join(" ")
                            }
                            Outcome::Open(fns) => format!(
                                "open {}",
                                fns.iter().map(named).collect::<Vec<_>>().join(" ")
                            ),
                            Outcome::Outside => "outside".to_string(),
                            Outcome::Unknown => "unknown".to_string(),
                        };
                        out.push(format!("{}: {call} -> {to}", fact.name));
                    }
                }
            }
        }
        out
    }

    #[test]
    fn self_and_a_written_receiver_resolve_to_their_types_method() {
        let got = resolutions(
            "recv",
            "",
            "pub struct A;\npub struct B;\n\
             impl A {\n pub fn m(&self) -> u8 { 1 }\n pub fn f() -> u8 { 1 }\n \
             pub fn g() -> u8 { Self::f() }\n pub fn h(&self) -> u8 { self.m() }\n}\n\
             impl B {\n pub fn m(&self) -> u8 { 2 }\n pub fn f() -> u8 { 2 }\n}\n\
             pub fn by_param(x: &A) -> u8 { x.m() }\n\
             pub fn by_let() -> u8 { let x: B = B; x.m() }\n",
        );
        for want in [
            "g: Self::f -> A::f@5",
            "h: .m -> A::m@4",
            "by_param: .m -> A::m@4",
            "by_let: .m -> B::m@10",
        ] {
            assert!(got.iter().any(|g| g == want), "{want} not in {got:#?}");
        }
    }

    #[test]
    fn a_path_resolves_through_use_and_the_nearest_module() {
        let got = resolutions(
            "path",
            "",
            "use crate::left::pick;\n\
             pub mod left {\n pub fn pick() -> u8 { 1 }\n pub fn near() -> u8 { 3 }\n \
             pub fn here() -> u8 { near() }\n}\n\
             pub mod right {\n pub fn pick() -> u8 { 2 }\n pub fn near() -> u8 { 4 }\n \
             pub fn there() -> u8 { super::left::near() + pick() }\n}\n\
             pub fn through_use() -> u8 { pick() }\n",
        );
        for want in [
            "here: near -> near@4",
            "through_use: pick -> pick@3",
            "there: super::left::near -> near@4",
            // `pick` in `right` is its own, not the crate root's import.
            "there: pick -> pick@8",
        ] {
            assert!(got.iter().any(|g| g == want), "{want} not in {got:#?}");
        }
    }

    #[test]
    fn a_call_the_written_code_leaves_open_stays_open() {
        let got = resolutions(
            "open",
            "",
            "pub struct A(u8);\npub struct B;\n\
             impl From<u8> for A { fn from(x: u8) -> A { A(x) } }\n\
             impl From<u16> for A { fn from(x: u16) -> A { A(x as u8) } }\n\
             pub fn unknown() -> A { A::from(Default::default()) }\n",
        );
        assert!(
            got.iter()
                .any(|g| g.starts_with("unknown: A::from -> open ")),
            "{got:#?}"
        );
    }
}
