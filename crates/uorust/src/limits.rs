//! The limits a projection's LexLean and PrismPM projects state.
//!
//! Neither SDK tool has a default of its own: LexLean's project schema
//! requires every limit ("Every listed positive integer is required. There
//! are no hidden compiler defaults", SPEC, project fields), and PrismPM's
//! project reader requires each of its three (`config.rs`, `required`). So
//! the limits are a fact a projection is given, not one the tool may type.
//!
//! - `--limits FILE` is the workspace's own statement: a TOML file with a
//!   `[lexlean]` and a `[prismpm]` table, each holding exactly the keys the
//!   SDK's project schema lists, each a positive integer. A missing key, a
//!   key the schema does not list, or a value that is not a positive integer
//!   is refused with the key named. The SDK's own upper bounds (PrismPM's
//!   `config.rs`, LexLean's checks) are not copied here: the SDK's checks of
//!   the written project (`prismpm check`, `lexlean verify`) are their one
//!   statement, and a value beyond them is refused there (`prismpm check`
//!   answers `PP1003 project limits are outside fixed bounds`, without
//!   naming which key).
//! - Without it, the projection takes the limits of the SDK's own scaffold
//!   (the `text-request` template of the `prism-code` package, byte copies in
//!   `assets/limits/`), and says so.
//!
//! Either way the frontier's `[limits]` table states which it took and the
//! SHA-256 of the bytes it read, never a path: a declared file is an input
//! of the projection, so its content is part of what the projection is a
//! function of, and its location is not.

use std::path::Path;

use crate::UorustError;

/// The SDK scaffold's LexLean project file (`prism-code` v0.5.0,
/// `templates/text-request/lexlean.toml`, SHA-256 `9380aa15…`), byte for byte.
const SDK_LEXLEAN: &str = include_str!("../assets/limits/lexlean.toml");

/// The SDK scaffold's PrismPM project file (`prism-code` v0.5.0,
/// `templates/text-request/prismpm.toml`, SHA-256 `873c23f7…`), byte for byte.
const SDK_PRISMPM: &str = include_str!("../assets/limits/prismpm.toml");

/// Where a projection's limits came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// The workspace's `--limits` file, by the SHA-256 of its bytes.
    Declared(String),
    /// The SDK scaffold's project files, by the SHA-256 of each.
    Sdk,
}

/// The `[limits]` tables of a projection's two project files, each in the
/// order the SDK's scaffold lists its keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Limits {
    pub(crate) lexlean: Vec<(String, u64)>,
    pub(crate) prismpm: Vec<(String, u64)>,
    pub(crate) source: Source,
}

/// The `[limits]` table of one of the SDK scaffold's files, in its order.
/// The scaffold is a fixed asset that a test reads whole, so a table it
/// lacks is a defect of the asset, not of an input.
fn scaffold(text: &str) -> Vec<(String, u64)> {
    let value: toml::Value = text.parse().unwrap_or(toml::Value::Boolean(false));
    let mut keys: Vec<(usize, String, u64)> = value
        .get("limits")
        .and_then(toml::Value::as_table)
        .map(|table| {
            table
                .iter()
                .filter_map(|(key, value)| {
                    let at = text.find(&format!("\n{key} = "))?;
                    Some((at, key.clone(), u64::try_from(value.as_integer()?).ok()?))
                })
                .collect()
        })
        .unwrap_or_default();
    keys.sort();
    keys.into_iter()
        .map(|(_, key, value)| (key, value))
        .collect()
}

impl Limits {
    /// The SDK scaffold's limits.
    pub fn sdk() -> Self {
        Self {
            lexlean: scaffold(SDK_LEXLEAN),
            prismpm: scaffold(SDK_PRISMPM),
            source: Source::Sdk,
        }
    }

