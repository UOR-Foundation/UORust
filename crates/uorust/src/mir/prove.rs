//! `uorust prove`: decide every translation of the MIR path with the
//! verifier, with no one in the loop.
//!
//! The prover projects the workspace with every undecided translation
//! admitted on trial, writes the projection to a scratch directory, and runs
//! `lexlean verify` on it. When it verifies, every translation on trial is
//! proved. When it does not, each diagnostic is attributed to the
//! declaration it names (or whose source lines it points at): a report of
//! the axioms a declaration elaborated with is learned, any other refusal
//! decides that translation refused, with the diagnostic as its cause. A
//! failure no diagnostic attributes splits the trial in halves until it
//! does. The verdicts are written after every decision, so an interrupted
//! run keeps what it decided.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;
use std::process::Command;

use super::verdict::{Decision, Verdict, Verdicts};
use super::{MirOptions, Trial, CANDIDATES};
use crate::error::io;
use crate::{Options, UorustError};

/// One translation a trial projection lists.
#[derive(Clone, Debug)]
struct Trialled {
    kappa: String,
    function: String,
    digest: String,
    model_context_sha256: Option<String>,
    trial: bool,
    decls: Vec<String>,
    dependencies: BTreeSet<String>,
}

/// An immutable dependency slice awaiting only observed policy corrections.
struct PolicyRetry {
    trial: Trial,
    files: BTreeMap<String, Vec<u8>>,
    candidates: Vec<Trialled>,
}

/// What a run decided.
#[derive(Clone, Debug, Default)]
pub struct Summary {
    /// Translations proved by this run.
    pub proved: usize,
    /// Translations refused by this run.
    pub refused: usize,
    /// Verifier runs.
    pub runs: usize,
    /// Actual whole-workspace projections, including the initial catalogue.
    pub projections: usize,
    /// Exact projected packages reused for observed policy-only retries.
    pub projection_reuses: usize,
}

fn candidates(files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<Trialled>, UorustError> {
    let text = String::from_utf8_lossy(files.get(CANDIDATES).map_or(&[][..], Vec::as_slice));
    let bad = |reason: String| UorustError::Malformed {
        path: CANDIDATES.into(),
        reason,
    };
    let doc: toml::Value = text.parse().map_err(|e| bad(format!("{e}")))?;
    let mut out = Vec::new();
    for row in doc
        .get("candidate")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
    {
        let field = |n: &str| {
            row.get(n)
                .and_then(toml::Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| bad(format!("a candidate without {n}")))
        };
        out.push(Trialled {
            kappa: field("kappa")?,
            function: field("function")?,
            digest: field("digest")?,
            model_context_sha256: row
                .get("model_context_sha256")
                .and_then(toml::Value::as_str)
                .map(str::to_string),
            trial: row
                .get("trial")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false),
            dependencies: row
                .get("dependencies")
                .and_then(toml::Value::as_array)
                .ok_or_else(|| bad("a candidate without dependencies".into()))?
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_string)
                        .ok_or_else(|| bad("a dependency is not a source identity".into()))
                })
                .collect::<Result<_, UorustError>>()?,
            decls: row
                .get("decls")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect(),
        });
    }
    Ok(out)
}

fn trial_behavior(files: &BTreeMap<String, Vec<u8>>) -> String {
    String::from_utf8_lossy(
        files
            .get("src/Behavior.lex.tex")
            .map_or(&[][..], Vec::as_slice),
    )
    .into_owned()
}

/// A verifier diagnostic.
#[derive(Clone, Debug)]
struct Diagnostic {
    code: String,
    message: String,
    path: String,
    line: usize,
}

fn shared_context_error(d: &Diagnostic) -> bool {
    let source = std::path::Path::new(&d.path);
    // A module-wide Behavior failure may have no declaration location;
    // it still belongs to the bounded trial and must be reduced by the
    // existing component splitter, not attributed to the shared prelude.
    (d.path.ends_with(".lex.tex")
        && !d.path.ends_with("Behavior.lex.tex")
        && named(d, "").is_empty())
        || (source.file_name().is_some_and(|n| n == "lexlean.toml")
            && !d.code.starts_with("LLV")
            && !d.message.contains(".Behavior"))
}

