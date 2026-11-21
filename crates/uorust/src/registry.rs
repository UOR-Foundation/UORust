//! A crate of crates.io as a subject of the MIR path: `uorust crate
//! <name>@<version>` fetches the crate's archive, checks it against the
//! checksum crates.io records, dumps the MIR of its library on the pinned
//! toolchain, and writes its register under `crates/uorust/crates/<name>`,
//! the same register core's and std's are: a row per body by κ in its
//! module's file, the summary in `frontier.toml`, and no κ recorded as
//! translated may come back as residue.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

use crate::mir::core::{census, core_regressions, write_register, Scope};
use crate::UorustError;

/// The module a crate's own items, outside any module, are recorded in:
/// `crate` is a keyword, so no module of the crate has the name.
pub const ROOT_MODULE: &str = "crate";

/// What `uorust crate` did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lifted {
    /// The bodies embedded, those evaluable and those that leave a callee
    /// to a dictionary (call something not linked), where the crate was
    /// embedded too.
    pub embedding: Option<(usize, usize, usize)>,
    /// The bodies the differential check ran and the theorems `lexlean`
    /// verified of them, where it was asked.
    pub differential: Option<(usize, usize)>,
    /// The modules translated.
    pub modules: usize,
    /// The bodies of the crate's MIR.
    pub bodies: usize,
    /// The bodies that translate.
    pub translated: usize,
}

fn tool(what: &str, reason: impl std::fmt::Display) -> UorustError {
    UorustError::Tool(format!("{what}: {reason}"))
}

fn run(what: &str, command: &mut Command) -> Result<Vec<u8>, UorustError> {
    let out = command.output().map_err(|e| tool(what, e))?;
    if !out.status.success() {
        return Err(tool(what, String::from_utf8_lossy(&out.stderr)));
    }
    Ok(out.stdout)
}

/// The user agent crates.io asks automated clients to send.
const USER_AGENT: &str = "uorust-register";

/// `name@version` split, both non-empty.
pub fn parse_spec(spec: &str) -> Result<(String, String), UorustError> {
    match spec.split_once('@') {
        Some((n, v)) if !n.is_empty() && !v.is_empty() => Ok((n.to_string(), v.to_string())),
        _ => Err(tool(
            "uorust crate",
            format!("`{spec}` is not <name>@<version>"),
        )),
    }
}

/// The item path a dump's file name names, its segments joined by `-`:
/// `serde.de-{impl#3}-f.2-0.ElaborateBoxDerefs.before.mir` names
/// `de-{impl#3}-f`.
pub fn dump_item(file: &str) -> Option<&str> {
    file.split_once('.')?.1.split('.').next()
}

/// The module a dumped item is in, given every item the crate's dumps name:
/// its first segment when that is a module's name (snake case, as a type's
/// or an impl's is not) and names no body itself (a function's closures go
/// with the function); [`ROOT_MODULE`] otherwise.
pub fn dump_module(item: &str, items: &std::collections::BTreeSet<&str>) -> String {
    match item.split_once('-') {
        Some((first, _))
            if first.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
                && !items.contains(first) =>
        {
            first.to_string()
        }
        _ => ROOT_MODULE.to_string(),
    }
}

/// The directory cargo keeps its registries in: `CARGO_HOME`, else
/// `~/.cargo`, as cargo itself resolves it.
pub(crate) fn cargo_home() -> PathBuf {
    std::env::var_os("CARGO_HOME").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cargo"),
        PathBuf::from,
    )
}

/// The archive of `name@version` that cargo's own cache holds, from any
/// registry index it has fetched it through.
fn cached_archive(name: &str, version: &str) -> Option<PathBuf> {
    let cache = cargo_home().join("registry").join("cache");
    let file = format!("{name}-{version}.crate");
    let mut found: Vec<PathBuf> = std::fs::read_dir(cache)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path().join(&file))
        .filter(|p| p.is_file())
        .collect();
    found.sort();
    found.into_iter().next()
}

/// The SHA-256 of `bytes`, in lower-case hex, as crates.io and a lock file
/// record a package's checksum.
fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Fetch `name@version` into `work`, checked against a checksum, and
/// return its unpacked directory. With `locked`, the checksum a lock file
/// records for it, the archive is cargo's cached one where the cache holds
/// it (no network), and crates.io's otherwise; without, the checksum is the
/// one crates.io records. An archive of another digest is refused either
/// way, never replaced.
fn fetch(
    name: &str,
    version: &str,
    work: &Path,
    locked: Option<&str>,
) -> Result<PathBuf, UorustError> {
    let checksum = match locked {
        Some(c) => c.to_string(),
        None => {
            let meta = run(
                "crates.io",
                Command::new("curl").args([
                    "-sSfL",
                    "-A",
                    USER_AGENT,
                    &format!("https://crates.io/api/v1/crates/{name}/{version}"),
                ]),
            )?;
            let doc: serde_json::Value =
                serde_json::from_slice(&meta).map_err(|e| tool("crates.io", e))?;
            doc.pointer("/version/checksum")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| tool("crates.io", format!("{name}@{version} has no checksum")))?
                .to_string()
        }
    };
    let archive = work.join(format!("{name}-{version}.crate"));
    match locked.and_then(|_| cached_archive(name, version)) {
        Some(cached) => {
            std::fs::copy(&cached, &archive).map_err(|e| tool("cargo cache", e))?;
        }
        None => {
            run(
                "crates.io download",
                Command::new("curl").args([
                    "-sSfL",
                    "-A",
                    USER_AGENT,
                    "-o",
                    &archive.display().to_string(),
                    &format!("https://static.crates.io/crates/{name}/{name}-{version}.crate"),
                ]),
            )?;
        }
    }
    let bytes = std::fs::read(&archive).map_err(|e| tool("crates.io download", e))?;
    let digest = sha256_hex(&bytes);
    if digest != checksum {
        return Err(tool(
            "crates.io download",
            format!("{name}@{version} is {digest}, the checksum recorded is {checksum}"),
        ));
    }
    run(
        "tar",
        Command::new("tar")
            .arg("-xzf")
            .arg(&archive)
            .arg("-C")
            .arg(work),
    )?;
    let dir = work.join(format!("{name}-{version}"));
    // Its own workspace root, whatever directory it is unpacked in.
    let manifest = dir.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).map_err(|e| tool("Cargo.toml", e))?;
    if !text.lines().any(|l| l.trim() == "[workspace]") {
        std::fs::write(&manifest, text + "\n[workspace]\n").map_err(|e| tool("Cargo.toml", e))?;
    }
    Ok(dir)
}

/// The flags of cargo that select the features `options` names: the
/// crate's defaults left off where it says so, and the features named.
fn build_flags(options: &Options<'_>) -> Vec<String> {
    let mut flags = Vec::new();
    if options.no_default_features {
        flags.push("--no-default-features".to_string());
    }
    if !options.features.is_empty() {
        flags.push(format!("--features={}", options.features.join(",")));
    }
    flags
}

/// Dump the MIR of the library of the crate at `dir` on the pinned
/// toolchain, with this machine's paths resolved away, into one directory
/// per module under `shards`, its library built with the features
/// `options` selects ([`build_flags`]).
fn dump(dir: &Path, work: &Path, shards: &Path, options: &Options<'_>) -> Result<(), UorustError> {
    let toolchain = format!("+{}", crate::mir::spec::TOOLCHAIN);
    let mir = work.join("mir");
    std::fs::create_dir_all(&mir).map_err(|e| tool("mir", e))?;
    let selected = build_flags(options);
    // Only the library's own flags carry the dump: `cargo rustc` passes them
    // to the one target.
    run(
        "cargo rustc",
        Command::new("cargo")
            .args([toolchain.as_str(), "rustc", "--lib", "-q"])
            .args(&selected)
            .arg("--target-dir")
            .arg(work.join("target"))
            .arg("--")
            .arg("-Zdump-mir=ElaborateBoxDerefs")
            .arg(format!("-Zdump-mir-dir={}", mir.display()))
            .args(["-Coverflow-checks=on", "-Cdebug-assertions=off"])
            // A lint the crate denies is a newer toolchain's opinion, not
            // its build: cargo caps a registry crate's lints so when it is
            // a dependency (try-lock's `#![deny(warnings)]`).
            .arg("--cap-lints=allow")
            .current_dir(dir)
            // An inherited build directory would hold the crate built
            // already, and cargo would not run rustc to dump it again.
            .env_remove("CARGO_BUILD_BUILD_DIR")
            .env_remove("CARGO_BUILD_TARGET_DIR")
            .env_remove("CARGO_TARGET_DIR")
            .env("RUSTC_BOOTSTRAP", "1")
            .env("CARGO_INCREMENTAL", "0"),
    )?;
    let sysroot = crate::shard::sysroot(dir, Some(&toolchain))?;
    crate::shard::lay_out(&mir, shards, &format!("{}/", dir.display()), &sysroot)?;
    Ok(())
}

/// How `uorust crate` builds, embeds and keeps a crate.
#[derive(Clone, Copy, Debug, Default)]
pub struct Options<'a> {
    /// The rust-src `library` directory of the pinned toolchain: with it,
    /// the crate's bodies are embedded too.
    pub library: Option<&'a Path>,
    /// With `library`, check the embedding against rustc's build.
    pub differential: bool,
    /// Library crates to link into the crate's program, `(crate, dumps)`,
    /// those a linked crate calls before it.
    pub link: &'a [(String, PathBuf)],
    /// The features to build the crate with, besides its default ones
    /// unless `no_default_features`.
    pub features: &'a [String],
    /// Build without the crate's default features: with `features`, the
    /// exact set a consuming workspace's build enables (`default` among
    /// them where it does).
    pub no_default_features: bool,
    /// The checksum a consuming workspace's lock records for the crate: the
    /// archive must have it, and is cargo's cached one where it is cached.
    pub checksum: Option<&'a str>,
    /// The consuming workspace's lock file: the crate's own dependencies
    /// are resolved at the versions it locks.
    pub lock: Option<&'a Path>,
    /// With `library`, the directory the crate's module shards, its
    /// `target.toml`, its layouts and its `dump.toml` are kept in: what
    /// `--link <crate>=<dumps>` reads.
    pub dumps: Option<&'a Path>,
}

/// Fetch, dump and translate `name@version` and write its register to
/// `register`, refusing it when a body the register records as translated
/// is now residue. A module file the crate no longer has is removed.
pub fn lift_crate(
    name: &str,
    version: &str,
    register: &Path,
    options: &Options<'_>,
) -> Result<Lifted, UorustError> {
    let work = std::env::temp_dir().join(format!("uorust-crate-{name}-{version}"));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).map_err(|e| tool("work", e))?;
    let result = fetch(name, version, &work, options.checksum)
        .and_then(|dir| lift_source(name, version, &dir, &work, register, options));
    let _ = std::fs::remove_dir_all(&work);
    result
}

