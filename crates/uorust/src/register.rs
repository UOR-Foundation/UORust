//! The lift register: every function of product code, keyed by the κ of its
//! question, with what the lift decided for it.
//!
//! A function's κ is the SHA-256 of [`DOMAIN`], its crate path and its
//! normalized token stream (signature and body as `syn` reads them, so
//! comments, doc comments and layout do not move it). The κ names the
//! question "does this function lift?", not the answer: a function whose
//! source changes is a new question with a new κ.
//!
//! A lifted κ is a proved claim. The ratchet is monotone, not a count: a
//! candidate register is admitted against the incumbent (the committed one)
//! exactly when every κ the incumbent records as lifted is still lifted, or
//! is absent because its source changed or it was removed ([`admit`]).
//! Residue, foreign and effect rows classify what is not lifted yet; none of
//! them is a budget. Their number is the distance to the requirement that
//! every function of product code lifts, an open claim the honesty report
//! states and no gate asserts.
//!
//! The register is also the work queue. Beside its rows it carries, as data,
//! the totals ([`Totals`], the `[lift]` table) and one `[[construct]]` row
//! for each cause and detail that stops a function ([`Construct`]), the one
//! that blocks the most functions first: the order and the row shape of the
//! queue `uorust core` writes into the core register ([`ranked`],
//! [`queue_row`]). Both are derived from the rows, so a register that states
//! them otherwise is not one ([`Register::parse`]); a register written
//! before they existed carries neither and reads as it always did.

use std::collections::BTreeMap;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::UorustError;

/// The domain tag every function κ starts with.
pub const DOMAIN: &str = "uorust/function/1";

/// The schema of a register file written apart from a projection.
pub const SCHEMA: &str = "uorust/register/1";

/// The table a register's totals are written under.
pub const LIFT: &str = "lift";

/// The table of a register's queue rows, and the key that names a row's
/// construct: the core register's.
pub const CONSTRUCT: &str = "construct";

/// The key a queue row's cause is written under.
const CAUSE: &str = "cause";

/// The unit a workspace register's queue counts in: what a construct blocks
/// is functions of product code.
const UNIT: &str = "functions";

/// The κ of a function: SHA-256 over [`DOMAIN`], its crate path and its
/// normalized token stream, one line each, as lowercase hex.
pub fn kappa(path: &str, tokens: &str) -> String {
    let digest = Sha256::digest(kappa_preimage(path, tokens));
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// The bytes [`kappa`] hashes: [`DOMAIN`], the path and the tokens, each
/// ended by a newline. The model decides the framing (`Kappa::kappaPreimage`
/// in app/src/Kappa.lex.tex, whose path is bytes that are never a newline):
/// the path and the tokens are each read back from the preimage
/// (`Kappa::thePathIsReadFromThePreimage`,
/// `Kappa::theTokensAreReadFromThePreimage`), so two functions with one
/// preimage are one function, and this adapter only hashes. A crate path is
/// `::`-joined identifiers and never holds a newline; one that did is the
/// one input the framing cannot tell apart
/// (`Kappa::aNewlineInThePathBreaksTheFraming`), so it is refused.
pub fn kappa_preimage(path: &str, tokens: &str) -> Vec<u8> {
    assert!(
        !path.contains('\n'),
        "a crate path holds a newline, which the κ framing cannot tell apart: {path:?}"
    );
    format!("{DOMAIN}\n{path}\n{tokens}\n").into_bytes()
}

/// What the lift decided for one function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// It lifted to this LexLean definition.
    Lifted(String),
    /// A pure function outside the lift's floor: the cause's code and detail.
    Residue(String, String),
    /// A parameter or the result is a registry crate's type: the detail names
    /// the crate and the type.
    Foreign(String),
    /// It performs or reaches an operating-system effect: the kind.
    Effect(String),
}

impl Status {
    /// The row table this status is written under.
    pub fn table(&self) -> &'static str {
        match self {
            Self::Lifted(_) => "lifted",
            Self::Residue(..) => "residue",
            Self::Foreign(_) => "foreign",
            Self::Effect(_) => "effect",
        }
    }
}

impl Status {
    /// What stops a function that is not lifted, as `(cause, construct)`:
    /// the construct is the row's own detail (a residue or foreign row's
    /// `detail`, an effect row's `effect`), so the rows a construct blocks
    /// are the rows that state it; the cause is a residue row's reason, or
    /// for a row of the MIR path the MIR kind or obligation its detail
    /// begins with, and the row's table for a foreign or an effect row. A
    /// lifted function is stopped by nothing.
    pub fn construct(&self) -> Option<(String, &str)> {
        match self {
            Self::Lifted(_) => None,
            Self::Residue(reason, detail) => Some((residue_cause(reason, detail).0, detail)),
            Self::Foreign(detail) => Some((self.table().to_string(), detail)),
            Self::Effect(effect) => Some((self.table().to_string(), effect)),
        }
    }
}