/// Type-check before proving, using only the native behavior's dependency closure.
fn verify(dir: &Path) -> Result<Option<Vec<Diagnostic>>, UorustError> {
    let mut doc = serde_json::Value::Null;
    for phase in ["check", "verify"] {
        let run = Command::new("lexlean")
            .args([phase, "src/Behavior.lex.tex", "--diagnostic-format", "json"])
            .current_dir(dir)
            .output()
            .map_err(|e| UorustError::Tool(format!("lexlean {phase}: {e}")))?;
        let receipt = dir.join("uorust-proof-result.json");
        std::fs::write(&receipt, &run.stdout).map_err(|e| io(&receipt, e))?;
        doc = serde_json::from_slice(&run.stdout).map_err(|e| {
            UorustError::Tool(format!(
                "lexlean {phase}: {e}: {}",
                String::from_utf8_lossy(&run.stderr)
            ))
        })?;
        if doc.get("success").and_then(serde_json::Value::as_bool) != Some(true) {
            break;
        }
    }
    if doc.get("success").and_then(serde_json::Value::as_bool) == Some(true) {
        return Ok(None);
    }
    let mut out = Vec::new();
    for d in doc
        .get("diagnostics")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        let s = |v: Option<&serde_json::Value>| {
            v.and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        let primary = d.get("primary");
        out.push(Diagnostic {
            code: s(d.get("code")),
            message: s(d.get("message")),
            path: s(primary.and_then(|p| p.get("path"))),
            line: primary
                .and_then(|p| p.get("line_start"))
                .and_then(serde_json::Value::as_u64)
                .and_then(|l| usize::try_from(l).ok())
                .unwrap_or(0),
        });
    }
    for diagnostic in &out {
        eprintln!(
            "{}: {}:{}: {}",
            diagnostic.code, diagnostic.path, diagnostic.line, diagnostic.message
        );
    }
    if let Some(diagnostic) = out.iter().find(|d| shared_context_error(d)) {
        return Err(UorustError::Tool(format!(
            "shared proof context {}:{}: {}: {}",
            diagnostic.path, diagnostic.line, diagnostic.code, diagnostic.message
        )));
    }
    Ok(Some(out))
}

/// The declarations a diagnostic names: `` `Prefix.Behavior.name` `` in its
/// message, or the declaration of `Behavior.lex.tex` at its line.
fn named(d: &Diagnostic, behavior: &str) -> Vec<String> {
    let mut out: Vec<String> = d
        .message
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|q| q.contains(".Behavior."))
        .filter_map(|q| q.rsplit('.').next().map(str::to_string))
        .collect();
    if out.is_empty() && d.path.ends_with("Behavior.lex.tex") && d.line > 0 {
        let lines: Vec<&str> = behavior.lines().collect();
        let mut current = None;
        for (i, line) in lines.iter().enumerate() {
            if let Some(n) = line.strip_prefix("  name: ") {
                if lines
                    .get(i.wrapping_sub(1))
                    .is_some_and(|p| p.ends_with('{'))
                {
                    current = Some(n.trim_end_matches(',').to_string());
                }
            }
            if i + 1 >= d.line {
                if !line.starts_with(' ') && line.ends_with('{') {
                    current = lines.get(i + 1).and_then(|next| {
                        next.strip_prefix("  name: ")
                            .map(|name| name.trim_end_matches(',').to_string())
                    });
                }
                break;
            }
        }
        out.extend(current);
    }
    out
}

