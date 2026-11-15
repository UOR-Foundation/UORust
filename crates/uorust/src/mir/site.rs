//! Where a body's source declares it, read from the spans a dump prints
//! under `-Zmir-include-spans` (`super::parse::dumped`): whether the body
//! tracks its caller (`#[track_caller]`), and the call sites whose span is
//! the location rustc's `Location::caller` reports.
//!
//! rustc reports a call's location at the outermost macro call its span
//! comes from (`Span::ctxt().outer_expn().expansion_cause()`), which a dump
//! does not print: a span inside a macro's definition is the definition's,
//! not the call's. So a call site is read only where it lies inside the
//! source text of the function its body is, and that function is not
//! written by a macro; every other call site is unread, and a location
//! read through it is undefined.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::ir::{IrBody, IrSpan};

/// A position in a source file: its 1-based line and 1-based column, in
/// characters, as rustc prints a span's start.
type Position = (u32, u32);

/// What a body's source declares of it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Declared {
    /// Whether it tracks its caller: `None` where its source is not read,
    /// or where `#[track_caller]` is conditional (`#[cfg_attr(miri,
    /// track_caller)]`), whose condition is the build's.
    pub(crate) tracked: Option<bool>,
    /// The file and the text of the function it is in, from `fn` to its
    /// closing brace, where a call's span is the call's own.
    pub(crate) extent: Option<(String, Position, Position)>,
}

impl Declared {
    /// Whether `span` is a call site whose location rustc reports as it
    /// prints it.
    pub(crate) fn reads(&self, span: &IrSpan) -> bool {
        self.extent.as_ref().is_some_and(|(file, start, end)| {
            *file == span.file && (*start..=*end).contains(&(span.line, span.column))
        })
    }
}

/// A source file's code with its comments and literals blanked, line by
/// line, and the macros it defines.
struct Source {
    /// Each line's characters, every one of a comment, a string or a
    /// character literal a space.
    code: Vec<Vec<char>>,
    /// Each macro definition's text, `macro_rules! m { .. }` or `macro m
    /// { .. }`: its first and last position.
    macros: Vec<(Position, Position)>,
}

/// The library's sources, read as a body's span names them.
pub(crate) struct Sources {
    root: PathBuf,
    files: RefCell<BTreeMap<String, Option<std::rc::Rc<Source>>>>,
}

impl Sources {
    /// The sources under `root`, the directory the dumps' spans are
    /// relative to (rust-src's `library` is under it).
    pub(crate) fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            files: RefCell::new(BTreeMap::new()),
        }
    }

    fn source(&self, file: &str) -> Option<std::rc::Rc<Source>> {
        self.files
            .borrow_mut()
            .entry(file.to_string())
            .or_insert_with(|| {
                std::fs::read_to_string(self.root.join(file))
                    .ok()
                    .map(|t| std::rc::Rc::new(Source::read(&t)))
            })
            .clone()
    }

    /// What the source declares of `body`.
    pub(crate) fn declared(&self, body: &IrBody) -> Declared {
        let path = &body.path;
        // Promoted and anonymous constants never track a caller: they are
        // no functions.
        if path.contains("promoted[") || path.contains("{constant#") {
            return Declared {
                tracked: Some(false),
                extent: None,
            };
        }
        let segments = super::parse::split_path(path);
        let closure = segments.last().is_some_and(|s| s.starts_with("{closure#"));
        let Some(name) = segments
            .iter()
            .rev()
            .find(|s| !s.starts_with("{closure#"))
            .map(|s| s.split('<').next().unwrap_or(s))
        else {
            return Declared::default();
        };
        let Some(span) = &body.span else {
            return Declared::default();
        };
        let Some(source) = self.source(&span.file) else {
            return Declared::default();
        };
        let Some((start, attributes)) = source.function(name, (span.line, span.column)) else {
            return Declared::default();
        };
        // A closure tracks no caller (`closure_track_caller` is unstable,
        // and core does not use it).
        let tracked = if closure {
            Some(false)
        } else {
            tracks(&attributes)
        };
        let in_macro = source
            .macros
            .iter()
            .any(|(from, to)| (*from..=*to).contains(&start));
        let extent = (!in_macro)
            .then(|| source.body_end(start))
            .flatten()
            .map(|end| (span.file.clone(), start, end));
        Declared { tracked, extent }
    }
}

