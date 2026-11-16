//! How a workspace holds each of its types: owned, shared (behind `Rc` or
//! `Arc`), or interior-mutable (inside a `Cell`, `RefCell`, `Mutex`, ...,
//! or behind a raw pointer, through which aliases mutate it). It is read
//! from two places, and the most of them is the type's holding:
//!
//! - the compiler's dump: a body's locals and signature print every type it
//!   holds with the wrappers around it (`let _7:
//!   &std::rc::Rc<std::cell::RefCell<inner::Tally>>`), so a type a state is
//!   passed in as `Rc<RefCell<T>>` reads interior-mutable with that chain;
//! - the workspace's own type declarations ([`Declared`]): a field's type
//!   holds the types in it the same way (`p: *mut Tally` holds `Tally`
//!   behind a raw pointer), and a type holds what its fields hold, at any
//!   depth (a `Cell` three fields down is mutated through a shared
//!   reference to the outermost type).
//!
//! A type is read by the path the dump prints it by, crate-relative for the
//! crate's own (`inner::Tally`), the signature's trimmed (`Tally`); paths of
//! core, alloc and std and the primitives are not types of the workspace.
//! A wrapper's name printed with one segment is the standard one only where
//! the crate declares no type of that name: rustc trims a path to its name
//! only where no other visible type has it, and prints a crate root's own
//! type with one segment ([`resolve`]). In a declaration's field the source
//! names a wrapper by whatever `use` binds, so a wrapper's name there is
//! taken as the standard one: a crate's own type of that name reads as a
//! cell, which states more holding than there may be, never less. A type
//! inside a crate type's generic arguments (`W<T>`) is held as `W` holds
//! what its fields hold. A type no dump holds is not stated.

use std::collections::{BTreeMap, BTreeSet};

/// How the program holds a type, least first: a type held more than one
/// way is held the most of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Holding {
    /// Never behind a shared pointer or inside a cell.
    Owned,
    /// Behind `Rc`, `Arc` or a `Weak` of one, and in no cell.
    Shared,
    /// Inside a cell or a lock, or behind a raw pointer: mutated through a
    /// shared reference or an alias.
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

/// The raw pointers, as a chain spells them: what is behind one is
/// reached by every copy of it.
const RAW: &[&str] = &["*mut", "*const"];

/// The crates whose paths name no type of the workspace.
const LIBRARY: &[&str] = &["std", "core", "alloc"];

/// One type's reading: its holding and each chain it was found in,
/// outermost first (`Rc<RefCell<_>>`). A chain's words are the wrappers, a
/// raw pointer (`*mut`), a crate type whose generic arguments hold it (its
/// path as printed), and, read from a dump, a wrapper's name printed with
/// one segment (`?Cell`), which [`resolve`] decides.
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

/// Where a type text was printed: a dump, or a declaration's source.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Text {
    Dump,
    Source,
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
            scan(ty, Text::Dump, out);
        }
    }
}

/// The word a generic head `path` is in a chain: a wrapper's name, a
/// one-segment wrapper's name of a dump marked `?`, a crate type's path, or
/// empty for a library type that wraps nothing (`Option`, `Vec`, `Box`).
fn word(path: &str, text: Text) -> String {
    let first = path.split("::").next().unwrap_or_default();
    let last = path.rsplit("::").next().unwrap_or_default();
    let wrapper = INTERIOR.contains(&last) || SHARED.contains(&last);
    if LIBRARY.contains(&first) {
        return if wrapper {
            last.to_string()
        } else {
            String::new()
        };
    }
    if wrapper && first == last {
        return match text {
            Text::Dump => format!("?{last}"),
            Text::Source => last.to_string(),
        };
    }
    if last.starts_with(|c: char| c.is_ascii_uppercase()) {
        path.to_string()
    } else {
        String::new()
    }
}