/// The axioms a `LLV7005` report says a declaration elaborated with.
fn observed(d: &Diagnostic) -> Option<Vec<String>> {
    if d.code != "LLV7005" {
        return None;
    }
    let list = d.message.rsplit("observed [").next()?.split(']').next()?;
    Some(
        list.split(',')
            .map(str::trim)
            .filter(|a| !a.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

/// Default maximum number of undecided functions in one verifier trial.
pub const DEFAULT_PROOF_BATCH_SIZE: usize = 8;

/// Partition a closed dependency graph into bounded dependency-ordered trials.
/// Recursive components stay together. Invalid budgets, missing dependencies,
/// and components larger than the budget are typed tool refusals.
pub fn proof_batches(
    graph: &BTreeMap<String, BTreeSet<String>>,
    maximum: usize,
) -> Result<Vec<BTreeSet<String>>, UorustError> {
    let refuse = |message| UorustError::Tool(format!("proof batch: {message}"));
    if maximum == 0 {
        return Err(refuse("the function budget must be positive".to_string()));
    }
    for (node, dependencies) in graph {
        if let Some(missing) = dependencies.iter().find(|name| !graph.contains_key(*name)) {
            return Err(refuse(format!("{node} names absent dependency {missing}")));
        }
    }
    let mut components = crate::lift::strong_components(graph);
    components.sort();
    if let Some(component) = components.iter().find(|group| group.len() > maximum) {
        return Err(refuse(format!(
            "recursive component of {} functions exceeds budget {maximum}: {}",
            component.len(),
            component.iter().cloned().collect::<Vec<_>>().join(", ")
        )));
    }
    let mut done = BTreeSet::new();
    let mut result = Vec::new();
    while !components.is_empty() {
        let ready: Vec<BTreeSet<String>> = components
            .iter()
            .filter(|group| {
                group.iter().all(|node| {
                    graph[node]
                        .iter()
                        .all(|dependency| done.contains(dependency) || group.contains(dependency))
                })
            })
            .cloned()
            .collect();
        assert!(
            !ready.is_empty(),
            "a closed SCC graph has a ready component"
        );
        let mut batch = BTreeSet::new();
        for group in &ready {
            if batch.len() + group.len() > maximum {
                result.push(std::mem::take(&mut batch));
            }
            batch.extend(group.iter().cloned());
        }
        if !batch.is_empty() {
            result.push(batch);
        }
        for group in &ready {
            done.extend(group.iter().cloned());
        }
        components.retain(|group| !group.is_subset(&done));
    }
    Ok(result)
}

fn trial_graph(candidates: &[Trialled]) -> BTreeMap<String, BTreeSet<String>> {
    let pending: BTreeSet<String> = candidates
        .iter()
        .filter(|row| row.trial)
        .map(|row| row.kappa.clone())
        .collect();
    candidates
        .iter()
        .filter(|row| row.trial)
        .map(|row| {
            (
                row.kappa.clone(),
                row.dependencies.intersection(&pending).cloned().collect(),
            )
        })
        .collect()
}

/// Decide every undecided translation of the workspace at `root`, writing
/// the verdict register to `verdicts` after each decision.
pub fn prove(
    root: &Path,
    options: &Options,
    dumps: &Path,
    verdicts: &Path,
    scratch: &Path,
) -> Result<Summary, UorustError> {
    prove_bounded(
        root,
        options,
        dumps,
        verdicts,
        scratch,
        DEFAULT_PROOF_BATCH_SIZE,
    )
}

/// Decide MIR translations in dependency-ordered trials bounded by `maximum`.
/// Every trial is checked by the actual SDK verifier; cached proved digests
/// are reused and no unverified translation is admitted.
pub fn prove_bounded(
    root: &Path,
    options: &Options,
    dumps: &Path,
    verdicts: &Path,
    scratch: &Path,
    maximum: usize,
) -> Result<Summary, UorustError> {
    proof_batches(&BTreeMap::new(), maximum)?;
    let mut register = Verdicts::read(verdicts)?;
    let mut summary = Summary::default();
    // Prior observations are policy hints, never proof of changed bytes.
    // A stale digest is still removed below and must pass the actual SDK.
    // The SDK's exact-policy diagnostics correct any obsolete hint.
    let mut learned: BTreeMap<String, BTreeMap<String, Vec<String>>> = register
        .rows
        .iter()
        .filter(|(_, row)| matches!(row.status, Decision::Proved))
        .map(|(key, row)| (key.clone(), row.axioms.clone()))
        .collect();
    summary.projections += 1;
    let catalogue = crate::project_workspace(
        root,
        &Options {
            mir: Some(MirOptions {
                dumps: dumps.to_path_buf(),
                verdicts: register.clone(),
                trial: Trial::All,
                learned: learned.clone(),
            }),
            ..options.clone()
        },
    )?;
    let all = candidates(&catalogue)?;
    // A legacy exact match already binds this emitted closure. Observe its
    // rooted identity without changing the original digest, status or axioms.
    for c in &all {
        if let Some(row) = register.rows.get_mut(&c.kappa) {
            if row.digest == c.digest && row.model_context_sha256.is_none() {
                row.model_context_sha256 = c.model_context_sha256.clone();
            }
        }
    }
    let live: BTreeMap<&str, &Trialled> = all
        .iter()
        .filter(|row| row.trial)
        .map(|row| (row.kappa.as_str(), row))
        .collect();
    register.retain(|key, value| {
        live.get(key.as_str())
            .is_none_or(|c| value.matches(&c.digest, c.model_context_sha256.as_deref()))
    });
    let mut groups: VecDeque<Trial> = proof_batches(&trial_graph(&all), maximum)?
        .into_iter()
        .map(Trial::Only)
        .collect();
    let mut policy_retry: Option<PolicyRetry> = None;
    while let Some(trial) = groups.pop_front() {
        let current = register.clone();
        let opts = Options {
            mir: Some(MirOptions {
                dumps: dumps.to_path_buf(),
                verdicts: current,
                trial: trial.clone(),
                // A translation on trial elaborates with what was learned for it.
                learned: learned.clone(),
            }),
            ..options.clone()
        };
        let (mut files, all, reused) = match policy_retry.take() {
            Some(PolicyRetry {
                trial: original_trial,
                files,
                candidates,
            }) => {
                if original_trial != trial {
                    return Err(UorustError::Tool(
                        "proof policy: retry changed its dependency trial".into(),
                    ));
                }
                summary.projection_reuses += 1;
                (files, candidates, true)
            }
            None => {
                summary.projections += 1;
                let files = crate::project_workspace(root, &opts)?;
                let all = candidates(&files)?;
                (files, all, false)
            }
        };
        // Only a currently verifiable translation can replace a decision.
        // A blocked caller has an incomplete dependency closure; its digest
        // cannot invalidate a proof obtained from the complete inputs.
        let live: BTreeMap<&str, &Trialled> = all
            .iter()
            .filter(|c| c.trial)
            .map(|c| (c.kappa.as_str(), c))
            .collect();
        register.retain(|key, value| {
            live.get(key.as_str())
                .is_none_or(|c| value.matches(&c.digest, c.model_context_sha256.as_deref()))
        });
        let on_trial: Vec<&Trialled> = all.iter().filter(|c| c.trial).collect();
        if on_trial.is_empty() {
            continue;
        }
        if !reused {
            files.remove(CANDIDATES);
            let roots: Vec<String> = on_trial.iter().flat_map(|c| c.decls.clone()).collect();
            let behavior = files.get_mut("src/Behavior.lex.tex").ok_or_else(|| {
                UorustError::Tool("proof closure: missing Behavior module".into())
            })?;
            *behavior =
                crate::lex::proof_closure(&String::from_utf8_lossy(behavior), &roots)?.into_bytes();
        }
        write_scratch(scratch, &files)?;
        summary.runs += 1;
        let behavior = trial_behavior(&files);
        let Some(diagnostics) = verify(scratch)? else {
            for c in &on_trial {
                register.rows.insert(
                    c.kappa.clone(),
                    Verdict {
                        function: c.function.clone(),
                        digest: c.digest.clone(),
                        model_context_sha256: c.model_context_sha256.clone(),
                        status: Decision::Proved,
                        axioms: learned.get(&c.kappa).cloned().unwrap_or_default(),
                    },
                );
                summary.proved += 1;
            }
            std::fs::write(verdicts, register.to_text()).map_err(|e| io(verdicts, e))?;
            continue;
        };
        let owner = |decl: &str| -> Vec<&Trialled> {
            on_trial
                .iter()
                .filter(|c| c.decls.iter().any(|d| d == decl))
                .copied()
                .collect()
        };
        let mut progress = false;
        let mut only_policy = true;
        let mut policies = BTreeMap::new();
        for d in &diagnostics {
            for decl in named(d, &behavior) {
                for c in owner(&decl) {
                    if let Some(axioms) = observed(d) {
                        let entry = learned.entry(c.kappa.clone()).or_default();
                        if entry.get(&decl) != Some(&axioms) {
                            entry.insert(decl.clone(), axioms.clone());
                            policies.insert(decl.clone(), axioms);
                            progress = true;
                        }
                    } else if !register.rows.contains_key(&c.kappa) {
                        only_policy = false;
                        let first = d.message.lines().next().unwrap_or_default();
                        register.rows.insert(
                            c.kappa.clone(),
                            Verdict {
                                function: c.function.clone(),
                                digest: c.digest.clone(),
                                model_context_sha256: c.model_context_sha256.clone(),
                                status: Decision::Refused(format!("{}: {first}", d.code)),
                                axioms: BTreeMap::new(),
                            },
                        );
                        summary.refused += 1;
                        progress = true;
                    }
                }
            }
        }
        if progress {
            std::fs::write(verdicts, register.to_text()).map_err(|e| io(verdicts, e))?;
            if only_policy && !policies.is_empty() {
                let body = files.get_mut("src/Behavior.lex.tex").ok_or_else(|| {
                    UorustError::Tool("proof policy: missing Behavior module".into())
                })?;
                *body = crate::lex::with_axiom_policies(&String::from_utf8_lossy(body), &policies)?
                    .into_bytes();
                policy_retry = Some(PolicyRetry {
                    trial: trial.clone(),
                    files,
                    candidates: all.clone(),
                });
            }
            groups.push_front(trial);
            continue;
        }
        // Only independent recursive components may be split. Splitting a
        // cycle would omit an undecided dependency and silently lose work.
        let components = crate::lift::strong_components(&trial_graph(&all));
        if components.len() <= 1 {
            let first = diagnostics
                .first()
                .map(|d| {
                    format!(
                        "{}: {}",
                        d.code,
                        d.message.lines().next().unwrap_or_default()
                    )
                })
                .unwrap_or_else(|| "the verifier failed without a diagnostic".to_string());
            return Err(UorustError::Tool(format!(
                "inconclusive native proof: {first}; no function verdict recorded; SDK receipt {}",
                scratch.join("uorust-proof-result.json").display()
            )));
        }
        let half = components.len() / 2;
        groups.push_front(Trial::Only(
            components[half..].iter().flatten().cloned().collect(),
        ));
        groups.push_front(Trial::Only(
            components[..half].iter().flatten().cloned().collect(),
        ));
    }
    std::fs::write(verdicts, register.to_text()).map_err(|e| io(verdicts, e))?;
    Ok(summary)
}

/// Replace the scratch projection with `files`, keeping the verifier's
/// build cache, whose entries are content-addressed.
fn write_scratch(scratch: &Path, files: &BTreeMap<String, Vec<u8>>) -> Result<(), UorustError> {
    if scratch.is_dir() {
        for entry in std::fs::read_dir(scratch).map_err(|e| io(scratch, e))? {
            let path = entry.map_err(|e| io(scratch, e))?.path();
            if path.file_name().is_some_and(|n| n == ".lexlean") {
                continue;
            }
            if path.is_dir() {
                std::fs::remove_dir_all(&path).map_err(|e| io(&path, e))?;
            } else {
                std::fs::remove_file(&path).map_err(|e| io(&path, e))?;
            }
        }
    }
    for (path, bytes) in files {
        let path = scratch.join(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
        }
        std::fs::write(&path, bytes).map_err(|e| io(&path, e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_feedback_uses_root_not_library_behavior_ue_54() {
        let files = BTreeMap::from([
            (
                "library/helper/src/Behavior.lex.tex".into(),
                b"theorem{\n  name: libraryClaim,\n}\n".to_vec(),
            ),
            (
                "src/Behavior.lex.tex".into(),
                b"theorem{\n  name: holdsNative,\n}\n".to_vec(),
            ),
        ]);
        let diagnostic = Diagnostic {
            code: "LLV7002".into(),
            message: "Lean rejected `Workspace.Behavior` (error): omega could not prove the goal"
                .into(),
            path: "src/Behavior.lex.tex".into(),
            line: 1,
        };
        assert_eq!(named(&diagnostic, &trial_behavior(&files)), ["holdsNative"]);
    }

    #[test]
    fn a_shared_model_failure_is_not_a_function_verdict_ue_52() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let opts = Options {
            name: "shared_failure".into(),
            last_modified: "2026-11-11T00:00:00Z".into(),
            advisories: None,
            mir: None,
            limits: None,
            embedding: None,
        };
        let mut files = crate::project_workspace(&root.join("tests/fixtures/thermostat"), &opts)
            .expect("valid generated project");
        let bad = crate::lex::Decl::Definition {
            name: "brokenSharedType".into(),
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            result: "Nat".into(),
            body: crate::lex::Expr::Flat("false".into()),
            axioms: Vec::new(),
            recursive: None,
            termination: None,
            mutual: None,
        };
        let prelude = String::from_utf8(files["src/Prelude.lex.tex"].clone()).expect("UTF-8");
        files.insert(
            "src/Prelude.lex.tex".into(),
            prelude
                .replace(
                    "\\end{semanticmodule}",
                    &format!("{}\\end{{semanticmodule}}", crate::lex::render_decl(&bad)),
                )
                .into_bytes(),
        );
        let dir =
            std::env::temp_dir().join(format!("uorust-shared-failure-{}", std::process::id()));
        write_scratch(&dir, &files).expect("planted owned-model defect");
        assert!(
            matches!(verify(&dir),Err(UorustError::Tool(message)) if message.contains("shared proof context") && message.contains("Prelude.lex.tex"))
        );
        let actual: serde_json::Value = serde_json::from_slice(
            &std::fs::read(dir.join("uorust-proof-result.json")).expect("raw SDK result"),
        )
        .expect("SDK JSON");
        assert_eq!(actual["success"], false);
        std::fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn diagnostics_name_their_declarations() {
        let d = Diagnostic {
            code: "LLV7005".into(),
            message: "`Uorust.Behavior.mirSum` violates its exact axiom policy: observed [Quot.sound, propext]".into(),
            path: "lexlean.toml".into(),
            line: 1,
        };
        assert_eq!(named(&d, ""), ["mirSum"]);
        assert_eq!(
            observed(&d),
            Some(vec!["Quot.sound".to_string(), "propext".to_string()])
        );
        let behavior = "x\ndefinition{\n  name: a,\n  body: 1\n}\n\ntheorem{\n  name: b,\n  statement: (1 = 1)\n}\n";
        let at = Diagnostic {
            code: "LLV7001".into(),
            message: "omega could not prove the goal".into(),
            path: "src/Behavior.lex.tex".into(),
            line: 9,
        };
        assert_eq!(named(&at, behavior), ["b"]);
        assert_eq!(observed(&at), None);
        // The SDK's Lean diagnostic points at the opening declaration,
        // before its name. It must never blame the preceding definition.
        assert_eq!(named(&Diagnostic { line: 7, ..at }, behavior), ["b"]);
    }

    #[test]
    fn module_behavior_failures_remain_batch_feedback_ue_52() {
        let module = Diagnostic {
            code: "LLV7002".into(),
            message: "Lean rejected `Example.Behavior`: ".into(),
            path: "lexlean.toml".into(),
            line: 1,
        };
        assert!(!shared_context_error(&module));
        let prelude = Diagnostic {
            code: "LLT4001".into(),
            message: "expected Nat, got Bool".into(),
            path: "src/Prelude.lex.tex".into(),
            line: 7,
        };
        assert!(shared_context_error(&prelude));
    }
}
