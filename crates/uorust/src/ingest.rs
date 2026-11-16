//! Ingest: `cargo metadata`, the manifests, `Cargo.lock` and every source file,
//! parsed with `syn`, into [`facts`](crate::facts) with spans.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use quote::ToTokens;
use serde_json::Value;
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::visit::Visit;

use crate::error::{io, UorustError};
use crate::facts::{
    Advisories, Advisory, Capability, Dep, EnumFact, FileSyntax, FnFact, FnSyntax, LockedPackage,
    Package, ScopeFact, SourceFile, Span, StructFact, Syntax, Target, TargetKind, Variant,
    Workspace,
};

/// Read a workspace from its root directory: its facts, and the syntax the
/// lift reads.
pub(crate) fn workspace(
    root: &Path,
    advisory_db: Option<&Path>,
) -> Result<(Workspace, Syntax), UorustError> {
    let root = root.canonicalize().map_err(|e| io(root, e))?;
    let metadata = metadata(&root)?;
    let root_manifest = read_toml(&root.join("Cargo.toml"))?;
    let workspace_lints = lints_of(root_manifest.get("workspace").and_then(|w| w.get("lints")));
    let members: Vec<&Value> = metadata
        .get("packages")
        .and_then(Value::as_array)
        .map(|p| p.iter().collect())
        .unwrap_or_default();
    let mut packages = Vec::new();
    let mut syntax = Syntax::default();
    for member in members {
        let (package, files) = package(&root, member, &workspace_lints)?;
        packages.push(package);
        syntax.files.extend(files);
    }
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    let lock = lock(&root.join("Cargo.lock"))?;
    let deny = root.join("deny.toml");
    let deny_ignores = if deny.is_file() {
        ignores(&read_toml(&deny)?)
    } else {
        BTreeMap::new()
    };
    let advisories = match advisory_db {
        Some(db) => Some(advisories(db, &lock)?),
        None => None,
    };
    let toolchain = root.join("rust-toolchain.toml");
    let targets = if toolchain.is_file() {
        declared_targets(&read_toml(&toolchain)?)
    } else {
        Vec::new()
    };
    Ok((
        Workspace {
            lock,
            has_deny: deny.is_file(),
            deny_ignores,
            advisories,
            packages,
            targets,
        },
        syntax,
    ))
}

/// Whether an item's generics are lifetimes alone, which say how long a
/// borrow lives and which a LexLean value does not have: they are erased.
fn only_lifetimes(generics: &syn::Generics) -> bool {
    generics
        .params
        .iter()
        .all(|p| matches!(p, syn::GenericParam::Lifetime(_)))
}