/// Whether attributes (the code before an item, comments and literals
/// blanked) make it track its caller: `#[track_caller]` does, a
/// conditional one (`#[cfg_attr(c, track_caller)]`) depends on the build.
fn tracks(attributes: &str) -> Option<bool> {
    let words: String = attributes.chars().filter(|c| !c.is_whitespace()).collect();
    if words.contains("#[track_caller]") {
        Some(true)
    } else if words.contains("track_caller") {
        None
    } else {
        Some(false)
    }
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl Source {
    fn read(text: &str) -> Self {
        let code = blank(text);
        let mut source = Self {
            code,
            macros: Vec::new(),
        };
        source.macros = source.macro_definitions();
        source
    }

    fn at(&self, (line, column): Position) -> Option<char> {
        let l = self.code.get(usize::try_from(line).ok()?.checked_sub(1)?)?;
        l.get(usize::try_from(column).ok()?.checked_sub(1)?)
            .copied()
    }

    /// Every position after `from`, in order.
    fn after(&self, from: Position) -> impl Iterator<Item = (Position, char)> + '_ {
        let (line, column) = from;
        self.code
            .iter()
            .enumerate()
            .skip(usize::try_from(line).unwrap_or_default().saturating_sub(1))
            .flat_map(move |(i, l)| {
                let n = u32::try_from(i + 1).unwrap_or_default();
                l.iter().enumerate().filter_map(move |(j, c)| {
                    let p = (n, u32::try_from(j + 1).unwrap_or_default());
                    (p > (line, column)).then_some((p, *c))
                })
            })
    }

    /// The position of the bracket that closes the one at `open`.
    fn closing(&self, open: Position) -> Option<Position> {
        let (o, c) = match self.at(open)? {
            '{' => ('{', '}'),
            '(' => ('(', ')'),
            '[' => ('[', ']'),
            _ => return None,
        };
        let mut depth = 1usize;
        for (p, ch) in self.after(open) {
            if ch == o {
                depth += 1;
            } else if ch == c {
                depth -= 1;
                if depth == 0 {
                    return Some(p);
                }
            }
        }
        None
    }

    /// The text of each `macro_rules! name { .. }` and `macro name ..
    /// { .. }` the file writes.
    fn macro_definitions(&self) -> Vec<(Position, Position)> {
        let mut out = Vec::new();
        for (i, l) in self.code.iter().enumerate() {
            let text: String = l.iter().collect();
            let line = u32::try_from(i + 1).unwrap_or_default();
            for (at, word) in [("macro_rules!", true), ("macro ", false)]
                .iter()
                .flat_map(|(w, rules)| text.match_indices(w).map(move |(a, _)| (a, *rules)))
            {
                let column = u32::try_from(text[..at].chars().count() + 1).unwrap_or_default();
                if text[..at].chars().last().is_some_and(is_ident) {
                    continue;
                }
                let start = (line, column);
                // `macro_rules! m {`, `(` or `[`; `macro m(..) {` or
                // `macro m {`: its body is the first brace after its name.
                let open = self
                    .after(start)
                    .find(|(_, c)| {
                        if word {
                            matches!(c, '{' | '(' | '[')
                        } else {
                            *c == '{'
                        }
                    })
                    .map(|(p, _)| p);
                if let Some(end) = open.and_then(|o| self.closing(o)) {
                    out.push((start, end));
                }
            }
        }
        out
    }

    /// The function `name` whose signature holds `within` (its return
    /// type's span): the position of its `fn`, and the code of its
    /// attributes and qualifiers. The nearest `fn` at or before `within`
    /// must be `name`'s.
    fn function(&self, name: &str, within: Position) -> Option<(Position, String)> {
        let (line, column) = within;
        for l in (1..=line).rev() {
            let chars = self.code.get(usize::try_from(l - 1).ok()?)?;
            let text: String = chars.iter().collect();
            let last = text
                .match_indices("fn")
                .filter(|(at, _)| {
                    let before = text[..*at].chars().last();
                    let after = text[at + 2..].chars().next();
                    !before.is_some_and(is_ident) && after.is_some_and(char::is_whitespace)
                })
                .map(|(at, _)| {
                    (
                        u32::try_from(text[..at].chars().count() + 1).unwrap_or(0),
                        at,
                    )
                })
                .filter(|(c, _)| l < line || *c < column)
                .last();
            let Some((c, at)) = last else {
                continue;
            };
            let named: String = text[at + 2..]
                .trim_start()
                .chars()
                .take_while(|c| is_ident(*c))
                .collect();
            if named != name {
                return None;
            }
            // Its attributes: the lines above up to the end of the item
            // before it, and its qualifiers before `fn` on its own line.
            let mut attributes = text[..at].to_string();
            for above in (1..l).rev() {
                let t: String = self.code[usize::try_from(above - 1).ok()?].iter().collect();
                let t = t.trim();
                if t.ends_with('}') || t.ends_with(';') || t.ends_with('{') {
                    break;
                }
                attributes = format!("{t}\n{attributes}");
            }
            return Some(((l, c), attributes));
        }
        None
    }

    /// The closing brace of the body of the function whose `fn` is at
    /// `start`: none where it declares no body.
    fn body_end(&self, start: Position) -> Option<Position> {
        let mut depth = 0usize;
        for (p, c) in self.after(start) {
            match c {
                '(' | '[' => depth += 1,
                ')' | ']' => depth = depth.saturating_sub(1),
                ';' if depth == 0 => return None,
                '{' if depth == 0 => return self.closing(p),
                _ => {}
            }
        }
        None
    }
}