/// [`lift_crate`] of the crate whose package is unpacked at `dir`, its
/// work done under `work`.
fn lift_source(
    name: &str,
    version: &str,
    dir: &Path,
    work: &Path,
    register: &Path,
    options: &Options<'_>,
) -> Result<Lifted, UorustError> {
    if options.dumps.is_some() && options.library.is_none() {
        // The dumps a link reads carry the target the embedding states.
        return Err(tool(
            &format!("{name}@{version}"),
            "kept dumps need the library to embed against",
        ));
    }
    if let Some(lock) = options.lock {
        std::fs::copy(lock, dir.join("Cargo.lock")).map_err(|e| tool("Cargo.lock", e))?;
    }
    let shards = work.join("shards");
    std::fs::create_dir_all(&shards).map_err(|e| tool("mir", e))?;
    dump(dir, work, &shards, options)?;
    let scope = Scope::registry(name, version);
    let census = census(&scope, dir, &shards, &[])?;
    let regressed = core_regressions(register, &census);
    if !regressed.is_empty() {
        return Err(tool(
            &format!("{name}@{version}"),
            format!(
                "a body the register records as translated is now residue: {}",
                regressed.join("; ")
            ),
        ));
    }
    if register.exists() {
        for entry in std::fs::read_dir(register)
            .map_err(|e| tool("register", e))?
            .filter_map(Result::ok)
        {
            let p = entry.path();
            let stem = p.file_stem().and_then(|n| n.to_str()).unwrap_or_default();
            let kept = stem == "frontier"
                || stem == crate::queue::RETRACTED
                || census.iter().any(|s| s.module == stem);
            if p.extension().is_some_and(|x| x == "toml") && !kept {
                std::fs::remove_file(&p).map_err(|e| tool("register", e))?;
            }
        }
    }
    write_register(&scope, register, &census)?;
    let (embedding, checked) = match options.library {
        Some(library) => {
            let (e, d) = embed_crate(
                &scope,
                name,
                dir,
                work,
                &shards,
                library,
                register,
                options.differential,
                options.link,
            )?;
            (Some((e.bodies, e.evaluable, e.bound)), d)
        }
        None => (None, None),
    };
    let lifted = Lifted {
        embedding,
        differential: checked,
        modules: census.len(),
        bodies: census.iter().map(|s| s.rows.len()).sum(),
        translated: census.iter().map(|s| s.translated_count()).sum(),
    };
    if let Some(kept) = options.dumps {
        keep_dumps(
            &shards,
            kept,
            &Kept {
                name,
                version,
                options,
                lifted: &lifted,
            },
        )?;
    }
    Ok(lifted)
}

/// What a kept dump was built from, recorded beside it.
struct Kept<'a> {
    name: &'a str,
    version: &'a str,
    options: &'a Options<'a>,
    lifted: &'a Lifted,
}

/// The name of the file a kept dump records its build in.
pub const DUMP_RECORD: &str = "dump.toml";

/// Replace `kept` with the shards at `shards` (each module's dumps, the
/// target, the layouts), and record beside them the package, its checksum,
/// the features it was built with, the libraries it was linked with and
/// what its run counted, so that a link names what it links and a run of
/// the same build is known as done.
fn keep_dumps(shards: &Path, kept: &Path, what: &Kept<'_>) -> Result<(), UorustError> {
    let _ = std::fs::remove_dir_all(kept);
    copy_tree(shards, kept)?;
    let quoted = |s: &str| toml::Value::String(s.to_string()).to_string();
    let list = |items: &mut dyn Iterator<Item = String>| {
        items.map(|x| quoted(&x)).collect::<Vec<_>>().join(", ")
    };
    let mut record = format!(
        "# The build this crate's dumps are of (written by `uorust crate`; do not edit).\n\n\
         name = {}\nversion = {}\n",
        quoted(what.name),
        quoted(what.version)
    );
    if let Some(c) = what.options.checksum {
        record.push_str(&format!("checksum = {}\n", quoted(c)));
    }
    let l = what.lifted;
    record.push_str(&format!(
        "default-features = {}\nfeatures = [{}]\nlink = [{}]\n\n[counts]\n\
         modules = {}\nbodies = {}\ntranslated = {}\n",
        !what.options.no_default_features,
        list(&mut what.options.features.iter().cloned()),
        list(&mut link_args(what.options.link).into_iter()),
        l.modules,
        l.bodies,
        l.translated,
    ));
    if let Some((b, e, u)) = l.embedding {
        record.push_str(&format!("embedded = {b}\nevaluable = {e}\nbound = {u}\n"));
    }
    std::fs::write(kept.join(DUMP_RECORD), record).map_err(|e| tool(DUMP_RECORD, e))
}

/// Each link as `--link` takes it, `<crate>=<dumps>`.
fn link_args(link: &[(String, PathBuf)]) -> Vec<String> {
    link.iter()
        .map(|(k, d)| format!("{k}={}", d.display()))
        .collect()
}

/// What the record of the kept dumps at `kept` counted, when it records
/// the build `options` asks for: the same checksum, features and links. A
/// run of that build is done; any other record is not this build's.
pub fn recorded(kept: &Path, options: &Options<'_>) -> Option<Lifted> {
    let doc: toml::Table = std::fs::read_to_string(kept.join(DUMP_RECORD))
        .ok()?
        .parse()
        .ok()?;
    let strings = |k: &str| -> Option<Vec<String>> {
        doc.get(k)?
            .as_array()?
            .iter()
            .map(|v| v.as_str().map(str::to_string))
            .collect()
    };
    let same = doc.get("checksum").and_then(toml::Value::as_str) == options.checksum
        && doc.get("default-features").and_then(toml::Value::as_bool)
            == Some(!options.no_default_features)
        && strings("features")? == options.features
        && strings("link")? == link_args(options.link);
    let counts = doc.get("counts")?;
    let count = |k: &str| -> Option<usize> {
        counts
            .get(k)?
            .as_integer()
            .and_then(|n| usize::try_from(n).ok())
    };
    let embedding = match (count("embedded"), count("evaluable"), count("bound")) {
        (Some(b), Some(e), Some(u)) => Some((b, e, u)),
        _ => None,
    };
    same.then_some(Lifted {
        embedding,
        differential: None,
        modules: count("modules")?,
        bodies: count("bodies")?,
        translated: count("translated")?,
    })
}

/// A library of crates.io as a workspace's build links it: the package at
/// its locked version and checksum, its library's crate name, the features
/// the build enables for it, and the libraries of crates.io it builds on
/// (indices into the same list, each before it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Locked {
    /// The package's name.
    pub name: String,
    /// Its version, as the lock records it.
    pub version: String,
    /// Its library's crate name, as the code that calls it names it.
    pub lib: String,
    /// The features the workspace's build enables for it, exactly.
    pub features: Vec<String>,
    /// The checksum the lock records.
    pub checksum: String,
    /// The libraries of crates.io it builds on, directly.
    pub deps: Vec<usize>,
}

impl Locked {
    /// The directory, under a kept dumps root, its dumps are kept in: one
    /// per package version, since a build may link two versions of one
    /// crate.
    pub fn dir(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }
}

/// A workspace member's library or binary, with the libraries of crates.io
/// it calls into directly (indices into [`Graph::locked`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    /// The package's name.
    pub package: String,
    /// The target's crate name.
    pub target: String,
    /// The libraries of crates.io it builds on, directly.
    pub deps: Vec<usize>,
}

/// What a workspace's build links of crates.io.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Graph {
    /// Every library of crates.io the build compiles into the code it
    /// builds, those each builds on first.
    pub locked: Vec<Locked>,
    /// The members' targets the build builds.
    pub members: Vec<Member>,
}

impl Graph {
    /// The libraries `deps` reach, each after those it builds on: the
    /// `--link` list of a body that calls them, in the order the linker
    /// takes them (a linked library's own calls are bound by those before
    /// it).
    pub fn closure(&self, deps: &[usize]) -> Vec<usize> {
        let mut seen = BTreeSet::new();
        let mut next: Vec<usize> = deps.to_vec();
        while let Some(i) = next.pop() {
            if seen.insert(i) {
                next.extend(self.locked.get(i).into_iter().flat_map(|l| l.deps.iter()));
            }
        }
        // `locked` is in dependency order already.
        seen.into_iter().collect()
    }

    /// The `--link` list of a body that calls `deps`: the libraries
    /// `first` names (core, alloc), then those `deps` reach ([`closure`]),
    /// each by its kept dumps under `dumps`. A library name two of them
    /// share (two versions of one crate) is left out of the list: a call
    /// names a library by its crate name alone, and the linker takes a
    /// link by that name, so a call into either would be bound to the first
    /// one's body; such calls are left to the dictionary instead.
    ///
    /// [`closure`]: Graph::closure
    pub fn links(
        &self,
        deps: &[usize],
        first: &[(String, PathBuf)],
        dumps: &Path,
    ) -> Vec<(String, PathBuf)> {
        let all: Vec<(String, PathBuf)> = first
            .iter()
            .cloned()
            .chain(
                self.closure(deps)
                    .into_iter()
                    .map(|d| (self.locked[d].lib.clone(), dumps.join(self.locked[d].dir()))),
            )
            .collect();
        let mut named: BTreeMap<&str, usize> = BTreeMap::new();
        for (lib, _) in &all {
            *named.entry(lib.as_str()).or_default() += 1;
        }
        let shared: BTreeSet<String> = named
            .into_iter()
            .filter(|(_, n)| *n > 1)
            .map(|(lib, _)| lib.to_string())
            .collect();
        all.into_iter()
            .filter(|(lib, _)| !shared.contains(lib))
            .collect()
    }
}

/// The package a cargo package id names, `(source, name, version)`: a
/// specification (`registry+https://…#name@1.0.0`, a path's `path+file://…#0.1.0`
/// naming no name, which is the directory's) or the older
/// `name 1.0.0 (registry+https://…)`.
fn package_id(id: &str) -> Option<(String, String, String)> {
    if let Some((source, rest)) = id.split_once('#') {
        let (name, version) = match rest.split_once('@') {
            Some((n, v)) => (n.to_string(), v.to_string()),
            None => {
                let last = source.trim_end_matches('/').rsplit('/').next()?;
                (last.to_string(), rest.to_string())
            }
        };
        return Some((source.to_string(), name, version));
    }
    let mut parts = id.splitn(3, ' ');
    let name = parts.next()?.to_string();
    let version = parts.next()?.to_string();
    let source = parts
        .next()?
        .trim_matches(|c| c == '(' || c == ')')
        .to_string();
    Some((source, name, version))
}

