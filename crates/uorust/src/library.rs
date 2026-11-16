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
const PROTOCOL: &str = "uorust/library-mir/1";

/// The code that decides a dump's bytes besides the inputs [`key`] names:
/// this module's steps and the layouts it asks rustc for. A change of
/// either changes the key, as `scripts/mir/ledger.py mir-key` digests
/// `dump-lib.sh` and `layouts.rs`.
const CODE: [&str; 2] = [include_str!("library.rs"), include_str!("layouts.rs")];

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
    /// The host triple the library is built for.
    pub triple: String,
    /// That target's data layout, as its specification states it.
    pub data_layout: String,
}

/// The key of a dump of `inputs`: the SHA-256 of [`PROTOCOL`], the crate,
/// the toolchain, the rust-src digest, the triple, the data layout, the
/// flags, and the digest of [`CODE`], each ended by a NUL. Two dumps of one
/// key hold the same bytes; the modules asked for are the record's, not the
/// key's (a dump of every module is the one a link reads).
pub fn key(inputs: &Inputs) -> String {
    let code = sha256_hex(CODE.concat().as_bytes());
    let parts = [
        PROTOCOL,
        &inputs.krate,
        &inputs.toolchain,
        &inputs.source_sha256,
        &inputs.triple,
        &inputs.data_layout,
        FLAGS,
        CHECK_FLAGS,
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

/// The [`Inputs`] of a dump of `krate` on the pinned toolchain, read from
/// the toolchain alone (run in `dir`).
pub fn inputs(krate: &str, dir: &Path) -> Result<Inputs, UorustError> {
    if !LIBRARIES.contains(&krate) {
        return Err(tool(
            "uorust library-mir",
            format!("`{krate}` is none of {}", LIBRARIES.join(", ")),
        ));
    }
    let toolchain = format!("+{}", crate::mir::TOOLCHAIN);
    let sysroot = crate::shard::sysroot(dir, Some(&toolchain))?;
    let (source_url, source_sha256) = source_of(Path::new(&sysroot))?;
    let triple = crate::shard::host(dir, Some(&toolchain))?;
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

/// The file in a source cache that names the archive digest its
/// `library` was unpacked from.
const SOURCE_DIGEST: &str = "rust-src.sha256";

/// rust-src's `library` directory under `cache`, unpacked from the archive
/// `inputs` names and taken only at its digest: a cache that records
/// another digest, or none, is fetched again.
pub fn source(inputs: &Inputs, cache: &Path) -> Result<PathBuf, UorustError> {
    let library = cache.join("library");
    let recorded = std::fs::read_to_string(cache.join(SOURCE_DIGEST)).unwrap_or_default();
    if recorded.trim() == inputs.source_sha256 && library.join(&inputs.krate).is_dir() {
        return Ok(library);
    }
    let _ = std::fs::remove_dir_all(cache);
    std::fs::create_dir_all(cache).map_err(|e| io(cache, e))?;
    let archive = cache.join("rust-src.tar.xz");
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
        let _ = std::fs::remove_dir_all(cache);
        return Err(tool(
            "rust-src fetch",
            format!(
                "{} is {found}, the channel manifest records {}",
                inputs.source_url, inputs.source_sha256
            ),
        ));
    }
    output(
        "tar",
        Command::new("tar")
            .arg("-xJf")
            .arg(&archive)
            .arg("-C")
            .arg(cache),
    )?;
    std::fs::remove_file(&archive).map_err(|e| io(&archive, e))?;
    // The archive holds `rust-src-<v>/rust-src/lib/rustlib/src/rust/library`.
    let unpacked = crate::embed::dir_paths(cache)?
        .into_iter()
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("rust-src-"))
        })
        .ok_or_else(|| tool("rust-src", "the archive holds no rust-src-* directory"))?;
    let from = unpacked.join("rust-src/lib/rustlib/src/rust/library");
    std::fs::rename(&from, &library).map_err(|e| io(&from, e))?;
    std::fs::remove_dir_all(&unpacked).map_err(|e| io(&unpacked, e))?;
    let file = cache.join(SOURCE_DIGEST);
    std::fs::write(&file, format!("{}\n", inputs.source_sha256)).map_err(|e| io(&file, e))?;
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

