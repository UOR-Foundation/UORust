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
    /// The workspace directory name, the project's name.
    pub(crate) name: String,
    pub(crate) packages: Vec<Package>,
    pub(crate) lock: Vec<LockedPackage>,
    pub(crate) has_deny: bool,
    /// Advisory IDs `deny.toml` ignores, with the reason it gives.
    pub(crate) deny_ignores: BTreeMap<String, String>,
    /// The advisories consulted, when an advisory database was given.
    pub(crate) advisories: Option<Advisories>,
    /// The targets `rust-toolchain.toml` declares the workspace is built for.
    pub(crate) targets: Vec<String>,
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
    /// `normal`, `dev` or `build`.
    pub(crate) kind: String,
    /// The path of a path dependency relative to the workspace root, or the
    /// absolute path when it lies outside it.
    pub(crate) path: Option<String>,
    pub(crate) inside: bool,
}

/// What one source file holds.
#[derive(Clone, Debug, Default)]
pub(crate) struct SourceFile {
    pub(crate) rel: String,
    /// The target this file belongs to: its root's index in `Package::targets`.
    pub(crate) target: usize,
    /// The module path inside the target, `""` for the root.
    pub(crate) module: String,
    pub(crate) parse_error: Option<String>,
    pub(crate) doc: String,
    pub(crate) forbids_unsafe: bool,
    pub(crate) fns: Vec<FnFact>,
    pub(crate) enums: Vec<EnumFact>,
    pub(crate) structs: Vec<StructFact>,
    pub(crate) error_impls: Vec<String>,
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
    /// Each name a `use` of the scope binds, with the path it names.
    pub(crate) uses: Vec<(String, String)>,
    /// Whether a `use` of the scope ends in `*`: it may bind any name.
    pub(crate) glob: bool,
    /// The traits the scope declares.
    pub(crate) traits: Vec<String>,
    /// The modules the scope declares out of line, each a file of its own.
    pub(crate) mods: Vec<String>,
    /// The inline modules from the file's own scope to this one, outermost
    /// first: empty for the file's own.
    pub(crate) inline: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct EnumFact {
    pub(crate) name: String,
    pub(crate) span: Span,
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
    /// The constants of product code outside tests: name, type, and the
    /// value as a block, which lifts like a function of no parameters.
    pub(crate) consts: Vec<(String, crate::ast::RustType, crate::ast::Block)>,
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
