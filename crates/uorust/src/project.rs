//! Projection: admitted facts to the files of a Prism project, byte for byte
//! the same for the same workspace.

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use crate::admit::Refusal;
use crate::facts::{Capability, FnFact, Package, SourceFile, Span, Syntax, TargetKind, Workspace};
use crate::ingest::summary;
use crate::lex::{pascal, string, string_list, Arm, Decl, Expr, Module};
use crate::lift::{self, Candidate, Residue, Ty, Types};
use crate::register::{queue_row, ranked, Register};
use crate::UorustError;

/// What a projection is parameterised by beyond the workspace.
#[derive(Clone, Debug)]
pub struct Options {
    /// The project name; its PascalCase form is the LexLean module prefix.
    pub name: String,
    /// The control set's `lastModified`, `YYYY-MM-DDThh:mm:ssZ`.
    pub last_modified: String,
    /// A RustSec advisory database checkout to check the locked packages
    /// against; without one, the frontier records that none was consulted.
    pub advisories: Option<std::path::PathBuf>,
    /// The MIR path of the lift, when it runs: see [`crate::mir`].
    pub mir: Option<crate::mir::MirOptions>,
}

/// The largest source file the projection's LexLean project admits.
pub(crate) const MAX_FILE_BYTES: usize = 4_194_304;

/// The projection's own register of what it did not admit or lift.
const FRONTIER: &str = "frontier.toml";

/// The projection's LexLean project file, which states its limits.
const LEXLEAN: &str = "lexlean.toml";

const ARCH: &str = "Architecture";
const IFACE: &str = "Interface";
const DESIGN: &str = "Design";
const SEC: &str = "Security";
const QUAL: &str = "Quality";
const BEHAVIOR: &str = "Behavior";
const SUPPLY: &str = "SupplyChain";

const BOOL: &str = "lexlean.std.bool@1.2.0";
const NAT: &str = "lexlean.std.nat@1.2.0";
const PARCH: &str = "prism.arch@1.0.0";
const PSEC: &str = "prism.sec@1.0.0";
const PQUAL: &str = "prism.qual@1.0.0";
const TITLES: &str = "uorust.titles@1.0.0";

/// A function in the workspace: package, file and position in the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct FnRef {
    pkg: usize,
    file: usize,
    idx: usize,
}

#[derive(Clone, Debug)]
struct Component {
    id: String,
    kind: &'static str,
    name: String,
    text: String,
}

/// What the projection derives from the workspace before it writes a facet:
/// the components, their edges, the resolved calls and the routed enums.
/// Built once, then only read.
struct Model {
    ws: Workspace,
    /// `(package, target index)` of a product target -> component ID.
    targets: BTreeMap<(usize, usize), String>,
    components: Vec<Component>,
    /// Component ID -> index.
    index: BTreeMap<String, usize>,
    /// Edge ID -> (from component ID, to component ID).
    edges: BTreeMap<String, (String, String)>,
    /// Resolved calls: caller -> callees.
    calls: BTreeMap<FnRef, BTreeSet<FnRef>>,
    /// The route the router admitted for each enum: `(package, file, enum)`.
    enum_routes: BTreeMap<(usize, usize, usize), &'static str>,
    /// The bare names a function's module binds by a `use` to a type of a
    /// registry crate, each with the crate.
    registry: BTreeMap<FnRef, BTreeMap<String, String>>,
    /// The bare names each module, by package, file and inline module,
    /// binds by a `use` or a glob to an item of a registry crate, each with
    /// the path it names: the module that declares a type names its
    /// fields' types so, and the module of a function its signature's.
    registry_paths: BTreeMap<(usize, usize, String), BTreeMap<String, String>>,
    /// What each name several of a crate's types share names in each
    /// module that may write it, by package, file and inline module, as
    /// the language resolves it.
    type_readings: BTreeMap<(usize, usize, String), BTreeMap<String, crate::names::TypeReading>>,
    /// The public type aliases the registry crates the packages depend on
    /// declare at their roots, by path (`krate::Alias`), read from each
    /// crate's source at the version Cargo.lock pins.
    crate_aliases: BTreeMap<String, crate::ast::Alias>,
}

/// What the frontier records, as the facets find it.
#[derive(Default)]
struct Notes {
    refusals: BTreeSet<Refusal>,
    residue: BTreeMap<String, (Span, Residue)>,
    /// Functions that perform an operating-system effect, directly or
    /// through a function that does: the I/O shell, recorded with the effect
    /// instead of as residue.
    effects: BTreeMap<String, (Span, String)>,
    /// Functions at the boundary with a foreign crate: a parameter or the
    /// result is a type of a registry dependency that has no LexLean model.
    foreign: BTreeMap<String, (Span, String)>,
    /// Functions the lift took: the definition each lifted to, and where it
    /// parts from the Rust function, if anywhere.
    lifted: BTreeMap<String, (Span, String, Vec<String>)>,
    /// The acceptance roots each package's tests yield, by package name: the
    /// closed Boolean definitions the Behavior module proves true.
    roots: BTreeMap<String, Vec<String>>,
}

impl Model {
    fn package(&self, r: FnRef) -> &Package {
        &self.ws.packages[r.pkg]
    }

    fn file(&self, r: FnRef) -> &SourceFile {
        &self.ws.packages[r.pkg].files[r.file]
    }

    fn func(&self, r: FnRef) -> &FnFact {
        &self.file(r).fns[r.idx]
    }

    fn kind(&self, r: FnRef) -> &TargetKind {
        let file = self.file(r);
        &self.package(r).targets[file.target].kind
    }

    fn crate_ident(&self, r: FnRef) -> String {
        let file = self.file(r);
        self.package(r).targets[file.target].name.replace('-', "_")
    }

    /// `crate::module::Type::name`; and `crate::module::<Type as
    /// Trait>::name` for a trait implementation's method whose type has
    /// another method of its name, which is how Rust names the two apart.
    fn path(&self, r: FnRef) -> String {
        let file = self.file(r);
        let f = self.func(r);
        let mut parts = vec![self.crate_ident(r)];
        if !file.module.is_empty() {
            parts.push(file.module.clone());
        }
        if let Some(owner) = &f.owner {
            match &f.trait_ref {
                Some(named) if f.apart => parts.push(format!("<{owner} as {named}>")),
                _ => parts.push(owner.clone()),
            }
        }
        parts.push(f.name.clone());
        parts.join("::")
    }

    /// The registry crate a type names through a `use` of the function's
    /// module: `Value` where `use serde_json::Value;` binds it.
    fn bound_crate(&self, r: FnRef, ty: &str) -> Option<String> {
        let bound = self.registry.get(&r)?;
        ty.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':'))
            .filter_map(|word| word.split("::").next())
            .find_map(|head| bound.get(head).cloned())
    }

    /// The registry crate whose item a leaf's head path names, among the
    /// registry dependencies of every package (the package that declares
    /// the type the leaf is a field of is the one that depends on it):
    /// `big` for `big::BigX` and `big::Ratio<i64>`. A borrow reads as what
    /// it borrows; a head the lift does not read (`RefCell<big::BigX>`) is
    /// the leaf whatever it holds.
    fn registry_crate(&self, leaf: &str) -> Option<String> {
        let (krate, _) = leaf_head(leaf).split_once("::")?;
        self.ws
            .packages
            .iter()
            .flat_map(|p| &p.deps)
            .filter(|d| d.path.is_none())
            .map(|d| d.name.replace('-', "_"))
            .find(|dep| dep == krate)
    }

    fn component_of(&self, r: FnRef) -> Option<&String> {
        self.targets.get(&(r.pkg, self.file(r).target))
    }

    /// A function the lift refused: residue, or a foreign row when what
    /// stopped it is a type of a registry dependency.
    fn record_residue(&self, notes: &mut Notes, r: FnRef, span: Span, reason: Residue) {
        if let Residue::Type(ty) = &reason {
            // The leaf decides the row. A chain through the fields of the
            // workspace's types names its leaf as the file that declares
            // the last of them writes it; a type the function's own
            // signature writes is named as its own file binds it.
            let dep = match chain_leaf(ty) {
                Some(leaf) => self.registry_crate(leaf),
                None => self
                    .registry_crate(ty)
                    .or_else(|| self.bound_crate(r, leaf_head(ty))),
            };
            if let Some(dep) = dep {
                let detail = format!(
                    "takes or returns {ty}, a type of the {dep} crate with no LexLean model"
                );
                // A path two functions project to (a name collision) keeps
                // one row: residue, wherever one of them is residue.
                if !notes.residue.contains_key(&self.path(r)) {
                    notes.foreign.insert(self.path(r), (span, detail));
                }
                return;
            }
        }
        notes.foreign.remove(&self.path(r));
        notes.residue.insert(self.path(r), (span, reason));
    }

    /// A function the lift did not take: an effect row when it reaches an
    /// effect, otherwise residue or a foreign row.
    fn stopped(
        &self,
        notes: &mut Notes,
        effectful: &BTreeMap<FnRef, String>,
        r: FnRef,
        reason: Residue,
    ) {
        let span = self.func(r).span.clone();
        match effectful.get(&r) {
            Some(effect) => {
                notes.effects.insert(self.path(r), (span, effect.clone()));
            }
            None => self.record_residue(notes, r, span, reason),
        }
    }

    fn product(&self, r: FnRef) -> bool {
        matches!(self.kind(r), TargetKind::Lib | TargetKind::Bin)
            && !self.func(r).in_test_module
            && !self.func(r).test
    }

    fn all_fns(&self) -> Vec<FnRef> {
        let mut out = Vec::new();
        for (pkg, package) in self.ws.packages.iter().enumerate() {
            for (file, source) in package.files.iter().enumerate() {
                for idx in 0..source.fns.len() {
                    out.push(FnRef { pkg, file, idx });
                }
            }
        }
        out
    }

    fn component_suffix(id: &str) -> &str {
        id.strip_prefix("Architecture::component").unwrap_or(id)
    }
}

fn qualified(module: &str, name: &str) -> String {
    format!("{module}::{name}")
}

/// Records of one type, sorted by ID bytes, each knowing its index.
fn indexed<T>(mut rows: Vec<(String, T)>) -> Vec<(String, usize, T)> {
    rows.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    rows.into_iter()
        .enumerate()
        .map(|(i, (id, row))| (id, i, row))
        .collect()
}

fn local(id: &str) -> &str {
    id.rsplit("::").next().unwrap_or(id)
}

