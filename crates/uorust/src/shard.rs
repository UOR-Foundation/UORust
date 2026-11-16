//! The layout `uorust embed` reads a library crate's MIR in, and the driver
//! that embeds a crate of a workspace by it.
//!
//! A crate's dumps are laid out one directory per module ([`lay_out`]),
//! with `target.toml` beside them ([`write_target`]): the target the bodies
//! were built for, whose pointer width and byte order the evaluator reads.
//! `uorust crate` writes that layout for a crate of crates.io and `uorust
//! mir` for each library of a workspace, from the dump of each, so that one
//! fact has one writer. Each dump is of one configuration (the cargo feature
//! set and the target, [`Configuration`]), recorded beside it: a function
//! that exists only under a feature has a body only in a dump of a
//! configuration that enables it, so a body is keyed by its path and the
//! configuration it was dumped in.
//!
//! None of this is translation: it lays files out and runs cargo and rustc,
//! so it sits outside the translator's modules.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::io;
use crate::UorustError;

/// The directory of `uorust mir`'s output that holds a workspace library's
/// module shards, `shards-<lib>`, beside its plain dump `lib-<lib>`.
pub const SHARDS: &str = "shards-";

/// The file that records the configuration a dump was built in.
pub const CONFIGURATION: &str = "configuration.toml";

/// The file of a crate's shards that lists its modules ([`write_modules`]),
/// which a factory declares one embedding obligation per module from.
pub const MODULES: &str = "modules.toml";

/// The file of a crate's shards that states the target they were built for.
pub const TARGET: &str = "target.toml";

/// The file of an embedded project that records what the embedding of one
/// module (or one set of modules) in one configuration counted
/// ([`render_record`]): the row a factory obligation keeps per
/// (configuration, module).
pub const RECORD: &str = "record.toml";

/// The directory of an embedding's output that holds the project
/// `lexlean verify` decides. The [`CONFIGURATION`] and the [`RECORD`] lie
/// beside it, never in it: a module whose dump is the same in two
/// configurations is one project, of one digest ([`project_digest`]), and
/// one verification of it is a receipt for both.
pub const PROJECT: &str = "project";

/// The configuration a crate is built and dumped in: the cargo feature set
/// and the target, as `cargo rustc` takes them. The default is the
/// workspace's default features on the host.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Configuration {
    /// Features to enable (`--features`), each as cargo names it.
    pub features: Vec<String>,
    /// Leave the default features off (`--no-default-features`).
    pub no_default_features: bool,
    /// Enable every feature (`--all-features`).
    pub all_features: bool,
    /// The target triple (`--target`); the host's when absent.
    pub target: Option<String>,
}

impl Configuration {
    /// The configuration package `name` is built in, which declares the
    /// features `declared`: cargo refuses a feature the selected package
    /// does not declare, so a plain feature is the package's where it
    /// declares it, and `name/feature` is always its (cargo's own spelling of
    /// a member's feature in a workspace); every other feature is another
    /// package's.
    pub fn for_package(&self, name: &str, declared: &BTreeSet<String>) -> Configuration {
        let features = self
            .features
            .iter()
            .filter_map(|f| match f.split_once('/') {
                Some((p, f)) => (p == name).then(|| f.to_string()),
                None => declared.contains(f).then(|| f.clone()),
            })
            .collect();
        Configuration {
            features,
            ..self.clone()
        }
    }

    /// The arguments that select this configuration, for `cargo rustc`
    /// and `cargo metadata` (`--filter-platform` there for the target).
    pub fn cargo_args(&self, metadata: bool) -> Vec<String> {
        let mut args = Vec::new();
        if !self.features.is_empty() {
            args.push("--features".to_string());
            args.push(self.features.join(","));
        }
        if self.no_default_features {
            args.push("--no-default-features".to_string());
        }
        if self.all_features {
            args.push("--all-features".to_string());
        }
        if let Some(t) = &self.target {
            args.push(
                if metadata {
                    "--filter-platform"
                } else {
                    "--target"
                }
                .to_string(),
            );
            args.push(t.clone());
        }
        args
    }

    /// The configuration's name as a register row states it: its features
    /// (every feature, or the default ones with those named, or only those
    /// named) and its target where one is named.
    pub fn label(&self) -> String {
        let features = if self.all_features {
            "all features".to_string()
        } else {
            match (self.no_default_features, self.features.is_empty()) {
                (false, true) => "default features".to_string(),
                (false, false) => format!("default features and {}", self.features.join(",")),
                (true, true) => "no features".to_string(),
                (true, false) => format!("features {}", self.features.join(",")),
            }
        };
        match &self.target {
            Some(t) => format!("{features} on {t}"),
            None => features,
        }
    }

    /// The configuration as [`CONFIGURATION`] records it, with the triple
    /// the build was for.
    pub fn render(&self, triple: &str) -> String {
        let quote = |s: &str| toml::Value::String(s.to_string()).to_string();
        let features: Vec<String> = self.features.iter().map(|f| quote(f)).collect();
        format!(
            "# The configuration the dumps beside this file were built in\n\
             # (written by `uorust mir`; do not edit): a body is keyed by its\n\
             # path and this configuration.\n\n\
             features = [{}]\ndefault_features = {}\nall_features = {}\ntarget = {}\n",
            features.join(", "),
            !self.no_default_features,
            self.all_features,
            quote(triple)
        )
    }
}

fn tool(what: &str, reason: impl std::fmt::Display) -> UorustError {
    UorustError::Tool(format!("{what}: {reason}"))
}

