//! The verdict register: what the verifier decided for each translation.
//!
//! A translation is a candidate until `lexlean verify` has elaborated its
//! definitions and proved its theorems (the obligations of its asserts,
//! panics and arithmetic, and the decrease of each loop's measure). The
//! register keeps, by the κ of the Rust function and the SHA-256 of the
//! translation's text, whether it was proved, or refused and why, and the
//! exact axioms each declaration elaborated with as the verifier observed
//! them. A projection admits a translation only on a `proved` row of the same
//! digest, so the gate never meets an obligation that did not discharge, and
//! a translation that changes is a new question. `uorust prove` writes it.

use std::collections::BTreeMap;
use std::path::Path;

use crate::error::io;
use crate::UorustError;

/// The register's schema.
pub const SCHEMA: &str = "uorust/verdicts/1";

/// What the verifier decided.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Every declaration elaborated and every theorem was proved.
    Proved,
    /// The verifier refused it: the diagnostic code and message.
    Refused(String),
}

/// One translation's verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    /// The Rust function's path.
    pub function: String,
    /// The SHA-256 of the translation's text.
    pub digest: String,
    /// Optional rooted model binding, learned only from an exact legacy
    /// binding or a new actual SDK decision. It is not an SDK semantic ID.
    pub model_context_sha256: Option<String>,
    /// The decision.
    pub status: Decision,
    /// The axioms each declaration elaborated with, where the verifier
    /// reported them.
    pub axioms: BTreeMap<String, Vec<String>>,
}

/// Every verdict, by the κ of its function.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Verdicts {
    /// Recorded decisions by κ, including currently unselected functions.
    /// Admission always matches the translation digest; the frontier, not
    /// this historical cache's row count, states current lifting eligibility.
    pub rows: BTreeMap<String, Verdict>,
    /// Intact decisions for other input contexts of the same questions.
    /// These are not additional lifted functions. Admission still requires
    /// each record's original digest or rooted dependency binding.
    pub historical: BTreeMap<String, Vec<Verdict>>,
}

impl Verdict {
    /// Reuse an intact decision at the same rooted model; legacy records
    /// without that observation still require their original full binding.
    pub(crate) fn matches(&self, digest: &str, model_context: Option<&str>) -> bool {
        match (self.model_context_sha256.as_deref(), model_context) {
            (Some(original), Some(current)) => original == current,
            _ => self.digest == digest,
        }
    }
}

impl Verdicts {
    /// Find a decision at the current exact input binding, including an
    /// earlier context. The matching rule is identical for both stores.
    pub(crate) fn matching(
        &self,
        kappa: &str,
        digest: &str,
        context: Option<&str>,
    ) -> Option<&Verdict> {
        self.rows
            .get(kappa)
            .filter(|v| v.matches(digest, context))
            .or_else(|| {
                self.historical
                    .get(kappa)?
                    .iter()
                    .rev()
                    .find(|v| v.matches(digest, context))
            })
    }

    /// Retain current decisions while preserving every displaced observation.
    pub(crate) fn retain(&mut self, mut keep: impl FnMut(&String, &Verdict) -> bool) {
        let historical = &mut self.historical;
        self.rows.retain(|key, value| {
            if keep(key, value) {
                return true;
            }
            let prior = historical.entry(key.clone()).or_default();
            if !prior.contains(value) {
                prior.push(value.clone());
            }
            false
        });
    }

    /// Import original records without replacing the current decisions or
    /// manufacturing observations. Unmatched records confer no admission.
    pub fn absorb(&mut self, other: &Self) {
        for (key, value) in other.rows.iter().chain(
            other
                .historical
                .iter()
                .flat_map(|(k, vs)| vs.iter().map(move |v| (k, v))),
        ) {
            if self.rows.get(key) == Some(value) {
                continue;
            }
            let prior = self.historical.entry(key.clone()).or_default();
            if !prior.contains(value) {
                prior.push(value.clone());
            }
        }
    }

