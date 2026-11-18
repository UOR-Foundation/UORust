//! Facts read from a workspace, each with the source span it came from.

use std::collections::BTreeMap;

/// A place in the workspace: a path relative to the workspace root and a line.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Span {
    pub(crate) file: String,
    pub(crate) line: usize,
}

impl Span {
    pub(crate) fn render(&self) -> String {
        format!("{}:{}", self.file, self.line)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Workspace {
    pub(crate) packages: Vec<Package>,
    pub(crate) lock: Vec<LockedPackage>,
    pub(crate) has_deny: bool,
    /// Advisory IDs `deny.toml` ignores, with the reason it gives.
    pub(crate) deny_ignores: BTreeMap<String, String>,
    /// The advisories consulted, when an advisory database was given.
    pub(crate) advisories: Option<Advisories>,
    /// The targets `rust-toolchain.toml` declares the workspace is built for.
    pub(crate) targets: Vec<String>,
    /// The Cargo configuration files that apply to the workspace and
    /// replace a source or override a package's path (`[source]`,
    /// `paths`): Cargo then builds what they name, not the locked archive.
    pub(crate) replaced: Vec<String>,
}

/// What an advisory database says about the locked packages.
#[derive(Clone, Debug)]
pub(crate) struct Advisories {
    /// The database's revision, or `unknown`.
    pub(crate) revision: String,
    /// Advisories that affect a locked package's version, by advisory ID.
    pub(crate) hits: Vec<Advisory>,
}

/// One RustSec advisory against one locked package version.
#[derive(Clone, Debug)]
pub(crate) struct Advisory {
    pub(crate) id: String,
    pub(crate) package: String,
    pub(crate) version: String,
    pub(crate) title: String,
    pub(crate) date: String,
    /// `unmaintained`, `unsound` or `notice` for an informational advisory.
    pub(crate) informational: Option<String>,
    pub(crate) categories: Vec<String>,
    pub(crate) patched: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct LockedPackage {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: String,
    pub(crate) checksum: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) description: String,
    /// Cargo's `repository` and `homepage` URLs, empty when undeclared.
    pub(crate) repository: String,
    pub(crate) homepage: String,
    pub(crate) license: String,
    pub(crate) rust_version: Option<String>,
    /// The Rust edition the package's manifest states.
    pub(crate) edition: String,
    pub(crate) authors: Vec<String>,
    /// The package directory relative to the workspace root.
    pub(crate) dir: String,
    pub(crate) targets: Vec<Target>,
    pub(crate) deps: Vec<Dep>,
    /// Lint levels the package declares or inherits.
    pub(crate) lints: BTreeMap<String, String>,
    pub(crate) files: Vec<SourceFile>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TargetKind {
    Lib,
    Bin,
    Test,
    Example,
    Bench,
}

#[derive(Clone, Debug)]
pub(crate) struct Target {
    pub(crate) name: String,
    pub(crate) kind: TargetKind,
    /// The root source file relative to the workspace root.
    pub(crate) src: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Dep {
    pub(crate) name: String,
    /// The name the package's code knows the dependency by, where the
    /// manifest renames it (`big = { package = "num-bigint" }`).
    pub(crate) rename: Option<String>,
    /// `normal`, `dev` or `build`.
    pub(crate) kind: String,
    /// The path of a path dependency relative to the workspace root, with
    /// `..` steps when it lies outside it.
    pub(crate) path: Option<String>,
    pub(crate) inside: bool,
    /// The platform a `[target.'..'.dependencies]` table restricts it to.
    pub(crate) platform: Option<String>,
}

/// What one source file holds.
#[derive(Clone, Debug, Default)]
pub(crate) struct SourceFile {
    pub(crate) rel: String,
    /// The target this file belongs to: its root's index in `Package::targets`.
    pub(crate) target: usize,
    /// The module path inside the target, `""` for the root.
    pub(crate) module: String,
    /// Whether the whole file is test code: a test target's file, or a
    /// file only test code declares as a module.
    pub(crate) test: bool,
    pub(crate) parse_error: Option<String>,
    pub(crate) doc: String,
    pub(crate) forbids_unsafe: bool,
    pub(crate) fns: Vec<FnFact>,
    pub(crate) enums: Vec<EnumFact>,
    pub(crate) structs: Vec<StructFact>,
    pub(crate) error_impls: Vec<String>,
    /// Every type the file declares, inline modules included: each struct,
    /// enum, union, trait, trait alias and type alias, with the module scope
    /// that declares it (an index into `scopes`), by name, at the line of its
    /// name. A construct of the lift register names the sites that define
    /// the types it writes from these.
    pub(crate) declared: Vec<(usize, String, Span)>,
    /// The traits the file declares, each with what a dictionary of its
    /// methods does not read.
    pub(crate) trait_decls: Vec<TraitFact>,
    /// The file's impls of a trait, methods or none: which types a call
    /// through the trait can select an impl of.
    pub(crate) trait_impls: Vec<TraitImpl>,
    /// The types the file declares in a shape the lift does not read, each
    /// with that shape (`a tuple struct`).
    pub(crate) unread: Vec<(String, &'static str)>,
    /// The file's module scopes: its own first, then each inline module's
    /// in source order.
    pub(crate) scopes: Vec<ScopeFact>,
    /// Whether the file, or a module it declares, is marked
    /// `no_implicit_prelude`.
    pub(crate) no_prelude: bool,
    pub(crate) capabilities: Vec<(Capability, Span, String)>,
    /// `use` imports: the imported name -> the path it names.
    pub(crate) imports: BTreeMap<String, String>,
    /// The macros product code of the file invokes in item position (or
    /// among a trait's items), by the last segment of their path: the
    /// items they expand to are in no fact (see [`Package::makes_items`]).
    pub(crate) item_macros: Vec<String>,
    /// The macros the file defines by `macro_rules!`.
    pub(crate) macro_defs: Vec<String>,
}

impl ScopeFact {
    /// Whether the scope (the `i`th of its file, the file's own first) has
    /// a glob import that may reach outside its file: any but `use
    /// super::*` of an inline module, which names the module around it in
    /// the same file.
    pub(crate) fn outward_glob(&self, i: usize) -> bool {
        self.globs.iter().any(|g| i == 0 || g != "super")
    }
}

/// The standard library's macros whose expansion in item position
/// declares no type, trait or module: `thread_local!` declares only the
/// `static` keys it names (`std::thread::LocalKey`, the macro's own
/// documentation).
const STATICS_ONLY: [&str; 1] = ["thread_local"];

impl Package {
    /// Whether file `fi` invokes a macro in item position that may make an
    /// item (a type, a trait, a module, an `impl`) no fact holds: any but a
    /// standard macro of [`STATICS_ONLY`], which counts only where nothing
    /// of the package defines or imports a macro of its name.
    pub(crate) fn file_makes_items(&self, fi: usize) -> bool {
        let shadowed = |name: &str| {
            self.files.iter().any(|f| {
                f.macro_defs.iter().any(|d| d == name)
                    || f.scopes
                        .iter()
                        .any(|s| s.uses.iter().any(|(n, _)| n == name))
            })
        };
        self.files[fi]
            .item_macros
            .iter()
            .any(|m| !STATICS_ONLY.contains(&m.as_str()) || shadowed(m))
    }

    /// Whether a file of the same target as file `fi` makes such items: a
    /// target is one crate, and another target's items are not in its
    /// scope.
    pub(crate) fn makes_items(&self, fi: usize) -> bool {
        let target = self.files[fi].target;
        (0..self.files.len()).any(|i| self.files[i].target == target && self.file_makes_items(i))
    }
}

impl SourceFile {
    /// The module an item of the scope `scope` (an index of `scopes`)
    /// stands in, inside its target: the file's module joined with the
    /// scope's inline modules. It is the module Rust names the item
    /// through, so two items of one kind and name are one item only where
    /// this is the same: `go` of the crate root and `go` of an inline
    /// `mod inner` beside it are two.
    pub(crate) fn item_module(&self, scope: usize) -> String {
        let inline = self.scopes.get(scope).map_or("", |s| s.module.as_str());
        match (self.module.as_str(), inline) {
            (outer, "") => outer.to_string(),
            ("", inner) => inner.to_string(),
            (outer, inner) => format!("{outer}::{inner}"),
        }
    }
}

/// A trait declaration: what, besides its methods, it asks of an
/// implementing type.
#[derive(Clone, Debug, Default)]
pub(crate) struct TraitFact {
    pub(crate) name: String,
    /// The index of the module scope it is declared in.
    pub(crate) scope: usize,
    /// Its type and const parameters: a trait implemented once per
    /// argument list.
    pub(crate) params: Vec<String>,
    /// Its supertraits as written, and any `where` clause on `Self`.
    pub(crate) supertraits: Vec<String>,
    /// Its associated types and constants, by name.
    pub(crate) associated: Vec<String>,
    /// `unsafe trait`: an implementation promises what no method states.
    pub(crate) marked: bool,
}

/// An `impl Trait for Type`, whatever methods it writes.
#[derive(Clone, Debug, Default)]
pub(crate) struct TraitImpl {
    /// The trait's path as written, without arguments.
    pub(crate) trait_path: String,
    /// The self type as written.
    pub(crate) self_ty: String,
    /// The self type where it is a path with no arguments that names no
    /// parameter of the impl: a type by its own name, not a reference, a
    /// box or any type a blanket impl covers.
    pub(crate) plain: Option<String>,
    /// The index of the module scope it is written in.
    pub(crate) scope: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct FnFact {
    pub(crate) name: String,
    /// `Type` for a method of `impl Type`.
    pub(crate) owner: Option<String>,
    pub(crate) public: bool,
    pub(crate) test: bool,
    pub(crate) in_test_module: bool,
    pub(crate) span: Span,
    pub(crate) doc: String,
    pub(crate) doctests: usize,
    pub(crate) signature: String,
    /// The signature and body as tokens, which name the function's κ in the
    /// lift register.
    pub(crate) tokens: String,
    /// Path calls, `a::b::c` as written.
    pub(crate) calls: Vec<String>,
    /// String literals in the body.
    pub(crate) literals: Vec<String>,
    /// Arms `E::A => E::B` of a match in the body.
    pub(crate) transitions: Vec<(String, String, String)>,
    /// The first operating-system effect the body performs directly.
    pub(crate) effect: Option<(Capability, String)>,
    /// The method is declared by a trait (`owner` names the trait), not by
    /// an `impl`.
    pub(crate) in_trait: bool,
    /// For a method of `impl Trait for Type`: the trait's last segment.
    pub(crate) implements: Option<String>,
    /// For a method of `impl Trait for Type`: the trait as the impl names
    /// it, its last segment with its arguments (`From<u8>`).
    pub(crate) trait_ref: Option<String>,
    /// Whether the method's type has another method of its name in the same
    /// build: an inherent one beside a trait's, two traits', or one generic
    /// trait's at two argument lists. Rust then names a trait's method
    /// `<Type as Trait>::name`, and `Type::name` otherwise.
    pub(crate) apart: bool,
    /// The receiver as the method writes it (`&mut self`), where it takes
    /// one.
    pub(crate) receiver: Option<String>,
    /// For a method of `impl Trait for Type`: the trait's path as the impl
    /// writes it, without its arguments (`fmt::Display`).
    pub(crate) trait_path: Option<String>,
    /// The type and const parameters of the method's impl block: a method
    /// of `impl<T: Bound> Trait for T` is one function per type.
    pub(crate) impl_params: Vec<String>,
    /// Which of its file's module scopes (`SourceFile::scopes`) declares
    /// the function.
    pub(crate) scope: usize,
    /// For a method of an `impl` block: the line and the column (from 1,
    /// in characters) its header starts at, by which the compiler's dump
    /// names the impl (`<impl at file:line:column: ..>`).
    pub(crate) impl_at: Option<(usize, usize)>,
}

/// One module scope of a file: the file's own items, or an inline module's.
/// A trait's methods are callable where the trait is in scope, and a scope
/// inherits no name from the module around it.
#[derive(Clone, Debug, Default)]
pub(crate) struct ScopeFact {
    /// The scope's module path inside its file: `""` for the file's own
    /// items, `a::b` for an inline `mod b` inside an inline `mod a`. Rust
    /// names every item of the scope through it (`SourceFile::item_module`).
    pub(crate) module: String,
    /// Each name a `use` of the scope binds, with the path it names.
    pub(crate) uses: Vec<(String, String)>,
    /// Whether a `use` of the scope ends in `*`: it may bind any name.
    pub(crate) glob: bool,
    /// The path of each glob import of the scope, as written.
    pub(crate) globs: Vec<String>,
    /// The traits the scope declares.
    pub(crate) traits: Vec<String>,
    /// The modules the scope declares out of line, each a file of its own.
    pub(crate) mods: Vec<String>,
    /// The macros the scope invokes in item position, each with the
    /// identifiers its arguments write and where: an item such a macro
    /// expands to is declared in the scope, though no item of the source
    /// names it.
    pub(crate) invoked: Vec<(Vec<String>, Span)>,
}

#[derive(Clone, Debug)]
pub(crate) struct EnumFact {
    pub(crate) name: String,
    pub(crate) span: Span,
    /// Which of its file's module scopes (`SourceFile::scopes`) declares
    /// the enum.
    pub(crate) scope: usize,
    pub(crate) derives: Vec<String>,
    /// Whether a variant sets its discriminant (`A = 5`): the values MIR
    /// switches on are then not the variants' positions.
    pub(crate) explicit: bool,
    /// Variant name, whether it is a unit variant, its doc, whether `#[default]`.
    pub(crate) variants: Vec<Variant>,
    /// Each variant's fields, `Variant.field` (a position for a tuple
    /// variant's), with its type as the source writes it.
    pub(crate) fields: Vec<(String, String)>,
}

/// A struct, whatever its shape: its syntax says which fields it has.
#[derive(Clone, Debug)]
pub(crate) struct StructFact {
    pub(crate) name: String,
    pub(crate) span: Span,
    /// Which of its file's module scopes (`SourceFile::scopes`) declares
    /// the struct.
    pub(crate) scope: usize,
    /// The traits a `#[derive(...)]` names.
    pub(crate) derives: Vec<String>,
    /// Each field (a position for a tuple struct's), with its type as the
    /// source writes it.
    pub(crate) fields: Vec<(String, String)>,
}

/// The syntax the lift reads, kept apart from the facts: per source file,
/// by its path relative to the workspace root.
#[derive(Clone, Debug, Default)]
pub(crate) struct Syntax {
    pub(crate) files: BTreeMap<String, FileSyntax>,
    /// What name resolution reads of the workspace: each module's bindings
    /// and each body's calls.
    pub(crate) names: crate::names::Names,
}

impl Syntax {
    /// The syntax of the `idx`th function of the file at `rel`.
    pub(crate) fn function(&self, rel: &str, idx: usize) -> &FnSyntax {
        &self.files[rel].fns[idx]
    }
}

/// A constant as [`FileSyntax::consts`] holds it.
pub(crate) type ConstSyntax = (
    String,
    crate::ast::RustType,
    crate::ast::Block,
    usize,
    String,
    usize,
);

/// One file's syntax, in the order of its [`SourceFile`]'s facts.
#[derive(Clone, Debug, Default)]
pub(crate) struct FileSyntax {
    /// One per [`SourceFile::fns`].
    pub(crate) fns: Vec<FnSyntax>,
    /// The named fields of each of [`SourceFile::structs`].
    pub(crate) structs: Vec<Vec<(String, crate::ast::RustType)>>,
    /// The payload of each variant of each of [`SourceFile::enums`]: its
    /// fields, named or positional, in order. Empty for a generic enum.
    pub(crate) enums: Vec<crate::lift::RawVariants>,
    /// The constants of product code outside tests: name, type, the value
    /// as a block, which lifts like a function of no parameters, the module
    /// scope (`SourceFile::scopes`) that declares it, its tokens, and the
    /// line of its name.
    pub(crate) consts: Vec<ConstSyntax>,
    /// The type aliases: `type Name<P, ..> = Type;`.
    pub(crate) aliases: Vec<(String, crate::ast::Alias)>,
}

/// One function's syntax.
#[derive(Clone, Debug)]
pub(crate) struct FnSyntax {
    pub(crate) signature: crate::ast::Signature,
    pub(crate) block: Option<crate::ast::Block>,
    /// `assert!`/`assert_eq!` arguments.
    pub(crate) asserts: Vec<(String, Vec<crate::ast::RustExpr>)>,
}

#[derive(Clone, Debug)]
pub(crate) struct Variant {
    pub(crate) name: String,
    pub(crate) unit: bool,
    pub(crate) doc: String,
    pub(crate) default: bool,
}

/// An effect a target reaches beyond pure computation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Capability {
    Fs,
    Net,
    Process,
    Env,
    Unsafe,
    Ffi,
}

impl Capability {
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Fs => "Fs",
            Self::Net => "Net",
            Self::Process => "Process",
            Self::Env => "Env",
            Self::Unsafe => "Unsafe",
            Self::Ffi => "Ffi",
        }
    }
}