/// Every type path in the type text `ty`, each read with the words around
/// it; the words of every head the text holds, in order.
fn scan(ty: &str, text: Text, out: &mut BTreeMap<String, Held>) -> Vec<String> {
    let b = ty.as_bytes();
    // The heads open around the place read, each with whether it is a raw
    // pointer, which ends at the end of the type it points at (a `,`, `)`,
    // `]`, `;` or `>` of its level).
    let mut open: Vec<(String, bool)> = Vec::new();
    let mut heads = Vec::new();
    let mut braces = 0usize;
    let mut i = 0;
    let close_raw = |open: &mut Vec<(String, bool)>| {
        while open.last().is_some_and(|(_, raw)| *raw) {
            open.pop();
        }
    };
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
            b'*' => {
                let mut j = i + 1;
                while b.get(j) == Some(&b' ') {
                    j += 1;
                }
                let rest = &ty[j..];
                let kind = ["mut", "const"].into_iter().find(|k| {
                    rest.starts_with(k)
                        && !rest[k.len()..]
                            .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
                });
                if let Some(k) = kind {
                    let w = format!("*{k}");
                    heads.push(w.clone());
                    open.push((w, true));
                    i = j + k.len();
                    continue;
                }
            }
            b',' | b')' | b']' | b';' => close_raw(&mut open),
            b'<' => open.push((String::new(), false)),
            b'>' => {
                close_raw(&mut open);
                open.pop();
                close_raw(&mut open);
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
                    let w = word(path, text);
                    if !w.is_empty() {
                        heads.push(w.clone());
                    }
                    open.push((w, false));
                    i += 1;
                }
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    heads
}

/// The holding a chain's word states by itself: a crate type's is read by
/// [`resolve`], and is none here.
fn alone(word: &str) -> Holding {
    let name = word.trim_start_matches('?');
    if RAW.contains(&word) || INTERIOR.contains(&name) {
        Holding::Interior
    } else if SHARED.contains(&name) {
        Holding::Shared
    } else {
        Holding::Owned
    }
}

/// A chain written as a record states it: `Rc<RefCell<_>>`, `*mut _`.
fn render_chain(words: &[&str]) -> String {
    let mut out = String::new();
    let mut closing = 0;
    for w in words {
        if RAW.contains(w) {
            out.push_str(w);
            out.push(' ');
        } else {
            out.push_str(w);
            out.push('<');
            closing += 1;
        }
    }
    out.push('_');
    out.push_str(&">".repeat(closing));
    out
}

/// The words of a chain [`render_chain`] wrote.
fn chain_words(chain: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut rest = chain;
    loop {
        rest = rest.trim_start();
        if let Some(raw) = RAW.iter().find(|r| {
            rest.strip_prefix(**r)
                .is_some_and(|after| after.starts_with(' '))
        }) {
            words.push((*raw).to_string());
            rest = &rest[raw.len()..];
            continue;
        }
        match rest.split_once('<') {
            Some((w, after)) if !w.is_empty() && !w.starts_with('_') => {
                words.push(w.to_string());
                rest = after;
            }
            _ => return words,
        }
    }
}

/// Read the type `path` held inside the heads `open`, if it can be a
/// type of the workspace.
fn note(path: &str, open: &[(String, bool)], out: &mut BTreeMap<String, Held>) {
    let first = path.split("::").next().unwrap_or_default();
    let last = path.rsplit("::").next().unwrap_or_default();
    if LIBRARY.contains(&first) || !last.starts_with(|c: char| c.is_ascii_uppercase()) {
        return;
    }
    if path.split("::").count() == 1 && (INTERIOR.contains(&last) || SHARED.contains(&last)) {
        return;
    }
    let chain: Vec<&str> = open
        .iter()
        .map(|(w, _)| w.as_str())
        .filter(|w| !w.is_empty())
        .collect();
    let mut within = BTreeSet::new();
    if !chain.is_empty() {
        within.insert(render_chain(&chain));
    }
    let held = Held {
        holding: chain
            .iter()
            .map(|w| alone(w))
            .max()
            .unwrap_or(Holding::Owned),
        within,
    };
    match out.get_mut(path) {
        Some(h) => h.merge(&held),
        None => {
            out.insert(path.to_string(), held);
        }
    }
}

/// One type of a crate as the join knows it: the path the frontier names
/// it by, its name, and each field's name and type as its declaration
/// writes them (`Variant.field` for an enum's).
#[derive(Clone, Debug)]
pub(crate) struct Declared {
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) fields: Vec<(String, String)>,
}

