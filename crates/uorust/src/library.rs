//! The MIR of a library crate of the pinned toolchain's rust-src (core,
//! alloc, std), dumped by the tool itself, so that a workspace's embedding
//! can link the calls its bodies make into the library
//! (`uorust embed --link core=DIR`) without a forge or an artifact.
//!
//! `uorust library-mir CRATE --out DIR` does what `scripts/mir/dump-lib.sh`
//! does, step for step: rust-src of the pinned toolchain, fetched and taken
//! only at the digest the toolchain's own channel manifest records; CRATE
//! built from it with `-Zbuild-std` and a `-Zdump-mir` filter; each of
//! CRATE's own bodies' `ElaborateBoxDerefs.before` dump kept under
//! `DIR/<module>`, this machine's paths resolved away; `target.toml`; and
//! the layouts rustc chose (`crate::layouts::run`). It adds
//! [`RECORD`]: the dump's key ([`key`], a function of the declared inputs
//! alone, so that a factory can name the obligation before it runs it)
//! and the digest of every file it wrote ([`digest`]), so that a consumer
//! declares the dump as an obligation keyed on its inputs and takes its
//! output only at that digest.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

use crate::error::{io, UorustError};

/// The libraries of rust-src a workspace's bodies call into, each dumped
/// by itself.
pub const LIBRARIES: [&str; 3] = ["core", "alloc", "std"];

/// The record [`dump`] writes beside the dumps: the key and the digest.
pub const RECORD: &str = "library-mir.toml";

/// The text that opens the key's preimage: the protocol of [`key`], so
/// that a change of what the key covers changes every key.
const PROTOCOL: &str = "uorust/library-mir/2";

/// The code that decides a dump's bytes besides the inputs [`key`] names:
/// every source file of the tool, and the lock file that fixes the
/// versions of the crates it is built with. The dump's path reaches far
/// past this module: `target.toml` is written by `shard::write_target`,
/// and `layouts.json` by `layouts::run`, which reads the dumps through
/// `embed` and `mir` (`instance_types`, `structs`, `core_decls`,
/// `substitute`), and those reach most of the crate. A list of the files
/// it reaches would have to follow every call to stay true; the whole
/// source cannot miss one. A file of the crate this list does not name
/// fails `stdlink_the_code_digest_names_every_source_file_of_the_tool`.
const CODE: [(&str, &str); 50] = [
    ("src/admit.rs", include_str!("admit.rs")),
    ("src/assets.rs", include_str!("assets.rs")),
    ("src/ast.rs", include_str!("ast.rs")),
    ("src/bridge.rs", include_str!("bridge.rs")),
    ("src/certified.rs", include_str!("certified.rs")),
    ("src/certify.rs", include_str!("certify.rs")),
    ("src/certify/cases.rs", include_str!("certify/cases.rs")),
    ("src/certify/profile.rs", include_str!("certify/profile.rs")),
    ("src/convert.rs", include_str!("convert.rs")),
    ("src/embed.rs", include_str!("embed.rs")),
    ("src/error.rs", include_str!("error.rs")),
    ("src/expand.rs", include_str!("expand.rs")),
    ("src/facts.rs", include_str!("facts.rs")),
    ("src/ingest.rs", include_str!("ingest.rs")),
    ("src/layouts.rs", include_str!("layouts.rs")),
    ("src/lex.rs", include_str!("lex.rs")),
    ("src/lib.rs", include_str!("lib.rs")),
    ("src/library.rs", include_str!("library.rs")),
    ("src/lift.rs", include_str!("lift.rs")),
    ("src/main.rs", include_str!("main.rs")),
    ("src/mir/core.rs", include_str!("mir/core.rs")),
    ("src/mir/coverage.rs", include_str!("mir/coverage.rs")),
    ("src/mir/float.rs", include_str!("mir/float.rs")),
    ("src/mir/generate.rs", include_str!("mir/generate.rs")),
    ("src/mir/int.rs", include_str!("mir/int.rs")),
    ("src/mir/intrinsic.rs", include_str!("mir/intrinsic.rs")),
    ("src/mir/ir.rs", include_str!("mir/ir.rs")),
    ("src/mir/layout.rs", include_str!("mir/layout.rs")),
    ("src/mir/link.rs", include_str!("mir/link.rs")),
    ("src/mir/lower.rs", include_str!("mir/lower.rs")),
    ("src/mir/memory.rs", include_str!("mir/memory.rs")),
    ("src/mir/mod.rs", include_str!("mir/mod.rs")),
    ("src/mir/parse.rs", include_str!("mir/parse.rs")),
    ("src/mir/prove.rs", include_str!("mir/prove.rs")),
    ("src/mir/site.rs", include_str!("mir/site.rs")),
    ("src/mir/spec.rs", include_str!("mir/spec.rs")),
    ("src/mir/std_vectors.rs", include_str!("mir/std_vectors.rs")),
    ("src/mir/stdlib.rs", include_str!("mir/stdlib.rs")),
    ("src/mir/trips.rs", include_str!("mir/trips.rs")),
    ("src/mir/verdict.rs", include_str!("mir/verdict.rs")),
    ("src/names.rs", include_str!("names.rs")),
    ("src/order.rs", include_str!("order.rs")),
    ("src/project.rs", include_str!("project.rs")),
    ("src/queue.rs", include_str!("queue.rs")),
    ("src/register.rs", include_str!("register.rs")),
    ("src/registry.rs", include_str!("registry.rs")),
    ("src/route.rs", include_str!("route.rs")),
    ("src/shard.rs", include_str!("shard.rs")),
    ("src/wf.rs", include_str!("wf.rs")),
    ("Cargo.lock", include_str!("../../../Cargo.lock")),
];

/// The flags every dump is built with: spans printed (a call's site is the
/// location `Location::caller` reports), overflow checks on and debug
/// assertions off, as the evaluator reads core's bodies.
const FLAGS: &str = "-Zmir-include-spans=on -Coverflow-checks=on -Cdebug-assertions=off";

/// The flags of the layouts' check of the library's copy.
const CHECK_FLAGS: &str = "-Ztrim-diagnostic-paths=no -Coverflow-checks=on -Cdebug-assertions=off";

/// The pass whose input each body is kept at.
const PASS: &str = "ElaborateBoxDerefs";