/// `[toolchain] targets` of `rust-toolchain.toml`.
fn declared_targets(toolchain: &toml::Value) -> Vec<String> {
    toolchain
        .get("toolchain")
        .and_then(|t| t.get("targets"))
        .and_then(|t| t.as_array())
        .map(|ts| {
            ts.iter()
                .filter_map(|t| t.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// `[advisories] ignore` of `deny.toml`: bare IDs, or `{ id, reason }`.
fn ignores(deny: &toml::Value) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let list = deny
        .get("advisories")
        .and_then(|a| a.get("ignore"))
        .and_then(toml::Value::as_array);
    for item in list.into_iter().flatten() {
        match item {
            toml::Value::String(id) => {
                out.insert(id.clone(), String::new());
            }
            toml::Value::Table(t) => {
                if let Some(id) = t.get("id").and_then(toml::Value::as_str) {
                    let reason = t.get("reason").and_then(toml::Value::as_str).unwrap_or("");
                    out.insert(id.to_string(), reason.to_string());
                }
            }
            _ => {}
        }
    }
    out
}

/// The advisories of a RustSec advisory database checkout (`crates/<name>/
/// RUSTSEC-*.md`) that affect a locked registry package's version.
fn advisories(db: &Path, lock: &[LockedPackage]) -> Result<Advisories, UorustError> {
    let crates = db.join("crates");
    if !crates.is_dir() {
        return Err(UorustError::Malformed {
            path: db.to_path_buf(),
            reason: "not a RustSec advisory database: no crates directory".to_string(),
        });
    }
    // The checkout's commit, when the database is a repository of its own;
    // a directory inside another repository has no revision of its own.
    let git = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(db)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    let own_repo = git(&["rev-parse", "--show-toplevel"])
        .and_then(|top| PathBuf::from(top).canonicalize().ok())
        .is_some_and(|top| db.canonicalize().is_ok_and(|d| d == top));
    let revision = own_repo
        .then(|| git(&["rev-parse", "HEAD"]))
        .flatten()
        .unwrap_or_else(|| "unversioned".to_string());
    let mut hits = Vec::new();
    for package in lock.iter().filter(|p| p.source.starts_with("registry+")) {
        let dir = crates.join(&package.name);
        if !dir.is_dir() {
            continue;
        }
        let Ok(version) = semver::Version::parse(&package.version) else {
            continue;
        };
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .map_err(|e| io(&dir, e))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "md"))
            .collect();
        files.sort();
        for file in files {
            let text = std::fs::read_to_string(&file).map_err(|e| io(&file, e))?;
            if let Some(advisory) = advisory(&file, &text, &package.name, &version)? {
                hits.push(advisory);
            }
        }
    }
    hits.sort_by(|a, b| (&a.id, &a.package, &a.version).cmp(&(&b.id, &b.package, &b.version)));
    Ok(Advisories { revision, hits })
}

/// One advisory file, when it applies to `version`.
fn advisory(
    file: &Path,
    text: &str,
    package: &str,
    version: &semver::Version,
) -> Result<Option<Advisory>, UorustError> {
    let malformed = |reason: &str| UorustError::Malformed {
        path: file.to_path_buf(),
        reason: reason.to_string(),
    };
    let body = text
        .strip_prefix("```toml")
        .ok_or_else(|| malformed("no TOML front matter"))?;
    let (front, rest) = body
        .split_once("\n```")
        .ok_or_else(|| malformed("unterminated TOML front matter"))?;
    let front: toml::Value = toml::from_str(front).map_err(|e| malformed(&e.to_string()))?;
    let section = front
        .get("advisory")
        .ok_or_else(|| malformed("no [advisory] table"))?;
    let text_of = |key: &str| {
        section
            .get(key)
            .and_then(toml::Value::as_str)
            .map(str::to_string)
    };
    if section.get("withdrawn").is_some() || text_of("package").as_deref() != Some(package) {
        return Ok(None);
    }
    let reqs = |key: &str| -> Vec<String> {
        front
            .get("versions")
            .and_then(|v| v.get(key))
            .and_then(toml::Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(toml::Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    };
    let patched = reqs("patched");
    let unaffected = reqs("unaffected");
    let spared = patched
        .iter()
        .chain(&unaffected)
        .any(|r| semver::VersionReq::parse(r).is_ok_and(|req| req.matches(version)));
    if spared {
        return Ok(None);
    }
    let title = rest
        .lines()
        .find_map(|l| l.strip_prefix("# "))
        .unwrap_or("")
        .trim()
        .to_string();
    Ok(Some(Advisory {
        id: text_of("id").ok_or_else(|| malformed("no advisory id"))?,
        package: package.to_string(),
        version: version.to_string(),
        title,
        date: text_of("date").unwrap_or_default(),
        informational: text_of("informational"),
        categories: section
            .get("categories")
            .and_then(toml::Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(toml::Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        patched,
    }))
}

fn metadata(root: &Path) -> Result<Value, UorustError> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .current_dir(root)
        .output()
        .map_err(|e| UorustError::CargoMetadata(e.to_string()))?;
    if !output.status.success() {
        return Err(UorustError::CargoMetadata(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| UorustError::Malformed {
        path: root.join("Cargo.toml"),
        reason: e.to_string(),
    })
}

fn read_toml(path: &Path) -> Result<toml::Value, UorustError> {
    let text = std::fs::read_to_string(path).map_err(|e| io(path, e))?;
    text.parse::<toml::Value>()
        .map_err(|e| UorustError::Malformed {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })
}

fn lints_of(table: Option<&toml::Value>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Some(groups) = table.and_then(toml::Value::as_table) else {
        return out;
    };
    for (group, lints) in groups {
        let Some(lints) = lints.as_table() else {
            continue;
        };
        for (lint, level) in lints {
            let level = level
                .as_str()
                .map(str::to_string)
                .or_else(|| {
                    level
                        .get("level")
                        .and_then(toml::Value::as_str)
                        .map(str::to_string)
                })
                .unwrap_or_default();
            out.insert(format!("{group}.{lint}"), level);
        }
    }
    out
}

/// `path` relative to `root`, both absolute and as text with `/`
/// separators: below `root` the path under it, and elsewhere (a member or
/// a path dependency Cargo reaches by `..`, as in `members = ["../m"]`) one
/// `..` for each component of `root` past the two paths' common ancestor,
/// then the rest of `path`. Never the absolute path, so that nothing of
/// where the workspace lies on the machine enters a projection.
fn rel(root: &str, path: &str) -> String {
    let parts = |text: &str| -> Vec<String> {
        text.split('/')
            .filter(|c| !c.is_empty() && *c != ".")
            .map(str::to_string)
            .collect()
    };
    let (root, path) = (parts(root), parts(path));
    let common = root.iter().zip(&path).take_while(|(a, b)| a == b).count();
    let mut out: Vec<String> = vec!["..".to_string(); root.len() - common];
    out.extend(path[common..].iter().cloned());
    out.join("/")
}

/// The file `rel` names under `root` (as [`rel`] writes it), with each `..`
/// taken lexically, as Cargo forms a member's path from its manifest.
fn under(root: &Path, rel: &str) -> PathBuf {
    let mut out = root.to_path_buf();
    for part in rel.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            part => out.push(part),
        }
    }
    out
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn package(
    root: &Path,
    meta: &Value,
    workspace_lints: &BTreeMap<String, String>,
) -> Result<(Package, BTreeMap<String, FileSyntax>), UorustError> {
    // Paths as the facts hold them: text with `/` separators.
    let path_text = |path: &Path| path.to_string_lossy().replace('\\', "/");
    let root_text = path_text(root);
    let manifest = PathBuf::from(text(meta, "manifest_path"));
    let dir_abs = manifest.parent().unwrap_or(root).to_path_buf();
    let dir = rel(&root_text, &path_text(&dir_abs));
    let raw = read_toml(&manifest)?;
    let lints = match raw.get("lints") {
        Some(table) if table.get("workspace").and_then(toml::Value::as_bool) == Some(true) => {
            workspace_lints.clone()
        }
        other => lints_of(other),
    };
    let mut targets = Vec::new();
    for target in meta
        .get("targets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let kinds: Vec<&str> = target
            .get("kind")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        let kind = if kinds.iter().any(|k| {
            matches!(
                *k,
                "lib" | "rlib" | "cdylib" | "staticlib" | "dylib" | "proc-macro"
            )
        }) {
            TargetKind::Lib
        } else if kinds.contains(&"bin") {
            TargetKind::Bin
        } else if kinds.contains(&"test") {
            TargetKind::Test
        } else if kinds.contains(&"example") {
            TargetKind::Example
        } else if kinds.contains(&"bench") {
            TargetKind::Bench
        } else {
            continue;
        };
        targets.push(Target {
            name: text(target, "name"),
            kind,
            src: rel(&root_text, &path_text(Path::new(&text(target, "src_path")))),
        });
    }
    targets
        .sort_by(|a, b| (a.src.as_str(), a.name.as_str()).cmp(&(b.src.as_str(), b.name.as_str())));
    let mut deps = Vec::new();
    for dep in meta
        .get("dependencies")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let path = dep.get("path").and_then(Value::as_str).map(PathBuf::from);
        let inside = path.as_ref().is_some_and(|p| p.starts_with(root));
        deps.push(Dep {
            name: text(dep, "name"),
            kind: dep
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("normal")
                .to_string(),
            path: path.map(|p| rel(&root_text, &path_text(&p))),
            inside,
        });
    }
    deps.sort_by(|a, b| {
        (a.name.as_str(), a.kind.as_str()).cmp(&(b.name.as_str(), b.kind.as_str()))
    });
    let mut files = Vec::new();
    let mut syntax = BTreeMap::new();
    let mut paths = Vec::new();
    for sub in ["src", "tests", "examples", "benches"] {
        collect_rs(&dir_abs.join(sub), &mut paths)?;
    }
    paths.sort();
    // Each file is read and parsed once, and the files that are test code
    // as a whole are found across them before any file's facts are read.
    let mut read = Vec::new();
    for path in paths {
        let rel_path = rel(&root_text, &path_text(&path));
        let Some((target, module)) = assign(&rel_path, &dir, &targets) else {
            continue;
        };
        let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
        read.push((path, rel_path, target, module, syn::parse_file(&text)));
    }
    let roots: Vec<PathBuf> = targets.iter().map(|t| under(root, &t.src)).collect();
    let trees: BTreeMap<&Path, &syn::File> = read
        .iter()
        .filter_map(|(path, _, _, _, parsed)| Some((path.as_path(), parsed.as_ref().ok()?)))
        .collect();
    let tests = test_files(&roots, &trees);
    for (path, rel_path, target, module, parsed) in &read {
        let (file, file_syntax) = match parsed {
            Ok(tree) => source(
                tree,
                rel_path.clone(),
                *target,
                module.clone(),
                &targets[*target].kind,
                tests.contains(path),
            ),
            Err(error) => unparsed(rel_path.clone(), *target, module.clone(), error),
        };
        syntax.insert(file.rel.clone(), file_syntax);
        files.push(file);
    }
    let package = Package {
        name: text(meta, "name"),
        version: text(meta, "version"),
        description: text(meta, "description"),
        repository: text(meta, "repository"),
        homepage: text(meta, "homepage"),
        license: text(meta, "license"),
        rust_version: meta
            .get("rust_version")
            .and_then(Value::as_str)
            .map(str::to_string),
        authors: meta
            .get("authors")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        dir,
        targets,
        deps,
        lints,
        files,
    };
    Ok((package, syntax))
}

/// Every `.rs` file under `dir`, not descending into a nested package or a
/// build directory.
fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), UorustError> {
    if !dir.is_dir() {
        return Ok(());
    }
    let entries = std::fs::read_dir(dir).map_err(|e| io(dir, e))?;
    for entry in entries {
        let path = entry.map_err(|e| io(dir, e))?.path();
        if path.is_dir() {
            if path.join("Cargo.toml").exists()
                || path
                    .file_name()
                    .is_some_and(|n| n == "target" || n == "fixtures")
            {
                continue;
            }
            collect_rs(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

/// The target a file belongs to and its module path inside that target.
fn assign(file: &str, dir: &str, targets: &[Target]) -> Option<(usize, String)> {
    let prefix = if dir.is_empty() {
        String::new()
    } else {
        format!("{dir}/")
    };
    let local = file.strip_prefix(&prefix)?;
    if let Some(index) = targets.iter().position(|t| t.src == file) {
        return Some((index, String::new()));
    }
    let parent_of = |t: &Target| {
        t.src
            .rsplit_once('/')
            .map(|(p, _)| p.to_string())
            .unwrap_or_default()
    };
    // A file in a target root's own directory tree, the most specific root first.
    let mut best: Option<(usize, usize)> = None;
    for (index, target) in targets.iter().enumerate() {
        let base = parent_of(target);
        let in_tree = file.starts_with(&format!("{base}/"));
        if !in_tree {
            continue;
        }
        let shared_src = local.starts_with("src/") && base == format!("{prefix}src");
        // In `src/`, a bin root shares the tree with the library: the library owns it.
        let rank = base.len() * 4
            + match (&target.kind, shared_src) {
                (TargetKind::Lib, _) => 3,
                (TargetKind::Bin, true) => 0,
                _ => 2,
            };
        if best.is_none_or(|(_, r)| rank > r) {
            best = Some((index, rank));
        }
    }
    let (index, _) = best?;
    let base = parent_of(&targets[index]);
    let inner = file.strip_prefix(&format!("{base}/")).unwrap_or(file);
    let inner = inner.strip_suffix(".rs").unwrap_or(inner);
    let inner = inner.strip_suffix("/mod").unwrap_or(inner);
    Some((index, inner.replace('/', "::")))
}

fn lock(path: &Path) -> Result<Vec<LockedPackage>, UorustError> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let value = read_toml(path)?;
    let mut out: Vec<LockedPackage> = value
        .get("package")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .map(|p| {
            let field = |k: &str| {
                p.get(k)
                    .and_then(toml::Value::as_str)
                    .unwrap_or_default()
                    .to_string()
            };
            LockedPackage {
                name: field("name"),
                version: field("version"),
                source: field("source"),
                checksum: field("checksum"),
            }
        })
        .collect();
    out.sort_by(|a, b| {
        (a.name.as_str(), a.version.as_str()).cmp(&(b.name.as_str(), b.version.as_str()))
    });
    Ok(out)
}

/// A file a `mod` declaration names, as the walk over a package's module
/// tree meets it.
struct ModFile {
    file: PathBuf,
    /// The directory its own out-of-line modules are found from.
    dir: PathBuf,
    /// The directory under `dir` that holds them where the file is neither
    /// a `mod.rs` nor named by a `#[path]`: its own name.
    own: Option<String>,
    /// Whether the declaration that names it stands in test code.
    test: bool,
}

/// The files of a package that are test code as a whole: each file an
/// out-of-line `mod name;` names under a test cfg, in a test module or in a
/// test file, unless another declaration names it outside all three, since
/// a build without tests then holds it too. Read from the target roots down,
/// as Rust reads the module tree.
fn test_files(roots: &[PathBuf], trees: &BTreeMap<&Path, &syn::File>) -> BTreeSet<PathBuf> {
    // Per named file: whether a declaration outside test code names it.
    let mut named: BTreeMap<PathBuf, bool> = BTreeMap::new();
    let mut seen: BTreeSet<(PathBuf, bool)> = BTreeSet::new();
    let mut todo: Vec<ModFile> = roots
        .iter()
        .map(|root| ModFile {
            file: root.clone(),
            dir: parent(root),
            own: None,
            test: false,
        })
        .collect();
    while let Some(at) = todo.pop() {
        if !seen.insert((at.file.clone(), at.test)) {
            continue;
        }
        let Some(tree) = trees.get(at.file.as_path()) else {
            continue;
        };
        let mut found = Vec::new();
        declared_files(&tree.items, &at.dir, at.own.as_deref(), at.test, &mut found);
        for next in found {
            if trees.contains_key(next.file.as_path()) {
                *named.entry(next.file.clone()).or_insert(false) |= !next.test;
                todo.push(next);
            }
        }
    }
    named
        .into_iter()
        .filter(|(_, product)| !product)
        .map(|(file, _)| file)
        .collect()
}

/// The files the out-of-line modules among `items` name, by Rust's own
/// rule: `name.rs` or `name/mod.rs` in the directory of a target root, a
/// `mod.rs` or a `#[path]` file, and under the declaring file's own name
/// otherwise; the inline modules around a declaration are directories on
/// the way; and a `#[path]` names the file itself, or an inline module's
/// directory. Both files a plain declaration could mean are given: the one
/// that exists is the module.
fn declared_files(
    items: &[syn::Item],
    dir: &Path,
    own: Option<&str>,
    test: bool,
    out: &mut Vec<ModFile>,
) {
    for item in items {
        let syn::Item::Mod(m) = item else {
            continue;
        };
        let test = test || is_cfg_test(&attrs(&m.attrs));
        let name = m.ident.unraw().to_string();
        let under = match own {
            Some(own) => dir.join(own),
            None => dir.to_path_buf(),
        };
        match (&m.content, path_attr(&m.attrs)) {
            (Some((_, inner)), Some(given)) => {
                declared_files(inner, &normal(&dir.join(given)), None, test, out);
            }
            (Some((_, inner)), None) => {
                declared_files(inner, &under.join(&name), None, test, out);
            }
            (None, Some(given)) => {
                let file = normal(&dir.join(given));
                out.push(ModFile {
                    dir: parent(&file),
                    file,
                    own: None,
                    test,
                });
            }
            (None, None) => {
                out.push(ModFile {
                    file: under.join(format!("{name}.rs")),
                    dir: under.clone(),
                    own: Some(name.clone()),
                    test,
                });
                out.push(ModFile {
                    file: under.join(&name).join("mod.rs"),
                    dir: under.join(&name),
                    own: None,
                    test,
                });
            }
        }
    }
}

/// The file or directory a `#[path = "..."]` attribute gives.
fn path_attr(list: &[syn::Attribute]) -> Option<String> {
    list.iter().find_map(|a| match &a.meta {
        syn::Meta::NameValue(nv) if a.path().is_ident("path") => match &nv.value {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) => Some(s.value()),
            _ => None,
        },
        _ => None,
    })
}

/// The directory of a file.
fn parent(file: &Path) -> PathBuf {
    file.parent().map(Path::to_path_buf).unwrap_or_default()
}

/// A path without its `.` and `..` components, each `..` taking the
/// component before it: the path a `#[path]` that climbs out of its
/// directory names.
fn normal(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// Rust names a method `Type::name` while its type has one method of that
/// name, and `<Type as Trait>::name` once it has several: an inherent one
/// beside a trait's, two traits', or one generic trait's at two argument
/// lists. Each method of a trait implementation that shares its type and
/// name with another method of the file, in the same build (with tests or
/// without), is marked as named apart.
fn name_apart(fns: &mut [FnFact]) {
    let mut sharing: BTreeMap<(String, String, bool), usize> = BTreeMap::new();
    for f in fns.iter().filter(|f| !f.in_trait) {
        if let Some(owner) = &f.owner {
            *sharing
                .entry((owner.clone(), f.name.clone(), f.in_test_module))
                .or_default() += 1;
        }
    }
    for f in fns.iter_mut().filter(|f| f.trait_ref.is_some()) {
        if let Some(owner) = &f.owner {
            let key = (owner.clone(), f.name.clone(), f.in_test_module);
            f.apart = sharing.get(&key).is_some_and(|n| *n > 1);
        }
    }
}

/// The types among `items` that are declared in a shape the lift does not
/// read, each with that shape, so that a refusal over one can say why.
fn unread_types(items: &[syn::Item], out: &mut Vec<(String, &'static str)>) {
    for item in items {
        match item {
            syn::Item::Struct(st) => {
                // A struct's fields are read whatever its form, named or
                // by position; only its own parameters leave it unread.
                if !only_lifetimes(&st.generics) {
                    out.push((st.ident.to_string(), crate::lift::PARAMETRIC_STRUCT));
                }
            }
            syn::Item::Enum(e) if !only_lifetimes(&e.generics) => {
                out.push((e.ident.to_string(), "an enum with type parameters"));
            }
            syn::Item::Union(u) => out.push((u.ident.to_string(), "a union")),
            syn::Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    unread_types(inner, out);
                }
            }
            _ => {}
        }
    }
}

/// The facts of a file `syn` does not parse: where it stands and why it is
/// not read, with no item of it. The projection records the reason and the
/// names resolved through the file stay open.
fn unparsed(
    rel: String,
    target: usize,
    module: String,
    error: &syn::Error,
) -> (SourceFile, FileSyntax) {
    let at = error.span().start();
    let file = SourceFile {
        parse_error: Some(format!("{}:{}: {error}", rel, at.line)),
        rel,
        target,
        module,
        ..SourceFile::default()
    };
    (file, FileSyntax::default())
}

fn source(
    parsed: &syn::File,
    rel: String,
    target: usize,
    module: String,
    kind: &TargetKind,
    test_file: bool,
) -> (SourceFile, FileSyntax) {
    let mut file = SourceFile {
        rel: rel.clone(),
        target,
        module,
        ..SourceFile::default()
    };
    let mut syntax = FileSyntax::default();
    let file_attrs = attrs(&parsed.attrs);
    file.doc = doc_of(&file_attrs);
    file.no_prelude = file_attrs.iter().any(|a| named(a, "no_implicit_prelude"));
    file.forbids_unsafe = file_attrs
        .iter()
        .any(|a| named(a, "forbid") && a.tokens.contains("unsafe_code"));
    let product = matches!(kind, TargetKind::Lib | TargetKind::Bin);
    let lib = matches!(kind, TargetKind::Lib);
    let test_target = matches!(kind, TargetKind::Test);
    items(
        &parsed.items,
        &rel,
        None,
        test_target || test_file,
        product,
        lib,
        &mut file,
        &mut syntax,
    );
    name_apart(&mut file.fns);
    unread_types(&parsed.items, &mut file.unread);
    file.capabilities.sort();
    file.capabilities.dedup();
    (file, syntax)
}

/// An attribute, as the facts read it.
#[derive(Clone, Debug)]
struct Attr {
    /// The path, when it is one identifier (`cfg`, `doc`, `forbid`, ...).
    ident: Option<String>,
    /// The path's last segment (`test` of `tokio::test`).
    last: String,
    /// The whole attribute, printed.
    tokens: String,
    /// The text of a `#[doc = "..."]` line, trimmed.
    doc: Option<String>,
    /// The type names a `#[derive(...)]` lists.
    derives: Vec<String>,
    /// For a `#[cfg(..)]`: whether its predicate holds in a test build
    /// alone.
    test_only: bool,
}

/// Whether a `cfg` predicate holds in a test build alone: `test` itself, an
/// `all(..)` one of whose members does, or an `any(..)` whose every member
/// does. A `not(..)`, a feature or a target holds in some build without
/// tests, where what stands under it is product code.
fn only_in_tests(predicate: &syn::Meta) -> bool {
    match predicate {
        syn::Meta::Path(p) => p.is_ident("test"),
        syn::Meta::List(list) => {
            let members: Vec<syn::Meta> = list
                .parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
                .map(|members| members.into_iter().collect())
                .unwrap_or_default();
            if list.path.is_ident("all") {
                members.iter().any(only_in_tests)
            } else {
                list.path.is_ident("any")
                    && !members.is_empty()
                    && members.iter().all(only_in_tests)
            }
        }
        syn::Meta::NameValue(_) => false,
    }
}

/// The attributes of an item, converted once from `syn`.
fn attrs(list: &[syn::Attribute]) -> Vec<Attr> {
    list.iter()
        .map(|a| {
            let ident = a.path().get_ident().map(ToString::to_string);
            let doc = match (&a.meta, ident.as_deref()) {
                (syn::Meta::NameValue(nv), Some("doc")) => match &nv.value {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) => Some(s.value().trim().to_string()),
                    _ => None,
                },
                _ => None,
            };
            let derives = if ident.as_deref() == Some("derive") {
                a.parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
                    .map(|paths| {
                        paths
                            .iter()
                            .filter_map(|p| p.segments.last().map(|s| s.ident.to_string()))
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            let test_only = ident.as_deref() == Some("cfg")
                && a.parse_args::<syn::Meta>()
                    .is_ok_and(|predicate| only_in_tests(&predicate));
            Attr {
                last: a
                    .path()
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default(),
                ident,
                tokens: a.to_token_stream().to_string(),
                doc,
                derives,
                test_only,
            }
        })
        .collect()
}

/// An attribute whose path is the identifier `name`.
fn named(attr: &Attr, name: &str) -> bool {
    match &attr.ident {
        Some(ident) => ident == name,
        None => false,
    }
}

/// Whether an item stands under a cfg that holds in a test build alone:
/// several `#[cfg]` attributes hold together, so one such is enough.
fn is_cfg_test(attrs: &[Attr]) -> bool {
    attrs.iter().any(|a| a.test_only)
}

fn doc_of(attrs: &[Attr]) -> String {
    let lines: Vec<String> = attrs.iter().filter_map(|a| a.doc.clone()).collect();
    lines.join("\n")
}

/// The first paragraph of a doc comment, on one line, without Markdown links.
pub(crate) fn summary(doc: &str) -> String {
    let para: Vec<&str> = doc
        .lines()
        .take_while(|l| !l.trim().is_empty())
        .map(str::trim)
        .collect();
    para.join(" ")
        .replace(['`', '[', ']'], "")
        .trim()
        .to_string()
}

fn doctests(doc: &str) -> usize {
    let mut count = 0;
    let mut open = false;
    for line in doc.lines() {
        let Some(info) = line.trim().strip_prefix("```") else {
            continue;
        };
        if open {
            open = false;
            continue;
        }
        open = true;
        let info = info.trim();
        if info.is_empty()
            || info
                .split(',')
                .all(|t| matches!(t.trim(), "rust" | "no_run" | "should_panic"))
        {
            count += 1;
        }
    }
    count
}

fn derives(attrs: &[Attr]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for a in attrs {
        for d in &a.derives {
            out.push(d.clone());
        }
    }
    out
}

/// Collapse the token spacing of printed Rust to the way it is written.
pub(crate) fn tidy(text: &str) -> String {
    let mut out = text.to_string();
    for (from, to) in [
        (" :: ", "::"),
        (":: ", "::"),
        (" ::", "::"),
        (" ,", ","),
        ("( ", "("),
        (" )", ")"),
        ("& ", "&"),
        (" <", "<"),
        ("< ", "<"),
        (" >", ">"),
        ("[ ", "["),
        (" ]", "]"),
        (" ;", ";"),
        (" :", ":"),
        ("- >", "->"),
        ("& 'static", "&'static"),
        ("' ", "'"),
        (" (", "("),
        ("->", " -> "),
        ("  ", " "),
    ] {
        out = out.replace(from, to);
    }
    out.trim().to_string()
}

#[allow(clippy::too_many_arguments)]
fn function(
    sig: &syn::Signature,
    block: Option<&syn::Block>,
    attrs: &[Attr],
    public: bool,
    owner: Option<String>,
    rel: &str,
    in_test: bool,
    lib: bool,
) -> (FnFact, FnSyntax) {
    let doc = doc_of(attrs);
    let owned = block.map(crate::convert::block);
    let body = match &owned {
        Some(b) => b.scan(),
        None => crate::ast::Scan::none(),
    };
    let test = attrs.iter().any(|a| a.last == "test");
    let fact = FnFact {
        name: sig.ident.to_string(),
        owner,
        public,
        test,
        in_test_module: in_test,
        span: Span {
            file: rel.to_string(),
            line: sig.ident.span().start().line,
        },
        doctests: if lib && public { doctests(&doc) } else { 0 },
        doc,
        signature: tidy(&sig.to_token_stream().to_string()),
        tokens: match block {
            Some(block) => format!("{} {}", sig.to_token_stream(), block.to_token_stream()),
            None => sig.to_token_stream().to_string(),
        },
        calls: body.calls,
        literals: body.literals,
        transitions: body.transitions,
        effect: block.and_then(effect_of),
        in_trait: false,
        implements: None,
        trait_ref: None,
        apart: false,
        receiver: sig
            .receiver()
            .map(|r| tidy(&r.to_token_stream().to_string())),
        trait_path: None,
        impl_params: Vec::new(),
        scope: 0,
    };
    let syntax = FnSyntax {
        signature: crate::convert::signature(sig),
        block: owned,
        asserts: body.asserts,
    };
    (fact, syntax)
}

/// The names one module scope binds for its trait methods: what its `use`
/// items name, whether one of them is a glob, and the traits it declares.
fn scope_fact(list: &[syn::Item]) -> ScopeFact {
    let mut fact = ScopeFact::default();
    for item in list {
        match item {
            syn::Item::Use(u) => scope_uses(&u.tree, &[], u.leading_colon.is_some(), &mut fact),
            syn::Item::Trait(t) => fact.traits.push(t.ident.unraw().to_string()),
            syn::Item::Mod(m) if m.content.is_none() => {
                fact.mods.push(m.ident.unraw().to_string());
            }
            _ => {}
        }
    }
    fact
}

/// The bindings of one `use` tree, each with the path it names; a path
/// written from the crate graph's root (`::name`) keeps its two colons.
fn scope_uses(tree: &syn::UseTree, prefix: &[String], rooted: bool, out: &mut ScopeFact) {
    let with = |ident: &syn::Ident| {
        let mut path = prefix.to_vec();
        path.push(ident.to_string());
        path
    };
    let printed = |path: Vec<String>| match rooted {
        true => format!("::{}", path.join("::")),
        false => path.join("::"),
    };
    match tree {
        syn::UseTree::Path(p) => scope_uses(&p.tree, &with(&p.ident), rooted, out),
        syn::UseTree::Name(n) if n.ident == "self" => {
            if let Some(last) = prefix.last() {
                out.uses.push((last.clone(), printed(prefix.to_vec())));
            }
        }
        syn::UseTree::Name(n) => out
            .uses
            .push((n.ident.to_string(), printed(with(&n.ident)))),
        syn::UseTree::Rename(r) if r.ident == "self" => {
            out.uses
                .push((r.rename.to_string(), printed(prefix.to_vec())));
        }
        syn::UseTree::Rename(r) => {
            out.uses
                .push((r.rename.to_string(), printed(with(&r.ident))));
        }
        syn::UseTree::Glob(_) => out.glob = true,
        syn::UseTree::Group(g) => {
            for item in &g.items {
                scope_uses(item, prefix, rooted, out);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn items(
    list: &[syn::Item],
    rel: &str,
    _owner: Option<String>,
    in_test: bool,
    product: bool,
    lib: bool,
    file: &mut SourceFile,
    syntax: &mut FileSyntax,
) {
    let scope = file.scopes.len();
    file.scopes.push(scope_fact(list));
    for item in list {
        match item {
            syn::Item::Fn(f) => {
                let public = matches!(f.vis, syn::Visibility::Public(_));
                let (mut fact, fn_syntax) = function(
                    &f.sig,
                    Some(&f.block),
                    &attrs(&f.attrs),
                    public,
                    None,
                    rel,
                    in_test,
                    lib,
                );
                fact.scope = scope;
                file.fns.push(fact);
                syntax.fns.push(fn_syntax);
            }
            syn::Item::Impl(imp) => {
                let self_ty = tidy(&imp.self_ty.to_token_stream().to_string());
                if let Some((path, _)) = &imp.trait_ {
                    if path.segments.last().is_some_and(|s| s.ident == "Error") {
                        file.error_impls.push(self_ty.clone());
                    }
                }
                let named_trait = imp
                    .trait_
                    .as_ref()
                    .and_then(|(path, _)| path.segments.last());
                let implements = named_trait.map(|s| s.ident.to_string());
                let trait_ref = named_trait.map(|s| tidy(&s.to_token_stream().to_string()));
                let trait_path = imp.trait_.as_ref().map(|(path, _)| {
                    let written: Vec<String> =
                        path.segments.iter().map(|s| s.ident.to_string()).collect();
                    match path.leading_colon {
                        Some(_) => format!("::{}", written.join("::")),
                        None => written.join("::"),
                    }
                });
                let impl_params: Vec<String> = imp
                    .generics
                    .params
                    .iter()
                    .filter_map(|p| match p {
                        syn::GenericParam::Type(t) => Some(t.ident.to_string()),
                        syn::GenericParam::Const(c) => Some(c.ident.to_string()),
                        syn::GenericParam::Lifetime(_) => None,
                    })
                    .collect();
                for member in &imp.items {
                    if let syn::ImplItem::Fn(f) = member {
                        let public = matches!(f.vis, syn::Visibility::Public(_));
                        let (mut fact, fn_syntax) = function(
                            &f.sig,
                            Some(&f.block),
                            &attrs(&f.attrs),
                            public,
                            Some(self_ty.clone()),
                            rel,
                            in_test,
                            lib,
                        );
                        fact.implements.clone_from(&implements);
                        fact.trait_ref.clone_from(&trait_ref);
                        fact.trait_path.clone_from(&trait_path);
                        fact.impl_params.clone_from(&impl_params);
                        fact.scope = scope;
                        file.fns.push(fact);
                        syntax.fns.push(fn_syntax);
                    }
                }
            }
            syn::Item::Trait(t) => {
                let public = matches!(t.vis, syn::Visibility::Public(_));
                for member in &t.items {
                    if let syn::TraitItem::Fn(f) = member {
                        let (mut fact, fn_syntax) = function(
                            &f.sig,
                            f.default.as_ref(),
                            &attrs(&f.attrs),
                            public,
                            Some(t.ident.to_string()),
                            rel,
                            in_test,
                            lib,
                        );
                        fact.in_trait = true;
                        fact.scope = scope;
                        file.fns.push(fact);
                        syntax.fns.push(fn_syntax);
                    }
                }
            }
            syn::Item::Enum(e) => {
                syntax.enums.push(if only_lifetimes(&e.generics) {
                    e.variants
                        .iter()
                        .map(|v| {
                            let fields = v
                                .fields
                                .iter()
                                .map(|f| {
                                    (
                                        f.ident.as_ref().map(ToString::to_string),
                                        crate::convert::ty(&f.ty),
                                    )
                                })
                                .collect();
                            (v.ident.to_string(), fields)
                        })
                        .collect()
                } else {
                    Vec::new()
                });
                file.enums.push(EnumFact {
                    name: e.ident.to_string(),
                    span: Span {
                        file: rel.to_string(),
                        line: e.ident.span().start().line,
                    },
                    scope,
                    derives: derives(&attrs(&e.attrs)),
                    explicit: e.variants.iter().any(|v| v.discriminant.is_some()),
                    variants: e
                        .variants
                        .iter()
                        .map(|v| Variant {
                            name: v.ident.to_string(),
                            unit: matches!(v.fields, syn::Fields::Unit),
                            doc: doc_of(&attrs(&v.attrs)),
                            default: attrs(&v.attrs).iter().any(|a| named(a, "default")),
                        })
                        .collect(),
                });
            }
            // Every struct is registered, so that a type the lift does not
            // read is named with its shape and not as an unknown name. A
            // unit struct is a struct of no fields.
            syn::Item::Struct(st) => {
                file.structs.push(StructFact {
                    name: st.ident.to_string(),
                    span: Span {
                        file: rel.to_string(),
                        line: st.ident.span().start().line,
                    },
                    scope,
                    derives: derives(&attrs(&st.attrs)),
                });
                syntax.structs.push(match &st.fields {
                    _ if !only_lifetimes(&st.generics) => {
                        vec![crate::lift::unread_shape(crate::lift::PARAMETRIC_STRUCT)]
                    }
                    syn::Fields::Named(named) => named
                        .named
                        .iter()
                        .filter_map(|f| {
                            Some((f.ident.as_ref()?.to_string(), crate::convert::ty(&f.ty)))
                        })
                        .collect(),
                    // A tuple struct's field is named by its position, as
                    // Rust reads it (`value.0`).
                    syn::Fields::Unnamed(unnamed) => unnamed
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, f)| (i.to_string(), crate::convert::ty(&f.ty)))
                        .collect(),
                    syn::Fields::Unit => Vec::new(),
                });
            }
            syn::Item::Type(t)
                if !in_test
                    && t.generics.params.iter().all(|p| {
                        matches!(
                            p,
                            syn::GenericParam::Type(_) | syn::GenericParam::Lifetime(_)
                        )
                    }) =>
            {
                syntax.aliases.push((
                    t.ident.to_string(),
                    crate::convert::alias(&t.generics, &t.ty),
                ));
            }
            syn::Item::Const(c) if !in_test && product && c.generics.params.is_empty() => {
                let value = &*c.expr;
                let value: syn::Block = syn::parse_quote!({ #value });
                syntax.consts.push((
                    c.ident.to_string(),
                    crate::convert::ty(&c.ty),
                    crate::convert::block(&value),
                    scope,
                    c.to_token_stream().to_string(),
                    c.ident.span().start().line,
                ));
            }
            syn::Item::Use(u) => {
                for (name, path) in use_bindings(&u.tree, &[]) {
                    file.imports.insert(name, path.join("::"));
                }
            }
            syn::Item::Mod(m) => {
                let marks = attrs(&m.attrs);
                if marks.iter().any(|a| named(a, "no_implicit_prelude")) {
                    file.no_prelude = true;
                }
                if let Some((_, inner)) = &m.content {
                    let test = in_test || is_cfg_test(&marks);
                    // The scopes the module's items push are named inside
                    // it; each module around them puts its name before.
                    let own = file.scopes.len();
                    items(inner, rel, None, test, product, lib, file, syntax);
                    for inside in &mut file.scopes[own..] {
                        inside.module = match inside.module.as_str() {
                            "" => m.ident.unraw().to_string(),
                            below => format!("{}::{below}", m.ident.unraw()),
                        };
                    }
                }
            }
            _ => {}
        }
        if product && !in_test && !matches!(item, syn::Item::Mod(_)) {
            let mut scan = CapabilityScan {
                rel,
                found: Vec::new(),
            };
            scan.visit_item(item);
            file.capabilities.extend(scan.found);
        }
    }
}

/// `(Enum, Variant)` of a path `.., Enum, Variant`, by its segments.
pub(crate) fn enum_path(segments: &[String]) -> Option<(String, String)> {
    match segments {
        [] => None,
        [owner, rest @ ..] => match rest {
            [] => None,
            [variant, tail @ ..] => {
                if tail.is_empty() {
                    Some((owner.clone(), variant.clone()))
                } else {
                    enum_path(rest)
                }
            }
        },
    }
}

/// Effects a product item reaches, by the paths it names.
struct CapabilityScan<'a> {
    rel: &'a str,
    found: Vec<(Capability, Span, String)>,
}

impl CapabilityScan<'_> {
    fn note(&mut self, cap: Capability, span: proc_macro2::Span, what: String) {
        self.found.push((
            cap,
            Span {
                file: self.rel.to_string(),
                line: span.start().line,
            },
            what,
        ));
    }

    fn classify(segments: &[String]) -> Option<Capability> {
        let tail: Vec<&str> = if segments.first().is_some_and(|s| s == "std") {
            segments[1..].iter().map(String::as_str).collect()
        } else {
            segments.iter().map(String::as_str).collect()
        };
        match tail.first().copied() {
            Some("fs") | Some("File") | Some("OpenOptions") => Some(Capability::Fs),
            Some("net") | Some("TcpStream") | Some("TcpListener") | Some("UdpSocket") => {
                Some(Capability::Net)
            }
            Some("process") | Some("Command") => Some(Capability::Process),
            Some("env") => Some(Capability::Env),
            _ => None,
        }
    }
}

/// The names a `use` tree binds, each with the path it names: an item's
/// own name, or the name it is renamed to.
fn use_bindings(tree: &syn::UseTree, prefix: &[String]) -> Vec<(String, Vec<String>)> {
    let with = |ident: &syn::Ident| {
        let mut path = prefix.to_vec();
        path.push(ident.to_string());
        path
    };
    match tree {
        syn::UseTree::Path(p) => use_bindings(&p.tree, &with(&p.ident)),
        syn::UseTree::Name(n) => vec![(n.ident.to_string(), with(&n.ident))],
        syn::UseTree::Rename(r) => vec![(r.rename.to_string(), with(&r.ident))],
        syn::UseTree::Glob(_) => prefix
            .last()
            .map(|last| (last.clone(), prefix.to_vec()))
            .into_iter()
            .collect(),
        syn::UseTree::Group(g) => g
            .items
            .iter()
            .flat_map(|item| use_bindings(item, prefix))
            .collect(),
    }
}

fn use_paths(tree: &syn::UseTree, prefix: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
    match tree {
        syn::UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            use_paths(&p.tree, prefix, out);
            prefix.pop();
        }
        syn::UseTree::Name(n) => {
            let mut path = prefix.clone();
            path.push(n.ident.to_string());
            out.push(path);
        }
        syn::UseTree::Rename(r) => {
            let mut path = prefix.clone();
            path.push(r.ident.to_string());
            out.push(path);
        }
        syn::UseTree::Glob(_) => out.push(prefix.clone()),
        syn::UseTree::Group(g) => {
            for item in &g.items {
                use_paths(item, prefix, out);
            }
        }
    }
}

impl<'ast> Visit<'ast> for CapabilityScan<'_> {
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        let mut out = Vec::new();
        use_paths(&node.tree, &mut Vec::new(), &mut out);
        for path in out {
            if path.first().is_some_and(|s| s == "std") {
                if let Some(cap) = Self::classify(&path) {
                    self.note(cap, node.use_token.span, path.join("::"));
                }
            }
        }
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        let segments: Vec<String> = node.segments.iter().map(|s| s.ident.to_string()).collect();
        if segments.len() >= 2 {
            if let Some(cap) = Self::classify(&segments) {
                let span = node.segments[0].ident.span();
                self.note(cap, span, segments.join("::"));
            }
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        self.note(
            Capability::Unsafe,
            node.unsafe_token.span,
            "unsafe block".to_string(),
        );
        syn::visit::visit_expr_unsafe(self, node);
    }

    fn visit_signature(&mut self, node: &'ast syn::Signature) {
        if let syn::Safety::Unsafe(token) = node.safety {
            self.note(
                Capability::Unsafe,
                token.span,
                format!("unsafe fn {}", node.ident),
            );
        }
        syn::visit::visit_signature(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if let Some(token) = node.unsafety {
            self.note(Capability::Unsafe, token.span, "unsafe impl".to_string());
        }
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_foreign_mod(&mut self, node: &'ast syn::ItemForeignMod) {
        self.note(
            Capability::Ffi,
            node.abi.extern_token.span,
            "extern block".to_string(),
        );
        syn::visit::visit_item_foreign_mod(self, node);
    }
}

/// The first operating-system effect a function body performs directly: a
/// call through `std::fs`, `std::process`, `std::env` or `std::net`, or a
/// file-system query on a path (`exists`, `is_dir`, `read_dir`, ...). The
/// capability is the one the security facet already records for the crate.
fn effect_of(block: &syn::Block) -> Option<(Capability, String)> {
    struct Scan(Option<(Capability, String)>);
    impl<'ast> Visit<'ast> for Scan {
        fn visit_path(&mut self, node: &'ast syn::Path) {
            let segments: Vec<String> = node.segments.iter().map(|s| s.ident.to_string()).collect();
            // A bare `Command::X` is the workspace's own type unless it is
            // `Command::new`, the constructor of `std::process::Command`.
            let own_command = segments.first().is_some_and(|s| s == "Command")
                && segments.get(1).is_some_and(|s| s != "new");
            if self.0.is_none() && segments.len() >= 2 && !own_command {
                if let Some(cap) = CapabilityScan::classify(&segments) {
                    self.0 = Some((cap, segments.join("::")));
                }
            }
            syn::visit::visit_path(self, node);
        }

        fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
            const FS_QUERIES: [&str; 8] = [
                "exists",
                "try_exists",
                "is_file",
                "is_dir",
                "read_dir",
                "canonicalize",
                "metadata",
                "symlink_metadata",
            ];
            let name = node.method.to_string();
            if self.0.is_none() && node.args.is_empty() && FS_QUERIES.contains(&name.as_str()) {
                self.0 = Some((Capability::Fs, format!("Path::{name}")));
            }
            syn::visit::visit_expr_method_call(self, node);
        }
    }
    let mut scan = Scan(None);
    scan.visit_block(block);
    scan.0
}

/// The facts and the syntax of one library source text, as [`source`]
/// reads a file.
#[cfg(test)]
pub(crate) fn read_text(text: &str) -> (SourceFile, FileSyntax) {
    let parsed = syn::parse_file(text).expect("the source parses");
    let mut file = SourceFile::default();
    let mut syntax = FileSyntax::default();
    items(
        &parsed.items,
        "lib.rs",
        None,
        false,
        true,
        true,
        &mut file,
        &mut syntax,
    );
    (file, syntax)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Types outside the lift.

    #[test]
    fn every_struct_is_registered_with_the_fields_its_shape_has() {
        let (file, syntax) = read_text(
            "struct Named { a: u8 } struct Meters(u32, bool); struct Marker; \
             struct Wrapper<T> { value: T } struct Reader<'a> { text: &'a str }",
        );
        let names: Vec<&str> = file.structs.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["Named", "Meters", "Marker", "Wrapper", "Reader"]);
        let fields = |i: usize| -> Vec<(&str, &str)> {
            syntax.structs[i]
                .iter()
                .map(|(n, t)| (n.as_str(), t.text.as_str()))
                .collect()
        };
        assert_eq!(fields(0), [("a", "u8")]);
        // A tuple struct's fields are named by position.
        assert_eq!(fields(1), [("0", "u32"), ("1", "bool")]);
        // A unit struct has none.
        assert!(fields(2).is_empty());
        // A struct with a type parameter is no one type: it is registered
        // under no field name, with its shape.
        assert_eq!(fields(3), [("", crate::lift::PARAMETRIC_STRUCT)]);
        // A lifetime is no such parameter.
        assert_eq!(fields(4), [("text", "&'a str")]);
    }

    // Functions refused by their signature or misread by where they stand:
    // which files and modules are test code, the names each module scope
    // binds, and the facts a trait implementation's method carries.

    fn facts(src: &str, test_file: bool) -> SourceFile {
        let parsed = syn::parse_file(src).unwrap_or_else(|e| panic!("parses: {e}"));
        let rel = "src/lib.rs".to_string();
        source(&parsed, rel, 0, String::new(), &TargetKind::Lib, test_file).0
    }

    fn method<'a>(file: &'a SourceFile, owner: &str, of: Option<&str>, name: &str) -> &'a FnFact {
        file.fns
            .iter()
            .find(|f| {
                f.owner.as_deref() == Some(owner)
                    && f.trait_ref.as_deref() == of
                    && f.name == name
                    && !f.in_trait
            })
            .unwrap_or_else(|| panic!("{owner} has {name} of {of:?}"))
    }

    #[test]
    fn a_cfg_marks_test_code_where_it_holds_in_a_test_build_alone() {
        let marked = |cfg: &str| -> bool {
            let item: syn::ItemMod =
                syn::parse_str(&format!("#[cfg({cfg})] mod m;")).expect("parses");
            is_cfg_test(&attrs(&item.attrs))
        };
        for cfg in [
            "test",
            "all(test, feature = \"x\")",
            "all(unix, any(test))",
            "any(test)",
            "any(test, all(test, unix))",
        ] {
            assert!(marked(cfg), "{cfg} holds in a test build alone");
        }
        for cfg in [
            "not(test)",
            "any(test, feature = \"x\")",
            "all(unix, not(test))",
            "feature = \"test\"",
            "target_os = \"test\"",
            "any()",
            "all()",
        ] {
            assert!(!marked(cfg), "{cfg} holds outside a test build too");
        }
        // Several attributes hold together: one that is a test build's is
        // enough.
        let both: syn::ItemMod =
            syn::parse_str("#[cfg(unix)] #[cfg(test)] mod m;").expect("parses");
        assert!(is_cfg_test(&attrs(&both.attrs)));
    }

    #[test]
    fn a_module_declared_out_of_line_under_a_test_cfg_marks_its_files() {
        let sources = [
            (
                "src/lib.rs",
                "#[cfg(test)] mod tests; mod a; mod plain; #[cfg(test)] mod deep; \
                 #[cfg(test)] #[path = \"elsewhere/odd.rs\"] mod odd; \
                 #[cfg(test)] pub mod both; #[cfg(not(test))] mod both; \
                 #[cfg(any(test, feature = \"x\"))] mod shared; \
                 mod inline { #[cfg(test)] mod under; mod over; }",
            ),
            ("src/tests.rs", "mod helpers;"),
            ("src/tests/helpers.rs", ""),
            ("src/a.rs", "#[cfg(test)] mod t; mod u;"),
            ("src/a/t.rs", ""),
            ("src/a/u.rs", ""),
            ("src/plain.rs", ""),
            ("src/deep/mod.rs", "mod inner;"),
            ("src/deep/inner.rs", ""),
            ("src/elsewhere/odd.rs", "mod beside;"),
            ("src/elsewhere/beside.rs", ""),
            ("src/both.rs", "mod part;"),
            ("src/both/part.rs", ""),
            ("src/shared.rs", ""),
            ("src/inline/under.rs", ""),
            ("src/inline/over.rs", ""),
            // A file no declaration names is no module at all.
            ("src/stray.rs", ""),
        ];
        let parsed: Vec<(PathBuf, syn::File)> = sources
            .iter()
            .map(|(path, src)| (PathBuf::from(path), syn::parse_file(src).expect("parses")))
            .collect();
        let trees: BTreeMap<&Path, &syn::File> =
            parsed.iter().map(|(p, f)| (p.as_path(), f)).collect();
        let found = test_files(&[PathBuf::from("src/lib.rs")], &trees);
        let expected: BTreeSet<PathBuf> = [
            "src/a/t.rs",
            "src/deep/inner.rs",
            "src/deep/mod.rs",
            "src/elsewhere/beside.rs",
            "src/elsewhere/odd.rs",
            "src/inline/under.rs",
            "src/tests.rs",
            "src/tests/helpers.rs",
        ]
        .iter()
        .map(PathBuf::from)
        .collect();
        assert_eq!(found, expected);
    }

    #[test]
    fn a_function_of_a_test_file_is_test_code() {
        let src = "pub fn helper() -> u32 { 1 } pub struct S; impl S { fn m(&self) {} }";
        assert!(facts(src, true).fns.iter().all(|f| f.in_test_module));
        assert!(facts(src, false).fns.iter().all(|f| !f.in_test_module));
    }

    #[test]
    fn a_trait_method_stands_apart_where_its_type_has_another_of_its_name() {
        let file = facts(
            "struct S; struct T; \
             trait A { fn m(&self) -> u32; fn n(&self) -> u32; } \
             trait B { fn m(&self) -> u32; } \
             impl S { fn n(&self) -> u32 { 0 } } \
             impl A for S { fn m(&self) -> u32 { 1 } fn n(&self) -> u32 { 2 } } \
             impl B for S { fn m(&self) -> u32 { 3 } } \
             impl A for T { fn m(&self) -> u32 { 4 } fn n(&self) -> u32 { 5 } } \
             impl From<u8> for T { fn from(_n: u8) -> T { T } } \
             impl From<u16> for T { fn from(_n: u16) -> T { T } } \
             impl From<u8> for S { fn from(_n: u8) -> S { S } }",
            false,
        );
        // Two traits give `S` an `m`, and an inherent `n` stands beside a
        // trait's; one generic trait gives `T` a `from` twice.
        assert!(method(&file, "S", Some("A"), "m").apart);
        assert!(method(&file, "S", Some("B"), "m").apart);
        assert!(method(&file, "S", Some("A"), "n").apart);
        assert!(method(&file, "T", Some("From<u8>"), "from").apart);
        assert!(method(&file, "T", Some("From<u16>"), "from").apart);
        // One trait for two types, and a type's only method of a name.
        assert!(!method(&file, "T", Some("A"), "m").apart);
        assert!(!method(&file, "T", Some("A"), "n").apart);
        assert!(!method(&file, "S", Some("From<u8>"), "from").apart);
        // An inherent method is `Type::name` whatever stands beside it.
        assert!(!method(&file, "S", None, "n").apart);
    }

    #[test]
    fn each_module_scope_keeps_the_names_it_binds() {
        let file = facts(
            "#![no_implicit_prelude] \
             use std::fmt; use crate::shapes::{self, Shape, Named as _}; use ::serde::Serialize; \
             mod child; \
             trait Local { fn m(&mut self); } \
             struct S; \
             impl Local for S { fn m(&mut self) {} } \
             pub mod inner { \
                 use super::*; \
                 pub struct T; \
                 impl crate::shapes::Shape for T { fn area(&self) -> u32 { 0 } } \
                 impl<X: Copy, const N: usize> From<[X; N]> for T { fn from(_x: [X; N]) -> T { T } } \
             }",
            false,
        );
        assert!(file.no_prelude);
        assert_eq!(file.scopes.len(), 2);
        let uses: Vec<(&str, &str)> = file.scopes[0]
            .uses
            .iter()
            .map(|(n, p)| (n.as_str(), p.as_str()))
            .collect();
        assert_eq!(
            uses,
            [
                ("fmt", "std::fmt"),
                ("shapes", "crate::shapes"),
                ("Shape", "crate::shapes::Shape"),
                ("_", "crate::shapes::Named"),
                ("Serialize", "::serde::Serialize"),
            ]
        );
        assert!(!file.scopes[0].glob);
        assert_eq!(file.scopes[0].traits, ["Local"]);
        assert_eq!(file.scopes[0].mods, ["child"]);
        // The inline module binds nothing by name, and its glob may bind
        // anything.
        assert!(file.scopes[1].glob && file.scopes[1].uses.is_empty());
        let m = method(&file, "S", Some("Local"), "m");
        assert_eq!((m.scope, m.trait_path.as_deref()), (0, Some("Local")));
        assert_eq!(m.receiver.as_deref(), Some("&mut self"));
        assert!(m.impl_params.is_empty());
        let area = method(&file, "T", Some("Shape"), "area");
        assert_eq!(
            (area.scope, area.trait_path.as_deref()),
            (1, Some("crate::shapes::Shape"))
        );
        let from = file.fns.iter().find(|f| f.name == "from").expect("from");
        assert_eq!(from.impl_params, ["X", "N"]);
        assert_eq!(from.receiver, None);
    }

    #[test]
    fn a_type_the_lift_does_not_read_says_its_shape() {
        let file = facts(
            "struct A(u32); struct B; struct C<T> { t: T } enum D<T> { X(T) } \
             union E { a: u32 } struct F { a: u32 } enum G { X } struct H<'a> { a: &'a str }",
            false,
        );
        assert_eq!(
            file.unread,
            [
                ("C".to_string(), crate::lift::PARAMETRIC_STRUCT),
                ("D".to_string(), "an enum with type parameters"),
                ("E".to_string(), "a union"),
            ]
        );
    }

    /// A path outside the root is written with `..` steps, never as the
    /// absolute path, and [`under`] reads it back to the same file.
    #[test]
    fn a_path_outside_the_root_is_relative_by_dot_dot_steps() {
        assert_eq!(rel("/w/root", "/w/root"), "");
        assert_eq!(rel("/w/root", "/w/root/src/lib.rs"), "src/lib.rs");
        assert_eq!(
            rel("/w/root", "/w/member/src/lib.rs"),
            "../member/src/lib.rs"
        );
        assert_eq!(rel("/w/root", "/w/rootless/a.rs"), "../rootless/a.rs");
        assert_eq!(rel("/w/a/b", "/x/y.rs"), "../../../x/y.rs");
        assert_eq!(
            under(Path::new("/w/root"), "../member/src/lib.rs"),
            Path::new("/w/member/src/lib.rs")
        );
        assert_eq!(
            under(Path::new("/w/root"), "src/lib.rs"),
            Path::new("/w/root/src/lib.rs")
        );
    }
}