/// A residue row's cause and the rest of its detail: for a row of the MIR
/// path (`mir::Kind: rest`) the MIR kind and what follows it, for any other
/// row its reason and its whole detail.
fn residue_cause(reason: &str, detail: &str) -> (String, String) {
    match detail.strip_prefix("mir::") {
        Some(d) => match d.split_once(": ") {
            Some((kind, rest)) => (format!("mir::{kind}"), rest.to_string()),
            None => (format!("mir::{d}"), String::new()),
        },
        None => (reason.to_string(), detail.to_string()),
    }
}

/// The totals of a register: the functions of product code, and how many of
/// them each table holds. The four tables part the functions, so they sum
/// to `functions`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Totals {
    /// Every function of product code.
    pub functions: usize,
    /// Those that lifted.
    pub lifted: usize,
    /// Those outside the lift's floor.
    pub residue: usize,
    /// Those at the boundary with a registry crate's type.
    pub foreign: usize,
    /// Those that perform or reach an operating-system effect.
    pub effect: usize,
}

impl Totals {
    /// The totals by the key each is written under, in the order written.
    pub fn fields(&self) -> [(&'static str, usize); 5] {
        [
            ("functions", self.functions),
            ("lifted", self.lifted),
            ("residue", self.residue),
            ("foreign", self.foreign),
            ("effect", self.effect),
        ]
    }
}

/// One row of the queue: what stops functions, and how many it stops.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Construct {
    /// The cause: see [`Status::construct`].
    pub cause: String,
    /// The construct: the detail the rows it blocks state.
    pub construct: String,
    /// The functions it blocks.
    pub functions: usize,
}

/// A queue in the order every register the tool writes ranks one: what
/// blocks the most first, and what blocks as many by its name.
pub fn ranked<K: Ord>(counts: BTreeMap<K, usize>) -> Vec<(K, usize)> {
    let mut queue: Vec<(K, usize)> = counts.into_iter().collect();
    queue.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    queue
}

/// One row of a ranked queue, in the shape of the core register's: its
/// table, the keys that name what blocks, and what it blocks counted under
/// `unit`.
pub fn queue_row(table: &str, names: &[(&str, &str)], unit: &str, count: usize) -> String {
    let mut out = format!("[[{table}]]\n");
    for (key, value) in names {
        out.push_str(&format!("{key} = {}\n", quoted(value)));
    }
    out.push_str(&format!("{unit} = {count}\n"));
    out
}

/// One function of the register.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// Its crate path, `crate::module::Type::name`.
    pub function: String,
    /// What the lift decided.
    pub status: Status,
    /// Where a lifted function's definition parts from the Rust function:
    /// a cause the lift states, such as a Unicode table LexLean does not
    /// bind. A row that is not lifted has none.
    pub divergence: Option<String>,
}

/// Every function of product code, by κ.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Register {
    /// Rows by κ.
    pub rows: BTreeMap<String, Row>,
}