fn output(what: &str, command: &mut Command) -> Result<String, UorustError> {
    let out = command.output().map_err(|e| tool(what, e))?;
    if !out.status.success() {
        return Err(tool(what, String::from_utf8_lossy(&out.stderr)));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// `rustc`, run in `dir` with the toolchain `toolchain` (`+1.97.1`) when it
/// is given, else the one the directory selects.
fn rustc(dir: &Path, toolchain: Option<&str>) -> Command {
    let mut c = Command::new("rustc");
    c.args(toolchain)
        .current_dir(dir)
        .env("RUSTC_BOOTSTRAP", "1");
    c
}

/// The host triple of the toolchain [`rustc`] runs.
pub fn host(dir: &Path, toolchain: Option<&str>) -> Result<String, UorustError> {
    output("rustc -vV", rustc(dir, toolchain).arg("-vV"))?
        .lines()
        .find_map(|l| l.strip_prefix("host: "))
        .map(str::to_string)
        .ok_or_else(|| tool("rustc -vV", "no host"))
}

/// The sysroot of the toolchain [`rustc`] runs, which a dump's paths into
/// it are written relative to.
pub fn sysroot(dir: &Path, toolchain: Option<&str>) -> Result<String, UorustError> {
    Ok(output(
        "rustc --print sysroot",
        rustc(dir, toolchain).args(["--print", "sysroot"]),
    )?
    .trim()
    .to_string())
}

/// Write [`TARGET`] in `shards` for `triple`: its pointer width and data
/// layout as rustc's own target specification states them.
pub fn write_target(
    shards: &Path,
    triple: &str,
    dir: &Path,
    toolchain: Option<&str>,
) -> Result<(), UorustError> {
    let width = output(
        "rustc --print cfg",
        rustc(dir, toolchain).args(["--print", "cfg", "--target", triple]),
    )?
    .lines()
    .find_map(|l| l.strip_prefix("target_pointer_width=\""))
    .map(|w| w.trim_end_matches('"').to_string())
    .ok_or_else(|| tool("rustc", "no pointer width"))?;
    let spec: serde_json::Value = serde_json::from_str(&output(
        "rustc --print target-spec-json",
        rustc(dir, toolchain).args([
            "-Zunstable-options",
            "--print",
            "target-spec-json",
            "--target",
            triple,
        ]),
    )?)
    .map_err(|e| tool("rustc", e))?;
    let layout = spec
        .get("data-layout")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| tool("rustc", "no data layout"))?;
    let file = shards.join(TARGET);
    std::fs::write(
        &file,
        format!("triple = \"{triple}\"\npointer_width = {width}\ndata_layout = \"{layout}\"\n"),
    )
    .map_err(|e| io(&file, e))
}

/// `text` with each path that starts with `own` (a crate's directory as a
/// dump names its sources, `member/` or `/abs/member/`) written from the
/// crate's own directory instead: an occurrence counts only where a path
/// starts, so `own` inside a longer path is kept.
fn without_prefix(text: &str, own: &str) -> String {
    if own.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(own) {
        let starts = rest[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !(c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | '\\')));
        out.push_str(&rest[..at]);
        if !starts {
            out.push_str(own);
        }
        rest = &rest[at + own.len()..];
    }
    out.push_str(rest);
    out
}

/// Lay the dump files of one library crate in `mir` out under `shards`,
/// one directory per module ([`crate::registry::dump_module`]), with the
/// machine's paths resolved away: the crate's own directory `own` (as the
/// dump names it) dropped, so its sources read from the crate's directory,
/// the registry's sources as `<registry>/` and the toolchain's `sysroot`
/// as `<sysroot>`. Returns how many files it wrote.
pub fn lay_out(mir: &Path, shards: &Path, own: &str, sysroot: &str) -> Result<usize, UorustError> {
    let cargo_home = std::env::var("CARGO_HOME")
        .unwrap_or_else(|_| format!("{}/.cargo", std::env::var("HOME").unwrap_or_default()));
    let registry = format!("{cargo_home}/registry/src/");
    let mut files: Vec<PathBuf> = std::fs::read_dir(mir)
        .map_err(|e| io(mir, e))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".ElaborateBoxDerefs.before.mir"))
        })
        .collect();
    files.sort();
    let names: Vec<String> = files
        .iter()
        .map(|f| {
            f.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string()
        })
        .collect();
    let items: BTreeSet<&str> = names
        .iter()
        .filter_map(|n| crate::registry::dump_item(n))
        .collect();
    let mut written = 0;
    for (file, name) in files.iter().zip(&names) {
        let Some(item) = crate::registry::dump_item(name) else {
            continue;
        };
        let module = crate::registry::dump_module(item, &items);
        let text = std::fs::read_to_string(file).map_err(|e| io(file, e))?;
        let text = without_prefix(&text, own)
            .replace(&registry, "<registry>/")
            .replace(sysroot, "<sysroot>");
        let dest = shards.join(&module);
        std::fs::create_dir_all(&dest).map_err(|e| io(&dest, e))?;
        let path = dest.join(name);
        std::fs::write(&path, text).map_err(|e| io(&path, e))?;
        written += 1;
    }
    Ok(written)
}

/// One module of a crate's shards: its name, its dump files and their
/// bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Module {
    /// The module's directory name.
    pub name: String,
    /// The dump files in it.
    pub files: usize,
    /// Their size.
    pub bytes: u64,
}

/// The modules of the shards under `shards`, smallest first (by bytes,
/// then name): the order a factory runs their obligations in, so the
/// cheap ones prove first.
pub fn modules(shards: &Path) -> Result<Vec<Module>, UorustError> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(shards).map_err(|e| io(shards, e))? {
        let path = entry.map_err(|e| io(shards, e))?.path();
        if !path.is_dir() {
            continue;
        }
        let (mut files, mut bytes) = (0, 0);
        for f in std::fs::read_dir(&path).map_err(|e| io(&path, e))? {
            let f = f.map_err(|e| io(&path, e))?;
            files += 1;
            bytes += f.metadata().map_err(|e| io(&f.path(), e))?.len();
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        out.push(Module { name, files, bytes });
    }
    out.sort_by(|a, b| a.bytes.cmp(&b.bytes).then_with(|| a.name.cmp(&b.name)));
    Ok(out)
}

/// Write [`MODULES`] in `shards`: each module with its files and bytes,
/// smallest first ([`modules`]).
pub fn write_modules(shards: &Path) -> Result<Vec<Module>, UorustError> {
    let found = modules(shards)?;
    let mut text = String::from(
        "# The modules of these shards, smallest first (written by `uorust mir`;\n\
         # do not edit): one `uorust embed --only <name>` project each.\n",
    );
    for m in &found {
        text.push_str(&format!(
            "\n[[module]]\nname = {}\nfiles = {}\nbytes = {}\n",
            toml::Value::String(m.name.clone()),
            m.files,
            m.bytes
        ));
    }
    let file = shards.join(MODULES);
    std::fs::write(&file, text).map_err(|e| io(&file, e))?;
    Ok(found)
}