/// What a workspace's build links of crates.io, read from cargo's unit
/// graph of that build (`cargo build --unit-graph`) and the workspace's
/// lock: from each root unit (a member's library or binary), every library
/// unit its compiled code depends on, through libraries only. A build
/// script's dependencies run on the host and are not linked into the code
/// built, and a procedural macro's expand the source, so neither is
/// followed: the features of a library are those of the unit the target
/// code links, which is what cargo's feature resolver decided for it.
/// A library of crates.io whose lock row has no checksum, or that two
/// units of the build link with different features, is refused: its
/// dumps would not be of one build.
pub fn graph(unit_graph: &str, lock: &str) -> Result<Graph, UorustError> {
    let bad = |reason: String| tool("unit graph", reason);
    let doc: serde_json::Value =
        serde_json::from_str(unit_graph).map_err(|e| bad(e.to_string()))?;
    let lock: toml::Table = lock.parse().map_err(|e| tool("Cargo.lock", e))?;
    let checksums: BTreeMap<(String, String), String> = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|p| {
            let s = |k: &str| p.get(k).and_then(toml::Value::as_str).map(str::to_string);
            Some(((s("name")?, s("version")?), s("checksum")?))
        })
        .collect();
    let units = doc
        .get("units")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| bad("no units".into()))?;
    let kinds = |u: &serde_json::Value| -> Vec<String> {
        u.pointer("/target/kind")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|k| k.as_str().map(str::to_string))
            .collect()
    };
    // A build script's kind is `custom-build` and a procedural macro's
    // `proc-macro`: neither is a library the built code links.
    let linked = |u: &serde_json::Value| {
        kinds(u)
            .iter()
            .any(|k| ["lib", "rlib", "dylib", "staticlib", "cdylib"].contains(&k.as_str()))
    };
    let deps_of = |u: &serde_json::Value| -> Vec<usize> {
        u.get("dependencies")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|d| {
                d.get("index")?
                    .as_u64()
                    .and_then(|i| usize::try_from(i).ok())
            })
            .filter(|i| units.get(*i).is_some_and(linked))
            .collect()
    };
    let roots: Vec<usize> = doc
        .get("roots")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|r| r.as_u64().and_then(|i| usize::try_from(i).ok()))
        .collect();
    // Every unit the roots' code links, each after those it builds on.
    let mut order: Vec<usize> = Vec::new();
    let mut state: BTreeMap<usize, bool> = BTreeMap::new();
    for &root in &roots {
        let mut stack: Vec<(usize, bool)> = vec![(root, false)];
        while let Some((i, done)) = stack.pop() {
            if done {
                state.insert(i, true);
                order.push(i);
                continue;
            }
            if state.contains_key(&i) {
                continue;
            }
            state.insert(i, false);
            stack.push((i, true));
            let u = units.get(i).ok_or_else(|| bad(format!("no unit {i}")))?;
            for d in deps_of(u).into_iter().rev() {
                if !state.contains_key(&d) {
                    stack.push((d, false));
                }
            }
        }
    }
    let registry_unit = |i: usize| -> Option<(String, String)> {
        let id = units.get(i)?.get("pkg_id")?.as_str()?;
        let (source, name, version) = package_id(id)?;
        source.starts_with("registry+").then_some((name, version))
    };
    let mut graph = Graph::default();
    let mut index: BTreeMap<usize, usize> = BTreeMap::new();
    let mut by_package: BTreeMap<(String, String), usize> = BTreeMap::new();
    let features_of = |u: &serde_json::Value| -> Vec<String> {
        let mut f: Vec<String> = u
            .get("features")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect();
        f.sort();
        f
    };
    for &i in &order {
        let Some((name, version)) = registry_unit(i) else {
            continue;
        };
        let u = &units[i];
        let features = features_of(u);
        let deps: Vec<usize> = deps_of(u)
            .into_iter()
            .filter_map(|d| index.get(&d).copied())
            .collect();
        let key = (name.clone(), version.clone());
        if let Some(&at) = by_package.get(&key) {
            let same: &Locked = &graph.locked[at];
            if same.features != features {
                return Err(bad(format!(
                    "{name}@{version} is linked with features [{}] and [{}]",
                    same.features.join(", "),
                    features.join(", ")
                )));
            }
            index.insert(i, at);
            continue;
        }
        let checksum = checksums
            .get(&key)
            .cloned()
            .ok_or_else(|| tool("Cargo.lock", format!("{name}@{version} has no checksum")))?;
        let lib = u
            .pointer("/target/name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(&name)
            .replace('-', "_");
        index.insert(i, graph.locked.len());
        by_package.insert(key, graph.locked.len());
        graph.locked.push(Locked {
            name,
            version,
            lib,
            features,
            checksum,
            deps,
        });
    }
    for &root in &roots {
        let u = &units[root];
        let (_, package, _) = u
            .get("pkg_id")
            .and_then(serde_json::Value::as_str)
            .and_then(package_id)
            .ok_or_else(|| bad(format!("unit {root} has no package")))?;
        let target = u
            .pointer("/target/name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(&package)
            .replace('-', "_");
        let mut deps: Vec<usize> = deps_of(u)
            .into_iter()
            .filter_map(|d| index.get(&d).copied())
            .collect();
        deps.sort_unstable();
        deps.dedup();
        graph.members.push(Member {
            package,
            target,
            deps,
        });
    }
    Ok(graph)
}

/// The unit graph of the build of the workspace at `workspace`, at its
/// lock (`--locked`: the lock is not rewritten) with `args` (features)
/// added, written by cargo without building anything. Cargo opens the
/// flag on the stable toolchain under `RUSTC_BOOTSTRAP`, as the dump does.
fn unit_graph(workspace: &Path, work: &Path, args: &[String]) -> Result<String, UorustError> {
    let toolchain = format!("+{}", crate::mir::spec::TOOLCHAIN);
    let out = run(
        "cargo build --unit-graph",
        Command::new("cargo")
            .args([
                toolchain.as_str(),
                "build",
                "--workspace",
                "--locked",
                "-q",
                "--unit-graph",
                "-Zunstable-options",
            ])
            .args(args)
            .current_dir(workspace)
            // Nothing is written into the workspace.
            .env_remove("CARGO_BUILD_BUILD_DIR")
            .env_remove("CARGO_BUILD_TARGET_DIR")
            .env("CARGO_TARGET_DIR", work.join("target"))
            .env("RUSTC_BOOTSTRAP", "1"),
    )?;
    String::from_utf8(out).map_err(|e| tool("cargo build --unit-graph", e))
}

/// What `uorust crate --workspace` did with one library of crates.io.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Dumped, translated and embedded, its dumps kept: what [`Lifted`]
    /// counts.
    Lifted(Lifted),
    /// Not lifted, with why: what failed, or a library it builds on that
    /// was not lifted, whose calls would be left unlinked.
    Refused(String),
}

/// The name of the index `uorust crate --workspace` writes under its kept
/// dumps.
pub const LINKS: &str = "links.toml";

/// What the workspace driver is given.
#[derive(Clone, Copy, Debug)]
pub struct Workspace<'a> {
    /// The workspace's root.
    pub root: &'a Path,
    /// What the workspace's build is run with (its features), as cargo
    /// takes them.
    pub cargo_args: &'a [String],
    /// The directory each crate's dumps are kept in, by [`Locked::dir`],
    /// with [`LINKS`].
    pub dumps: &'a Path,
    /// The directory each crate's register is written in, by
    /// [`Locked::dir`].
    pub registers: &'a Path,
    /// The rust-src `library` directory of the pinned toolchain.
    pub library: &'a Path,
    /// Only these packages (by name) and those they build on; every
    /// library the build links when empty.
    pub only: &'a [String],
    /// Libraries linked into every crate's program before those it builds
    /// on (core, alloc), `(crate, dumps)`.
    pub link: &'a [(String, PathBuf)],
}