fn tool(what: &str, reason: impl std::fmt::Display) -> UorustError {
    UorustError::Tool(format!("{what}: {reason}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn output(what: &str, command: &mut Command) -> Result<String, UorustError> {
    let out = command.output().map_err(|e| tool(what, e))?;
    if !out.status.success() {
        return Err(tool(what, String::from_utf8_lossy(&out.stderr)));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// What a dump is a function of, read before anything is built: the
/// toolchain, the digest of its rust-src archive (its channel manifest's),
/// the host it builds for and that host's data layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inputs {
    /// The library: core, alloc or std.
    pub krate: String,
    /// The toolchain's channel, [`crate::mir::TOOLCHAIN`].
    pub toolchain: String,
    /// The rust-src archive's URL in the channel manifest.
    pub source_url: String,
    /// Its SHA-256, as the channel manifest records it.
    pub source_sha256: String,
    /// The triple the library is built for (the host's unless asked).
    pub triple: String,
    /// That target's data layout, as its specification states it.
    pub data_layout: String,
}

/// The modules a dump of `only` holds, as [`key`] names them: every
/// module of the crate's tree (`*`, a function of the rust-src digest)
/// when `only` is empty, else the modules asked for, sorted, each once.
pub fn selection(only: &[String]) -> String {
    if only.is_empty() {
        return "*".to_string();
    }
    let set: BTreeSet<&str> = only.iter().map(String::as_str).collect();
    set.into_iter().collect::<Vec<_>>().join(",")
}

/// The key of a dump of `inputs` holding the modules `only` asks for: the
/// SHA-256 of [`PROTOCOL`], the crate, the toolchain, the rust-src digest,
/// the triple, the data layout, the flags, the module [`selection`], and
/// the digest of [`CODE`] (each file's path and text), each ended by a
/// NUL. Two dumps of one key hold the same bytes: the library's text is a
/// function of the archive's digest ([`source`] unpacks it anew from an
/// archive checked against that digest), the target's of the triple and
/// its layout, which modules are kept of the selection, and every step
/// of the tool of [`CODE`].
pub fn key(inputs: &Inputs, only: &[String]) -> String {
    let mut code = String::new();
    for (path, text) in CODE {
        code.push_str(path);
        code.push('\0');
        code.push_str(&sha256_hex(text.as_bytes()));
        code.push('\n');
    }
    let code = sha256_hex(code.as_bytes());
    let modules = selection(only);
    let parts = [
        PROTOCOL,
        &inputs.krate,
        &inputs.toolchain,
        &inputs.source_sha256,
        &inputs.triple,
        &inputs.data_layout,
        FLAGS,
        CHECK_FLAGS,
        &modules,
        &code,
    ];
    let mut text = String::new();
    for p in parts {
        text.push_str(p);
        text.push('\0');
    }
    sha256_hex(text.as_bytes())
}

/// The rust-src archive the channel manifest under `sysroot` records:
/// its URL and its SHA-256.
fn source_of(sysroot: &Path) -> Result<(String, String), UorustError> {
    let file = sysroot.join("lib/rustlib/multirust-channel-manifest.toml");
    let text = std::fs::read_to_string(&file).map_err(|e| io(&file, e))?;
    let doc: toml::Table = text.parse().map_err(|e| UorustError::Malformed {
        path: file.clone(),
        reason: format!("{e}"),
    })?;
    let target = doc
        .get("pkg")
        .and_then(|p| p.get("rust-src"))
        .and_then(|p| p.get("target"))
        .and_then(|t| t.get("*"));
    let field = |name: &str| {
        target
            .and_then(|t| t.get(name))
            .and_then(toml::Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| UorustError::Malformed {
                path: file.clone(),
                reason: format!("no pkg.rust-src.target.\"*\".{name}"),
            })
    };
    Ok((field("xz_url")?, field("xz_hash")?))
}

/// The [`Inputs`] of a dump of `krate` on the pinned toolchain for
/// `target` (the host when none), read from the toolchain alone (run in
/// `dir`). A library is compiled for one target: its `cfg`s choose its
/// items (`impl usize` is `u32`'s width on a 32-bit target), so a dump
/// is linked only into a program of the same triple and data layout
/// (`embed` refuses another).
pub fn inputs(krate: &str, dir: &Path, target: Option<&str>) -> Result<Inputs, UorustError> {
    if !LIBRARIES.contains(&krate) {
        return Err(tool(
            "uorust library-mir",
            format!("`{krate}` is none of {}", LIBRARIES.join(", ")),
        ));
    }
    let toolchain = format!("+{}", crate::mir::TOOLCHAIN);
    let sysroot = crate::shard::sysroot(dir, Some(&toolchain))?;
    let (source_url, source_sha256) = source_of(Path::new(&sysroot))?;
    let triple = match target {
        Some(t) => t.to_string(),
        None => crate::shard::host(dir, Some(&toolchain))?,
    };
    let spec: serde_json::Value = serde_json::from_str(&output(
        "rustc --print target-spec-json",
        Command::new("rustc")
            .arg(&toolchain)
            .args(["-Zunstable-options", "--print", "target-spec-json"])
            .args(["--target", &triple])
            .current_dir(dir)
            .env("RUSTC_BOOTSTRAP", "1"),
    )?)
    .map_err(|e| tool("rustc", e))?;
    let data_layout = spec
        .get("data-layout")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| tool("rustc", "no data layout"))?
        .to_string();
    Ok(Inputs {
        krate: krate.to_string(),
        toolchain: crate::mir::TOOLCHAIN.to_string(),
        source_url,
        source_sha256,
        triple,
        data_layout,
    })
}

/// The archive a source cache keeps: rust-src as its channel manifest
/// names it, checked against the manifest's digest each time it is read.
const SOURCE_ARCHIVE: &str = "rust-src.tar.xz";

/// rust-src's `library` directory under `cache`, unpacked anew from the
/// archive `inputs` names, every time: the archive kept in `cache` is
/// taken only while its bytes hash to the manifest's digest (else it is
/// fetched again and checked), and a `library` tree already in `cache` is
/// never reused, so that no file of it can differ from the archive's (a
/// label beside an unpacked tree says nothing of the tree).
pub fn source(inputs: &Inputs, cache: &Path) -> Result<PathBuf, UorustError> {
    std::fs::create_dir_all(cache).map_err(|e| io(cache, e))?;
    let library = cache.join("library");
    if library.exists() {
        std::fs::remove_dir_all(&library).map_err(|e| io(&library, e))?;
    }
    let archive = cache.join(SOURCE_ARCHIVE);
    let held = std::fs::read(&archive).is_ok_and(|b| sha256_hex(&b) == inputs.source_sha256);
    if !held {
        let _ = std::fs::remove_file(&archive);
        output(
            "rust-src fetch",
            Command::new("curl")
                .args(["-sSfL", "-o"])
                .arg(&archive)
                .arg(&inputs.source_url),
        )?;
        let bytes = std::fs::read(&archive).map_err(|e| io(&archive, e))?;
        let found = sha256_hex(&bytes);
        drop(bytes);
        if found != inputs.source_sha256 {
            let _ = std::fs::remove_file(&archive);
            return Err(tool(
                "rust-src fetch",
                format!(
                    "{} is {found}, the channel manifest records {}",
                    inputs.source_url, inputs.source_sha256
                ),
            ));
        }
    }
    let unpack = cache.join("unpack");
    let _ = std::fs::remove_dir_all(&unpack);
    std::fs::create_dir_all(&unpack).map_err(|e| io(&unpack, e))?;
    output(
        "tar",
        Command::new("tar")
            .arg("-xJf")
            .arg(&archive)
            .arg("-C")
            .arg(&unpack),
    )?;
    // The archive holds `rust-src-<v>/rust-src/lib/rustlib/src/rust/library`.
    let unpacked = crate::embed::dir_paths(&unpack)?
        .into_iter()
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("rust-src-"))
        })
        .ok_or_else(|| tool("rust-src", "the archive holds no rust-src-* directory"))?;
    let from = unpacked.join("rust-src/lib/rustlib/src/rust/library");
    std::fs::rename(&from, &library).map_err(|e| io(&from, e))?;
    std::fs::remove_dir_all(&unpack).map_err(|e| io(&unpack, e))?;
    Ok(library)
}

