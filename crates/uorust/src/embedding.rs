//! The join of a projection's function rows to the bodies `uorust embed
//! --workspace` recorded (`--embedding DIR`).
//!
//! Each record ([`crate::shard::RECORD`]) of one module project in one
//! configuration lists its bodies by the path the dump prints
//! ([`crate::embed::BodyFact`]): `inner::run`, `a::<impl at src/a.rs:3:1:
//! 3:9>::f`. A path the dump prints names the item the compiler resolved:
//! its module path, inline modules included, the impl by the place of its
//! header, and the item's own name. A function of the surface reader is
//! keyed the same way from its own facts (its file's module, the inline
//! modules around it, the line and column its impl's header starts at in
//! its package), so a row is joined to the body of the one item Rust means
//! by it, never by the surface path, which names a trait's method in two
//! impls of one type, or an inline module's function beside its file
//! module's of one name, alike. A function nested in a function prints the
//! path a module's function of the outer one's name prints (`outer::helper`):
//! a body whose owner path is a body's path is not told apart, and its row
//! stays as it was.
//!
//! A row that did not lift is embedded-closed ([`crate::register::Status::
//! Embedded`]) when, in every configuration the workspace declares
//! ([`crate::project::Embedding::configurations`]), the one body of its key
//! is embedded with its reach closed, or the configuration's records cover
//! every module of the crate and none holds the key (its dump does not
//! compile the function), and at least one configuration holds it. A
//! declared configuration with no record of the crate, or whose records do
//! not cover every module and lack the key, leaves the row as it was:
//! whether that dump compiles the function is not read. Two bodies of one
//! key in one configuration, or two functions of one surface path, leave
//! the row as it was too. None of this is translation.
//!
//! Each record states the digest of the workspace's sources it was embedded
//! from ([`crate::shard::source_digest`]); a record whose digest is not the
//! projected workspace's is of another tree and is refused, so a changed
//! body is never joined to the record of the body it replaced.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::register::EmbeddedAt;
use crate::shard::{Configuration, RECORD};
use crate::UorustError;

/// The item a body is of, as the dump prints its path: the crate, the
/// module path (a trait's name too, for a trait's provided method), the
/// impl by its package-relative file and the line and column its header
/// starts at, and the item's name. Two impls on one line are two impls, so
/// the column is part of the key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Key {
    pub(crate) krate: String,
    pub(crate) module: Vec<String>,
    pub(crate) impl_at: Option<(String, usize, usize)>,
    pub(crate) name: String,
}

/// One body of a record: its key, the path the dump prints, the digest of
/// its term, and whether it is evaluable with its reach closed.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Body {
    key: Key,
    path: String,
    embedded: String,
    closed: bool,
}

/// One record: its configuration, its project's digest, the modules it
/// embedded (empty: every module), every module of the crate, its bodies.
#[derive(Clone, Debug)]
pub(crate) struct Recorded {
    krate: String,
    configuration: String,
    project: String,
    modules: Vec<String>,
    crate_modules: Vec<String>,
    bodies: Vec<Body>,
    /// How its bodies hold each type, by the path the dump prints it by.
    holdings: BTreeMap<String, crate::ownership::Held>,
}