    /// Merge existing verifier-written registers into an input cache. Every
    /// source is parsed before the target is atomically replaced.
    pub fn import_files(target: &Path, sources: &[std::path::PathBuf]) -> Result<(), UorustError> {
        if sources.is_empty() {
            return Ok(());
        }
        let mut combined = Self::read(target)?;
        for source in sources {
            let text = std::fs::read_to_string(source).map_err(|e| io(source, e))?;
            combined.absorb(&Self::parse(source, &text)?);
        }
        if let Some(parent) = target.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
        }
        let temporary = target.with_extension(format!("{}.importing", std::process::id()));
        std::fs::write(&temporary, combined.to_text()).map_err(|e| io(&temporary, e))?;
        std::fs::rename(&temporary, target).map_err(|e| io(target, e))
    }

    /// Read the register at `path`; an absent file is an empty register.
    pub fn read(path: &Path) -> Result<Verdicts, UorustError> {
        if !path.is_file() {
            return Ok(Verdicts::default());
        }
        let text = std::fs::read_to_string(path).map_err(|e| io(path, e))?;
        Self::parse(path, &text)
    }

    /// Parse a register's text.
    pub fn parse(path: &Path, text: &str) -> Result<Verdicts, UorustError> {
        let bad = |reason: String| UorustError::Malformed {
            path: path.to_path_buf(),
            reason,
        };
        let doc: toml::Value = text.parse().map_err(|e| bad(format!("{e}")))?;
        let mut out = Verdicts::default();
        for (table, historical) in [("verdict", false), ("prior_verdict", true)] {
            let rows = doc
                .get(table)
                .map(|value| {
                    value
                        .as_array()
                        .ok_or_else(|| bad(format!("{table} is not an array")))
                })
                .transpose()?;
            for row in rows.into_iter().flatten() {
                let field = |name: &str| {
                    row.get(name)
                        .and_then(toml::Value::as_str)
                        .map(str::to_string)
                        .ok_or_else(|| bad(format!("a verdict without {name}")))
                };
                let status = match field("status")?.as_str() {
                    "proved" => Decision::Proved,
                    "refused" => Decision::Refused(field("cause")?),
                    other => return Err(bad(format!("a verdict of status {other}"))),
                };
                let model_context_sha256 = row
                    .get("model_context_sha256")
                    .map(|v| {
                        let value = v
                            .as_str()
                            .ok_or_else(|| bad("a model context binding is not a string".into()))?;
                        if value.len() != 64
                            || !value
                                .bytes()
                                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                        {
                            return Err(bad("a model context binding is not a SHA-256".into()));
                        }
                        Ok(value.to_string())
                    })
                    .transpose()?;
                let mut axioms = BTreeMap::new();
                if let Some(table) = row
                    .get("axioms")
                    .map(|value| {
                        value
                            .as_table()
                            .ok_or_else(|| bad("axioms are not a table".into()))
                    })
                    .transpose()?
                {
                    for (decl, list) in table {
                        let list = list
                            .as_array()
                            .ok_or_else(|| bad(format!("axioms of {decl} are not a list")))?;
                        axioms.insert(
                            decl.clone(),
                            list.iter()
                                .map(|v| {
                                    v.as_str().map(str::to_string).ok_or_else(|| {
                                        bad(format!("an axiom of {decl} is not a string"))
                                    })
                                })
                                .collect::<Result<Vec<String>, UorustError>>()?,
                        );
                    }
                }
                let key = field("kappa")?;
                let verdict = Verdict {
                    function: field("function")?,
                    digest: field("digest")?,
                    model_context_sha256,
                    status,
                    axioms,
                };
                if historical {
                    out.historical.entry(key).or_default().push(verdict);
                } else {
                    out.rows.insert(key, verdict);
                }
            }
        }
        Ok(out)
    }

    /// The register's text: rows by κ, the same bytes for the same rows.
    pub fn to_text(&self) -> String {
        let mut out = format!(
            "# The verdict register of the MIR path, written by `uorust prove`: for each\n\
             # translated function (by the κ of its question and the SHA-256 of its\n\
             # translation), whether `lexlean verify` proved every theorem of the\n\
             # translation, and the axioms each declaration elaborated with. A\n\
             # projection admits a translation only on a proved row of its digest.\n\n\
             spec = \"{SCHEMA}\"\n"
        );
        for (table, kappa, v) in self.rows.iter().map(|(k, v)| ("verdict", k, v)).chain(
            self.historical
                .iter()
                .flat_map(|(k, vs)| vs.iter().map(move |v| ("prior_verdict", k, v))),
        ) {
            out.push_str(&format!(
                "\n[[{table}]]\nkappa = {}\nfunction = {}\ndigest = {}\n",
                verdict_quoted(kappa),
                verdict_quoted(&v.function),
                verdict_quoted(&v.digest)
            ));
            if let Some(model) = &v.model_context_sha256 {
                out.push_str(&format!(
                    "model_context_sha256 = {}\n",
                    verdict_quoted(model)
                ));
            }
            match &v.status {
                Decision::Proved => out.push_str("status = \"proved\"\n"),
                Decision::Refused(cause) => {
                    out.push_str(&format!(
                        "status = \"refused\"\ncause = {}\n",
                        verdict_quoted(cause)
                    ));
                }
            }
            if !v.axioms.is_empty() {
                let parts: Vec<String> = v
                    .axioms
                    .iter()
                    .map(|(d, list)| {
                        let items: Vec<String> = list.iter().map(|a| verdict_quoted(a)).collect();
                        format!("{d} = [{}]", items.join(", "))
                    })
                    .collect();
                out.push_str(&format!("axioms = {{ {} }}\n", parts.join(", ")));
            }
        }
        out
    }
}