/// The top-level modules of `krate` under rust-src's `library`: each
/// directory and `.rs` file of its `src` but the crate root, and each
/// module the root declares at a path of its own inside `src`
/// (`#[path = "num/f32.rs"] pub mod f32;`); one at a path outside it (a
/// submodule's crate, `../../stdarch/..`) is that crate's. These are the
/// shards `dump-lib.sh --modules` lists.
pub fn modules(library: &Path, krate: &str) -> Result<Vec<String>, UorustError> {
    let src = library.join(krate).join("src");
    let mut out = BTreeSet::new();
    for p in crate::embed::dir_paths(&src)? {
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let stem = if p.is_dir() {
            name
        } else if let Some(stem) = name.strip_suffix(".rs") {
            stem
        } else {
            continue;
        };
        if stem != "lib" && stem != "lib.miri" {
            out.insert(stem.to_string());
        }
    }
    let root = src.join("lib.rs");
    let text = std::fs::read_to_string(&root).map_err(|e| io(&root, e))?;
    let mut path: Option<&str> = None;
    for line in text.lines() {
        let t = line.trim();
        if let Some(p) = t
            .strip_prefix("#[path")
            .map(|r| r.trim_start().trim_start_matches('=').trim_start())
            .and_then(|r| r.strip_prefix('"'))
            .and_then(|r| r.split('"').next())
        {
            path = Some(p);
            continue;
        }
        if let Some(name) = declared_module(t) {
            if path.is_some_and(|p| !p.starts_with("..")) {
                out.insert(name.to_string());
            }
        }
        if !t.starts_with('#') {
            path = None;
        }
    }
    Ok(out.into_iter().collect())
}

/// The name a line `[pub[(..)]] mod name;` declares, none for any other.
fn declared_module(line: &str) -> Option<&str> {
    let mut rest = line;
    if let Some(r) = rest.strip_prefix("pub") {
        rest = r.trim_start();
        if let Some(r) = rest.strip_prefix('(') {
            rest = r.split_once(')')?.1.trim_start();
        }
    }
    let rest = rest.strip_prefix("mod ")?.trim_start();
    let name = rest.strip_suffix(';')?.trim_end();
    (!name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'))
    .then_some(name)
}

/// The digest of the files under `dir` but [`RECORD`]: the SHA-256 of one
/// line per file, sorted by its path relative to `dir`, `path NUL
/// sha256(bytes)`, joined by newlines (`scripts/mir/ledger.py`'s tree κ).
pub fn digest(dir: &Path) -> Result<String, UorustError> {
    let mut lines = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(at) = stack.pop() {
        for p in crate::embed::dir_paths(&at)? {
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            let rel = p
                .strip_prefix(dir)
                .map(|r| r.to_string_lossy().into_owned())
                .unwrap_or_default();
            if rel == RECORD {
                continue;
            }
            let bytes = std::fs::read(&p).map_err(|e| io(&p, e))?;
            lines.push(format!("{rel}\0{}", sha256_hex(&bytes)));
        }
    }
    lines.sort();
    Ok(sha256_hex(lines.join("\n").as_bytes()))
}

/// Move each of `krate`'s own dumps of `module` from `mir` (the flat
/// `-Zdump-mir-dir`) into `dest`, with the paths of this machine resolved
/// away: rust-src's `library/` under its cache written `library/`, the
/// sysroot written `<sysroot>`. Returns the files and bytes kept.
pub fn keep_module(
    mir: &Path,
    dest: &Path,
    krate: &str,
    module: &str,
    library: &Path,
    sysroot: &str,
) -> Result<(usize, u64), UorustError> {
    if dest.exists() {
        std::fs::remove_dir_all(dest).map_err(|e| io(dest, e))?;
    }
    std::fs::create_dir_all(dest).map_err(|e| io(dest, e))?;
    let prefix = format!("{krate}.{module}-");
    let suffix = format!("{PASS}.before.mir");
    let cache = format!("{}/", library.display());
    let (mut files, mut bytes) = (0, 0);
    for p in crate::embed::dir_paths(mir)? {
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !(name.starts_with(&prefix) && name.ends_with(&suffix)) {
            continue;
        }
        let text = std::fs::read_to_string(&p).map_err(|e| io(&p, e))?;
        let text = text
            .replace(&cache, "library/")
            .replace(sysroot, "<sysroot>");
        let to = dest.join(name);
        std::fs::write(&to, &text).map_err(|e| io(&to, e))?;
        std::fs::remove_file(&p).map_err(|e| io(&p, e))?;
        files += 1;
        bytes += text.len() as u64;
    }
    Ok((files, bytes))
}

/// One module kept: its name, files and bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kept {
    /// The module.
    pub module: String,
    /// Its dump files.
    pub files: usize,
    /// Their bytes.
    pub bytes: u64,
}

/// [`RECORD`] of a dump of `inputs`'s crate holding the modules `only`
/// asks for: its key, its inputs, the module selection, the digest of the
/// rust-src tree it was built from (`source_tree`, [`digest`] of the
/// unpacked `library`: what an embedding's `--library` is checked
/// against), the modules kept, and the digest of what was written.
pub fn render_record(
    inputs: &Inputs,
    only: &[String],
    kept: &[Kept],
    layouts: usize,
    digest: &str,
    source_tree: &str,
) -> String {
    let quote = |s: &str| toml::Value::String(s.to_string()).to_string();
    let mut out = format!(
        "# The MIR of a library of the pinned toolchain's rust-src\n\
         # (written by `uorust library-mir`; do not edit).\n\n\
         crate = {}\nkey = {}\ntoolchain = {}\nsource_url = {}\nsource_sha256 = {}\n\
         source_tree = {}\nselection = {}\n\
         triple = {}\ndata_layout = {}\nlayouts = {layouts}\nfiles = {}\nbytes = {}\ndigest = {}\n",
        quote(&inputs.krate),
        quote(&key(inputs, only)),
        quote(&inputs.toolchain),
        quote(&inputs.source_url),
        quote(&inputs.source_sha256),
        quote(source_tree),
        quote(&selection(only)),
        quote(&inputs.triple),
        quote(&inputs.data_layout),
        kept.iter().map(|k| k.files).sum::<usize>(),
        kept.iter().map(|k| k.bytes).sum::<u64>(),
        quote(digest),
    );
    for k in kept {
        out.push_str(&format!(
            "\n[[module]]\nname = {}\nfiles = {}\nbytes = {}\n",
            quote(&k.module),
            k.files,
            k.bytes
        ));
    }
    out
}