/// `text`'s lines, with every character of a comment, a string literal or a
/// character literal a space, so that a bracket or a word in one is never
/// code.
fn blank(text: &str) -> Vec<Vec<char>> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<char> = Vec::with_capacity(chars.len());
    let mut i = 0;
    let space = |c: char| if c == '\n' { '\n' } else { ' ' };
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        let ident_before = i > 0 && is_ident(chars[i - 1]);
        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if c == '/' && next == Some('*') {
            let mut depth = 0usize;
            while i < chars.len() {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    out.extend([' ', ' ']);
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    out.extend([' ', ' ']);
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    out.push(space(chars[i]));
                    i += 1;
                }
            }
        } else if c == 'r' && !ident_before && matches!(next, Some('#' | '"')) && {
            let hashes = chars[i + 1..].iter().take_while(|c| **c == '#').count();
            chars.get(i + 1 + hashes) == Some(&'"')
        } {
            // A raw string: `r#"` .. `"#`, with as many hashes.
            let hashes = chars[i + 1..].iter().take_while(|c| **c == '#').count();
            let open = 2 + hashes;
            out.extend(std::iter::repeat_n(' ', open));
            i += open;
            while i < chars.len() {
                if chars[i] == '"'
                    && chars[i + 1..]
                        .iter()
                        .take(hashes)
                        .filter(|c| **c == '#')
                        .count()
                        == hashes
                {
                    out.extend(std::iter::repeat_n(' ', 1 + hashes));
                    i += 1 + hashes;
                    break;
                }
                out.push(space(chars[i]));
                i += 1;
            }
        } else if c == '"' {
            out.push(' ');
            i += 1;
            while i < chars.len() {
                let d = chars[i];
                if d == '\\' {
                    out.push(' ');
                    if let Some(e) = chars.get(i + 1) {
                        out.push(space(*e));
                    }
                    i += 2;
                    continue;
                }
                out.push(space(d));
                i += 1;
                if d == '"' {
                    break;
                }
            }
        } else if c == '\'' && !ident_before {
            // A character literal, `'x'` or `'\n'`; a lifetime (`'a`) has
            // no closing quote after its first character.
            let end = if next == Some('\\') {
                chars[i + 2..]
                    .iter()
                    .position(|c| *c == '\'')
                    .map(|p| i + 2 + p)
            } else if chars.get(i + 2) == Some(&'\'') {
                Some(i + 2)
            } else {
                None
            };
            match end {
                Some(e) => {
                    out.extend(std::iter::repeat_n(' ', e + 1 - i));
                    i = e + 1;
                }
                None => {
                    out.push(c);
                    i += 1;
                }
            }
        } else {
            out.push(c);
            i += 1;
        }
    }
    out.split(|c| *c == '\n').map(<[char]>::to_vec).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = r#"/// Docs that name #[track_caller] and a brace {.
#[stable(feature = "track_caller", since = "1.46.0")]
#[track_caller]
#[inline]
pub const fn caller() -> &'static str {
    let c = '}';
    inner("}")
}

#[cfg_attr(miri, track_caller)]
pub fn maybe(x: u32) -> u32 {
    x
}

pub fn plain() {
    let f = || caller();
    f();
}

macro_rules! m {
    () => {
        #[track_caller]
        pub fn made() -> u32 { caller().len() as u32 }
    };
}
"#;

    fn body(path: &str, line: u32, column: u32) -> IrBody {
        IrBody {
            path: path.to_string(),
            span: Some(IrSpan {
                file: "library/core/src/x.rs".to_string(),
                line,
                column,
            }),
            ..IrBody::default()
        }
    }

    fn sources() -> (Sources, PathBuf) {
        let dir = std::env::temp_dir().join(format!("uorust-site-{}", std::process::id()));
        let file = dir.join("library/core/src/x.rs");
        std::fs::create_dir_all(file.parent().unwrap_or(&dir)).unwrap_or_default();
        std::fs::write(&file, SOURCE).unwrap_or_default();
        (Sources::new(&dir), dir)
    }

    #[test]
    fn reads_whether_a_body_tracks_its_caller_and_where_its_calls_are() {
        let (sources, dir) = sources();
        let site = |line, column| IrSpan {
            file: "library/core/src/x.rs".to_string(),
            line,
            column,
        };
        let caller = sources.declared(&body("x::caller", 5, 26));
        assert_eq!(caller.tracked, Some(true));
        assert!(caller.reads(&site(7, 5)));
        assert!(!caller.reads(&site(16, 16)));
        assert_eq!(sources.declared(&body("x::maybe", 11, 25)).tracked, None);
        let plain = sources.declared(&body("x::plain", 15, 15));
        assert_eq!(plain.tracked, Some(false));
        assert!(plain.reads(&site(17, 5)));
        let closure = sources.declared(&body("x::plain::{closure#0}", 16, 16));
        assert_eq!(closure.tracked, Some(false));
        assert!(closure.reads(&site(16, 16)));
        // A function a macro writes: its calls' spans are the macro's.
        let made = sources.declared(&body("x::made", 23, 26));
        assert_eq!(made.tracked, Some(true));
        assert!(!made.reads(&site(23, 32)));
        // A body whose span names no function of its name.
        assert_eq!(
            sources.declared(&body("x::other", 15, 15)),
            Declared::default()
        );
        std::fs::remove_dir_all(dir).unwrap_or_default();
    }
}