/// The segments of a path the dump prints, split at each `::` outside an
/// angle bracket (`<impl at src/a.rs:1:1: 1:9>` holds none).
fn segments(path: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    let bytes = path.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'<' => depth += 1,
            b'>' => depth = depth.saturating_sub(1),
            b':' if depth == 0 && bytes.get(i + 1) == Some(&b':') => {
                out.push(path[start..i].to_string());
                i += 2;
                start = i;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(path[start..].to_string());
    out
}

/// The key of a body of `krate` whose dump prints `path`; `None` for a
/// path of a form no function of the surface has (a closure, a promoted
/// or anonymous constant, a path past an impl's item).
pub(crate) fn body_key(krate: &str, path: &str) -> Option<Key> {
    if !crate::embed::has_row(path) {
        return None;
    }
    let mut parts = segments(path);
    let name = parts.pop()?;
    let mut module = Vec::new();
    let mut impl_at = None;
    for p in parts {
        match p
            .strip_prefix("<impl at ")
            .and_then(|r| r.strip_suffix('>'))
        {
            Some(span) => {
                if impl_at.is_some() {
                    return None;
                }
                // `src/a.rs:3:1: 3:9`: the file, the first line and its
                // column.
                let (file, rest) = span.split_once(':')?;
                let mut at = rest.split(':');
                let line = at.next()?.parse().ok()?;
                let column = at.next()?.trim().parse().ok()?;
                impl_at = Some((file.to_string(), line, column));
            }
            None if impl_at.is_some() || p.starts_with('<') || p.contains('{') => return None,
            None => module.push(p),
        }
    }
    Some(Key {
        krate: krate.to_string(),
        module,
        impl_at,
        name,
    })
}

/// The label of the configuration a record states: its features, whether
/// the default ones are on, and its target.
fn label(doc: &toml::Table) -> String {
    let features: Vec<String> = doc
        .get("features")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|f| f.as_str().map(str::to_string))
        .collect();
    Configuration {
        features,
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
            .filter(|t| !t.is_empty())
            .map(str::to_string),
    }
    .label()
}

/// Every record under `dir`, at any depth; a record whose source digest
/// is not `source`, the projected workspace's, is refused.
pub(crate) fn read(dir: &Path, source: &str) -> Result<Vec<Recorded>, UorustError> {
    let mut files = Vec::new();
    let mut next = vec![dir.to_path_buf()];
    while let Some(d) = next.pop() {
        for entry in std::fs::read_dir(&d).map_err(|e| crate::error::io(&d, e))? {
            let path = entry.map_err(|e| crate::error::io(&d, e))?.path();
            if path.is_dir() {
                next.push(path);
            } else if path.file_name().and_then(|n| n.to_str()) == Some(RECORD) {
                files.push(path);
            }
        }
    }
    files.sort();
    let mut out = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).map_err(|e| crate::error::io(&file, e))?;
        let bad = |reason: String| UorustError::Malformed {
            path: file.clone(),
            reason,
        };
        let doc: toml::Table = text.parse().map_err(|e| bad(format!("{e}")))?;
        let text_of = |key: &str| {
            doc.get(key)
                .and_then(toml::Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| bad(format!("a record without {key}")))
        };
        let list = |key: &str| -> Vec<String> {
            doc.get(key)
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        };
        let krate = text_of("crate")?;
        let stated = text_of("source")?;
        if stated != source {
            return Err(bad(format!(
                "a record of another source tree (source {stated}; the workspace projected is {source}): embed the workspace again"
            )));
        }
        let mut bodies = Vec::new();
        for b in doc
            .get("body")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
        {
            let field = |key: &str| {
                b.get(key)
                    .and_then(toml::Value::as_str)
                    .map(str::to_string)
                    .ok_or_else(|| bad(format!("a body without {key}: {b}")))
            };
            let blocked = b
                .get("blocked")
                .and_then(toml::Value::as_array)
                .is_some_and(|l| !l.is_empty());
            let path = field("path")?;
            let Some(key) = body_key(&krate, &path) else {
                continue;
            };
            bodies.push(Body {
                key,
                path,
                embedded: field("embedded")?,
                closed: field("reach")? == "closed" && !blocked,
            });
        }
        out.push(Recorded {
            configuration: label(&doc),
            project: text_of("project")?,
            modules: list("modules"),
            crate_modules: list("crate_modules"),
            holdings: crate::ownership::parse(&doc),
            krate,
            bodies,
        });
    }
    Ok(out)
}