/// The crates `-Zbuild-std` builds for `krate`: alloc with the core it
/// is built on named too, since a `no_std` probe takes its core from the
/// sysroot otherwise and two cores' lang items collide (E0152).
fn build_std(krate: &str) -> &'static str {
    match krate {
        "core" => "core",
        "alloc" => "core,alloc",
        _ => "std",
    }
}

/// What [`dump`] wrote.
#[derive(Clone, Debug)]
pub struct Dumped {
    /// The key of the dump.
    pub key: String,
    /// The digest of what it wrote ([`digest`]).
    pub digest: String,
    /// rust-src's `library` directory the dumps were built from: what
    /// `uorust embed --library` takes.
    pub library: PathBuf,
    /// The modules kept.
    pub kept: Vec<Kept>,
    /// The layouts written.
    pub layouts: usize,
}

/// Dump `krate`'s MIR for `target` (the host when none) into `out`, one
/// directory per module (every module when `only` is empty), with
/// rust-src unpacked under `cache` and the build done under `work`, which
/// is removed afterwards; write `target.toml`, `layouts.json` and
/// [`RECORD`] beside the modules.
pub fn dump(
    krate: &str,
    target: Option<&str>,
    out: &Path,
    only: &[String],
    cache: &Path,
    work: &Path,
) -> Result<Dumped, UorustError> {
    std::fs::create_dir_all(out).map_err(|e| io(out, e))?;
    let inputs = inputs(krate, out, target)?;
    let library = source(&inputs, cache)?;
    let source_tree = digest(&library)?;
    let library = std::fs::canonicalize(&library).map_err(|e| io(&library, e))?;
    let toolchain = format!("+{}", inputs.toolchain);
    let sysroot = crate::shard::sysroot(out, Some(&toolchain))?;
    let all = modules(&library, krate)?;
    for m in only {
        if !all.contains(m) {
            return Err(tool(
                "uorust library-mir",
                format!("`{m}` is no top-level module of {krate}"),
            ));
        }
    }
    let wanted: Vec<String> = if only.is_empty() { all } else { only.to_vec() };
    crate::shard::write_target(out, &inputs.triple, out, Some(&toolchain))?;
    let _ = std::fs::remove_dir_all(work);
    let result = (|| {
        let probe = work.join("probe");
        let src = probe.join("src");
        std::fs::create_dir_all(&src).map_err(|e| io(&src, e))?;
        let manifest = probe.join("Cargo.toml");
        std::fs::write(
            &manifest,
            "[package]\nname = \"probe\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n",
        )
        .map_err(|e| io(&manifest, e))?;
        // A probe that links the crate and nothing more.
        let root = src.join("lib.rs");
        let text = match krate {
            "core" => "#![no_std]\n",
            "alloc" => "#![no_std]\nextern crate alloc;\n",
            _ => "\n",
        };
        std::fs::write(&root, text).map_err(|e| io(&root, e))?;
        // One build dumps every module asked for: a module's filter where
        // there is one, the pass alone (every item) where there are several.
        let filter = match wanted.as_slice() {
            [one] => format!("{PASS}&{one}::"),
            _ => PASS.to_string(),
        };
        let mir = work.join("mir");
        std::fs::create_dir_all(&mir).map_err(|e| io(&mir, e))?;
        let target = work.join("target");
        let cargo = |lib: &Path, flags: String, verb: &str| {
            let mut c = Command::new("cargo");
            c.arg(&toolchain)
                .args([verb, "-q", &format!("-Zbuild-std={}", build_std(krate))])
                .args(["--target", &inputs.triple])
                .current_dir(&probe)
                .env("__CARGO_TESTS_ONLY_SRC_ROOT", lib)
                .env("RUSTC_BOOTSTRAP", "1")
                .env("CARGO_INCREMENTAL", "0")
                .env("CARGO_TARGET_DIR", &target)
                .env_remove("CARGO_BUILD_TARGET_DIR")
                .env_remove("CARGO_BUILD_BUILD_DIR")
                .env_remove("CARGO_ENCODED_RUSTFLAGS")
                .env("RUSTFLAGS", flags);
            c
        };
        output(
            "cargo build -Zbuild-std",
            &mut cargo(
                &library,
                format!(
                    "{FLAGS} -Zdump-mir={filter} -Zdump-mir-dir={}",
                    mir.display()
                ),
                "build",
            ),
        )?;
        std::fs::remove_dir_all(&target).map_err(|e| io(&target, e))?;
        let mut kept = Vec::new();
        for m in &wanted {
            let (files, bytes) = keep_module(&mir, &out.join(m), krate, m, &library, &sysroot)?;
            kept.push(Kept {
                module: m.clone(),
                files,
                bytes,
            });
        }
        std::fs::remove_dir_all(&mir).map_err(|e| io(&mir, e))?;
        // The layouts rustc chose, asked in a copy of the library.
        let copy = work.join("library");
        output("cp", Command::new("cp").arg("-r").arg(&library).arg(&copy))?;
        let check = format!(
            "cd '{}' && __CARGO_TESTS_ONLY_SRC_ROOT='{}' RUSTC_BOOTSTRAP=1 CARGO_INCREMENTAL=0 \
             CARGO_TARGET_DIR='{}' RUSTFLAGS='{CHECK_FLAGS}' cargo {toolchain} check -q \
             -Zbuild-std={} --target {}",
            probe.display(),
            copy.display(),
            target.display(),
            build_std(krate),
            inputs.triple
        );
        let layouts = crate::layouts::run(krate, out, &copy, &check)?;
        Ok::<_, UorustError>((kept, layouts))
    })();
    let _ = std::fs::remove_dir_all(work);
    let (kept, layouts) = result?;
    let digest = digest(out)?;
    let file = out.join(RECORD);
    std::fs::write(
        &file,
        render_record(&inputs, only, &kept, layouts, &digest, &source_tree),
    )
    .map_err(|e| io(&file, e))?;
    Ok(Dumped {
        key: key(&inputs, only),
        digest,
        library,
        kept,
        layouts,
    })
}

/// What one library crate of rust-src makes nameable from outside: its
/// modules, the items each declares, and each module's `pub use` names
/// and globs, every target written from a crate's root
/// (`["core", "option"]`).
#[derive(Clone, Debug, Default)]
struct Exports {
    /// Every module, by its path from the crate's root ("" the root).
    modules: BTreeSet<String>,
    /// (module, name) of every item a module declares, but modules.
    items: BTreeSet<(String, String)>,
    /// (module, name) of every `pub use` name, with its target.
    named: std::collections::BTreeMap<(String, String), Vec<String>>,
    /// Every `pub use target::*` of a module, by the module.
    globs: std::collections::BTreeMap<String, Vec<Vec<String>>>,
}

