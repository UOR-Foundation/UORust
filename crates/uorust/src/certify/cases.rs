//! Agreement for any workspace: the certified package and the function it
//! certifies, on the cases the workspace's own tests state.
//!
//! The translator's differential ([`super::agree`]) compares whole outputs
//! of one program run twice, once through its certified packages and once
//! through its hand-written functions. It relies on what only the
//! translator has: a `certified!` call site written as the first statement
//! of each hand-written function, an adapter per root that converts the
//! function's own types to the package's (generated from the crate's MIR
//! dumps, into a module of the crate itself), the certified crate as a
//! dependency of the crate it certifies, a `--translator` switch in the
//! binary, a build flag that hides the call site from the lift, and a corpus
//! the binary translates (core, and itself). A workspace that is only
//! projected has none of these.
//!
//! What every projected workspace does have is its tests. The projection
//! states each closed assertion of a test over a lifted function as a closed
//! Boolean definition and proves it true ([`cases`]). Each is a production
//! root with no parameter: `lexlean production` certifies the Rust of the
//! whole assertion, the function's definition in its closure, and rustc runs
//! it ([`answers`]). The same assertion over the Rust function is the test
//! itself, which cargo runs ([`tests`]). A case agrees when the certified
//! package answers true and its test passes ([`verdicts`]); the cases are
//! the tests' own, none is invented, and a function no test asserts on has
//! none.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::{Definitions, Kind, Manifest, Profile, Root, BEHAVIOR};
use crate::error::io;
use crate::register::{Register, Status};
use crate::UorustError;

/// The domain tag of a case's κ.
pub const DOMAIN: &str = "uorust/certified-case/1";

/// What the projection ends the name of a case's definition with, and what
/// it adds for the theorem that the case holds.
const ROOT: &str = "Root";
const HOLDS: &str = "Holds";

/// What the projection starts the name of a case's definition with, before
/// the test's package and name in Pascal case.
const ACCEPT: &str = "accept";

/// The example of the certified crate that runs every case.
pub const HARNESS: &str = "cases";

/// A case: a closed assertion of a test, as a production root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Case {
    /// Its root: the definition, the κ of the definitions it runs, and the
    /// functions in scope it asserts on, by crate path.
    pub root: Root,
    /// Those functions.
    pub functions: Vec<String>,
}

/// The cases of a projection under `profile`: every definition of
/// `Behavior` the projection proves true as a closed Boolean (`<name>Holds`
/// states `((<name>) = true)`) whose closure reaches the definition of a
/// lifted function in scope, by definition. A case's κ is the SHA-256 of
/// [`DOMAIN`] and every definition of its closure, by name with its text, so
/// it is a new question when the assertion or anything it runs changes.
pub fn cases(
    profile: &Profile,
    modules: &BTreeMap<String, String>,
    register: &Register,
) -> Vec<Case> {
    let definitions = Definitions::of(modules);
    let behavior = modules.get(BEHAVIOR).map_or("", String::as_str);
    // The lifted functions in scope, by their definitions.
    let lifted: BTreeMap<String, &str> = register
        .rows
        .values()
        .filter(|row| profile.in_scope(&row.function))
        .filter_map(|row| match &row.status {
            Status::Lifted(definition) => Some((
                format!(
                    "{BEHAVIOR}::{}",
                    definition.rsplit("::").next().unwrap_or(definition)
                ),
                row.function.as_str(),
            )),
            _ => None,
        })
        .collect();
    let mut out = Vec::new();
    for chunk in super::chunks(behavior) {
        let Some(name) = chunk.name else { continue };
        if !name.ends_with(ROOT)
            || !behavior.contains(&format!("name: {name}{HOLDS},"))
            || !behavior.contains(&format!("statement: (({name}) = true)"))
        {
            continue;
        }
        let closure = definitions.closure(&format!("{BEHAVIOR}::{name}"));
        let functions: BTreeSet<&str> = closure
            .iter()
            .filter_map(|d| lifted.get(d).copied())
            .collect();
        if functions.is_empty() {
            continue;
        }
        let mut text = Vec::from(DOMAIN.as_bytes());
        for def in &closure {
            let body = definitions.text(def).unwrap_or_default();
            text.extend_from_slice(format!("\n{def}\n{}\n", body.len()).as_bytes());
            text.extend_from_slice(body.as_bytes());
        }
        let functions: Vec<String> = functions.into_iter().map(str::to_string).collect();
        out.push(Case {
            root: Root {
                function: functions.join(", "),
                kappa: crate::project::sha(&text),
                definition: name,
                prefix: profile.package.clone(),
            },
            functions,
        });
    }
    out.sort_by(|a, b| a.root.definition.cmp(&b.root.definition));
    out
}

