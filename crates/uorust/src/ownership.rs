//! How a workspace holds each of its types, read from the compiler's dump:
//! owned, shared (behind `Rc` or `Arc`), or interior-mutable (inside a
//! `Cell`, `RefCell`, `Mutex`, ...). A body's locals and signature print
//! every type it holds, with the wrappers around it (`let _7:
//! &std::rc::Rc<std::cell::RefCell<inner::Tally>>`), so a type a state is
//! passed in as `Rc<RefCell<T>>` reads interior-mutable with that chain.
//!
//! A type is read by the path the dump prints it by, crate-relative for the
//! crate's own (`inner::Tally`), the signature's trimmed (`Tally`); paths of
//! core, alloc and std and the primitives are not types of the workspace
//! and are not read. A type no dump holds is not stated. What a type holds
//! inside its own fields is not read here: the reading is of how the
//! program holds the type, which is what passing state through it needs.

use std::collections::{BTreeMap, BTreeSet};

/// How the program holds a type, least first: a type held more than one
/// way is held the most of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Holding {
    /// Never behind a shared pointer or inside a cell.
    Owned,
    /// Behind `Rc`, `Arc` or a `Weak` of one, and in no cell.
    Shared,
    /// Inside a cell or a lock: mutated through a shared reference.
    Interior,
}

impl Holding {
    /// The word the record and the register state it by.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Holding::Owned => "owned",
            Holding::Shared => "shared",
            Holding::Interior => "interior-mutable",
        }
    }

    /// The holding a [`Holding::word`] names.
    pub(crate) fn from_word(word: &str) -> Option<Holding> {
        [Holding::Owned, Holding::Shared, Holding::Interior]
            .into_iter()
            .find(|h| h.word() == word)
    }
}

/// The wrappers that make what they hold mutable through a shared
/// reference.
const INTERIOR: &[&str] = &[
    "Cell",
    "RefCell",
    "UnsafeCell",
    "OnceCell",
    "OnceLock",
    "Mutex",
    "RwLock",
];

/// The wrappers that share what they hold.
const SHARED: &[&str] = &["Rc", "Arc", "Weak"];

/// The crates whose paths name no type of the workspace.
const LIBRARY: &[&str] = &["std", "core", "alloc"];

/// One type's reading: its holding and each chain of sharing and cell
/// wrappers it was found in, outermost first (`Rc<RefCell<_>>`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Held {
    pub(crate) holding: Holding,
    pub(crate) within: BTreeSet<String>,
}

impl Held {
    /// The two readings of one type as one: the most it is held, every chain.
    pub(crate) fn merge(&mut self, other: &Held) {
        self.holding = self.holding.max(other.holding);
        self.within.extend(other.within.iter().cloned());
    }
}

/// Read one dump file's text into `out`, by the path each type is printed
/// by: its signature and each local's declared type.
pub(crate) fn read(text: &str, out: &mut BTreeMap<String, Held>) {
    for line in text.lines().map(str::trim) {
        let ty = if let Some(sig) = line.strip_prefix("fn ") {
            // `fn f(_1: A) -> B {`: from the parameters on.
            sig.find('(')
                .map(|at| sig[at..].trim_end_matches('{').trim_end())
        } else if line.starts_with("let ") {
            line.split_once(": ").map(|(_, t)| t.trim_end_matches(';'))
        } else {
            None
        };
        if let Some(ty) = ty {
            scan(ty, out);
        }
    }
}

/// A wrapper's name if `path` is one of `set`, as the dump prints it:
/// under core, alloc or std, or trimmed to its name.
fn wrapper<'a>(path: &'a str, set: &[&str]) -> Option<&'a str> {
    let mut segments = path.split("::");
    let first = segments.next()?;
    let last = path.rsplit("::").next()?;
    let library = first == last || LIBRARY.contains(&first);
    (library && set.contains(&last)).then_some(last)
}

/// Every type path in the type text `ty`, each read with the wrappers
/// around it.
fn scan(ty: &str, out: &mut BTreeMap<String, Held>) {
    let b = ty.as_bytes();
    // The generic heads open around the place read: a wrapper's name, or
    // empty for any other.
    let mut open: Vec<&str> = Vec::new();
    let mut braces = 0usize;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        match c {
            b'{' => braces += 1,
            b'}' => braces = braces.saturating_sub(1),
            _ if braces > 0 => {}
            b'-' if b.get(i + 1) == Some(&b'>') => i += 1,
            b'\'' => {
                // A lifetime.
                i += 1;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                continue;
            }
            b'<' => open.push(""),
            b'>' => {
                open.pop();
            }
            _ if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;
                while i < b.len() {
                    if b[i].is_ascii_alphanumeric() || b[i] == b'_' {
                        i += 1;
                    } else if b[i] == b':' && b.get(i + 1) == Some(&b':') {
                        i += 2;
                    } else {
                        break;
                    }
                }
                let path = &ty[start..i];
                note(path, &open, out);
                if b.get(i) == Some(&b'<') {
                    open.push(
                        wrapper(path, INTERIOR)
                            .or_else(|| wrapper(path, SHARED))
                            .unwrap_or(""),
                    );
                    i += 1;
                }
                continue;
            }
            _ => {}
        }
        i += 1;
    }
}