/// The public paths of the library crates of rust-src, read from their
/// sources: how a path a workspace's MIR prints (rustc prints an item by a
/// path visible from the caller's crate, `std::vec::Vec`,
/// `std::cmp::max`) names the item a library defines (`alloc::vec::Vec`,
/// `core::cmp::max`), by Rust's own resolution of the `pub use`
/// declarations it passes through.
///
/// The reading holds for every program because it is name resolution as
/// the language states it, applied to the library's own source at the
/// pinned toolchain: in a module, an item the module declares or a name
/// it imports explicitly shadows a glob import; a name two globs bring
/// in is ambiguous, and an ambiguous name is never taken here; `crate`,
/// `self`, `super` and an extern crate's name (its `extern crate .. as`
/// alias included) start a path as the language reads them. A path that
/// does not resolve so is left as printed, so it stays unlinked rather
/// than linked to another item.
#[derive(Clone, Debug, Default)]
pub struct Paths {
    by_crate: std::collections::BTreeMap<String, Exports>,
}

impl Paths {
    /// The public paths of core, alloc and std under rust-src's `library`.
    pub fn read(library: &Path) -> Paths {
        let by_crate = LIBRARIES
            .iter()
            .map(|k| (k.to_string(), exports(library, k)))
            .collect();
        Paths { by_crate }
    }

    /// `text` (a callee, or a type, as MIR prints it) with each path that
    /// starts with a library's name written from the crate that defines
    /// what it names (`std::vec::Vec::<u32>::push` as
    /// `alloc::vec::Vec::<u32>::push`); a path that does not resolve is
    /// kept as printed.
    pub fn canonical_text(&self, text: &str) -> String {
        let bytes = text.as_bytes();
        let ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        let mut out = String::with_capacity(text.len());
        let mut i = 0;
        while i < bytes.len() {
            let at_start = i == 0 || !(ident(bytes[i - 1]) || bytes[i - 1] == b':');
            let lib = at_start
                .then(|| {
                    self.by_crate
                        .keys()
                        .find(|k| text[i..].starts_with(&format!("{k}::")))
                })
                .flatten();
            let Some(_) = lib else {
                let next = text[i..]
                    .char_indices()
                    .nth(1)
                    .map_or(text.len(), |(n, _)| i + n);
                out.push_str(&text[i..next]);
                i = next;
                continue;
            };
            // The run `ident(::ident)*` from here.
            let mut end = i;
            let mut segs: Vec<String> = Vec::new();
            loop {
                let from = end;
                while end < bytes.len() && ident(bytes[end]) {
                    end += 1;
                }
                if end == from {
                    break;
                }
                segs.push(text[from..end].to_string());
                if text[end..].starts_with("::") && bytes.get(end + 2).copied().is_some_and(ident) {
                    end += 2;
                } else {
                    break;
                }
            }
            let run_end = end;
            match self.resolve(&segs, &mut BTreeSet::new()) {
                Some(found) => out.push_str(&found.join("::")),
                None => out.push_str(&text[i..run_end]),
            }
            i = run_end;
        }
        out
    }

    /// The path `segs` (a crate's name first, then identifiers) names, from
    /// the crate that declares it: `None` where a segment names nothing
    /// the crate's modules, items or `pub use` names give, or a glob
    /// import leaves it ambiguous.
    fn resolve(&self, segs: &[String], seen: &mut BTreeSet<String>) -> Option<Vec<String>> {
        let ex = self.by_crate.get(segs.first()?)?;
        if !seen.insert(segs.join("::")) {
            return None;
        }
        let mut module: Vec<&str> = Vec::new();
        for (i, s) in segs.iter().enumerate().skip(1) {
            let m = module.join("::");
            if ex.items.contains(&(m.clone(), s.clone())) {
                return Some(segs.to_vec());
            }
            let child = if m.is_empty() {
                s.clone()
            } else {
                format!("{m}::{s}")
            };
            if ex.modules.contains(&child) {
                module.push(s);
                continue;
            }
            if let Some(t) = ex.named.get(&(m.clone(), s.clone())) {
                let mut next = t.clone();
                next.extend(segs[i + 1..].iter().cloned());
                return self.resolve(&next, seen);
            }
            let mut found: Vec<Vec<String>> = ex
                .globs
                .get(&m)
                .into_iter()
                .flatten()
                .filter_map(|g| {
                    let mut next = g.clone();
                    next.extend(segs[i..].iter().cloned());
                    self.resolve(&next, &mut seen.clone())
                })
                .collect();
            found.sort();
            found.dedup();
            return match found.as_slice() {
                [one] => Some(one.clone()),
                _ => None,
            };
        }
        Some(segs.to_vec())
    }
}

/// `text` with each path that starts with `krate::` written from that
/// crate's root (`core::ops::range::Range<u32>` as `ops::range::Range<u32>`),
/// as the crate's own dumps print its items; a `krate::` inside a longer
/// path is kept.
pub fn without_crate(text: &str, krate: &str) -> String {
    let prefix = format!("{krate}::");
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        let at_start = i == 0
            || !(bytes[i - 1].is_ascii_alphanumeric()
                || bytes[i - 1] == b'_'
                || bytes[i - 1] == b':');
        if at_start && text[i..].starts_with(&prefix) {
            i += prefix.len();
            continue;
        }
        let next = text[i..]
            .char_indices()
            .nth(1)
            .map_or(text.len(), |(n, _)| i + n);
        out.push_str(&text[i..next]);
        i = next;
    }
    out
}