    /// The limits a workspace declares in `path`: each of the two tables
    /// holds exactly the keys the scaffold lists, each a positive integer.
    pub fn declared(path: &Path) -> Result<Self, UorustError> {
        let bytes = std::fs::read(path).map_err(|e| UorustError::Io {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;
        let refused = |reason: String| UorustError::Limits {
            path: path.to_path_buf(),
            reason,
        };
        let value: toml::Value = String::from_utf8_lossy(&bytes)
            .parse()
            .map_err(|e| refused(format!("{e}")))?;
        let sdk = Self::sdk();
        let lexlean = table(&value, "lexlean", &sdk.lexlean, path)?;
        let prismpm = table(&value, "prismpm", &sdk.prismpm, path)?;
        if let Some(other) = value
            .as_table()
            .and_then(|top| top.keys().find(|k| *k != "lexlean" && *k != "prismpm"))
        {
            return Err(refused(format!(
                "it has a table `{other}`; a limits file has only [lexlean] and [prismpm]"
            )));
        }
        Ok(Self {
            lexlean,
            prismpm,
            source: Source::Declared(crate::project::sha(&bytes)),
        })
    }

    /// The value of the LexLean limit `key`.
    pub fn lexlean(&self, key: &str) -> Option<u64> {
        self.lexlean.iter().find(|(k, _)| k == key).map(|(_, v)| *v)
    }

    /// The largest LexLean module the projection may write.
    pub fn max_file_bytes(&self) -> usize {
        self.lexlean("max_file_bytes")
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or(usize::MAX)
    }

    /// The `[limits]` table of a project file, `key = value` per line.
    pub(crate) fn table_text(rows: &[(String, u64)]) -> String {
        let mut out = String::from("[limits]\n");
        for (key, value) in rows {
            out.push_str(&format!("{key} = {value}\n"));
        }
        out
    }

    /// The frontier's `[limits]` table: where the limits came from, by
    /// digest, never by path.
    pub(crate) fn frontier_text(&self) -> String {
        match &self.source {
            Source::Declared(sha) => format!(
                "\n[limits]\nsource = \"declared\"\nsha256 = \"{sha}\"\n"
            ),
            Source::Sdk => format!(
                "\n[limits]\nsource = \"sdk-scaffold\"\nlexlean_sha256 = \"{}\"\nprismpm_sha256 = \"{}\"\n",
                crate::project::sha(SDK_LEXLEAN.as_bytes()),
                crate::project::sha(SDK_PRISMPM.as_bytes())
            ),
        }
    }
}

/// The table `name` of a limits file at `path`, checked against the
/// scaffold's keys and returned in the scaffold's order. A refusal names the
/// key.
fn table(
    value: &toml::Value,
    name: &str,
    keys: &[(String, u64)],
    path: &Path,
) -> Result<Vec<(String, u64)>, UorustError> {
    let refused = |reason: String| UorustError::Limits {
        path: path.to_path_buf(),
        reason,
    };
    let Some(table) = value.get(name).and_then(toml::Value::as_table) else {
        return Err(refused(format!("it states no [{name}] table")));
    };
    if let Some(key) = table.keys().find(|k| !keys.iter().any(|(s, _)| s == *k)) {
        return Err(refused(format!(
            "[{name}] states `{key}`, which the SDK's project schema does not list"
        )));
    }
    keys.iter()
        .map(|(key, _)| {
            let stated = table.get(key).ok_or_else(|| {
                refused(format!(
                    "[{name}] states no `{key}`, which the SDK requires"
                ))
            })?;
            stated
                .as_integer()
                .and_then(|n| u64::try_from(n).ok())
                .filter(|n| *n > 0)
                .map(|n| (key.clone(), n))
                .ok_or_else(|| {
                    refused(format!(
                        "[{name}] `{key}` is {stated}, not a positive integer"
                    ))
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Limits, Source, SDK_LEXLEAN, SDK_PRISMPM};
    use crate::UorustError;

    /// The lines of a scaffold file's `[limits]` table.
    fn limits_lines(text: &str) -> usize {
        text.split("[limits]\n").nth(1).map_or(0, |rest| {
            rest.lines().take_while(|l| l.contains(" = ")).count()
        })
    }

    /// A limits file stating the scaffold's values, with `edit` applied.
    fn limits_file(tag: &str, edit: impl Fn(String) -> String) -> std::path::PathBuf {
        let sdk = Limits::sdk();
        let text = format!(
            "{}\n{}",
            Limits::table_text(&sdk.lexlean).replace("[limits]", "[lexlean]"),
            Limits::table_text(&sdk.prismpm).replace("[limits]", "[prismpm]")
        );
        let path =
            std::env::temp_dir().join(format!("uorust-limits-{tag}-{}.toml", std::process::id()));
        std::fs::write(&path, edit(text)).expect("a limits file");
        path
    }

    fn limits_refusal(tag: &str, edit: impl Fn(String) -> String) -> String {
        let path = limits_file(tag, edit);
        let got = Limits::declared(&path);
        let _ = std::fs::remove_file(&path);
        match got {
            Err(UorustError::Limits { reason, .. }) => reason,
            other => panic!("not refused: {other:?}"),
        }
    }

    #[test]
    fn limits_the_scaffold_is_read_whole_and_in_its_order() {
        let sdk = Limits::sdk();
        assert_eq!(sdk.lexlean.len(), limits_lines(SDK_LEXLEAN));
        assert_eq!(sdk.prismpm.len(), limits_lines(SDK_PRISMPM));
        assert!(!sdk.lexlean.is_empty() && !sdk.prismpm.is_empty());
        let table = Limits::table_text(&sdk.lexlean);
        assert!(SDK_LEXLEAN.ends_with(&table), "{table}");
        assert!(SDK_PRISMPM.ends_with(&Limits::table_text(&sdk.prismpm)));
        assert_eq!(
            crate::project::MAX_FILE_BYTES,
            sdk.max_file_bytes(),
            "the tool's own default is the scaffold's"
        );
        assert_eq!(sdk.source, Source::Sdk);
    }

    #[test]
    fn limits_a_declared_file_is_taken_by_its_digest_in_the_scaffold_order() {
        let path = limits_file("declared", |text| {
            // Stated in another order, with one value changed.
            let (lexlean, prismpm) = text.split_once("\n[prismpm]").expect("two tables");
            let mut lines: Vec<&str> = lexlean.lines().skip(1).collect();
            lines.reverse();
            format!("[prismpm]{prismpm}\n[lexlean]\n{}\n", lines.join("\n"))
                .replace("max_ir_nodes = ", "max_ir_nodes = 7")
        });
        let bytes = std::fs::read(&path).expect("the file");
        let declared = Limits::declared(&path).expect("taken");
        let _ = std::fs::remove_file(&path);
        let sdk = Limits::sdk();
        let keys = |rows: &[(String, u64)]| rows.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>();
        assert_eq!(keys(&declared.lexlean), keys(&sdk.lexlean));
        assert_eq!(declared.prismpm, sdk.prismpm);
        let nodes = |l: &Limits| l.lexlean("max_ir_nodes").expect("a limit");
        assert_eq!(
            nodes(&declared).to_string(),
            format!("7{}", nodes(&sdk)),
            "the declared value, not the scaffold's"
        );
        assert_eq!(
            declared.source,
            Source::Declared(crate::project::sha(&bytes))
        );
        let stated = declared.frontier_text();
        assert!(stated.contains("source = \"declared\""), "{stated}");
        assert!(!stated.contains("uorust-limits"), "a path in: {stated}");
    }

    #[test]
    fn limits_a_file_the_schema_does_not_admit_is_refused_naming_the_key() {
        let missing = limits_refusal("missing", |t| t.replace("max_scope_depth = ", "# "));
        assert!(
            missing.contains("[lexlean] states no `max_scope_depth`"),
            "{missing}"
        );
        let unknown = limits_refusal("unknown", |t| t + "max_widgets = 3\n");
        assert!(
            unknown.contains("[prismpm] states `max_widgets`"),
            "{unknown}"
        );
        let zero = limits_refusal("zero", |t| {
            t.replace("max_entities = ", "max_entities = 0#")
        });
        assert!(
            zero.contains("`max_entities` is 0, not a positive integer"),
            "{zero}"
        );
        let negative = limits_refusal("negative", |t| {
            t.replace("max_diagnostics = ", "max_diagnostics = -")
        });
        assert!(negative.contains("not a positive integer"), "{negative}");
        let table = limits_refusal("table", |t| t.replace("[prismpm]", "[prism]"));
        assert!(table.contains("states no [prismpm] table"), "{table}");
        let extra = limits_refusal("extra", |t| t + "\n[lake]\nx = 1\n");
        assert!(extra.contains("a table `lake`"), "{extra}");
    }
}