fn kebab(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// The whole projection: path -> bytes. There is none where the function
/// rows it would write are not a lift register, or where a module it would
/// write is larger than the limit its own `lexlean.toml` states.
pub(crate) fn project(
    ws: &Workspace,
    syntax: &Syntax,
    options: &Options,
    dumps: Option<&crate::mir::Dumps>,
) -> Result<BTreeMap<String, Vec<u8>>, UorustError> {
    let mut notes = Notes::default();
    // Items under `cfg` predicates no build holds together are one fact:
    // the projection states the variant its configuration compiles.
    let (ws, syntax, variants) = crate::names::stated(ws, syntax);
    let (ws, syntax) = (&ws, &syntax);
    notes.refusals.extend(variants);
    let resolver = crate::names::Resolver::new(ws, syntax);
    for package in &ws.packages {
        for file in &package.files {
            if let Some(reason) = &file.parse_error {
                let line = reason
                    .split(':')
                    .nth(1)
                    .and_then(|l| l.parse().ok())
                    .unwrap_or(1);
                notes.refusals.insert(Refusal::Unparsable {
                    span: Span {
                        file: file.rel.clone(),
                        line,
                    },
                    reason: reason.clone(),
                });
            }
        }
    }
    let (targets, components, index, refused) = components(ws);
    notes.refusals.extend(refused);
    let mut ctx = Model {
        ws: ws.clone(),
        targets,
        components,
        index,
        edges: BTreeMap::new(),
        calls: BTreeMap::new(),
        enum_routes: BTreeMap::new(),
        registry: BTreeMap::new(),
        registry_paths: BTreeMap::new(),
        type_readings: BTreeMap::new(),
        crate_aliases: BTreeMap::new(),
    };
    let registry_deps: BTreeSet<String> = ws
        .packages
        .iter()
        .flat_map(|p| &p.deps)
        .filter(|d| d.path.is_none())
        .map(|d| d.name.replace('-', "_"))
        .collect();
    let cargo_home = crate::registry::cargo_home();
    for krate in &registry_deps {
        // A crate whose source is not read declares no alias the lift
        // reads: a type it names stays a type of that crate.
        if let Ok(aliases) = crate::registry::crate_aliases(&cargo_home, krate, &ws.lock) {
            ctx.crate_aliases.extend(aliases);
        }
    }
    for (pkg, package) in ws.packages.iter().enumerate() {
        let mut declared: BTreeMap<&str, usize> = BTreeMap::new();
        for file in &package.files {
            let names = file.enums.iter().map(|e| e.name.as_str());
            for name in names.chain(file.structs.iter().map(|st| st.name.as_str())) {
                *declared.entry(name).or_default() += 1;
            }
        }
        let shared: Vec<&str> = declared
            .into_iter()
            .filter(|(_, n)| *n > 1)
            .map(|(name, _)| name)
            .collect();
        for (fi, file) in package.files.iter().enumerate() {
            for inline in syntax.names.inline_modules(&file.rel) {
                // The shared names, and each method's self type written by
                // a path (`super::T`), which only the module reads.
                let pathed = file
                    .fns
                    .iter()
                    .filter(|f| syntax.names.fn_module(&file.rel, f.span.line, &f.name) == inline)
                    .filter_map(|f| f.owner.as_deref())
                    .map(|o| o.split('<').next().unwrap_or(o).trim())
                    .filter(|o| o.contains("::"));
                let read: BTreeMap<String, crate::names::TypeReading> = shared
                    .iter()
                    .copied()
                    .chain(pathed)
                    .map(|n| (n.to_string(), resolver.type_reading(pkg, fi, &inline, n)))
                    .collect();
                if !read.is_empty() {
                    ctx.type_readings.insert((pkg, fi, inline), read);
                }
            }
        }
    }
    ctx.registry_paths = ws
        .packages
        .iter()
        .enumerate()
        .flat_map(|(pkg, package)| (0..package.files.len()).map(move |fi| (pkg, fi)))
        .flat_map(|(pkg, fi)| {
            let file = &ws.packages[pkg].files[fi];
            let written = bare_names(&syntax.files[&file.rel], file);
            let resolver = &resolver;
            syntax
                .names
                .inline_modules(&file.rel)
                .into_iter()
                .map(move |inline| {
                    // A `use` binds a name before a glob does.
                    let mut bound = resolver.registry_paths(pkg, fi, &inline);
                    for (name, path) in resolver.glob_paths(pkg, fi, &inline, &written) {
                        bound.entry(name).or_insert(path);
                    }
                    ((pkg, fi, inline), bound)
                })
        })
        .filter(|(_, bound)| !bound.is_empty())
        .collect();
    ctx.registry = ctx
        .all_fns()
        .into_iter()
        .map(|r| (r, resolver.registry_types((r.pkg, r.file, r.idx))))
        .filter(|(_, bound)| !bound.is_empty())
        .collect();
    ctx.edges = edges(&ctx);
    let (calls, refused) = resolve(&ctx, &resolver);
    ctx.calls = calls;
    notes.refusals.extend(refused);
    let (enum_routes, refused) = route_enums(&ctx);
    ctx.enum_routes = enum_routes;
    notes.refusals.extend(refused);
    let ctx = ctx;

    let prefix = pascal(&[&options.name]);
    let mut modules = Vec::new();
    let arch = architecture(&ctx);
    let (iface, refused) = interface(&ctx);
    notes.refusals.extend(refused);
    let (design, refused) = design(&ctx);
    notes.refusals.extend(refused);
    match &ws.advisories {
        None => {
            notes.refusals.insert(Refusal::AdvisoriesUnconsulted);
        }
        Some(db) => {
            for a in &db.hits {
                if !ws.has_deny && !ws.deny_ignores.contains_key(&a.id) {
                    notes.refusals.insert(Refusal::UnansweredAdvisory {
                        id: a.id.clone(),
                        package: a.package.clone(),
                        version: a.version.clone(),
                    });
                }
            }
        }
    }
    let security = security(&ctx, options);
    let mir = dumps.zip(options.mir.as_ref());
    let (behavior, candidates) = behavior(&ctx, syntax, &mut notes, mir);
    let supply = supply(&ctx);
    let library = library(ws, &prefix, &behavior, &mut notes);
    // The κ of every function of product code: the question its row answers.
    let kappas: BTreeMap<String, String> = ctx
        .all_fns()
        .into_iter()
        .filter(|&r| ctx.product(r))
        .map(|r| {
            let path = ctx.path(r);
            let kappa = crate::register::kappa(&path, &ctx.func(r).tokens);
            (path, kappa)
        })
        .collect();
    // The lift register of this run, read from the function rows the
    // frontier is about to carry: what the queue ranks and the Quality facet
    // measures is what a reader of the frontier finds.
    let rows = function_rows(&notes, &kappas);
    let lift = Register::parse(std::path::Path::new(FRONTIER), &rows)?;
    // Quality is written last of the facets: it measures what the others
    // admitted, refused and lifted.
    let (quality, refused) = quality(&ctx, iface.feature_count, &lift, &notes.refusals);
    notes.refusals.extend(refused);
    modules.push(arch);
    modules.push(iface.module);
    modules.push(design);
    modules.push(security);
    modules.push(quality);
    modules.push(behavior);
    modules.push(supply);
    modules.extend(library);
    // LexLean refuses a semantic module with no declaration (LLT4001), and a
    // workspace with nothing to lift (a fresh `cargo new`) leaves Behavior so.
    modules.retain(|m| !m.decls.is_empty());

    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for (path, bytes) in crate::assets::FIXED {
        files.insert((*path).to_string(), bytes.as_bytes().to_vec());
    }
    let mut entrypoints: Vec<String> = files
        .keys()
        .filter(|p| p.ends_with(MODULE_SUFFIX))
        .cloned()
        .collect();
    // Every file that is no generated module comes first: beside the modules
    // themselves, these are what can cite a record of one by its ID.
    for (path, bytes) in register(&iface.register) {
        files.insert(path, bytes);
    }
    files.insert("supply/spdx.json".into(), spdx(ws, options).into_bytes());
    // The prover's trial run lists the translations it admitted on trial.
    if let Some(listing) = candidates {
        files.insert(crate::mir::CANDIDATES.into(), listing.into_bytes());
    }
    files.insert(FRONTIER.into(), frontier(&notes, &lift, &rows).into_bytes());
    let cited = |id: &str| {
        files
            .values()
            .any(|bytes| String::from_utf8_lossy(bytes).contains(id))
    };
    // Each module within the limit the project file below states, in parts
    // where it is beyond it and its declarations can be written apart.
    let written = sized(&modules, MAX_FILE_BYTES, &cited);
    for (path, text) in written {
        entrypoints.push(path.clone());
        files.insert(path, text.into_bytes());
    }
    entrypoints.sort();
    let toml_name: String = options
        .name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let lexlean = format!(
        "spec = \"lexlean/project/1\"\nname = \"{toml_name}\"\nlanguage = \"1.2\"\n\
         module_prefix = \"{prefix}\"\nsource_roots = [\"src\"]\nentrypoints = [{}]\n\
         build_root = \".lexlean\"\nlockfile = \"lexlean.lock\"\nlean_workspace = \".\"\n\
         lean_toolchain = \"leanprover/lean4:v4.32.1\"\n\n\
         [[lexicon_source]]\npackage = \"lexlean.std.nat\"\nkind = \"builtin\"\n\n\
         [[lexicon_source]]\npackage = \"prism.arch\"\nkind = \"path\"\npath = \"language/prism.arch\"\n\n\
         [[lexicon_source]]\npackage = \"prism.qual\"\nkind = \"path\"\npath = \"language/prism.qual\"\n\n\
         [[lexicon_source]]\npackage = \"prism.sec\"\nkind = \"path\"\npath = \"language/prism.sec\"\n\n\
         [[lexicon_source]]\npackage = \"uorust.titles\"\nkind = \"path\"\npath = \"language/uorust.titles\"\n\n\
         [limits]\nmax_file_bytes = {MAX_FILE_BYTES}\nmax_total_source_bytes = 67108864\n\
         max_primitive_atoms = 2000000\nmax_token_lattice_edges = 4000000\nmax_parse_states = 4000000\n\
         max_ir_nodes = 2000000\nmax_scope_depth = 1024\nmax_import_depth = 128\nmax_diagnostics = 256\n\
         max_child_output_bytes = 16777216\nchild_timeout_ms = 300000\n",
        entrypoints
            .iter()
            .map(|e| format!("\"{e}\""))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let lakefile = format!(
        "name = \"{prefix}\"\nversion = \"0.1.0\"\ndefaultTargets = [\"{prefix}\"]\n\n[[lean_lib]]\nname = \"{prefix}\"\n"
    );
    let manifest = format!(
        "{{\"version\":\"1.2.0\",\"packagesDir\":\".lake/packages\",\"packages\":[],\"name\":\"{prefix}\",\"lakeDir\":\".lake\",\"fixedToolchain\":false}}\n"
    );
    let toolchain = files.get("lean-toolchain").cloned().unwrap_or_default();
    let lock = format!(
        "spec = \"lexlean/lock/2\"\nlanguage = \"1.2\"\n\
         compiler_semantics = \"{}\"\n\
         project_config_sha256 = \"{}\"\nlean_toolchain = \"leanprover/lean4:v4.32.1\"\n\n\
         [[workspace_file]]\npath = \"lake-manifest.json\"\nsha256 = \"{}\"\n\n\
         [[workspace_file]]\npath = \"lakefile.toml\"\nsha256 = \"{}\"\n\n\
         [[workspace_file]]\npath = \"lean-toolchain\"\nsha256 = \"{}\"\n\n{}",
        compiler_semantics(),
        sha(lexlean.as_bytes()),
        sha(manifest.as_bytes()),
        sha(lakefile.as_bytes()),
        sha(&toolchain),
        crate::assets::LOCK_PACKAGES
    );
    let prismpm = format!(
        "spec = \"prismpm/project/1\"\nproject = \"{prefix}\"\nlexlean_project = \"lexlean.toml\"\n\
         build_root = \".prism\"\n\n[limits]\nmax_holo_bytes = 16777216\nmax_entities = 100000\n\
         max_diagnostics = 256\n"
    );
    files.insert(LEXLEAN.into(), lexlean.into_bytes());
    files.insert("lexlean.lock".into(), lock.into_bytes());
    files.insert("lakefile.toml".into(), lakefile.into_bytes());
    files.insert("lake-manifest.json".into(), manifest.into_bytes());
    files.insert("prismpm.toml".into(), prismpm.into_bytes());
    files.insert(".gitattributes".into(), b"* -text\n".to_vec());
    files.insert(
        ".gitignore".into(),
        b"/.lexlean/\n/.prism/\n/.lake/\n".to_vec(),
    );
    within_limit(&files)?;
    Ok(files)
}

/// The LexLean compiler semantics a projection locks to: the one UORust's own
/// self model is locked to (`app/lexlean.lock`, which `lexlean lock` in the
/// locked SDK rewrites), so a projection follows the SDK's compiler without a
/// copied identity.
fn compiler_semantics() -> &'static str {
    const SELF_LOCK: &str = include_str!("../../../app/lexlean.lock");
    SELF_LOCK
        .lines()
        .find_map(|line| line.strip_prefix("compiler_semantics = \""))
        .and_then(|rest| rest.strip_suffix('"'))
        .expect("app/lexlean.lock names its compiler semantics")
}

/// The native-library profile's bound on each root list (SPEC 12.6: 1..1024
/// roots), which PrismPM's projection enforces.
const LIBRARY_ROOTS: usize = 1024;

/// The crate's profile when it is a library (SPEC 12.6): the one
/// `Foundation.Library.V1.Model::NativeLibrary` value of the project, its
/// metadata from Cargo, its exports every lifted definition and its
/// acceptance roots the closed Boolean claims its tests make (UR-R29).
///
/// A slot Cargo leaves empty is inferred where Cargo itself defines the
/// default: the homepage is the repository when only that is declared
/// (Cargo's manifest reference: a homepage distinct from the source
/// repository), and the description is the crate doc's summary line, the
/// text rustdoc and crates.io show for the crate. A slot no rule defines is
/// an undefined slot in the register, and no profile is written.
fn library(ws: &Workspace, prefix: &str, behavior: &Module, notes: &mut Notes) -> Option<Module> {
    let libraries: Vec<&Package> = ws
        .packages
        .iter()
        .filter(|p| p.targets.iter().any(|t| t.kind == TargetKind::Lib))
        .collect();
    let [package] = libraries.as_slice() else {
        if libraries.len() > 1 {
            for package in libraries {
                notes.refusals.insert(Refusal::UndefinedSlot {
                    package: package.name.clone(),
                    slot: "library",
                    reason: "a project holds one NativeLibrary value and the workspace has \
                             more than one library package"
                        .to_string(),
                });
            }
        }
        return None;
    };
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
    // The library's exports are the definitions its own functions lifted to
    // (their crate path begins with the lib target's crate name) and its
    // acceptance roots; every acceptance root is an export (SPEC 12.6).
    let lib_crate = package
        .targets
        .iter()
        .find(|t| t.kind == TargetKind::Lib)
        .map(|t| format!("{}::", t.name.replace('-', "_")))
        .unwrap_or_default();
    let defined: BTreeSet<&str> = behavior
        .decls
        .iter()
        .filter_map(|decl| match decl {
            Decl::Definition { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    let in_behavior = |q: &str| q.strip_prefix(&qualified(BEHAVIOR, "")).map(str::to_string);
    let mut acceptance: Vec<String> = notes
        .roots
        .get(&package.name)
        .into_iter()
        .flatten()
        .filter(|root| defined.contains(root.as_str()))
        .map(|root| format!("{prefix}.{BEHAVIOR}.{root}"))
        .collect();
    let mut exports: Vec<String> = notes
        .lifted
        .iter()
        .filter(|(path, _)| path.starts_with(&lib_crate))
        .filter_map(|(_, (_, q, _))| in_behavior(q))
        .filter(|name| defined.contains(name.as_str()))
        .map(|name| format!("{prefix}.{BEHAVIOR}.{name}"))
        .chain(acceptance.iter().cloned())
        .collect();
    exports.dedup();
    exports.sort();
    exports.dedup();
    acceptance.sort();
    let mut undefined = Vec::new();
    for (slot, value) in [
        ("description", &description),
        ("repository", &package.repository),
        ("homepage", &homepage),
    ] {
        if value.trim().is_empty() {
            undefined.push((
                slot,
                format!("Cargo declares no {slot} and no rule infers one"),
            ));
        }
    }
    for (slot, roots) in [("exportRoots", &exports), ("acceptanceRoots", &acceptance)] {
        if roots.is_empty() {
            undefined.push((slot, format!("the crate yields no {slot}")));
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
        list(&exports),
        list(&acceptance),
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

/// The SDK module that declares the native-library profile's type.
const LIBRARY_MODEL: &str = "Foundation.Library.V1.Model";

pub(crate) fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The components: each product target and each third-party dependency, with
/// the component ID of each product target and each component's index.
#[allow(clippy::type_complexity)]
fn components(
    ws: &Workspace,
) -> (
    BTreeMap<(usize, usize), String>,
    Vec<Component>,
    BTreeMap<String, usize>,
    Vec<Refusal>,
) {
    let mut targets = BTreeMap::new();
    let mut refusals = Vec::new();
    let mut rows: Vec<(String, Component)> = Vec::new();
    let internal: BTreeSet<&str> = ws.packages.iter().map(|p| p.name.as_str()).collect();
    for (pkg, package) in ws.packages.iter().enumerate() {
        for (t, target) in package.targets.iter().enumerate() {
            let (kind, tag) = match target.kind {
                TargetKind::Lib => ("service", "Lib"),
                TargetKind::Bin => ("releaseBinary", "Bin"),
                _ => continue,
            };
            let root = package.files.iter().find(|f| f.rel == target.src);
            let doc = root.map(|f| summary(&f.doc)).unwrap_or_default();
            let text = if doc.is_empty() {
                package.description.clone()
            } else {
                doc
            };
            let id = qualified(ARCH, &format!("component{tag}{}", pascal(&[&target.name])));
            targets.insert((pkg, t), id.clone());
            rows.push((
                id.clone(),
                Component {
                    id,
                    kind,
                    name: target.name.clone(),
                    text,
                },
            ));
        }
        for dep in &package.deps {
            if dep.kind != "normal" || internal.contains(dep.name.as_str()) {
                continue;
            }
            if dep.path.is_some() {
                if !dep.inside {
                    refusals.push(Refusal::ExternalPath {
                        package: package.dir.clone(),
                        dependency: dep.name.clone(),
                    });
                }
                continue;
            }
            let id = qualified(ARCH, &format!("componentExt{}", pascal(&[&dep.name])));
            let version = ws
                .lock
                .iter()
                .filter(|l| l.name == dep.name)
                .map(|l| l.version.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let text = if version.is_empty() {
                format!("Third-party crate {} from crates.io.", dep.name)
            } else {
                format!(
                    "Third-party crate {} {version}, locked in Cargo.lock.",
                    dep.name
                )
            };
            rows.push((
                id.clone(),
                Component {
                    id,
                    kind: "externalSystem",
                    name: dep.name.clone(),
                    text,
                },
            ));
        }
    }
    rows.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    rows.dedup_by(|a, b| a.0 == b.0);
    let mut index = BTreeMap::new();
    let mut components = Vec::new();
    for (i, (id, row)) in rows.into_iter().enumerate() {
        index.insert(id, i);
        components.push(row);
    }
    (targets, components, index, refusals)
}

fn edges(ctx: &Model) -> BTreeMap<String, (String, String)> {
    let ws = &ctx.ws;
    let mut edges = BTreeMap::new();
    let lib_of: BTreeMap<&str, String> = ws
        .packages
        .iter()
        .enumerate()
        .filter_map(|(pkg, p)| {
            p.targets
                .iter()
                .position(|t| t.kind == TargetKind::Lib)
                .and_then(|t| ctx.targets.get(&(pkg, t)))
                .map(|id| (p.name.as_str(), id.clone()))
        })
        .collect();
    for (pkg, package) in ws.packages.iter().enumerate() {
        for (t, target) in package.targets.iter().enumerate() {
            let Some(from) = ctx.targets.get(&(pkg, t)).cloned() else {
                continue;
            };
            let mut to = Vec::new();
            if target.kind == TargetKind::Bin {
                if let Some(own) = lib_of.get(package.name.as_str()) {
                    to.push(own.clone());
                }
            }
            for dep in &package.deps {
                if dep.kind != "normal" {
                    continue;
                }
                if let Some(lib) = lib_of.get(dep.name.as_str()) {
                    to.push(lib.clone());
                } else if dep.path.is_none() {
                    to.push(qualified(
                        ARCH,
                        &format!("componentExt{}", pascal(&[&dep.name])),
                    ));
                }
            }
            for target_id in to {
                if target_id == from || !ctx.index.contains_key(&target_id) {
                    continue;
                }
                let id = qualified(
                    ARCH,
                    &format!(
                        "edge{}To{}",
                        Model::component_suffix(&from),
                        Model::component_suffix(&target_id)
                    ),
                );
                edges.insert(id, (from.clone(), target_id));
            }
        }
    }
    edges
}

/// Resolve every call to workspace functions: by the language's own name
/// resolution where the written code decides it ([`crate::names`]), and by
/// name where those rules could not follow the call.
#[allow(clippy::type_complexity)]
fn resolve(
    ctx: &Model,
    resolver: &crate::names::Resolver<'_>,
) -> (BTreeMap<FnRef, BTreeSet<FnRef>>, Vec<Refusal>) {
    use crate::names::Outcome;
    let mut calls = BTreeMap::new();
    let mut refusals = Vec::new();
    let all = ctx.all_fns();
    let mut by_name: BTreeMap<&str, Vec<FnRef>> = BTreeMap::new();
    for &r in &all {
        if ctx.product(r) {
            by_name
                .entry(ctx.func(r).name.as_str())
                .or_default()
                .push(r);
        }
    }
    let crate_idents: BTreeMap<String, usize> = ctx
        .ws
        .packages
        .iter()
        .enumerate()
        .flat_map(|(pkg, p)| {
            p.targets
                .iter()
                .filter(|t| t.kind == TargetKind::Lib)
                .map(move |t| (t.name.replace('-', "_"), pkg))
        })
        .collect();
    let fn_ref = |(pkg, file, idx): crate::names::FnId| FnRef { pkg, file, idx };
    for &caller in &all {
        let f = ctx.func(caller);
        let mut out = BTreeSet::new();
        let ruled: Vec<(String, Option<Outcome>)> =
            match resolver.calls((caller.pkg, caller.file, caller.idx)) {
                Some(list) => list.into_iter().map(|(c, o)| (c, Some(o))).collect(),
                None => f.calls.iter().map(|c| (c.clone(), None)).collect(),
            };
        for (call, outcome) in &ruled {
            // Product code's functions only, as by name below.
            let product = |fns: &[crate::names::FnId]| -> Vec<FnRef> {
                let mut v: Vec<FnRef> = fns.iter().map(|g| fn_ref(*g)).collect();
                v.retain(|r| ctx.product(*r));
                v
            };
            match outcome {
                // One function, or the variants of one no build holds
                // together: the call reaches each.
                Some(Outcome::Resolved(fns)) => {
                    out.extend(product(fns));
                    continue;
                }
                Some(Outcome::Outside) => continue,
                // The written code leaves it open between these.
                Some(Outcome::Open(fns)) => {
                    match product(fns).as_slice() {
                        [] => {}
                        [only] => {
                            out.insert(*only);
                        }
                        many => {
                            let mut names: Vec<String> = many
                                .iter()
                                .map(|r| {
                                    format!("{} ({})", ctx.path(*r), ctx.func(*r).span.render())
                                })
                                .collect();
                            names.sort();
                            refusals.push(Refusal::AmbiguousCall {
                                span: f.span.clone(),
                                callee: call.clone(),
                                candidates: names,
                            });
                        }
                    }
                    continue;
                }
                Some(Outcome::Unknown) | None => {}
            }
            if let Some(method) = call.strip_prefix('.') {
                let candidates: Vec<FnRef> = by_name
                    .get(method)
                    .map(|v| {
                        v.iter()
                            .copied()
                            .filter(|r| ctx.func(*r).owner.is_some())
                            .collect()
                    })
                    .unwrap_or_default();
                // A method a trait declares, with every other candidate an
                // implementation of that trait, is a call of the trait's
                // interface.
                let declared: Vec<FnRef> = candidates
                    .iter()
                    .copied()
                    .filter(|r| ctx.func(*r).in_trait)
                    .collect();
                if let [decl] = declared.as_slice() {
                    let name = ctx.func(*decl).owner.clone();
                    let through = candidates
                        .iter()
                        .all(|r| r == decl || ctx.func(*r).implements == name);
                    if through {
                        out.insert(*decl);
                        continue;
                    }
                }
                if let [only] = candidates.as_slice() {
                    out.insert(*only);
                }
                continue;
            }
            let imports = &ctx.file(caller).imports;
            let expanded = match call.split_once("::") {
                Some((head, rest)) => imports.get(head).map(|p| format!("{p}::{rest}")),
                None => imports.get(call.as_str()).cloned(),
            };
            let call = expanded.as_ref().unwrap_or(call);
            let segments: Vec<&str> = call.split("::").collect();
            let Some(name) = segments.last() else {
                continue;
            };
            let Some(found) = by_name.get(name) else {
                continue;
            };
            let mut candidates: Vec<FnRef> = found.clone();
            if segments.len() >= 2 {
                let first = segments[0];
                let owner = segments[segments.len() - 2];
                if let Some(&pkg) = crate_idents.get(first) {
                    candidates.retain(|r| r.pkg == pkg && ctx.kind(*r) == &TargetKind::Lib);
                } else if first.starts_with(|c: char| c.is_ascii_lowercase()) {
                    // `crate::`, `self::`, `super::` or a local module path:
                    // only a function of the caller's own target whose module
                    // ends with that path. Anything else (`std::fs::read`, an
                    // external crate) is not a workspace call.
                    let module: Vec<&str> = segments[..segments.len() - 1]
                        .iter()
                        .copied()
                        .filter(|s| !matches!(*s, "crate" | "self" | "super"))
                        .filter(|s| !s.starts_with(|c: char| c.is_ascii_uppercase()))
                        .collect();
                    let module = module.join("::");
                    candidates.retain(|r| {
                        r.pkg == caller.pkg
                            && ctx.file(*r).target == ctx.file(caller).target
                            && (module.is_empty() || ctx.file(*r).module.ends_with(&module))
                    });
                }
                if owner == "Self" {
                    let own = f.owner.clone();
                    candidates.retain(|r| ctx.func(*r).owner == own);
                } else if owner.starts_with(|c: char| c.is_ascii_uppercase()) {
                    candidates.retain(|r| {
                        ctx.func(*r)
                            .owner
                            .as_deref()
                            .is_some_and(|o| o.split('<').next() == Some(owner))
                    });
                } else {
                    candidates.retain(|r| ctx.func(*r).owner.is_none());
                }
                if candidates.len() > 1 {
                    let module = segments[..segments.len() - 1]
                        .iter()
                        .skip(usize::from(crate_idents.contains_key(first)))
                        .filter(|s| !matches!(**s, "crate" | "self" | "super"))
                        .filter(|s| !s.starts_with(|c: char| c.is_ascii_uppercase()))
                        .copied()
                        .collect::<Vec<_>>()
                        .join("::");
                    let narrowed: Vec<FnRef> = candidates
                        .iter()
                        .copied()
                        .filter(|r| ctx.file(*r).module.ends_with(&module))
                        .collect();
                    if !narrowed.is_empty() {
                        candidates = narrowed;
                    }
                }
            } else {
                candidates.retain(|r| ctx.func(*r).owner.is_none());
                let same_file: Vec<FnRef> = candidates
                    .iter()
                    .copied()
                    .filter(|r| r.pkg == caller.pkg && r.file == caller.file)
                    .collect();
                if !same_file.is_empty() {
                    candidates = same_file;
                } else {
                    candidates.retain(|r| {
                        r.pkg == caller.pkg && ctx.file(*r).target == ctx.file(caller).target
                    });
                }
            }
            match candidates.as_slice() {
                [] => {}
                [only] => {
                    out.insert(*only);
                }
                many => {
                    let mut names: Vec<String> = many
                        .iter()
                        .map(|r| format!("{} ({})", ctx.path(*r), ctx.func(*r).span.render()))
                        .collect();
                    names.sort();
                    refusals.push(Refusal::AmbiguousCall {
                        span: f.span.clone(),
                        callee: call.clone(),
                        candidates: names,
                    });
                }
            }
        }
        if !out.is_empty() {
            calls.insert(caller, out);
        }
    }
    (calls, refusals)
}

fn record(ty: &str, fields: &[(&str, String)]) -> String {
    let fields: Vec<String> = fields.iter().map(|(k, v)| format!("{k} := {v}")).collect();
    format!("(record {ty} {{{}}})", fields.join(", "))
}

fn architecture(ctx: &Model) -> Module {
    let ws = &ctx.ws;
    let t = "Foundation.Arch::";
    let mut decls = Vec::new();
    for (i, c) in ctx.components.iter().enumerate() {
        decls.push(Decl::value(
            local(&c.id),
            format!("{t}Component"),
            record(
                &format!("{t}Component"),
                &[
                    ("index", i.to_string()),
                    ("kind", format!("(ctor {t}ComponentKind.{})", c.kind)),
                    ("name", string(&c.name)),
                    ("text", string(&c.text)),
                ],
            ),
        ));
    }
    for (id, i, (from, to)) in indexed(
        ctx.edges
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    ) {
        decls.push(Decl::value(
            local(&id),
            format!("{t}Edge"),
            record(
                &format!("{t}Edge"),
                &[
                    ("index", i.to_string()),
                    ("fromIndex", ctx.index[&from].to_string()),
                    ("toIndex", ctx.index[&to].to_string()),
                    ("kind", format!("(ctor {t}EdgeKind.controlFlow)")),
                ],
            ),
        ));
    }
    let (stakeholders, concerns, viewpoints) = audience(ctx);
    for (id, i, (name, text)) in indexed(stakeholders) {
        decls.push(Decl::value(
            local(&id),
            format!("{t}Stakeholder"),
            record(
                &format!("{t}Stakeholder"),
                &[
                    ("index", i.to_string()),
                    ("name", string(&name)),
                    ("text", string(&text)),
                ],
            ),
        ));
    }
    let concern_rows = indexed(concerns);
    for (id, i, (name, text)) in &concern_rows {
        decls.push(Decl::value(
            local(id),
            format!("{t}Concern"),
            record(
                &format!("{t}Concern"),
                &[
                    ("index", i.to_string()),
                    ("name", string(name)),
                    ("text", string(text)),
                ],
            ),
        ));
    }
    let (stakeholder_index, concern_index) = audience_index(ctx);
    let viewpoint_rows = indexed(viewpoints);
    for (id, i, (stakeholder, concern, model)) in &viewpoint_rows {
        decls.push(Decl::value(
            local(id),
            format!("{t}Viewpoint"),
            record(
                &format!("{t}Viewpoint"),
                &[
                    ("index", i.to_string()),
                    (
                        "stakeholderIndex",
                        stakeholder_index[stakeholder].to_string(),
                    ),
                    ("concernIndex", concern_index[concern].to_string()),
                    ("modelKindIndex", model.to_string()),
                ],
            ),
        ));
    }
    for (id, i, (_, _, model)) in &viewpoint_rows {
        let view = id.replace("::viewpoint", "::view");
        decls.push(Decl::value(
            local(&view),
            format!("{t}View"),
            record(
                &format!("{t}View"),
                &[
                    ("index", i.to_string()),
                    ("viewpointIndex", i.to_string()),
                    ("modelKindIndex", model.to_string()),
                ],
            ),
        ));
    }
    for (name, index, id) in [
        ("standardQuality", 0, "ISO-25010-2023"),
        ("standardRiskManagement", 1, "ISO-27005-2022"),
        ("standardApplicationSecurity", 2, "ISO-27034-1-2011"),
        ("standardSecurityControls", 3, "ISO-27034-5-2017"),
        ("standardArchitectureDescription", 4, "ISO-42010-2022"),
    ] {
        let edition = id.rsplit('-').next().unwrap_or_default();
        decls.push(Decl::value(
            name,
            format!("{t}StandardReference"),
            record(
                &format!("{t}StandardReference"),
                &[
                    ("standardIndex", index.to_string()),
                    ("editionIndex", "0".to_string()),
                    ("standardId", string(id)),
                    ("edition", string(edition)),
                ],
            ),
        ));
    }
    let _ = ws;
    Module {
        name: ARCH.into(),
        glossaries: vec![BOOL, NAT, PARCH],
        imports: vec!["Foundation.Arch".into()],
        title: "architectural component",
        decls,
    }
}

type Named = Vec<(String, (String, String))>;
type Viewpoints = Vec<(String, (String, String, usize))>;

fn has_kind(ws: &Workspace, kind: TargetKind) -> bool {
    ws.packages
        .iter()
        .any(|p| p.targets.iter().any(|t| t.kind == kind))
}

/// Stakeholders, concerns and viewpoints, each derived from what the workspace has.
fn audience(ctx: &Model) -> (Named, Named, Viewpoints) {
    let ws = &ctx.ws;
    let a = |n: &str| qualified(ARCH, n);
    let mut authors: Vec<String> = ws.packages.iter().flat_map(|p| p.authors.clone()).collect();
    authors.sort();
    authors.dedup();
    let maintainers = if authors.is_empty() {
        format!(
            "Maintain the {} workspace and its {} packages.",
            ws.name,
            ws.packages.len()
        )
    } else {
        format!(
            "Maintain the {} workspace: {}.",
            ws.name,
            authors.join(", ")
        )
    };
    let mut stakeholders = vec![
        (
            a("stakeholderMaintainers"),
            ("Maintainers".to_string(), maintainers),
        ),
        (
            a("stakeholderSecurityReviewers"),
            (
                "Security reviewers".to_string(),
                "Judge what the code can reach beyond computation and what the build trusts."
                    .to_string(),
            ),
        ),
        (
            a("stakeholderQualityReviewers"),
            (
                "Quality reviewers".to_string(),
                "Judge documentation, tests, lints and compatibility.".to_string(),
            ),
        ),
    ];
    if has_kind(ws, TargetKind::Bin) {
        stakeholders.push((
            a("stakeholderCliUsers"),
            (
                "Command-line users".to_string(),
                "Run the workspace's binaries.".to_string(),
            ),
        ));
    }
    if has_kind(ws, TargetKind::Lib) {
        stakeholders.push((
            a("stakeholderLibraryUsers"),
            (
                "Library users".to_string(),
                "Call the public interface of the workspace's libraries.".to_string(),
            ),
        ));
    }
    let description = ws
        .packages
        .iter()
        .map(|p| p.description.clone())
        .filter(|d| !d.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    // UR-R04: every package author is a stakeholder.
    for author in &authors {
        let name = author
            .split('<')
            .next()
            .unwrap_or(author)
            .trim()
            .to_string();
        stakeholders.push((
            a(&format!("stakeholderAuthor{}", pascal(&[&name]))),
            (
                name,
                format!("Listed as an author of the {} workspace.", ws.name),
            ),
        ));
    }
    let mut concerns = vec![
        (
            a("concernFunctionality"),
            (
                "Functionality".to_string(),
                if description.is_empty() {
                    "What the public interface does and how it is accepted.".to_string()
                } else {
                    description
                },
            ),
        ),
        (
            a("concernSecurity"),
            (
                "Security".to_string(),
                "Files, network, processes, environment and unsafe code the targets reach."
                    .to_string(),
            ),
        ),
        (
            a("concernQuality"),
            (
                "Quality".to_string(),
                "Documentation coverage, tests, lint levels and the minimum supported Rust."
                    .to_string(),
            ),
        ),
        (
            a("concernSupplyChain"),
            (
                "Supply chain".to_string(),
                format!(
                    "The {} packages Cargo.lock pins and their sources.",
                    ws.lock.len()
                ),
            ),
        ),
    ];
    // UR-R03: a crate's doc comment states the concern the crate answers.
    for (pkg, package) in ws.packages.iter().enumerate() {
        for (t, target) in package.targets.iter().enumerate() {
            let Some(component) = ctx.targets.get(&(pkg, t)) else {
                continue;
            };
            let doc = package
                .files
                .iter()
                .find(|f| f.rel == target.src)
                .map(|f| summary(&f.doc))
                .unwrap_or_default();
            if !doc.is_empty() {
                concerns.push((
                    a(&format!(
                        "concernCrate{}",
                        Model::component_suffix(component)
                    )),
                    (target.name.clone(), doc),
                ));
            }
        }
    }
    let user = if has_kind(ws, TargetKind::Bin) {
        a("stakeholderCliUsers")
    } else if has_kind(ws, TargetKind::Lib) {
        a("stakeholderLibraryUsers")
    } else {
        a("stakeholderMaintainers")
    };
    let viewpoints = vec![
        (
            a("viewpointMaintenance"),
            (a("stakeholderMaintainers"), a("concernFunctionality"), 0),
        ),
        (a("viewpointUsage"), (user, a("concernFunctionality"), 0)),
        (
            a("viewpointQuality"),
            (a("stakeholderQualityReviewers"), a("concernQuality"), 1),
        ),
        (
            a("viewpointSecurity"),
            (a("stakeholderSecurityReviewers"), a("concernSecurity"), 2),
        ),
        (
            a("viewpointSupplyChain"),
            (
                a("stakeholderSecurityReviewers"),
                a("concernSupplyChain"),
                2,
            ),
        ),
    ];
    (stakeholders, concerns, viewpoints)
}

fn audience_index(ctx: &Model) -> (BTreeMap<String, usize>, BTreeMap<String, usize>) {
    let (s, c, _) = audience(ctx);
    (
        indexed(s).into_iter().map(|(id, i, _)| (id, i)).collect(),
        indexed(c).into_iter().map(|(id, i, _)| (id, i)).collect(),
    )
}

struct Interface {
    module: Module,
    feature_count: usize,
    register: Vec<RegisterRow>,
}

struct RegisterRow {
    id: String,
    suite: String,
    record: String,
    statement: String,
    calls: Vec<String>,
    acceptance: Vec<String>,
}

struct Feature {
    id: String,
    statement: String,
    calls: Vec<String>,
    acceptance: Vec<String>,
    span: Span,
    suite: String,
    journey: Option<(String, String)>,
}

/// Public functions of a library, in their order of declaration.
fn interface_fns(ctx: &Model) -> Vec<FnRef> {
    ctx.all_fns()
        .into_iter()
        .filter(|r| {
            let f = ctx.func(*r);
            ctx.product(*r) && f.public && ctx.kind(*r) == &TargetKind::Lib
        })
        .collect()
}

fn call_name(ctx: &Model, r: FnRef) -> String {
    let path = ctx.path(r);
    let parts: Vec<&str> = path.split("::").collect();
    qualified(IFACE, &format!("call{}", pascal(&parts)))
}

/// Whether a string literal of `r`, or of a function `r` reaches through
/// resolved calls, satisfies `pred`: a test that runs the binary through a
/// helper still runs it.
fn reaches_literal(ctx: &Model, r: FnRef, pred: impl Fn(&String) -> bool) -> bool {
    let mut seen = BTreeSet::new();
    let mut stack = vec![r];
    while let Some(f) = stack.pop() {
        if !seen.insert(f) {
            continue;
        }
        if ctx.func(f).literals.iter().any(&pred) {
            return true;
        }
        stack.extend(ctx.calls.get(&f).into_iter().flatten().copied());
        // Helpers of a test file are not workspace functions the resolver
        // admits; a bare call to one in the same file still reaches it.
        let file = ctx.file(f);
        for call in &ctx.func(f).calls {
            for (idx, g) in file.fns.iter().enumerate() {
                if g.owner.is_none() && &g.name == call {
                    stack.push(FnRef { idx, ..f });
                }
            }
        }
    }
    false
}

fn interface(ctx: &Model) -> (Interface, Vec<Refusal>) {
    let ws = &ctx.ws;
    let mut refusals = Vec::new();
    let mut decls: Vec<Decl> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut push = |refusals: &mut Vec<Refusal>, decl: Decl, span: &Span| -> bool {
        if seen.insert(decl.name().to_string()) {
            decls.push(decl);
            true
        } else {
            refusals.push(Refusal::Collision {
                span: span.clone(),
                name: qualified(IFACE, decl.name()),
            });
            false
        }
    };
    // Acceptance rows: tests, doctests, examples.
    let mut tests: Vec<(FnRef, String)> = Vec::new();
    for r in ctx.all_fns() {
        let f = ctx.func(r);
        if !f.test {
            continue;
        }
        let file = ctx.file(r);
        let package = ctx.package(r);
        let target = &package.targets[file.target];
        let parts = [
            package.name.as_str(),
            target.name.as_str(),
            file.module.as_str(),
            f.name.as_str(),
        ];
        let name = format!("ac{}", pascal(&parts));
        let doc = summary(&f.doc);
        let text = if doc.is_empty() {
            format!("{} at {} passes.", f.name, f.span.render())
        } else {
            format!("{} at {}: {doc}", f.name, f.span.render())
        };
        if push(
            &mut refusals,
            Decl::value(name.clone(), "String", string(&text)),
            &f.span,
        ) {
            tests.push((r, qualified(IFACE, &name)));
        }
    }
    let mut doctests: BTreeMap<FnRef, String> = BTreeMap::new();
    for r in interface_fns(ctx) {
        let f = ctx.func(r);
        if f.doctests == 0 {
            continue;
        }
        let path = ctx.path(r);
        let parts: Vec<&str> = path.split("::").collect();
        let name = format!("acDoc{}", pascal(&parts));
        let text = format!(
            "The {} doctest(s) of {path} at {} pass.",
            f.doctests,
            f.span.render()
        );
        if push(
            &mut refusals,
            Decl::value(name.clone(), "String", string(&text)),
            &f.span,
        ) {
            doctests.insert(r, qualified(IFACE, &name));
        }
    }
    // Calls.
    let mut call_of: BTreeMap<FnRef, String> = BTreeMap::new();
    for r in interface_fns(ctx) {
        let f = ctx.func(r);
        let id = call_name(ctx, r);
        let text = format!("{}: {}", ctx.path(r), f.signature);
        if push(
            &mut refusals,
            Decl::value(local(&id), "String", string(&text)),
            &f.span,
        ) {
            call_of.insert(r, id);
        }
    }
    let mut features: Vec<Feature> = Vec::new();
    // Library modules with public functions.
    let mut by_module: BTreeMap<(usize, usize, String), Vec<FnRef>> = BTreeMap::new();
    for r in call_of.keys() {
        let file = ctx.file(*r);
        by_module
            .entry((r.pkg, file.target, file.module.clone()))
            .or_default()
            .push(*r);
    }
    for ((pkg, target, module), fns) in &by_module {
        let package = &ws.packages[*pkg];
        let crate_name = package.targets[*target].name.replace('-', "_");
        let module_file = package
            .files
            .iter()
            .find(|f| f.target == *target && &f.module == module);
        let doc = module_file.map(|f| summary(&f.doc)).unwrap_or_default();
        let path = if module.is_empty() {
            crate_name.clone()
        } else {
            format!("{crate_name}::{module}")
        };
        let names: Vec<String> = fns.iter().map(|r| ctx.func(*r).name.clone()).collect();
        let statement = if doc.is_empty() {
            format!("{path} offers {}.", names.join(", "))
        } else {
            format!("{path}: {doc}")
        };
        let mut acceptance: BTreeSet<String> = BTreeSet::new();
        let set: BTreeSet<FnRef> = fns.iter().copied().collect();
        for (test, row) in &tests {
            let reaches = ctx.calls.get(test).is_some_and(|c| !c.is_disjoint(&set));
            if reaches {
                acceptance.insert(row.clone());
            }
        }
        for r in fns {
            if let Some(row) = doctests.get(r) {
                acceptance.insert(row.clone());
            }
        }
        let module_parts: Vec<&str> = module.split("::").collect();
        let mut parts = vec![crate_name.as_str()];
        parts.extend(module_parts.iter().filter(|p| !p.is_empty()));
        features.push(Feature {
            id: qualified(IFACE, &format!("feature{}", pascal(&parts))),
            statement,
            calls: fns.iter().map(|r| call_of[r].clone()).collect(),
            acceptance: acceptance.into_iter().collect(),
            span: ctx.func(fns[0]).span.clone(),
            suite: package.name.clone(),
            journey: Some((
                qualified(ARCH, "stakeholderLibraryUsers"),
                qualified(IFACE, &format!("journeyLibrary{}", pascal(&[&crate_name]))),
            )),
        });
    }
    // Binaries: subcommands, or the entry point.
    for (pkg, package) in ws.packages.iter().enumerate() {
        for target in package.targets.iter().filter(|t| t.kind == TargetKind::Bin) {
            let bin = &target.name;
            let journey = Some((
                qualified(ARCH, "stakeholderCliUsers"),
                qualified(IFACE, &format!("journeyCli{}", pascal(&[bin]))),
            ));
            let package_tests: Vec<&(FnRef, String)> =
                tests.iter().filter(|(r, _)| r.pkg == pkg).collect();
            let exe = format!("CARGO_BIN_EXE_{bin}");
            let mut subcommands = Vec::new();
            for file in &package.files {
                let in_bin = package.targets[file.target].name == *bin
                    || package.targets[file.target].kind == TargetKind::Lib;
                if !in_bin {
                    continue;
                }
                for e in &file.enums {
                    if e.derives.iter().any(|d| d == "Subcommand") {
                        for v in &e.variants {
                            subcommands.push((v.clone(), e.span.clone()));
                        }
                    }
                }
            }
            if subcommands.is_empty() {
                let main = package.files.iter().find(|f| f.rel == target.src);
                let doc = main.map(|f| summary(&f.doc)).unwrap_or_default();
                let span = main
                    .and_then(|f| f.fns.iter().find(|x| x.name == "main"))
                    .map(|x| x.span.clone())
                    .unwrap_or(Span {
                        file: target.src.clone(),
                        line: 1,
                    });
                let call = format!("callCli{}Main", pascal(&[bin]));
                let text = if doc.is_empty() {
                    format!("{bin}: the binary's entry point.")
                } else {
                    format!("{bin}: {doc}")
                };
                if !push(
                    &mut refusals,
                    Decl::value(call.clone(), "String", string(&text)),
                    &span,
                ) {
                    continue;
                }
                let acceptance: Vec<String> = package_tests
                    .iter()
                    .filter(|(r, _)| reaches_literal(ctx, *r, |l| l.contains(&exe)))
                    .map(|(_, row)| row.clone())
                    .collect();
                features.push(Feature {
                    id: qualified(IFACE, &format!("feature{}Main", pascal(&[bin]))),
                    statement: text,
                    calls: vec![qualified(IFACE, &call)],
                    acceptance,
                    span,
                    suite: package.name.clone(),
                    journey: journey.clone(),
                });
            } else {
                for (variant, span) in subcommands {
                    let command = kebab(&variant.name);
                    let call = format!("callCli{}{}", pascal(&[bin]), pascal(&[&variant.name]));
                    let doc = summary(&variant.doc);
                    let text = if doc.is_empty() {
                        format!("{bin} {command}")
                    } else {
                        format!("{bin} {command}: {doc}")
                    };
                    if !push(
                        &mut refusals,
                        Decl::value(call.clone(), "String", string(&text)),
                        &span,
                    ) {
                        continue;
                    }
                    let acceptance: Vec<String> = package_tests
                        .iter()
                        .filter(|(r, _)| reaches_literal(ctx, *r, |l| *l == command))
                        .map(|(_, row)| row.clone())
                        .collect();
                    // A subcommand named as a library module of the same
                    // crate is told apart from that module's feature.
                    let mut id = qualified(
                        IFACE,
                        &format!("feature{}{}", pascal(&[bin]), pascal(&[&variant.name])),
                    );
                    if features.iter().any(|f| f.id == id) {
                        id = qualified(
                            IFACE,
                            &format!(
                                "feature{}Command{}",
                                pascal(&[bin]),
                                pascal(&[&variant.name])
                            ),
                        );
                    }
                    features.push(Feature {
                        id,
                        statement: text,
                        calls: vec![qualified(IFACE, &call)],
                        acceptance,
                        span,
                        suite: package.name.clone(),
                        journey: journey.clone(),
                    });
                }
            }
        }
    }
    // Admission: a feature without acceptance is refused, not emitted.
    let mut admitted = Vec::new();
    for feature in features {
        if feature.acceptance.is_empty() {
            refusals.push(Refusal::Unaccepted {
                span: feature.span.clone(),
                feature: feature.id.clone(),
            });
            continue;
        }
        admitted.push(feature);
    }
    let t = "Foundation.Arch::";
    let mut journeys: BTreeMap<String, (String, Vec<String>)> = BTreeMap::new();
    for feature in &admitted {
        decls.push(Decl::value(
            local(&feature.id),
            format!("{t}Feature"),
            record(
                &format!("{t}Feature"),
                &[
                    ("statement", string(&feature.statement)),
                    ("calls", string_list(&feature.calls)),
                    ("acceptance", string_list(&feature.acceptance)),
                ],
            ),
        ));
        if let Some((actor, journey)) = &feature.journey {
            journeys
                .entry(journey.clone())
                .or_insert_with(|| (actor.clone(), Vec::new()))
                .1
                .push(feature.id.clone());
        }
    }
    // UR-R09: an example is a journey through the features its main calls.
    let feature_of: BTreeMap<&String, &String> = admitted
        .iter()
        .flat_map(|f| f.calls.iter().map(move |c| (c, &f.id)))
        .collect();
    for r in ctx.all_fns() {
        let f = ctx.func(r);
        if f.name != "main" || ctx.kind(r) != &TargetKind::Example {
            continue;
        }
        let file = ctx.file(r);
        let package = ctx.package(r);
        let mut callees: Vec<FnRef> = ctx.calls.get(&r).into_iter().flatten().copied().collect();
        callees.sort_by_key(|c| (ctx.func(*c).span.clone(), *c));
        let mut steps: Vec<String> = Vec::new();
        for callee in callees {
            if let Some(feature) = call_of.get(&callee).and_then(|c| feature_of.get(c)) {
                if steps.last() != Some(*feature) {
                    steps.push((*feature).clone());
                }
            }
        }
        if steps.is_empty() {
            continue;
        }
        let target = &package.targets[file.target].name;
        journeys.insert(
            qualified(
                IFACE,
                &format!("journeyExample{}", pascal(&[&package.name, target])),
            ),
            (qualified(ARCH, "stakeholderLibraryUsers"), steps),
        );
    }
    for (journey, (actor, steps)) in &journeys {
        decls.push(Decl::value(
            local(journey),
            format!("{t}Journey"),
            record(
                &format!("{t}Journey"),
                &[("actor", string(actor)), ("steps", string_list(steps))],
            ),
        ));
    }
    let mut sorted: Vec<&Feature> = admitted.iter().collect();
    sorted.sort_by(|a, b| a.id.as_bytes().cmp(b.id.as_bytes()));
    let width = sorted.len().to_string().len().max(2);
    let register = sorted
        .iter()
        .enumerate()
        .map(|(i, f)| RegisterRow {
            id: format!("FT-{:0width$}", i + 1),
            suite: f.suite.clone(),
            record: f.id.clone(),
            statement: f.statement.clone(),
            calls: f.calls.clone(),
            acceptance: f.acceptance.clone(),
        })
        .collect();
    let interface = Interface {
        feature_count: admitted.len(),
        module: Module {
            name: IFACE.into(),
            glossaries: vec![BOOL, NAT, PARCH],
            imports: vec!["Foundation.Arch".into()],
            title: "feature",
            decls,
        },
        register,
    };
    (interface, refusals)
}

/// Unit-variant enums of product code named for a state, or with transitions.
/// Ask the router for every enum of product code: `error-enum` when it is named
/// for an error or implements `Error`, `state-enum` when it is fieldless and
/// named for a state or matched in transitions. An enum with both tags is
/// refused as ambiguous and becomes neither record.
#[allow(clippy::type_complexity)]
fn route_enums(ctx: &Model) -> (BTreeMap<(usize, usize, usize), &'static str>, Vec<Refusal>) {
    let mut routes = BTreeMap::new();
    let mut refusals = Vec::new();
    let transitions: BTreeSet<String> = ctx
        .all_fns()
        .into_iter()
        .filter(|r| ctx.product(*r))
        .flat_map(|r| {
            ctx.func(r)
                .transitions
                .iter()
                .map(|t| t.0.clone())
                .collect::<Vec<_>>()
        })
        .collect();
    for (pkg, package) in ctx.ws.packages.iter().enumerate() {
        for (fi, file) in package.files.iter().enumerate() {
            if !matches!(
                package.targets[file.target].kind,
                TargetKind::Lib | TargetKind::Bin
            ) {
                continue;
            }
            for (ei, e) in file.enums.iter().enumerate() {
                let unit = e.variants.len() >= 2 && e.variants.iter().all(|v| v.unit);
                let named = ["State", "Mode", "Phase", "Status", "Stage"]
                    .iter()
                    .any(|s| e.name.ends_with(s));
                let cli = e
                    .derives
                    .iter()
                    .any(|d| matches!(d.as_str(), "Subcommand" | "ValueEnum" | "Parser"));
                let mut tags = Vec::new();
                if !e.variants.is_empty()
                    && (e.name.ends_with("Error") || file.error_impls.contains(&e.name))
                {
                    tags.push("error-enum");
                }
                if unit && !cli && (named || transitions.contains(&e.name)) {
                    tags.push("state-enum");
                }
                match crate::route::classify(&tags) {
                    Some(crate::route::Routing::Routed(route)) => {
                        routes.insert((pkg, fi, ei), route.id);
                    }
                    Some(crate::route::Routing::Ambiguous(a, b)) => {
                        refusals.push(Refusal::AmbiguousFact {
                            span: e.span.clone(),
                            fact: e.name.clone(),
                            tags: vec![a, b],
                        });
                    }
                    Some(crate::route::Routing::Unknown(_)) | None => {}
                }
            }
        }
    }
    (routes, refusals)
}

fn state_enums(ctx: &Model) -> Vec<(usize, usize, usize)> {
    ctx.enum_routes
        .iter()
        .filter(|(_, route)| **route == "UR-R26")
        .map(|(key, _)| *key)
        .collect()
}

fn design(ctx: &Model) -> (Module, Vec<Refusal>) {
    let mut refusals = Vec::new();
    let t = "Foundation.Arch::";
    let mut decls = Vec::new();
    let mut processes: BTreeMap<FnRef, String> = BTreeMap::new();
    let process_id = |ctx: &Model, r: FnRef| {
        let path = ctx.path(r);
        let parts: Vec<&str> = path.split("::").collect();
        qualified(DESIGN, &format!("process{}", pascal(&parts)))
    };
    for r in interface_fns(ctx) {
        processes.insert(r, process_id(ctx, r));
    }
    for r in ctx.all_fns() {
        let f = ctx.func(r);
        if ctx.product(r)
            && (f.name == "main" && f.owner.is_none() && ctx.kind(r) == &TargetKind::Bin
                || !f.transitions.is_empty())
        {
            processes.insert(r, process_id(ctx, r));
        }
    }
    let mut process_names: BTreeSet<String> = BTreeSet::new();
    processes.retain(|r, id| ctx.component_of(*r).is_some() && process_names.insert(id.clone()));
    for (r, id) in &processes {
        let owner = ctx.component_of(*r).cloned().unwrap_or_default();
        decls.push(Decl::value(
            local(id),
            format!("{t}SystemProcess"),
            record(&format!("{t}SystemProcess"), &[("owner", string(&owner))]),
        ));
    }
    // States.
    let mut machines = Vec::new();
    for (pkg, fi, ei) in state_enums(ctx) {
        let package = &ctx.ws.packages[pkg];
        let file = &package.files[fi];
        let e = &file.enums[ei];
        let Some(owner) = ctx.targets.get(&(pkg, file.target)).cloned() else {
            continue;
        };
        let base = pascal(&[&package.targets[file.target].name, &e.name]);
        let states: Vec<(String, String)> = e
            .variants
            .iter()
            .map(|v| {
                (
                    v.name.clone(),
                    qualified(DESIGN, &format!("state{base}{}", pascal(&[&v.name]))),
                )
            })
            .collect();
        for (_, id) in &states {
            decls.push(Decl::value(
                local(id),
                format!("{t}SystemState"),
                record(&format!("{t}SystemState"), &[("owner", string(&owner))]),
            ));
        }
        // Transitions from fns of the same component.
        let mut transitions = Vec::new();
        for (r, process) in &processes {
            if ctx.component_of(*r) != Some(&owner) {
                continue;
            }
            for (enum_name, from, to) in &ctx.func(*r).transitions {
                if enum_name == &e.name {
                    transitions.push((
                        ctx.func(*r).span.clone(),
                        from.clone(),
                        to.clone(),
                        process.clone(),
                    ));
                }
            }
        }
        transitions.sort();
        transitions.dedup();
        let initial = if let Some(v) = e.variants.iter().find(|v| v.default) {
            Some(v.name.clone())
        } else {
            let entered: BTreeSet<&String> = transitions.iter().map(|t| &t.2).collect();
            let sources: Vec<&String> = e
                .variants
                .iter()
                .map(|v| &v.name)
                .filter(|v| !entered.contains(v))
                .collect();
            match sources.as_slice() {
                [only] if !transitions.is_empty() => Some((*only).clone()),
                _ => None,
            }
        };
        if initial.is_none() {
            refusals.push(Refusal::NoInitialState {
                span: e.span.clone(),
                machine: format!("{}::{}", package.targets[file.target].name, e.name),
            });
        }
        machines.push((base, e.name.clone(), owner, states, transitions, initial));
    }
    // UML component view.
    let comp_view = qualified(DESIGN, "umlComponents");
    decls.push(Decl::value(
        local(&comp_view),
        format!("{t}DesignView"),
        record(
            &format!("{t}DesignView"),
            &[
                ("title", string("Components and dependencies")),
                (
                    "viewpoint",
                    string(&qualified(ARCH, "viewpointMaintenance")),
                ),
                ("diagram", format!("(ctor {t}DesignDiagram.umlComponent)")),
            ],
        ),
    ));
    for c in &ctx.components {
        decls.push(Decl::value(
            format!("uml{}", Model::component_suffix(&c.id)),
            format!("{t}UmlComponent"),
            record(
                &format!("{t}UmlComponent"),
                &[("view", string(&comp_view)), ("component", string(&c.id))],
            ),
        ));
    }
    for id in ctx.edges.keys() {
        decls.push(Decl::value(
            format!("umlConnector{}", local(id).trim_start_matches("edge")),
            format!("{t}UmlConnector"),
            record(
                &format!("{t}UmlConnector"),
                &[("view", string(&comp_view)), ("edge", string(id))],
            ),
        ));
    }
    let connected = |a: &str, b: &str| ctx.edges.values().any(|(f, t)| f == a && t == b);
    // Sequence views: each bin main's calls into other components.
    for (r, process) in &processes {
        let f = ctx.func(*r);
        if f.name != "main" || ctx.kind(*r) != &TargetKind::Bin {
            continue;
        }
        let Some(bin) = ctx.component_of(*r).cloned() else {
            continue;
        };
        let mut messages = Vec::new();
        let mut lifelines = vec![bin.clone()];
        let mut order: Vec<FnRef> = ctx.calls.get(r).into_iter().flatten().copied().collect();
        order.sort_by_key(|c| (ctx.func(*c).span.clone(), *c));
        for callee in order {
            let Some(target) = ctx.component_of(callee).cloned() else {
                continue;
            };
            let Some(callee_process) = processes.get(&callee) else {
                continue;
            };
            if target == bin || !connected(&bin, &target) {
                continue;
            }
            if !lifelines.contains(&target) {
                lifelines.push(target.clone());
            }
            messages.push((target, callee_process.clone()));
        }
        if messages.is_empty() {
            continue;
        }
        let base = local(process).trim_start_matches("process").to_string();
        let view = qualified(DESIGN, &format!("sequence{base}"));
        decls.push(Decl::value(
            local(&view),
            format!("{t}DesignView"),
            record(
                &format!("{t}DesignView"),
                &[
                    ("title", string(&format!("{} calls", ctx.path(*r)))),
                    ("viewpoint", string(&qualified(ARCH, "viewpointUsage"))),
                    ("diagram", format!("(ctor {t}DesignDiagram.umlSequence)")),
                ],
            ),
        ));
        for (i, lifeline) in lifelines.iter().enumerate() {
            decls.push(Decl::value(
                format!("lifeline{base}{}", Model::component_suffix(lifeline)),
                format!("{t}UmlLifeline"),
                record(
                    &format!("{t}UmlLifeline"),
                    &[
                        ("view", string(&view)),
                        ("component", string(lifeline)),
                        ("order", i.to_string()),
                    ],
                ),
            ));
        }
        for (i, (target, callee_process)) in messages.iter().enumerate() {
            decls.push(Decl::value(
                format!("message{base}{i}"),
                format!("{t}UmlMessage"),
                record(
                    &format!("{t}UmlMessage"),
                    &[
                        ("view", string(&view)),
                        ("kind", format!("(ctor {t}UmlMessageKind.synchronous)")),
                        ("order", i.to_string()),
                        ("source", string(&bin)),
                        ("target", string(target)),
                        ("process", string(callee_process)),
                    ],
                ),
            ));
        }
    }
    // State machines.
    for (base, name, _owner, states, transitions, initial) in &machines {
        let view = qualified(DESIGN, &format!("machine{base}"));
        decls.push(Decl::value(
            local(&view),
            format!("{t}DesignView"),
            record(
                &format!("{t}DesignView"),
                &[
                    ("title", string(&format!("{name} states"))),
                    (
                        "viewpoint",
                        string(&qualified(ARCH, "viewpointMaintenance")),
                    ),
                    (
                        "diagram",
                        format!("(ctor {t}DesignDiagram.umlStateMachine)"),
                    ),
                ],
            ),
        ));
        for (variant, id) in states {
            decls.push(Decl::value(
                format!("uml{}", local(id)),
                format!("{t}UmlState"),
                record(
                    &format!("{t}UmlState"),
                    &[
                        ("view", string(&view)),
                        ("state", string(id)),
                        ("initial", (initial.as_ref() == Some(variant)).to_string()),
                    ],
                ),
            ));
        }
        let state_of: BTreeMap<&String, &String> = states.iter().map(|(v, id)| (v, id)).collect();
        for (i, (_, from, to, process)) in transitions.iter().enumerate() {
            let (Some(source), Some(target)) = (state_of.get(from), state_of.get(to)) else {
                continue;
            };
            decls.push(Decl::value(
                format!("transition{base}{i}"),
                format!("{t}UmlTransition"),
                record(
                    &format!("{t}UmlTransition"),
                    &[
                        ("view", string(&view)),
                        ("order", i.to_string()),
                        ("source", string(source)),
                        ("target", string(target)),
                        ("trigger", string(process)),
                    ],
                ),
            ));
        }
    }
    // OPM: each component with processes, its processes, invocations among them
    // and the states of its machines.
    let mut by_component: BTreeMap<String, Vec<(FnRef, String)>> = BTreeMap::new();
    for (r, id) in &processes {
        if let Some(c) = ctx.component_of(*r) {
            by_component
                .entry(c.clone())
                .or_default()
                .push((*r, id.clone()));
        }
    }
    for (component, procs) in &by_component {
        let suffix = Model::component_suffix(component).to_string();
        let view = qualified(DESIGN, &format!("opm{suffix}"));
        decls.push(Decl::value(
            local(&view),
            format!("{t}DesignView"),
            record(
                &format!("{t}DesignView"),
                &[
                    (
                        "title",
                        string(&format!(
                            "{} processes",
                            ctx.components[ctx.index[component]].name
                        )),
                    ),
                    (
                        "viewpoint",
                        string(&qualified(ARCH, "viewpointMaintenance")),
                    ),
                    (
                        "diagram",
                        format!("(ctor {t}DesignDiagram.opmObjectProcess)"),
                    ),
                ],
            ),
        ));
        decls.push(Decl::value(
            format!("opm{suffix}Object"),
            format!("{t}OpmThing"),
            record(
                &format!("{t}OpmThing"),
                &[
                    ("view", string(&view)),
                    ("kind", format!("(ctor {t}OpmThingKind.object)")),
                    ("subject", string(component)),
                ],
            ),
        ));
        let in_view: BTreeSet<FnRef> = procs.iter().map(|(r, _)| *r).collect();
        for (r, id) in procs {
            let p = local(id).trim_start_matches("process");
            decls.push(Decl::value(
                format!("opm{suffix}Thing{p}"),
                format!("{t}OpmThing"),
                record(
                    &format!("{t}OpmThing"),
                    &[
                        ("view", string(&view)),
                        ("kind", format!("(ctor {t}OpmThingKind.process)")),
                        ("subject", string(id)),
                    ],
                ),
            ));
            decls.push(Decl::value(
                format!("opm{suffix}Instrument{p}"),
                format!("{t}OpmLink"),
                record(
                    &format!("{t}OpmLink"),
                    &[
                        ("view", string(&view)),
                        ("kind", format!("(ctor {t}OpmLinkKind.instrument)")),
                        ("source", string(component)),
                        ("target", string(id)),
                    ],
                ),
            ));
            for callee in ctx.calls.get(r).into_iter().flatten() {
                if !in_view.contains(callee) || callee == r {
                    continue;
                }
                let q = local(&processes[callee])
                    .trim_start_matches("process")
                    .to_string();
                decls.push(Decl::value(
                    format!("opm{suffix}Invokes{p}{q}"),
                    format!("{t}OpmLink"),
                    record(
                        &format!("{t}OpmLink"),
                        &[
                            ("view", string(&view)),
                            ("kind", format!("(ctor {t}OpmLinkKind.invocation)")),
                            ("source", string(id)),
                            ("target", string(&processes[callee])),
                        ],
                    ),
                ));
            }
        }
        for (_, _, owner, states, _, _) in &machines {
            if owner != component {
                continue;
            }
            for (_, state) in states {
                decls.push(Decl::value(
                    format!("opm{suffix}{}", pascal(&[local(state)])),
                    format!("{t}OpmState"),
                    record(
                        &format!("{t}OpmState"),
                        &[
                            ("view", string(&view)),
                            ("thing", string(component)),
                            ("state", string(state)),
                        ],
                    ),
                ));
            }
        }
    }
    decls.extend(module_opm(ctx, &processes));
    let mut names = BTreeSet::new();
    decls.retain(|d| names.insert(d.name().to_string()));
    let module = Module {
        name: DESIGN.into(),
        glossaries: vec![BOOL, NAT, PARCH],
        imports: vec!["Foundation.Arch".into()],
        title: "design view",
        decls,
    };
    (module, refusals)
}

/// OPM per module (UR-R28): the module's component is the object, its
/// processes are the module's functions, each invocation inside the module is
/// a link and each process zooms into the processes it calls there.
fn module_opm(ctx: &Model, processes: &BTreeMap<FnRef, String>) -> Vec<Decl> {
    let t = "Foundation.Arch::";
    let mut by_module: BTreeMap<(String, String, String), Vec<FnRef>> = BTreeMap::new();
    for r in processes.keys() {
        let Some(component) = ctx.component_of(*r) else {
            continue;
        };
        let file = ctx.file(*r);
        let target = &ctx.package(*r).targets[file.target].name;
        let path = if file.module.is_empty() {
            target.replace('-', "_")
        } else {
            format!("{}::{}", target.replace('-', "_"), file.module)
        };
        by_module
            .entry((component.clone(), file.module.clone(), path))
            .or_default()
            .push(*r);
    }
    let mut decls = Vec::new();
    for ((component, module, path), procs) in &by_module {
        let parts: Vec<&str> = module.split("::").filter(|p| !p.is_empty()).collect();
        let suffix = format!("{}{}", Model::component_suffix(component), pascal(&parts));
        let view = qualified(DESIGN, &format!("opmModule{suffix}"));
        let name = local(&view).to_string();
        decls.push(Decl::value(
            name.clone(),
            format!("{t}DesignView"),
            record(
                &format!("{t}DesignView"),
                &[
                    ("title", string(&format!("{path} processes"))),
                    (
                        "viewpoint",
                        string(&qualified(ARCH, "viewpointMaintenance")),
                    ),
                    (
                        "diagram",
                        format!("(ctor {t}DesignDiagram.opmObjectProcess)"),
                    ),
                ],
            ),
        ));
        decls.push(Decl::value(
            format!("{name}Object"),
            format!("{t}OpmThing"),
            record(
                &format!("{t}OpmThing"),
                &[
                    ("view", string(&view)),
                    ("kind", format!("(ctor {t}OpmThingKind.object)")),
                    ("subject", string(component)),
                ],
            ),
        ));
        let in_view: BTreeSet<FnRef> = procs.iter().copied().collect();
        let short = |r: &FnRef| {
            local(&processes[r])
                .trim_start_matches("process")
                .to_string()
        };
        let mut parent: BTreeMap<FnRef, FnRef> = BTreeMap::new();
        for r in procs {
            let id = &processes[r];
            let p = short(r);
            decls.push(Decl::value(
                format!("{name}Thing{p}"),
                format!("{t}OpmThing"),
                record(
                    &format!("{t}OpmThing"),
                    &[
                        ("view", string(&view)),
                        ("kind", format!("(ctor {t}OpmThingKind.process)")),
                        ("subject", string(id)),
                    ],
                ),
            ));
            decls.push(Decl::value(
                format!("{name}Instrument{p}"),
                format!("{t}OpmLink"),
                record(
                    &format!("{t}OpmLink"),
                    &[
                        ("view", string(&view)),
                        ("kind", format!("(ctor {t}OpmLinkKind.instrument)")),
                        ("source", string(component)),
                        ("target", string(id)),
                    ],
                ),
            ));
            let mut order = 0usize;
            for callee in ctx.calls.get(r).into_iter().flatten() {
                if !in_view.contains(callee) || callee == r {
                    continue;
                }
                let q = short(callee);
                decls.push(Decl::value(
                    format!("{name}Invokes{p}{q}"),
                    format!("{t}OpmLink"),
                    record(
                        &format!("{t}OpmLink"),
                        &[
                            ("view", string(&view)),
                            ("kind", format!("(ctor {t}OpmLinkKind.invocation)")),
                            ("source", string(id)),
                            ("target", string(&processes[callee])),
                        ],
                    ),
                ));
                // A process zooms under one parent only, its first caller in
                // declaration order, and never under its own descendant.
                let mut ancestor = Some(*r);
                let mut cycle = false;
                while let Some(a) = ancestor {
                    cycle |= a == *callee;
                    ancestor = parent.get(&a).copied().filter(|_| !cycle);
                }
                if cycle || parent.contains_key(callee) {
                    continue;
                }
                parent.insert(*callee, *r);
                decls.push(Decl::value(
                    format!("{name}Zoom{p}{q}"),
                    format!("{t}OpmZoom"),
                    record(
                        &format!("{t}OpmZoom"),
                        &[
                            ("view", string(&view)),
                            ("process", string(id)),
                            ("subprocess", string(&processes[callee])),
                            ("order", order.to_string()),
                        ],
                    ),
                ));
                order += 1;
            }
        }
    }
    decls
}

struct Threat {
    asset: String,
    asset_name: String,
    asset_text: String,
    component: String,
    threat: String,
    threat_name: String,
    threat_text: String,
    stride: &'static str,
    likelihood: &'static str,
    impact: &'static str,
    control_name: String,
    control_text: String,
    evidence: Vec<(&'static str, String)>,
    /// Whether the control's verification passes today.
    passing: bool,
    /// The rationale of an accepted risk, which has no control.
    accepted: Option<String>,
}

fn capability_threat(
    cap: Capability,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
) {
    // (asset noun, threat sentence, STRIDE, likelihood, impact)
    match cap {
        Capability::Fs => (
            "Files it reads and writes",
            "A path or file content an attacker controls alters what is read or overwritten.",
            "tampering",
            "medium",
            "major",
        ),
        Capability::Net => (
            "Network connections it opens",
            "Data sent or received over the network is observed or redirected.",
            "informationDisclosure",
            "medium",
            "major",
        ),
        Capability::Process => (
            "Processes it starts",
            "A started process runs with arguments or a program an attacker chose.",
            "elevationOfPrivilege",
            "medium",
            "critical",
        ),
        Capability::Env => (
            "Environment and arguments it reads",
            "A variable or argument an attacker sets impersonates the configuration the operator meant.",
            "spoofing",
            "low",
            "major",
        ),
        Capability::Unsafe => (
            "Memory it manages with unsafe code",
            "An unsafe block breaks an invariant and corrupts memory.",
            "elevationOfPrivilege",
            "low",
            "critical",
        ),
        Capability::Ffi => (
            "Foreign functions it calls",
            "A foreign function violates the contract its declaration states.",
            "elevationOfPrivilege",
            "low",
            "critical",
        ),
    }
}

fn threats(ctx: &Model) -> Vec<Threat> {
    let ws = &ctx.ws;
    let mut out = Vec::new();
    for (pkg, package) in ws.packages.iter().enumerate() {
        for (t, target) in package.targets.iter().enumerate() {
            let Some(component) = ctx.targets.get(&(pkg, t)) else {
                continue;
            };
            let suffix = Model::component_suffix(component).to_string();
            let files: Vec<&SourceFile> = package.files.iter().filter(|f| f.target == t).collect();
            let mut sites: BTreeMap<Capability, Vec<String>> = BTreeMap::new();
            for file in &files {
                for (cap, span, what) in &file.capabilities {
                    sites
                        .entry(*cap)
                        .or_default()
                        .push(format!("{} {what}", span.render()));
                }
            }
            // Every target answers for memory safety, unsafe code or not.
            let forbids = files
                .iter()
                .any(|f| f.rel == target.src && f.forbids_unsafe)
                || package
                    .lints
                    .get("rust.unsafe_code")
                    .is_some_and(|l| l == "forbid");
            if !sites.contains_key(&Capability::Unsafe) {
                let evidence = if forbids {
                    let where_ = if files
                        .iter()
                        .any(|f| f.rel == target.src && f.forbids_unsafe)
                    {
                        format!("#![forbid(unsafe_code)] in {}", target.src)
                    } else {
                        format!(
                            "[lints.rust] unsafe_code = forbid in {}/Cargo.toml",
                            package.dir
                        )
                    };
                    vec![("gate", where_)]
                } else {
                    vec![(
                        "review",
                        format!(
                            "uorust found no unsafe block, unsafe fn or unsafe impl in {}",
                            target.name
                        ),
                    )]
                };
                out.push(Threat {
                    asset: qualified(SEC, &format!("asset{suffix}Memory")),
                    asset_name: format!("{} memory", target.name),
                    asset_text: format!("The memory of {} while it runs.", target.name),
                    component: component.clone(),
                    threat: qualified(SEC, &format!("threat{suffix}Memory")),
                    threat_name: format!("Memory corruption in {}", target.name),
                    threat_text: "Unsafe code breaks an invariant the borrow checker would have kept."
                        .to_string(),
                    stride: "elevationOfPrivilege",
                    likelihood: "low",
                    impact: "critical",
                    control_name: if forbids {
                        "Unsafe code forbidden".to_string()
                    } else {
                        "Safe Rust only".to_string()
                    },
                    control_text: format!(
                        "{} is checked by the borrow checker throughout; no unsafe code is admitted.",
                        target.name
                    ),
                    evidence,
                    passing: true,
                    accepted: None,
                });
            }
            for (cap, list) in sites {
                let (noun, sentence, stride, likelihood, impact) = capability_threat(cap);
                let key = cap.key();
                out.push(Threat {
                    asset: qualified(SEC, &format!("asset{suffix}{key}")),
                    asset_name: format!("{}: {}", target.name, noun.to_lowercase()),
                    asset_text: format!("{noun} in {}, at {} site(s).", target.name, list.len()),
                    component: component.clone(),
                    threat: qualified(SEC, &format!("threat{suffix}{key}")),
                    threat_name: format!("{} misuse in {}", key, target.name),
                    threat_text: sentence.to_string(),
                    stride,
                    likelihood,
                    impact,
                    control_name: format!("Reviewed {} sites", key.to_lowercase()),
                    control_text: format!(
                        "Each {} site of {} is listed with its span for review.",
                        key.to_lowercase(),
                        target.name
                    ),
                    evidence: list.into_iter().map(|s| ("review", s)).collect(),
                    passing: true,
                    accepted: None,
                });
            }
        }
    }
    if let Some(first) = ctx.components.iter().find(|c| c.kind != "externalSystem") {
        let mut evidence = vec![(
            "gate",
            format!(
                "Cargo.lock pins {} packages, {} with a registry checksum",
                ws.lock.len(),
                ws.lock.iter().filter(|l| !l.checksum.is_empty()).count()
            ),
        )];
        if ws.has_deny {
            evidence.push(("gate", "cargo deny check: deny.toml".to_string()));
        }
        out.push(Threat {
            asset: qualified(SEC, "assetLockedDependencies"),
            asset_name: "Locked dependency graph".to_string(),
            asset_text: format!("The {} packages Cargo.lock pins.", ws.lock.len()),
            component: first.id.clone(),
            threat: qualified(SEC, "threatDependencySubstitution"),
            threat_name: "Dependency substitution".to_string(),
            threat_text: "A dependency is replaced by a package with the same name and different code."
                .to_string(),
            stride: "tampering",
            likelihood: "low",
            impact: "critical",
            control_name: "Locked and checked dependencies".to_string(),
            control_text: "Every dependency is pinned with its checksum and checked against advisories and licences."
                .to_string(),
            evidence,
            passing: true,
            accepted: None,
        });
        out.extend(advisory_threats(ctx, &first.id));
    }
    out
}

/// UR-R21: each advisory that affects a locked package is a risk on the
/// supply chain, answered by the advisory gate or accepted where `deny.toml`
/// ignores it.
fn advisory_threats(ctx: &Model, first: &str) -> Vec<Threat> {
    let ws = &ctx.ws;
    let mut out = Vec::new();
    let Some(db) = &ws.advisories else {
        return out;
    };
    for a in &db.hits {
        let key: String = format!("{} v{}", a.id, a.version.replace('.', "p"))
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        let suffix = pascal(&[&key]);
        let component = ctx
            .components
            .iter()
            .find(|c| c.kind == "externalSystem" && c.name == a.package)
            .map_or_else(|| first.to_string(), |c| c.id.clone());
        let has = |c: &str| a.categories.iter().any(|x| x == c);
        let stride =
            if has("code-execution") || has("memory-corruption") || has("privilege-escalation") {
                "elevationOfPrivilege"
            } else if has("denial-of-service") {
                "denialOfService"
            } else if has("memory-exposure") || has("crypto-failure") {
                "informationDisclosure"
            } else {
                "tampering"
            };
        let (likelihood, impact) = match (&a.informational, stride) {
            (Some(_), _) => ("low", "minor"),
            (None, "elevationOfPrivilege") => ("medium", "critical"),
            (None, _) => ("medium", "major"),
        };
        let patched = if a.patched.is_empty() {
            "no patched release".to_string()
        } else {
            format!("patched in {}", a.patched.join(", "))
        };
        let accepted = ws.deny_ignores.get(&a.id).map(|reason| {
            if reason.is_empty() {
                format!("deny.toml ignores {} with no reason given", a.id)
            } else {
                format!("deny.toml ignores {}: {reason}", a.id)
            }
        });
        if accepted.is_none() && !ws.has_deny {
            continue;
        }
        let title = if a.title.is_empty() {
            a.id.clone()
        } else {
            a.title.clone()
        };
        out.push(Threat {
            asset: qualified(SEC, &format!("assetAdvisory{suffix}")),
            asset_name: format!("{} {}", a.package, a.version),
            asset_text: format!("The locked package {} {}.", a.package, a.version),
            component,
            threat: qualified(SEC, &format!("threatAdvisory{suffix}")),
            threat_name: format!("{}: {title}", a.id),
            threat_text: format!(
                "{} ({}) affects {} {}; {patched}.",
                a.id,
                if a.date.is_empty() {
                    "undated"
                } else {
                    &a.date
                },
                a.package,
                a.version
            ),
            stride,
            likelihood,
            impact,
            control_name: format!("Advisory gate for {}", a.id),
            control_text: format!(
                "cargo deny check advisories fails while {} affects {} {}.",
                a.id, a.package, a.version
            ),
            evidence: vec![(
                "gate",
                format!(
                    "cargo deny check advisories: deny.toml; advisory database {}",
                    db.revision
                ),
            )],
            passing: false,
            accepted,
        });
    }
    out
}

fn security(ctx: &Model, options: &Options) -> Module {
    let s = "Foundation.Sec::";
    let tm = "Production.ThreatModel::";
    let rows = threats(ctx);
    let mut decls = Vec::new();
    let assets = indexed(rows.iter().map(|r| (r.asset.clone(), r)).collect());
    for (id, i, r) in &assets {
        decls.push(Decl::value(
            local(id),
            format!("{s}Asset"),
            record(
                &format!("{s}Asset"),
                &[
                    ("index", i.to_string()),
                    ("componentIndex", ctx.index[&r.component].to_string()),
                ],
            ),
        ));
    }
    let asset_index: BTreeMap<&String, usize> = assets.iter().map(|(id, i, _)| (id, *i)).collect();
    let threat_rows = indexed(rows.iter().map(|r| (r.threat.clone(), r)).collect());
    for (id, i, _) in &threat_rows {
        decls.push(Decl::value(
            local(id),
            format!("{s}Threat"),
            record(&format!("{s}Threat"), &[("index", i.to_string())]),
        ));
    }
    let threat_index: BTreeMap<&String, usize> =
        threat_rows.iter().map(|(id, i, _)| (id, *i)).collect();
    let risk_id = |r: &Threat| r.threat.replace("::threat", "::risk");
    let control_id = |r: &Threat| r.threat.replace("::threat", "::control");
    let risks = indexed(rows.iter().map(|r| (risk_id(r), r)).collect());
    for (id, i, r) in &risks {
        decls.push(Decl::value(
            local(id),
            format!("{s}Risk"),
            record(
                &format!("{s}Risk"),
                &[
                    ("index", i.to_string()),
                    ("assetIndex", asset_index[&r.asset].to_string()),
                    ("threatIndex", threat_index[&r.threat].to_string()),
                    (
                        "likelihood",
                        format!("(ctor {s}Likelihood.{})", r.likelihood),
                    ),
                    ("impact", format!("(ctor {s}Impact.{})", r.impact)),
                ],
            ),
        ));
    }
    let risk_index: BTreeMap<String, usize> =
        risks.iter().map(|(id, i, _)| (id.clone(), *i)).collect();
    let controls = indexed(
        rows.iter()
            .filter(|r| r.accepted.is_none())
            .map(|r| (control_id(r), r))
            .collect(),
    );
    for (id, i, r) in &controls {
        decls.push(Decl::value(
            local(id),
            format!("{s}ApplicationSecurityControl"),
            record(
                &format!("{s}ApplicationSecurityControl"),
                &[
                    ("index", i.to_string()),
                    ("riskIndex", risk_index[&risk_id(r)].to_string()),
                ],
            ),
        ));
    }
    for (id, i, _) in &controls {
        decls.push(Decl::value(
            local(&id.replace("::control", "::activity")),
            format!("{s}SecurityActivity"),
            record(
                &format!("{s}SecurityActivity"),
                &[("index", i.to_string()), ("controlIndex", i.to_string())],
            ),
        ));
    }
    for (id, i, r) in &controls {
        let gated = r.passing && r.evidence.iter().any(|(k, _)| *k == "gate");
        decls.push(Decl::value(
            local(&id.replace("::control", "::measurement")),
            format!("{s}VerificationMeasurement"),
            record(
                &format!("{s}VerificationMeasurement"),
                &[
                    ("index", i.to_string()),
                    ("controlIndex", i.to_string()),
                    ("passed", gated.to_string()),
                ],
            ),
        ));
    }
    // Production.ThreatModel rows.
    let version = ctx
        .ws
        .packages
        .iter()
        .map(|p| p.version.clone())
        .next()
        .unwrap_or_else(|| "0.0.0".to_string());
    decls.push(Decl::value(
        "controlSet",
        format!("{tm}ControlSet"),
        record(
            &format!("{tm}ControlSet"),
            &[
                ("title", string(&format!("{} security controls", options.name))),
                (
                    "text",
                    string(&format!(
                        "Controls uorust derived from the {} workspace: memory safety, capability sites and the locked dependency graph.",
                        options.name
                    )),
                ),
                ("version", string(&version)),
                ("lastModified", string(&options.last_modified)),
            ],
        ),
    ));
    for (id, _, r) in &assets {
        decls.push(Decl::value(
            format!("modeled{}", pascal(&[local(id)])),
            format!("{tm}ModeledAsset"),
            record(
                &format!("{tm}ModeledAsset"),
                &[
                    ("asset", string(id)),
                    ("name", string(&r.asset_name)),
                    ("text", string(&r.asset_text)),
                ],
            ),
        ));
    }
    for (id, _, r) in &threat_rows {
        decls.push(Decl::value(
            format!("modeled{}", pascal(&[local(id)])),
            format!("{tm}ModeledThreat"),
            record(
                &format!("{tm}ModeledThreat"),
                &[
                    ("threat", string(id)),
                    ("name", string(&r.threat_name)),
                    ("text", string(&r.threat_text)),
                    ("category", format!("(ctor {tm}Stride.{})", r.stride)),
                ],
            ),
        ));
    }
    for (id, _, r) in &controls {
        // A long list is balanced appends of short literals, so the
        // linker's JSON reader meets a shallow term however many sites a
        // control lists.
        let items: Vec<String> = r
            .evidence
            .iter()
            .map(|(kind, reference)| {
                record(
                    &format!("{tm}Evidence"),
                    &[
                        ("kind", format!("(ctor {tm}EvidenceKind.{kind})")),
                        ("reference", string(&truncate(reference, 256))),
                    ],
                )
            })
            .collect();
        let evidence = crate::embed::term_list(&items, &format!("{tm}Evidence"));
        decls.push(Decl::value(
            format!("modeled{}", pascal(&[local(id)])),
            format!("{tm}ModeledControl"),
            record(
                &format!("{tm}ModeledControl"),
                &[
                    ("control", string(id)),
                    ("name", string(&r.control_name)),
                    ("text", string(&r.control_text)),
                    ("evidence", evidence),
                ],
            ),
        ));
    }
    for (id, _, r) in &threat_rows {
        if let Some(rationale) = &r.accepted {
            decls.push(Decl::value(
                format!("accepted{}", pascal(&[local(id)])),
                format!("{tm}AcceptedRisk"),
                record(
                    &format!("{tm}AcceptedRisk"),
                    &[
                        ("threat", string(id)),
                        ("rationale", string(&truncate(rationale, 256))),
                    ],
                ),
            ));
            continue;
        }
        decls.push(Decl::value(
            format!("mapping{}", pascal(&[local(id)])),
            format!("{tm}ThreatMapping"),
            record(
                &format!("{tm}ThreatMapping"),
                &[
                    ("threat", string(id)),
                    ("controls", string_list(&[control_id(r)])),
                ],
            ),
        ));
    }
    Module {
        name: SEC.into(),
        glossaries: vec![BOOL, NAT, PARCH, PSEC],
        imports: vec!["Foundation.Sec".into(), "Production.ThreatModel".into()],
        title: "security threat",
        decls,
    }
}

fn truncate(text: &str, limit: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= limit {
        collapsed
    } else {
        // The ellipsis takes three of the limit; a limit below three keeps
        // no character of the text, as `saturating_sub` says.
        collapsed
            .chars()
            .take(limit.saturating_sub(3))
            .collect::<String>()
            + "..."
    }
}

/// The Quality facet. Beside what it counts in the workspace, it states the
/// lift and the admission as measures of the model: the functions of product
/// code and those of them that lifted, from `lift` (the register this run
/// writes), and the open refusals, `admitted`'s and its own. Each is a
/// measured count under its own requirement, as every other measure here is;
/// none is a threshold the tool chose.
fn quality(
    ctx: &Model,
    features: usize,
    lift: &Register,
    admitted: &BTreeSet<Refusal>,
) -> (Module, Vec<Refusal>) {
    let mut refusals = Vec::new();
    let ws = &ctx.ws;
    let q = "Foundation.Qual::";
    let fns = ctx.all_fns();
    let public: Vec<FnRef> = fns
        .iter()
        .copied()
        .filter(|r| ctx.product(*r) && ctx.func(*r).public)
        .collect();
    let documented = public
        .iter()
        .filter(|r| !ctx.func(**r).doc.is_empty())
        .count();
    let tests = fns.iter().filter(|r| ctx.func(**r).test).count();
    let doctests: usize = fns.iter().map(|r| ctx.func(*r).doctests).sum();
    let failures: usize = ws
        .packages
        .iter()
        .flat_map(|p| p.files.iter())
        .flat_map(|f| {
            f.enums
                .iter()
                .filter(|e| e.name.ends_with("Error") || f.error_impls.contains(&e.name))
                .map(|e| e.variants.len())
                .collect::<Vec<_>>()
        })
        .sum();
    let mut lints: BTreeSet<String> = BTreeSet::new();
    for p in &ws.packages {
        for (lint, level) in &p.lints {
            if level == "deny" || level == "forbid" {
                lints.insert(lint.clone());
            }
        }
    }
    let unsafe_sites: usize = ws
        .packages
        .iter()
        .flat_map(|p| p.files.iter())
        .map(|f| {
            f.capabilities
                .iter()
                .filter(|c| matches!(c.0, Capability::Unsafe | Capability::Ffi))
                .count()
        })
        .sum();
    let benches = ws
        .packages
        .iter()
        .flat_map(|p| p.targets.iter())
        .filter(|t| t.kind == TargetKind::Bench)
        .count();
    let mut msrv: Option<usize> = None;
    for p in &ws.packages {
        match p
            .rust_version
            .as_deref()
            .and_then(|v| v.split('.').nth(1))
            .and_then(|m| m.parse().ok())
        {
            Some(minor) => msrv = Some(msrv.map_or(minor, |m: usize| m.max(minor))),
            None => {
                refusals.push(Refusal::NoMsrv {
                    package: p.dir.clone(),
                });
            }
        }
    }
    let crates = ws.packages.len();
    // The refusals the frontier will count: the other facets' and the ones
    // found above, each once.
    let open_refusals = admitted
        .iter()
        .chain(&refusals)
        .collect::<BTreeSet<_>>()
        .len();
    let lift = lift.totals();
    // (characteristic, subcharacteristic, measure, threshold)
    let mut measures: Vec<(&str, &str, &str, usize)> = vec![
        (
            "FunctionalSuitability",
            "FunctionalCompleteness",
            "AdmittedFeatures",
            features,
        ),
        (
            "FunctionalSuitability",
            "FunctionalCompleteness",
            "OpenRefusals",
            open_refusals,
        ),
        (
            "FunctionalSuitability",
            "FunctionalCompleteness",
            "ProductFunctions",
            lift.functions,
        ),
        (
            "FunctionalSuitability",
            "FunctionalCompleteness",
            "LiftedFunctions",
            lift.lifted,
        ),
        (
            "Maintainability",
            "Analysability",
            "DocumentedPublicItems",
            documented,
        ),
        ("Maintainability", "Modularity", "WorkspacePackages", crates),
        ("Maintainability", "Testability", "Tests", tests + doctests),
        (
            "Reliability",
            "Faultlessness",
            "TypedFailureVariants",
            failures,
        ),
        ("Reliability", "Faultlessness", "DeniedLints", lints.len()),
        (
            "Security",
            "Integrity",
            "UnsafeAndForeignSites",
            unsafe_sites,
        ),
    ];
    // UR-R23: each denied or forbidden lint is a requirement, measured by the
    // count of lints the workspace denies.
    let lint_names: Vec<String> = lints.iter().map(|l| pascal(&[l])).collect();
    if let Some(minor) = msrv {
        measures.push(("Flexibility", "Adaptability", "MinimumRustMinor", minor));
    }
    if benches > 0 {
        measures.push((
            "PerformanceEfficiency",
            "TimeBehaviour",
            "Benchmarks",
            benches,
        ));
    }
    let chars = indexed(
        measures
            .iter()
            .map(|m| (qualified(QUAL, &format!("characteristic{}", m.0)), ()))
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .collect(),
    );
    let char_index: BTreeMap<String, usize> =
        chars.iter().map(|(id, i, _)| (id.clone(), *i)).collect();
    let subs = indexed(
        measures
            .iter()
            .map(|m| {
                (
                    qualified(QUAL, &format!("subcharacteristic{}", m.1)),
                    qualified(QUAL, &format!("characteristic{}", m.0)),
                )
            })
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .collect(),
    );
    let sub_index: BTreeMap<String, usize> =
        subs.iter().map(|(id, i, _)| (id.clone(), *i)).collect();
    let reqs = indexed(
        measures
            .iter()
            .map(|m| {
                (
                    qualified(QUAL, &format!("requirement{}", m.2)),
                    qualified(QUAL, &format!("subcharacteristic{}", m.1)),
                )
            })
            .chain(lint_names.iter().map(|l| {
                (
                    qualified(QUAL, &format!("requirementLint{l}")),
                    qualified(QUAL, "subcharacteristicFaultlessness"),
                )
            }))
            .collect(),
    );
    let req_index: BTreeMap<String, usize> =
        reqs.iter().map(|(id, i, _)| (id.clone(), *i)).collect();
    let meas = indexed(
        measures
            .iter()
            .map(|m| {
                (
                    qualified(QUAL, &format!("measure{}", m.2)),
                    (qualified(QUAL, &format!("requirement{}", m.2)), m.3),
                )
            })
            .collect(),
    );
    let mut decls = Vec::new();
    for (id, i, _) in &chars {
        decls.push(Decl::value(
            local(id),
            format!("{q}ProductQualityCharacteristic"),
            record(
                &format!("{q}ProductQualityCharacteristic"),
                &[("index", i.to_string())],
            ),
        ));
    }
    for (id, i, c) in &subs {
        decls.push(Decl::value(
            local(id),
            format!("{q}QualitySubcharacteristic"),
            record(
                &format!("{q}QualitySubcharacteristic"),
                &[
                    ("index", i.to_string()),
                    ("characteristicIndex", char_index[c].to_string()),
                ],
            ),
        ));
    }
    for (id, i, s) in &reqs {
        decls.push(Decl::value(
            local(id),
            format!("{q}QualityRequirement"),
            record(
                &format!("{q}QualityRequirement"),
                &[
                    ("index", i.to_string()),
                    ("subcharacteristicIndex", sub_index[s].to_string()),
                ],
            ),
        ));
    }
    for (id, i, (r, threshold)) in &meas {
        decls.push(Decl::value(
            local(id),
            format!("{q}QualityMeasure"),
            record(
                &format!("{q}QualityMeasure"),
                &[
                    ("index", i.to_string()),
                    ("requirementIndex", req_index[r].to_string()),
                    ("threshold", threshold.to_string()),
                ],
            ),
        ));
    }
    let module = Module {
        name: QUAL.into(),
        glossaries: vec![BOOL, NAT, PQUAL],
        imports: vec!["Foundation.Qual".into()],
        title: "quality measure",
        decls,
    };
    (module, refusals)
}

fn behavior(
    ctx: &Model,
    syntax: &Syntax,
    notes: &mut Notes,
    mir: Option<(&crate::mir::Dumps, &crate::mir::MirOptions)>,
) -> (Module, Option<String>) {
    let ws = &ctx.ws;
    let mut decls = Vec::new();
    // Each crate's type aliases, which its types and signatures are read
    // through. An alias spelled like a type the crate declares names one
    // thing in its module and another elsewhere: it is not read.
    let type_aliases: Vec<BTreeMap<String, crate::ast::Alias>> = ws
        .packages
        .iter()
        .enumerate()
        .map(|(pkg, package)| {
            let declared: BTreeSet<&str> = package
                .files
                .iter()
                .flat_map(|file| {
                    file.enums
                        .iter()
                        .map(|e| e.name.as_str())
                        .chain(file.structs.iter().map(|s| s.name.as_str()))
                })
                .collect();
            package
                .files
                .iter()
                .enumerate()
                .flat_map(|(fi, file)| {
                    let bound = ctx.registry_paths.get(&(pkg, fi, String::new()));
                    syntax.files[&file.rel]
                        .aliases
                        .iter()
                        .filter(|(n, _)| !declared.contains(n.as_str()))
                        .map(move |(n, a)| {
                            let to = a.to.localized(&file.imports);
                            let to = bound.map_or_else(|| to.clone(), |b| to.renamed(b));
                            let alias = crate::ast::Alias {
                                params: a.params.clone(),
                                to,
                            };
                            (n.clone(), alias)
                        })
                })
                .chain(ctx.crate_aliases.clone())
                .collect()
        })
        .collect();
    // A chain of aliases that does not repeat one is at most as long as
    // the crate has aliases.
    let expanded = |pkg: usize, ty: &crate::ast::RustType| {
        ty.expanded(&type_aliases[pkg], type_aliases[pkg].len())
    };
    // A type as the file that writes it names it, a field's or a
    // signature's: a bare name its `use` binds to a registry crate's item
    // is that item's path, so the chain through a field ends at the
    // crate's type, by its crate, and an alias the crate declares reads as
    // what it stands for.
    let declared_in = |pkg: usize, fi: usize, inline: &str, ty: &crate::ast::RustType| {
        let file = &ws.packages[pkg].files[fi];
        let ty = ty.localized(&file.imports);
        match ctx.registry_paths.get(&(pkg, fi, inline.to_string())) {
            Some(bound) => expanded(pkg, &ty.renamed(bound)),
            None => expanded(pkg, &ty),
        }
    };
    // A type of product code is keyed by its full path in its crate: its
    // target, its module (an inline module among them) and its name. A key
    // declared twice (once per configuration) is refused; one name in two
    // modules or two crates is two types, each lifting under the shortest
    // name that tells it apart.
    type Key = (usize, String, String);
    /// A struct's fields as its declaration writes them.
    type Written = Vec<(String, crate::ast::RustType)>;
    /// The lifted kinds of one crate's types, by key, each struct and data
    /// enum with the file and the inline module that declare it, which its
    /// fields are read in.
    #[derive(Default)]
    struct Declared {
        enums: BTreeMap<Key, Vec<String>>,
        structs: BTreeMap<Key, (usize, String, Written)>,
        datas: BTreeMap<Key, (usize, String, lift::RawVariants)>,
        spans: BTreeMap<Key, Span>,
        /// The names of the keys declared twice, which name no one type.
        twice: BTreeSet<String>,
    }
    let mut declared: Vec<Declared> = Vec::new();
    for (pkg, package) in ws.packages.iter().enumerate() {
        let mut d = Declared::default();
        let mut clashing: BTreeSet<Key> = BTreeSet::new();
        let product = |file: &crate::facts::SourceFile| {
            matches!(
                package.targets[file.target].kind,
                TargetKind::Lib | TargetKind::Bin
            )
        };
        for (fi, file) in package.files.iter().enumerate() {
            if !product(file) {
                continue;
            }
            for (e, payload) in file.enums.iter().zip(&syntax.files[&file.rel].enums) {
                // An enum whose variants carry data is an inductive with
                // fields, where its syntax reads.
                let data = !e.variants.iter().all(|v| v.unit);
                if e.variants.is_empty() || (data && payload.is_empty()) {
                    continue;
                }
                let inline = syntax.names.type_module(&file.rel, e.span.line, &e.name);
                let key = (
                    file.target,
                    crate::names::join(&file.module, inline),
                    e.name.clone(),
                );
                if d.spans.insert(key.clone(), e.span.clone()).is_some() {
                    clashing.insert(key);
                    notes.refusals.insert(Refusal::Collision {
                        span: e.span.clone(),
                        name: qualified(BEHAVIOR, &e.name),
                    });
                    continue;
                }
                if data {
                    let variants = payload
                        .iter()
                        .map(|(v, fs)| {
                            let fs = fs
                                .iter()
                                .map(|(n, t)| (n.clone(), declared_in(pkg, fi, inline, t)))
                                .collect();
                            (v.clone(), fs)
                        })
                        .collect();
                    d.datas.insert(key, (fi, inline.to_string(), variants));
                } else {
                    d.enums
                        .insert(key, e.variants.iter().map(|v| v.name.clone()).collect());
                }
            }
        }
        for (fi, file) in package.files.iter().enumerate() {
            if !product(file) {
                continue;
            }
            for (st, fields) in file.structs.iter().zip(&syntax.files[&file.rel].structs) {
                let inline = syntax.names.type_module(&file.rel, st.span.line, &st.name);
                let key = (
                    file.target,
                    crate::names::join(&file.module, inline),
                    st.name.clone(),
                );
                if d.spans.insert(key.clone(), st.span.clone()).is_some() {
                    clashing.insert(key);
                    notes.refusals.insert(Refusal::Collision {
                        span: st.span.clone(),
                        name: qualified(BEHAVIOR, &st.name),
                    });
                    continue;
                }
                let fields = fields
                    .iter()
                    .map(|(n, t)| (n.clone(), declared_in(pkg, fi, inline, t)))
                    .collect();
                d.structs.insert(key, (fi, inline.to_string(), fields));
            }
        }
        for key in &clashing {
            d.enums.remove(key);
            d.structs.remove(key);
            d.datas.remove(key);
            d.twice.insert(key.2.clone());
        }
        declared.push(d);
    }
    let keys = |d: &Declared| -> Vec<Key> {
        d.enums
            .keys()
            .chain(d.structs.keys())
            .chain(d.datas.keys())
            .cloned()
            .collect()
    };
    // A name one type of the workspace has lifts as itself; one a crate's
    // type shares with another crate's, qualified by the crate; one two of
    // a crate's types share, by the crate, the target where it is not the
    // library, and the module.
    let mut everywhere: BTreeMap<String, usize> = BTreeMap::new();
    for d in &declared {
        for (_, _, name) in keys(d) {
            *everywhere.entry(name).or_default() += 1;
        }
    }
    let mut lexes: Vec<BTreeMap<Key, String>> = Vec::new();
    // Each crate's names that two of its types share: a file reads one by
    // its own module's declaration or its import of it.
    let mut ambiguous: Vec<BTreeSet<String>> = Vec::new();
    for (d, package) in declared.iter().zip(&ws.packages) {
        let ks = keys(d);
        let mut in_crate: BTreeMap<&str, usize> = BTreeMap::new();
        for (_, _, name) in &ks {
            *in_crate.entry(name.as_str()).or_default() += 1;
        }
        let named = ks
            .iter()
            .map(|key| {
                let (target, module, name) = key;
                // A name LexLean writes for a type of its own (`Prop`) is
                // shared with it as with another crate's type.
                let taken = lift::name_taken(name, !d.enums.contains_key(key));
                let lex = if everywhere[name] == 1 && !taken {
                    name.clone()
                } else if in_crate[name.as_str()] == 1 {
                    format!("{}{name}", pascal(&[&package.name]))
                } else {
                    let mut parts: Vec<&str> = vec![&package.name];
                    let t = &package.targets[*target];
                    if !matches!(t.kind, TargetKind::Lib) {
                        parts.push(&t.name);
                    }
                    parts.extend(module.split("::").filter(|s| !s.is_empty()));
                    format!("{}{name}", pascal(&parts))
                };
                (key.clone(), lex)
            })
            .collect();
        lexes.push(named);
        ambiguous.push(
            in_crate
                .into_iter()
                .filter(|(_, n)| *n > 1)
                .map(|(name, _)| name.to_string())
                .collect(),
        );
    }
    // Two keys whose qualified names meet name neither.
    let mut lex_count: BTreeMap<String, usize> = BTreeMap::new();
    for lex in lexes.iter().flat_map(BTreeMap::values) {
        *lex_count.entry(lex.clone()).or_default() += 1;
    }
    for (d, named) in declared.iter_mut().zip(&mut lexes) {
        let met: Vec<Key> = named
            .iter()
            .filter(|(_, lex)| lex_count[*lex] > 1)
            .map(|(key, _)| key.clone())
            .collect();
        for key in met {
            if let Some(span) = d.spans.get(&key) {
                notes.refusals.insert(Refusal::Collision {
                    span: span.clone(),
                    name: qualified(BEHAVIOR, &named[&key]),
                });
            }
            named.remove(&key);
            d.enums.remove(&key);
            d.structs.remove(&key);
            d.datas.remove(&key);
        }
    }
    // Each module's reading of its crate's shared names, by package, file
    // and inline module: what the name resolves to there (names.rs), the
    // type the module declares or its `use` or glob binds, an item outside
    // the workspace by its path, and otherwise none.
    type Overlay = BTreeMap<String, String>;
    let mut overlays: BTreeMap<(usize, usize, String), Overlay> = BTreeMap::new();
    for (pkg, package) in ws.packages.iter().enumerate() {
        for (fi, file) in package.files.iter().enumerate() {
            for inline in syntax.names.inline_modules(&file.rel) {
                let readings = ctx.type_readings.get(&(pkg, fi, inline.clone()));
                // A self type written by a path reads as the type it names.
                let paths = readings
                    .into_iter()
                    .flatten()
                    .filter(|(n, _)| n.contains("::"))
                    .map(|(n, _)| n);
                let names: Overlay = ambiguous[pkg]
                    .iter()
                    .chain(paths)
                    .map(|n| {
                        let read = match readings.and_then(|r| r.get(n)) {
                            Some(crate::names::TypeReading::Declared(p, t, m, name)) => lexes
                                .get(*p)
                                .and_then(|l| l.get(&(*t, m.clone(), name.clone())))
                                .cloned(),
                            Some(crate::names::TypeReading::Extern(path)) => Some(path.clone()),
                            _ => None,
                        };
                        (n.clone(), read.unwrap_or_else(|| n.clone()))
                    })
                    .collect();
                overlays.insert((pkg, fi, inline), names);
            }
        }
    }
    // The overlay of a file's own module, and of an inline module of it.
    let overlay = |pkg: usize, fi: usize, inline: &str| -> &Overlay {
        overlays
            .get(&(pkg, fi, inline.to_string()))
            .unwrap_or_else(|| &overlays[&(pkg, fi, String::new())])
    };
    let scopes: Vec<Types> = declared
        .into_iter()
        .enumerate()
        .map(|(pkg, d)| {
            let package = &ws.packages[pkg];
            let named = &lexes[pkg];
            // A name no other of the crate's types shares reads as its type
            // in every file.
            let mut aliases: BTreeMap<String, String> = named
                .iter()
                .filter(|((_, _, name), lex)| *lex != name && !ambiguous[pkg].contains(name))
                .map(|((_, _, name), lex)| (name.clone(), lex.clone()))
                .collect();
            let mut enums: Variants = d
                .enums
                .into_iter()
                .map(|(key, vs)| (named[&key].clone(), vs))
                .collect();
            // `std::cmp::Ordering`, which derived orders result in, unless
            // a crate declares a type of its name.
            let rust = crate::order::RUST_ORDERING;
            if !everywhere.contains_key(rust) && !everywhere.contains_key(crate::order::ORDERING) {
                enums.insert(
                    crate::order::ORDERING.to_string(),
                    crate::order::ORDERINGS
                        .iter()
                        .map(|v| v.to_string())
                        .collect(),
                );
                aliases.insert(rust.to_string(), crate::order::ORDERING.to_string());
            }
            // A field's type is read in the file that declares it.
            let mut structs: Fields = d
                .structs
                .into_iter()
                .map(|(key, (fi, inline, fs))| {
                    let names = overlay(pkg, fi, &inline);
                    let fs = fs.into_iter().map(|(n, t)| (n, t.renamed(names))).collect();
                    (named[&key].clone(), fs)
                })
                .collect();
            let datas: Datas = d
                .datas
                .into_iter()
                .map(|(key, (fi, inline, vs))| {
                    let names = overlay(pkg, fi, &inline);
                    let vs = vs
                        .into_iter()
                        .map(|(v, fs)| {
                            (
                                v,
                                fs.into_iter().map(|(n, t)| (n, t.renamed(names))).collect(),
                            )
                        })
                        .collect();
                    (named[&key].clone(), vs)
                })
                .collect();
            // The traits each lifted type derives, which say where its
            // order and equality are the structural ones.
            let derives: BTreeMap<String, BTreeSet<String>> = package
                .files
                .iter()
                .flat_map(|file| {
                    let enums = file.enums.iter().map(|e| (&e.name, &e.span, &e.derives));
                    let structs = file
                        .structs
                        .iter()
                        .map(|st| (&st.name, &st.span, &st.derives));
                    enums
                        .chain(structs)
                        .filter_map(|(n, span, ds)| {
                            let inline = syntax.names.type_module(&file.rel, span.line, n);
                            let module = crate::names::join(&file.module, inline);
                            let lex = named.get(&(file.target, module, n.clone()))?;
                            Some((lex.clone(), ds.iter().cloned().collect()))
                        })
                        .collect::<Vec<_>>()
                })
                .collect();
            // A name of the crate that reads as no one type is registered
            // with the reason, so that a row it stops states it: a key
            // declared twice, and a name several of the crate's types
            // share where a file does not say which it means.
            let unread = d
                .twice
                .iter()
                .map(|n| (n, lift::DECLARED_TWICE))
                .chain(ambiguous[pkg].iter().map(|n| (n, lift::SHARED_NAME)));
            for (name, why) in unread {
                let free = !enums.contains_key(name) && !datas.contains_key(name);
                if free && !structs.contains_key(name) {
                    structs.insert(name.clone(), vec![lift::unread_shape(why)]);
                }
            }
            Types {
                derives,
                ..Types::with_data(enums, &datas, &structs, aliases)
            }
        })
        .collect();
    // Each file's scope: its crate's, with the crate's shared names read as
    // the file reads them.
    let file_scopes: BTreeMap<(usize, usize, String), Types> = overlays
        .iter()
        .map(|((pkg, fi, inline), names)| {
            let mut scope = scopes[*pkg].clone();
            scope.aliases.extend(names.clone());
            ((*pkg, *fi, inline.clone()), scope)
        })
        .collect();
    // The inline module a function stands in, which reads its names.
    let inline_of = |r: FnRef| -> String {
        let f = ctx.func(r);
        syntax
            .names
            .fn_module(&ctx.file(r).rel, f.span.line, &f.name)
            .to_string()
    };
    // A function's scope: its module's, an inline module's among them.
    let fn_scope = |r: FnRef| -> &Types {
        file_scopes
            .get(&(r.pkg, r.file, inline_of(r)))
            .unwrap_or_else(|| &file_scopes[&(r.pkg, r.file, String::new())])
    };
    // Each target's scope for the MIR path, which prints a type of the
    // crate by its full path: `project::Model`.
    let mut mir_scopes: BTreeMap<(usize, usize), Types> = BTreeMap::new();
    for (pkg, package) in ws.packages.iter().enumerate() {
        for target in 0..package.targets.len() {
            let mut scope = scopes[pkg].clone();
            for ((t, module, name), lex) in &lexes[pkg] {
                if *t == target {
                    let printed = if module.is_empty() {
                        name.clone()
                    } else {
                        format!("{module}::{name}")
                    };
                    scope.aliases.insert(printed, lex.clone());
                }
            }
            mir_scopes.insert((pkg, target), scope);
        }
    }
    // Every crate's lifted types, for the declarations they emit.
    let mut types = Types::default();
    for scope in &scopes {
        types.enums.extend(scope.enums.clone());
        types.structs.extend(scope.structs.clone());
        types.datas.extend(scope.datas.clone());
        types.derives.extend(scope.derives.clone());
        for family in &scope.families {
            if !types.families.contains(family) {
                types.families.push(family.clone());
            }
        }
    }
    // Typed failures: every error enum's variants, as an inductive.
    for (pkg, package) in ws.packages.iter().enumerate() {
        for (fi, file) in package.files.iter().enumerate() {
            if !matches!(
                package.targets[file.target].kind,
                TargetKind::Lib | TargetKind::Bin
            ) {
                continue;
            }
            for (ei, e) in file.enums.iter().enumerate() {
                if ctx.enum_routes.get(&(pkg, fi, ei)) != Some(&"UR-R12") {
                    continue;
                }
                let name = format!("{}Failure", pascal(&[&e.name]));
                if decls.iter().any(|d: &Decl| d.name() == name) {
                    notes.refusals.insert(Refusal::Collision {
                        span: e.span.clone(),
                        name: qualified(BEHAVIOR, &name),
                    });
                    continue;
                }
                decls.push(Decl::Inductive {
                    name,
                    constructors: e.variants.iter().map(|v| v.name.clone()).collect(),
                });
            }
        }
    }
    let all = ctx.all_fns();
    // The I/O shell: a function that performs an effect, or calls one that does.
    let mut effectful: BTreeMap<FnRef, String> = BTreeMap::new();
    for &r in &all {
        if let Some((cap, op)) = &ctx.func(r).effect {
            effectful.insert(r, format!("{} effect: {op}", cap.key()));
        }
    }
    loop {
        let mut grew = false;
        for &r in &all {
            if effectful.contains_key(&r) {
                continue;
            }
            let via = ctx
                .calls
                .get(&r)
                .and_then(|callees| callees.iter().find(|c| effectful.contains_key(c)));
            if let Some(&callee) = via {
                let reason = format!(
                    "calls {}, which {}",
                    ctx.path(callee),
                    effect_phrase(&effectful[&callee])
                );
                effectful.insert(r, reason);
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    // Each crate lifts on its own: its types, and calls among its functions.
    // A function name two crates share lifts qualified by the crate's name.
    // A free function's name two modules of a crate share lifts qualified
    // by its module.
    let homonyms = homonym_fns(ctx);
    let mut scoped: Vec<(Vec<Candidate<'_>>, BTreeMap<String, FnRef>)> = ws
        .packages
        .iter()
        .map(|_| (Vec::new(), BTreeMap::new()))
        .collect();
    // How many functions of product code each path names, and how many
    // methods each type has of each name over its crate. Rust reads
    // `Type::name` and `value.name()` as the type's one method of that name,
    // an inherent one before a trait's.
    let mut pathed: BTreeMap<String, usize> = BTreeMap::new();
    let mut methods: BTreeMap<(usize, String, String), usize> = BTreeMap::new();
    for &r in all.iter().filter(|&&r| ctx.product(r)) {
        *pathed.entry(ctx.path(r)).or_default() += 1;
        let f = ctx.func(r);
        if let (Some(o), false) = (&f.owner, f.in_trait) {
            let owner = fn_scope(r).resolve(o);
            *methods.entry((r.pkg, owner, f.name.clone())).or_default() += 1;
        }
    }
    // The trait methods a call can mean, each with its crate, the key such a
    // call reads and its own key; and the candidates that give way.
    let mut reach: Vec<(usize, String, String, FnRef)> = Vec::new();
    let mut yielding: BTreeSet<(usize, String)> = BTreeSet::new();
    for &r in &all {
        let f = ctx.func(r);
        let types = fn_scope(r);
        if !ctx.product(r) {
            continue;
        }
        // Methods of a type the lift names lift with `self` as their first
        // parameter, inherent ones and a trait implementation's alike;
        // methods of other types stop, each with a row of its own that says
        // why its type is outside the lift.
        let known = |o: &String| {
            types.enums.contains_key(&types.resolve(o))
                || types.structs.contains_key(&types.resolve(o))
                || types.datas.contains_key(&types.resolve(o))
        };
        // A path Rust's own naming leaves to two trait methods (an impl or
        // a trait declared once per configuration) is one row, and names
        // neither of them.
        if (f.implements.is_some() || f.in_trait) && pathed[&ctx.path(r)] > 1 {
            ctx.stopped(notes, &effectful, r, Residue::Collision(ctx.path(r)));
            continue;
        }
        let fn_syntax = syntax.function(&ctx.file(r).rel, r.idx);
        // `Display::fmt` lifts as the function from the value to the text
        // it writes: `write!` into its formatter is that `format!`.
        let display = matches!(
            (&f.owner, &f.implements),
            (Some(o), Some(t)) if t == "Display" && f.name == "fmt" && !f.in_trait && known(o)
        );
        let method = match (&f.owner, &f.implements) {
            (None, _) => None,
            // A method a trait declares: its default body, where it is one
            // function for every implementing type.
            (Some(declarer), _) if f.in_trait => {
                let refusal = match &fn_syntax.block {
                    None => Some(Residue::Signature(format!(
                        "a method {declarer} declares without a body, which each implementing \
                         type gives"
                    ))),
                    Some(block) => default_refusal(ctx.file(r), f, declarer, block),
                };
                if let Some(reason) = refusal {
                    ctx.stopped(notes, &effectful, r, reason);
                    continue;
                }
                None
            }
            (Some(o), Some(_)) if display => Some(types.resolve(o)),
            (Some(o), implements) => {
                // A trait method over a registry crate's types, such as a
                // `syn` visitor, is the converter shell's: a foreign row.
                let foreign = implements.as_ref().and_then(|_| {
                    let sig = &fn_syntax.signature;
                    sig.inputs
                        .iter()
                        .filter_map(|i| match i {
                            crate::ast::FnArg::Typed(p) => Some(p.ty.text.clone()),
                            crate::ast::FnArg::Receiver { .. } => None,
                        })
                        .chain(sig.output.iter().map(|o| o.text.clone()))
                        .find(|ty| foreign_crate(&ctx.ws.packages[r.pkg], ty).is_some())
                });
                if let Some(ty) = foreign {
                    ctx.stopped(notes, &effectful, r, Residue::Type(ty));
                    continue;
                }
                // An impl with parameters of its own is one impl for each
                // type or constant that takes their place, and its methods
                // are generic as a function with parameters is.
                if implements.is_some() && !f.impl_params.is_empty() {
                    let reason = Residue::Signature(format!(
                        "a method of an impl generic in {}: one function for each type or \
                         constant that takes its place",
                        f.impl_params.join(", ")
                    ));
                    ctx.stopped(notes, &effectful, r, reason);
                    continue;
                }
                if known(o) {
                    Some(types.resolve(o))
                } else if foreign_crate(&ctx.ws.packages[r.pkg], o).is_some() {
                    ctx.stopped(notes, &effectful, r, Residue::Type(o.clone()));
                    continue;
                } else {
                    // The type and why the lift does not name it: a type
                    // the workspace registers by the chain down to its
                    // leaf, as a parameter of the type is, and any other by
                    // what the crate's source shows. The leaf decides the
                    // row: a registry crate's type there makes it foreign.
                    let reason = if types.outside.contains_key(&types.resolve(o)) {
                        types.unnamed(o)
                    } else {
                        let why =
                            unlifted_type(ctx, syntax, &file_scopes, &expanded, r.pkg, r.file, o);
                        Residue::Type(format!("{o}, {why}"))
                    };
                    ctx.stopped(notes, &effectful, r, reason);
                    continue;
                }
            }
        };
        let mut sig = fn_syntax.signature.clone();
        for input in &mut sig.inputs {
            if let crate::ast::FnArg::Typed(t) = input {
                t.ty = declared_in(r.pkg, r.file, &inline_of(r), &t.ty);
            }
        }
        if let Some(t) = &mut sig.output {
            *t = declared_in(r.pkg, r.file, &inline_of(r), t);
        }
        let formatter = match sig.inputs.as_slice() {
            [crate::ast::FnArg::Receiver { plain: true }, crate::ast::FnArg::Typed(t)]
                if display =>
            {
                match &*t.pat {
                    crate::ast::Pat::Ident(i) => Some(i.ident.clone()),
                    _ => None,
                }
            }
            _ => None,
        };
        let sig = &sig;
        let Some(block) = &fn_syntax.block else {
            continue;
        };
        let site = (
            r.pkg,
            ctx.file(r).target,
            ctx.file(r).module.clone(),
            f.name.clone(),
        );
        // The key a call written `Type::name` or `value.name()` reads, where
        // this is a trait's method such a call can mean.
        let mut reached = None;
        let (rust, name) = match (&method, &f.owner, homonyms.get(&site)) {
            (Some(o), _, _) => {
                let plain = (
                    format!("{o}::{}", f.name),
                    lift::local_name(&format!("{o}_{}", f.name)),
                );
                let sole = methods[&(r.pkg, o.clone(), f.name.clone())] == 1;
                match &f.trait_ref {
                    None => plain,
                    // The text a value displays as keeps the key and the
                    // name it had: no call reads it, `write!` and `format!`
                    // do.
                    Some(_) if display && sole => plain,
                    // A trait's method lifts under a key no call writes,
                    // and each scope that sees the trait reads the call's
                    // key as it (`trait_in_scope`). Where its type has
                    // another method of the name, Rust names this one by
                    // its trait and no call is read as it. Nor where the
                    // trait is generic: it is implemented once per argument
                    // list (`From<A>` beside `From<B>` and core's own
                    // `From<T> for T`), and Rust picks among them by the
                    // types at the call. Its definition is named by its
                    // type and its trait whatever else its type has, so the
                    // name is one for each trait of a type and each type of
                    // a trait, and a method the type gains later leaves it.
                    Some(t) => {
                        if sole && !t.contains('<') {
                            reached = Some(plain.0);
                        }
                        (
                            format!("{o}::<{t}>::{}", f.name),
                            lift::local_name(&format!("{o}_{t}_{}", f.name)),
                        )
                    }
                }
            }
            // A default method lifts under its trait's name, and under a
            // key no call writes: a call means an implementing type's.
            (None, Some(declarer), _) => (
                format!("<{declarer}>::{}", f.name),
                lift::local_name(&format!("{declarer}_{}", f.name)),
            ),
            (None, None, Some((rust, lex))) => (rust.clone(), lex.clone()),
            (None, None, None) => (f.name.clone(), lift::local_name(&f.name)),
        };
        // What a call of the function by itself is written as: a free
        // function's name, a method's key, and a trait's method as its own
        // scope reads it.
        let called = match (&method, &f.owner, &reached) {
            (None, None, _) => f.name.clone(),
            (_, _, Some(plain)) if trait_in_scope(ctx, r, r) => plain.clone(),
            _ => rust.clone(),
        };
        if let Some(plain) = reached {
            reach.push((r.pkg, plain, rust.clone(), r));
        }
        // A trait's method gives way: a free function or an inherent method
        // that lifts to the same name keeps it.
        let yields = f.in_trait || (f.implements.is_some() && !display);
        let owner = match (&method, &f.owner) {
            (Some(o), _) => Some(lift::Owner::Type {
                name: o,
                receiver: f.receiver.as_deref(),
            }),
            (None, Some(_)) => Some(lift::Owner::Implementor),
            (None, None) => None,
        };
        let (candidates, names) = &mut scoped[r.pkg];
        let signature = match (&formatter, &method) {
            (Some(_), Some(o)) => Ok((
                vec![(
                    "self".to_string(),
                    lift::local_name("self"),
                    lift::Ty::Named(o.clone()),
                )],
                lift::Ty::Str,
            )),
            (None, _) if display => Err(Residue::Signature(
                "a Display impl whose formatter is not a plain parameter".to_string(),
            )),
            _ => types.signature(sig, owner.as_ref()),
        };
        match signature {
            Ok((params, result)) => {
                let holder = candidates
                    .iter()
                    .position(|c: &Candidate<'_>| c.name == name)
                    .filter(|&i| {
                        !yields
                            && !names.contains_key(&rust)
                            && yielding.contains(&(r.pkg, candidates[i].rust.clone()))
                    });
                if let Some(held) = holder.and_then(|i| names.remove(&candidates[i].rust)) {
                    // The name's holder gives way, and keeps its row.
                    candidates.retain(|c| c.name != name);
                    notes.refusals.insert(Refusal::Collision {
                        span: ctx.func(held).span.clone(),
                        name: qualified(BEHAVIOR, &name),
                    });
                    ctx.stopped(notes, &effectful, held, Residue::Collision(name.clone()));
                } else if names.contains_key(&rust)
                    || candidates.iter().any(|c: &Candidate<'_>| c.name == name)
                {
                    notes.refusals.insert(Refusal::Collision {
                        span: f.span.clone(),
                        name: qualified(BEHAVIOR, &name),
                    });
                    // The register keeps a row for it all the same.
                    ctx.stopped(notes, &effectful, r, Residue::Collision(name));
                    continue;
                }
                if yields {
                    yielding.insert((r.pkg, rust.clone()));
                }
                names.insert(rust.clone(), r);
                candidates.push(Candidate {
                    rust,
                    name,
                    params,
                    result,
                    block,
                    imports: ctx.file(r).imports.clone(),
                    aliases: overlay(r.pkg, r.file, &inline_of(r)).clone(),
                    homonyms: homonym_reading(ctx, &homonyms, r.pkg, r.file),
                    called,
                    formatter,
                });
            }
            Err(reason) => ctx.stopped(notes, &effectful, r, reason),
        }
    }
    // Constants lift as definitions of no parameters.
    for (pkg, package) in ws.packages.iter().enumerate() {
        for (fi, file) in package.files.iter().enumerate() {
            let types = &file_scopes[&(pkg, fi, String::new())];
            if !matches!(
                package.targets[file.target].kind,
                TargetKind::Lib | TargetKind::Bin
            ) {
                continue;
            }
            for (rust, ty, block) in &syntax.files[&file.rel].consts {
                let Ok(result) = types.ty(ty) else {
                    continue;
                };
                let (candidates, names) = &mut scoped[pkg];
                // A constant spelled like a function, `REVERSE` beside
                // `reverse`, takes its kind as a suffix.
                let plain = lift::local_name(rust);
                let name = if candidates.iter().any(|c: &Candidate<'_>| c.name == plain) {
                    lift::local_name(&format!("{rust}_CONST"))
                } else {
                    plain
                };
                if names.contains_key(rust)
                    || candidates
                        .iter()
                        .any(|c: &Candidate<'_>| c.rust == *rust || c.name == name)
                {
                    continue;
                }
                candidates.push(Candidate {
                    rust: rust.clone(),
                    name,
                    params: Vec::new(),
                    result,
                    block,
                    imports: file.imports.clone(),
                    aliases: overlay(pkg, fi, "").clone(),
                    homonyms: homonym_reading(ctx, &homonyms, pkg, fi),
                    called: rust.clone(),
                    formatter: None,
                });
            }
        }
    }
    // Each candidate reads a call's key as the trait's method where its
    // scope sees the trait, and as no function where it does not. A
    // constant's value calls no trait's method.
    for (pkg, (candidates, names)) in scoped.iter_mut().enumerate() {
        for c in candidates.iter_mut() {
            let caller = names.get(&c.rust).copied();
            for (_, plain, key, imp) in reach.iter().filter(|(p, ..)| *p == pkg) {
                let seen = caller.is_some_and(|caller| trait_in_scope(ctx, caller, *imp));
                c.homonyms.insert(plain.clone(), seen.then(|| key.clone()));
            }
        }
    }
    let mut shared: BTreeMap<String, usize> = BTreeMap::new();
    for (candidates, _) in &scoped {
        for c in candidates {
            *shared.entry(c.name.clone()).or_default() += 1;
        }
    }
    for ((candidates, _), package) in scoped.iter_mut().zip(&ws.packages) {
        for c in candidates.iter_mut() {
            if shared[&c.name] > 1 {
                c.name = lift::local_name(&format!("{}_{}", package.name, c.name));
            }
        }
    }
    // A parameter may not be spelled like a definition: generated Lean would
    // capture the definition.
    let defined: BTreeSet<String> = scoped
        .iter()
        .flat_map(|(candidates, _)| candidates.iter().map(|c| c.name.clone()))
        .collect();
    for (candidates, _) in &mut scoped {
        for c in candidates.iter_mut() {
            let mut taken: BTreeSet<String> = c.params.iter().map(|(_, n, _)| n.clone()).collect();
            for (_, n, _) in &mut c.params {
                if defined.contains(n.as_str()) {
                    let mut k = 1;
                    let renamed = loop {
                        let candidate = format!(
                            "{n}Arg{}",
                            if k == 1 { String::new() } else { k.to_string() }
                        );
                        if !defined.contains(&candidate) && !taken.contains(&candidate) {
                            break candidate;
                        }
                        k += 1;
                    };
                    taken.insert(renamed.clone());
                    *n = renamed;
                }
            }
        }
    }
    let mut lifted = Vec::new();
    let mut mentioned: BTreeSet<Ty> = BTreeSet::new();
    let mut ordered: BTreeSet<Ty> = BTreeSet::new();
    // Per crate: the lifted functions' signatures and their axioms.
    let mut results: Vec<(Sigs, Axioms)> = Vec::new();
    // Every lifted function by its crate path, for the MIR path's calls.
    let mut callees: BTreeMap<String, crate::mir::lower::Callee> = BTreeMap::new();
    let every_name: BTreeSet<String> = scoped
        .iter()
        .flat_map(|(candidates, _)| candidates.iter().map(|c| c.name.clone()))
        .collect();
    for ((candidates, names), types) in scoped.iter().zip(&scopes) {
        let widths = lift::pointer_widths(&ctx.ws.targets);
        let out = lift::lift_all(candidates, types, &widths, &every_name);
        for (rust, reason) in out.failed {
            if let Some(&r) = names.get(&rust) {
                ctx.stopped(notes, &effectful, r, reason);
            }
        }
        mentioned.extend(out.types);
        ordered.extend(out.ordered);
        for c in candidates {
            // A constant has no function row of its own.
            if let (true, Some(&r)) = (
                out.decls.iter().any(|d| d.name() == c.name),
                names.get(&c.rust),
            ) {
                // A path has one row and one κ. Two functions share a path
                // where two traits give one type a method of one name (the
                // `fmt` of `Display` and of `Debug`): where the other did
                // not lift, the path's row stays the one that says so,
                // since the κ the register holds for the path may be
                // either function's.
                let path = ctx.path(r);
                if notes.residue.contains_key(&path)
                    || notes.foreign.contains_key(&path)
                    || notes.effects.contains_key(&path)
                {
                    continue;
                }
                notes.lifted.insert(
                    path,
                    (
                        ctx.func(r).span.clone(),
                        qualified(BEHAVIOR, &c.name),
                        out.divergences.get(&c.rust).cloned().unwrap_or_default(),
                    ),
                );
            }
        }
        // A closed instance is a theorem only where the kernel can compute it.
        let sigs: BTreeMap<String, lift::Sig> = candidates
            .iter()
            .filter(|c| out.decls.iter().any(|d| d.name() == c.name))
            .filter(|c| !out.opaque.contains(&c.rust))
            .map(|c| {
                (
                    c.rust.clone(),
                    lift::Sig {
                        name: c.name.clone(),
                        params: c.params.iter().map(|(_, _, t)| t.clone()).collect(),
                        result: c.result.clone(),
                    },
                )
            })
            .collect();
        for c in candidates {
            if let (true, Some(&r)) = (
                out.decls.iter().any(|d| d.name() == c.name),
                names.get(&c.rust),
            ) {
                callees.insert(
                    ctx.path(r),
                    crate::mir::lower::Callee {
                        name: c.name.clone(),
                        params: c.params.iter().map(|(_, _, t)| t.clone()).collect(),
                        result: c.result.clone(),
                        threads: None,
                    },
                );
            }
        }
        lifted.extend(out.decls);
        results.push((sigs, out.axioms));
    }
    // The MIR path, for every function the surface lift left as residue.
    let mut listing = None;
    let mut translated = Vec::new();
    if let Some((dumps, options)) = mir {
        let mut taken = every_name.clone();
        taken.extend(lifted.iter().map(|d| d.name().to_string()));
        taken.extend(types.enums.keys().cloned());
        taken.extend(types.structs.keys().cloned());
        taken.extend(types.datas.keys().cloned());
        let out = mir_lift(ctx, notes, dumps, options, &mir_scopes, &callees, &taken);
        mentioned.extend(out.types);
        translated = out.decls;
        listing = out.listing;
    }
    // The enums, structs and tuples the lifted definitions name, each after
    // the types its fields name.
    let mut rendered: Vec<String> = lifted.iter().map(crate::lex::render_decl).collect();
    rendered.extend(translated.iter().map(crate::lex::render_decl));
    let compared: BTreeSet<String> = types
        .enums
        .keys()
        .filter(|n| {
            let call = format!("({} ", lift::equality_name(n));
            rendered.iter().any(|d| d.contains(&call))
        })
        .cloned()
        .collect();
    let mut emitted: BTreeSet<Ty> = BTreeSet::new();
    // The raw pointer and the memory, where a translation reaches memory:
    // the MIR path's own structures (see `mir_lift`'s scopes).
    let mut with_memory = types.clone();
    for (name, fields) in [
        (
            crate::mir::lower::POINTER.to_string(),
            crate::mir::lower::pointer_fields(),
        ),
        (
            crate::mir::memory::MEMORY.to_string(),
            crate::mir::memory::memory_fields(),
        ),
    ] {
        if mentioned.contains(&Ty::Named(name.clone())) {
            with_memory.structs.entry(name).or_insert(fields);
        }
    }
    for ty in &mentioned {
        type_decls(ty, &with_memory, &compared, &mut emitted, &mut decls);
    }
    // The derived orders the definitions compare by, after the types.
    if !ordered.is_empty() {
        decls.extend(crate::order::tests());
        decls.push(crate::order::reverse());
        decls.extend(crate::order::decls(&ordered, &types));
    }
    decls.extend(lifted);
    decls.extend(translated);
    // Closed assertions of tests over lifted functions are theorems.
    let mut theorems = BTreeSet::new();
    for &r in &all {
        let f = ctx.func(r);
        if !f.test {
            continue;
        }
        let (sigs, axioms) = &results[r.pkg];
        for (i, (kind, args)) in syntax
            .function(&ctx.file(r).rel, r.idx)
            .asserts
            .iter()
            .enumerate()
        {
            if let Some(accepted) = lift::acceptance(kind, args, sigs, fn_scope(r)) {
                let name = format!("accept{}{}", pascal(&[&ctx.package(r).name, &f.name]), i);
                if theorems.insert(name.clone()) {
                    let axioms = axioms.get(&accepted.callee).cloned().unwrap_or_default();
                    // The acceptance root: the claim as a closed Boolean
                    // definition, which a native library lists and its
                    // generated package executes (SPEC 12.6).
                    let root = format!("{name}Root");
                    notes
                        .roots
                        .entry(ctx.package(r).name.clone())
                        .or_default()
                        .push(root.clone());
                    decls.push(Decl::Definition {
                        name: root.clone(),
                        parameters: Vec::new(),
                        result: "Bool".to_string(),
                        body: Expr::Flat(accepted.check),
                        axioms: axioms.clone(),
                        recursive: None,
                        termination: None,
                        mutual: None,
                    });
                    decls.push(Decl::Theorem {
                        name: format!("{root}Holds"),
                        parameters: Vec::new(),
                        statement: format!("(({root}) = true)"),
                        proof: "decide{}".to_string(),
                        axioms: axioms.clone(),
                    });
                    decls.push(Decl::Theorem {
                        name,
                        parameters: Vec::new(),
                        statement: accepted.statement,
                        proof: "decide{}".to_string(),
                        axioms,
                    });
                }
            }
        }
    }
    (
        Module {
            name: BEHAVIOR.into(),
            glossaries: vec![BOOL, NAT, TITLES],
            imports: vec![lift::PRELUDE.into()],
            title: "Lifted Rust behavior",
            decls,
        },
        listing,
    )
}

/// What the MIR path adds to the behavior module.
struct MirOut {
    decls: Vec<Decl>,
    types: BTreeSet<Ty>,
    /// The trial run's listing of the translations it admitted on trial.
    listing: Option<String>,
}

/// One function's translation.
struct Translation {
    r: FnRef,
    path: String,
    kappa: String,
    name: String,
    lowered: crate::mir::lower::Lowered,
    digest: String,
}

/// The MIR path: every function the surface lift left as residue, whose
/// body the dump holds, translated from MIR. A translation is admitted when
/// the verdict register records it proved (or, on the prover's trial run,
/// when it has no verdict yet) and every translation it calls is admitted;
/// every other function's residue row names the MIR kind or the obligation
/// that stopped it.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn mir_lift(
    ctx: &Model,
    notes: &mut Notes,
    dumps: &crate::mir::Dumps,
    options: &crate::mir::MirOptions,
    scopes: &BTreeMap<(usize, usize), Types>,
    lifted: &BTreeMap<String, crate::mir::lower::Callee>,
    taken: &BTreeSet<String>,
) -> MirOut {
    use crate::mir::lower;
    use crate::mir::verdict::Decision;
    let homonyms = homonym_fns(ctx);
    // Every crate's closures, by the type MIR prints for each: a closure's
    // type is the struct of the values it captures.
    let closures: BTreeMap<String, crate::mir::IrBody> = dumps
        .closures
        .values()
        .flat_map(|c| c.iter().map(|(t, b)| (t.clone(), b.clone())))
        .collect();
    let signatures: BTreeMap<String, Vec<String>> = closures
        .iter()
        .filter(|(_, b)| b.upvars.iter().all(Option::is_some))
        .map(|(t, b)| {
            let captures = b.upvars.iter().flatten().map(|(_, t)| t.clone()).collect();
            (t.clone(), captures)
        })
        .collect();
    // The data layouts memory is laid out by: only where one is recorded
    // for every declared target, so that a layout used holds on each.
    let layouts: Vec<crate::mir::layout::DataLayout> = if !ctx.ws.targets.is_empty()
        && ctx
            .ws
            .targets
            .iter()
            .all(|t| dumps.layouts.iter().any(|l| &l.target == t))
    {
        dumps
            .layouts
            .iter()
            .filter(|l| ctx.ws.targets.contains(&l.target))
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    let scopes: BTreeMap<(usize, usize), Types> = scopes
        .iter()
        .map(|(&at, s)| {
            let mut scope = Types {
                closures: signatures.clone(),
                ..s.clone()
            };
            // Where the targets' layouts are known, a raw pointer and the
            // memory it points into are declared, as in core's scope.
            if !layouts.is_empty() {
                scope
                    .structs
                    .entry(lower::POINTER.to_string())
                    .or_insert_with(lower::pointer_fields);
                scope
                    .structs
                    .entry(crate::mir::memory::MEMORY.to_string())
                    .or_insert_with(crate::mir::memory::memory_fields);
            }
            (at, scope)
        })
        .collect();
    let scope = |r: FnRef| &scopes[&(r.pkg, ctx.file(r).target)];
    let widths = lift::pointer_widths(&ctx.ws.targets);
    // Enums whose variants set their discriminants, by LexLean name.
    let mut explicit: BTreeSet<String> = BTreeSet::new();
    for (pkg, package) in ctx.ws.packages.iter().enumerate() {
        for file in &package.files {
            for e in file.enums.iter().filter(|e| e.explicit) {
                let printed = if file.module.is_empty() {
                    e.name.clone()
                } else {
                    format!("{}::{}", file.module, e.name)
                };
                explicit.insert(scopes[&(pkg, file.target)].resolve(&printed));
            }
        }
    }
    let product: BTreeMap<String, FnRef> = ctx
        .all_fns()
        .into_iter()
        .filter(|&r| ctx.product(r))
        .map(|r| (ctx.path(r), r))
        .collect();
    // The residue functions whose bodies the dump holds, with their names
    // and signatures.
    let mut todo: Vec<(FnRef, String, &crate::mir::IrBody)> = Vec::new();
    let mut callees = lifted.clone();
    let mut names: BTreeSet<String> = taken.clone();
    let residue: Vec<String> = notes.residue.keys().cloned().collect();
    for path in residue {
        let Some(&r) = product.get(&path) else {
            continue;
        };
        let krate = ctx.crate_ident(r);
        let local = path
            .strip_prefix(&format!("{krate}::"))
            .unwrap_or(&path)
            .to_string();
        let Some(body) = dumps.crates.get(&krate).and_then(|c| c.get(&local)) else {
            continue;
        };
        let f = ctx.func(r);
        let site = (
            r.pkg,
            ctx.file(r).target,
            ctx.file(r).module.clone(),
            f.name.clone(),
        );
        let base = match (&f.owner, homonyms.get(&site)) {
            (Some(o), _) => lift::local_name(&format!("{o}_{}", f.name)),
            (None, Some((_, lex))) => lex.clone(),
            (None, None) => lift::local_name(&f.name),
        };
        let name = if names.contains(&base) {
            lift::local_name(&format!("{}_{base}", ctx.package(r).name))
        } else {
            base
        };
        if !names.insert(name.clone()) {
            let span = f.span.clone();
            notes.residue.insert(path, (span, Residue::Collision(name)));
            continue;
        }
        let types = scope(r);
        // The lift's signature: its parameters (a `&mut T` as the `T` it
        // borrows), its forward result (`_0`'s type), the positions of its
        // threaded parameters (`lower::threaded`), and the definition's
        // result.
        let sig: Result<(Vec<Ty>, Ty, Vec<usize>, Ty), Residue> = (|| {
            let mut params = Vec::new();
            for l in 1..=body.arity {
                let text = &body.locals[l];
                params.push(lower::ty(lower::mut_target(text).unwrap_or(text), types)?);
            }
            let ret = lower::ty(&body.locals[0], types)?;
            let closure = crate::mir::closure_type(body).is_some();
            let positions: Vec<usize> = lower::threaded(&body.locals, body.arity, closure, types)
                .into_iter()
                .map(|l| l - 1)
                .collect();
            let rest: Vec<Ty> = positions.iter().map(|&i| params[i].clone()).collect();
            let full = lower::threaded_result(&ret, &rest, types);
            Ok((params, ret, positions, full))
        })();
        match sig {
            Ok((params, result, positions, full)) => {
                callees.insert(
                    path.clone(),
                    lower::Callee {
                        name: name.clone(),
                        params,
                        result,
                        threads: (!positions.is_empty()).then_some((full, positions)),
                    },
                );
                todo.push((r, name, body));
            }
            Err(reason) => {
                let span = f.span.clone();
                notes.residue.insert(path, (span, mir_residue(reason)));
            }
        }
    }
    // The functions the MIR path translates, by LexLean name.
    let translations: BTreeMap<String, String> = todo
        .iter()
        .map(|(r, name, _)| (name.clone(), ctx.path(*r)))
        .collect();
    // Translate each.
    let no_consts = BTreeMap::new();
    let mut done: BTreeMap<String, Translation> = BTreeMap::new();
    for (r, name, body) in todo {
        let path = ctx.path(r);
        let cx = lower::Cx {
            types: scope(r),
            widths: &widths,
            callees: &callees,
            krate: ctx.crate_ident(r),
            taken: &names,
            explicit: &explicit,
            limit: MAX_FILE_BYTES,
            consts: dumps.consts.get(&ctx.crate_ident(r)).unwrap_or(&no_consts),
            closures: &closures,
            active: &[],
            core: None,
            layouts: &layouts,
            backward: false,
        };
        match lower::lower(&cx, body, &name) {
            Ok(lowered) => {
                let text: String = lowered
                    .aux
                    .values()
                    .chain(&lowered.decls)
                    .map(crate::lex::render_decl)
                    .collect();
                let digest = sha(text.as_bytes());
                let kappa = crate::register::kappa(&path, &ctx.func(r).tokens);
                done.insert(
                    name.clone(),
                    Translation {
                        r,
                        path,
                        kappa,
                        name,
                        lowered,
                        digest,
                    },
                );
            }
            Err(reason) => {
                let span = ctx.func(r).span.clone();
                notes.residue.insert(path, (span, mir_residue(reason)));
            }
        }
    }
    // Admission: a verdict of the digest, or the trial; then every
    // translation a translation calls must be admitted too.
    let mut admitted: BTreeMap<String, bool> = BTreeMap::new();
    let mut causes: BTreeMap<String, String> = BTreeMap::new();
    let mut on_trial: BTreeSet<String> = BTreeSet::new();
    // A translation that calls one the path could not translate, directly
    // or through others, is never admitted, whatever its verdict: that
    // callee is its cause, and it is not put on trial.
    let mut blocked: BTreeMap<String, String> = BTreeMap::new();
    loop {
        let mut changed = false;
        for t in done.values() {
            if blocked.contains_key(&t.name) {
                continue;
            }
            let stopped = t.lowered.calls.iter().find(|c| {
                translations.contains_key(*c)
                    && (!done.contains_key(*c) || blocked.contains_key(*c))
            });
            if let Some(c) = stopped {
                blocked.insert(t.name.clone(), translations[c].clone());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for t in done.values() {
        if let Some(callee) = blocked.get(&t.name) {
            causes.insert(
                t.name.clone(),
                format!("TerminatorKind::Call: {callee}, whose translation is not admitted"),
            );
            admitted.insert(t.name.clone(), false);
            continue;
        }
        let verdict = options
            .verdicts
            .rows
            .get(&t.kappa)
            .filter(|v| v.digest == t.digest);
        let ok = match verdict.map(|v| &v.status) {
            Some(Decision::Proved) => true,
            Some(Decision::Refused(cause)) => {
                causes.insert(t.name.clone(), format!("Verdict: {cause}"));
                false
            }
            None if options.trial.admits(&t.kappa) => {
                on_trial.insert(t.name.clone());
                true
            }
            None => {
                causes.insert(
                    t.name.clone(),
                    "Verdict: a translation the verifier has not decided yet".to_string(),
                );
                false
            }
        };
        admitted.insert(t.name.clone(), ok);
    }
    loop {
        let mut changed = false;
        for t in done.values() {
            if !admitted[&t.name] {
                continue;
            }
            let stopped = t
                .lowered
                .calls
                .iter()
                .find(|c| translations.contains_key(*c) && admitted.get(*c) != Some(&true));
            if let Some(c) = stopped {
                let callee = translations[c].clone();
                causes.insert(
                    t.name.clone(),
                    format!("TerminatorKind::Call: {callee}, whose translation is not admitted"),
                );
                admitted.insert(t.name.clone(), false);
                on_trial.remove(&t.name);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // The admitted translations, each after the translations it calls, with
    // the axioms the verifier observed.
    let mut order: Vec<String> = Vec::new();
    let mut placed: BTreeSet<String> = BTreeSet::new();
    let names_in: Vec<String> = done.keys().cloned().collect();
    for n in &names_in {
        place_after_callees(n, &done, &admitted, &mut placed, &mut order);
    }
    let mut out = MirOut {
        decls: Vec::new(),
        types: BTreeSet::new(),
        listing: None,
    };
    let mut aux: BTreeMap<String, Decl> = BTreeMap::new();
    let mut body_decls = Vec::new();
    let mut listing = String::from("spec = \"uorust/mir-candidates/1\"\n");
    for n in &order {
        let t = &done[n];
        let learned = options
            .verdicts
            .rows
            .get(&t.kappa)
            .filter(|v| v.digest == t.digest)
            .map(|v| v.axioms.clone())
            .or_else(|| options.learned.get(&t.kappa).cloned())
            .unwrap_or_default();
        for (k, d) in &t.lowered.aux {
            aux.entry(k.clone())
                .or_insert_with(|| with_axioms(d.clone(), &learned));
        }
        for d in &t.lowered.decls {
            body_decls.push(with_axioms(d.clone(), &learned));
        }
        out.types.extend(t.lowered.types.iter().cloned());
        notes.residue.remove(&t.path);
        notes.lifted.insert(
            t.path.clone(),
            (
                ctx.func(t.r).span.clone(),
                qualified(BEHAVIOR, &t.name),
                Vec::new(),
            ),
        );
    }
    for t in done.values() {
        let decls: Vec<String> = t
            .lowered
            .aux
            .keys()
            .cloned()
            .chain(t.lowered.decls.iter().map(|d| d.name().to_string()))
            .map(|d| toml_string(&d))
            .collect();
        listing.push_str(&format!(
            "\n[[candidate]]\nkappa = {}\nfunction = {}\ndigest = {}\ntrial = {}\ndecls = [{}]\n",
            toml_string(&t.kappa),
            toml_string(&t.path),
            toml_string(&t.digest),
            on_trial.contains(&t.name),
            decls.join(", ")
        ));
        if let Some(cause) = causes.get(&t.name) {
            let span = ctx.func(t.r).span.clone();
            notes
                .residue
                .insert(t.path.clone(), (span, Residue::Mir(cause.clone())));
        }
    }
    out.decls.extend(in_dependency_order(aux));
    out.decls.extend(body_decls);
    if options.trial != crate::mir::Trial::Off {
        out.listing = Some(listing);
    }
    out
}

/// Auxiliary declarations, each after every other one its text names (a
/// helper after the definitions it applies, a theorem after the
/// definitions it states), in name order otherwise.
fn in_dependency_order(aux: BTreeMap<String, Decl>) -> Vec<Decl> {
    let texts: BTreeMap<String, String> = aux
        .iter()
        .map(|(n, d)| (n.clone(), crate::lex::render_decl(d)))
        .collect();
    let names: Vec<String> = aux.keys().cloned().collect();
    let mentions = |text: &str, name: &str| {
        text.match_indices(name).any(|(i, _)| {
            let word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
            !word(text[..i].chars().next_back()) && !word(text[i + name.len()..].chars().next())
        })
    };
    let mut placed: BTreeSet<String> = BTreeSet::new();
    let mut order: Vec<String> = Vec::new();
    fn visit(
        n: &str,
        names: &[String],
        texts: &BTreeMap<String, String>,
        mentions: &dyn Fn(&str, &str) -> bool,
        placed: &mut BTreeSet<String>,
        order: &mut Vec<String>,
    ) {
        if !placed.insert(n.to_string()) {
            return;
        }
        for m in names {
            if m != n && mentions(&texts[n], m) {
                visit(m, names, texts, mentions, placed, order);
            }
        }
        order.push(n.to_string());
    }
    for n in &names {
        visit(n, &names, &texts, &mentions, &mut placed, &mut order);
    }
    let mut aux = aux;
    order.into_iter().filter_map(|n| aux.remove(&n)).collect()
}

/// A translation's residue: a type the lift does not name is a MIR type
/// residue too.
pub(crate) fn mir_residue(reason: Residue) -> Residue {
    match reason {
        Residue::Mir(_) => reason,
        Residue::Type(t) => Residue::Mir(format!("Ty: {t}")),
        other => Residue::Mir(format!("Body: {}", other.detail())),
    }
}

/// `decl` with the axioms the verifier observed for it, where it reported
/// them.
fn with_axioms(decl: Decl, learned: &BTreeMap<String, Vec<String>>) -> Decl {
    let Some(observed) = learned.get(decl.name()) else {
        return decl;
    };
    match decl {
        Decl::Definition {
            name,
            parameters,
            result,
            body,
            recursive,
            termination,
            mutual,
            ..
        } => Decl::Definition {
            name,
            parameters,
            result,
            body,
            axioms: observed.clone(),
            recursive,
            termination,
            mutual,
        },
        Decl::Theorem {
            name,
            parameters,
            statement,
            proof,
            ..
        } => Decl::Theorem {
            name,
            parameters,
            statement,
            proof,
            axioms: observed.clone(),
        },
        other => other,
    }
}

/// Place translation `n` after every admitted translation it calls.
fn place_after_callees(
    n: &str,
    done: &BTreeMap<String, Translation>,
    admitted: &BTreeMap<String, bool>,
    placed: &mut BTreeSet<String>,
    order: &mut Vec<String>,
) {
    if !admitted.get(n).copied().unwrap_or(false) || !placed.insert(n.to_string()) {
        return;
    }
    for c in &done[n].lowered.calls {
        if done.contains_key(c) {
            place_after_callees(c, done, admitted, placed, order);
        }
    }
    order.push(n.to_string());
}

/// The definition deciding equality of a unit-variant enum's values: a
/// match on both.
fn enum_equality(name: &str, variants: &[String]) -> Decl {
    let arms = |left: &String| {
        variants
            .iter()
            .map(|v| Arm {
                pattern: format!("{name}.{v}"),
                body: Expr::Flat((v == left).to_string()),
            })
            .collect()
    };
    Decl::Definition {
        name: lift::equality_name(name),
        parameters: vec![("left".into(), name.into()), ("right".into(), name.into())],
        result: "Bool".into(),
        body: Expr::Match {
            scrutinee: "left".into(),
            arms: variants
                .iter()
                .map(|v| Arm {
                    pattern: format!("{name}.{v}"),
                    body: Expr::Match {
                        scrutinee: "right".into(),
                        arms: arms(v),
                    },
                })
                .collect(),
        },
        axioms: Vec::new(),
        recursive: None,
        termination: None,
        mutual: None,
    }
}

/// Unit-variant enums by name, with their variants.
type Variants = BTreeMap<String, Vec<String>>;
/// Structs by name, with their named fields.
type Fields = BTreeMap<String, Vec<(String, crate::ast::RustType)>>;
/// Data-carrying enum name -> its variants' fields.
type Datas = BTreeMap<String, lift::RawVariants>;
/// A crate's lifted functions, by Rust name.
type Sigs = BTreeMap<String, lift::Sig>;
/// The Lean axioms of each lifted function, by Rust name.
type Axioms = BTreeMap<String, Vec<String>>;

/// A recursive family, after the types its members name: one inductive per
/// member (one mutual group when there are several, a struct as an
/// inductive of one constructor `mk`), the accessors of the structs' fields,
/// and, for a nested family, the mutual group of its size functions that
/// well-founded recursion over it measures by.
fn family_decls(
    family: &BTreeSet<String>,
    types: &Types,
    compared: &BTreeSet<String>,
    emitted: &mut BTreeSet<Ty>,
    decls: &mut Vec<Decl>,
) {
    for m in family {
        emitted.insert(Ty::Named(m.clone()));
    }
    for m in family {
        for t in types.member_fields(m) {
            let inner = t.mentions();
            for dep in &inner {
                type_decls(dep, types, compared, emitted, decls);
            }
        }
    }
    let first = family.iter().next().cloned().unwrap_or_default();
    let label = (family.len() > 1).then(|| format!("{first}Group"));
    // Each member's constructors: name and field types.
    let ctors = |m: &String| -> Vec<(String, Vec<Ty>)> {
        match types.datas.get(m) {
            Some(variants) => variants
                .iter()
                .map(|(v, fs)| (v.clone(), fs.iter().map(|(_, t)| t.clone()).collect()))
                .collect(),
            None => vec![(
                "mk".to_string(),
                types.structs[m].iter().map(|(_, t)| t.clone()).collect(),
            )],
        }
    };
    for m in family {
        decls.push(Decl::Data {
            name: m.clone(),
            constructors: ctors(m)
                .into_iter()
                .map(|(c, fs)| (c, fs.iter().map(Ty::render).collect()))
                .collect(),
            mutual: label.clone(),
        });
    }
    for m in family.iter().filter(|m| types.boxed(m)) {
        let fields = &types.structs[m];
        let binders: Vec<String> = (0..fields.len()).map(|i| format!("x{i}")).collect();
        for (i, (f, t)) in fields.iter().enumerate() {
            decls.push(Decl::Definition {
                name: Types::accessor(m, f),
                parameters: vec![("value".into(), m.clone())],
                result: t.render(),
                body: Expr::Match {
                    scrutinee: "value".into(),
                    arms: vec![Arm {
                        pattern: format!("{m}.mk {}", binders.join(" ")),
                        body: Expr::Flat(binders[i].clone()),
                    }],
                },
                axioms: Vec::new(),
                recursive: None,
                termination: None,
                mutual: None,
            });
        }
    }
    if !types.nested(&first) || !types.sizable(family) {
        return;
    }
    // The types the sizes range over: the members, then the lists and
    // options of members the members hold.
    let domains = types.size_domains(family);
    let size_label = format!("SizeOf{first}");
    let measure = |t: &Ty, v: &str| types.size_name(t, family).map(|f| format!("({f} {v})"));
    let sum = |base: &str, parts: Vec<String>| {
        parts
            .into_iter()
            .fold(base.to_string(), |acc, p| format!("({acc} + {p})"))
    };
    for d in &domains {
        let arms: Vec<Arm> = match d {
            Ty::Named(m) => ctors(m)
                .into_iter()
                .map(|(c, fs)| {
                    let names: Vec<String> = (0..fs.len()).map(|i| format!("x{i}")).collect();
                    let parts = fs
                        .iter()
                        .zip(&names)
                        .filter_map(|(t, v)| measure(t, v))
                        .collect();
                    Arm {
                        pattern: format!(
                            "{m}.{c}{}",
                            names.iter().map(|v| format!(" {v}")).collect::<String>()
                        ),
                        body: Expr::Flat(sum("1", parts)),
                    }
                })
                .collect(),
            Ty::List(t) => vec![
                Arm {
                    pattern: "List.nil".into(),
                    body: Expr::Flat("0".into()),
                },
                Arm {
                    pattern: "List.cons x0 x1".into(),
                    body: Expr::Flat(sum(
                        "1",
                        vec![
                            measure(t, "x0").unwrap_or_default(),
                            measure(d, "x1").unwrap_or_default(),
                        ],
                    )),
                },
            ],
            Ty::Option(t) => vec![
                Arm {
                    pattern: "Option.none".into(),
                    body: Expr::Flat("0".into()),
                },
                Arm {
                    pattern: "Option.some x0".into(),
                    body: Expr::Flat(sum("1", vec![measure(t, "x0").unwrap_or_default()])),
                },
            ],
            _ => continue,
        };
        decls.push(Decl::Definition {
            name: types.size_name(d, family).unwrap_or_default(),
            parameters: vec![("value".into(), d.render())],
            result: "Nat".into(),
            body: Expr::Match {
                scrutinee: "value".into(),
                arms,
            },
            axioms: Vec::new(),
            recursive: Some("value".into()),
            termination: None,
            mutual: Some(size_label.clone()),
        });
    }
}

/// The declaration of a named type or tuple, after those of the types its
/// fields name.
pub(crate) fn type_decls(
    ty: &Ty,
    types: &Types,
    compared: &BTreeSet<String>,
    emitted: &mut BTreeSet<Ty>,
    decls: &mut Vec<Decl>,
) {
    if !emitted.insert(ty.clone()) {
        return;
    }
    if let Ty::Named(n) = ty {
        if let Some(family) = types.family_of(n) {
            if family.len() > 1 || types.boxed(n) || types.nested(n) {
                family_decls(&family.clone(), types, compared, emitted, decls);
                return;
            }
        }
        if let Some(variants) = types.datas.get(n) {
            for (_, fields) in variants {
                for (_, t) in fields {
                    let inner = t.mentions();
                    for dep in &inner {
                        type_decls(dep, types, compared, emitted, decls);
                    }
                }
            }
            decls.push(Decl::Data {
                name: n.clone(),
                constructors: variants
                    .iter()
                    .map(|(v, fields)| {
                        (v.clone(), fields.iter().map(|(_, t)| t.render()).collect())
                    })
                    .collect(),
                mutual: None,
            });
            return;
        }
    }
    let fields: Vec<(String, Ty)> = match ty {
        Ty::Named(n) => match (types.enums.get(n), types.structs.get(n)) {
            (Some(variants), _) => {
                decls.push(Decl::Inductive {
                    name: n.clone(),
                    constructors: variants.clone(),
                });
                if compared.contains(n) {
                    decls.push(enum_equality(n, variants));
                }
                return;
            }
            (None, Some(fields)) => fields.clone(),
            (None, None) => return,
        },
        Ty::Tuple(ts) => ts
            .iter()
            .enumerate()
            .map(|(i, t)| (format!("item{i}"), t.clone()))
            .collect(),
        _ => return,
    };
    for (_, t) in &fields {
        let inner = t.mentions();
        for dep in &inner {
            type_decls(dep, types, compared, emitted, decls);
        }
    }
    // Widths and characters render alike: one structure serves them all.
    if decls.iter().any(|d| d.name() == ty.render()) {
        return;
    }
    decls.push(Decl::Structure {
        name: ty.render(),
        fields: fields
            .iter()
            .map(|(f, t)| (f.clone(), t.render()))
            .collect(),
    });
}

fn supply(ctx: &Model) -> Module {
    let mut decls = vec![Decl::Structure {
        name: "LockedPackage".into(),
        fields: vec![
            ("name".into(), "String".into()),
            ("version".into(), "String".into()),
            ("source".into(), "String".into()),
            ("checksum".into(), "String".into()),
        ],
    }];
    let mut seen = BTreeSet::new();
    for l in &ctx.ws.lock {
        let name = format!(
            "package{}",
            pascal(&[&l.name, &l.version.replace('.', "_")])
        );
        if !seen.insert(name.clone()) {
            continue;
        }
        decls.push(Decl::value(
            name,
            "LockedPackage",
            record(
                "LockedPackage",
                &[
                    ("name", string(&l.name)),
                    ("version", string(&l.version)),
                    ("source", string(&l.source)),
                    ("checksum", string(&l.checksum)),
                ],
            ),
        ));
    }
    Module {
        name: SUPPLY.into(),
        glossaries: vec![BOOL, NAT, TITLES],
        imports: Vec::new(),
        title: "Locked supply chain",
        decls,
    }
}

fn register(rows: &[RegisterRow]) -> Vec<(String, Vec<u8>)> {
    let mut ids = String::from(
        "# model/ids.toml: one row per admitted feature, generated by uorust.\n\nspec = \"template/1\"\n",
    );
    let mut suites: BTreeMap<&str, Vec<&RegisterRow>> = BTreeMap::new();
    for row in rows {
        ids.push_str(&format!(
            "\n[[id]]\nid = \"{}\"\nlevel = \"build\"\nsuite = \"{}\"\nrecord = \"{}\"\nstatement = {}\n",
            row.id,
            row.suite,
            row.record,
            string(&row.statement)
        ));
        suites.entry(row.suite.as_str()).or_default().push(row);
    }
    let mut out = vec![("model/ids.toml".to_string(), ids.into_bytes())];
    for (suite, rows) in suites {
        let mut text = format!(
            "Feature: {suite}\n\n  The features uorust admitted for the {suite} package, each with the\n  interface calls that realize it and the acceptance rows that accept it.\n"
        );
        for row in rows {
            text.push_str(&format!(
                "\n  @{} @build\n  Scenario: {}\n    Given the record {}\n",
                row.id,
                crate::lex::escape(&row.statement),
                row.record
            ));
            for (i, call) in row.calls.iter().enumerate() {
                let word = if i == 0 { "When" } else { "And" };
                text.push_str(&format!("    {word} the interface call {call} is made\n"));
            }
            for (i, row) in row.acceptance.iter().enumerate() {
                let word = if i == 0 { "Then" } else { "And" };
                text.push_str(&format!("    {word} the acceptance row {row} holds\n"));
            }
        }
        out.push((
            format!("features/suites/{suite}.feature"),
            text.into_bytes(),
        ));
    }
    out
}

fn toml_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A free function of product code by where it is declared: package,
/// target, module and name.
type FnSite = (usize, usize, String, String);

/// Whether the trait a method implements is in scope where `caller` stands:
/// Rust reads `Type::name` and `value.name()` as a trait's method there
/// alone. Three rules of the language decide it, and whatever they leave
/// open counts as out of scope, so a call is refused before it is misread:
/// the scope that names a trait by one segment has it in scope (it declares
/// the trait, imports it, or takes it from the prelude); a name the impl's
/// scope neither declares nor imports, with no glob that could bind it, is
/// the standard prelude's, in scope everywhere in a crate that keeps its
/// prelude; and a file's own scope has a trait it declares or imports by a
/// path that leads to the trait's declaration (`declared_trait`).
fn trait_in_scope(ctx: &Model, caller: FnRef, imp: FnRef) -> bool {
    let method = ctx.func(imp);
    let Some(path) = &method.trait_path else {
        return false;
    };
    let single = !path.contains("::");
    let from = ctx.func(caller);
    if single && caller.pkg == imp.pkg && caller.file == imp.file && from.scope == method.scope {
        return true;
    }
    let package = &ctx.ws.packages[imp.pkg];
    let home = &ctx.file(imp).scopes[method.scope];
    let bound = home.traits.contains(path) || home.uses.iter().any(|(n, _)| n == path);
    if single && !bound && !home.glob {
        return caller.pkg == imp.pkg && !package.files.iter().any(|f| f.no_prelude);
    }
    if caller.pkg != imp.pkg || from.scope != 0 || method.scope != 0 {
        return false;
    }
    let Some(named) = declared_trait(ctx, imp.pkg, imp.file, path) else {
        return false;
    };
    let here = ctx.file(caller);
    let (file, name) = &named;
    (*file == caller.file && here.scopes[0].traits.contains(name))
        || here.scopes[0].uses.iter().any(|(_, used)| {
            declared_trait(ctx, caller.pkg, caller.file, used).as_ref() == Some(&named)
        })
}

/// The trait a path names from a file's own scope: the file of the crate
/// that declares it, and its name there. The path is read as Rust reads it,
/// through the scope's own `use` bindings, from a module the scope declares
/// and from `crate`, `self` and `super`, and through each module that only
/// re-exports the name. A path
/// into another crate, and one Rust would read from a name this does not
/// see, names no trait here.
fn declared_trait(ctx: &Model, pkg: usize, fi: usize, path: &str) -> Option<(usize, String)> {
    let files = &ctx.ws.packages[pkg].files;
    let mut at = fi;
    let mut path = path.to_string();
    let mut seen: BTreeSet<(usize, String)> = BTreeSet::new();
    // Each step moves to another module's file or reads another binding, and
    // no step is taken twice.
    while seen.insert((at, path.clone())) {
        let file = &files[at];
        let scope = file.scopes.first()?;
        let mut segs: Vec<&str> = path.split("::").collect();
        let first = *segs.first()?;
        if first.is_empty() {
            return None;
        }
        if segs.len() == 1 && scope.traits.iter().any(|t| t == first) {
            return Some((at, first.to_string()));
        }
        let own = scope.mods.iter().any(|m| m == first);
        if !own && !matches!(first, "crate" | "self" | "super") {
            // The scope's own binding of the first name, read in its place.
            let (_, bound) = scope
                .uses
                .iter()
                .find(|(n, used)| n == first && *used != path)?;
            let rest = segs[1..].join("::");
            path = match rest.is_empty() {
                true => bound.clone(),
                false => format!("{bound}::{rest}"),
            };
            continue;
        }
        let mut module: Vec<&str> = file.module.split("::").filter(|s| !s.is_empty()).collect();
        match first {
            "crate" => {
                module.clear();
                segs.remove(0);
            }
            "self" => {
                segs.remove(0);
            }
            _ => {}
        }
        while segs.first() == Some(&"super") {
            module.pop()?;
            segs.remove(0);
        }
        let name = segs.pop()?;
        module.extend(segs);
        let module = module.join("::");
        at = files
            .iter()
            .position(|f| f.target == file.target && f.module == module)?;
        path = name.to_string();
    }
    None
}

/// Why a default method a trait declares is not one function for every
/// implementing type, if it is not: it calls a method through `self` or
/// `Self`, which is each implementing type's (the trait's own body of it is
/// one an implementation may replace), or reads `self` or names `Self`
/// otherwise. `None` for a body that does neither: it means the same for
/// every implementing type, and lifts as written.
fn default_refusal(
    file: &SourceFile,
    f: &FnFact,
    declarer: &str,
    block: &crate::ast::Block,
) -> Option<Residue> {
    let mut called = Vec::new();
    calls_through_self(block, &mut called);
    if let Some(method) = called.first() {
        let declared = file
            .fns
            .iter()
            .find(|g| g.in_trait && g.owner.as_deref() == Some(declarer) && &g.name == method);
        let whose = match declared {
            Some(g) if g.tokens.ends_with('}') => format!(
                "{declarer}::{method}, whose body in the trait an implementing type may replace"
            ),
            Some(_) => format!("{declarer}::{method}, which the trait declares without a body"),
            None => format!("{method}, which {declarer} does not declare"),
        };
        return Some(Residue::Signature(format!(
            "a default method of {declarer} that calls {whose}: it means each implementing \
             type's"
        )));
    }
    let words = |word: &str| {
        f.tokens
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .filter(|w| *w == word)
            .count()
    };
    if words("Self") > 0 {
        return Some(Residue::Signature(format!(
            "a default method of {declarer} that names Self, each implementing type"
        )));
    }
    // The receiver is the one `self` a signature writes.
    if words("self") > usize::from(f.receiver.is_some()) {
        return Some(Residue::Signature(format!(
            "a default method of {declarer} that reads self, each implementing type's value"
        )));
    }
    None
}

/// The methods a body calls on `self` and the functions it calls as
/// `Self::name`, in source order.
fn calls_through_self(block: &crate::ast::Block, out: &mut Vec<String>) {
    use crate::ast::{RustExpr, Stmt, UnOp};
    fn is_self(e: &RustExpr) -> bool {
        match e {
            RustExpr::Path(p) => p.is_ident("self"),
            RustExpr::Paren(inner) => is_self(inner),
            RustExpr::Reference(r) => is_self(&r.expr),
            RustExpr::Unary(u) => matches!(u.op, UnOp::Deref) && is_self(&u.expr),
            _ => false,
        }
    }
    fn expr(e: &RustExpr, out: &mut Vec<String>) {
        match e {
            RustExpr::MethodCall(m) if is_self(&m.receiver) => out.push(m.method.clone()),
            RustExpr::Call(c) => {
                if let RustExpr::Path(p) = &*c.func {
                    if let [owner, name] = p.segments.as_slice() {
                        if owner.ident == "Self" {
                            out.push(name.ident.clone());
                        }
                    }
                }
            }
            RustExpr::Macro(m) => {
                for arg in m.args.iter().flat_map(|a| &a.items) {
                    expr(arg, out);
                }
            }
            _ => {}
        }
        let (exprs, blocks) = e.parts();
        for inner in exprs {
            expr(inner, out);
        }
        for inner in blocks {
            calls_through_self(inner, out);
        }
    }
    for stmt in &block.stmts {
        match stmt {
            Stmt::Local(l) => {
                if let Some(init) = &l.init {
                    expr(&init.expr, out);
                    if let Some(diverge) = &init.diverge {
                        expr(diverge, out);
                    }
                }
            }
            Stmt::Expr(e, _) => expr(e, out),
            Stmt::Macro(m) => {
                for arg in m.args.iter().flat_map(|a| &a.items) {
                    expr(arg, out);
                }
            }
            Stmt::Other => {}
        }
    }
}

/// Why the lift does not name the type a method is of, as the file `fi` of
/// package `pkg` writes it: the shape it is declared in, or the first of
/// its fields whose type is outside the lift and, where that is another
/// type of the crate, why that one is, down to a type the crate does not
/// declare. Each type is explained once, so a family of types that hold
/// each other ends.
fn unlifted_type(
    ctx: &Model,
    syntax: &Syntax,
    scopes: &BTreeMap<(usize, usize, String), Types>,
    expanded: &dyn Fn(usize, &crate::ast::RustType) -> crate::ast::RustType,
    pkg: usize,
    fi: usize,
    written: &str,
) -> String {
    let package = &ctx.ws.packages[pkg];
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out: Vec<String> = Vec::new();
    let mut next = Some((fi, written.to_string()));
    while let Some((from, ty)) = next.take() {
        // The name a declaration would carry: the path's last segment,
        // without its arguments.
        let head = ty.split('<').next().unwrap_or(&ty);
        let name = head.rsplit("::").next().unwrap_or(head).trim().to_string();
        let plain = name.chars().all(|c| c.is_alphanumeric() || c == '_') && !name.is_empty();
        if !plain || !seen.insert(name.clone()) {
            out.push(if plain {
                "which holds itself where no inductive type can".to_string()
            } else {
                "which is not a struct or an enum by name".to_string()
            });
            break;
        }
        if ty.contains('<') && out.is_empty() {
            out.push(
                "written with arguments, where a method's type is read by its name alone"
                    .to_string(),
            );
            break;
        }
        // Its declaration: the writing file's own, else the crate's first.
        let order = std::iter::once(from).chain((0..package.files.len()).filter(|i| *i != from));
        let mut found = None;
        for at in order {
            let file = &package.files[at];
            let read = &syntax.files[&file.rel];
            if let Some(shape) = file.unread.iter().find(|(n, _)| *n == name) {
                found = Some(Err(shape.1.to_string()));
            } else if let Some(i) = file.structs.iter().position(|st| st.name == name) {
                let fields = read.structs[i]
                    .iter()
                    .map(|(field, t)| (format!("field {field}"), t.clone()))
                    .collect::<Vec<_>>();
                found = Some(Ok((at, fields)));
            } else if let Some(i) = file.enums.iter().position(|e| e.name == name) {
                let fields = read.enums[i]
                    .iter()
                    .flat_map(|(variant, fields)| {
                        fields
                            .iter()
                            .map(move |(_, t)| (format!("variant {variant}"), t.clone()))
                    })
                    .collect::<Vec<_>>();
                found = Some(Ok((at, fields)));
            }
            if found.is_some() {
                break;
            }
        }
        match found {
            None => out.push("which this crate does not declare".to_string()),
            Some(Err(shape)) => out.push(format!("{shape}, which the lift does not read")),
            Some(Ok((at, fields))) => {
                let scope = &scopes[&(pkg, at, String::new())];
                let imports = &package.files[at].imports;
                let outside = fields.iter().find_map(|(part, t)| {
                    match scope.ty(&expanded(pkg, &t.localized(imports))) {
                        Err(Residue::Type(inner)) => Some((part, inner)),
                        _ => None,
                    }
                });
                match outside {
                    Some((part, inner)) => {
                        out.push(format!("whose {part} holds {inner}"));
                        next = Some((at, inner));
                    }
                    None => out.push(
                        "which the lift leaves out though each of its parts lifts: it has no \
                         part, it is declared twice, its name is one the projection keeps, or \
                         it holds itself where no inductive type can"
                            .to_string(),
                    ),
                }
            }
        }
    }
    out.join(", ")
}

/// The free functions of product code whose name two modules (or two
/// targets) of one crate share, `toml_string` in `project` and in
/// `certify`, each -> its `rust` key and the LexLean name it lifts under:
/// qualified by its module, and by its target where that is not the
/// library, so that no two of them can meet. A name only one module
/// declares keeps its own, and so its definition and every κ's verdict.
fn homonym_fns(ctx: &Model) -> BTreeMap<FnSite, (String, String)> {
    let mut sites: BTreeMap<(usize, String), BTreeSet<FnSite>> = BTreeMap::new();
    for r in ctx.all_fns() {
        let f = ctx.func(r);
        if f.owner.is_some() || !ctx.product(r) {
            continue;
        }
        let file = ctx.file(r);
        let site = (r.pkg, file.target, file.module.clone(), f.name.clone());
        sites
            .entry((r.pkg, f.name.clone()))
            .or_default()
            .insert(site);
    }
    let mut out = BTreeMap::new();
    for ((pkg, name), at) in sites {
        if at.len() < 2 {
            continue;
        }
        let package = &ctx.ws.packages[pkg];
        for site in at {
            let target = &package.targets[site.1];
            let mut parts: Vec<String> = Vec::new();
            if !matches!(target.kind, TargetKind::Lib) {
                parts.push(target.name.replace('-', "_"));
            }
            parts.extend(
                site.2
                    .split("::")
                    .filter(|m| !m.is_empty())
                    .map(str::to_string),
            );
            if parts.is_empty() {
                parts.push(package.name.replace('-', "_"));
            }
            parts.push(name.clone());
            let rust = format!("{}#{}", site.1, parts.join("::"));
            let lex = lift::local_name(&parts.join("_"));
            out.insert(site, (rust, lex));
        }
    }
    out
}

/// How a file reads each name its crate's free functions share: the
/// function its module declares, else the one its `use` names, else none.
fn homonym_reading(
    ctx: &Model,
    homonyms: &BTreeMap<FnSite, (String, String)>,
    pkg: usize,
    fi: usize,
) -> BTreeMap<String, Option<String>> {
    let file = &ctx.ws.packages[pkg].files[fi];
    let mut out = BTreeMap::new();
    for ((p, target, module, name), (rust, _)) in homonyms {
        if *p != pkg {
            continue;
        }
        let entry = out.entry(name.clone()).or_insert(None);
        if *target != file.target {
            continue;
        }
        if *module == file.module {
            *entry = Some(rust.clone());
            continue;
        }
        let Some(path) = file.imports.get(name) else {
            continue;
        };
        // `crate::`, `self::` and `super::` as Rust reads them, any other
        // path from the file's module.
        let mut at: Vec<&str> = file.module.split("::").filter(|s| !s.is_empty()).collect();
        let mut segs: Vec<&str> = path.split("::").collect();
        match segs.first() {
            Some(&"crate") => {
                at.clear();
                segs.remove(0);
            }
            Some(&"self") => {
                segs.remove(0);
            }
            _ => {}
        }
        while segs.first() == Some(&"super") {
            at.pop();
            segs.remove(0);
        }
        if let Some((last, prefix)) = segs.split_last() {
            at.extend(prefix);
            if *last == name.as_str() && at.join("::") == *module && entry.is_none() {
                *entry = Some(rust.clone());
            }
        }
    }
    out
}

/// The registry dependency a type the lift refused belongs to: `syn` for
/// `syn::Expr`, `&mut toml::Value` or `Option<serde_json::Value>`.
fn foreign_crate(package: &Package, ty: &str) -> Option<String> {
    let deps: BTreeSet<String> = package
        .deps
        .iter()
        .filter(|d| d.path.is_none())
        .map(|d| d.name.replace('-', "_"))
        .collect();
    ty.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':'))
        .filter_map(|word| word.split_once("::").map(|(head, _)| head.to_string()))
        .find(|head| deps.contains(head))
}

/// The bare type names a file writes in its declarations and signatures,
/// less every name a parameter of an item of the file takes: a type
/// parameter is bound by its item, not by its module.
fn bare_names(file: &crate::facts::FileSyntax, facts: &SourceFile) -> BTreeSet<String> {
    fn walk(t: &crate::ast::RustType, out: &mut BTreeSet<String>) {
        use crate::ast::{GenericArgument, PathArguments, TypeKind};
        match &t.kind {
            TypeKind::Reference { elem, .. } => walk(elem, out),
            TypeKind::Slice(t) | TypeKind::Array(t, _) | TypeKind::Paren(t) => walk(t, out),
            TypeKind::Fn(ins, ret) => {
                ins.iter().for_each(|t| walk(t, out));
                walk(ret, out);
            }
            TypeKind::Tuple(ts) => ts.iter().for_each(|t| walk(t, out)),
            TypeKind::Path { qself, path } => {
                if let ([one], false) = (path.segments.as_slice(), qself) {
                    out.insert(one.ident.clone());
                }
                for segment in &path.segments {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        for a in args {
                            if let GenericArgument::Type(t) = a {
                                walk(t, out);
                            }
                        }
                    }
                }
            }
            TypeKind::Other => {}
        }
    }
    let mut out = BTreeSet::new();
    let fields = file.structs.iter().flatten().map(|(_, t)| t);
    let variants = file
        .enums
        .iter()
        .flatten()
        .flat_map(|(_, fs)| fs.iter().map(|(_, t)| t));
    let aliases = file.aliases.iter().map(|(_, a)| &a.to);
    let signatures = file.fns.iter().flat_map(|f| {
        let sig = &f.signature;
        sig.inputs
            .iter()
            .filter_map(|i| match i {
                crate::ast::FnArg::Typed(p) => Some(&p.ty),
                crate::ast::FnArg::Receiver { .. } => None,
            })
            .chain(sig.output.iter())
    });
    for t in fields.chain(variants).chain(aliases).chain(signatures) {
        walk(t, &mut out);
    }
    for (_, alias) in &file.aliases {
        alias.params.iter().for_each(|p| {
            out.remove(p);
        });
    }
    for f in &file.fns {
        for p in &f.signature.generics {
            if let crate::ast::GenericParam::Type(n) | crate::ast::GenericParam::Const(n) = p {
                out.remove(n);
            }
        }
    }
    for f in &facts.fns {
        f.impl_params.iter().for_each(|p| {
            out.remove(p);
        });
    }
    out
}

/// The path a leaf's text is headed by, without its arguments: what the
/// lift read first and did not read. A borrow reads as what it borrows.
fn leaf_head(leaf: &str) -> &str {
    let mut head = leaf.trim_start();
    while let Some(rest) = head.strip_prefix('&') {
        let rest = rest.trim_start();
        head = rest.strip_prefix("mut ").unwrap_or(rest).trim_start();
    }
    let end = head
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':'))
        .unwrap_or(head.len());
    &head[..end]
}

/// The leaf of a type residue that is a chain through the fields of the
/// workspace's types (`Outer.inner: Inner::Some.0: big::BigX`): what follows
/// the last place, `Type.field` or `Type::Variant.field`, the chain passes
/// through. A residue that passes through no place has none.
fn chain_leaf(detail: &str) -> Option<&str> {
    let ident = |s: &str| {
        s.chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    };
    let place = |s: &str| {
        let Some((owner, field)) = s.split_once('.') else {
            return false;
        };
        let owner_ok = match owner.split_once("::") {
            Some((ty, variant)) => ident(ty) && ident(variant),
            None => ident(owner),
        };
        owner_ok
            && !field.is_empty()
            && field.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    };
    let mut rest = detail;
    let mut through = false;
    while let Some((head, tail)) = rest.split_once(": ") {
        if !place(head) {
            break;
        }
        through = true;
        rest = tail;
    }
    through.then_some(rest)
}

/// How an effect reads after "which": `performs a Fs effect: std::fs::read`.
fn effect_phrase(effect: &str) -> String {
    match effect.strip_prefix("calls ") {
        Some(_) => "reaches an effect".to_string(),
        None => format!("performs a {effect}"),
    }
}

/// The table the frontier's count of refusals is written under: the stage
/// that refused them.
const ADMISSION: &str = "admission";

/// The unit the frontier's refusal queue counts in.
const REFUSALS: &str = "refusals";

/// The projection's `frontier.toml`: the counts and the ranked queues, then
/// every refusal and every function row. `lift` is the register of `rows`,
/// the function rows ([`function_rows`]).
fn frontier(notes: &Notes, lift: &Register, rows: &str) -> String {
    let mut out = String::from(
        "# frontier.toml: what uorust did not admit or lift, each with a typed reason.\n\
         # A refusal is an open claim: the router proposed it and admission would\n\
         # not decide it. A residue row is a pure function outside the lift's floor.\n\
         # An effect row is the I/O shell: a function that performs an operating-system\n\
         # effect (file system, process, environment, network), or calls one that does;\n\
         # the security facet records each effect as a capability of its crate.\n\
         # A foreign row is a function whose parameter or result is a type of a registry\n\
         # dependency (syn, toml, serde_json, ...) that has no LexLean model.\n\
         # A lifted row names the definition a function lifted to and, where that\n\
         # definition parts from the Rust function, its divergence. Every function of\n\
         # product code has exactly one residue, foreign, effect or lifted row, keyed\n\
         # by the κ of its question (crate path and tokens): a lifted κ never turns\n\
         # unlifted in a later projection.\n\
         # The work queue comes first: [admission] counts the refusals and each\n\
         # [[reason]] row the refusals of one reason; [lift] counts the functions by\n\
         # table and each [[construct]] row the functions one cause and detail stops.\n\
         # Both queues are ranked, what stops the most first.\n\n\
         spec = \"uorust/frontier/1\"\n",
    );
    out.push_str(&refusal_queue(&notes.refusals));
    out.push_str(&lift.queue_text());
    for r in &notes.refusals {
        out.push_str(&format!(
            "\n[[refusal]]\nstatus = \"open\"\nreason = \"{}\"\nroute = \"{}\"\nspan = {}\ndetail = {}\n",
            r.code(),
            r.route(),
            toml_string(&r.span()),
            toml_string(&r.detail())
        ));
    }
    out.push_str(rows);
    out
}

/// The refusals as the frontier counts and ranks them: the `[admission]`
/// table, then one `[[reason]]` row per reason a refusal carries, with the
/// route that refused it and the refusals of that reason, in the order and
/// the row shape of the lift's queue ([`ranked`], [`queue_row`]).
fn refusal_queue(refusals: &BTreeSet<Refusal>) -> String {
    let mut reasons: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for r in refusals {
        *reasons.entry((r.code(), r.route())).or_default() += 1;
    }
    let mut out = format!("\n[{ADMISSION}]\n{REFUSALS} = {}\n", refusals.len());
    for ((reason, route), count) in ranked(reasons) {
        out.push('\n');
        out.push_str(&queue_row(
            "reason",
            &[("reason", reason), ("route", route)],
            REFUSALS,
            count,
        ));
    }
    out
}

/// The frontier's function rows: every function of product code under the
/// table of what the lift decided for it, keyed by its κ.
fn function_rows(notes: &Notes, kappas: &BTreeMap<String, String>) -> String {
    let kappa = |path: &str| {
        kappas
            .get(path)
            .map(|k| format!("kappa = \"{k}\"\n"))
            .unwrap_or_default()
    };
    let mut out = String::new();
    for (path, (span, reason)) in &notes.residue {
        out.push_str(&format!(
            "\n[[residue]]\nstatus = \"open\"\n{}function = {}\nspan = {}\nreason = \"{}\"\ndetail = {}\n",
            kappa(path),
            toml_string(path),
            toml_string(&span.render()),
            reason.code(),
            toml_string(&reason.detail())
        ));
    }
    for (path, (span, detail)) in &notes.foreign {
        out.push_str(&format!(
            "\n[[foreign]]\n{}function = {}\nspan = {}\ndetail = {}\n",
            kappa(path),
            toml_string(path),
            toml_string(&span.render()),
            toml_string(detail)
        ));
    }
    for (path, (span, effect)) in &notes.effects {
        out.push_str(&format!(
            "\n[[effect]]\n{}function = {}\nspan = {}\neffect = {}\n",
            kappa(path),
            toml_string(path),
            toml_string(&span.render()),
            toml_string(effect)
        ));
    }
    for (path, (span, definition, divergence)) in &notes.lifted {
        out.push_str(&format!(
            "\n[[lifted]]\n{}function = {}\nspan = {}\ndefinition = {}\n",
            kappa(path),
            toml_string(path),
            toml_string(&span.render()),
            toml_string(definition)
        ));
        if !divergence.is_empty() {
            out.push_str(&format!(
                "divergence = {}\n",
                toml_string(&divergence.join("; "))
            ));
        }
    }
    out
}

/// Every module of the projection within the `max_file_bytes` its own
/// `lexlean.toml` states, which LexLean refuses a source file beyond
/// (LLS8002): the limit is read back from the project file the same run
/// writes, so the check and the stated limit cannot part. A module beyond it
/// is [`UorustError::ModuleTooLarge`], naming the first such module.
fn within_limit(files: &BTreeMap<String, Vec<u8>>) -> Result<(), UorustError> {
    let at = std::path::Path::new(LEXLEAN);
    let bad = |reason: String| UorustError::Malformed {
        path: at.to_path_buf(),
        reason,
    };
    let text = String::from_utf8_lossy(files.get(LEXLEAN).map_or(&[][..], Vec::as_slice));
    let config: toml::Value = text.parse().map_err(|e| bad(format!("{e}")))?;
    let limit = config
        .get("limits")
        .and_then(|limits| limits.get("max_file_bytes"))
        .and_then(toml::Value::as_integer)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| bad("it states no [limits] max_file_bytes".to_string()))?;
    match files
        .iter()
        .find(|(path, bytes)| path.ends_with(MODULE_SUFFIX) && bytes.len() > limit)
    {
        Some((path, bytes)) => Err(UorustError::ModuleTooLarge {
            module: path.clone(),
            bytes: bytes.len(),
            limit,
        }),
        None => Ok(()),
    }
}

/// What the path of a LexLean module's source ends with.
const MODULE_SUFFIX: &str = ".lex.tex";

/// The modules as they are written, each `(path, text)`: a module within
/// `limit` bytes whole, and a module beyond it as its [`parts`] where its
/// records can be written apart and nothing but the module itself cites one
/// (no other module, and no other file, which `cited` answers for the text
/// `Module::`). A module beyond the limit that cannot be parted is written
/// whole, and [`within_limit`] refuses the projection by its name and size.
fn sized(modules: &[Module], limit: usize, cited: &dyn Fn(&str) -> bool) -> Vec<(String, String)> {
    let texts: Vec<String> = modules.iter().map(Module::render).collect();
    let taken: BTreeSet<&str> = modules.iter().map(|m| m.name.as_str()).collect();
    let mut out = Vec::new();
    for (i, (module, text)) in modules.iter().zip(&texts).enumerate() {
        if text.len() > limit {
            let id = format!("{}::", module.name);
            let elsewhere = cited(&id)
                || texts
                    .iter()
                    .enumerate()
                    .any(|(j, other)| j != i && other.contains(&id));
            let parted = (!elsewhere).then(|| parts(module, limit, &taken)).flatten();
            if let Some(parted) = parted {
                out.extend(parted.iter().map(|part| (part.path(), part.render())));
                continue;
            }
        }
        out.push((module.path(), text.clone()));
    }
    out
}

/// The parts a module is written as when it is beyond `limit` bytes: its
/// declarations in their order, each part the longest run of them that
/// stays within the limit. The first part keeps the module's name and the
/// `n`th is named `{Module}{n}`, so a record's ID (`Module::name`, which
/// PrismPM reads from the module that declares it) names its part, and every
/// reference to a record of the module is rewritten to the part it is in.
///
/// Only a module of separate records is parted: every declaration a closed
/// value in which no word outside a string is another declaration's name
/// (the term is not read further, so a field or a constructor spelled like
/// one counts as naming it), and none carrying a canonical `index`, which
/// PrismPM derives from the order of the IDs a parting changes. `None` where
/// the module is not one, where one record alone is beyond the limit, or
/// where a part's name is `taken`.
fn parts(module: &Module, limit: usize, taken: &BTreeSet<&str>) -> Option<Vec<Module>> {
    let names: BTreeSet<&str> = module.decls.iter().map(Decl::name).collect();
    let mut records = Vec::new();
    for decl in &module.decls {
        let Decl::Definition {
            name,
            parameters,
            result,
            body: Expr::Flat(body),
            axioms,
            recursive: None,
            termination: None,
            mutual: None,
        } = decl
        else {
            return None;
        };
        let indexed = body.contains("{index := ") || body.contains(", index := ");
        if !parameters.is_empty()
            || !axioms.is_empty()
            || indexed
            || words(body).any(|word| names.contains(word))
        {
            return None;
        }
        records.push((name, result, body));
    }
    let part = |decls: Vec<Decl>, name: String| Module {
        name,
        glossaries: module.glossaries.clone(),
        imports: module.imports.clone(),
        title: module.title,
        decls,
    };
    // The size of each record under the longest name a part can have: there
    // are no more parts than records, so no part's number is wider than
    // their count. A shorter name never prints a record larger, since a
    // line that fits stays flat.
    let widest = format!("{}{}", module.name, module.decls.len());
    let room = limit.checked_sub(part(Vec::new(), widest.clone()).render().len() + "\n".len())?;
    let between = "\n\n".len();
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut used = 0;
    for (i, (name, result, body)) in records.iter().enumerate() {
        let widened = renamed(body, &module.name, &|n| {
            names.contains(n).then_some(widest.as_str())
        });
        let size = crate::lex::render_decl(&Decl::value(*name, *result, widened)).len();
        match groups.last_mut() {
            Some(group) if used + between + size <= room => {
                group.push(i);
                used += between + size;
            }
            _ if size <= room => {
                groups.push(vec![i]);
                used = size;
            }
            _ => return None,
        }
    }
    let part_names: Vec<String> = (0..groups.len())
        .map(|k| match k {
            0 => module.name.clone(),
            _ => format!("{}{}", module.name, k + 1),
        })
        .collect();
    if part_names[1..].iter().any(|n| taken.contains(n.as_str())) {
        return None;
    }
    let part_of: BTreeMap<&str, &str> = groups
        .iter()
        .zip(&part_names)
        .flat_map(|(group, part)| {
            group
                .iter()
                .map(|&i| (records[i].0.as_str(), part.as_str()))
        })
        .collect();
    Some(
        groups
            .iter()
            .zip(&part_names)
            .map(|(group, name)| {
                let decls = group
                    .iter()
                    .map(|&i| {
                        let (record, result, body) = records[i];
                        let body = renamed(body, &module.name, &|n| part_of.get(n).copied());
                        Decl::value(record, result, body)
                    })
                    .collect();
                part(decls, name.clone())
            })
            .collect(),
    )
}

/// `text` with every string that is an ID of `module` (`"Module::name"`)
/// naming the module `part_of` gives for its `name`; an ID it gives none for
/// stays as it is.
fn renamed<'a>(text: &str, module: &str, part_of: &dyn Fn(&str) -> Option<&'a str>) -> String {
    let opening = format!("\"{module}::");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(&opening) {
        let after = &rest[at + opening.len()..];
        let name = &after[..after.find('"').unwrap_or(after.len())];
        out.push_str(&rest[..at]);
        match part_of(name) {
            Some(part) => out.push_str(&format!("\"{part}::")),
            None => out.push_str(&opening),
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

/// The words of a term outside its string literals: each run of letters,
/// digits and underscores, the shape of a declaration's name.
fn words(text: &str) -> impl Iterator<Item = &str> {
    let (mut quoted, mut escaped) = (false, false);
    text.split(move |c: char| {
        let inside = quoted;
        match c {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            _ => {}
        }
        inside || !(c.is_alphanumeric() || c == '_')
    })
    .filter(|word| !word.is_empty())
}

fn spdx(ws: &Workspace, options: &Options) -> String {
    let id = |name: &str, version: &str| {
        let raw = format!("SPDXRef-Package-{name}-{version}");
        raw.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                    c
                } else {
                    '-'
                }
            })
            .collect::<String>()
    };
    let internal: BTreeMap<&str, &Package> =
        ws.packages.iter().map(|p| (p.name.as_str(), p)).collect();
    let mut packages = Vec::new();
    for l in &ws.lock {
        let license = internal
            .get(l.name.as_str())
            .map(|p| p.license.clone())
            .filter(|l| !l.is_empty())
            .unwrap_or_else(|| "NOASSERTION".to_string());
        let mut package = serde_json::json!({
            "SPDXID": id(&l.name, &l.version),
            "name": l.name,
            "versionInfo": l.version,
            "downloadLocation": if l.source.starts_with("registry+") {
                format!("https://crates.io/api/v1/crates/{}/{}/download", l.name, l.version)
            } else {
                "NOASSERTION".to_string()
            },
            "licenseConcluded": license,
            "filesAnalyzed": false,
        });
        if !l.checksum.is_empty() {
            package["checksums"] =
                serde_json::json!([{"algorithm": "SHA256", "checksumValue": l.checksum}]);
        }
        packages.push(package);
    }
    let lock_digest = sha(ws
        .lock
        .iter()
        .map(|l| format!("{} {} {}\n", l.name, l.version, l.checksum))
        .collect::<String>()
        .as_bytes());
    let doc = serde_json::json!({
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": options.name,
        "documentNamespace": format!("https://uor.foundation/spdx/{}/{}", options.name, lock_digest),
        "creationInfo": {"created": options.last_modified, "creators": ["Tool: uorust"]},
        "packages": packages,
    });
    let mut text = serde_json::to_string_pretty(&doc).unwrap_or_default();
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    use super::sha;

    /// NIST CAVP SHA-256 short byte-oriented known answers, the response
    /// file UOR-Foundation/compression locks at `standards/nist-sha256`.
    const SHORT: &str = include_str!("../tests/fixtures/nist-sha256/SHA256ShortMsg.rsp");

    fn vectors() -> Vec<(Vec<u8>, String)> {
        let mut out = Vec::new();
        let mut len = 0;
        let mut msg = Vec::new();
        for line in SHORT.lines() {
            if let Some(v) = line.strip_prefix("Len = ") {
                len = v.parse::<usize>().expect("a bit length") / 8;
            } else if let Some(v) = line.strip_prefix("Msg = ") {
                msg = (0..v.len() / 2)
                    .map(|i| u8::from_str_radix(&v[2 * i..2 * i + 2], 16).expect("hex"))
                    .take(len)
                    .collect();
            } else if let Some(v) = line.strip_prefix("MD = ") {
                out.push((std::mem::take(&mut msg), v.to_string()));
            }
        }
        out
    }

    #[test]
    fn sha_meets_every_nist_short_vector() {
        let vectors = vectors();
        assert_eq!(vectors.len(), 65);
        for (msg, md) in vectors {
            assert_eq!(sha(&msg), md);
        }
    }

    /// The vectors at the bounds of SHA-256 padding, which the lift proves of
    /// the prelude's digest: empty, one to three bytes, the longest message
    /// whose length fits its last block, the shortest that does not, a
    /// block less a byte and a whole block. The literals agree with the
    /// response file because both agree with `sha`.
    #[test]
    fn sha_meets_the_nist_vectors_at_the_padding_bounds() {
        assert_eq!(
            sha(&[]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha(&[0xd3]),
            "28969cdfa74a12c82f3bad960b0b000aca2ac329deea5c2328ebc6f2ba9802c1"
        );
        assert_eq!(
            sha(&[0x11, 0xaf]),
            "5ca7133fa735326081558ac312c620eeca9970d1e70a4b95533d956f072d1f98"
        );
        assert_eq!(
            sha(&[0xb4, 0x19, 0x0e]),
            "dff2e73091f6c05e528896c4c831b9448653dc2ff043528f6769437bc7b975c2"
        );
        assert_eq!(
            sha(&[
                0x3e, 0xbf, 0xb0, 0x6d, 0xb8, 0xc3, 0x8d, 0x5b, 0xa0, 0x37, 0xf1, 0x36, 0x3e, 0x11,
                0x85, 0x50, 0xaa, 0xd9, 0x46, 0x06, 0xe2, 0x68, 0x35, 0xa0, 0x1a, 0xf0, 0x50, 0x78,
                0x53, 0x3c, 0xc2, 0x5f, 0x2f, 0x39, 0x57, 0x3c, 0x04, 0xb6, 0x32, 0xf6, 0x2f, 0x68,
                0xc2, 0x94, 0xab, 0x31, 0xf2, 0xa3, 0xe2, 0xa1, 0xa0, 0xd8, 0xc2, 0xbe, 0x51
            ]),
            "6595a2ef537a69ba8583dfbf7f5bec0ab1f93ce4c8ee1916eff44a93af5749c4"
        );
        assert_eq!(
            sha(&[
                0x2d, 0x52, 0x44, 0x7d, 0x12, 0x44, 0xd2, 0xeb, 0xc2, 0x86, 0x50, 0xe7, 0xb0, 0x56,
                0x54, 0xba, 0xd3, 0x5b, 0x3a, 0x68, 0xee, 0xdc, 0x7f, 0x85, 0x15, 0x30, 0x6b, 0x49,
                0x6d, 0x75, 0xf3, 0xe7, 0x33, 0x85, 0xdd, 0x1b, 0x00, 0x26, 0x25, 0x02, 0x4b, 0x81,
                0xa0, 0x2f, 0x2f, 0xd6, 0xdf, 0xfb, 0x6e, 0x6d, 0x56, 0x1c, 0xb7, 0xd0, 0xbd, 0x7a
            ]),
            "cfb88d6faf2de3a69d36195acec2e255e2af2b7d933997f348e09f6ce5758360"
        );
        assert_eq!(
            sha(&[
                0xe2, 0xf7, 0x6e, 0x97, 0x60, 0x6a, 0x87, 0x2e, 0x31, 0x74, 0x39, 0xf1, 0xa0, 0x3f,
                0xcd, 0x92, 0xe6, 0x32, 0xe5, 0xbd, 0x4e, 0x7c, 0xbc, 0x4e, 0x97, 0xf1, 0xaf, 0xc1,
                0x9a, 0x16, 0xfd, 0xe9, 0x2d, 0x77, 0xcb, 0xe5, 0x46, 0x41, 0x6b, 0x51, 0x64, 0x0c,
                0xdd, 0xb9, 0x2a, 0xf9, 0x96, 0x53, 0x4d, 0xfd, 0x81, 0xed, 0xb1, 0x7c, 0x44, 0x24,
                0xcf, 0x1a, 0xc4, 0xd7, 0x5a, 0xce, 0xeb
            ]),
            "18041bd4665083001fba8c5411d2d748e8abbfdcdfd9218cb02b68a78e7d4c23"
        );
        assert_eq!(
            sha(&[
                0x5a, 0x86, 0xb7, 0x37, 0xea, 0xea, 0x8e, 0xe9, 0x76, 0xa0, 0xa2, 0x4d, 0xa6, 0x3e,
                0x7e, 0xd7, 0xee, 0xfa, 0xd1, 0x8a, 0x10, 0x1c, 0x12, 0x11, 0xe2, 0xb3, 0x65, 0x0c,
                0x51, 0x87, 0xc2, 0xa8, 0xa6, 0x50, 0x54, 0x72, 0x08, 0x25, 0x1f, 0x6d, 0x42, 0x37,
                0xe6, 0x61, 0xc7, 0xbf, 0x4c, 0x77, 0xf3, 0x35, 0x39, 0x03, 0x94, 0xc3, 0x7f, 0xa1,
                0xa9, 0xf9, 0xbe, 0x83, 0x6a, 0xc2, 0x85, 0x09
            ]),
            "42e61e174fbb3897d6dd6cef3dd2802fe67b331953b06114a65c772859dfc1aa"
        );
        for (msg, md) in vectors() {
            if [0, 1, 2, 3, 55, 56, 63, 64].contains(&msg.len()) {
                assert_eq!(sha(&msg), md);
            }
        }
    }

    // Types outside the lift.

    /// The frontier of a one-crate workspace of these files, written under
    /// the system's temporary directory and projected.
    fn frontier_of(name: &str, files: &[(&str, &str)]) -> String {
        let dir = std::env::temp_dir().join(format!("uorust-lib-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let manifest =
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n");
        for (path, body) in [("Cargo.toml", manifest.as_str())].iter().chain(files) {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
            std::fs::write(path, body).expect("a file");
        }
        let options = super::Options {
            name: name.to_string(),
            last_modified: "2026-11-11T00:00:00Z".to_string(),
            advisories: None,
            mir: None,
        };
        let projected = crate::project_workspace(&dir, &options).expect("a projection");
        let _ = std::fs::remove_dir_all(&dir);
        String::from_utf8(projected["frontier.toml"].clone()).expect("text")
    }

    /// The rows of a frontier that are of this function: each one's table
    /// and the line of it that starts with this key.
    fn rows_of(frontier: &str, function: &str, key: &str) -> Vec<(String, String)> {
        frontier
            .split("\n[[")
            .filter(|row| row.contains(&format!("\nfunction = \"{function}\"\n")))
            .map(|row| {
                let table = row.split("]]").next().unwrap_or_default().to_string();
                let line = row
                    .lines()
                    .find_map(|l| l.strip_prefix(&format!("{key} = ")))
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_string();
                (table, line)
            })
            .collect()
    }

    const NAMES: &[(&str, &str)] = &[
        (
            "src/lib.rs",
            "//! Names that read as one type, as none, or as a leaf.\n\
             pub mod first;\npub mod reader;\npub mod second;\npub mod teller;\n\n\
             /// One type for each build configuration.\n#[cfg(unix)]\n\
             pub struct Handle {\n    /// A descriptor.\n    pub fd: u32,\n}\n\n\
             /// One type for each build configuration.\n#[cfg(not(unix))]\n\
             pub struct Handle {\n    /// A raw handle.\n    pub raw: u64,\n}\n\n\
             /// Stopped by a name two declarations share.\n\
             pub fn opened(h: &Handle) -> bool {\n    let _ = h;\n    true\n}\n\n\
             /// A float.\npub struct Leaf {\n    /// The float.\n    pub x: f64,\n}\n\n\
             /// Holds the leaf.\npub struct Mid {\n    /// A count.\n    pub n: u32,\n    \
             /// The leaf.\n    pub leaf: Leaf,\n}\n\n\
             impl Mid {\n    /// Stopped by the type it is a method of.\n    \
             pub fn count(&self) -> u32 {\n        self.n\n    }\n}\n\n\
             /// A type named as LexLean names one of its own.\n\
             pub struct Prop {\n    /// A power.\n    pub power: u8,\n}\n\n\
             /// Reads it.\npub fn power(p: &Prop) -> u8 {\n    p.power\n}\n\n\
             /// Two traits give one type a method of one name.\n\
             pub struct Label {\n    /// The text.\n    pub text: String,\n}\n\n\
             impl std::fmt::Display for Label {\n    \
             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        \
             write!(f, \"{}\", self.text)\n    }\n}\n\n\
             impl std::fmt::Debug for Label {\n    \
             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        \
             f.write_str(\"label\")\n    }\n}\n\n\
             /// One function for each build configuration.\n#[cfg(unix)]\n\
             pub fn width() -> u32 {\n    1\n}\n\n\
             /// One function for each build configuration.\n#[cfg(not(unix))]\n\
             pub fn width(scale: f64) -> u32 {\n    let _ = scale;\n    2\n}\n",
        ),
        (
            "src/first.rs",
            "//! The first of two modules that declare a type of one name.\n\n\
             /// A shape with sides.\npub struct Shape {\n    /// The sides.\n    pub sides: u32,\n}\n\n\
             /// Reads the shape its own module declares.\n\
             pub fn sides(s: &Shape) -> u32 {\n    s.sides\n}\n",
        ),
        (
            "src/second.rs",
            "//! The second of two modules that declare a type of one name.\n\n\
             /// A shape with a radius.\npub struct Shape {\n    /// The radius.\n    pub radius: u32,\n}\n\n\
             /// Reads the shape its own module declares.\n\
             pub fn radius(s: &Shape) -> u32 {\n    s.radius\n}\n",
        ),
        (
            "src/teller.rs",
            "//! A file that says which of the two shapes it means.\n\n\
             use crate::second::Shape;\n\n\
             /// Reads the shape its import names.\n\
             pub fn round(s: &Shape) -> u32 {\n    s.radius\n}\n",
        ),
        (
            "src/reader.rs",
            "//! A file that imports one of the two shapes by a glob.\n\n\
             use crate::first::*;\n\n\
             /// Reads the shape its glob imports.\n\
             pub fn corners(s: &Shape) -> u32 {\n    s.sides\n}\n",
        ),
    ];

    #[test]
    fn a_name_that_reads_as_no_one_type_states_why() {
        let frontier = frontier_of("names", NAMES);
        let detail = |function: &str| rows_of(&frontier, function, "detail");
        let residue = |why: String| vec![("residue".to_string(), why)];
        // One name, two declarations in one module: neither is the type.
        assert_eq!(
            detail("names::opened"),
            residue(format!(
                "type outside the lift: Handle: {}",
                crate::lift::DECLARED_TWICE
            ))
        );
        // One name, two modules: the module that declares one, the file
        // that imports one and the file whose glob imports one read it.
        for told in [
            "names::first::sides",
            "names::second::radius",
            "names::teller::round",
            "names::reader::corners",
        ] {
            let rows = rows_of(&frontier, told, "definition");
            assert_eq!(rows.len(), 1, "{told}: {rows:?}");
            assert_eq!(rows[0].0, "lifted", "{told}");
        }
    }

    #[test]
    fn a_method_of_a_type_outside_the_lift_names_the_leaf() {
        let frontier = frontier_of("owners", NAMES);
        assert_eq!(
            rows_of(&frontier, "owners::Mid::count", "detail"),
            vec![(
                "residue".to_string(),
                "type outside the lift: Mid.leaf: Leaf.x: f64".to_string()
            )]
        );
    }

    #[test]
    fn a_type_named_as_lexlean_names_its_own_lifts_under_its_crate() {
        let frontier = frontier_of("taken", NAMES);
        let rows = rows_of(&frontier, "taken::power", "definition");
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert_eq!(rows[0].0, "lifted");
    }

    #[test]
    fn two_functions_of_one_path_share_one_row() {
        let frontier = frontier_of("paths", NAMES);
        let rows = rows_of(&frontier, "paths::width", "detail");
        assert_eq!(rows.len(), 1, "{rows:?}");
        // The row is the one that claims least: the function not lifted.
        assert_eq!(rows[0].0, "residue");
        // Two traits' methods of one name on one type are two paths, each
        // with its own row.
        for path in [
            "paths::<Label as Display>::fmt",
            "paths::<Label as Debug>::fmt",
        ] {
            let rows = rows_of(&frontier, path, "function");
            assert_eq!(rows.len(), 1, "{path}: {rows:?}");
        }
    }

    // Functions refused by their signature or misread by where they stand:
    // a trait implementation's method, a default method, a method of a type
    // outside the lift, and a module declared out of line under a test cfg.
    // Each projects a crate written for the test.

    /// The frontier and the behaviour module of the crate `name` of `files`.
    fn projected(name: &str, files: &[(&str, &str)]) -> (String, String) {
        let dir = std::env::temp_dir().join(format!("uorust-stand-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let manifest =
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n");
        let manifest = ("Cargo.toml", manifest.as_str());
        for (path, body) in std::iter::once(&manifest).chain(files) {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
            std::fs::write(path, body).expect("write");
        }
        let options = crate::Options {
            name: name.to_string(),
            last_modified: "2026-11-11T00:00:00Z".to_string(),
            advisories: None,
            mir: None,
        };
        let out = crate::project_workspace(&dir, &options).expect("projects");
        let _ = std::fs::remove_dir_all(&dir);
        // A crate none of whose functions lifts has no behaviour module.
        let text = |path: &str| {
            String::from_utf8(out.get(path).cloned().unwrap_or_default()).expect("utf-8")
        };
        (text("frontier.toml"), text("src/Behavior.lex.tex"))
    }

    /// The register row of a function: its kind and its lines.
    fn row(frontier: &str, function: &str) -> String {
        let needle = format!("function = \"{function}\"\n");
        let rows: Vec<&str> = frontier
            .split("\n\n")
            .filter(|row| row.contains(&needle))
            .collect();
        assert_eq!(rows.len(), 1, "{function} has one row:\n{frontier}");
        rows[0].to_string()
    }

    /// One definition of the behaviour module, flattened to a line.
    fn definition(behavior: &str, name: &str) -> String {
        let found = behavior
            .split("definition{")
            .find(|d| d.trim_start().starts_with(&format!("name: {name},")))
            .unwrap_or_else(|| panic!("{name} is defined:\n{behavior}"));
        found.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn a_trait_implementations_method_lifts_as_a_function_of_its_type_ue_15() {
        let (frontier, behavior) = projected(
            "shapes",
            &[(
                "src/lib.rs",
                "//! Shapes.\n\
                 /// Something with an area.\n\
                 pub trait Shape {\n    /// The area.\n    fn area(&self) -> u32;\n}\n\
                 /// Something with a label.\n\
                 pub trait Label {\n    /// An area.\n    fn area(&self) -> u32;\n    \
                 /// The label.\n    fn label(&self) -> String;\n}\n\
                 /// A square.\npub struct Square {\n    /// Its side.\n    pub side: u32,\n}\n\
                 /// A rectangle.\npub struct Rect {\n    /// Its width.\n    pub w: u32,\n    \
                 /// Its height.\n    pub h: u32,\n}\n\
                 impl Shape for Square {\n    fn area(&self) -> u32 {\n        \
                 self.side.saturating_mul(self.side)\n    }\n}\n\
                 impl Shape for Rect {\n    fn area(&self) -> u32 {\n        \
                 self.w.saturating_mul(self.h)\n    }\n}\n\
                 impl Label for Rect {\n    fn area(&self) -> u32 {\n        0\n    }\n    \
                 fn label(&self) -> String {\n        \"rect\".to_string()\n    }\n}\n\
                 impl Default for Square {\n    fn default() -> Self {\n        \
                 Square { side: 1 }\n    }\n}\n\
                 impl From<u32> for Rect {\n    fn from(side: u32) -> Self {\n        \
                 Rect { w: side, h: side }\n    }\n}\n\
                 impl From<bool> for Rect {\n    fn from(whole: bool) -> Self {\n        \
                 if whole {\n            Rect { w: 1, h: 1 }\n        } else {\n            \
                 Rect { w: 0, h: 0 }\n        }\n    }\n}\n\
                 impl std::fmt::Display for Rect {\n    \
                 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        \
                 write!(f, \"rect\")\n    }\n}\n\
                 /// The area of a square.\npub fn square_area(s: &Square) -> u32 {\n    \
                 s.area()\n}\n\
                 /// The unit square.\npub fn unit() -> Square {\n    Square::default()\n}\n\
                 /// A rectangle's label.\npub fn label_of(r: &Rect) -> String {\n    \
                 r.label()\n}\n\
                 /// A rectangle of a side.\npub fn of_side(side: u32) -> Rect {\n    \
                 Rect::from(side)\n}\n",
            )],
        );
        // A function of the implementing type, with `self` first; one name
        // for each type of a trait and each trait of a type.
        for (name, needle) in [
            (
                "squareShapeArea",
                "parameters: [selfValue : Square], result: Nat",
            ),
            (
                "rectShapeArea",
                "parameters: [selfValue : Rect], result: Nat",
            ),
            (
                "rectLabelArea",
                "parameters: [selfValue : Rect], result: Nat, body: 0",
            ),
            (
                "rectLabelLabel",
                "parameters: [selfValue : Rect], result: String",
            ),
            ("squareDefaultDefault", "parameters: [], result: Square"),
            ("rectFromU32From", "parameters: [side : Nat], result: Rect"),
            (
                "rectFromBoolFrom",
                "parameters: [whole : Bool], result: Rect",
            ),
            // The text a value displays as keeps the name it had.
            ("rectFmt", "parameters: [selfValue : Rect], result: String"),
            // A call written on the value or the type means the type's one
            // method of the name.
            ("squareArea", "body: (squareShapeArea s)"),
            ("unit", "body: (squareDefaultDefault)"),
            ("labelOf", "body: (rectLabelLabel r)"),
        ] {
            let found = definition(&behavior, name);
            assert!(found.contains(needle), "{name}: {found}");
        }
        // Rust's own name for each: `Type::name` where the type has one
        // method of the name, `<Type as Trait>::name` where it has several.
        for (function, lifted) in [
            ("shapes::Square::area", "Behavior::squareShapeArea"),
            ("shapes::<Rect as Shape>::area", "Behavior::rectShapeArea"),
            ("shapes::<Rect as Label>::area", "Behavior::rectLabelArea"),
            ("shapes::Rect::label", "Behavior::rectLabelLabel"),
            ("shapes::Square::default", "Behavior::squareDefaultDefault"),
            (
                "shapes::<Rect as From<u32>>::from",
                "Behavior::rectFromU32From",
            ),
            (
                "shapes::<Rect as From<bool>>::from",
                "Behavior::rectFromBoolFrom",
            ),
            ("shapes::Rect::fmt", "Behavior::rectFmt"),
        ] {
            let found = row(&frontier, function);
            assert!(
                found.starts_with("[[lifted]]")
                    && found.contains(&format!("definition = \"{lifted}\"")),
                "{found}"
            );
        }
        // A generic trait is implemented once per argument list, and the
        // types at the call pick among them: no call is read as one.
        let found = row(&frontier, "shapes::of_side");
        assert!(found.contains("reason = \"call\""), "{found}");
        // A method the trait declares without a body is no one function.
        let found = row(&frontier, "shapes::Shape::area");
        assert!(
            found.contains(
                "detail = \"signature outside the lift: a method Shape declares without a \
                 body, which each implementing type gives\""
            ),
            "{found}"
        );
    }

    #[test]
    fn a_call_means_a_traits_method_only_where_the_trait_is_in_scope() {
        let (frontier, behavior) = projected(
            "scopes",
            &[
                (
                    "src/lib.rs",
                    "//! Scopes.\npub mod far;\npub mod near;\npub mod shapes;\n\
                     pub use shapes::Label;\n",
                ),
                (
                    "src/shapes.rs",
                    "//! A square and its traits.\n\
                     /// A square.\n#[derive(Clone)]\npub struct Square {\n    \
                     /// Its side.\n    pub side: u32,\n}\n\
                     /// Something that gives a fresh square.\n\
                     pub trait Fresh {\n    /// A fresh square.\n    \
                     fn clone(&self) -> Square;\n}\n\
                     impl Fresh for Square {\n    fn clone(&self) -> Square {\n        \
                     Square { side: 0 }\n    }\n}\n\
                     /// Something with a label.\n\
                     pub trait Label {\n    /// The label.\n    fn label(&self) -> String;\n}\n\
                     impl Label for Square {\n    fn label(&self) -> String {\n        \
                     \"square\".to_string()\n    }\n}\n\
                     /// The label, where the trait is declared.\n\
                     pub fn here(s: &Square) -> String {\n    s.label()\n}\n",
                ),
                (
                    "src/far.rs",
                    "//! Where `Fresh` is not in scope.\nuse crate::shapes::Square;\n\
                     /// The square again, as `Clone` gives it.\n\
                     pub fn again(s: &Square) -> Square {\n    s.clone()\n}\n",
                ),
                (
                    "src/near.rs",
                    "//! Where the traits are imported.\n\
                     use crate::shapes::{Fresh, Square};\nuse crate::Label as _;\n\
                     /// A fresh square, named by its trait.\n\
                     pub fn fresh(s: &Square) -> Square {\n    Fresh::clone(s)\n}\n\
                     /// The label, through a trait the crate root re-exports.\n\
                     pub fn label_of(s: &Square) -> String {\n    s.label()\n}\n\
                     /// A module with a scope of its own.\n\
                     pub mod inner {\n    use crate::shapes::Square;\n    \
                     /// The square again: this scope does not import `Fresh`.\n    \
                     pub fn inner_again(s: &Square) -> Square {\n        s.clone()\n    }\n}\n",
                ),
            ],
        );
        // The trait's method lifts wherever it is called from.
        let found = row(&frontier, "scopes::shapes::Square::clone");
        assert!(
            found.contains("definition = \"Behavior::squareFreshClone\""),
            "{found}"
        );
        for (name, needle) in [
            // Out of the trait's scope `.clone()` is `Clone`'s: the value.
            ("again", "body: s}"),
            ("innerAgain", "body: s}"),
            // In its scope a call is the trait's method: where the trait is
            // declared, and where a `use` leads to its declaration.
            ("here", "body: (squareLabelLabel s)"),
            ("labelOf", "body: (squareLabelLabel s)"),
        ] {
            let found = definition(&behavior, name);
            assert!(found.contains(needle), "{name}: {found}");
        }
    }

    #[test]
    fn a_traits_method_gives_way_to_the_function_that_holds_its_name() {
        let (frontier, _) = projected(
            "yields",
            &[(
                "src/lib.rs",
                "//! Names.\n\
                 /// Something with an area.\n\
                 pub trait Shape {\n    /// The area.\n    fn area(&self) -> u32;\n}\n\
                 /// A square.\npub struct Square {\n    /// Its side.\n    pub side: u32,\n}\n\
                 /// A rectangle.\npub struct Rect {\n    /// Its width.\n    pub w: u32,\n}\n\
                 /// Spelled as the method of `Rect` below lifts.\n\
                 pub fn rect_shape_area(n: u32) -> u32 {\n    n\n}\n\
                 impl Shape for Rect {\n    fn area(&self) -> u32 {\n        self.w\n    }\n}\n\
                 impl Shape for Square {\n    fn area(&self) -> u32 {\n        self.side\n    }\n}\n\
                 /// Spelled as the method of `Square` above lifts.\n\
                 pub fn square_shape_area(n: u32) -> u32 {\n    n\n}\n",
            )],
        );
        // Before the method or after it, the function keeps the name.
        for (function, method) in [
            ("yields::rect_shape_area", "yields::Rect::area"),
            ("yields::square_shape_area", "yields::Square::area"),
        ] {
            let kept = row(&frontier, function);
            assert!(kept.starts_with("[[lifted]]"), "{kept}");
            let gave = row(&frontier, method);
            assert!(
                gave.starts_with("[[residue]]") && gave.contains("reason = \"collision\""),
                "{gave}"
            );
        }
    }

    #[test]
    fn a_default_method_lifts_where_its_body_is_one_for_every_implementing_type() {
        let (frontier, behavior) = projected(
            "defaults",
            &[(
                "src/lib.rs",
                "//! Defaults.\n\
                 /// Something with an area.\n\
                 pub trait Shape {\n    /// The area.\n    fn area(&self) -> u32;\n    \
                 /// The corners of a shape that does not say.\n    \
                 fn corners(&self) -> u32 {\n        4\n    }\n    \
                 /// The area doubled.\n    \
                 fn doubled(&self) -> u32 {\n        self.area().saturating_mul(2)\n    }\n    \
                 /// The corners doubled.\n    \
                 fn corners_doubled(&self) -> u32 {\n        \
                 self.corners().saturating_mul(2)\n    }\n    \
                 /// The shape itself.\n    fn same(&self) -> &Self {\n        self\n    }\n    \
                 /// A hook that does nothing.\n    fn visit(&mut self, _n: u32) {}\n}\n\
                 /// Spelled like a default method.\n\
                 pub fn corners() -> u32 {\n    3\n}\n\
                 /// A trait whose default method calls a function of that spelling.\n\
                 pub trait Count {\n    /// The count.\n    \
                 fn count(&self) -> u32 {\n        corners()\n    }\n}\n",
            )],
        );
        // One function for every implementing type takes no value of one.
        for (name, needle) in [
            ("shapeCorners", "parameters: [], result: Nat, body: 4}"),
            ("shapeVisit", "parameters: [n : Nat], result: Unit"),
            // A free function of a method's spelling is the free function.
            (
                "countCount",
                "parameters: [], result: Nat, body: (corners)}",
            ),
        ] {
            let found = definition(&behavior, name);
            assert!(found.contains(needle), "{name}: {found}");
        }
        // What it uses of an implementing type keeps it out, by name.
        for (function, detail) in [
            (
                "defaults::Shape::doubled",
                "a default method of Shape that calls Shape::area, which the trait declares \
                 without a body: it means each implementing type's",
            ),
            (
                "defaults::Shape::corners_doubled",
                "a default method of Shape that calls Shape::corners, whose body in the trait \
                 an implementing type may replace: it means each implementing type's",
            ),
            (
                "defaults::Shape::same",
                "a default method of Shape that names Self, each implementing type",
            ),
        ] {
            let found = row(&frontier, function);
            assert!(
                found.contains("reason = \"signature\"")
                    && found.contains(&format!(
                        "detail = \"signature outside the lift: {detail}\""
                    )),
                "{found}"
            );
        }
    }

    #[test]
    fn a_method_outside_the_lift_names_its_type_and_why() {
        let (frontier, _) = projected(
            "reasons",
            &[(
                "src/lib.rs",
                "//! Reasons.\n\
                 /// A circle.\npub struct Circle(pub u32);\n\
                 /// A blob.\npub struct Blob {\n    /// Its weight.\n    pub weight: f64,\n}\n\
                 /// A bag.\npub struct Bag {\n    /// Its blob.\n    pub blob: Blob,\n}\n\
                 /// A square.\npub struct Square {\n    /// Its side.\n    pub side: u32,\n}\n\
                 impl Circle {\n    /// The radius.\n    \
                 pub fn radius(&self) -> u32 {\n        self.0\n    }\n}\n\
                 impl Bag {\n    /// How many blobs.\n    \
                 pub fn count(&self) -> u32 {\n        1\n    }\n}\n\
                 impl Square {\n    /// Grow.\n    pub fn grow(&mut self, by: u32) {\n        \
                 self.side = self.side.saturating_add(by);\n    }\n}\n\
                 /// Something that scales by a factor of some type.\n\
                 pub trait Scale<T> {\n    /// The side scaled.\n    \
                 fn scale(&self, by: T) -> u32;\n}\n\
                 impl<T: Into<u32>> Scale<T> for Square {\n    \
                 fn scale(&self, by: T) -> u32 {\n        \
                 self.side.saturating_mul(by.into())\n    }\n}\n",
            )],
        );
        // A tuple struct's fields are read by position, so its method
        // lifts.
        let radius = row(&frontier, "reasons::Circle::radius");
        assert!(
            radius.starts_with("[[lifted]]")
                && radius.contains("definition = \"Behavior::circleRadius\""),
            "{radius}"
        );
        for (function, reason, detail) in [
            (
                "reasons::Bag::count",
                "type",
                "type outside the lift: Bag.blob: Blob.weight: f64",
            ),
            (
                "reasons::Square::grow",
                "signature",
                "signature outside the lift: takes &mut self of Square: the lift reads self \
                 and &self as the value, and threads no other receiver",
            ),
            (
                "reasons::Square::scale",
                "signature",
                "signature outside the lift: a method of an impl generic in T: one function \
                 for each type or constant that takes its place",
            ),
        ] {
            let found = row(&frontier, function);
            assert!(
                found.starts_with("[[residue]]")
                    && found.contains(&format!("reason = \"{reason}\""))
                    && found.contains(&format!("detail = \"{detail}\"")),
                "{found}"
            );
        }
    }

    #[test]
    fn a_module_declared_out_of_line_under_a_test_cfg_is_no_product_code() {
        let (frontier, _) = projected(
            "marks",
            &[
                (
                    "src/lib.rs",
                    "//! Marks.\n#[cfg(test)]\nmod tests;\n\
                     #[cfg(any(test, feature = \"x\"))]\nmod shared;\n\
                     /// Twice.\npub fn twice(n: u32) -> u32 {\n    n.saturating_mul(2)\n}\n",
                ),
                (
                    "src/tests.rs",
                    "mod helpers;\n\nfn four() -> u32 {\n    helpers::helper(super::twice(2))\n}\n",
                ),
                (
                    "src/tests/helpers.rs",
                    "pub fn helper(n: u32) -> u32 {\n    n\n}\n",
                ),
                (
                    "src/shared.rs",
                    "/// In a build with the feature too.\n\
                     pub fn shared_fn(n: u32) -> u32 {\n    n\n}\n",
                ),
            ],
        );
        // The crate's own function, and the module a build without tests
        // can hold, have their rows; the test tree has none.
        row(&frontier, "marks::twice");
        row(&frontier, "marks::shared::shared_fn");
        for helper in ["four", "helper"] {
            assert!(
                !frontier.contains(&format!("::{helper}\"")),
                "{helper} is test code:\n{frontier}"
            );
        }
    }

    #[test]
    fn two_traits_methods_of_one_name_keep_a_row_each() {
        let (frontier, _) = projected(
            "texts",
            &[(
                "src/lib.rs",
                "//! Texts.\n\
                 /// A point.\npub struct Point {\n    /// Its place.\n    pub x: u32,\n}\n\
                 impl std::fmt::Display for Point {\n    \
                 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        \
                 write!(f, \"point\")\n    }\n}\n\
                 impl std::fmt::Debug for Point {\n    \
                 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        \
                 write!(f, \"Point\")\n    }\n}\n",
            )],
        );
        let shown = row(&frontier, "texts::<Point as Display>::fmt");
        assert!(
            shown.contains("definition = \"Behavior::pointDisplayFmt\""),
            "{shown}"
        );
        let debugged = row(&frontier, "texts::<Point as Debug>::fmt");
        assert!(debugged.starts_with("[[residue]]"), "{debugged}");
        // One row per function and per κ: the frontier is a register.
        let read =
            crate::register::Register::parse(std::path::Path::new("frontier.toml"), &frontier);
        assert!(read.is_ok(), "{:?}", read.err().map(|e| e.to_string()));
    }

    // The register as the work queue: the frontier's ranked refusals and
    // lift queue, the lift as Quality measures, and the size of every
    // module against the limit the projection states.

    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    use super::{
        frontier, function_rows, refusal_queue, within_limit, Notes, Refusal, Register, Residue,
        Span, UorustError, FRONTIER, LEXLEAN, MAX_FILE_BYTES,
    };

    fn at(line: usize) -> Span {
        Span {
            file: "src/lib.rs".into(),
            line,
        }
    }

    /// A one-package workspace with no `rust-version`, written fresh: `over`
    /// lifts, and `next` and `after` stop at the same arithmetic.
    fn workspace(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("uorust-queue-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).expect("a scratch workspace");
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"queue\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
        )
        .expect("a manifest");
        std::fs::write(
            dir.join("src/lib.rs"),
            "//! Q.\n\n/// Over one.\npub fn over(x: u8) -> bool {\n    x > 1\n}\n\n\
             /// One more.\npub fn next(x: u8) -> u8 {\n    x + 1\n}\n\n\
             /// One more again.\npub fn after(x: u8) -> u8 {\n    x + 1\n}\n",
        )
        .expect("a library");
        dir
    }

    fn projected_files(name: &str) -> BTreeMap<String, Vec<u8>> {
        let dir = workspace(name);
        let options = super::Options {
            name: "queue".into(),
            last_modified: "2026-11-11T00:00:00Z".into(),
            advisories: None,
            mir: None,
        };
        let files = crate::project_workspace(&dir, &options).expect("projects");
        let _ = std::fs::remove_dir_all(dir);
        files
    }

    fn toml_of(files: &BTreeMap<String, Vec<u8>>, path: &str) -> toml::Value {
        String::from_utf8_lossy(&files[path])
            .parse()
            .expect("the file is TOML")
    }

    fn count(value: &toml::Value) -> usize {
        usize::try_from(value.as_integer().expect("a count")).expect("a count")
    }

    /// The `(requirementIndex, threshold)` of the Quality measure `name`.
    fn measure(quality: &str, name: &str) -> (usize, usize) {
        let from = quality
            .find(&format!("name: measure{name},"))
            .unwrap_or_else(|| panic!("no measure{name} in:\n{quality}"));
        let record = &quality[from..from + quality[from..].find("\n}").expect("a record")];
        let field = |key: &str| {
            let rest = &record[record.find(&format!("{key} := ")).expect("a field") + key.len()..];
            rest.trim_start_matches(" := ")
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<usize>()
                .expect("a number")
        };
        (field("requirementIndex"), field("threshold"))
    }

    /// The `index` of the Quality requirement `name`.
    fn requirement(quality: &str, name: &str) -> usize {
        let from = quality
            .find(&format!("name: requirement{name},"))
            .unwrap_or_else(|| panic!("no requirement{name} in:\n{quality}"));
        let rest = &quality[from + quality[from..].find("{index := ").expect("an index")..];
        rest.trim_start_matches("{index := ")
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse()
            .expect("a number")
    }

    #[test]
    fn the_frontier_ranks_its_refusals_by_reason() {
        let collision = |line: usize| Refusal::Collision {
            span: at(line),
            name: format!("n{line}"),
        };
        let unaccepted = |line: usize| Refusal::Unaccepted {
            span: at(line),
            feature: format!("f{line}"),
        };
        let refusals: BTreeSet<Refusal> = [
            Refusal::AdvisoriesUnconsulted,
            Refusal::NoMsrv {
                package: "p".into(),
            },
            unaccepted(1),
            unaccepted(2),
            collision(3),
            collision(4),
            collision(5),
        ]
        .into_iter()
        .collect();
        let doc: toml::Value = refusal_queue(&refusals).parse().expect("TOML");
        assert_eq!(count(&doc["admission"]["refusals"]), refusals.len());
        let rows: Vec<(&str, &str, usize)> = doc["reason"]
            .as_array()
            .expect("the queue")
            .iter()
            .map(|r| {
                (
                    r["reason"].as_str().expect("a reason"),
                    r["route"].as_str().expect("a route"),
                    count(&r["refusals"]),
                )
            })
            .collect();
        // The most refusals first; as many, by reason.
        assert_eq!(
            rows,
            [
                ("name-collision", "UR-R13", 3),
                ("feature-without-acceptance", "UR-R10", 2),
                ("advisories-unconsulted", "UR-R21", 1),
                ("no-msrv", "UR-R24", 1),
            ]
        );
        assert_eq!(rows.iter().map(|r| r.2).sum::<usize>(), refusals.len());
        // No refusal: the count is stated, and the queue is empty.
        let none: toml::Value = refusal_queue(&BTreeSet::new()).parse().expect("TOML");
        assert_eq!(count(&none["admission"]["refusals"]), 0);
        assert!(none.get("reason").is_none());
    }

    #[test]
    fn the_frontier_carries_the_queue_of_its_own_rows() {
        let mut notes = Notes::default();
        notes.refusals.insert(Refusal::AdvisoriesUnconsulted);
        for (i, name) in ["c::t1", "c::t2"].into_iter().enumerate() {
            let reason = Residue::Type("Wide".into());
            notes.residue.insert(name.into(), (at(i + 1), reason));
        }
        let form = Residue::Form("a\tloop".into());
        notes.residue.insert("c::g".into(), (at(3), form));
        notes
            .effects
            .insert("c::e".into(), (at(4), "Fs effect: std::fs::read".into()));
        notes.lifted.insert(
            "c::f".into(),
            (at(5), "Behavior::f".into(), vec!["ASCII only".into()]),
        );
        let kappas: BTreeMap<String, String> = ["c::t1", "c::t2", "c::g", "c::e", "c::f"]
            .into_iter()
            .map(|path| (path.to_string(), crate::register::kappa(path, "fn f () {}")))
            .collect();
        let rows = function_rows(&notes, &kappas);
        let lift = Register::parse(Path::new(FRONTIER), &rows).expect("the rows are a register");
        let text = frontier(&notes, &lift, &rows);
        let doc: toml::Value = text.parse().expect("the frontier is TOML");
        let totals: Vec<usize> = ["functions", "lifted", "residue", "foreign", "effect"]
            .iter()
            .map(|key| count(&doc["lift"][key]))
            .collect();
        assert_eq!(totals, [5, 1, 3, 0, 1]);
        let queue: Vec<(&str, &str, usize)> = doc["construct"]
            .as_array()
            .expect("the queue")
            .iter()
            .map(|r| {
                (
                    r["cause"].as_str().expect("a cause"),
                    r["construct"].as_str().expect("a construct"),
                    count(&r["functions"]),
                )
            })
            .collect();
        // A construct is the detail as a reader of the rows finds it: the
        // tab the frontier writes as a space is a space in the queue too.
        assert_eq!(
            queue,
            [
                ("type", "type outside the lift: Wide", 2),
                ("effect", "Fs effect: std::fs::read", 1),
                ("form", "expression form outside the lift: a loop", 1),
            ]
        );
        assert_eq!(count(&doc["admission"]["refusals"]), 1);
        assert_eq!(doc["refusal"].as_array().map(Vec::len), Some(1));
        // The frontier is the register it was written from, queue and all.
        assert_eq!(Register::parse(Path::new(FRONTIER), &text), Ok(lift));
        // Rows that are not a register give no projection: a function with
        // no κ has no row to key.
        assert!(matches!(
            Register::parse(
                Path::new(FRONTIER),
                &function_rows(&notes, &BTreeMap::new())
            ),
            Err(UorustError::Malformed { .. })
        ));
    }

    #[test]
    fn quality_states_the_lift_and_the_open_refusals_as_measures() {
        let files = projected_files("quality");
        let frontier = toml_of(&files, FRONTIER);
        let quality = String::from_utf8_lossy(&files["src/Quality.lex.tex"]).to_string();
        // The register the same run writes.
        let totals = crate::lift_register(&files).expect("a register").totals();
        assert_eq!((totals.functions, totals.lifted, totals.residue), (3, 1, 2));
        let refusals = frontier["refusal"].as_array().expect("refusals").len();
        // Quality's own refusal (no `rust-version`) is one of them.
        assert!(frontier["refusal"]
            .as_array()
            .expect("refusals")
            .iter()
            .any(|r| r["reason"].as_str() == Some("no-msrv")));
        assert_eq!(count(&frontier["admission"]["refusals"]), refusals);
        assert_eq!(count(&frontier["lift"]["functions"]), totals.functions);
        assert_eq!(count(&frontier["lift"]["lifted"]), totals.lifted);
        for (name, measured) in [
            ("ProductFunctions", totals.functions),
            ("LiftedFunctions", totals.lifted),
            ("OpenRefusals", refusals),
        ] {
            let (of, threshold) = measure(&quality, name);
            assert_eq!(threshold, measured, "measure{name}");
            // Built as every other measure is: under its own requirement.
            assert_eq!(of, requirement(&quality, name), "measure{name}");
        }
        // The measure this one is modelled on is still built the same way.
        let (of, _) = measure(&quality, "AdmittedFeatures");
        assert_eq!(of, requirement(&quality, "AdmittedFeatures"));
        // The queue of the two functions one construct stops.
        let first = &frontier["construct"].as_array().expect("the queue")[0];
        assert_eq!(first["cause"].as_str(), Some("arithmetic"));
        assert_eq!(count(&first["functions"]), 2);
    }

    #[test]
    fn a_module_beyond_the_stated_limit_is_a_typed_error() {
        let project = |limit: usize, module: usize| -> BTreeMap<String, Vec<u8>> {
            [
                (
                    LEXLEAN.to_string(),
                    format!("[limits]\nmax_file_bytes = {limit}\n").into_bytes(),
                ),
                ("src/Design.lex.tex".to_string(), vec![b' '; module]),
                // No module: LexLean does not read it as a source file.
                (FRONTIER.to_string(), vec![b'#'; limit + 1]),
            ]
            .into_iter()
            .collect()
        };
        let limit = 64;
        // At the limit a module is admitted; one byte beyond it is refused.
        assert_eq!(within_limit(&project(limit, limit)), Ok(()));
        let refused = within_limit(&project(limit, limit + 1)).expect_err("beyond the limit");
        assert_eq!(
            refused,
            UorustError::ModuleTooLarge {
                module: "src/Design.lex.tex".into(),
                bytes: limit + 1,
                limit,
            }
        );
        let said = refused.to_string();
        assert!(
            said.contains("src/Design.lex.tex") && said.contains("65 bytes") && said.contains("64"),
            "{said}"
        );
        // A project file that states no limit is not one to check against.
        let mut silent = project(limit, limit);
        silent.insert(LEXLEAN.to_string(), b"[limits]\n".to_vec());
        assert!(matches!(
            within_limit(&silent),
            Err(UorustError::Malformed { .. })
        ));
    }

    /// The limit checked is the one the projection's own `lexlean.toml`
    /// states: a projection is within it, and the same files under a lower
    /// stated limit are refused at their largest module.
    #[test]
    fn a_projection_is_checked_against_the_limit_it_states() {
        let mut files = projected_files("limit");
        let stated = format!("max_file_bytes = {MAX_FILE_BYTES}\n");
        let config = String::from_utf8_lossy(&files[LEXLEAN]).to_string();
        assert!(config.contains(&stated), "{config}");
        assert_eq!(within_limit(&files), Ok(()));
        // The first of the largest, in the order the check reads them.
        let (largest, bytes) = files
            .iter()
            .rev()
            .filter(|(path, _)| path.ends_with(".lex.tex"))
            .map(|(path, bytes)| (path.clone(), bytes.len()))
            .max_by_key(|(_, bytes)| *bytes)
            .expect("a module");
        let lower = bytes - 1;
        files.insert(
            LEXLEAN.to_string(),
            config
                .replace(&stated, &format!("max_file_bytes = {lower}\n"))
                .into_bytes(),
        );
        assert_eq!(
            within_limit(&files),
            Err(UorustError::ModuleTooLarge {
                module: largest,
                bytes,
                limit: lower,
            })
        );
    }

    use super::{parts, record, sized, string, Decl, Module};

    /// A module of separate records in the shape of a design facet: a view,
    /// `n` processes, and `n` things that each cite the view and a process.
    fn design_like(n: usize) -> Module {
        let t = "Foundation.Arch::";
        let id = |name: &str| string(&format!("Design::{name}"));
        let mut decls = vec![Decl::value(
            "opmView",
            format!("{t}DesignView"),
            record(&format!("{t}DesignView"), &[("title", string("Processes"))]),
        )];
        for i in 0..n {
            decls.push(Decl::value(
                format!("process{i}"),
                format!("{t}SystemProcess"),
                record(&format!("{t}SystemProcess"), &[("owner", string("c"))]),
            ));
        }
        for i in 0..n {
            decls.push(Decl::value(
                format!("thing{i}"),
                format!("{t}OpmThing"),
                record(
                    &format!("{t}OpmThing"),
                    &[
                        ("view", id("opmView")),
                        ("subject", id(&format!("process{i}"))),
                    ],
                ),
            ));
        }
        Module {
            name: "Design".into(),
            glossaries: vec![super::BOOL],
            imports: vec!["Foundation.Arch".into()],
            title: "design view",
            decls,
        }
    }

    /// The names a module text declares, in order.
    fn declared(text: &str) -> Vec<String> {
        text.split("name: ")
            .skip(1)
            .map(|rest| rest.split(',').next().expect("a name").to_string())
            .collect()
    }

    /// Every ID a module text cites: each string of the form `"Design..::name"`.
    fn cited_ids(text: &str) -> Vec<String> {
        text.split('"')
            .filter(|piece| piece.starts_with("Design") && piece.contains("::"))
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn a_module_of_separate_records_beyond_the_limit_is_written_in_parts() {
        let module = design_like(40);
        let whole = module.render();
        let nothing = |_: &str| false;
        // Within the limit it is written whole, byte for byte.
        assert_eq!(
            sized(std::slice::from_ref(&module), whole.len(), &nothing),
            [(module.path(), whole.clone())]
        );
        // A third of its size holds no more than a third of it.
        let limit = whole.len() / 3;
        let written = sized(std::slice::from_ref(&module), limit, &nothing);
        assert!(written.len() > 3, "{} parts", written.len());
        let mut ids = BTreeSet::new();
        let mut order = Vec::new();
        for (k, (path, text)) in written.iter().enumerate() {
            // Each part is a module of its own, within the limit, named in turn.
            let name = match k {
                0 => "Design".to_string(),
                _ => format!("Design{}", k + 1),
            };
            assert_eq!(path, &format!("src/{name}.lex.tex"));
            assert!(text.starts_with(&format!("\\begin{{lexlean}}{{{name}}}\n")));
            assert!(text.len() <= limit, "{path}: {} > {limit}", text.len());
            for declared in declared(text) {
                assert!(
                    ids.insert(format!("{name}::{declared}")),
                    "{declared} twice"
                );
                order.push(declared);
            }
        }
        // Every record is written once, in its order.
        assert_eq!(order, declared(&whole));
        // Every reference names the part its record is in, and some reach
        // back into the first part and some into a later one.
        let cited: Vec<String> = written.iter().flat_map(|(_, t)| cited_ids(t)).collect();
        assert_eq!(cited.len(), cited_ids(&whole).len());
        for id in &cited {
            assert!(ids.contains(id), "{id} names no record");
        }
        let last = &written[written.len() - 1].1;
        assert!(last.contains("\"Design::opmView\""), "{last}");
        assert!(cited.iter().any(|id| !id.starts_with("Design::")));
        // The parts are the same on a second run.
        assert_eq!(
            written,
            sized(std::slice::from_ref(&module), limit, &nothing)
        );
    }

    /// What is not a module of separate records, or is cited from outside
    /// itself, is written whole, and the size check refuses the projection.
    #[test]
    fn a_module_that_cannot_be_parted_is_left_to_the_typed_error() {
        let module = design_like(40);
        let whole = module.render();
        let limit = whole.len() / 3;
        let taken = BTreeSet::new();
        assert!(parts(&module, limit, &taken).is_some());
        let unparted = [(module.path(), whole.clone())];
        // Another file cites one of its records.
        let elsewhere = |id: &str| id == "Design::";
        assert_eq!(
            sized(std::slice::from_ref(&module), limit, &elsewhere),
            unparted
        );
        // Another module does.
        let citing = Module {
            name: "Interface".into(),
            decls: vec![Decl::value("journey", "String", string("Design::opmView"))],
            ..design_like(0)
        };
        let both = [module.clone(), citing];
        assert_eq!(sized(&both, limit, &|_| false)[0], unparted[0]);
        // A record with a canonical index, whose order its ID decides.
        let mut indexed = design_like(40);
        indexed.decls[0] = Decl::value("opmView", "T", record("T", &[("index", "0".into())]));
        assert!(parts(&indexed, limit, &taken).is_none());
        // A declaration that names another outside a string, or takes a
        // parameter, is no separate record.
        let mut naming = design_like(40);
        naming.decls[1] = Decl::value("process0", "T", "opmView".to_string());
        assert!(parts(&naming, limit, &taken).is_none());
        let mut taking = design_like(40);
        if let Decl::Definition { parameters, .. } = &mut taking.decls[1] {
            parameters.push(("x".into(), "Nat".into()));
        }
        assert!(parts(&taking, limit, &taken).is_none());
        // One record alone beyond the limit, and a part's name already a
        // module's.
        let one = crate::lex::render_decl(&module.decls[0]).len();
        assert!(parts(&module, one, &taken).is_none());
        assert!(parts(&module, limit, &BTreeSet::from(["Design2"])).is_none());
        // Left whole, it is the typed error's.
        let files: BTreeMap<String, Vec<u8>> = [
            (
                LEXLEAN.to_string(),
                format!("[limits]\nmax_file_bytes = {limit}\n").into_bytes(),
            ),
            (module.path(), whole.clone().into_bytes()),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            within_limit(&files),
            Err(UorustError::ModuleTooLarge {
                module: "src/Design.lex.tex".into(),
                bytes: whole.len(),
                limit,
            })
        );
    }

    // Name resolution: admission resolves names as the language does.

    /// The frontier of a one-library workspace with `source` as its `lib.rs`.
    fn resolved_frontier(tag: &str, manifest_extra: &str, source: &str) -> String {
        let dir = std::env::temp_dir().join(format!("uorust-resolve-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"w\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\
                 rust-version = \"1.70\"\n{manifest_extra}"
            ),
        )
        .unwrap();
        std::fs::write(dir.join("src/lib.rs"), source).unwrap();
        let options = super::Options {
            name: "w".to_string(),
            last_modified: "2026-11-15T00:00:00Z".to_string(),
            advisories: None,
            mir: None,
        };
        let files = crate::project_workspace(&dir, &options).expect("a projection");
        let _ = std::fs::remove_dir_all(&dir);
        String::from_utf8(files["frontier.toml"].clone()).unwrap()
    }

    #[test]
    fn a_call_resolves_in_its_nearest_module_before_it_is_ambiguous() {
        let frontier = resolved_frontier(
            "near",
            "",
            "pub mod left {\n pub fn near() -> u8 { 3 }\n pub fn here() -> u8 { near() }\n}\n\
             pub mod right {\n pub fn near() -> u8 { 4 }\n}\n",
        );
        assert!(!frontier.contains("ambiguous-call"), "{frontier}");
    }

    #[test]
    fn a_registry_type_a_use_binds_is_a_foreign_row() {
        let frontier = resolved_frontier(
            "foreign",
            "\n[dependencies]\nserde_json = \"1\"\n",
            "use serde_json::Value;\npub fn null(v: Value) -> bool { v.is_null() }\n",
        );
        assert!(
            frontier.contains("takes or returns serde_json::Value, a type of the serde_json crate"),
            "{frontier}"
        );
    }

    #[test]
    fn complementary_cfg_variants_are_one_fact_not_a_collision() {
        let frontier = resolved_frontier(
            "variant",
            "\n[features]\nwide = []\n",
            "#[cfg(feature = \"wide\")]\npub fn variant() -> u8 { 1 }\n\
             #[cfg(not(feature = \"wide\"))]\npub fn variant() -> u8 { 2 }\n",
        );
        assert!(!frontier.contains("name-collision"), "{frontier}");
        assert!(frontier.contains("reason = \"cfg-variant\""), "{frontier}");
        assert!(
            frontier.contains("w::variant has a variant under cfg(feature = \\\"wide\\\")"),
            "{frontier}"
        );
    }

    // What a type name means: the leaf of a chain is named as the file
    // that declares the last type of the chain writes it.

    /// The row the frontier writes for `function`: its table and detail.
    fn type_names_row(frontier: &str, function: &str) -> String {
        let at = frontier
            .find(&format!("function = \"{function}\""))
            .unwrap_or_else(|| panic!("no row of {function}: {frontier}"));
        let start = frontier[..at].rfind("[[").unwrap();
        let end = frontier[at..]
            .find("\n\n")
            .map_or(frontier.len(), |e| at + e);
        frontier[start..end].to_string()
    }

    #[test]
    fn type_names_a_chain_ending_at_a_registry_type_is_a_foreign_row() {
        let frontier = resolved_frontier(
            "leaf-foreign",
            "\n[dependencies]\nbignum = \"1\"\nwide = \"1\"\n",
            "pub mod num {\n use wide::prelude::*;\n use bignum::BigX;\n\
             pub struct Ratio { pub q: BigX }\n}\n\
             pub mod calc {\n use crate::num::Ratio;\n\
             pub fn keep(r: Ratio) -> Ratio { r }\n}\n",
        );
        let row = type_names_row(&frontier, "w::keep");
        assert!(row.starts_with("[[foreign]]"), "{row}");
        assert!(
            row.contains("Ratio.q: bignum::BigX, a type of the bignum crate"),
            "{row}"
        );
    }

    #[test]
    fn type_names_a_head_the_lift_does_not_read_is_the_leaf() {
        let frontier = resolved_frontier(
            "leaf-head",
            "\n[dependencies]\nbignum = \"1\"\n",
            "use std::cell::RefCell;\nuse bignum::BigX;\n\
             pub struct Held { pub c: RefCell<BigX> }\n\
             pub fn keep(h: Held) -> Held { h }\n",
        );
        let row = type_names_row(&frontier, "w::keep");
        assert!(row.starts_with("[[residue]]"), "{row}");
        assert!(row.contains("Held.c: RefCell<bignum::BigX>"), "{row}");
    }

    #[test]
    fn type_names_a_registry_map_a_use_binds_is_not_the_standard_one() {
        let frontier = resolved_frontier(
            "leaf-map",
            "\n[dependencies]\nfastmap = \"1\"\n",
            "use fastmap::HashMap;\n\
             pub struct Index { pub at: HashMap<u8, u8> }\n\
             pub fn size(i: &Index) -> usize { i.at.len() }\n",
        );
        let row = type_names_row(&frontier, "w::size");
        assert!(row.starts_with("[[foreign]]"), "{row}");
        assert!(row.contains("a type of the fastmap crate"), "{row}");
    }

    #[test]
    fn type_names_types_of_two_inline_modules_of_a_file_are_two_types() {
        let frontier = resolved_frontier(
            "inline-types",
            "",
            "pub struct Shape { pub w: u8 }\npub fn wide(s: Shape) -> u8 { s.w }\n\
             pub mod ext {\n pub struct Shape { pub h: u8 }\n\
             pub fn tall(s: Shape) -> u8 { s.h }\n}\n",
        );
        assert!(!frontier.contains("declared twice"), "{frontier}");
        assert!(type_names_row(&frontier, "w::wide").starts_with("[[lifted]]"));
        assert!(type_names_row(&frontier, "w::tall").starts_with("[[lifted]]"));
    }

    #[test]
    fn type_names_a_shared_name_reads_as_what_the_module_binds() {
        let frontier = resolved_frontier(
            "shared-glob",
            "",
            "pub mod a {\n pub struct Op { pub k: u8 }\n}\n\
             pub mod b {\n pub struct Op { pub j: u8 }\n}\n\
             pub mod c {\n use crate::b::*;\n pub fn j(o: Op) -> u8 { o.j }\n}\n\
             pub mod d {\n use std::cell::Cell;\n\
             pub struct Count { pub n: Cell<u8> }\n\
             pub fn keep(c: Count) -> Count { c }\n}\n\
             pub mod e {\n pub struct Cell { pub v: u8 }\n}\n\
             pub mod f {\n pub struct Cell { pub u: u8 }\n}\n",
        );
        assert!(
            type_names_row(&frontier, "w::j").starts_with("[[lifted]]"),
            "{frontier}"
        );
        let row = type_names_row(&frontier, "w::keep");
        assert!(row.contains("Count.n: std::cell::Cell<u8>"), "{row}");
        assert!(!frontier.contains("one of several types"), "{frontier}");
    }

    #[test]
    fn type_names_a_name_two_globs_of_outside_crates_may_bind_stays_open() {
        let frontier = resolved_frontier(
            "shared-open",
            "\n[dependencies]\nwide = \"1\"\nthin = \"1\"\n",
            "pub mod a {\n pub struct Op { pub k: u8 }\n}\n\
             pub mod b {\n pub struct Op { pub j: u8 }\n}\n\
             pub mod c {\n use wide::*;\n use thin::*;\n pub struct Holder { pub o: Op }\n\
             pub fn keep(h: Holder) -> Holder { h }\n}\n",
        );
        let row = type_names_row(&frontier, "w::keep");
        assert!(row.contains(crate::lift::SHARED_NAME), "{row}");
    }

    #[test]
    fn type_names_a_standard_map_with_a_hasher_reads_as_the_unordered_map() {
        let frontier = resolved_frontier(
            "hasher-map",
            "",
            "use std::collections::{HashMap, HashSet};\nuse std::hash::BuildHasherDefault;\n\
             pub struct Mix { pub state: u64 }\n\
             pub struct Index {\n pub at: HashMap<u8, u8, BuildHasherDefault<Mix>>,\n\
             pub seen: HashSet<u8, BuildHasherDefault<Mix>>,\n}\n\
             pub fn has(i: &Index, k: u8) -> bool { i.at.contains_key(&k) }\n\
             pub fn saw(i: &Index, k: u8) -> bool { i.seen.contains(&k) }\n\
             pub fn keys(i: &Index) -> Vec<u8> { i.at.keys().cloned().collect() }\n",
        );
        assert!(
            type_names_row(&frontier, "w::has").starts_with("[[lifted]]"),
            "{frontier}"
        );
        assert!(
            type_names_row(&frontier, "w::saw").starts_with("[[lifted]]"),
            "{frontier}"
        );
        let keys = type_names_row(&frontier, "w::keys");
        assert!(keys.contains("whose order is unspecified"), "{keys}");
    }

    #[test]
    fn type_names_a_name_only_a_registry_glob_binds_is_that_crates() {
        let frontier = resolved_frontier(
            "glob-leaf",
            "\n[dependencies]\nbits = \"1\"\n",
            "pub mod store {\n use bits::prelude::*;\n\
             pub struct Map { pub bits: BitBox, pub n: Vec<u8> }\n}\n\
             pub mod read {\n use crate::store::Map;\n pub fn keep(m: Map) -> Map { m }\n}\n\
             pub struct Shape { pub w: u8 }\npub struct Holder { pub s: Shape }\n\
             pub fn width(h: &Holder) -> u8 { h.s.w }\n\
             pub mod ext {\n use bits::prelude::*;\n pub fn lone(b: BitBox) -> BitBox { b }\n}\n",
        );
        let row = type_names_row(&frontier, "w::keep");
        assert!(
            row.contains("Map.bits: bits::prelude::BitBox, a type of the bits crate"),
            "{row}"
        );
        // The glob of an inline module binds names in that module only.
        assert!(
            type_names_row(&frontier, "w::width").starts_with("[[lifted]]"),
            "{frontier}"
        );
        assert!(type_names_row(&frontier, "w::lone").contains("a type of the bits crate"));
    }

    #[test]
    fn type_names_a_self_type_written_by_a_path_reads_as_the_type_it_names() {
        let frontier = resolved_frontier(
            "owner-path",
            "",
            "pub struct Shape { pub w: u8 }\n\
             pub mod ext {\n impl super::Shape {\n  pub fn width(&self) -> u8 { self.w }\n }\n}\n",
        );
        let at = frontier.find("::width\"").expect("a row of width");
        let start = frontier[..at].rfind("[[").unwrap();
        assert!(frontier[start..].starts_with("[[lifted]]"), "{frontier}");
    }
}