/// The source of the example that runs every case of a manifest and prints
/// `<definition> <what its run returned>` for each, as Rust prints the
/// value: a Boolean, or one in the package's own bounded result.
pub fn harness(profile: &Profile, manifest: &Manifest) -> String {
    let krate = profile.crate_name();
    let mut out = String::from(
        "//! Written by `uorust certify agree`: runs every certified case of\n\
         //! `manifest.toml` and prints what each returns.\n\n\
         fn main() {\n",
    );
    for root in &manifest.roots {
        out.push_str(&format!(
            "    println!(\"{} {{:?}}\", {krate}::{}::run());\n",
            root.definition,
            profile.module(&root.package)
        ));
    }
    out.push_str("}\n");
    out
}

/// What a case answers, from what its run printed: `true` or `false`, bare
/// or as the `Ok` of a package whose naturals are the target's width, and
/// `overflow` for that package's failure.
fn answer(printed: &str) -> String {
    match printed
        .strip_prefix("Ok(")
        .and_then(|p| p.strip_suffix(')'))
    {
        Some(held) => held.to_string(),
        None if printed.starts_with("Err(") => "overflow".to_string(),
        None => printed.to_string(),
    }
}

/// What each case of the certified crate at `dir` answers, by definition:
/// every package source checked against the SHA-256 the manifest records
/// and the crate's `lib.rs` against the manifest's own, then the harness
/// written and run by cargo.
pub fn answers(
    dir: &Path,
    profile: &Profile,
    manifest: &Manifest,
) -> Result<BTreeMap<String, String>, UorustError> {
    let bad = |path: &Path, reason: String| UorustError::Malformed {
        path: path.to_path_buf(),
        reason,
    };
    for root in &manifest.roots {
        let path = dir.join("roots").join(format!("{}.rs", root.package));
        let bytes = std::fs::read(&path).map_err(|e| io(&path, e))?;
        let digest = crate::project::sha(&bytes);
        if digest != root.source {
            return Err(bad(
                &path,
                format!("SHA-256 {digest}, the manifest records {}", root.source),
            ));
        }
    }
    let lib = dir.join("src").join("lib.rs");
    let found = std::fs::read_to_string(&lib).map_err(|e| io(&lib, e))?;
    if found != profile.crate_source(&manifest.roots) {
        return Err(bad(&lib, "not the crate of the manifest's roots".into()));
    }
    let examples = dir.join("examples");
    std::fs::create_dir_all(&examples).map_err(|e| io(&examples, e))?;
    let path = examples.join(format!("{HARNESS}.rs"));
    std::fs::write(&path, harness(profile, manifest)).map_err(|e| io(&path, e))?;
    let run = std::process::Command::new("cargo")
        .args(["run", "--quiet", "--offline", "--example", HARNESS])
        .current_dir(dir)
        .output()
        .map_err(|e| UorustError::Tool(format!("cargo run --example {HARNESS}: {e}")))?;
    if !run.status.success() {
        return Err(UorustError::Tool(format!(
            "cargo run --example {HARNESS} in {}: {}",
            dir.display(),
            String::from_utf8_lossy(&run.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&run.stdout)
        .lines()
        .filter_map(|l| l.split_once(' '))
        .map(|(d, a)| (d.to_string(), answer(a)))
        .collect())
}

/// One test function's run, over every test of its name in its package.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ran {
    /// The tests libtest reported `ok`.
    pub passed: Vec<String>,
    /// Those it reported otherwise, with what it reported.
    pub other: Vec<String>,
}

/// The test functions libtest reports in `output`, the standard output of
/// `cargo test` of `package`, by the prefix of the cases the projection
/// states of each: `accept` and the package and the function's name in
/// Pascal case.
pub fn ran(package: &str, output: &str, into: &mut BTreeMap<String, Ran>) {
    for line in output.lines() {
        let Some((path, verdict)) = line
            .strip_prefix("test ")
            .and_then(|l| l.rsplit_once(" ... "))
        else {
            continue;
        };
        // A doc test's path names a file and a line, never a function.
        if path.contains(' ') {
            continue;
        }
        let name = path.rsplit("::").next().unwrap_or(path);
        let key = format!("{ACCEPT}{}", crate::lex::pascal(&[package, name]));
        let entry = into.entry(key).or_default();
        if verdict == "ok" {
            entry.passed.push(path.to_string());
        } else {
            entry.other.push(format!("{path} {verdict}"));
        }
    }
}

/// The packages whose tests may state a case: each whose name in Pascal
/// case the case's definition has after `accept`. One package's name can
/// start another's, so every one that fits is run, and the test function's
/// own key decides ([`test_of`]).
fn packages_of<'a>(definition: &str, packages: &'a [String]) -> Vec<&'a str> {
    packages
        .iter()
        .filter(|p| definition.starts_with(&format!("{ACCEPT}{}", crate::lex::pascal(&[p]))))
        .map(String::as_str)
        .collect()
}