fn verdict_quoted(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_bindings_and_import_failures_preserve_records_ue_74() {
        let row = Verdict {
            function: "sample::query".into(),
            digest: "d".repeat(64),
            model_context_sha256: Some("a".repeat(64)),
            status: Decision::Refused("original diagnostic".into()),
            axioms: BTreeMap::new(),
        };
        let mut register = Verdicts::default();
        register.rows.insert("question".into(), row.clone());
        register.retain(|_, _| false);
        assert_eq!(register.historical["question"], vec![row.clone()]);
        assert_eq!(
            register.matching(
                "question",
                "unrelated global context",
                Some(&"a".repeat(64))
            ),
            Some(&row)
        );
        assert!(register
            .matching("question", &row.digest, Some(&"0".repeat(64)))
            .is_none());
        let bytes = register.to_text();
        assert_eq!(
            Verdicts::parse(Path::new("sample"), &bytes).unwrap(),
            register
        );
        let invalid = bytes.replace(&"a".repeat(64), "invalid context");
        assert!(Verdicts::parse(Path::new("sample"), &invalid).is_err());
        assert!(Verdicts::parse(Path::new("sample"), "prior_verdict = 3").is_err());
        assert!(Verdicts::parse(
            Path::new("sample"),
            &format!("{bytes}axioms = {{ proof = [3] }}\n")
        )
        .is_err());

        let root =
            std::env::temp_dir().join(format!("uorust-history-import-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let target = root.join("target.toml");
        let valid = root.join("valid.toml");
        let invalid = root.join("invalid.toml");
        std::fs::write(&target, Verdicts::default().to_text()).unwrap();
        let before = std::fs::read(&target).unwrap();
        std::fs::write(&valid, &bytes).unwrap();
        std::fs::write(&invalid, "[[").unwrap();
        assert!(Verdicts::import_files(&target, &[valid.clone(), invalid.clone()]).is_err());
        assert_eq!(
            std::fs::read(&target).unwrap(),
            before,
            "a partially parsed import replaced the original register"
        );
        assert!(Verdicts::import_files(&target, &[root.join("missing.toml")]).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), before);
        Verdicts::import_files(&target, &[valid]).unwrap();
        assert_eq!(Verdicts::read(&target).unwrap(), register);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rooted_bindings_reject_wrong_or_malformed_observations_ue_68() {
        let prefix="spec = \"uorust/verdicts/1\"\n[[verdict]]\nkappa=\"key\"\nfunction=\"f\"\ndigest=\"legacy\"\nstatus=\"proved\"\n";
        let path = Path::new("verdicts.toml");
        let legacy = Verdicts::parse(path, prefix).unwrap();
        assert!(legacy.rows["key"].matches("legacy", Some(&"a".repeat(64))));
        assert!(!legacy.rows["key"].matches("changed", Some(&"a".repeat(64))));
        let good = Verdicts::parse(
            path,
            &format!("{prefix}model_context_sha256=\"{}\"\n", "a".repeat(64)),
        )
        .unwrap();
        assert!(good.rows["key"].matches("changed", Some(&"a".repeat(64))));
        assert!(!good.rows["key"].matches("legacy", Some(&"b".repeat(64))));
        assert!(Verdicts::parse(path, &format!("{prefix}model_context_sha256=\"bad\"\n")).is_err());
        assert!(Verdicts::parse(path, &format!("{prefix}model_context_sha256=7\n")).is_err());
    }

    #[test]
    fn verdicts_round_trip() {
        let mut v = Verdicts::default();
        v.rows.insert(
            "k1".into(),
            Verdict {
                model_context_sha256: None,
                function: "c::f".into(),
                digest: "d".into(),
                status: Decision::Proved,
                axioms: [("holds_f_0".to_string(), vec!["propext".to_string()])]
                    .into_iter()
                    .collect(),
            },
        );
        v.rows.insert(
            "k2".into(),
            Verdict {
                model_context_sha256: None,
                function: "c::g".into(),
                digest: "e".into(),
                status: Decision::Refused("LLV7001: \"omega\" failed".into()),
                axioms: BTreeMap::new(),
            },
        );
        let text = v.to_text();
        assert_eq!(Verdicts::parse(Path::new("v.toml"), &text).unwrap(), v);
    }
}