/// Read the type `path` held inside the wrappers `open`, if it can be a
/// type of the workspace.
fn note(path: &str, open: &[&str], out: &mut BTreeMap<String, Held>) {
    let first = path.split("::").next().unwrap_or_default();
    let last = path.rsplit("::").next().unwrap_or_default();
    if LIBRARY.contains(&first) || !last.starts_with(|c: char| c.is_ascii_uppercase()) {
        return;
    }
    if path.split("::").count() == 1
        && (wrapper(path, INTERIOR).is_some() || wrapper(path, SHARED).is_some())
    {
        return;
    }
    let chain: Vec<&str> = open.iter().copied().filter(|w| !w.is_empty()).collect();
    let holding = if chain.iter().any(|w| INTERIOR.contains(w)) {
        Holding::Interior
    } else if chain.iter().any(|w| SHARED.contains(w)) {
        Holding::Shared
    } else {
        Holding::Owned
    };
    let mut within = BTreeSet::new();
    if !chain.is_empty() {
        within.insert(format!(
            "{}_{}",
            chain.iter().map(|w| format!("{w}<")).collect::<String>(),
            ">".repeat(chain.len())
        ));
    }
    let held = Held { holding, within };
    match out.get_mut(path) {
        Some(h) => h.merge(&held),
        None => {
            out.insert(path.to_string(), held);
        }
    }
}

/// The `[[holding]]` rows of a record: each type by the path the dump
/// prints it by.
pub(crate) fn render(read: &BTreeMap<String, Held>) -> String {
    let quote = |s: &str| toml::Value::String(s.to_string()).to_string();
    let mut out = String::new();
    for (path, held) in read {
        out.push_str(&format!(
            "\n[[holding]]\ntype = {}\nholding = \"{}\"\nwithin = [{}]\n",
            quote(path),
            held.holding.word(),
            held.within
                .iter()
                .map(|w| quote(w))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    out
}

/// The `[[holding]]` rows of a record read back; a row of another shape
/// is not a reading and is left out.
pub(crate) fn parse(doc: &toml::Table) -> BTreeMap<String, Held> {
    let mut out = BTreeMap::new();
    for row in doc
        .get("holding")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
    {
        let ty = row.get("type").and_then(toml::Value::as_str);
        let holding = row
            .get("holding")
            .and_then(toml::Value::as_str)
            .and_then(Holding::from_word);
        let within: BTreeSet<String> = row
            .get("within")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|w| w.as_str().map(str::to_string))
            .collect();
        if let (Some(ty), Some(holding)) = (ty, holding) {
            let held = Held { holding, within };
            match out.get_mut(ty) {
                Some(h) => Held::merge(h, &held),
                None => {
                    out.insert(ty.to_string(), held);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dump of three bodies of a synthetic crate, as rustc prints it.
    const LIFT_DUMP: &str = "// MIR for `bump` before ElaborateBoxDerefs\n\n\
        fn bump(_1: &Rc<RefCell<Tally>>) -> () {\n    let mut _0: ();\n\
        \x20   let mut _2: &mut inner::Tally;\n\
        \x20   let mut _4: std::cell::RefMut<'_, inner::Tally>;\n\
        \x20   let mut _7: &std::rc::Rc<std::cell::RefCell<inner::Tally>>;\n}\n\n\
        fn share(_1: Book) -> Arc<Book> {\n    let mut _0: std::sync::Arc<Book>;\n\
        \x20   let mut _2: Book;\n}\n\n\
        fn keep(_1: Plain) -> u32 {\n    let mut _0: u32;\n\
        \x20   let _3: std::option::Option<Wrap<Plain>>;\n\
        \x20   let _4: {closure@src/lib.rs:9:5: 9:7};\n}\n";

    #[test]
    fn lift_each_type_is_read_by_how_the_program_holds_it() {
        let mut read_back = BTreeMap::new();
        read(LIFT_DUMP, &mut read_back);
        let holding = |p: &str| read_back.get(p).map(|h| h.holding);
        // A state passed as `Rc<RefCell<_>>`: interior-mutable, by the
        // crate-relative path of the locals and the signature's trimmed one.
        assert_eq!(holding("inner::Tally"), Some(Holding::Interior));
        assert_eq!(
            read_back["inner::Tally"].within,
            BTreeSet::from(["Rc<RefCell<_>>".to_string()])
        );
        assert_eq!(holding("Tally"), Some(Holding::Interior));
        assert_eq!(holding("Book"), Some(Holding::Shared));
        assert_eq!(holding("Plain"), Some(Holding::Owned));
        // A generic of the crate is a type too; a `std` path and a
        // wrapper are not.
        assert_eq!(holding("Wrap"), Some(Holding::Owned));
        assert!(read_back
            .keys()
            .all(|k| !k.starts_with("std") && k != "Rc" && k != "RefCell" && k != "Arc"));
        // What a record states reads back the same.
        let doc: toml::Table = render(&read_back).parse().unwrap();
        assert_eq!(parse(&doc), read_back);
    }
}