/// The tests of the workspace at `workspace` that state the manifest's
/// cases, run by cargo: every test of each package a case names, by
/// [`ran`]'s keys.
pub fn tests(workspace: &Path, manifest: &Manifest) -> Result<BTreeMap<String, Ran>, UorustError> {
    let tool = |what: &str, reason: String| UorustError::Tool(format!("{what}: {reason}"));
    let listed = std::process::Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(workspace)
        .output()
        .map_err(|e| tool("cargo metadata", e.to_string()))?;
    if !listed.status.success() {
        return Err(UorustError::CargoMetadata(
            String::from_utf8_lossy(&listed.stderr).into_owned(),
        ));
    }
    let metadata: serde_json::Value = serde_json::from_slice(&listed.stdout)
        .map_err(|e| UorustError::CargoMetadata(e.to_string()))?;
    let packages: Vec<String> = metadata
        .get("packages")
        .and_then(|p| p.as_array())
        .into_iter()
        .flatten()
        .filter_map(|p| p.get("name")?.as_str().map(str::to_string))
        .collect();
    let named: BTreeSet<&str> = manifest
        .roots
        .iter()
        .flat_map(|r| packages_of(&r.definition, &packages))
        .collect();
    let mut out = BTreeMap::new();
    for package in named {
        // The verdict is each test's own line, never cargo's exit: a
        // failing test is a disagreement of its cases, named below.
        let run = std::process::Command::new("cargo")
            .args(["test", "--no-fail-fast", "--package", package])
            .current_dir(workspace)
            .output()
            .map_err(|e| tool("cargo test", e.to_string()))?;
        let text = String::from_utf8_lossy(&run.stdout);
        if !text.lines().any(|l| l.starts_with("test result: ")) {
            return Err(tool(
                &format!("cargo test --package {package}"),
                String::from_utf8_lossy(&run.stderr).into_owned(),
            ));
        }
        ran(package, &text, &mut out);
    }
    Ok(out)
}