impl Register {
    /// Read the function rows of a `frontier.toml` or a register file. Every
    /// row carries a κ that appears once, every function appears once, and
    /// every row that is not lifted is typed: a residue row its reason and
    /// detail, a foreign row the crate of its type, an effect row its effect.
    /// Where it states its totals or its queue, they are those of its rows;
    /// a register written before either existed states neither and is read
    /// by its rows alone. A register that is not one is
    /// [`UorustError::Malformed`] at `path`.
    pub fn parse(path: &Path, text: &str) -> Result<Register, UorustError> {
        let bad = |reason: String| UorustError::Malformed {
            path: path.to_path_buf(),
            reason,
        };
        let doc: toml::Value = text.parse().map_err(|e| bad(format!("{e}")))?;
        let mut register = Register::default();
        let mut functions = BTreeMap::new();
        for table in ["lifted", "residue", "foreign", "effect"] {
            let rows = doc.get(table).and_then(toml::Value::as_array);
            for row in rows.into_iter().flatten() {
                let field = |name: &str| {
                    row.get(name)
                        .and_then(toml::Value::as_str)
                        .filter(|v| !v.is_empty())
                        .map(str::to_string)
                        .ok_or_else(|| bad(format!("a {table} row without {name}: {row}")))
                };
                let kappa = field("kappa")?;
                let function = field("function")?;
                let status = match table {
                    "lifted" => Status::Lifted(field("definition")?),
                    "residue" => Status::Residue(field("reason")?, field("detail")?),
                    "foreign" => {
                        let detail = field("detail")?;
                        if !detail.contains(" crate with no LexLean model") {
                            return Err(bad(format!("a foreign row names no crate: {row}")));
                        }
                        Status::Foreign(detail)
                    }
                    _ => Status::Effect(field("effect")?),
                };
                if kappa.len() != 64 || !kappa.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err(bad(format!("{function}: κ {kappa} is not a SHA-256")));
                }
                if functions.insert(function.clone(), ()).is_some() {
                    return Err(bad(format!("{function} has more than one row")));
                }
                let divergence = match row.get("divergence") {
                    None => None,
                    Some(_) if table != "lifted" => {
                        return Err(bad(format!("a {table} row with a divergence: {row}")));
                    }
                    Some(_) => Some(field("divergence")?),
                };
                let new = Row {
                    function,
                    status,
                    divergence,
                };
                if register.rows.insert(kappa.clone(), new).is_some() {
                    return Err(bad(format!("κ {kappa} has more than one row")));
                }
            }
        }
        register.stated(&doc).map_err(bad)?;
        Ok(register)
    }

    /// Whether the totals and the queue `doc` states are those of the rows:
    /// the totals where it has a `[lift]` table, and the queue, row for row
    /// in its ranked order, where it has that table or any `[[construct]]`
    /// row. Both are derived data, so a register that states them otherwise
    /// was not written from its rows; one that states neither is older than
    /// both and is not asked for them.
    fn stated(&self, doc: &toml::Value) -> Result<(), String> {
        let totals = doc.get(LIFT);
        if let Some(stated) = totals {
            for (key, counted) in self.totals().fields() {
                let found = stated.get(key).and_then(toml::Value::as_integer);
                if found != i64::try_from(counted).ok() {
                    return Err(format!(
                        "[{LIFT}] states {key} = {} and the rows hold {counted}",
                        found.map_or_else(|| "nothing".to_string(), |n| n.to_string())
                    ));
                }
            }
        }
        let rows = doc.get(CONSTRUCT).and_then(toml::Value::as_array);
        if totals.is_none() && rows.is_none() {
            return Ok(());
        }
        let stated: Vec<&toml::Value> = rows.into_iter().flatten().collect();
        let queue = self.queue();
        if stated.len() != queue.len() {
            return Err(format!(
                "the queue states {} [[{CONSTRUCT}]] rows and the rows give {}",
                stated.len(),
                queue.len()
            ));
        }
        for (i, (found, expected)) in stated.iter().zip(&queue).enumerate() {
            let text = |key: &str| found.get(key).and_then(toml::Value::as_str);
            let same = text(CONSTRUCT) == Some(expected.construct.as_str())
                && text(CAUSE) == Some(expected.cause.as_str())
                && found.get(UNIT).and_then(toml::Value::as_integer)
                    == i64::try_from(expected.functions).ok();
            if !same {
                return Err(format!(
                    "[[{CONSTRUCT}]] row {} of the queue is {found} and the rows give \
                     {} under {}, blocking {} {UNIT}",
                    i + 1,
                    quoted(&expected.construct),
                    quoted(&expected.cause),
                    expected.functions
                ));
            }
        }
        Ok(())
    }

    /// The totals: every function, and how many each table holds.
    pub fn totals(&self) -> Totals {
        let mut totals = Totals {
            functions: self.rows.len(),
            ..Totals::default()
        };
        for row in self.rows.values() {
            match row.status {
                Status::Lifted(_) => totals.lifted += 1,
                Status::Residue(..) => totals.residue += 1,
                Status::Foreign(_) => totals.foreign += 1,
                Status::Effect(_) => totals.effect += 1,
            }
        }
        totals
    }

    /// The work queue: each cause and detail that stops a function, with
    /// the functions it blocks, ranked ([`ranked`]). Every function that is
    /// not lifted is blocked by exactly one row, so the rows sum to the
    /// residue, foreign and effect totals.
    pub fn queue(&self) -> Vec<Construct> {
        let mut counts: BTreeMap<(String, &str), usize> = BTreeMap::new();
        for row in self.rows.values() {
            if let Some(key) = row.status.construct() {
                *counts.entry(key).or_default() += 1;
            }
        }
        ranked(counts)
            .into_iter()
            .map(|((cause, construct), functions)| Construct {
                cause,
                construct: construct.to_string(),
                functions,
            })
            .collect()
    }

    /// The totals and the queue as a register states them: the `[lift]`
    /// table, then one `[[construct]]` row per row of the queue. A register
    /// file and a projection's `frontier.toml` both carry this text.
    pub fn queue_text(&self) -> String {
        let mut out = format!("\n[{LIFT}]\n");
        for (key, count) in self.totals().fields() {
            out.push_str(&format!("{key} = {count}\n"));
        }
        for row in self.queue() {
            out.push('\n');
            out.push_str(&queue_row(
                CONSTRUCT,
                &[(CONSTRUCT, &row.construct), (CAUSE, &row.cause)],
                UNIT,
                row.functions,
            ));
        }
        out
    }

    /// How many functions lifted.
    pub fn lifted(&self) -> usize {
        self.rows
            .values()
            .filter(|r| matches!(r.status, Status::Lifted(_)))
            .count()
    }

    /// How many functions did not lift yet: the observation of the measure
    /// whose requirement is that every function of product code lifts. The
    /// rows the two counts take part the register, so they sum to its
    /// length; each is counted, so neither is a difference that could
    /// underflow.
    pub fn unlifted(&self) -> usize {
        self.rows
            .values()
            .filter(|r| !matches!(r.status, Status::Lifted(_)))
            .count()
    }

    /// The honesty report: how many functions lifted, and every row that
    /// did not by its cause (a residue row of the MIR path by the MIR kind
    /// or obligation that stopped it), as comment lines.
    pub fn honesty(&self) -> String {
        let mut by: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        let (mut foreign, mut effect) = (0usize, 0usize);
        let mut mir = false;
        for row in self.rows.values() {
            match &row.status {
                Status::Lifted(_) => {}
                Status::Foreign(_) => foreign += 1,
                Status::Effect(_) => effect += 1,
                Status::Residue(reason, detail) => {
                    mir |= detail.starts_with("mir::");
                    let (cause, rest) = residue_cause(reason, detail);
                    *by.entry(cause).or_default().entry(rest).or_default() += 1;
                }
            }
        }
        let residue: usize = by.values().flat_map(BTreeMap::values).sum();
        let mut out = format!(
            "# Honesty report: {} of {} functions of product code are lifted; {residue}\n\
             # are residue, {foreign} foreign and {effect} effect rows.\n",
            self.lifted(),
            self.rows.len()
        );
        if residue > 0 {
            out.push_str("# Residue by cause:\n");
        }
        for (cause, details) in &by {
            let n: usize = details.values().sum();
            out.push_str(&format!("#   {cause}: {n}\n"));
            if cause.starts_with("mir::") {
                for (detail, k) in details.iter().filter(|(d, _)| !d.is_empty()) {
                    out.push_str(&format!("#     {k} {detail}\n"));
                }
            }
        }
        if mir {
            out.push_str(&crate::mir::coverage::report());
        }
        out.push('\n');
        out
    }

    /// The register file: the totals and the queue, then one row per
    /// function, by table and then function, without spans, so that only a
    /// function's own source moves its row.
    pub fn to_text(&self) -> String {
        let mut out = format!(
            "# The lift register, written by uorust: every function of product code by\n\
             # the κ of its question (crate path and normalized tokens). A lifted κ is a\n\
             # proved claim; a later projection that decides it unlifted is refused.\n\
             # [{LIFT}] counts the functions by table, and each [[{CONSTRUCT}]] row is what\n\
             # stops the {UNIT} it counts (the detail their rows state, under its\n\
             # cause), the one that stops the most first: the work queue.\n\n\
             spec = \"{SCHEMA}\"\n"
        );
        out.insert_str(out.find("spec = ").unwrap_or(0), &self.honesty());
        out.push_str(&self.queue_text());
        let mut rows: Vec<(&String, &Row)> = self.rows.iter().collect();
        rows.sort_by(|a, b| a.1.function.cmp(&b.1.function));
        for table in ["residue", "foreign", "effect", "lifted"] {
            for (kappa, row) in &rows {
                if row.status.table() != table {
                    continue;
                }
                out.push_str(&format!(
                    "\n[[{table}]]\nkappa = \"{kappa}\"\nfunction = {}\n",
                    quoted(&row.function)
                ));
                match &row.status {
                    Status::Lifted(d) => out.push_str(&format!("definition = {}\n", quoted(d))),
                    Status::Residue(reason, detail) => out.push_str(&format!(
                        "reason = {}\ndetail = {}\n",
                        quoted(reason),
                        quoted(detail)
                    )),
                    Status::Foreign(detail) => {
                        out.push_str(&format!("detail = {}\n", quoted(detail)));
                    }
                    Status::Effect(effect) => {
                        out.push_str(&format!("effect = {}\n", quoted(effect)));
                    }
                }
                if let Some(divergence) = &row.divergence {
                    out.push_str(&format!("divergence = {}\n", quoted(divergence)));
                }
            }
        }
        out
    }
}