/// The library crates the package at `dir` builds on, as cargo resolves
/// them under `args` (a toolchain and a [`Configuration`]'s metadata
/// arguments): each normal dependency, and theirs, by its library's crate
/// name with the directory of its package. The package is `package` (a
/// member's name, for a workspace) or the resolve's root.
pub fn dependencies(
    dir: &Path,
    args: &[String],
    package: Option<&str>,
) -> Result<Vec<(String, PathBuf)>, UorustError> {
    let out = output(
        "cargo metadata",
        Command::new("cargo")
            .args(args.iter().take_while(|a| a.starts_with('+')))
            .args(["metadata", "-q", "--format-version", "1"])
            .args(args.iter().skip_while(|a| a.starts_with('+')))
            .current_dir(dir),
    )?;
    let meta: serde_json::Value = serde_json::from_str(&out).map_err(|e| tool("metadata", e))?;
    let str_of =
        |v: &serde_json::Value, k: &str| v.get(k).and_then(|x| x.as_str()).map(str::to_string);
    let packages: Vec<&serde_json::Value> = meta
        .get("packages")
        .and_then(|p| p.as_array())
        .into_iter()
        .flatten()
        .collect();
    let root = match package {
        Some(name) => packages
            .iter()
            .filter(|p| str_of(p, "name").as_deref() == Some(name))
            .find_map(|p| str_of(p, "id"))
            .ok_or_else(|| tool("cargo metadata", format!("no package `{name}`")))?,
        None => meta
            .pointer("/resolve/root")
            .and_then(|r| r.as_str())
            .map(str::to_string)
            .unwrap_or_default(),
    };
    let nodes: std::collections::BTreeMap<String, &serde_json::Value> = meta
        .pointer("/resolve/nodes")
        .and_then(|n| n.as_array())
        .into_iter()
        .flatten()
        .filter_map(|n| Some((str_of(n, "id")?, n)))
        .collect();
    let mut seen = BTreeSet::new();
    let mut next = vec![root.clone()];
    while let Some(id) = next.pop() {
        let Some(node) = nodes.get(&id) else { continue };
        for dep in node
            .get("deps")
            .and_then(|d| d.as_array())
            .into_iter()
            .flatten()
        {
            let normal = dep
                .get("dep_kinds")
                .and_then(|k| k.as_array())
                .into_iter()
                .flatten()
                .any(|k| k.get("kind").is_some_and(serde_json::Value::is_null));
            if let Some(pkg) = str_of(dep, "pkg").filter(|_| normal) {
                if seen.insert(pkg.clone()) {
                    next.push(pkg);
                }
            }
        }
    }
    let mut found = Vec::new();
    for package in packages {
        let id = str_of(package, "id").unwrap_or_default();
        if id == root || !seen.contains(&id) {
            continue;
        }
        let manifest = str_of(package, "manifest_path").map(PathBuf::from);
        let lib = library_target(package);
        if let (Some(lib), Some(dir)) = (lib, manifest.as_deref().and_then(Path::parent)) {
            found.push((lib.replace('-', "_"), dir.to_path_buf()));
        }
    }
    Ok(found)
}

/// The name of a `cargo metadata` package's library target, if it has one.
pub fn library_target(package: &serde_json::Value) -> Option<String> {
    package
        .get("targets")
        .and_then(|t| t.as_array())
        .into_iter()
        .flatten()
        .find(|t| {
            t.get("kind")
                .and_then(|k| k.as_array())
                .is_some_and(|k| k.iter().any(|k| k == "lib" || k == "rlib"))
        })
        .and_then(|t| t.get("name").and_then(|x| x.as_str()).map(str::to_string))
}

/// Mount, under `root/library`, what the embedding reads declarations
/// from: `library`'s core, alloc and std, the crate `krate` from its
/// package directory `dir`, and each crate it builds on from its own,
/// all as links. Returns the crates it builds on.
pub fn mount(
    root: &Path,
    library: &Path,
    krate: &str,
    dir: &Path,
    builds_on: &[(String, PathBuf)],
) -> Result<Vec<String>, UorustError> {
    let mounted = root.join("library");
    std::fs::create_dir_all(&mounted).map_err(|e| io(&mounted, e))?;
    let library = std::fs::canonicalize(library).map_err(|e| io(library, e))?;
    let dir = std::fs::canonicalize(dir).map_err(|e| io(dir, e))?;
    let link = |from: &Path, name: &str| {
        let at = mounted.join(name);
        std::os::unix::fs::symlink(from, &at).map_err(|e| io(&at, e))
    };
    for k in ["core", "alloc", "std"] {
        link(&library.join(k), k)?;
    }
    link(&dir, krate)?;
    let mut out = Vec::new();
    for (lib, source) in builds_on {
        if lib != krate && !mounted.join(lib).exists() {
            link(source, lib)?;
            out.push(lib.clone());
        }
    }
    Ok(out)
}

/// What [`embed_workspace_crate`] embeds and where.
#[derive(Clone, Debug)]
pub struct WorkspaceEmbed<'a> {
    /// The workspace root `uorust mir` dumped.
    pub workspace: &'a Path,
    /// The library crate, by its crate name.
    pub krate: &'a str,
    /// `uorust mir`'s output: the crate's shards are its `shards-<crate>`.
    pub dumps: &'a Path,
    /// The rust-src `library` directory of the pinned toolchain.
    pub library: &'a Path,
    /// The output directory: the project in its [`PROJECT`], the
    /// configuration and the record beside it; with `each_module`, one
    /// such directory per module under it.
    pub out: &'a Path,
    /// Embed only these modules into one project.
    pub only: &'a [String],
    /// One project per module, `out/<module>`, smallest module first.
    pub each_module: bool,
    /// The libraries linked into the program, `(crate, dumps)`, each
    /// after those it builds on: another library of the workspace (its
    /// `shards-<lib>`), a registry crate `uorust crate --workspace` kept
    /// ([`member_links`]), core or alloc. A call into one names its body.
    pub link: &'a [(String, PathBuf)],
}

/// One project's figures: the module it holds (none for the whole crate
/// or an `--only` set) and its census.
#[derive(Clone, Debug)]
pub struct Embedded {
    /// The module, with `each_module`.
    pub module: Option<Module>,
    /// The project's census.
    pub census: crate::embed::EmbedCensus,
}