/// [`RECORD`] of a dump of `inputs`'s crate: its key, its inputs, the
/// modules kept, and the digest of what was written.
pub fn render_record(inputs: &Inputs, kept: &[Kept], layouts: usize, digest: &str) -> String {
    let quote = |s: &str| toml::Value::String(s.to_string()).to_string();
    let mut out = format!(
        "# The MIR of a library of the pinned toolchain's rust-src\n\
         # (written by `uorust library-mir`; do not edit).\n\n\
         crate = {}\nkey = {}\ntoolchain = {}\nsource_url = {}\nsource_sha256 = {}\n\
         triple = {}\ndata_layout = {}\nlayouts = {layouts}\nfiles = {}\nbytes = {}\ndigest = {}\n",
        quote(&inputs.krate),
        quote(&key(inputs)),
        quote(&inputs.toolchain),
        quote(&inputs.source_url),
        quote(&inputs.source_sha256),
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

/// Dump `krate`'s MIR into `out`, one directory per module (every module
/// when `only` is empty), with rust-src unpacked under `cache` and the
/// build done under `work`, which is removed afterwards; write
/// `target.toml`, `layouts.json` and [`RECORD`] beside the modules.
pub fn dump(
    krate: &str,
    out: &Path,
    only: &[String],
    cache: &Path,
    work: &Path,
) -> Result<Dumped, UorustError> {
    std::fs::create_dir_all(out).map_err(|e| io(out, e))?;
    let inputs = inputs(krate, out)?;
    let library = source(&inputs, cache)?;
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
                .args([verb, "-q", &format!("-Zbuild-std={krate}")])
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
             -Zbuild-std={krate} --target {}",
            probe.display(),
            copy.display(),
            target.display(),
            inputs.triple
        );
        let layouts = crate::layouts::run(krate, out, &copy, &check)?;
        Ok::<_, UorustError>((kept, layouts))
    })();
    let _ = std::fs::remove_dir_all(work);
    let (kept, layouts) = result?;
    let digest = digest(out)?;
    let file = out.join(RECORD);
    std::fs::write(&file, render_record(&inputs, &kept, layouts, &digest))
        .map_err(|e| io(&file, e))?;
    Ok(Dumped {
        key: key(&inputs),
        digest,
        library,
        kept,
        layouts,
    })
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
        let k = key(&base);
        assert_eq!(k, key(&base.clone()));
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
            assert_ne!(key(c), k, "{c:?}");
        }
        // The archive is named by its digest: a mirror is the same input.
        let mirrored = Inputs {
            source_url: "file:///mirror/rust-src.tar.xz".into(),
            ..base
        };
        assert_eq!(key(&mirrored), k);
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
        let record = render_record(
            &stdlink_inputs(),
            &[Kept {
                module: "num".into(),
                files: 1,
                bytes: 11,
            }],
            0,
            &d,
        );
        let doc: toml::Table = record.parse().unwrap();
        assert_eq!(doc["key"].as_str(), Some(key(&stdlink_inputs()).as_str()));
        assert_eq!(doc["digest"].as_str(), Some(d.as_str()));
        assert_eq!(doc["files"].as_integer(), Some(1));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stdlink_a_source_cache_is_taken_only_at_the_manifests_digest() {
        let dir = stdlink_dir("source");
        let cache = dir.join("cache");
        let inputs = stdlink_inputs();
        stdlink_write(&cache.join("library/core/src/lib.rs"), "");
        stdlink_write(
            &cache.join(SOURCE_DIGEST),
            &format!("{}\n", inputs.source_sha256),
        );
        assert_eq!(source(&inputs, &cache).unwrap(), cache.join("library"));
        // Another digest recorded: the cache is fetched again, and the
        // fetch of an unreachable archive fails rather than keeping it.
        let other = Inputs {
            source_url: format!("file://{}", dir.join("absent.tar.xz").display()),
            source_sha256: "22".repeat(32),
            ..inputs
        };
        assert!(source(&other, &cache).is_err());
        assert!(!cache.join("library").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