/// The κ the incumbent records as lifted that the candidate decides
/// otherwise, each with both decisions: empty exactly when the candidate is
/// admitted. A κ the candidate does not hold is a question no longer asked
/// (its function changed or went away), not a regression.
pub fn admit(incumbent: &Register, candidate: &Register) -> Vec<String> {
    incumbent
        .rows
        .iter()
        .filter(|(_, row)| matches!(row.status, Status::Lifted(_)))
        .filter_map(|(kappa, row)| {
            let now = candidate.rows.get(kappa)?;
            match &now.status {
                // A κ lifted exactly stays exact: a divergence is a weaker
                // claim.
                Status::Lifted(_) if row.divergence.is_none() && now.divergence.is_some() => {
                    Some(format!(
                        "{} (κ {kappa}) was lifted exactly and now diverges: {}",
                        row.function,
                        now.divergence.as_deref().unwrap_or_default()
                    ))
                }
                Status::Lifted(_) => None,
                other => Some(format!(
                    "{} (κ {kappa}) was lifted and is now {}",
                    row.function,
                    describe(other)
                )),
            }
        })
        .collect()
}

fn describe(status: &Status) -> String {
    match status {
        Status::Lifted(d) => format!("lifted to {d}"),
        Status::Residue(_, detail) => format!("residue: {detail}"),
        Status::Foreign(detail) => format!("foreign: {detail}"),
        Status::Effect(effect) => format!("an effect: {effect}"),
    }
}