/// The test function's key a case's definition is stated under: the key
/// the definition has before the assertion's index and `Root`, the longest
/// when several fit.
fn test_of<'a>(definition: &str, tests: &'a BTreeMap<String, Ran>) -> Option<&'a Ran> {
    tests
        .iter()
        .filter(|(key, _)| {
            definition
                .strip_prefix(key.as_str())
                .and_then(|rest| rest.strip_suffix(ROOT))
                .is_some_and(|index| !index.is_empty() && index.chars().all(|c| c.is_ascii_digit()))
        })
        .max_by_key(|(key, _)| key.len())
        .map(|(_, ran)| ran)
}

/// The verdict of every case of a manifest: one line per case that agrees,
/// `<definition> <functions>`. Refused, naming each case with what parted,
/// when a certified package does not answer true, when the test that states
/// a case did not pass or did not run, and when there is no case at all, so
/// that an agreement is never an empty one.
pub fn verdicts(
    manifest: &Manifest,
    answers: &BTreeMap<String, String>,
    tests: &BTreeMap<String, Ran>,
) -> Result<Vec<String>, UorustError> {
    let mut lines = Vec::new();
    let mut disagree = Vec::new();
    for root in &manifest.roots {
        let case = format!("{} ({})", root.definition, root.function);
        let mut parted = Vec::new();
        match answers.get(&root.definition).map(String::as_str) {
            Some("true") => {}
            Some(other) => parted.push(format!("the certified package answers {other}")),
            None => parted.push("the certified package gave no answer".to_string()),
        }
        match test_of(&root.definition, tests) {
            Some(ran) if ran.other.is_empty() && !ran.passed.is_empty() => {}
            Some(ran) => parted.push(format!("its test did not pass: {}", ran.other.join(", "))),
            None => parted.push("no test of the workspace that states it ran".to_string()),
        }
        if parted.is_empty() {
            lines.push(format!("{} {}", root.definition, root.function));
        } else {
            disagree.push(format!("{case}: {}", parted.join("; ")));
        }
    }
    if manifest.roots.is_empty() {
        disagree.push("the manifest certifies no case".to_string());
    }
    if disagree.is_empty() {
        Ok(lines)
    } else {
        Err(UorustError::Disagree(disagree))
    }
}