/// Embed a library crate of a workspace from the shards `uorust mir` wrote
/// for it: its declarations and those of the crates it builds on (as cargo
/// resolves them in the dump's configuration) are mounted beside core,
/// alloc and std, and the crate is embedded as one project, or, with
/// `each_module`, as one project per module, each linked against every
/// module the shards hold, as CI embeds core. Each project is written to
/// `<out>/`[`PROJECT`]; the dump's configuration and the [`RECORD`] are
/// written to `<out>`, outside the project.
pub fn embed_workspace_crate(e: &WorkspaceEmbed<'_>) -> Result<Vec<Embedded>, UorustError> {
    let shards = e.dumps.join(format!("{SHARDS}{}", e.krate));
    if !shards.join(TARGET).is_file() {
        return Err(tool(
            "uorust embed",
            format!(
                "{} holds no shards of `{}`: dump the workspace with `uorust mir` first",
                e.dumps.display(),
                e.krate
            ),
        ));
    }
    let configuration = std::fs::read_to_string(shards.join(CONFIGURATION)).ok();
    let config = read_configuration(configuration.as_deref());
    let meta = output(
        "cargo metadata",
        Command::new("cargo")
            .args(["metadata", "-q", "--no-deps", "--format-version", "1"])
            .current_dir(e.workspace),
    )?;
    let meta: serde_json::Value = serde_json::from_str(&meta).map_err(|x| tool("metadata", x))?;
    let package = meta
        .get("packages")
        .and_then(|p| p.as_array())
        .into_iter()
        .flatten()
        .find(|p| library_target(p).is_some_and(|l| l.replace('-', "_") == e.krate))
        .ok_or_else(|| {
            tool(
                "uorust embed",
                format!("no library `{}` in the workspace", e.krate),
            )
        })?;
    let name = package
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or_default();
    let dir = package
        .get("manifest_path")
        .and_then(|m| m.as_str())
        .map(PathBuf::from)
        .and_then(|m| m.parent().map(Path::to_path_buf))
        .ok_or_else(|| tool("cargo metadata", "a package without a manifest path"))?;
    // The shards record the package's own features; cargo's resolve of a
    // workspace names a member's feature `package/feature`.
    let resolve = Configuration {
        features: config
            .features
            .iter()
            .map(|f| format!("{name}/{f}"))
            .collect(),
        ..config.clone()
    };
    let deps = dependencies(e.workspace, &resolve.cargo_args(true), Some(name))?;
    // One mount per call: two embeddings of one crate in one process
    // (tests run in parallel) never share it.
    static CALL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let call = CALL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "uorust-embed-{}-{}-{call}",
        e.krate,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let result = (|| {
        let builds_on = mount(&root, e.library, e.krate, &dir, &deps)?;
        let library = root.join("library");
        let link: Vec<(&str, &Path)> = e
            .link
            .iter()
            .map(|(k, d)| (k.as_str(), d.as_path()))
            .collect();
        let all = modules(&shards)?;
        let run = |only: &[String], out: &Path| {
            let dir = out.join(PROJECT);
            let census = crate::embed::embed_with(
                e.krate,
                &builds_on,
                &library,
                &shards,
                &dir,
                only,
                &crate::embed::Outputs {
                    link: &link,
                    ..crate::embed::Outputs::default()
                },
            )?;
            // The project as the embedding wrote it; the configuration
            // and the record lie beside it, outside what it names.
            let project = project_digest(&dir)?;
            if let Some(text) = &configuration {
                let file = out.join(CONFIGURATION);
                std::fs::write(&file, text).map_err(|x| io(&file, x))?;
            }
            let held: Vec<&Module> = all
                .iter()
                .filter(|m| only.is_empty() || only.contains(&m.name))
                .collect();
            // How the embedded modules' bodies hold each type.
            let mut holdings = std::collections::BTreeMap::new();
            for m in &held {
                let dir = shards.join(&m.name);
                let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
                    .map_err(|x| io(&dir, x))?
                    .filter_map(Result::ok)
                    .map(|f| f.path())
                    .filter(|f| f.extension().is_some_and(|x| x == "mir"))
                    .collect();
                files.sort();
                for f in files {
                    let text = std::fs::read_to_string(&f).map_err(|x| io(&f, x))?;
                    crate::ownership::read(&text, &mut holdings);
                }
            }
            let file = out.join(RECORD);
            let mut text = render_record(
                e.krate, &config, only, &held, &all, &link, &census, &project,
            );
            text.push_str(&crate::ownership::render(&holdings));
            std::fs::write(&file, text).map_err(|x| io(&file, x))?;
            Ok::<_, UorustError>(census)
        };
        if !e.each_module {
            return Ok(vec![Embedded {
                module: None,
                census: run(e.only, e.out)?,
            }]);
        }
        let mut done = Vec::new();
        for m in all.iter().cloned() {
            let census = run(std::slice::from_ref(&m.name), &e.out.join(&m.name))?;
            done.push(Embedded {
                module: Some(m),
                census,
            });
        }
        Ok(done)
    })();
    let _ = std::fs::remove_dir_all(&root);
    result
}