/// Every library of crates.io the build of a workspace links, each at its
/// locked version and checksum and with the features the build enables
/// for it, dumped, translated and embedded with the libraries it builds on
/// linked (those first), its dumps kept; then [`LINKS`], which names each
/// crate's kept dumps and, for each member's target, the `--link` list of
/// its embedding. A crate that fails is a refused row of the index, and so
/// is one that builds on a refused one; a panic of the embedding is caught
/// as such a row, so that the rest are measured.
pub fn lift_workspace(ws: &Workspace<'_>) -> Result<(Graph, Vec<Outcome>), UorustError> {
    let work = std::env::temp_dir().join(format!("uorust-workspace-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).map_err(|e| tool("work", e))?;
    let lock_file = ws.root.join("Cargo.lock");
    let lock = crate::embed::read_text(&lock_file)?;
    let graph = unit_graph(ws.root, &work, ws.cargo_args).and_then(|g| graph(&g, &lock));
    let _ = std::fs::remove_dir_all(&work);
    let graph = graph?;
    let wanted: BTreeSet<usize> = if ws.only.is_empty() {
        (0..graph.locked.len()).collect()
    } else {
        let named: Vec<usize> = (0..graph.locked.len())
            .filter(|&i| ws.only.contains(&graph.locked[i].name))
            .collect();
        graph.closure(&named).into_iter().collect()
    };
    std::fs::create_dir_all(ws.dumps).map_err(|e| tool("dumps", e))?;
    let mut outcomes: Vec<Option<Outcome>> = vec![None; graph.locked.len()];
    for i in 0..graph.locked.len() {
        if !wanted.contains(&i) {
            continue;
        }
        let l = &graph.locked[i];
        let refused: Vec<String> = graph
            .closure(&l.deps)
            .into_iter()
            .filter(|d| !matches!(outcomes[*d], Some(Outcome::Lifted(_))))
            .map(|d| graph.locked[d].dir())
            .collect();
        let outcome = if refused.is_empty() {
            let link = graph.links(&l.deps, ws.link, ws.dumps);
            let kept = ws.dumps.join(l.dir());
            let options = Options {
                library: Some(ws.library),
                differential: false,
                link: &link,
                features: &l.features,
                no_default_features: true,
                checksum: Some(&l.checksum),
                lock: Some(&lock_file),
                dumps: Some(&kept),
            };
            let register = ws.registers.join(l.dir());
            match recorded(&kept, &options) {
                Some(done) => Outcome::Lifted(done),
                None => lift_apart(&l.name, &l.version, &register, &options),
            }
        } else {
            Outcome::Refused(format!(
                "builds on what was not lifted: {}",
                refused.join(", ")
            ))
        };
        eprintln!(
            "crate {}: {}",
            l.dir(),
            match &outcome {
                Outcome::Lifted(x) => format!(
                    "{} bodies, {}",
                    x.bodies,
                    x.embedding.map_or(String::new(), |(b, e, u)| format!(
                        "{b} embedded, {e} evaluable, {u} bound"
                    ))
                ),
                Outcome::Refused(why) => format!("refused: {why}"),
            }
        );
        outcomes[i] = Some(outcome);
        // The index as it stands after each crate, so that what was done
        // is named whatever ends the run.
        let done: Vec<(usize, Outcome)> = outcomes
            .iter()
            .enumerate()
            .filter_map(|(i, o)| Some((i, o.clone()?)))
            .collect();
        std::fs::write(ws.dumps.join(LINKS), render_links(&graph, &done))
            .map_err(|e| tool(LINKS, e))?;
    }
    let done: Vec<Outcome> = outcomes.into_iter().flatten().collect();
    Ok((graph, done))
}

/// [`lift_crate`] run by this tool's own executable as a process of its
/// own (`uorust crate <spec>` with the build `options` names), so that
/// what ends that run (an exhausted stack aborts the process) is the
/// crate's refused row and not the end of the rest; what it counted is read
/// back from its kept dumps' record.
fn lift_apart(name: &str, version: &str, register: &Path, options: &Options<'_>) -> Outcome {
    let (Some(library), Some(kept)) = (options.library, options.dumps) else {
        return Outcome::Refused("a crate of a build is embedded and kept".into());
    };
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => return Outcome::Refused(format!("this tool's executable: {e}")),
    };
    let mut command = Command::new(exe);
    command
        .arg("crate")
        .arg(format!("{name}@{version}"))
        .arg("--register")
        .arg(register)
        .arg("--library")
        .arg(library)
        .arg("--dumps")
        .arg(kept);
    command.args(build_flags(options));
    if let Some(c) = options.checksum {
        command.args(["--checksum", c]);
    }
    if let Some(lock) = options.lock {
        command.arg("--lock").arg(lock);
    }
    for l in link_args(options.link) {
        command.arg("--link").arg(l);
    }
    let out = match command.output() {
        Ok(o) => o,
        Err(e) => return Outcome::Refused(format!("`uorust crate` did not start: {e}")),
    };
    let stderr = String::from_utf8_lossy(&out.stderr);
    eprint!("{stderr}");
    if !out.status.success() {
        return Outcome::Refused(format!(
            "`uorust crate` ended {}: {}",
            out.status,
            why_ended(&stderr)
        ));
    }
    match recorded(kept, options) {
        Some(lifted) => Outcome::Lifted(lifted),
        None => Outcome::Refused(format!(
            "`uorust crate` kept no record of this build in {}",
            kept.display()
        )),
    }
}

/// What a run's stderr says ended it: a panic's place and message, else
/// its last line that says something (an abort's `fatal runtime error`,
/// an error the tool returned), past the notes a panic prints after it.
fn why_ended(stderr: &str) -> String {
    let lines: Vec<&str> = stderr.lines().map(str::trim).collect();
    if let Some(at) = lines.iter().rposition(|l| l.contains(" panicked at ")) {
        let place = lines[at].split(" panicked at ").nth(1).unwrap_or_default();
        let message = lines.get(at + 1).copied().unwrap_or_default();
        return format!("panicked at {place} {message}");
    }
    lines
        .iter()
        .rev()
        .find(|l| !l.is_empty() && !l.starts_with("note:"))
        .map(|l| (*l).to_string())
        .unwrap_or_default()
}

/// [`LINKS`]: each crate lifted or refused, with its build and its
/// embedding's counts, then each member's target with the `--link`
/// arguments of its embedding, the dumps named relative to the index.
pub fn render_links(graph: &Graph, done: &[(usize, Outcome)]) -> String {
    let quoted = |s: &str| toml::Value::String(s.to_string()).to_string();
    let list = |items: &[String]| {
        items
            .iter()
            .map(|x| quoted(x))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let lifted: BTreeSet<usize> = done
        .iter()
        .filter(|(_, o)| matches!(o, Outcome::Lifted(_)))
        .map(|(i, _)| *i)
        .collect();
    // The lists the embedding is given ([`Graph::links`]), of what was
    // lifted, the dumps named relative to the index.
    let lifted_dirs: BTreeSet<String> = lifted.iter().map(|&d| graph.locked[d].dir()).collect();
    let link_args = |deps: &[usize]| -> Vec<String> {
        graph
            .links(deps, &[], Path::new(""))
            .into_iter()
            .filter(|(_, d)| lifted_dirs.contains(&d.display().to_string()))
            .map(|(lib, d)| format!("{lib}={}", d.display()))
            .collect()
    };
    let mut out = String::from(
        "# The libraries of crates.io a workspace's build links (written by\n\
         # `uorust crate --workspace`; do not edit): each at its locked version,\n\
         # built with the features the build enables for it, its dumps kept in\n\
         # its directory beside this file, embedded with those it builds on\n\
         # linked; then each member's target with the `--link` list of its\n\
         # embedding.\n",
    );
    for (i, outcome) in done {
        let l = &graph.locked[*i];
        out.push_str(&format!(
            "\n[[crate]]\nname = {}\nversion = {}\nlib = {}\nchecksum = {}\nfeatures = [{}]\ndumps = {}\nlink = [{}]\n",
            quoted(&l.name),
            quoted(&l.version),
            quoted(&l.lib),
            quoted(&l.checksum),
            list(&l.features),
            quoted(&l.dir()),
            list(&link_args(&l.deps)),
        ));
        match outcome {
            Outcome::Lifted(x) => {
                out.push_str(&format!(
                    "bodies = {}\ntranslated = {}\n",
                    x.bodies, x.translated
                ));
                if let Some((b, e, u)) = x.embedding {
                    out.push_str(&format!("embedded = {b}\nevaluable = {e}\nbound = {u}\n"));
                }
            }
            Outcome::Refused(why) => out.push_str(&format!("refused = {}\n", quoted(why))),
        }
    }
    for m in &graph.members {
        out.push_str(&format!(
            "\n[[member]]\npackage = {}\ntarget = {}\nlink = [{}]\n",
            quoted(&m.package),
            quoted(&m.target),
            list(&link_args(&m.deps)),
        ));
    }
    out
}

/// Embed the crate's bodies in LexLean with the one evaluator core's and
/// std's are, and stamp each row of its register with its embedding: the
/// crate's sources sit beside `library`'s core, alloc and std, as a crate
/// of the library does, so its declarations are read as std's are.
#[allow(clippy::too_many_arguments)]
fn embed_crate(
    scope: &Scope,
    name: &str,
    dir: &Path,
    work: &Path,
    shards: &Path,
    library: &Path,
    register: &Path,
    differential: bool,
    link: &[(String, PathBuf)],
) -> Result<(crate::embed::EmbedCensus, Option<(usize, usize)>), UorustError> {
    let link: Vec<(&str, &Path)> = link
        .iter()
        .map(|(k, d)| (k.as_str(), d.as_path()))
        .collect();
    let toolchain = format!("+{}", crate::mir::spec::TOOLCHAIN);
    // The target the bodies were built for: the host, as the dump builds.
    let host = crate::shard::host(dir, Some(&toolchain))?;
    crate::shard::write_target(shards, &host, dir, Some(&toolchain))?;
    let root = work.join("root").join("library");
    std::fs::create_dir_all(&root).map_err(|e| tool("library", e))?;
    // A link's target is read from the link's own directory: each is named
    // absolutely, whatever directory `library` and the crate were given from.
    let library = std::fs::canonicalize(library).map_err(|e| tool("library", e))?;
    let dir = std::fs::canonicalize(dir).map_err(|e| tool("library", e))?;
    for krate in ["core", "alloc", "std"] {
        std::os::unix::fs::symlink(library.join(krate), root.join(krate))
            .map_err(|e| tool("library", e))?;
    }
    std::os::unix::fs::symlink(&dir, root.join(name)).map_err(|e| tool("library", e))?;
    // The crates it builds on, beside it by their names, whose enums and
    // structs its bodies make values of.
    let mut builds_on = Vec::new();
    for (lib, source) in dependencies(&dir, &toolchain)? {
        if lib != name && !root.join(&lib).exists() {
            std::os::unix::fs::symlink(&source, root.join(&lib)).map_err(|e| tool("library", e))?;
            builds_on.push(lib);
        }
    }
    layouts(name, &dir, work, shards, &library, &toolchain)?;
    let harness = work.join("harness");
    let census = crate::embed::embed_with(
        name,
        &builds_on,
        &root,
        shards,
        &work.join("embed"),
        &[],
        &crate::embed::Outputs {
            harness: differential.then_some(harness.as_path()),
            register: Some(register),
            link: &link,
            ..Default::default()
        },
    )?;
    let mut checked = None;
    if differential {
        let observed = work.join("observed.txt");
        let alone = work.join("differential");
        let theorems = if harness.join("Cargo.toml").is_file() {
            run_harness(&harness, &dir, &library, &toolchain, &observed)?;
            crate::embed::embed_with(
                name,
                &builds_on,
                &root,
                shards,
                &work.join("embed"),
                &[],
                &crate::embed::Outputs {
                    observed: Some(&observed),
                    differential: Some(&alone),
                    link: &link,
                    ..Default::default()
                },
            )?;
            verify_differential(&alone)?
        } else {
            BTreeMap::new()
        };
        stamp_differential(register, &theorems)?;
        checked = Some((theorems.len(), theorems.values().sum()));
    }
    crate::mir::core::summarize_register(scope, register)?;
    // The embedding refuses no body: every κ the register records is
    // embedded, a macro's impls of one text once.
    let missed = crate::mir::core::unembedded(register)?;
    if !missed.is_empty() {
        return Err(tool(
            &format!("{name} embedding"),
            format!(
                "{} bodies the register records are not embedded: {}",
                missed.len(),
                missed
                    .iter()
                    .take(8)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        ));
    }
    Ok((census, checked))
}

/// The flags the crate's MIR was dumped with ([`dump`]), which the harness
/// builds the crate and the library with, so the two builds are one.
const CODEGEN: &str = "-Coverflow-checks=on -Cdebug-assertions=off";

/// Build and run the differential check's harness (`uorust embed
/// --harness`) for the crate at `dir`, writing what it prints to
/// `observed`: the crate is copied beside the harness with each wrapper
/// `wrappers.json` asks for written into its function's module, and the
/// library built from `library` with the dump's flags, as
/// scripts/mir/harness.sh builds core. A wrapper rustc refuses (a function
/// its module's file does not see, one generic past its scalars) is left
/// out, and its cases print `unbuilt`.
fn run_harness(
    harness: &Path,
    dir: &Path,
    library: &Path,
    toolchain: &str,
    observed: &Path,
) -> Result<(), UorustError> {
    #[derive(serde::Deserialize)]
    struct Wrapper {
        sample: usize,
        module: Vec<String>,
        /// For a method, the file of its impl, from the crate's root.
        #[serde(default)]
        file: Option<String>,
        code: String,
    }
    let wrappers: Vec<Wrapper> =
        serde_json::from_str(&crate::embed::read_text(&harness.join("wrappers.json"))?)
            .map_err(|e| tool("wrappers.json", e))?;
    let copy = harness.with_file_name("harness-crate");
    copy_tree(dir, &copy)?;
    let copy = std::fs::canonicalize(&copy).map_err(|e| tool("harness", e))?;
    let home = std::fs::canonicalize(harness).map_err(|e| tool("harness", e))?;
    let lib = copy.join(crate::layouts::library_file(dir)?);
    let target = harness.join("target");
    let mut pristine: BTreeMap<PathBuf, String> = BTreeMap::new();
    let mut live: BTreeSet<usize> = wrappers.iter().map(|w| w.sample).collect();
    let cargo = |args: &[&str], flags: &str| {
        let mut c = Command::new("cargo");
        c.arg(toolchain)
            .args(args)
            .current_dir(harness)
            .env_remove("CARGO_BUILD_BUILD_DIR")
            .env_remove("CARGO_BUILD_TARGET_DIR")
            .env("CARGO_TARGET_DIR", &target)
            .env("RUSTC_BOOTSTRAP", "1")
            .env("CARGO_INCREMENTAL", "0")
            // No debug info: what the build computes is the same, at a
            // fraction of the disk.
            .env("CARGO_PROFILE_DEV_DEBUG", "0")
            .env("RUSTFLAGS", format!("{CODEGEN} --cap-lints=allow {flags}"));
        c
    };
    loop {
        // Each live wrapper on a line of its own past the end of its
        // module's file, so an error's line names it.
        let mut at: BTreeMap<(PathBuf, usize), usize> = BTreeMap::new();
        let mut by_file: BTreeMap<PathBuf, Vec<&Wrapper>> = BTreeMap::new();
        for w in wrappers.iter().filter(|w| live.contains(&w.sample)) {
            let module: Vec<&str> = w.module.iter().map(String::as_str).collect();
            let home = match &w.file {
                Some(f) => copy.join(f),
                None => crate::layouts::module_file(&lib, &module),
            };
            by_file.entry(home).or_default().push(w);
        }
        for f in by_file.keys() {
            if !pristine.contains_key(f) {
                pristine.insert(f.clone(), crate::embed::read_text(f)?);
            }
        }
        for (f, text) in &pristine {
            let mut body = text.clone();
            if !body.ends_with('\n') {
                body.push('\n');
            }
            let mut line = body.lines().count();
            for w in by_file.get(f).into_iter().flatten() {
                line += 1;
                body.push_str(&w.code);
                body.push('\n');
                at.insert((f.clone(), line), w.sample);
            }
            std::fs::write(f, body).map_err(|e| tool("harness", e))?;
        }
        let out = cargo(&["check", "-q", "--message-format=short"], "")
            .output()
            .map_err(|e| tool("cargo check", e))?;
        if out.status.success() {
            break;
        }
        let stderr = String::from_utf8_lossy(&out.stderr);
        let refused: BTreeSet<usize> = stderr
            .lines()
            .filter(|l| l.contains(": error"))
            .filter_map(|l| {
                let mut parts = l.splitn(3, ':');
                let file = parts.next()?;
                let line: usize = parts.next()?.trim().parse().ok()?;
                let file = Path::new(file);
                let file = if file.is_absolute() {
                    file.to_path_buf()
                } else {
                    home.join(file)
                };
                let file = std::fs::canonicalize(&file).unwrap_or(file);
                at.get(&(file, line)).copied()
            })
            .collect();
        if refused.is_empty() {
            // An error at no wrapper's line: the crate's copy does not
            // build beside the harness, and no wrapper is kept.
            eprintln!(
                "harness: the crate's copy does not check: {}",
                stderr.lines().take(4).collect::<Vec<_>>().join(" | ")
            );
            live.clear();
            for (f, text) in &pristine {
                std::fs::write(f, text).map_err(|e| tool("harness", e))?;
            }
            break;
        }
        eprintln!(
            "harness: rustc refused {} of {} wrappers",
            refused.len(),
            live.len()
        );
        live.retain(|k| !refused.contains(k));
    }
    // The calls of a refused wrapper's sample give way to its cases
    // printed unbuilt.
    let main = harness.join("src").join("main.rs");
    // The crate as its library is named, which its manifest may name
    // apart from its package (rustls-webpki's `webpki`).
    let pristine = crate::embed::read_text(&main)?;
    let lib = lib_name(dir)?;
    let pristine: String = pristine
        .lines()
        .map(|l| {
            match l
                .strip_prefix("use ")
                .and_then(|r| r.strip_suffix(" as _;"))
            {
                Some(_) => format!("use {lib} as _;\n"),
                None => format!("{l}\n"),
            }
        })
        .collect();
    let samples_of = sample_cases(&pristine);
    let host = String::from_utf8_lossy(&run(
        "rustc",
        Command::new("rustc").args([toolchain, "-vV"]),
    )?)
    .lines()
    .find_map(|l| l.strip_prefix("host: ").map(str::to_string))
    .ok_or_else(|| tool("rustc", "no host"))?;
    let out = loop {
        std::fs::write(&main, select_samples(&pristine, &live)?).map_err(|e| tool("main.rs", e))?;
        let out = cargo(
            &[
                "run",
                "-q",
                "-Zbuild-std=std,panic_unwind",
                "--target",
                &host,
            ],
            "",
        )
        .env("__CARGO_TESTS_ONLY_SRC_ROOT", library)
        .output()
        .map_err(|e| tool("harness", e))?;
        if out.status.success() {
            break out.stdout;
        }
        // A case that aborts the process (a panic that does not unwind)
        // is no value to compare: its sample, the one whose case ran past
        // the last the harness printed, is left out, and the rest run
        // again.
        let stdout = String::from_utf8_lossy(&out.stdout);
        let ran = stdout
            .lines()
            .next()
            .is_some_and(|l| l.starts_with("checks "));
        let next = stdout
            .lines()
            .skip(1)
            .filter_map(|l| l.split_once(' ')?.0.parse::<usize>().ok())
            .max()
            .map_or(0, |n| n + 1);
        match samples_of.get(&next).filter(|k| ran && live.contains(k)) {
            Some(&k) => {
                eprintln!("harness: case {next} aborts the run; its sample {k} is left out");
                live.remove(&k);
            }
            None => return Err(tool("harness", String::from_utf8_lossy(&out.stderr))),
        }
    };
    std::fs::write(observed, out).map_err(|e| tool("observed", e))?;
    let _ = std::fs::remove_dir_all(&target);
    Ok(())
}

/// The name of the library of the package at `dir`: its manifest's
/// `[lib] name`, else its package's name with each `-` a `_`.
fn lib_name(dir: &Path) -> Result<String, UorustError> {
    let manifest: toml::Table = crate::embed::read_text(&dir.join("Cargo.toml"))?
        .parse()
        .map_err(|e| tool("Cargo.toml", e))?;
    let named = manifest
        .get("lib")
        .and_then(|l| l.get("name"))
        .and_then(toml::Value::as_str);
    let package = manifest
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| tool("Cargo.toml", "no package name"))?;
    Ok(named.unwrap_or(package).replace('-', "_"))
}

/// The sample each case of the harness's `main.rs` belongs to, by case,
/// for the samples called through a wrapper.
fn sample_cases(main: &str) -> BTreeMap<usize, usize> {
    let mut out = BTreeMap::new();
    let mut at: Option<usize> = None;
    for line in main.lines() {
        let t = line.trim();
        if let Some(k) = t.strip_prefix("// sample ") {
            at = k.parse().ok();
        } else if t.starts_with("// unbuilt ") {
            at = None;
        } else if let (Some(k), Some(rest)) = (at, t.strip_prefix("Ok(r) => println!(\"")) {
            if let Some(case) = rest.split(' ').next().and_then(|c| c.parse().ok()) {
                out.insert(case, k);
            }
        }
    }
    out
}

/// The harness's `main.rs` with the calls of each sample not `live` given
/// way to its cases printed unbuilt ([`crate::embed`]'s `harness_crate`
/// writes both, the second commented out).
fn select_samples(main: &str, live: &BTreeSet<usize>) -> Result<String, UorustError> {
    let mut kept = String::new();
    let mut skipping: Option<String> = None;
    let mut fallback = false;
    for line in main.lines() {
        let t = line.trim();
        if let Some(k) = t.strip_prefix("// sample ") {
            let k: usize = k.parse().map_err(|e| tool("main.rs", e))?;
            if !live.contains(&k) {
                skipping = Some(k.to_string());
                continue;
            }
        }
        if let Some(k) = t.strip_prefix("// unbuilt ") {
            fallback = skipping.as_deref() == Some(k);
            skipping = (!fallback).then(|| format!("end {k}"));
            continue;
        }
        if let Some(k) = t.strip_prefix("// end ") {
            fallback = false;
            if skipping.as_deref() == Some(&format!("end {k}")) {
                skipping = None;
            }
            continue;
        }
        if skipping.is_some() {
            continue;
        }
        if fallback {
            kept.push_str(&line.replacen("// ", "", 1));
        } else {
            kept.push_str(line);
        }
        kept.push('\n');
    }
    Ok(kept)
}

/// Verify the differential check's project at `dir` with `lexlean`, and
/// read from its index the theorems of each body, by κ: a theorem `lexlean`
/// refutes (the evaluator returns other than rustc's build did) fails the
/// verify, and so the crate.
fn verify_differential(dir: &Path) -> Result<BTreeMap<String, usize>, UorustError> {
    let index = dir.join("differential.toml");
    if !index.is_file() {
        return Ok(BTreeMap::new());
    }
    // Read by its lines: a case's inputs are u128 bits, past what a TOML
    // integer holds.
    let mut by_kappa: BTreeMap<String, usize> = BTreeMap::new();
    for line in crate::embed::read_text(&index)?.lines() {
        if let Some(k) = line
            .strip_prefix("kappa = \"")
            .and_then(|r| r.strip_suffix('"'))
        {
            *by_kappa.entry(k.to_string()).or_default() += 1;
        }
    }
    if by_kappa.is_empty() {
        return Ok(by_kappa);
    }
    for args in [&["lock"][..], &["verify"][..]] {
        run(
            "lexlean",
            Command::new("lexlean").args(args).current_dir(dir),
        )?;
    }
    Ok(by_kappa)
}

/// Stamp each row of the register `dir` with the differential theorems
/// verified of its body, `differential = <n>`, by κ, and clear the stamp
/// of every other row.
fn stamp_differential(dir: &Path, theorems: &BTreeMap<String, usize>) -> Result<(), UorustError> {
    for file in crate::embed::dir_paths(dir)? {
        let stem = file
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if file.extension().is_none_or(|x| x != "toml") || stem == "frontier" {
            continue;
        }
        let text = crate::embed::read_text(&file)?;
        let mut out = String::with_capacity(text.len());
        for line in text.lines() {
            if line.starts_with("differential = ") {
                continue;
            }
            out.push_str(line);
            out.push('\n');
            let kappa = line
                .strip_prefix("kappa = \"")
                .and_then(|r| r.strip_suffix('"'));
            if let Some(n) = kappa.and_then(|k| theorems.get(k)) {
                out.push_str(&format!("differential = {n}\n"));
            }
        }
        if out != text {
            std::fs::write(&file, out).map_err(|e| tool("register", e))?;
        }
    }
    Ok(())
}

/// The library crates the crate at `dir` builds on, as cargo resolved them
/// for its dump: each normal dependency, and theirs, by its library's crate
/// name with the directory of its package.
fn dependencies(dir: &Path, toolchain: &str) -> Result<Vec<(String, PathBuf)>, UorustError> {
    crate::shard::dependencies(dir, &[toolchain.to_string()], None)
}

/// Ask rustc for the layouts of the aggregates the crate's dumps print, as
/// the library's dump step does (`crate::layouts`): in a copy of the crate
/// beside `library`'s core, alloc and std, checked by cargo with
/// `rustc_attrs` enabled for the aliases, into `shards`' layouts file.
fn layouts(
    name: &str,
    dir: &Path,
    work: &Path,
    shards: &Path,
    library: &Path,
    toolchain: &str,
) -> Result<usize, UorustError> {
    let copy = work.join("layouts").join("library");
    std::fs::create_dir_all(&copy).map_err(|e| tool("layouts", e))?;
    for krate in ["core", "alloc", "std"] {
        std::os::unix::fs::symlink(library.join(krate), copy.join(krate))
            .map_err(|e| tool("layouts", e))?;
    }
    let own = copy.join(name);
    copy_tree(dir, &own)?;
    let target = work.join("layouts").join("target");
    let quote = |p: &Path| format!("'{}'", p.display());
    let check = format!(
        "cd {} && RUSTC_BOOTSTRAP=1 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR={} \
         RUSTFLAGS='-Ztrim-diagnostic-paths=no -Zcrate-attr=feature(rustc_attrs) -Coverflow-checks=on -Cdebug-assertions=off --cap-lints=allow' \
         cargo {toolchain} check -q --lib",
        quote(&own),
        quote(&target),
    );
    let n = crate::layouts::run(name, shards, &copy, &check)?;
    let _ = std::fs::remove_dir_all(&target);
    Ok(n)
}

/// What marks a published link's dumps as unpacked whole under its
/// digest: written last, so a directory without it is an interrupted
/// unpacking, done again.
const UNPACKED: &str = ".unpacked";

/// The dumps a `--link <crate>=<dumps>` names: a directory as it is, or a
/// published archive of one, `<url>#sha256=<hex>` (a release asset, any
/// URL curl reads, `file://` among them), fetched into `cache` under its
/// digest and unpacked there only when the archive's SHA-256 is the one
/// named; the archive holds the dumps at its root (`target.toml`, the
/// layouts, a directory per module). The digest is the trust: a link pins
/// the bytes it was measured with, as a lock pins a crate's.
pub fn link_dumps(dumps: &str, cache: &Path) -> Result<PathBuf, UorustError> {
    let Some((url, digest)) = dumps.split_once("#sha256=") else {
        return Ok(PathBuf::from(dumps));
    };
    let digest = digest.to_ascii_lowercase();
    if digest.len() != 64 || !digest.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(tool(
            "--link",
            format!("`{digest}` is not a SHA-256 in hex"),
        ));
    }
    let dir = cache.join(&digest);
    if dir.join(UNPACKED).is_file() {
        return Ok(dir);
    }
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(cache).map_err(|e| tool("--link", e))?;
    let archive = cache.join(format!("{digest}.tar.gz"));
    run(
        "--link fetch",
        Command::new("curl")
            .args(["-sSfL", "-A", USER_AGENT, "-o"])
            .arg(&archive)
            .arg(url),
    )?;
    let bytes = std::fs::read(&archive).map_err(|e| tool("--link fetch", e))?;
    let found = sha256_hex(&bytes);
    if found != digest {
        let _ = std::fs::remove_file(&archive);
        return Err(tool(
            "--link fetch",
            format!("{url} is {found}, the link names {digest}"),
        ));
    }
    std::fs::create_dir_all(&dir).map_err(|e| tool("--link", e))?;
    let unpacked = run(
        "tar",
        Command::new("tar")
            .arg("-xzf")
            .arg(&archive)
            .arg("-C")
            .arg(&dir),
    );
    let _ = std::fs::remove_file(&archive);
    unpacked?;
    if !dir.join("target.toml").is_file() {
        let _ = std::fs::remove_dir_all(&dir);
        return Err(tool(
            "--link fetch",
            format!("{url} holds no target.toml at its root"),
        ));
    }
    std::fs::write(dir.join(UNPACKED), &digest).map_err(|e| tool("--link", e))?;
    Ok(dir)
}

/// Copy the tree `from` to `to`, its links followed.
fn copy_tree(from: &Path, to: &Path) -> Result<(), UorustError> {
    std::fs::create_dir_all(to).map_err(|e| tool("copy", e))?;
    for entry in std::fs::read_dir(from).map_err(|e| tool("copy", e))? {
        let entry = entry.map_err(|e| tool("copy", e))?;
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if src.is_dir() {
            copy_tree(&src, &dst)?;
        } else {
            std::fs::copy(&src, &dst).map_err(|e| tool("copy", e))?;
        }
    }
    Ok(())
}

/// The public type aliases a registry crate's library root declares, by
/// their path (`krate::Alias`), each with what it stands for written by
/// full paths. They are read from the crate's own source: its archive in
/// Cargo's cache at the version Cargo.lock pins, checked against the
/// checksum Cargo.lock records, which is the source Cargo builds. A name
/// the root declares once, or binds by one `use`, is that declaration or
/// binding whatever `cfg` it stands under: the workspace that names the
/// alias compiles, so the alias and what it names are compiled in. A name
/// declared or bound more than once is not read. Nothing is read (`None`)
/// where the crate is not pinned once, or no archive of it at the locked
/// checksum is in the cache, or its manifest or library root does not read.
pub(crate) fn crate_aliases(
    cargo_home: &Path,
    krate: &str,
    lock: &[crate::facts::LockedPackage],
) -> Option<BTreeMap<String, crate::ast::Alias>> {
    let pinned: Vec<&crate::facts::LockedPackage> = lock
        .iter()
        .filter(|l| l.name.replace('-', "_") == krate && l.source.starts_with("registry+"))
        .collect();
    let [locked] = pinned.as_slice() else {
        return None;
    };
    let stem = format!("{}-{}", locked.name, locked.version);
    let caches = std::fs::read_dir(cargo_home.join("registry").join("cache")).ok()?;
    let archive = caches
        .filter_map(Result::ok)
        .map(|index| index.path().join(format!("{stem}.crate")))
        .find(|archive| {
            std::fs::read(archive).is_ok_and(|bytes| {
                let digest: String = Sha256::digest(&bytes)
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect();
                digest == locked.checksum
            })
        })?;
    // A member of the archive as text, or none where it does not extract
    // or is not UTF-8.
    let member = |inside: &str| -> Option<String> {
        let out = run(
            "tar",
            Command::new("tar")
                .arg("-xzOf")
                .arg(&archive)
                .arg(format!("{stem}/{inside}")),
        )
        .ok()?;
        String::from_utf8(out).ok()
    };
    let manifest: toml::Value = member("Cargo.toml")?.parse().ok()?;
    let root = manifest
        .get("lib")
        .and_then(|lib| lib.get("path"))
        .and_then(toml::Value::as_str)
        .unwrap_or("src/lib.rs")
        .to_string();
    let file = syn::parse_file(&member(&root)?).ok()?;
    let (uses, declared) = crate::names::root_names(&file);
    // What each name the root writes bare names, by its full path.
    let mut paths: BTreeMap<String, String> = declared
        .iter()
        .filter(|n| !uses.contains_key(*n))
        .map(|n| (n.clone(), format!("{krate}::{n}")))
        .collect();
    for (name, bound) in &uses {
        let [path] = bound.iter().collect::<Vec<_>>()[..] else {
            continue;
        };
        let path = match path.first().map(String::as_str) {
            Some("crate" | "self") => std::iter::once(krate.to_string())
                .chain(path[1..].iter().cloned())
                .collect::<Vec<_>>(),
            Some("super") => continue,
            Some("") => path[1..].to_vec(),
            _ => path.clone(),
        };
        if !declared.contains(name) {
            paths.insert(name.clone(), path.join("::"));
        }
    }
    let mut written: BTreeMap<String, Vec<&syn::ItemType>> = BTreeMap::new();
    for item in &file.items {
        if let syn::Item::Type(t) = item {
            written.entry(t.ident.to_string()).or_default().push(t);
        }
    }
    let mut out = BTreeMap::new();
    for (name, items) in written {
        let [t] = items.as_slice() else {
            continue;
        };
        if !matches!(t.vis, syn::Visibility::Public(_)) {
            continue;
        }
        let alias = crate::convert::alias(&t.generics, &t.ty);
        // A type parameter of the alias is no name of the root.
        let names: BTreeMap<String, String> = paths
            .iter()
            .filter(|(n, _)| !alias.params.contains(n))
            .map(|(n, p)| (n.clone(), p.clone()))
            .collect();
        let to = alias.to.renamed(&names);
        out.insert(
            format!("{krate}::{name}"),
            crate::ast::Alias {
                params: alias.params,
                to,
            },
        );
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dump_is_filed_under_its_module() {
        let names = [
            "serde.de-{impl#3}-f.2-0.ElaborateBoxDerefs.before.mir",
            "serde.f.2-0.ElaborateBoxDerefs.before.mir",
            "serde.f-{closure#0}.2-0.ElaborateBoxDerefs.before.mir",
            "serde.{impl#0}-g.2-0.ElaborateBoxDerefs.before.mir",
            "serde.Buffer-new.2-0.ElaborateBoxDerefs.before.mir",
        ];
        let items: std::collections::BTreeSet<&str> =
            names.iter().filter_map(|n| dump_item(n)).collect();
        let modules: Vec<String> = names
            .iter()
            .filter_map(|n| dump_item(n))
            .map(|i| dump_module(i, &items))
            .collect();
        assert_eq!(
            modules,
            ["de", ROOT_MODULE, ROOT_MODULE, ROOT_MODULE, ROOT_MODULE]
        );
    }

    /// A sample left out gives way to its cases printed unbuilt, and each
    /// case is known by its sample.
    #[test]
    fn a_sample_left_out_prints_its_cases_unbuilt() {
        let main = "fn main() {\n    // sample 0\n    match f() {\n        Ok(r) => println!(\"0 {}\", r),\n    }\n    // unbuilt 0\n    // println!(\"0 unbuilt\");\n    // end 0\n    // sample 1\n    match g() {\n        Ok(r) => println!(\"1 {}\", r),\n        Ok(r) => println!(\"2 {}\", r),\n    }\n    // unbuilt 1\n    // println!(\"1 unbuilt\");\n    // println!(\"2 unbuilt\");\n    // end 1\n}\n";
        assert_eq!(sample_cases(main), BTreeMap::from([(0, 0), (1, 1), (2, 1)]));
        let kept = select_samples(main, &BTreeSet::from([0])).unwrap();
        assert!(kept.contains("Ok(r) => println!(\"0 {}\", r)"), "{kept}");
        assert!(
            kept.contains("    println!(\"1 unbuilt\");\n    println!(\"2 unbuilt\");"),
            "{kept}"
        );
        assert!(
            !kept.contains("g()") && !kept.contains("0 unbuilt"),
            "{kept}"
        );
    }

    #[test]
    fn a_spec_is_a_name_at_a_version() {
        assert_eq!(
            parse_spec("serde@1.0.0").ok(),
            Some(("serde".to_string(), "1.0.0".to_string()))
        );
        assert!(parse_spec("serde").is_err() && parse_spec("@1").is_err());
    }

    // link: a library of crates.io kept, read from a build's unit graph,
    // and linked into the crate that calls it.

    /// A package `name` at `dir` whose library is `lib`, building on the
    /// packages `deps` beside it, each by its absolute path (the layouts
    /// step checks a copy of the package elsewhere).
    fn link_package(dir: &Path, name: &str, deps: &[&str], lib: &str) {
        std::fs::create_dir_all(dir.join("src")).expect("the package");
        let beside = dir.parent().unwrap_or(dir);
        let deps: String = deps
            .iter()
            .map(|d| format!("{d} = {{ path = \"{}\" }}\n", beside.join(d).display()))
            .collect();
        std::fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
                 [dependencies]\n{deps}\n[workspace]\n"
            ),
        )
        .expect("the manifest");
        std::fs::write(dir.join("src/lib.rs"), lib).expect("the library");
    }

    /// A `library` directory of core, alloc and std with no declarations:
    /// what the embedding reads besides the dumps, for bodies over
    /// integers alone.
    fn link_library_root(at: &Path) -> PathBuf {
        let library = at.join("library");
        for krate in ["core", "alloc", "std"] {
            let src = library.join(krate).join("src");
            std::fs::create_dir_all(&src).expect("the library");
            std::fs::write(src.join("lib.rs"), "").expect("the library");
        }
        library
    }

    /// The callees the embedded project under `out` leaves to a
    /// dictionary and those it names a body of, by their texts.
    fn link_callees(out: &Path) -> (Vec<String>, Vec<String>) {
        fn walk(dir: &Path, text: &mut String) {
            for p in crate::embed::dir_paths(dir).unwrap_or_default() {
                if p.is_dir() {
                    walk(&p, text);
                } else if p.extension().is_some_and(|x| x == "tex") {
                    text.push_str(&std::fs::read_to_string(&p).unwrap_or_default());
                }
            }
        }
        // The crate's modules, each a directory of `src`; the evaluator's
        // own files beside them name no body of the crate.
        let mut text = String::new();
        for p in crate::embed::dir_paths(&out.join("src")).unwrap_or_default() {
            if p.is_dir() {
                walk(&p, &mut text);
            }
        }
        let of = |tag: &str| -> Vec<String> {
            text.match_indices(tag)
                .filter_map(|(i, _)| {
                    let rest = &text[i + tag.len()..];
                    let rest = rest.strip_prefix('"')?;
                    Some(rest[..rest.find('"')?].to_string())
                })
                .collect()
        };
        (of("Callee.bound "), of("Callee.body "))
    }

    /// A call into a crate whose dumps `uorust crate` kept is the linked
    /// crate's body when the crate is linked, and a dictionary's entry when
    /// it is not; the kept dumps record the build they are of.
    #[test]
    fn link_a_call_into_a_kept_crate_is_its_body() {
        let work = std::env::temp_dir().join(format!("uorust-link-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&work);
        let library = link_library_root(&work);
        link_package(
            &work.join("dep"),
            "dep",
            &[],
            "pub fn max(a: u32, b: u32) -> u32 {\n    if a > b { a } else { b }\n}\n",
        );
        // `max` is core's name too, so the call prints its crate's path.
        link_package(
            &work.join("user"),
            "user",
            &["dep"],
            "pub fn top(a: u32, b: u32) -> u32 {\n    dep::max(a, b)\n}\n",
        );
        let dumps = work.join("dumps").join("dep");
        let kept = lift_source(
            "dep",
            "0.1.0",
            &work.join("dep"),
            &work.join("w-dep"),
            &work.join("reg").join("dep"),
            &Options {
                library: Some(&library),
                dumps: Some(&dumps),
                checksum: Some("c0ffee"),
                ..Options::default()
            },
        )
        .expect("dep lifts");
        assert_eq!(kept.embedding, Some((1, 1, 0)), "{kept:?}");
        for file in ["target.toml", "layouts.json", DUMP_RECORD] {
            assert!(dumps.join(file).is_file(), "{file} is not kept");
        }
        assert!(
            dumps.join(ROOT_MODULE).is_dir(),
            "the module shard is not kept"
        );
        let record = std::fs::read_to_string(dumps.join(DUMP_RECORD)).expect("the record");
        assert!(
            record.contains("name = \"dep\"\nversion = \"0.1.0\"\nchecksum = \"c0ffee\"\ndefault-features = true\nfeatures = []\n"),
            "{record}"
        );
        // The record is the run's: the same build is known as done, with
        // what it counted; another build is not.
        let asked = Options {
            library: Some(&library),
            dumps: Some(&dumps),
            checksum: Some("c0ffee"),
            ..Options::default()
        };
        assert_eq!(recorded(&dumps, &asked), Some(kept.clone()));
        let other = ["std".to_string()];
        assert_eq!(
            recorded(
                &dumps,
                &Options {
                    features: &other,
                    ..asked
                }
            ),
            None
        );
        assert_eq!(
            recorded(
                &dumps,
                &Options {
                    checksum: None,
                    ..asked
                }
            ),
            None
        );
        let user = |w: &str, link: &[(String, PathBuf)]| {
            let lifted = lift_source(
                "user",
                "0.1.0",
                &work.join("user"),
                &work.join(w),
                &work.join("reg").join(w),
                &Options {
                    library: Some(&library),
                    link,
                    ..Options::default()
                },
            )
            .expect("user lifts");
            (lifted, link_callees(&work.join(w).join("embed")))
        };
        let (alone, (bound, _)) = user("w-alone", &[]);
        assert_eq!(alone.embedding, Some((1, 1, 1)), "{alone:?}");
        assert_eq!(bound, ["dep::max"]);
        let (linked, (bound, bodies)) = user("w-linked", &[("dep".to_string(), dumps.clone())]);
        assert_eq!(linked.embedding, Some((1, 1, 0)), "{linked:?}");
        assert!(bound.is_empty(), "{bound:?}");
        assert_eq!(bodies, ["dep::max"]);
        let _ = std::fs::remove_dir_all(&work);
    }

    /// A unit of a unit graph: its package id, target kinds and name, its
    /// features and the units it depends on.
    fn link_unit(id: &str, kind: &str, name: &str, features: &[&str], deps: &[usize]) -> String {
        let deps: Vec<String> = deps
            .iter()
            .map(|d| format!("{{\"index\": {d}, \"extern_crate_name\": \"x\"}}"))
            .collect();
        let features: Vec<String> = features.iter().map(|f| format!("\"{f}\"")).collect();
        format!(
            "{{\"pkg_id\": \"{id}\", \"target\": {{\"kind\": [\"{kind}\"], \"name\": \"{name}\"}}, \
             \"mode\": \"build\", \"features\": [{}], \"platform\": null, \"dependencies\": [{}]}}",
            features.join(", "),
            deps.join(", ")
        )
    }

    const LINK_REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";

    fn link_lock(packages: &[(&str, &str)]) -> String {
        packages
            .iter()
            .map(|(n, v)| {
                format!(
                    "[[package]]\nname = \"{n}\"\nversion = \"{v}\"\nsource = \"{LINK_REGISTRY}\"\nchecksum = \"sum-{n}\"\n\n"
                )
            })
            .collect()
    }

    /// The build links what its members' code calls, each library after
    /// those it builds on, with the features the unit graph gives it; a
    /// build script's dependencies and a procedural macro's are not linked.
    #[test]
    fn link_a_build_links_what_its_members_code_calls() {
        let reg = |n: &str, v: &str| format!("{LINK_REGISTRY}#{n}@{v}");
        let units = [
            // 0: the member's library.
            link_unit("path+file:///w/app#0.1.0", "lib", "app", &[], &[1, 3, 4, 6]),
            // 1: a library of crates.io building on 2, its features exact.
            link_unit(&reg("bit-set", "0.5.3"), "lib", "bit_set", &[], &[2]),
            link_unit(&reg("bit-vec", "0.6.3"), "lib", "bit_vec", &[], &[]),
            // 3: a procedural macro and what it builds on: not linked.
            link_unit(&reg("derive", "1.0.0"), "proc-macro", "derive", &[], &[5]),
            // 4: the member's build script and what it builds on: not linked.
            link_unit(
                "path+file:///w/app#0.1.0",
                "custom-build",
                "build-script-build",
                &[],
                &[5],
            ),
            link_unit(&reg("syn", "2.0.0"), "lib", "syn", &["full"], &[]),
            // 6: the older form of a package id.
            link_unit(
                &format!("fnv 1.0.7 ({LINK_REGISTRY})"),
                "lib",
                "fnv",
                &["std", "default"],
                &[],
            ),
            // 7: the member's binary, building on its library and on 2.
            link_unit("path+file:///w/app#0.1.0", "bin", "app-cli", &[], &[0, 2]),
        ];
        let graph_json = format!(
            "{{\"version\": 1, \"units\": [{}], \"roots\": [0, 7]}}",
            units.join(", ")
        );
        let lock = link_lock(&[
            ("bit-set", "0.5.3"),
            ("bit-vec", "0.6.3"),
            ("derive", "1.0.0"),
            ("syn", "2.0.0"),
            ("fnv", "1.0.7"),
        ]);
        let g = graph(&graph_json, &lock).expect("the graph");
        let names: Vec<(&str, &str, Vec<String>, &[usize])> = g
            .locked
            .iter()
            .map(|l| {
                (
                    l.name.as_str(),
                    l.lib.as_str(),
                    l.features.clone(),
                    l.deps.as_slice(),
                )
            })
            .collect();
        assert_eq!(
            names,
            [
                ("bit-vec", "bit_vec", vec![], &[][..]),
                ("bit-set", "bit_set", vec![], &[0][..]),
                (
                    "fnv",
                    "fnv",
                    vec!["default".to_string(), "std".to_string()],
                    &[][..]
                ),
            ]
        );
        assert_eq!(g.locked[1].checksum, "sum-bit-set");
        assert_eq!(g.locked[1].dir(), "bit-set-0.5.3");
        let members: Vec<(&str, &str, &[usize])> = g
            .members
            .iter()
            .map(|m| (m.package.as_str(), m.target.as_str(), m.deps.as_slice()))
            .collect();
        assert_eq!(
            members,
            [("app", "app", &[1, 2][..]), ("app", "app_cli", &[0][..])]
        );
        // A library is linked after those it builds on, and after the
        // libraries every crate links.
        assert_eq!(g.closure(&[1]), [0, 1]);
        let core = [("core".to_string(), PathBuf::from("core-mir"))];
        // Two versions of one crate under one library name: neither is
        // linked, the rest are.
        let mut two = g.clone();
        two.locked.push(Locked {
            name: "bit-vec".into(),
            version: "0.8.0".into(),
            lib: "bit_vec".into(),
            features: vec![],
            checksum: "sum".into(),
            deps: vec![],
        });
        two.locked[1].deps.push(3);
        assert_eq!(
            two.links(&[1], &core, Path::new("d")),
            [
                ("core".to_string(), PathBuf::from("core-mir")),
                ("bit_set".to_string(), PathBuf::from("d/bit-set-0.5.3")),
            ]
        );
        assert_eq!(
            g.links(&[1], &core, Path::new("d")),
            [
                ("core".to_string(), PathBuf::from("core-mir")),
                ("bit_vec".to_string(), PathBuf::from("d/bit-vec-0.6.3")),
                ("bit_set".to_string(), PathBuf::from("d/bit-set-0.5.3")),
            ]
        );
        let done = [
            (0, Outcome::Refused("the embedding panicked: x".into())),
            (
                1,
                Outcome::Refused("builds on what was not lifted: bit-vec-0.6.3".into()),
            ),
            (
                2,
                Outcome::Lifted(Lifted {
                    embedding: Some((4, 4, 1)),
                    differential: None,
                    modules: 1,
                    bodies: 4,
                    translated: 0,
                }),
            ),
        ];
        let links = render_links(&g, &done);
        assert!(
            links.contains("name = \"fnv\"\nversion = \"1.0.7\"\nlib = \"fnv\"\nchecksum = \"sum-fnv\"\nfeatures = [\"default\", \"std\"]\ndumps = \"fnv-1.0.7\"\nlink = []\nbodies = 4\ntranslated = 0\nembedded = 4\nevaluable = 4\nbound = 1\n"),
            "{links}"
        );
        assert!(
            links.contains("refused = \"the embedding panicked: x\"\n"),
            "{links}"
        );
        // Only what was lifted is linked.
        assert!(
            links.contains(
                "[[member]]\npackage = \"app\"\ntarget = \"app\"\nlink = [\"fnv=fnv-1.0.7\"]\n"
            ),
            "{links}"
        );
        assert!(links.parse::<toml::Table>().is_ok(), "{links}");
    }

    /// A library two units of one build link with different features, or
    /// one whose lock row records no checksum, is refused.
    #[test]
    fn link_a_library_of_two_builds_or_no_checksum_is_refused() {
        let reg = |n: &str| format!("{LINK_REGISTRY}#{n}@1.0.0");
        let units = [
            link_unit("path+file:///w/a#0.1.0", "lib", "a", &[], &[1]),
            link_unit(&reg("x"), "lib", "x", &["std"], &[]),
            link_unit("path+file:///w/b#0.1.0", "lib", "b", &[], &[3]),
            link_unit(&reg("x"), "lib", "x", &[], &[]),
        ];
        let two = format!(
            "{{\"version\": 1, \"units\": [{}], \"roots\": [0, 2]}}",
            units.join(", ")
        );
        let refused = graph(&two, &link_lock(&[("x", "1.0.0")]));
        assert!(
            refused.as_ref().is_err_and(|e| e
                .to_string()
                .contains("x@1.0.0 is linked with features [std] and []")),
            "{refused:?}"
        );
        let one = format!(
            "{{\"version\": 1, \"units\": [{}], \"roots\": [0]}}",
            units[..2].join(", ")
        );
        assert!(graph(&one, &link_lock(&[("x", "1.0.0")])).is_ok());
        let unsummed = graph(&one, "");
        assert!(
            unsummed
                .as_ref()
                .is_err_and(|e| e.to_string().contains("x@1.0.0 has no checksum")),
            "{unsummed:?}"
        );
    }

    /// The features a consuming build enables are passed exactly: the
    /// crate's defaults off, each feature named.
    #[test]
    fn link_a_dump_takes_the_features_it_is_given() {
        let exact = ["default".to_string(), "std".to_string()];
        assert_eq!(
            build_flags(&Options {
                features: &exact,
                no_default_features: true,
                ..Options::default()
            }),
            ["--no-default-features", "--features=default,std"]
        );
        assert_eq!(
            build_flags(&Options {
                no_default_features: true,
                ..Options::default()
            }),
            ["--no-default-features"]
        );
        assert!(build_flags(&Options::default()).is_empty());
        assert_eq!(
            package_id("registry+https://x/index#a-b@1.2.3"),
            Some((
                "registry+https://x/index".into(),
                "a-b".into(),
                "1.2.3".into()
            ))
        );
        assert_eq!(
            package_id("path+file:///w/app#0.1.0").map(|p| p.1),
            Some("app".to_string())
        );
    }

    /// An archive whose digest is not the one recorded is refused, and the
    /// kept dumps need the library their target is read with.
    #[test]
    fn link_a_kept_dump_needs_its_library_and_the_recorded_digest() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let work = std::env::temp_dir().join(format!("uorust-link-nolib-{}", std::process::id()));
        let refused = lift_source(
            "dep",
            "0.1.0",
            &work,
            &work,
            &work,
            &Options {
                dumps: Some(&work),
                ..Options::default()
            },
        );
        assert!(
            refused
                .as_ref()
                .is_err_and(|e| e.to_string().contains("kept dumps need the library")),
            "{refused:?}"
        );
    }

    /// A link named by a published archive is fetched, checked against the
    /// digest it names and unpacked under it; an archive of another digest
    /// is refused and nothing of it is kept; a directory is itself.
    #[test]
    fn link_a_published_archive_is_taken_by_its_digest() {
        let work = std::env::temp_dir().join(format!("uorust-link-fetch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&work);
        let dumps = work.join("dumps");
        std::fs::create_dir_all(dumps.join("num")).expect("the dumps");
        std::fs::write(dumps.join("target.toml"), "pointer_width = 64\n").expect("the target");
        std::fs::write(dumps.join("num").join("core.f.mir"), "fn f() {}\n").expect("a dump");
        let archive = work.join("core.tar.gz");
        let packed = Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(&dumps)
            .args(["target.toml", "num"])
            .status()
            .expect("tar");
        assert!(packed.success());
        let digest = sha256_hex(&std::fs::read(&archive).expect("the archive"));
        let url = format!("file://{}", archive.display());
        let cache = work.join("cache");
        let got = link_dumps(&format!("{url}#sha256={digest}"), &cache).expect("fetched");
        assert_eq!(got, cache.join(&digest));
        assert_eq!(
            std::fs::read_to_string(got.join("num").join("core.f.mir")).expect("unpacked"),
            "fn f() {}\n"
        );
        // Taken again from the cache, unfetched: the archive is gone.
        std::fs::remove_file(&archive).expect("removed");
        assert_eq!(
            link_dumps(&format!("{url}#sha256={digest}"), &cache).ok(),
            Some(cache.join(&digest))
        );
        // Another archive under the digest named: refused, nothing kept.
        std::fs::write(&archive, b"not the archive").expect("replaced");
        let other = "0".repeat(64);
        let refused = link_dumps(&format!("{url}#sha256={other}"), &cache);
        assert!(
            refused
                .as_ref()
                .is_err_and(|e| e.to_string().contains(&format!("the link names {other}"))),
            "{refused:?}"
        );
        assert!(!cache.join(&other).exists());
        assert!(link_dumps(&format!("{url}#sha256=xyz"), &cache).is_err());
        assert_eq!(
            link_dumps("target/core-mir", &cache).ok(),
            Some(PathBuf::from("target/core-mir"))
        );
        let _ = std::fs::remove_dir_all(&work);
    }

    /// A run that panicked is refused with the panic's place and message,
    /// not the note printed after it; an abort with its last line.
    #[test]
    fn link_a_crate_that_ends_its_run_is_refused_with_why() {
        let panicked = "\nthread 'main' (7) panicked at src/x.rs:3:31:\nindex out of bounds\nnote: run with `RUST_BACKTRACE=1`\n";
        assert_eq!(
            why_ended(panicked),
            "panicked at src/x.rs:3:31: index out of bounds"
        );
        let aborted = "thread 'main' (7) has overflowed its stack\nfatal runtime error: stack overflow, aborting\n";
        assert_eq!(
            why_ended(aborted),
            "fatal runtime error: stack overflow, aborting"
        );
    }

    // What a type name means: a registry crate's alias, read from its own
    // source at the locked version.

    /// A cargo home whose cache holds one archive of a synthetic hasher
    /// crate, and the lock entry that pins it.
    fn type_names_cache(tag: &str) -> (PathBuf, crate::facts::LockedPackage) {
        let home =
            std::env::temp_dir().join(format!("uorust-aliases-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        let src = home.join("work/fastmap-1.0.0/src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.parent().unwrap().join("Cargo.toml"),
            "[package]\nname = \"fastmap\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        std::fs::write(
            src.join("lib.rs"),
            "#![no_std]\n#[cfg(feature = \"std\")]\nextern crate std;\n\
             use core::hash::BuildHasherDefault;\n\
             #[cfg(feature = \"std\")]\nuse std::collections::{HashMap, HashSet};\n\
             pub struct FastHasher { hash: usize }\n\
             #[cfg(feature = \"std\")]\n\
             pub type FastMap<K, V> = HashMap<K, V, BuildHasherDefault<FastHasher>>;\n\
             pub type FastSet<V> = HashSet<V, BuildHasherDefault<FastHasher>>;\n\
             type Private = u8;\n",
        )
        .unwrap();
        let cache = home.join("registry/cache/index-x");
        std::fs::create_dir_all(&cache).unwrap();
        let archive = cache.join("fastmap-1.0.0.crate");
        let status = Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(home.join("work"))
            .arg("fastmap-1.0.0")
            .status()
            .unwrap();
        assert!(status.success());
        let checksum: String = Sha256::digest(std::fs::read(&archive).unwrap())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let locked = crate::facts::LockedPackage {
            name: "fastmap".to_string(),
            version: "1.0.0".to_string(),
            source: "registry+https://github.com/rust-lang/crates.io-index".to_string(),
            checksum,
        };
        (home, locked)
    }

    #[test]
    fn type_names_a_crate_alias_reads_as_the_standard_map_it_names() {
        let (home, locked) = type_names_cache("read");
        let aliases = crate_aliases(&home, "fastmap", std::slice::from_ref(&locked))
            .expect("the archive at the locked checksum reads");
        let _ = std::fs::remove_dir_all(&home);
        assert_eq!(
            aliases.keys().collect::<Vec<_>>(),
            ["fastmap::FastMap", "fastmap::FastSet"]
        );
        assert_eq!(
            aliases["fastmap::FastMap"].to.text,
            "std::collections::HashMap<K, V, core::hash::BuildHasherDefault<fastmap::FastHasher>>"
        );
        // The workspace writes the alias; the lift reads the standard map.
        let written: syn::Type = syn::parse_str("fastmap::FastMap<u8, String>").unwrap();
        let read = crate::convert::ty(&written).expanded(&aliases, aliases.len());
        let types = crate::lift::Types::default();
        assert!(
            matches!(types.ty(&read), Ok(crate::lift::Ty::Map(_, _, false))),
            "{}",
            read.text
        );
        let set: syn::Type = syn::parse_str("fastmap::FastSet<u8>").unwrap();
        let read = crate::convert::ty(&set).expanded(&aliases, aliases.len());
        assert!(matches!(
            types.ty(&read),
            Ok(crate::lift::Ty::Set(_, false))
        ));
    }

    #[test]
    fn source_alias_reads_preserve_scope_arity_and_cycle_guards_ue_120() {
        use crate::lift::{Ty, Types};
        let (home, locked) = type_names_cache("source-annotations");
        let mut aliases = crate_aliases(&home, "fastmap", &[locked]).unwrap();
        std::fs::remove_dir_all(home).unwrap();
        let read = |name: &str| crate::convert::ty(&syn::parse_str::<syn::Type>(name).unwrap());
        aliases.insert(
            "Loop".into(),
            crate::ast::Alias {
                params: vec![],
                to: read("Vec<Loop>"),
            },
        );
        let mut types = Types {
            source_aliases: std::sync::Arc::new(aliases),
            ..Types::default()
        };
        types
            .aliases
            .insert("Local".into(), "fastmap::FastMap".into());
        assert!(
            matches!(types.ty(&read("Option<Local<u8,u16>>")), Ok(Ty::Option(inner)) if matches!(*inner, Ty::Map(_,_,false)))
        );
        assert!(types.ty(&read("Local<u8>")).is_err());
        assert!(types.ty(&read("Loop")).is_err());
        types.generic.insert("Local".into(), Ty::Param("T".into()));
        assert_eq!(types.ty(&read("Local")).unwrap(), Ty::Param("T".into()));
        types.generic.clear();
        types.aliases.insert("Local".into(), "Foreign".into());
        assert!(types.ty(&read("Local<u8,u16>")).is_err());
    }

    #[test]
    fn type_names_an_archive_not_at_the_locked_checksum_is_not_read() {
        let (home, mut locked) = type_names_cache("checksum");
        locked.checksum = "0".repeat(64);
        let read = crate_aliases(&home, "fastmap", std::slice::from_ref(&locked));
        let _ = std::fs::remove_dir_all(&home);
        // The archive is in the cache: only its checksum keeps it unread.
        assert!(read.is_none());
    }
}