/// The rows of `keys` (each function's path and the key of the item it
/// is) that `records` show embedded-closed in every one of the declared
/// `configurations`, with where: one [`EmbeddedAt`] per configuration that
/// holds the body.
pub(crate) fn closed(
    records: &[Recorded],
    configurations: &[String],
    keys: &BTreeMap<String, Key>,
) -> BTreeMap<String, Vec<EmbeddedAt>> {
    // By crate and declared configuration: the records.
    let mut by: BTreeMap<(&str, &str), Vec<&Recorded>> = BTreeMap::new();
    for r in records {
        if configurations.contains(&r.configuration) {
            by.entry((&r.krate, &r.configuration)).or_default().push(r);
        }
    }
    // By crate and configuration: the paths of the bodies, so that a body
    // whose owner is itself a body (a function nested in a function prints
    // `outer::helper`, as a module `outer`'s function does) is told apart.
    let owners: BTreeMap<(&str, &str), BTreeSet<&str>> = by
        .iter()
        .map(|(at, records)| {
            let paths = records
                .iter()
                .flat_map(|r| r.bodies.iter().map(|b| b.path.as_str()))
                .collect();
            (*at, paths)
        })
        .collect();
    let mut out = BTreeMap::new();
    'rows: for (path, key) in keys {
        let mut at = Vec::new();
        for configuration in configurations {
            let Some((krate, records)) = by
                .get_key_value(&(key.krate.as_str(), configuration.as_str()))
                .map(|((k, _), r)| (*k, r))
            else {
                // A declared configuration with no record of the crate.
                continue 'rows;
            };
            let found: Vec<(&Recorded, &Body)> = records
                .iter()
                .flat_map(|r| r.bodies.iter().map(move |b| (*r, b)))
                .filter(|(_, b)| &b.key == key)
                .collect();
            // A body of the key whose owner path is a function's: the dump
            // does not say whether it is that function's nested item or a
            // module's of the owner's name, so the row stays as it was.
            let nested = found.iter().any(|(_, b)| {
                let mut parts = segments(&b.path);
                parts.pop();
                owners
                    .get(&(krate, configuration.as_str()))
                    .is_some_and(|o| o.contains(parts.join("::").as_str()))
            });
            if nested {
                continue 'rows;
            }
            match found.as_slice() {
                [(r, b)] if b.closed => at.push(EmbeddedAt {
                    configuration: configuration.clone(),
                    project: r.project.clone(),
                    body: b.embedded.clone(),
                }),
                [] if covers(records) => {}
                _ => continue 'rows,
            }
        }
        if !at.is_empty() {
            // In label order, whatever order the configurations were
            // declared in, so one declaration writes one register.
            at.sort_by(|a, b| a.configuration.cmp(&b.configuration));
            out.insert(path.clone(), at);
        }
    }
    out
}

/// A type of the surface as the dump prints it: its crate, its file's
/// module path and its name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TypeKey {
    pub(crate) krate: String,
    pub(crate) module: Vec<String>,
    pub(crate) name: String,
}

/// How `records` hold each type of `types` (each type's path and its key),
/// in every configuration and module they hold: the most it is held
/// anywhere. A record names a type of its crate by its
/// crate-relative path, or, in a signature, by its name alone, which
/// names the one type of that name in the crate or none. A type no
/// record holds is not stated.
pub(crate) fn holdings(
    records: &[Recorded],
    types: &BTreeMap<String, TypeKey>,
) -> BTreeMap<String, crate::ownership::Held> {
    let mut out: BTreeMap<String, crate::ownership::Held> = BTreeMap::new();
    for r in records {
        for (printed, held) in &r.holdings {
            let segments: Vec<&str> = printed.split("::").collect();
            let Some((name, module)) = segments.split_last() else {
                continue;
            };
            let found: Vec<&String> = types
                .iter()
                .filter(|(_, t)| t.krate == r.krate && t.name == *name)
                .filter(|(_, t)| segments.len() == 1 || t.module == module)
                .map(|(path, _)| path)
                .collect();
            if let [path] = found.as_slice() {
                match out.get_mut(*path) {
                    Some(h) => h.merge(held),
                    None => {
                        out.insert((*path).clone(), held.clone());
                    }
                }
            }
        }
    }
    out
}