/// The SHA-256 of a project directory's files: each file's path under
/// `dir` and its bytes, in path order, every file as the embedding wrote
/// it. It names the project a verification of it is a receipt of.
pub fn project_digest(dir: &Path) -> Result<String, UorustError> {
    use sha2::{Digest, Sha256};
    let mut files: Vec<PathBuf> = Vec::new();
    let mut next = vec![dir.to_path_buf()];
    while let Some(d) = next.pop() {
        for entry in std::fs::read_dir(&d).map_err(|x| io(&d, x))? {
            let path = entry.map_err(|x| io(&d, x))?.path();
            if path.is_dir() {
                next.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    let mut hash = Sha256::new();
    for f in files {
        let name = f
            .strip_prefix(dir)
            .unwrap_or(&f)
            .to_string_lossy()
            .into_owned();
        let bytes = std::fs::read(&f).map_err(|x| io(&f, x))?;
        hash.update(name.as_bytes());
        hash.update([0u8]);
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(&bytes);
    }
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// [`RECORD`] of one embedding of the modules `only` (every module when
/// empty) of `krate`, dumped in `config`, into the project whose digest is
/// `project` ([`project_digest`]): the configuration, the modules and
/// their dump files, every module of the crate (`every`), the libraries linked, then the census's counts:
/// bodies embedded, evaluable, leaving a callee to a dictionary, and the
/// calls that name a body, by where it is, and those left to a dictionary,
/// by why ([`crate::embed::unlinked_reason`]); then one `[[body]]` row per
/// body ([`crate::embed::BodyFact`]): the path the dump prints, its κ, its
/// term's digest, what blocks it, its reach and the callees it leaves
/// unlinked, each with why.
#[allow(clippy::too_many_arguments)]
pub fn render_record(
    krate: &str,
    config: &Configuration,
    only: &[String],
    held: &[&Module],
    every: &[Module],
    link: &[(&str, &Path)],
    census: &crate::embed::EmbedCensus,
    project: &str,
) -> String {
    let quote = |s: &str| toml::Value::String(s.to_string()).to_string();
    let list =
        |items: &mut dyn Iterator<Item = &str>| items.map(quote).collect::<Vec<_>>().join(", ");
    let mut out = format!(
        "# One embedding of a workspace library's modules in one configuration\n\
         # (written by `uorust embed --workspace`; do not edit).\n\n\
         crate = {}\nmodules = [{}]\nfeatures = [{}]\ndefault_features = {}\nall_features = {}\n\
         target = {}\nlinked_libraries = [{}]\ndump_files = {}\ndump_bytes = {}\ncrate_modules = [{}]\nproject = {}\n\
         embedded = {}\nevaluable = {}\nbound = {}\ncalls_linked = {}\ncalls_unlinked = {}\n\
         closed = {}\n",
        quote(krate),
        list(&mut only.iter().map(String::as_str)),
        list(&mut config.features.iter().map(String::as_str)),
        !config.no_default_features,
        config.all_features,
        quote(config.target.as_deref().unwrap_or_default()),
        list(&mut link.iter().map(|(k, _)| *k)),
        held.iter().map(|m| m.files).sum::<usize>(),
        held.iter().map(|m| m.bytes).sum::<u64>(),
        list(&mut every.iter().map(|m| m.name.as_str())),
        quote(project),
        census.bodies,
        census.evaluable,
        census.bound,
        census.linked.values().sum::<usize>(),
        census.unlinked.values().sum::<usize>(),
        census.rows.iter().filter(|r| r.open.is_none()).count(),
    );
    for (at, n) in &census.linked {
        out.push_str(&format!(
            "\n[[linked]]\ninto = {}\ncalls = {n}\n",
            quote(at)
        ));
    }
    for (why, n) in &census.unlinked {
        out.push_str(&format!(
            "\n[[unlinked]]\nreason = {}\ncalls = {n}\n",
            quote(why)
        ));
    }
    for r in &census.rows {
        let blocked: Vec<String> = r.blocked.iter().map(|b| quote(b)).collect();
        let unlinked: Vec<String> = r
            .unlinked
            .iter()
            .flat_map(|(why, callees)| {
                callees
                    .iter()
                    .map(move |c| format!("{{ reason = {}, callee = {} }}", quote(why), quote(c)))
            })
            .collect();
        out.push_str(&format!(
            "\n[[body]]\npath = {}\nkappa = {}\nembedded = {}\nblocked = [{}]\nreach = {}\nunlinked = [{}]\n",
            quote(&r.path),
            quote(&r.kappa),
            quote(&r.embedded),
            blocked.join(", "),
            quote(&r.reach()),
            unlinked.join(", ")
        ));
    }
    out
}

/// The `--link` list `uorust crate --workspace` wrote for the member
/// target `krate` into its index ([`crate::registry::LINKS`]) under
/// `dumps`, each library's dumps resolved against that directory: the
/// registry crates its build links, each after those it builds on. A
/// target the index does not name links none.
pub fn member_links(dumps: &Path, krate: &str) -> Result<Vec<(String, PathBuf)>, UorustError> {
    let file = dumps.join(crate::registry::LINKS);
    let text = std::fs::read_to_string(&file).map_err(|x| io(&file, x))?;
    let doc: toml::Table = text.parse().map_err(|x| UorustError::Malformed {
        path: file.clone(),
        reason: format!("{x}"),
    })?;
    let member = doc
        .get("member")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .find(|m| m.get("target").and_then(toml::Value::as_str) == Some(krate));
    Ok(member
        .and_then(|m| m.get("link"))
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .filter_map(|l| l.split_once('='))
        .map(|(k, d)| (k.to_string(), dumps.join(d)))
        .collect())
}

/// The configuration [`Configuration::render`] wrote, read back for cargo's
/// resolve; the default where there is none.
fn read_configuration(text: Option<&str>) -> Configuration {
    let Some(doc) = text.and_then(|t| t.parse::<toml::Table>().ok()) else {
        return Configuration::default();
    };
    Configuration {
        features: doc
            .get("features")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|f| f.as_str().map(str::to_string))
            .collect(),
        no_default_features: doc
            .get("default_features")
            .and_then(toml::Value::as_bool)
            .is_some_and(|d| !d),
        all_features: doc
            .get("all_features")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false),
        target: doc
            .get("target")
            .and_then(toml::Value::as_str)
            .map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    // embed: the shard layout of a workspace crate and its configuration.

    #[test]
    fn embed_layout_drops_the_crates_own_directory_where_a_path_starts() {
        let text = "fn f() {\n    // scope 0 at member/src/a.rs:1:1: 1:2\n    let _1: {closure@member/src/a.rs:2:3: 2:4};\n    // at othermember/src/b.rs:1:1: 1:2 and /x/member/c.rs\n}\n";
        let out = without_prefix(text, "member/");
        assert!(out.contains("at src/a.rs:1:1"), "{out}");
        assert!(out.contains("{closure@src/a.rs:2:3"), "{out}");
        assert!(out.contains("othermember/src/b.rs"), "{out}");
        assert!(out.contains("/x/member/c.rs"), "{out}");
        assert_eq!(without_prefix(text, ""), text);
    }

    #[test]
    fn embed_layout_files_each_dump_under_its_module_with_its_size() {
        let work = std::env::temp_dir().join(format!("uorust-shard-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&work);
        let mir = work.join("lib-m");
        std::fs::create_dir_all(&mir).unwrap();
        let dump = |name: &str, text: &str| std::fs::write(mir.join(name), text).unwrap();
        dump(
            "m.a-f.2-2-009.ElaborateBoxDerefs.before.mir",
            "fn a::f() -> () { // at member/src/a.rs:1:1: 1:2\n}\n",
        );
        dump(
            "m.g.2-2-009.ElaborateBoxDerefs.before.mir",
            "fn g() -> () {\n}\n",
        );
        dump("m.g.2-2-009.Other.after.mir", "not read");
        let shards = work.join("shards-m");
        assert_eq!(lay_out(&mir, &shards, "member/", "/sysroot").unwrap(), 2);
        let a =
            std::fs::read_to_string(shards.join("a/m.a-f.2-2-009.ElaborateBoxDerefs.before.mir"))
                .unwrap();
        assert!(a.contains("at src/a.rs"), "{a}");
        assert!(shards
            .join(crate::registry::ROOT_MODULE)
            .join("m.g.2-2-009.ElaborateBoxDerefs.before.mir")
            .is_file());
        let found = write_modules(&shards).unwrap();
        let names: Vec<&str> = found.iter().map(|m| m.name.as_str()).collect();
        // Smallest first: the root module's one body is the shorter text.
        assert_eq!(names, [crate::registry::ROOT_MODULE, "a"]);
        assert!(found.iter().all(|m| m.files == 1));
        let listed = std::fs::read_to_string(shards.join(MODULES)).unwrap();
        assert!(listed.contains("name = \"a\"\nfiles = 1\n"), "{listed}");
        std::fs::remove_dir_all(&work).unwrap();
    }

    #[test]
    fn embed_configuration_selects_and_records_its_features_and_target() {
        let c = Configuration {
            features: vec!["x".into(), "y".into()],
            no_default_features: true,
            all_features: false,
            target: Some("wasm32-unknown-unknown".into()),
        };
        assert_eq!(
            c.cargo_args(false),
            [
                "--features",
                "x,y",
                "--no-default-features",
                "--target",
                "wasm32-unknown-unknown"
            ]
        );
        assert_eq!(c.cargo_args(true)[3], "--filter-platform");
        let text = c.render("wasm32-unknown-unknown");
        assert!(text.contains("features = [\"x\", \"y\"]\ndefault_features = false\n"));
        assert_eq!(read_configuration(Some(&text)), c);
        assert_eq!(read_configuration(None), Configuration::default());
        assert!(Configuration::default().cargo_args(false).is_empty());
        // Each package is built with the features it declares, and with
        // those named as its own.
        let asked = Configuration {
            features: vec!["x".into(), "b/y".into(), "c/z".into()],
            ..Configuration::default()
        };
        let declares = |f: &[&str]| f.iter().map(|f| f.to_string()).collect();
        assert_eq!(asked.for_package("a", &declares(&["x"])).features, ["x"]);
        assert_eq!(
            asked.for_package("b", &declares(&["x", "y"])).features,
            ["x", "y"]
        );
        assert!(asked.for_package("d", &declares(&[])).features.is_empty());
    }

    #[test]
    fn embed_a_workspace_crate_is_one_project_per_module() {
        let work = std::env::temp_dir().join(format!("uorust-shard-ws-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&work);
        let ws = work.join("ws");
        let w = |p: &str, t: &str| {
            let p = ws.join(p);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, t).unwrap();
        };
        w(
            "Cargo.toml",
            "[workspace]\nmembers = [\"m\"]\nresolver = \"2\"\n",
        );
        w(
            "m/Cargo.toml",
            "[package]\nname = \"m\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        w("m/src/lib.rs", "pub mod inner;\npub fn f() {}\n");
        w("m/src/inner.rs", "pub fn g(b: bool) {}\n");
        let dumps = work.join("dumps");
        // As `uorust mir` lays a library's dumps out.
        let body = |path: &str, terminator: &str| {
            format!("// MIR for `{path}` before ElaborateBoxDerefs\n\nfn {path}(_1: bool) -> () {{\n    let mut _0: ();\n\n    bb0: {{\n        {terminator}\n    }}\n\n    bb1: {{\n        return;\n    }}\n}}\n")
        };
        let shards = dumps.join(format!("{SHARDS}m"));
        for (module, name, text) in [
            (
                crate::registry::ROOT_MODULE,
                "m.f.2-2-009.ElaborateBoxDerefs.before.mir",
                body("f", "goto -> bb1;"),
            ),
            (
                "inner",
                "m.inner-g.2-2-009.ElaborateBoxDerefs.before.mir",
                body("inner::g", "switchInt(copy _1) -> bb1;"),
            ),
        ] {
            std::fs::create_dir_all(shards.join(module)).unwrap();
            std::fs::write(shards.join(module).join(name), text).unwrap();
        }
        std::fs::write(
            shards.join(TARGET),
            "triple = \"x86_64-unknown-linux-gnu\"\npointer_width = 64\n",
        )
        .unwrap();
        let recorded = Configuration::default().render("x86_64-unknown-linux-gnu");
        std::fs::write(shards.join(CONFIGURATION), &recorded).unwrap();
        let library = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/core/library");
        let out = work.join("out");
        let e = WorkspaceEmbed {
            workspace: &ws,
            krate: "m",
            dumps: &dumps,
            library: &library,
            out: &out,
            only: &[],
            each_module: true,
            link: &[],
        };
        let done = embed_workspace_crate(&e).expect("the embedding");
        let modules: Vec<&str> = done
            .iter()
            .filter_map(|d| d.module.as_ref().map(|m| m.name.as_str()))
            .collect();
        assert_eq!(modules.len(), 2);
        assert!(modules.contains(&"inner") && modules.contains(&crate::registry::ROOT_MODULE));
        for d in &done {
            assert_eq!(d.census.bodies, 1);
            assert_eq!(d.census.evaluable, 1, "{:?}", d.census.blocked);
            let out = out.join(&d.module.as_ref().unwrap().name);
            assert!(out.join(PROJECT).join("census.toml").is_file());
            assert!(!out.join(PROJECT).join(CONFIGURATION).exists());
            assert_eq!(
                std::fs::read_to_string(out.join(CONFIGURATION)).unwrap(),
                recorded
            );
        }
        // The same dumps in another configuration: each module's project
        // has the digest it has in the first, its record another label.
        let recorded_of = |out: &Path, module: &str| -> toml::Table {
            std::fs::read_to_string(out.join(module).join(RECORD))
                .unwrap()
                .parse()
                .unwrap()
        };
        let all = Configuration {
            all_features: true,
            ..Configuration::default()
        }
        .render("x86_64-unknown-linux-gnu");
        std::fs::write(shards.join(CONFIGURATION), &all).unwrap();
        let other = work.join("other");
        embed_workspace_crate(&WorkspaceEmbed {
            out: &other,
            ..e.clone()
        })
        .expect("the embedding in the other configuration");
        for module in ["inner", crate::registry::ROOT_MODULE] {
            let (a, b) = (recorded_of(&out, module), recorded_of(&other, module));
            assert_eq!(a.get("project"), b.get("project"), "{module}");
            assert_ne!(a.get("all_features"), b.get("all_features"), "{module}");
        }
        std::fs::write(shards.join(CONFIGURATION), &recorded).unwrap();
        // One project of the whole crate.
        let whole = embed_workspace_crate(&WorkspaceEmbed {
            each_module: false,
            ..e.clone()
        })
        .expect("the embedding");
        assert_eq!(whole.len(), 1);
        assert_eq!(whole[0].census.bodies, 2);
        // A crate `uorust mir` laid out no shards for is refused by name.
        let refused = embed_workspace_crate(&WorkspaceEmbed { krate: "n", ..e });
        assert!(
            matches!(&refused, Err(UorustError::Tool(t)) if t.contains("holds no shards of `n`")),
            "{refused:?}"
        );
        std::fs::remove_dir_all(&work).unwrap();
    }

    // join: one module's embedding in one configuration, linked against
    // the crate's other modules and a library its build links, recorded.

    /// A dump of `path(_1: u32, _2: u32) -> u32` whose entry block is
    /// `entry`, as `uorust mir` lays one out.
    fn join_body(path: &str, entry: &str) -> String {
        format!("// MIR for `{path}` before ElaborateBoxDerefs\n\nfn {path}(_1: u32, _2: u32) -> u32 {{\n    let mut _0: u32;\n\n    bb0: {{\n        {entry}\n    }}\n\n    bb1: {{\n        return;\n    }}\n}}\n")
    }

    fn join_write(dir: &Path, name: &str, text: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(name), text).unwrap();
    }

    #[test]
    fn join_a_module_links_its_crate_and_a_kept_library_and_records_both() {
        let work = std::env::temp_dir().join(format!("uorust-shard-join-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&work);
        // A library outside the workspace that the member builds on, and
        // its dumps as `uorust crate --dumps` keeps them.
        join_write(
            &work.join("dep"),
            "Cargo.toml",
            "[package]\nname = \"dep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
        );
        join_write(
            &work.join("dep/src"),
            "lib.rs",
            "pub fn max(a: u32, b: u32) -> u32 {\n    a\n}\n",
        );
        let kept = work.join("kept");
        join_write(
            &kept.join("dep-0.1.0").join(crate::registry::ROOT_MODULE),
            "dep.max.2-2-009.ElaborateBoxDerefs.before.mir",
            &join_body("max", "_0 = copy _1;\n        goto -> bb1;"),
        );
        let target = "triple = \"x86_64-unknown-linux-gnu\"\npointer_width = 64\n";
        join_write(&kept.join("dep-0.1.0"), TARGET, target);
        join_write(
            &kept,
            crate::registry::LINKS,
            "[[member]]\npackage = \"m\"\ntarget = \"m\"\nlink = [\"dep=dep-0.1.0\"]\n",
        );
        let ws = work.join("ws");
        join_write(
            &ws,
            "Cargo.toml",
            "[workspace]\nmembers = [\"m\"]\nresolver = \"2\"\n",
        );
        join_write(
            &ws.join("m"),
            "Cargo.toml",
            &format!(
                "[package]\nname = \"m\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
                 [features]\nwide = []\n\n[dependencies]\ndep = {{ path = \"{}\" }}\n",
                work.join("dep").display()
            ),
        );
        join_write(
            &ws.join("m/src"),
            "lib.rs",
            "pub mod inner;\npub fn f(a: u32, b: u32) -> u32 {\n    inner::top(a, b)\n}\n",
        );
        join_write(
            &ws.join("m/src"),
            "inner.rs",
            "pub fn top(a: u32, b: u32) -> u32 {\n    dep::max(a, b)\n}\n",
        );
        let dumps = work.join("dumps");
        let shards = dumps.join(format!("{SHARDS}m"));
        join_write(
            &shards.join(crate::registry::ROOT_MODULE),
            "m.f.2-2-009.ElaborateBoxDerefs.before.mir",
            &join_body(
                "f",
                "_0 = inner::top(copy _1, copy _2) -> [return: bb1, unwind continue];",
            ),
        );
        join_write(
            &shards.join("inner"),
            "m.inner-top.2-2-009.ElaborateBoxDerefs.before.mir",
            &join_body(
                "inner::top",
                "_0 = dep::max(copy _1, copy _2) -> [return: bb1, unwind continue];",
            ),
        );
        join_write(&shards, TARGET, target);
        // The second configuration: a feature of the member on.
        let config = Configuration {
            features: vec!["wide".into()],
            ..Configuration::default()
        };
        join_write(
            &shards,
            CONFIGURATION,
            &config.render("x86_64-unknown-linux-gnu"),
        );
        let library = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/core/library");
        let linked = member_links(&kept, "m").unwrap();
        assert_eq!(linked, [("dep".to_string(), kept.join("dep-0.1.0"))]);
        assert!(member_links(&kept, "other").unwrap().is_empty());
        let embed = |out: &str, link: &[(String, PathBuf)]| {
            let out = work.join(out);
            let done = embed_workspace_crate(&WorkspaceEmbed {
                workspace: &ws,
                krate: "m",
                dumps: &dumps,
                library: &library,
                out: &out,
                only: &["inner".to_string()],
                each_module: false,
                link,
            })
            .expect("the embedding");
            let record = std::fs::read_to_string(out.join(RECORD)).expect("the record");
            (done[0].census.clone(), record)
        };
        // Unlinked, the call into the library is left to a dictionary,
        // named by why.
        let (alone, record) = embed("alone", &[]);
        assert_eq!((alone.bodies, alone.bound), (1, 1), "{record}");
        assert_eq!(
            alone.unlinked.get("crate `dep` is not linked"),
            Some(&1),
            "{record}"
        );
        assert!(
            record.contains(
                "modules = [\"inner\"]\nfeatures = [\"wide\"]\ndefault_features = true\n"
            ),
            "{record}"
        );
        assert!(record.contains("calls_unlinked = 1\n"), "{record}");
        // Linked, it names the library's body.
        let (both, record) = embed("linked", &linked);
        assert_eq!((both.bodies, both.bound), (1, 0), "{record}");
        assert_eq!(both.linked.get("dep"), Some(&1), "{record}");
        assert!(both.unlinked.is_empty(), "{record}");
        assert!(
            record.contains("linked_libraries = [\"dep\"]\n")
                && record.contains("[[linked]]\ninto = \"dep\"\ncalls = 1\n"),
            "{record}"
        );
        // The root module's call into the crate's other module is linked
        // as the crate's own.
        let out = work.join("each");
        let done = embed_workspace_crate(&WorkspaceEmbed {
            workspace: &ws,
            krate: "m",
            dumps: &dumps,
            library: &library,
            out: &out,
            only: &[],
            each_module: true,
            link: &linked,
        })
        .expect("the embedding");
        let root = done
            .iter()
            .find(|d| d.module.as_ref().unwrap().name == crate::registry::ROOT_MODULE)
            .unwrap();
        assert_eq!(
            root.census.linked.get(crate::embed::OWN),
            Some(&1),
            "{:?}",
            root.census
        );
        assert!(out
            .join(crate::registry::ROOT_MODULE)
            .join(RECORD)
            .is_file());
        std::fs::remove_dir_all(&work).unwrap();
    }

    #[test]
    fn join_an_unlinked_call_is_named_by_the_crate_its_path_names() {
        let reason = |n: &str| crate::embed::unlinked_reason(n, "m", &["fnv"]);
        assert_eq!(
            reason("core::num::<impl u32>::max"),
            "core, alloc and std are not linked"
        );
        assert_eq!(
            reason("<alloc::vec::Vec<u8> as core::clone::Clone>::clone"),
            "core, alloc and std are not linked"
        );
        assert_eq!(
            reason("<u32 as std::ops::Add>::add"),
            "core, alloc and std are not linked"
        );
        assert_eq!(reason("<T as Trait>::go"), "a generic parameter's bound");
        assert_eq!(reason("<&T as Trait>::go"), "a generic parameter's bound");
        assert_eq!(
            reason("bit_set::BitSet::new"),
            "crate `bit_set` is not linked"
        );
        assert_eq!(
            reason("<fnv::FnvHasher as core::hash::Hasher>::write"),
            "a linked library leaves it to a dictionary"
        );
        assert_eq!(
            reason("<u8 as other::Trait>::go"),
            "crate `other` is not linked"
        );
        assert_eq!(
            reason("m::inner::gone"),
            "the crate's own, with no body in the dump"
        );
        assert_eq!(
            reason("sqrtf64"),
            "the platform (an intrinsic, assembly, a foreign item)"
        );
    }

    // lift: each body of a module's embedding is recorded by the path the
    // dump prints, with its reach and the callees it leaves unlinked.

    /// The source of a synthetic crate `m` whose dump holds a trait's
    /// method in two impls of one trait, a generic impl's method and an
    /// inline module's function of a name the root module also declares,
    /// and the dumps `uorust mir` lays out for it, under `work`.
    fn lift_crate(work: &Path) -> (PathBuf, PathBuf) {
        let ws = work.join("ws");
        join_write(
            &ws,
            "Cargo.toml",
            "[workspace]\nmembers = [\"m\"]\nresolver = \"2\"\n",
        );
        join_write(
            &ws.join("m"),
            "Cargo.toml",
            "[package]\nname = \"m\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        join_write(
            &ws.join("m/src"),
            "lib.rs",
            "pub mod inner {\n    pub fn go(a: u32, b: u32) -> u32 { a }\n    pub fn run(a: u32, b: u32) -> u32 { go(a, b) }\n}\n\
             pub trait Shape { fn f(&self, b: u32) -> u32; }\npub struct Sq(pub u32);\npub struct Ci(pub u32);\n\
             impl Shape for Sq { fn f(&self, b: u32) -> u32 { self.0 } }\n\
             impl Shape for Ci { fn f(&self, b: u32) -> u32 { b } }\n\
             pub struct Wrap<T>(pub T);\nimpl<T: Copy> Wrap<T> { pub fn get(&self) -> T { self.0 } }\n\
             pub fn go(a: u32, b: u32) -> u32 { a.wrapping_add(b) }\n",
        );
        let dumps = work.join("dumps");
        let shards = dumps.join(format!("{SHARDS}m"));
        let root = shards.join(crate::registry::ROOT_MODULE);
        let call = |callee: &str| {
            format!("_0 = {callee}(copy _1, copy _2) -> [return: bb1, unwind continue];")
        };
        for (dir, name, text) in [
            (&root, "m.{impl#0}-f", join_body("<impl at src/lib.rs:8:1: 8:18>::f", "_0 = copy _2;\n        goto -> bb1;")),
            (&root, "m.{impl#1}-f", join_body("<impl at src/lib.rs:9:1: 9:18>::f", "_0 = copy _2;\n        goto -> bb1;")),
            (
                &root,
                "m.{impl#2}-get",
                "// MIR for `<impl at src/lib.rs:11:1: 11:22>::get` before ElaborateBoxDerefs\n\nfn <impl at src/lib.rs:11:1: 11:22>::get(_1: &Wrap<T>) -> T {\n    debug self => _1;\n    let mut _0: T;\n\n    bb0: {\n        _0 = copy ((*_1).0: T);\n        return;\n    }\n}\n".to_string(),
            ),
            (&root, "m.go", join_body("go", &call("core::num::<impl u32>::wrapping_add"))),
            (&root, "m.pick", join_body("pick", &call("<T as Shape>::f"))),
            // A drop of a place with drop glue, and a call of a value: each
            // reaches what no named body states.
            (&root, "m.defuse", join_body("defuse", "drop(_1) -> [return: bb1, unwind continue];")),
            (&root, "m.apply", join_body("apply", "_0 = copy _1(copy _2) -> [return: bb1, unwind continue];")),
            (&shards.join("inner"), "m.inner-go", join_body("inner::go", "_0 = copy _1;\n        goto -> bb1;")),
            (&shards.join("inner"), "m.inner-run", join_body("inner::run", &call("inner::go"))),
        ] {
            join_write(dir, &format!("{name}.2-2-009.ElaborateBoxDerefs.before.mir"), &text);
        }
        join_write(
            &shards,
            TARGET,
            "triple = \"x86_64-unknown-linux-gnu\"\npointer_width = 64\n",
        );
        join_write(
            &shards,
            CONFIGURATION,
            &Configuration::default().render("x86_64-unknown-linux-gnu"),
        );
        (ws, dumps)
    }

    /// The record of the whole crate `m` embedded into `out`.
    fn lift_record(work: &Path, out: &str) -> (crate::embed::EmbedCensus, toml::Table) {
        let (ws, dumps) = lift_crate(work);
        let library = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/core/library");
        let out = work.join(out);
        let done = embed_workspace_crate(&WorkspaceEmbed {
            workspace: &ws,
            krate: "m",
            dumps: &dumps,
            library: &library,
            out: &out,
            only: &[],
            each_module: false,
            link: &[],
        })
        .expect("the embedding");
        let record = std::fs::read_to_string(out.join(RECORD)).expect("the record");
        (
            done[0].census.clone(),
            record.parse().expect("a record is TOML"),
        )
    }

    #[test]
    fn lift_each_body_is_recorded_by_the_path_its_dump_prints_with_its_reach() {
        let work = std::env::temp_dir().join(format!("uorust-shard-lift-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&work);
        let (census, record) = lift_record(&work, "out");
        let bodies: BTreeMap<String, &toml::Value> = record["body"]
            .as_array()
            .expect("body rows")
            .iter()
            .map(|b| (b["path"].as_str().unwrap().to_string(), b))
            .collect();
        // One row per body, by the path the dump prints: the two impls of
        // one trait apart, the generic impl's method, and the inline
        // module's function apart from the root's of its name.
        let paths: Vec<&str> = bodies.keys().map(String::as_str).collect();
        assert_eq!(
            paths,
            [
                "<impl at src/lib.rs:11:1: 11:22>::get",
                "<impl at src/lib.rs:8:1: 8:18>::f",
                "<impl at src/lib.rs:9:1: 9:18>::f",
                "apply",
                "defuse",
                "go",
                "inner::go",
                "inner::run",
                "pick"
            ],
            "{record}"
        );
        assert_eq!(census.rows.len(), 9);
        // A drop runs drop glue the evaluator does not run, and a call of a
        // value names no body: both open at themselves.
        assert_eq!(
            bodies["defuse"]["reach"].as_str(),
            Some("open: drops a value, whose drop glue the evaluator does not run (at defuse)"),
            "{record}"
        );
        assert_eq!(
            bodies["apply"]["reach"].as_str(),
            Some("open: calls a function pointer or closure value, which names no body (at apply)"),
            "{record}"
        );
        let reach = |p: &str| bodies[p]["reach"].as_str().unwrap().to_string();
        // A body that calls a body of the crate reaches it: closed.
        assert_eq!(reach("inner::run"), "closed");
        assert_eq!(reach("<impl at src/lib.rs:8:1: 8:18>::f"), "closed");
        // A generic impl's method that reads its value whatever the type
        // reaches every body it calls; one that calls its parameter's
        // trait leaves the callee to the bound's dictionary, and a call
        // into core, which is not linked, leaves it to core's: each open
        // at itself, the callee named by why.
        assert_eq!(reach("<impl at src/lib.rs:11:1: 11:22>::get"), "closed");
        assert_eq!(
            reach("pick"),
            "open: leaves a callee of the crate to a dictionary (at pick)"
        );
        assert_eq!(
            bodies["pick"]["unlinked"][0]["reason"].as_str(),
            Some("a generic parameter's bound")
        );
        assert_eq!(
            reach("go"),
            "open: leaves a callee of core to a dictionary (at go)"
        );
        let unlinked = bodies["go"]["unlinked"].as_array().unwrap();
        assert_eq!(unlinked.len(), 1, "{record}");
        assert_eq!(
            unlinked[0]["reason"].as_str(),
            Some("core, alloc and std are not linked")
        );
        assert_eq!(
            unlinked[0]["callee"].as_str(),
            Some("core::num::<impl u32>::wrapping_add")
        );
        assert!(bodies["inner::run"]["unlinked"]
            .as_array()
            .unwrap()
            .is_empty());
        // Each row's digest is its term's, as the register's stamp writes
        // it, and its κ the body's; the counts agree with the rows.
        for r in &census.rows {
            assert_eq!(
                bodies[&r.path]["embedded"].as_str(),
                Some(r.embedded.as_str())
            );
            assert_eq!(bodies[&r.path]["kappa"].as_str(), Some(r.kappa.as_str()));
        }
        assert_eq!(record["closed"].as_integer(), Some(5), "{record}");
        assert_eq!(
            record["project"].as_str().map(str::len),
            Some(64),
            "{record}"
        );
        std::fs::remove_dir_all(&work).unwrap();
    }
}