/// [`Exports`] of `krate` under rust-src's `library`: each source file is
/// the module its path names (`src/a/b.rs`, `src/a/b/mod.rs`: `a::b`),
/// except a root module the crate root mounts by `#[path]`.
fn exports(library: &Path, krate: &str) -> Exports {
    fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
        for p in crate::embed::dir_paths(dir).unwrap_or_default() {
            if p.is_dir() {
                walk(&p, files);
            } else if p.extension().is_some_and(|x| x == "rs") {
                files.push(p);
            }
        }
    }
    let src = library.join(krate).join("src");
    let mut files = Vec::new();
    walk(&src, &mut files);
    // The extern crates a path may start with: the prelude's, and each
    // `extern crate x as y;` of the crate root.
    let mut externs: std::collections::BTreeMap<String, String> = LIBRARIES
        .iter()
        .take_while(|k| **k != krate)
        .chain(std::iter::once(&krate))
        .map(|k| (k.to_string(), k.to_string()))
        .collect();
    let mut mounted: std::collections::BTreeMap<PathBuf, String> = Default::default();
    let root = src.join("lib.rs");
    if let Some(parsed) = parse_items(&root) {
        for item in &parsed {
            match item {
                syn::Item::ExternCrate(e) => {
                    let name = e.ident.to_string();
                    let name = if name == "self" {
                        krate.to_string()
                    } else {
                        name
                    };
                    let alias = e
                        .rename
                        .as_ref()
                        .map_or_else(|| name.clone(), |(_, r)| r.to_string());
                    externs.insert(alias, name);
                }
                syn::Item::Mod(m) if m.content.is_none() => {
                    let path = m.attrs.iter().find_map(|a| match &a.meta {
                        syn::Meta::NameValue(nv) if nv.path.is_ident("path") => match &nv.value {
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(s),
                                ..
                            }) => Some(s.value()),
                            _ => None,
                        },
                        _ => None,
                    });
                    if let Some(p) = path.filter(|p| !p.starts_with("..")) {
                        mounted.insert(src.join(p), m.ident.to_string());
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = Exports::default();
    out.modules.insert(String::new());
    for file in files {
        let Some(parsed) = parse_items(&file) else {
            continue;
        };
        let module: Vec<String> = match mounted.get(&file) {
            Some(m) => vec![m.clone()],
            None => {
                let rel = file.strip_prefix(&src).unwrap_or(&file).with_extension("");
                let mut m: Vec<String> = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect();
                if matches!(m.last().map(String::as_str), Some("mod" | "lib")) {
                    m.pop();
                }
                m
            }
        };
        export_items(krate, &externs, &module, &parsed, &mut out);
    }
    out
}

/// Record the modules, items and public `use` names of `items`, the
/// items of `module` of `krate`, into `out`.
fn export_items(
    krate: &str,
    externs: &std::collections::BTreeMap<String, String>,
    module: &[String],
    items: &[syn::Item],
    out: &mut Exports,
) {
    let m = module.join("::");
    for i in items {
        let declared = match i {
            syn::Item::Mod(md) => {
                let mut path = module.to_vec();
                path.push(md.ident.to_string());
                out.modules.insert(path.join("::"));
                if let Some((_, inner)) = &md.content {
                    export_items(krate, externs, &path, inner, out);
                }
                None
            }
            syn::Item::Fn(f) => Some(&f.sig.ident),
            syn::Item::Struct(s) => Some(&s.ident),
            syn::Item::Enum(e) => Some(&e.ident),
            syn::Item::Union(u) => Some(&u.ident),
            syn::Item::Trait(t) => Some(&t.ident),
            syn::Item::TraitAlias(t) => Some(&t.ident),
            syn::Item::Type(t) => Some(&t.ident),
            syn::Item::Const(c) => Some(&c.ident),
            syn::Item::Static(s) => Some(&s.ident),
            _ => None,
        };
        if let Some(name) = declared {
            out.items.insert((m.clone(), name.to_string()));
        }
        let syn::Item::Use(u) = i else {
            continue;
        };
        if !matches!(u.vis, syn::Visibility::Public(_)) {
            continue;
        }
        let mut found = Vec::new();
        use_paths(&[], &u.tree, &mut found);
        for (name, path) in found {
            let Some(target) = use_target(krate, externs, module, &path) else {
                continue;
            };
            match name {
                Some(n) => {
                    out.named.insert((m.clone(), n), target);
                }
                None => out.globs.entry(m.clone()).or_default().push(target),
            }
        }
    }
}

/// A source file's items, read past the const-trait syntax the library is
/// written in and syn does not read (`const trait`, `impl const`,
/// `[const]` and `~const` bounds): what is left declares the same names.
fn parse_items(file: &Path) -> Option<Vec<syn::Item>> {
    let text = std::fs::read_to_string(file).ok()?;
    // syn reads a `const trait` as tokens it does not parse
    // (`Item::Verbatim`), so the syntax is left out before it reads.
    let plain = text
        .replace("const trait ", "trait ")
        .replace("impl const ", "impl ")
        .replace("> const ", "> ")
        .replace("[const] ", "")
        .replace("~const ", "");
    syn::parse_file(&plain)
        .or_else(|_| syn::parse_file(&text))
        .ok()
        .map(|f| f.items)
}

/// Each name a use tree brings in with the path it names (a glob's name
/// none, its path the module), in the tree's own words.
fn use_paths(prefix: &[String], tree: &syn::UseTree, out: &mut Vec<(Option<String>, Vec<String>)>) {
    let with = |s: String| {
        let mut p = prefix.to_vec();
        p.push(s);
        p
    };
    match tree {
        syn::UseTree::Path(p) => use_paths(&with(p.ident.to_string()), &p.tree, out),
        syn::UseTree::Name(n) if n.ident == "self" => {
            if let Some(last) = prefix.last() {
                out.push((Some(last.clone()), prefix.to_vec()));
            }
        }
        syn::UseTree::Name(n) => out.push((Some(n.ident.to_string()), with(n.ident.to_string()))),
        syn::UseTree::Rename(r) if r.rename != "_" => {
            out.push((Some(r.rename.to_string()), with(r.ident.to_string())));
        }
        syn::UseTree::Rename(_) => {}
        syn::UseTree::Glob(_) => out.push((None, prefix.to_vec())),
        syn::UseTree::Group(g) => {
            for t in &g.items {
                use_paths(prefix, t, out);
            }
        }
    }
}

/// The path `path`, written in a `use` of `module` of `krate`, names from
/// a crate's root, its crate's name first: `crate`, `self`, `super` and an
/// extern crate's name as the language reads them; a path that starts
/// with any other name is relative to the module (the 2018 edition's
/// uniform paths).
fn use_target(
    krate: &str,
    externs: &std::collections::BTreeMap<String, String>,
    module: &[String],
    path: &[String],
) -> Option<Vec<String>> {
    let first = path.first()?;
    let here = || {
        let mut m = vec![krate.to_string()];
        m.extend(module.iter().cloned());
        m
    };
    let (mut at, rest) = match first.as_str() {
        "crate" => (vec![krate.to_string()], &path[1..]),
        "self" => (here(), &path[1..]),
        "super" => (here(), path),
        f => match externs.get(f) {
            Some(k) => (vec![k.clone()], &path[1..]),
            None => (here(), path),
        },
    };
    for s in rest {
        match s.as_str() {
            "super" if at.len() > 1 => {
                at.pop();
            }
            "super" | "self" => {}
            _ => at.push(s.clone()),
        }
    }
    Some(at)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stdlink_inputs() -> Inputs {
        Inputs {
            krate: "core".into(),
            toolchain: "1.0.0".into(),
            source_url: "https://example.invalid/rust-src.tar.xz".into(),
            source_sha256: "00".repeat(32),
            triple: "x86_64-unknown-linux-gnu".into(),
            data_layout: "e-m:e".into(),
        }
    }

    fn stdlink_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("uorust-stdlink-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn stdlink_write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn stdlink_the_key_moves_with_every_input_but_not_the_url() {
        let base = stdlink_inputs();
        let k = key(&base, &[]);
        assert_eq!(k, key(&base.clone(), &[]));
        let changed = [
            Inputs {
                krate: "alloc".into(),
                ..base.clone()
            },
            Inputs {
                toolchain: "1.0.1".into(),
                ..base.clone()
            },
            Inputs {
                source_sha256: "11".repeat(32),
                ..base.clone()
            },
            Inputs {
                triple: "aarch64-unknown-linux-gnu".into(),
                ..base.clone()
            },
            Inputs {
                data_layout: "E-m:e".into(),
                ..base.clone()
            },
        ];
        for c in &changed {
            assert_ne!(key(c, &[]), k, "{c:?}");
        }
        // The archive is named by its digest: a mirror is the same input.
        let mirrored = Inputs {
            source_url: "file:///mirror/rust-src.tar.xz".into(),
            ..base
        };
        assert_eq!(key(&mirrored, &[]), k);
    }

    #[test]
    fn stdlink_a_dump_of_some_modules_has_another_key_than_one_of_every_module() {
        // refute-stdlink-7 item 3: `--only hint` (1 module) and the whole
        // crate (67) printed one key.
        let base = stdlink_inputs();
        let every = key(&base, &[]);
        let hint = key(&base, &["hint".to_string()]);
        assert_ne!(hint, every);
        // The selection is a set: order and repetition name the same dump.
        let two = key(&base, &["num".to_string(), "hint".to_string()]);
        assert_eq!(
            two,
            key(
                &base,
                &["hint".to_string(), "num".to_string(), "hint".to_string()]
            )
        );
        assert_ne!(two, hint);
        assert_ne!(two, every);
    }

    #[test]
    fn stdlink_the_code_digest_names_every_source_file_of_the_tool() {
        // refute-stdlink-7 item 4: `target.toml` and `layouts.json` are
        // written by code outside this module (shard.rs, embed.rs, mir/).
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut found = BTreeSet::new();
        let mut stack = vec![src.clone()];
        while let Some(at) = stack.pop() {
            for p in crate::embed::dir_paths(&at).unwrap() {
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    let rel = p.strip_prefix(&src).unwrap().to_string_lossy();
                    found.insert(format!("src/{rel}"));
                }
            }
        }
        let named: BTreeSet<String> = CODE
            .iter()
            .map(|(p, _)| p.to_string())
            .filter(|p| p.starts_with("src/"))
            .collect();
        assert_eq!(named, found);
        for module in [
            "src/shard.rs",
            "src/embed.rs",
            "src/mir/core.rs",
            "src/layouts.rs",
        ] {
            assert!(named.contains(module), "{module}");
        }
        assert!(CODE.iter().any(|(p, _)| *p == "Cargo.lock"));
        // Each file's text is what it holds on disk.
        for (path, text) in CODE.iter().filter(|(p, _)| p.starts_with("src/")) {
            let on_disk = std::fs::read_to_string(src.join(&path[4..])).unwrap();
            assert_eq!(*text, on_disk, "{path}");
        }
    }

    #[test]
    fn stdlink_modules_are_the_crates_top_level_files_and_its_own_path_modules() {
        let dir = stdlink_dir("modules");
        let src = dir.join("library/core/src");
        stdlink_write(
            &src.join("lib.rs"),
            "pub mod num;\n#[path = \"num/f32.rs\"]\npub mod f32;\n#[path = \"../../stdarch/crates/core_arch/src/mod.rs\"]\n#[allow(missing_docs)]\npub mod arch;\n#[path = \"x.rs\"]\n\npub(crate) mod not_after_a_blank;\nmod ops;\n",
        );
        stdlink_write(&src.join("lib.miri.rs"), "");
        stdlink_write(&src.join("num/mod.rs"), "");
        stdlink_write(&src.join("num/f32.rs"), "");
        stdlink_write(&src.join("ops.rs"), "");
        stdlink_write(&src.join("README.md"), "");
        let found = modules(&dir.join("library"), "core").unwrap();
        assert_eq!(found, ["f32", "num", "ops"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stdlink_a_module_keeps_its_own_dumps_with_the_machines_paths_resolved() {
        let dir = stdlink_dir("keep");
        let mir = dir.join("mir");
        let lib = dir.join("cache/library");
        let sysroot = "/opt/toolchains/1.0.0";
        let body = format!(
            "// MIR for `num::f`\nfn num::f() -> () {{ // at {}/core/src/num/mod.rs:1:1: 1:2\n    // {sysroot}/lib/x\n}}\n",
            lib.display()
        );
        stdlink_write(
            &mir.join("core.num-f.-------.ElaborateBoxDerefs.before.mir"),
            &body,
        );
        stdlink_write(
            &mir.join("core.num-f.-------.ElaborateBoxDerefs.after.mir"),
            &body,
        );
        stdlink_write(
            &mir.join("core.num_x-g.-------.ElaborateBoxDerefs.before.mir"),
            &body,
        );
        stdlink_write(
            &mir.join("alloc.num-h.-------.ElaborateBoxDerefs.before.mir"),
            &body,
        );
        let dest = dir.join("out/num");
        stdlink_write(&dest.join("stale.mir"), "");
        let (files, bytes) = keep_module(&mir, &dest, "core", "num", &lib, sysroot).unwrap();
        assert_eq!(files, 1);
        let kept = crate::embed::dir_paths(&dest).unwrap();
        assert_eq!(kept.len(), 1);
        let text = std::fs::read_to_string(&kept[0]).unwrap();
        assert_eq!(bytes, text.len() as u64);
        assert!(text.contains("at library/core/src/num/mod.rs"), "{text}");
        assert!(text.contains("// <sysroot>/lib/x"), "{text}");
        assert!(!text.contains(&dir.display().to_string()), "{text}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stdlink_the_digest_covers_every_written_file_but_the_record() {
        let dir = stdlink_dir("digest");
        stdlink_write(&dir.join("target.toml"), "triple = \"t\"\n");
        stdlink_write(&dir.join("num/a.mir"), "fn a() {}\n");
        let d = digest(&dir).unwrap();
        stdlink_write(&dir.join(RECORD), "digest = \"x\"\n");
        assert_eq!(digest(&dir).unwrap(), d);
        stdlink_write(&dir.join("num/a.mir"), "fn a() { }\n");
        assert_ne!(digest(&dir).unwrap(), d);
        stdlink_write(&dir.join("num/a.mir"), "fn a() {}\n");
        assert_eq!(digest(&dir).unwrap(), d);
        std::fs::rename(dir.join("num/a.mir"), dir.join("num/b.mir")).unwrap();
        assert_ne!(digest(&dir).unwrap(), d);
        let only = ["num".to_string()];
        let record = render_record(
            &stdlink_inputs(),
            &only,
            &[Kept {
                module: "num".into(),
                files: 1,
                bytes: 11,
            }],
            0,
            &d,
            "ab",
        );
        let doc: toml::Table = record.parse().unwrap();
        assert_eq!(
            doc["key"].as_str(),
            Some(key(&stdlink_inputs(), &only).as_str())
        );
        assert_eq!(doc["selection"].as_str(), Some("num"));
        assert_eq!(doc["source_tree"].as_str(), Some("ab"));
        assert_eq!(doc["digest"].as_str(), Some(d.as_str()));
        assert_eq!(doc["files"].as_integer(), Some(1));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A rust-src archive as the channel names one, holding `files`
    /// under `library/`; its URL and digest.
    fn stdlink_archive(dir: &Path, name: &str, files: &[(&str, &str)]) -> (String, String) {
        let stage = dir.join(format!("stage-{name}"));
        let root = stage.join("rust-src-1.0.0/rust-src/lib/rustlib/src/rust/library");
        for (path, text) in files {
            stdlink_write(&root.join(path), text);
        }
        let archive = dir.join(format!("{name}.tar.xz"));
        let ok = Command::new("tar")
            .arg("-cJf")
            .arg(&archive)
            .arg("-C")
            .arg(&stage)
            .arg("rust-src-1.0.0")
            .status()
            .unwrap()
            .success();
        assert!(ok);
        let sha = sha256_hex(&std::fs::read(&archive).unwrap());
        (format!("file://{}", archive.display()), sha)
    }

    #[test]
    fn stdlink_a_source_cache_is_taken_only_at_the_manifests_digest() {
        let dir = stdlink_dir("source");
        let cache = dir.join("cache");
        let hint = "pub fn spin_loop() {}\n";
        let (url, sha) = stdlink_archive(&dir, "good", &[("core/src/hint.rs", hint)]);
        let inputs = Inputs {
            source_url: url,
            source_sha256: sha,
            ..stdlink_inputs()
        };
        let library = source(&inputs, &cache).unwrap();
        assert_eq!(library, cache.join("library"));
        let file = library.join("core/src/hint.rs");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), hint);
        // refute-stdlink-7 item 2: the unpacked tree edited after the
        // fetch, a body planted, a file added. The cache's archive still
        // hashes to the digest, so the tree is unpacked from it again and
        // nothing planted survives.
        stdlink_write(&file, "pub fn spin_loop() {}\npub fn refute_plant() {}\n");
        stdlink_write(&library.join("core/src/planted.rs"), "");
        let again = source(&inputs, &cache).unwrap();
        assert_eq!(
            std::fs::read_to_string(again.join("core/src/hint.rs")).unwrap(),
            hint
        );
        assert!(!again.join("core/src/planted.rs").exists());
        // The kept archive replaced by another one's bytes: it no longer
        // hashes to the digest and is fetched again from the URL.
        let (_, _) = stdlink_archive(&dir, "bad", &[("core/src/hint.rs", "pub fn x() {}\n")]);
        std::fs::copy(dir.join("bad.tar.xz"), cache.join(SOURCE_ARCHIVE)).unwrap();
        let third = source(&inputs, &cache).unwrap();
        assert_eq!(
            std::fs::read_to_string(third.join("core/src/hint.rs")).unwrap(),
            hint
        );
        // An archive at the URL that is not the digest's: refused, and no
        // tree is left to be taken.
        let other = Inputs {
            source_sha256: "22".repeat(32),
            ..inputs
        };
        assert!(source(&other, &cache).is_err());
        assert!(!cache.join(SOURCE_ARCHIVE).exists());
        assert!(!cache.join("library").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn stdlink_library(dir: &Path) -> PathBuf {
        let lib = dir.join("library");
        let w = |p: &str, t: &str| stdlink_write(&lib.join(p), t);
        w(
            "core/src/lib.rs",
            "pub mod option;\npub mod cmp;\npub mod clone;\n",
        );
        w(
            "core/src/option.rs",
            "pub enum Option<T> {\n    None,\n    Some(T),\n}\n",
        );
        w(
            "core/src/cmp.rs",
            "pub fn max<T>(a: T, b: T) -> T {\n    a\n}\n",
        );
        w(
            "core/src/clone.rs",
            "pub const trait Clone {\n    fn clone(&self) -> Self;\n}\n",
        );
        w("alloc/src/lib.rs", "#![no_std]\npub mod vec;\n");
        w("alloc/src/vec/mod.rs", "pub struct Vec<T>(T);\n");
        w(
            "std/src/lib.rs",
            "extern crate alloc as alloc_crate;\npub use core::option;\npub use core::cmp;\n\
             pub use core::clone;\npub use alloc_crate::vec;\npub mod collections;\n\
             #[path = \"num/f64.rs\"]\npub mod f64;\npub mod alloc;\n\
             pub mod g1 {\n    pub fn f() {}\n}\npub mod g2 {\n    pub fn f() {}\n}\n\
             pub mod amb {\n    pub use crate::g1::*;\n    pub use crate::g2::*;\n}\n\
             pub mod sh {\n    pub use crate::g1::*;\n    pub fn f() {}\n}\n\
             pub mod hidden {\n    pub(crate) use core::option::Option;\n}\n",
        );
        w("std/src/alloc.rs", "pub struct System;\n");
        w("std/src/num/f64.rs", "pub fn local() {}\n");
        w(
            "std/src/collections/mod.rs",
            "pub use self::hash_map::HashMap;\nmod hash;\npub mod hash_map {\n    pub use super::hash::map::*;\n}\n",
        );
        w("std/src/collections/hash/mod.rs", "pub mod map;\n");
        w(
            "std/src/collections/hash/map.rs",
            "pub struct HashMap<K, V>(K, V);\n",
        );
        lib
    }

    #[test]
    fn stdlink_a_printed_library_path_is_written_from_the_crate_that_defines_it() {
        let dir = stdlink_dir("paths");
        let paths = Paths::read(&stdlink_library(&dir));
        let c = |t: &str| paths.canonical_text(t);
        // Through a re-export of another crate's module, `extern crate .. as`
        // included.
        assert_eq!(
            c("std::vec::Vec::<u32>::push"),
            "alloc::vec::Vec::<u32>::push"
        );
        assert_eq!(c("std::cmp::max::<u64>"), "core::cmp::max::<u64>");
        // Inside a qualified path and its generic arguments; a trait the
        // library writes `const trait` is read.
        assert_eq!(
            c("<std::option::Option<std::vec::Vec<u8>> as std::clone::Clone>::clone"),
            "<core::option::Option<alloc::vec::Vec<u8>> as core::clone::Clone>::clone"
        );
        // A name, then a glob of an inline module, to a private module's item.
        assert_eq!(
            c("std::collections::HashMap::<K, V>::new"),
            "std::collections::hash::map::HashMap::<K, V>::new"
        );
        // The crate's own module, mounted by `#[path]`, and its own `alloc`.
        assert_eq!(
            c("std::f64::<impl f64>::sqrt"),
            "std::f64::<impl f64>::sqrt"
        );
        assert_eq!(c("std::f64::local"), "std::f64::local");
        assert_eq!(c("std::alloc::System"), "std::alloc::System");
        // A name two globs bring in is ambiguous and kept; an item the
        // module declares shadows its glob; a `pub(crate) use` is not a
        // public path; a path inside a longer one is not a library's.
        assert_eq!(c("std::amb::f"), "std::amb::f");
        assert_eq!(c("std::sh::f"), "std::sh::f");
        assert_eq!(c("std::hidden::Option"), "std::hidden::Option");
        assert_eq!(c("my::std::vec::Vec"), "my::std::vec::Vec");
        assert_eq!(c("mystd::vec::Vec"), "mystd::vec::Vec");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stdlink_a_crates_own_paths_are_written_from_its_root() {
        assert_eq!(
            without_crate(
                "<core::ops::Range<core::num::NonZero<u32>> as core::iter::Iterator>::next",
                "core"
            ),
            "<ops::Range<num::NonZero<u32>> as iter::Iterator>::next"
        );
        assert_eq!(without_crate("x::core::y", "core"), "x::core::y");
        assert_eq!(without_crate("alloc::vec::Vec", "core"), "alloc::vec::Vec");
    }
}