/// Whether one configuration's records embed every module of the crate,
/// so that a body none of them holds is one its dump does not compile.
fn covers(records: &[&Recorded]) -> bool {
    let every: BTreeSet<&String> = records.iter().flat_map(|r| &r.crate_modules).collect();
    if every.is_empty() {
        return false;
    }
    if records.iter().any(|r| r.modules.is_empty()) {
        return true;
    }
    let held: BTreeSet<&String> = records.iter().flat_map(|r| &r.modules).collect();
    every.is_subset(&held)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lift_a_body_is_keyed_by_the_item_its_dump_path_names() {
        let k = |p: &str| body_key("m", p);
        assert_eq!(
            k("a::<impl at src/a.rs:3:1: 3:9>::f"),
            Some(Key {
                krate: "m".into(),
                module: vec!["a".into()],
                impl_at: Some(("src/a.rs".into(), 3, 1)),
                name: "f".into(),
            })
        );
        assert_eq!(
            k("inner::run").map(|k| (k.module, k.impl_at, k.name)),
            Some((vec!["inner".to_string()], None, "run".to_string()))
        );
        assert_eq!(k("f::{closure#0}"), None);
        assert_eq!(k("<impl at src/a.rs:3:1: 3:9>::f::g"), None);
    }

    /// A one-crate workspace whose three functions the surface leaves
    /// residue: two generic impls' methods of one name, and a generic
    /// function of an inline module.
    const LIFT_SOURCE: &str = "pub struct Wrap<T>(pub T);\n\
        impl<T: Copy> Wrap<T> { pub fn get(&self) -> T { self.0 } }\n\
        pub struct Pair<T>(pub T, pub T);\n\
        impl<T: Copy> Pair<T> { pub fn get(&self) -> T { self.1 } }\n\
        pub mod inner {\n    pub fn first<T: Copy>(x: T, _y: T) -> T { x }\n}\n";

    /// A record of the crate `probe` in `configuration`, embedding
    /// `modules` (every one when empty), with these bodies (path, reach).
    fn lift_write_record(
        dir: &Path,
        configuration: &str,
        modules: &[&str],
        bodies: &[(&str, &str)],
    ) {
        let quote = |s: &str| toml::Value::String(s.to_string()).to_string();
        let mut text = format!(
            "crate = \"probe\"\nmodules = [{}]\ncrate_modules = [\"crate\", \"inner\"]\n{configuration}\
             target = \"\"\nproject = \"{}\"\nsource = \"{LIFT_SOURCE_DIGEST}\"\n",
            modules.iter().map(|m| quote(m)).collect::<Vec<_>>().join(", "),
            "c".repeat(64)
        );
        for (path, reach) in bodies {
            text.push_str(&format!(
                "\n[[body]]\npath = {}\nkappa = \"k\"\nembedded = \"{}\"\nblocked = []\nreach = {}\nunlinked = []\n",
                quote(path),
                "e".repeat(64),
                quote(reach)
            ));
        }
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(RECORD), text).unwrap();
    }

    /// What a test record states as its source digest until
    /// [`lift_options`] writes the workspace's in its place.
    const LIFT_SOURCE_DIGEST: &str = "the workspace's";

    /// The two configurations the tests' workspace declares.
    const LIFT_DECLARED: [&str; 2] = ["default features", "all features"];

    /// The options that project the workspace `ws` with the records under
    /// `records`, declaring `declared`; each record that states the
    /// placeholder digest is given the workspace's sources' digest.
    fn lift_options(ws: &Path, records: &Path, declared: &[&str]) -> crate::Options {
        let digest = crate::shard::source_digest(ws).unwrap();
        let mut next = vec![records.to_path_buf()];
        while let Some(d) = next.pop() {
            for entry in std::fs::read_dir(&d).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    next.push(path);
                } else {
                    let text = std::fs::read_to_string(&path).unwrap();
                    std::fs::write(&path, text.replace(LIFT_SOURCE_DIGEST, &digest)).unwrap();
                }
            }
        }
        crate::Options {
            name: "probe".into(),
            last_modified: "2026-11-15T00:00:00Z".into(),
            advisories: None,
            mir: None,
            embedding: Some(crate::Embedding {
                records: records.to_path_buf(),
                configurations: declared.iter().map(|c| c.to_string()).collect(),
            }),
        }
    }

    /// The register of the workspace projected with the records under
    /// `records`.
    fn lift_projected(work: &Path, records: &Path) -> crate::register::Register {
        lift_projected_from(work, records, LIFT_SOURCE)
    }

    /// The register of the workspace whose library is `source`, projected
    /// with the records under `records`.
    fn lift_projected_from(work: &Path, records: &Path, source: &str) -> crate::register::Register {
        lift_projected_declaring(work, records, source, &LIFT_DECLARED).expect("a projection")
    }

    /// The projection's register, the workspace declaring `declared`.
    fn lift_projected_declaring(
        work: &Path,
        records: &Path,
        source: &str,
        declared: &[&str],
    ) -> Result<crate::register::Register, UorustError> {
        let ws = work.join("ws");
        std::fs::create_dir_all(ws.join("src")).unwrap();
        std::fs::write(
            ws.join("Cargo.toml"),
            "[package]\nname = \"probe\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nx = []\n\n[workspace]\n",
        )
        .unwrap();
        std::fs::write(ws.join("src/lib.rs"), source).unwrap();
        let options = lift_options(&ws, records, declared);
        let files = crate::project_workspace(&ws, &options)?;
        Ok(crate::lift_register(&files).expect("a register"))
    }

    #[test]
    fn lift_a_row_is_embedded_closed_by_the_body_of_the_item_it_is() {
        use crate::register::Status;
        let work = std::env::temp_dir().join(format!("uorust-embedding-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&work);
        let records = work.join("records");
        let default = "features = []\ndefault_features = true\nall_features = false\n";
        let all = "features = []\ndefault_features = true\nall_features = true\n";
        // Default features: the whole crate; `Pair`'s method is open.
        lift_write_record(
            &records.join("default"),
            default,
            &[],
            &[
                ("<impl at src/lib.rs:2:1: 2:22>::get", "closed"),
                (
                    "<impl at src/lib.rs:4:1: 4:22>::get",
                    "open: is generic itself (at x)",
                ),
                ("inner::first", "closed"),
            ],
        );
        // All features, one module a project: the crate's module holds
        // both methods, and the inline module's project holds no body of
        // `first` (its dump does not compile it there).
        lift_write_record(
            &records.join("all/crate"),
            all,
            &["crate"],
            &[
                ("<impl at src/lib.rs:2:1: 2:22>::get", "closed"),
                ("<impl at src/lib.rs:4:1: 4:22>::get", "closed"),
            ],
        );
        lift_write_record(&records.join("all/inner"), all, &["inner"], &[]);
        let register = lift_projected(&work, &records);
        let status = |register: &crate::register::Register, f: &str| {
            register
                .rows
                .values()
                .find(|r| r.function == f)
                .map(|r| r.status.clone())
                .expect(f)
        };
        // Each impl's method is joined to its own impl's body: `Wrap`'s is
        // closed in both configurations, `Pair`'s is open in one.
        match status(&register, "probe::Wrap<T>::get") {
            Status::Embedded(at) => {
                let configurations: Vec<&str> =
                    at.iter().map(|a| a.configuration.as_str()).collect();
                assert_eq!(configurations, ["all features", "default features"]);
                assert!(at
                    .iter()
                    .all(|a| a.project == "c".repeat(64) && a.body == "e".repeat(64)));
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            status(&register, "probe::Pair<T>::get"),
            Status::Residue(..)
        ));
        // The inline module's function by its module: closed where its
        // dump compiles it, and absent where every module is embedded.
        assert!(matches!(
            status(&register, "probe::first"),
            Status::Embedded(at) if at.len() == 1
        ));
        assert_eq!(register.totals().embedded, 2);
        // Where a configuration's records do not cover its module, whether
        // that dump compiles it is not read: it stays residue.
        std::fs::remove_dir_all(records.join("all/inner")).unwrap();
        let register = lift_projected(&work, &records);
        assert!(matches!(
            status(&register, "probe::first"),
            Status::Residue(..)
        ));
        std::fs::remove_dir_all(&work).unwrap();
    }

    /// Two impls on one line, one per configuration, and a module `outer`
    /// whose function's path a function nested in a function `outer`
    /// prints too (refute-lift-7, counterexamples 3 and 4).
    const LIFT_ONE_LINE: &str = "pub struct A<T>(pub T);\npub struct B<T>(pub T);\n\
        #[cfg(not(feature = \"x\"))] impl<T: Copy> A<T> { pub fn get(&self) -> T { self.0 } } \
        #[cfg(feature = \"x\")] impl<T: Copy> B<T> { pub fn get(&self) -> T { self.0 } }\n\
        pub mod outer {\n    pub fn helper<T: Copy>(x: T, _y: T) -> T { x }\n}\n";

    #[test]
    fn lift_two_impls_on_one_line_and_a_nested_function_are_told_apart() {
        use crate::register::Status;
        let work = std::env::temp_dir().join(format!("uorust-one-line-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&work);
        let records = work.join("records");
        let default = "features = []\ndefault_features = true\nall_features = false\n";
        let all = "features = []\ndefault_features = true\nall_features = true\n";
        // Each impl's header as the dump prints it: line 3, at the column
        // of its `impl`, counted from 1.
        let line = LIFT_ONE_LINE.lines().nth(2).unwrap();
        let header = |of: &str| {
            let column = line.find(&format!("impl<T: Copy> {of}<T>")).unwrap() + 1;
            format!("<impl at src/lib.rs:3:{column}: 3:{}>::get", column + 15)
        };
        lift_write_record(
            &records.join("default"),
            default,
            &[],
            &[(&header("A"), "closed"), ("outer::helper", "closed")],
        );
        // With `x`, `outer` is a function and `outer::helper` the function
        // nested in it.
        lift_write_record(
            &records.join("all"),
            all,
            &[],
            &[
                (&header("B"), "closed"),
                ("outer", "closed"),
                ("outer::helper", "closed"),
            ],
        );
        let register = lift_projected_from(&work, &records, LIFT_ONE_LINE);
        let status = |f: &str| {
            register
                .rows
                .values()
                .find(|r| r.function == f)
                .map(|r| r.status.clone())
                .expect(f)
        };
        let configurations = |f: &str| match status(f) {
            Status::Embedded(at) => at.into_iter().map(|a| a.configuration).collect(),
            other => vec![format!("{other:?}")],
        };
        // Each impl's method by its own impl: closed where its dump compiles
        // it, absent where it does not, never the other's body.
        assert_eq!(configurations("probe::A<T>::get"), ["default features"]);
        assert_eq!(configurations("probe::B<T>::get"), ["all features"]);
        // The module's function is not joined to the nested function.
        assert!(
            matches!(status("probe::helper"), Status::Residue(..)),
            "{:?}",
            status("probe::helper")
        );
        std::fs::remove_dir_all(&work).unwrap();
    }

    #[test]
    fn lift_a_declared_configuration_unread_or_a_record_of_another_tree_stamps_nothing() {
        use crate::register::Status;
        let work = std::env::temp_dir().join(format!("uorust-declared-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&work);
        let records = work.join("records");
        let default = "features = []\ndefault_features = true\nall_features = false\n";
        // The default configuration's records alone, closed, while the
        // workspace declares two (refute-lift-7, counterexample 5): the
        // other's dump may compile the function and leave it open.
        lift_write_record(
            &records.join("default"),
            default,
            &[],
            &[
                ("<impl at src/lib.rs:2:1: 2:22>::get", "closed"),
                ("inner::first", "closed"),
            ],
        );
        let status = |register: &crate::register::Register, f: &str| {
            register
                .rows
                .values()
                .find(|r| r.function == f)
                .map(|r| r.status.clone())
                .expect(f)
        };
        let register = lift_projected(&work, &records);
        assert!(
            matches!(
                status(&register, "probe::Wrap<T>::get"),
                Status::Residue(..)
            ),
            "{:?}",
            status(&register, "probe::Wrap<T>::get")
        );
        assert_eq!(register.totals().embedded, 0);
        // Declaring the one configuration the records hold, it is closed.
        let ws = work.join("ws");
        let one = lift_projected_declaring(&work, &records, LIFT_SOURCE, &["default features"])
            .expect("a projection");
        assert!(matches!(
            status(&one, "probe::Wrap<T>::get"),
            Status::Embedded(at) if at.len() == 1
        ));
        // A change of the workspace's sources after the embedding (the
        // refuter's counterexample 7: the records of one tree projected
        // with another): the records are refused.
        std::fs::write(
            ws.join("src/lib.rs"),
            format!("{LIFT_SOURCE}pub fn more() -> u32 {{ 1 }}\n"),
        )
        .unwrap();
        let options = crate::Options {
            embedding: Some(crate::Embedding {
                records: records.clone(),
                configurations: vec!["default features".into()],
            }),
            ..lift_options(&ws, &records, &["default features"])
        };
        match crate::project_workspace(&ws, &options) {
            Err(UorustError::Malformed { reason, .. }) => {
                assert!(
                    reason.starts_with("a record of another source tree"),
                    "{reason}"
                )
            }
            other => panic!("{:?}", other.map(|f| f.len())),
        }
        std::fs::remove_dir_all(&work).unwrap();
    }

    #[test]
    fn lift_the_frontier_states_how_each_type_is_held() {
        let work = std::env::temp_dir().join(format!("uorust-holding-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&work);
        let records = work.join("records");
        let default = "features = []\ndefault_features = true\nall_features = false\n";
        let all = "features = []\ndefault_features = true\nall_features = true\n";
        lift_write_record(&records.join("default"), default, &[], &[]);
        lift_write_record(&records.join("all"), all, &[], &[]);
        // As the embedding reads a dump of each configuration: `Wrap` is
        // owned in one and passed as `Rc<RefCell<_>>` in the other (by its
        // signature's trimmed name), `Pair` shared; a type of no module of
        // the crate is no type of it.
        let held = |dir: &str, dump: &str| {
            let mut read = BTreeMap::new();
            crate::ownership::read(dump, &mut read);
            let file = records.join(dir).join(RECORD);
            let mut text = std::fs::read_to_string(&file).unwrap();
            text.push_str(&crate::ownership::render(&read));
            std::fs::write(&file, text).unwrap();
        };
        held(
            "default",
            "fn f(_1: Wrap<u32>) -> () {\n    let _2: std::sync::Arc<Pair<u8>>;\n}\n",
        );
        held(
            "all",
            "fn g(_1: &Rc<RefCell<Wrap<u32>>>) -> () {\n    let _2: other::Wrap<u8>;\n}\n",
        );
        let ws = work.join("ws");
        lift_projected(&work, &records);
        let options = lift_options(&ws, &records, &LIFT_DECLARED);
        let files = crate::project_workspace(&ws, &options).expect("a projection");
        let frontier: toml::Table = String::from_utf8(files["frontier.toml"].clone())
            .unwrap()
            .parse()
            .unwrap();
        let rows: Vec<(String, String, Vec<String>)> = frontier["holding"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                let text = |k: &str| r[k].as_str().unwrap().to_string();
                let within = r["within"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|w| w.as_str().unwrap().to_string())
                    .collect();
                (text("type"), text("holding"), within)
            })
            .collect();
        assert_eq!(
            rows,
            [
                (
                    "probe::Pair".to_string(),
                    "shared".to_string(),
                    vec!["Arc<_>".to_string()]
                ),
                (
                    "probe::Wrap".to_string(),
                    "interior-mutable".to_string(),
                    vec!["Rc<RefCell<_>>".to_string()]
                ),
            ]
        );
        std::fs::remove_dir_all(&work).unwrap();
    }
}