/// The holdings of one crate's types: `read`, what the dumps state of each
/// type (by its path in `declared`), and the crate's declarations. A word
/// `?X` of a dump is the standard wrapper `X` where the crate declares no
/// type named `X`, and that type otherwise; a crate type's word holds what
/// that type's fields hold. Each type the dumps hold is stated with the
/// most of: its dumps' chains, the chains its crate's fields hold it in
/// (`Raw.p: *mut _`), and what its own fields hold, at any depth (`field
/// mid: Mid`, `field c: Cell<u32>`).
pub(crate) fn resolve(
    read: &BTreeMap<String, Held>,
    declared: &[Declared],
) -> BTreeMap<String, Held> {
    let by_name: BTreeMap<&str, Vec<&Declared>> =
        declared.iter().fold(BTreeMap::new(), |mut m, d| {
            m.entry(d.name.as_str()).or_insert_with(Vec::new).push(d);
            m
        });
    let named = |word: &str| -> Vec<&Declared> {
        let last = word.rsplit("::").next().unwrap_or_default();
        by_name.get(last).cloned().unwrap_or_default()
    };
    // What each type's fields hold, by itself and through the crate's
    // types they name, to a fixed point.
    let mut content: BTreeMap<&str, (Holding, BTreeSet<String>)> = BTreeMap::new();
    let mut refs: BTreeMap<&str, Vec<(String, &str)>> = BTreeMap::new();
    let mut in_fields: BTreeMap<&str, Held> = BTreeMap::new();
    for d in declared {
        let mut own = (Holding::Owned, BTreeSet::new());
        for (field, ty) in &d.fields {
            let mut noted = BTreeMap::new();
            let heads = scan(ty, Text::Source, &mut noted);
            let direct = heads
                .iter()
                .map(|w| alone(w))
                .max()
                .unwrap_or(Holding::Owned);
            if direct > Holding::Owned {
                own.0 = own.0.max(direct);
                own.1.insert(format!("field {field}: {ty}"));
            }
            for (path, held) in noted {
                for u in named(&path) {
                    refs.entry(d.path.as_str())
                        .or_default()
                        .push((format!("field {field}: {ty}"), u.path.as_str()));
                    let at = Held {
                        holding: held.holding,
                        within: held
                            .within
                            .iter()
                            .map(|c| format!("{}.{field}: {c}", d.path))
                            .collect(),
                    };
                    match in_fields.get_mut(u.path.as_str()) {
                        Some(h) => h.merge(&at),
                        None => {
                            in_fields.insert(u.path.as_str(), at);
                        }
                    }
                }
            }
            for w in heads.iter().filter(|w| alone(w) == Holding::Owned) {
                for u in named(w) {
                    refs.entry(d.path.as_str())
                        .or_default()
                        .push((format!("field {field}: {ty}"), u.path.as_str()));
                }
            }
        }
        content.insert(d.path.as_str(), own);
    }
    loop {
        let mut changed = false;
        for (from, edges) in &refs {
            for (why, to) in edges {
                let theirs = content.get(to).map_or(Holding::Owned, |c| c.0);
                let mine = content
                    .entry(from)
                    .or_insert((Holding::Owned, BTreeSet::new()));
                if theirs > Holding::Owned && !mine.1.contains(why) {
                    mine.0 = mine.0.max(theirs);
                    mine.1.insert(why.clone());
                    changed = true;
                } else if theirs > mine.0 {
                    mine.0 = theirs;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    // What the crate types a word names hold in their fields.
    let holds = |word: &str| {
        named(word)
            .iter()
            .map(|u| content.get(u.path.as_str()).map_or(Holding::Owned, |c| c.0))
            .max()
            .unwrap_or(Holding::Owned)
    };
    // A chain's words decided: each word's holding, and the chain as shown,
    // of the words that hold anything.
    let decide = |words: Vec<String>| -> (Holding, String) {
        let mut holding = Holding::Owned;
        let mut shown: Vec<String> = Vec::new();
        for w in words {
            let (h, name) = match w.strip_prefix('?') {
                Some(name) if by_name.contains_key(name) => (holds(name), name.to_string()),
                Some(name) => (alone(&w), name.to_string()),
                None if alone(&w) > Holding::Owned => (alone(&w), w.clone()),
                None => (holds(&w), w.clone()),
            };
            if h > Holding::Owned {
                holding = holding.max(h);
                shown.push(name);
            }
        }
        let words: Vec<&str> = shown.iter().map(String::as_str).collect();
        let shown = if words.is_empty() {
            String::new()
        } else {
            render_chain(&words)
        };
        (holding, shown)
    };
    let mut out = BTreeMap::new();
    for (path, held) in read {
        let mut stated = Held {
            holding: Holding::Owned,
            within: BTreeSet::new(),
        };
        for chain in &held.within {
            let (holding, shown) = decide(chain_words(chain));
            stated.holding = stated.holding.max(holding);
            if !shown.is_empty() {
                stated.within.insert(shown);
            }
        }
        if let Some(h) = in_fields.get(path.as_str()) {
            for chain in &h.within {
                // `Type.field: chain`, of a source's words.
                let words = chain.rsplit_once(": ").map_or(chain.as_str(), |(_, c)| c);
                let (holding, _) = decide(chain_words(words));
                if holding > Holding::Owned {
                    stated.holding = stated.holding.max(holding);
                    stated.within.insert(chain.clone());
                }
            }
        }
        if let Some((holding, why)) = content.get(path.as_str()) {
            stated.holding = stated.holding.max(*holding);
            stated.within.extend(why.iter().cloned());
        }
        out.insert(path.clone(), stated);
    }
    out
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

    /// The refuter's crate (refute-lift-7, the ownership reading): a cell
    /// three fields down, a raw pointer two values alias, an `Rc` one and
    /// two fields down, and a crate root type named `Cell` that is a plain
    /// wrapper; and how its bodies hold each, as rustc prints them (a crate
    /// root's own type with one segment, so `Cell<Plain3>` is the crate's).
    const LIFT_OWN_DUMP: &str = "fn bump(_1: &own::Top) -> u32 {\n    let mut _0: u32;\n}\n\
        fn via_raw(_1: &own::Raw) -> u32 {\n    let mut _2: *mut own::Tally;\n}\n\
        fn share(_1: &own::Outer) -> own::Outer {\n    let mut _3: std::rc::Rc<own::Count>;\n}\n\
        fn wrap3(_1: Plain3) -> Cell<Plain3> {\n    let mut _0: Cell<Plain3>;\n}\n";

    fn lift_declared() -> Vec<Declared> {
        let d = |path: &str, fields: &[(&str, &str)]| Declared {
            path: path.to_string(),
            name: path.rsplit("::").next().unwrap().to_string(),
            fields: fields
                .iter()
                .map(|(f, t)| (f.to_string(), t.to_string()))
                .collect(),
        };
        vec![
            d("probe::own::Leaf", &[("c", "Cell<u32>")]),
            d("probe::own::Mid", &[("leaf", "Leaf")]),
            d("probe::own::Top", &[("mid", "Mid")]),
            d("probe::own::Tally", &[("n", "u32")]),
            d("probe::own::Raw", &[("p", "* mut Tally")]),
            d("probe::own::Count", &[("n", "u32")]),
            d("probe::own::HoldsRc", &[("inner", "Rc<Count>")]),
            d("probe::own::Outer", &[("h", "HoldsRc")]),
            d("probe::Cell", &[("0", "T")]),
            d("probe::Plain3", &[("n", "u32")]),
        ]
    }

    #[test]
    fn lift_a_type_holds_what_its_fields_hold_and_a_crate_cell_is_no_cell() {
        let mut dumped = BTreeMap::new();
        read(LIFT_OWN_DUMP, &mut dumped);
        // The join names each printed path by its declared one.
        let read_back: BTreeMap<String, Held> = dumped
            .into_iter()
            .map(|(p, h)| (format!("probe::{p}"), h))
            .collect();
        let stated = resolve(&read_back, &lift_declared());
        let holding = |p: &str| stated.get(p).map(|h| h.holding);
        // Mutated through `&Top` by a cell three fields down.
        assert_eq!(holding("probe::own::Top"), Some(Holding::Interior));
        assert!(stated["probe::own::Top"].within.contains("field mid: Mid"));
        // Two `Raw` alias one `Tally` through a raw pointer: both are held
        // so, by the field and by the dump's local.
        assert_eq!(holding("probe::own::Raw"), Some(Holding::Interior));
        assert_eq!(holding("probe::own::Tally"), Some(Holding::Interior));
        assert!(
            stated["probe::own::Tally"].within.contains("*mut _"),
            "{:?}",
            stated["probe::own::Tally"]
        );
        // An `Rc` one and two fields down shares what it holds.
        assert_eq!(holding("probe::own::Outer"), Some(Holding::Shared));
        assert_eq!(holding("probe::own::Count"), Some(Holding::Shared));
        // The crate's own `Cell` is no cell: `Plain3` is owned.
        assert_eq!(holding("probe::Plain3"), Some(Holding::Owned));
        assert!(stated["probe::Plain3"].within.is_empty());
        // Where the crate declares no type of a wrapper's name, a trimmed
        // one-segment name is the standard wrapper.
        let declared: Vec<Declared> = lift_declared()
            .into_iter()
            .filter(|d| d.name != "Cell")
            .collect();
        let stated = resolve(&read_back, &declared);
        assert_eq!(
            stated.get("probe::Plain3").map(|h| h.holding),
            Some(Holding::Interior)
        );
    }
}