fn quoted(text: &str) -> String {
    toml::Value::String(text.to_string()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(function: &str, status: Status) -> (String, Row) {
        (
            kappa(function, "fn f () {}"),
            Row {
                function: function.into(),
                status,
                divergence: None,
            },
        )
    }

    #[test]
    fn a_lifted_kappa_that_turns_unlifted_is_refused() {
        let lifted = row("c::f", Status::Lifted("Behavior::f".into()));
        let other = row("c::g", Status::Residue("form".into(), "a loop".into()));
        let incumbent = Register {
            rows: [lifted.clone(), other.clone()].into_iter().collect(),
        };
        // Still lifted, and the residue row lifted too: admitted.
        let better = Register {
            rows: [
                lifted.clone(),
                (
                    other.0.clone(),
                    Row {
                        function: "c::g".into(),
                        status: Status::Lifted("Behavior::g".into()),
                        divergence: None,
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };
        assert!(admit(&incumbent, &better).is_empty());
        // The lifted function's source changed: a new question, admitted.
        let changed = Register {
            rows: [other.clone()].into_iter().collect(),
        };
        assert!(admit(&incumbent, &changed).is_empty());
        // The same κ decided as residue: refused, naming it.
        let worse = Register {
            rows: [
                (
                    lifted.0.clone(),
                    Row {
                        function: "c::f".into(),
                        status: Status::Residue("call".into(), "calls g".into()),
                        divergence: None,
                    },
                ),
                other,
            ]
            .into_iter()
            .collect(),
        };
        let refused = admit(&incumbent, &worse);
        assert_eq!(refused.len(), 1);
        assert!(refused[0].contains(&lifted.0), "{refused:?}");
        // The same κ lifted, but now with a divergence: a weaker claim,
        // refused; a κ that diverged and now lifts exactly is admitted.
        let diverging = |r: &(String, Row)| {
            let mut r = r.clone();
            r.1.divergence = Some("no Unicode case table".into());
            r
        };
        let weaker = Register {
            rows: [diverging(&lifted)].into_iter().collect(),
        };
        let refused = admit(&incumbent, &weaker);
        assert_eq!(refused.len(), 1);
        assert!(refused[0].contains("now diverges"), "{refused:?}");
        assert!(admit(&weaker, &incumbent).is_empty());
    }

    #[test]
    fn the_register_round_trips_and_rows_are_typed() {
        let register = Register {
            rows: [
                row("c::f", Status::Lifted("Behavior::f".into())),
                {
                    let (k, mut r) = row("c::e", Status::Lifted("Behavior::e".into()));
                    r.divergence = Some("lowers ASCII letters only".into());
                    (k, r)
                },
                row("c::g", Status::Effect("Fs effect: std::fs::read".into())),
                row(
                    "c::h",
                    Status::Foreign(
                        "takes or returns syn::Expr, a type of the syn crate with no LexLean model"
                            .into(),
                    ),
                ),
                row("c::i", Status::Residue("form".into(), "a \"loop\"".into())),
            ]
            .into_iter()
            .collect(),
        };
        let at = Path::new("frontier.toml");
        assert_eq!(
            Register::parse(at, &register.to_text()),
            Ok(register.clone())
        );
        assert_eq!((register.lifted(), register.unlifted()), (2, 3));
        assert_eq!(register.lifted() + register.unlifted(), register.rows.len());
        let stray = "[[residue]]\nkappa = \"00\"\nfunction = \"c::f\"\nreason = \"form\"\ndetail = \"d\"\ndivergence = \"x\"\n";
        assert!(Register::parse(at, stray).is_err());
        let untyped = "[[residue]]\nkappa = \"00\"\nfunction = \"c::f\"\n";
        assert!(Register::parse(at, untyped).is_err());
        let twice = format!(
            "{}\n[[lifted]]\nkappa = \"{}\"\nfunction = \"c::f\"\ndefinition = \"D\"\n",
            register.to_text(),
            kappa("c::z", "")
        );
        assert!(
            Register::parse(at, &twice).is_err_and(|e| e.to_string().contains("more than one row"))
        );
    }

    #[test]
    fn the_preimage_is_the_models_fixture() {
        // Kappa::kappaPreimageFixture, which Kappa::theFixtureIsThePreimage
        // proves is the model's preimage of these two inputs.
        let model = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../app/src/Kappa.lex.tex"),
        )
        .expect("Kappa.lex.tex");
        let at = model
            .find("name: kappaPreimageFixture,")
            .expect("the fixture");
        let body = &model[at..model[at..].find("\n}").map_or(model.len(), |e| at + e)];
        let mut bytes = Vec::new();
        for piece in body.split("(ctor Symbol.").skip(1) {
            if piece.starts_with("newline") {
                bytes.push(b'\n');
            } else {
                let n = piece
                    .trim_start_matches("byte ")
                    .split(')')
                    .next()
                    .expect("a byte");
                bytes.push(n.trim().parse::<u8>().expect("a byte"));
            }
        }
        assert_eq!(kappa_preimage("c::f", "fn f () {}"), bytes);
    }

    #[test]
    #[should_panic(expected = "newline")]
    fn a_path_with_a_newline_is_refused() {
        let _ = kappa_preimage("c::f\ng", "x");
    }

    #[test]
    fn kappa_depends_on_path_and_tokens() {
        assert_ne!(kappa("c::f", "fn f () {}"), kappa("c::g", "fn f () {}"));
        assert_ne!(kappa("c::f", "fn f () {}"), kappa("c::f", "fn f () { 1 }"));
        assert_eq!(kappa("c::f", "x").len(), 64);
    }

    // The register as the work queue: its totals and its ranked
    // `[[construct]]` rows, written by the tool as data.

    /// Two lifted functions, and eight that are not: three stopped by one
    /// type, two by one MIR kind and detail, one by another detail of that
    /// kind, one foreign and one effect row.
    fn queued() -> Register {
        let foreign = "takes or returns syn::Expr, a type of the syn crate with no LexLean model";
        let residue = |reason: &str, detail: &str| Status::Residue(reason.into(), detail.into());
        Register {
            rows: [
                row("c::a", Status::Lifted("Behavior::a".into())),
                row("c::b", Status::Lifted("Behavior::b".into())),
                row("c::t1", residue("type", "type outside the lift: Wide")),
                row("c::t2", residue("type", "type outside the lift: Wide")),
                row("c::t3", residue("type", "type outside the lift: Wide")),
                row("c::m1", residue("mir", "mir::Rvalue::Ref: Mut")),
                row("c::m2", residue("mir", "mir::Rvalue::Ref: Mut")),
                row("c::m3", residue("mir", "mir::Rvalue::Ref: Shared")),
                row("c::x", Status::Foreign(foreign.into())),
                row("c::e", Status::Effect("Fs effect: std::fs::read".into())),
            ]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn the_register_carries_its_totals_and_its_ranked_queue() {
        let register = queued();
        let totals = register.totals();
        assert_eq!(
            totals,
            Totals {
                functions: 10,
                lifted: 2,
                residue: 6,
                foreign: 1,
                effect: 1
            }
        );
        // The tables part the functions.
        assert_eq!(
            totals.lifted + totals.residue + totals.foreign + totals.effect,
            totals.functions
        );
        // Ranked: what blocks the most first, then by cause and construct.
        let queue = register.queue();
        let listed: Vec<(&str, &str, usize)> = queue
            .iter()
            .map(|c| (c.cause.as_str(), c.construct.as_str(), c.functions))
            .collect();
        assert_eq!(
            listed,
            [
                ("type", "type outside the lift: Wide", 3),
                ("mir::Rvalue::Ref", "mir::Rvalue::Ref: Mut", 2),
                ("effect", "Fs effect: std::fs::read", 1),
                (
                    "foreign",
                    "takes or returns syn::Expr, a type of the syn crate with no LexLean model",
                    1
                ),
                ("mir::Rvalue::Ref", "mir::Rvalue::Ref: Shared", 1),
            ]
        );
        // Every function that is not lifted is blocked by exactly one row.
        assert_eq!(
            queue.iter().map(|c| c.functions).sum::<usize>(),
            register.unlifted()
        );
        // The file states both as data, in that order, and reads back.
        let text = register.to_text();
        let doc: toml::Value = text.parse().expect("the register is TOML");
        for (key, count) in totals.fields() {
            assert_eq!(
                doc[LIFT].get(key).and_then(toml::Value::as_integer),
                i64::try_from(count).ok(),
                "{key}"
            );
        }
        let written: Vec<(&str, &str, i64)> = doc[CONSTRUCT]
            .as_array()
            .expect("the queue")
            .iter()
            .map(|r| {
                (
                    r[CAUSE].as_str().expect("a cause"),
                    r[CONSTRUCT].as_str().expect("a construct"),
                    r[UNIT].as_integer().expect("a count"),
                )
            })
            .collect();
        assert_eq!(written.len(), listed.len());
        for (w, l) in written.iter().zip(&listed) {
            assert_eq!((w.0, w.1, usize::try_from(w.2).ok()), (l.0, l.1, Some(l.2)));
        }
        // A construct is the detail the rows it blocks state.
        let blocked = doc["residue"]
            .as_array()
            .expect("residue rows")
            .iter()
            .filter(|r| r["detail"].as_str() == Some(listed[0].1))
            .count();
        assert_eq!(blocked, listed[0].2);
        assert_eq!(
            Register::parse(Path::new("register.toml"), &text),
            Ok(register)
        );
    }

    /// The totals and the queue are derived data: a register that states
    /// them otherwise than its rows give was not written from its rows.
    #[test]
    fn a_register_whose_queue_is_not_its_rows_is_refused() {
        let at = Path::new("register.toml");
        let text = queued().to_text();
        let refused = |planted: String| match Register::parse(at, &planted) {
            Err(UorustError::Malformed { reason, .. }) => reason,
            other => panic!("expected Malformed, got {other:?}"),
        };
        assert!(text.contains("lifted = 2\n") && text.contains("functions = 3\n"));
        // A total raised by hand.
        let reason = refused(text.replace("lifted = 2\n", "lifted = 3\n"));
        assert!(
            reason.contains("lifted = 3") && reason.contains("hold 2"),
            "{reason}"
        );
        // A total left out.
        let reason = refused(text.replace("foreign = 1\n", ""));
        assert!(reason.contains("foreign = nothing"), "{reason}");
        // A count of the queue lowered by hand.
        let reason = refused(text.replace("functions = 3\n", "functions = 2\n"));
        assert!(reason.contains("row 1"), "{reason}");
        // The queue cut off after its totals.
        let cut = format!(
            "{}{}",
            &text[..text.find("\n[[construct]]").expect("a queue")],
            &text[text.find("\n[[residue]]").expect("rows")..]
        );
        let reason = refused(cut);
        assert!(reason.contains("0 [[construct]] rows"), "{reason}");
        // Two rows out of rank.
        let rows: Vec<&str> = text.split("\n[[construct]]").collect();
        let mut swapped = rows.clone();
        swapped.swap(1, 2);
        assert_ne!(rows, swapped);
        let reason = refused(swapped.join("\n[[construct]]"));
        assert!(reason.contains("row 1"), "{reason}");
        // A queue without its totals is held to the rows as well.
        let headless = format!(
            "{}{}",
            &text[..text.find("\n[lift]\n").expect("the totals")],
            &text[text.find("\n[[construct]]").expect("a queue")..]
        );
        let doc: toml::Value = headless.parse().expect("TOML");
        assert!(doc.get(LIFT).is_none() && doc.get(CONSTRUCT).is_some());
        assert!(Register::parse(at, &headless).is_ok());
        let reason = refused(headless.replace("functions = 3\n", "functions = 4\n"));
        assert!(reason.contains("row 1"), "{reason}");
    }

    /// A register as the tool wrote it before it carried its totals and its
    /// queue: its rows are read, admitted against and checked as they were.
    #[test]
    fn an_older_register_is_still_read_admitted_and_checked() {
        let (kf, kg) = (kappa("c::f", "fn f () {}"), kappa("c::g", "fn f () {}"));
        let older = format!(
            "# The lift register, written by uorust: every function of product code by\n\
             # the κ of its question (crate path and normalized tokens). A lifted κ is a\n\
             # proved claim; a later projection that decides it unlifted is refused.\n\n\
             # Honesty report: 1 of 2 functions of product code are lifted; 1\n\
             # are residue, 0 foreign and 0 effect rows.\n\
             # Residue by cause:\n\
             #   form: 1\n\n\
             spec = \"uorust/register/1\"\n\n\
             [[residue]]\nkappa = \"{kg}\"\nfunction = \"c::g\"\nreason = \"form\"\ndetail = \"a loop\"\n\n\
             [[lifted]]\nkappa = \"{kf}\"\nfunction = \"c::f\"\ndefinition = \"Behavior::f\"\n"
        );
        let register = Register {
            rows: [
                row("c::f", Status::Lifted("Behavior::f".into())),
                row("c::g", Status::Residue("form".into(), "a loop".into())),
            ]
            .into_iter()
            .collect(),
        };
        let at = Path::new("register.toml");
        assert_eq!(Register::parse(at, &older), Ok(register.clone()));
        // What this tool writes for the same rows differs only by the queue.
        let now = register.to_text();
        assert!(now.contains("\n[lift]\n") && !older.contains("\n[lift]\n"));
        assert_eq!(Register::parse(at, &now), Ok(register.clone()));
        // Admission against the older register: the same rows are admitted,
        // and its lifted κ decided otherwise is refused.
        let worse = Register {
            rows: [
                row("c::f", Status::Residue("call".into(), "calls g".into())),
                row("c::g", Status::Residue("form".into(), "a loop".into())),
            ]
            .into_iter()
            .collect(),
        };
        let incumbent = Register::parse(at, &older).expect("an older register");
        assert!(admit(&incumbent, &register).is_empty());
        assert_eq!(admit(&incumbent, &worse).len(), 1);
        // Through the file: `check_register` reads the older file, admits
        // against it, and reports that it is not what this run writes;
        // `write_register` then writes the queue beside the same rows.
        let dir =
            std::env::temp_dir().join(format!("uorust-older-register-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        let file = dir.join("register.toml");
        let projection = |r: &Register| -> BTreeMap<String, Vec<u8>> {
            [("frontier.toml".to_string(), r.to_text().into_bytes())]
                .into_iter()
                .collect()
        };
        std::fs::write(&file, &older).expect("write");
        assert_eq!(crate::admit(&file, &projection(&register)), Ok(()));
        assert!(matches!(
            crate::check_register(&file, &projection(&register)),
            Err(UorustError::Stale(_))
        ));
        assert!(matches!(
            crate::check_register(&file, &projection(&worse)),
            Err(UorustError::Regressed(_))
        ));
        assert!(matches!(
            crate::write_register(&file, &projection(&worse)),
            Err(UorustError::Regressed(_))
        ));
        assert_eq!(std::fs::read_to_string(&file).expect("kept"), older);
        crate::write_register(&file, &projection(&register)).expect("the register is rewritten");
        assert_eq!(std::fs::read_to_string(&file).expect("written"), now);
        assert_eq!(crate::check_register(&file, &projection(&register)), Ok(()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The order and the row shape of a queue are the core register's: its
    /// committed `[[construct]]`, `[[pending]]` and `[[cause]]` rows are in
    /// [`ranked`] order and are, byte for byte, [`queue_row`]'s.
    #[test]
    fn a_queue_is_ranked_and_written_as_the_core_registers() {
        let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("core/frontier.toml");
        let text = std::fs::read_to_string(&file).expect("the core register");
        let doc: toml::Value = text.parse().expect("the core register is TOML");
        let mut rows = 0;
        for (table, key) in [(CONSTRUCT, CONSTRUCT), ("pending", "shape"), (CAUSE, CAUSE)] {
            let committed: Vec<(String, usize)> = doc
                .get(table)
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .map(|r| {
                    let n = r["bodies"].as_integer().expect("a count");
                    (
                        r[key].as_str().expect("a name").to_string(),
                        usize::try_from(n).expect("a count"),
                    )
                })
                .collect();
            let counts: BTreeMap<String, usize> = committed.iter().cloned().collect();
            assert_eq!(counts.len(), committed.len(), "{table}: a name twice");
            assert_eq!(ranked(counts), committed, "{table}");
            let written: Vec<String> = committed
                .iter()
                .map(|(name, n)| queue_row(table, &[(key, name)], "bodies", *n))
                .collect();
            assert!(text.contains(&written.join("\n")), "{table}");
            rows += committed.len();
        }
        assert!(rows > 0, "the core register has no queue to compare with");
        // The workspace register's rows are that shape with their cause.
        assert_eq!(
            queue_row(
                CONSTRUCT,
                &[(CONSTRUCT, "a \"loop\""), (CAUSE, "form")],
                UNIT,
                2
            ),
            "[[construct]]\nconstruct = 'a \"loop\"'\ncause = \"form\"\nfunctions = 2\n"
        );
    }
}