/// The agreement of the certified cases taken into `dir` with the tests of
/// the workspace at `workspace`: [`verdicts`] over what the packages answer
/// ([`answers`]) and what the tests did ([`tests`]).
pub fn agree(dir: &Path, workspace: &Path) -> Result<Vec<String>, UorustError> {
    let path = dir.join("manifest.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
    let profile = Profile::recorded(&path, &text)?;
    if profile.kind != Kind::Cases {
        return Err(UorustError::Malformed {
            path,
            reason: "not the manifest of a cases export (certify production --cases)".to_string(),
        });
    }
    let manifest = Manifest::parse(&path, &text)?;
    let answered = answers(dir, &profile, &manifest)?;
    let tested = tests(workspace, &manifest)?;
    verdicts(&manifest, &answered, &tested)
}

/// The functions the cases of a manifest assert on, by crate path.
pub fn covered(manifest: &Manifest) -> BTreeSet<String> {
    manifest
        .roots
        .iter()
        .flat_map(|r| r.function.split(", "))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::Certified;
    use super::*;
    use crate::register::Row;

    fn register(rows: &[(&str, &str, &str)]) -> Register {
        let mut r = Register::default();
        for (kappa, function, definition) in rows {
            r.rows.insert(
                (*kappa).to_string(),
                Row {
                    function: (*function).to_string(),
                    status: Status::Lifted(format!("Behavior::{definition}")),
                    divergence: None,
                },
            );
        }
        r
    }

    fn profile() -> Profile {
        Profile {
            scope: vec!["meter::".to_string()],
            entry: None,
            package: "meter_certified_".to_string(),
            kind: Kind::Cases,
            ..Profile::default()
        }
    }

    fn behavior(above: &str) -> BTreeMap<String, String> {
        let text = format!(
            "definition{{\n  name: above,\n  parameters: [n : Nat],\n  result: Nat,\n  body: {above}\n}}\n\n\
             definition{{\n  name: other,\n  parameters: [n : Nat],\n  result: Nat,\n  body: n\n}}\n\n\
             definition{{\n  name: acceptMeterCountsAbove0Root,\n  parameters: [],\n  result: Bool,\n  body: ((above 2) =? 3)\n}}\n\n\
             theorem{{\n  name: acceptMeterCountsAbove0RootHolds,\n  parameters: [],\n  statement: ((acceptMeterCountsAbove0Root) = true),\n  proof: decide{{}}\n}}\n\n\
             definition{{\n  name: acceptGaugeReads0Root,\n  parameters: [],\n  result: Bool,\n  body: ((other 2) =? 2)\n}}\n\n\
             theorem{{\n  name: acceptGaugeReads0RootHolds,\n  parameters: [],\n  statement: ((acceptGaugeReads0Root) = true),\n  proof: decide{{}}\n}}\n\n\
             definition{{\n  name: plainRoot,\n  parameters: [],\n  result: Bool,\n  body: ((above 1) =? 2)\n}}\n"
        );
        [
            ("Behavior".to_string(), text),
            ("Prelude".to_string(), String::new()),
        ]
        .into_iter()
        .collect()
    }

    // A case is a closed assertion the projection proves, over a lifted
    // function in scope; its κ is the question of everything it runs.
    #[test]
    fn a_case_is_a_proved_closed_assertion_over_a_function_in_scope() {
        let reg = register(&[
            ("k1", "meter::above", "above"),
            ("k2", "gauge::other", "other"),
        ]);
        let found = cases(&profile(), &behavior("(n + 1)"), &reg);
        // `acceptGaugeReads0Root` asserts on a function outside the scope,
        // and `plainRoot` has no theorem that it holds.
        assert_eq!(found.len(), 1);
        let case = &found[0];
        assert_eq!(case.root.definition, "acceptMeterCountsAbove0Root");
        assert_eq!(case.functions, ["meter::above"]);
        assert_eq!(
            case.root.package(),
            "meter_certified_accept_meter_counts_above0_root"
        );
        // The function's definition changed: the case is a new question.
        let changed = cases(&profile(), &behavior("(n + 2)"), &reg);
        assert_ne!(changed[0].root.kappa, case.root.kappa);
        // An unrelated definition changed: the same question.
        let mut elsewhere = behavior("(n + 1)");
        let text = elsewhere["Behavior"].replace("  body: n\n", "  body: (n + 0)\n");
        elsewhere.insert("Behavior".to_string(), text);
        assert_eq!(
            cases(&profile(), &elsewhere, &reg)[0].root.kappa,
            case.root.kappa
        );
    }

    fn manifest(definitions: &[&str]) -> Manifest {
        Manifest {
            roots: definitions
                .iter()
                .map(|d| Certified {
                    function: "meter::above".into(),
                    kappa: format!("k-{d}"),
                    definition: (*d).to_string(),
                    package: format!("meter_certified_{}", d.to_lowercase()),
                    source: "s".into(),
                    certification: "kernel-checked".into(),
                    identity: None,
                })
                .collect(),
            ..Manifest::default()
        }
    }

    #[test]
    fn libtest_lines_are_read_by_the_cases_they_state() {
        let mut out = BTreeMap::new();
        ran(
            "meter",
            "running 3 tests\ntest tests::counts_above ... ok\ntest tests::reads_low ... FAILED\ntest src/lib.rs - above (line 3) ... ok\n\ntest result: FAILED. 1 passed; 1 failed\n",
            &mut out,
        );
        assert_eq!(
            out["acceptMeterCountsAbove"].passed,
            ["tests::counts_above"]
        );
        assert_eq!(
            out["acceptMeterReadsLow"].other,
            ["tests::reads_low FAILED"]
        );
        assert_eq!(out.len(), 2);
    }

    // The planted defects of the agreement: a certified package that answers
    // false, a test that failed, a case no test ran and a manifest of no
    // case are each refused, naming the case and what parted.
    #[test]
    fn a_case_agrees_only_when_the_package_answers_true_and_its_test_passes() {
        let m = manifest(&[
            "acceptMeterCountsAbove0Root",
            "acceptMeterCountsAbove12Root",
        ]);
        let mut tests = BTreeMap::new();
        ran("meter", "test tests::counts_above ... ok\n", &mut tests);
        let both: BTreeMap<String, String> = m
            .roots
            .iter()
            .map(|r| (r.definition.clone(), "true".to_string()))
            .collect();
        assert_eq!(
            verdicts(&m, &both, &tests),
            Ok(vec![
                "acceptMeterCountsAbove0Root meter::above".to_string(),
                "acceptMeterCountsAbove12Root meter::above".to_string()
            ])
        );
        let mut planted = both.clone();
        planted.insert("acceptMeterCountsAbove12Root".into(), "false".into());
        match verdicts(&m, &planted, &tests) {
            Err(UorustError::Disagree(rows)) => assert_eq!(
                rows,
                ["acceptMeterCountsAbove12Root (meter::above): the certified package answers false"]
            ),
            other => panic!("not refused: {other:?}"),
        }
        let mut failed = BTreeMap::new();
        ran(
            "meter",
            "test tests::counts_above ... FAILED\n",
            &mut failed,
        );
        match verdicts(&m, &both, &failed) {
            Err(UorustError::Disagree(rows)) => {
                assert_eq!(rows.len(), 2);
                assert!(rows[0].ends_with("its test did not pass: tests::counts_above FAILED"));
            }
            other => panic!("not refused: {other:?}"),
        }
        match verdicts(&m, &both, &BTreeMap::new()) {
            Err(UorustError::Disagree(rows)) => {
                assert!(rows[0].ends_with("no test of the workspace that states it ran"));
            }
            other => panic!("not refused: {other:?}"),
        }
        assert!(matches!(
            verdicts(&manifest(&[]), &BTreeMap::new(), &tests),
            Err(UorustError::Disagree(_))
        ));
    }

    #[test]
    fn every_package_whose_name_fits_a_case_is_run() {
        let packages = [
            "meter".to_string(),
            "meter-core".to_string(),
            "gauge".to_string(),
        ];
        assert_eq!(
            packages_of("acceptMeterCoreReads0Root", &packages),
            ["meter", "meter-core"]
        );
        // The test function's own key decides: the longest that fits.
        let mut tests = BTreeMap::new();
        ran("meter", "test core_reads ... FAILED\n", &mut tests);
        ran(
            "meter-core",
            "test reads ... ok\ntest reads_low ... ok\n",
            &mut tests,
        );
        assert_eq!(tests.len(), 2);
        let found = test_of("acceptMeterCoreReads0Root", &tests).expect("a test");
        assert_eq!(found.passed, ["reads"]);
        assert_eq!(found.other, ["core_reads FAILED"]);
        assert!(test_of("acceptMeterCoreReadsLow0Root", &tests).is_some());
        assert!(test_of("acceptMeterCoreReadsHigh0Root", &tests).is_none());
    }

    #[test]
    fn the_harness_runs_every_case_of_the_manifest() {
        let text = harness(&profile(), &manifest(&["acceptMeterCountsAbove0Root"]));
        assert!(text.contains(
            "println!(\"acceptMeterCountsAbove0Root {:?}\", meter_certified::acceptmetercountsabove0root::run());"
        ));
        assert_eq!(answer("true"), "true");
        assert_eq!(answer("Ok(true)"), "true");
        assert_eq!(answer("Ok(false)"), "false");
        assert_eq!(answer("Err(Overflow)"), "overflow");
    }
}
