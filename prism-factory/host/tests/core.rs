//! Differential and behavioral checks of the certified factory core through
//! the host, on this tool's graph. The kernel proves the routing and the graph
//! theorems in the model; these hold the byte-level parts the kernel cannot
//! evaluate (SHA-256 over `Bytes`, the text protocol) against an independent
//! oracle, Python's `hashlib` over the README's identity text
//! (`tests/identity.py`), and show which obligations a change reaches.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::process::{Command, Stdio};

use prism_factory_host::{plan, reweigh};

fn planned(request: &str) -> String {
    String::from_utf8(plan(request.as_bytes()).expect("no overflow")).expect("UTF-8")
}

/// Each obligation's route and identity, from the plan's `obligation` lines.
fn routes(plan: &str) -> BTreeMap<String, (String, String)> {
    plan.lines()
        .filter_map(|line| line.strip_prefix("obligation "))
        .map(|rest| {
            let mut parts = rest.split(' ');
            let name = parts.next().unwrap().to_string();
            (name, (parts.next().unwrap().to_string(), parts.next().unwrap().to_string()))
        })
        .collect()
}

/// A synthetic tree with one file under every prefix the graph reads, and one
/// under a path no obligation reads.
const TREE: &str = "tree 100644:a001 Cargo.lock
tree 100644:a002 Cargo.toml
tree 100644:a003 Justfile
tree 100644:a004 app/src/Routes.lex.tex
tree 100644:a005 crates/model/src/lib.rs
tree 100644:a006 crates/uorust/core/frontier.toml
tree 100644:a007 crates/uorust/tests/fixtures/advised/Cargo.toml
tree 100644:a008 crates/uorust/tests/fixtures/mir-path/lift-verdicts.toml
tree 100644:a009 crates/uorust/tests/fixtures/thermostat/src/lib.rs
tree 100644:a010 deny.toml
tree 100644:a011 frontier.toml
tree 100644:a012 lift-verdicts.toml
tree 100644:a013 model/ids.toml
tree 100644:a014 prism-factory/graph.toml
tree 100644:a015 prism-factory/sdk.sh
tree 100644:a016 prism-factory/written.sh
tree 100644:a017 prismpm.lock
tree 100644:a018 rustfmt.toml
tree 100644:a019 site/index.html
tree 100644:a020 clippy.toml
";

fn changed(path: &str) -> String {
    let line = TREE.lines().find(|line| line.ends_with(&format!(" {path}"))).expect("path in TREE");
    TREE.replace(line, &line.replace("100644:a", "100644:b"))
}

/// The obligations whose identity differs between the two plans.
fn reached(before: &str, after: &str) -> BTreeSet<String> {
    let (before, after) = (routes(before), routes(after));
    before.iter().filter(|(name, (_, kappa))| after[*name].1 != *kappa).map(|(name, _)| name.clone()).collect()
}

fn names(list: &[&str]) -> BTreeSet<String> {
    list.iter().map(|name| name.to_string()).collect()
}

#[test]
fn every_identity_matches_an_independent_sha256() {
    let graph = Command::new("python3").args(["-I", "../graph.py", "runs"]).output().expect("python3");
    assert!(graph.status.success(), "{}", String::from_utf8_lossy(&graph.stderr));
    let runs = std::env::temp_dir().join(format!("prism-factory-runs-{}.jsonl", std::process::id()));
    std::fs::write(&runs, &graph.stdout).unwrap();
    let mut oracle = Command::new("python3")
        .args(["-I", "tests/identity.py", runs.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("python3");
    oracle.stdin.take().unwrap().write_all(TREE.as_bytes()).unwrap();
    let out = oracle.wait_with_output().unwrap();
    std::fs::remove_file(&runs).ok();
    assert!(out.status.success());
    let expected: BTreeMap<String, String> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|line| {
            let (name, kappa) = line.split_once(' ').unwrap();
            (name.to_string(), kappa.to_string())
        })
        .collect();
    let plan = routes(&planned(TREE));
    assert_eq!(plan.len(), expected.len(), "the plan and the graph name the same obligations");
    for (name, kappa) in &expected {
        assert_eq!(&plan[name].1, kappa, "{name}");
    }
}

#[test]
fn a_file_no_obligation_declares_reaches_nothing() {
    assert!(reached(&planned(TREE), &planned(&changed("site/index.html"))).is_empty());
}

#[test]
fn a_change_reaches_exactly_the_obligations_that_declare_it_and_those_that_need_them() {
    let before = planned(TREE);
    // rustfmt.toml: only the formatting part, and the gate that needs it.
    assert_eq!(reached(&before, &planned(&changed("rustfmt.toml"))), names(&["fmt-check", "vv"]));
    // deny.toml: only the dependency audit, and the gate.
    assert_eq!(reached(&before, &planned(&changed("deny.toml"))), names(&["deny", "vv"]));
    // The app's sources: the app, the self model that renders from it, the suite that reads it, the gate.
    assert_eq!(reached(&before, &planned(&changed("app/src/Routes.lex.tex"))), names(&["app", "self-model-check", "test", "vv"]));
    // The graph's source: the factory checking itself, here and against
    // upstream, and the suite whose UE-20 checks the component, so the gate.
    assert_eq!(reached(&before, &planned(&changed("prism-factory/graph.toml"))), names(&["factory-model", "factory-upstream", "test", "vv"]));
    // A fixture's verdicts: the writer that owns them; the workspace's own projection reads every crate.
    assert_eq!(
        reached(&before, &planned(&changed("crates/uorust/tests/fixtures/mir-path/lift-verdicts.toml"))),
        names(&["prove-fixture", "uorust", "prove", "certified", "test", "model", "template-check", "vv"])
    );
}

#[test]
fn receipts_route_by_kind_and_the_gate_waits_for_its_parts() {
    let first = routes(&planned(TREE));
    assert_eq!(first["vv"].0, "blocked");
    assert_eq!(first["fmt-check"].0, "observe");
    let receipt = |name: &str, verdict: &str| format!("receipt {} {verdict}\n", first[name].1);
    let failed = routes(&planned(&format!("{TREE}{}{}", receipt("fmt-check", "fail"), receipt("deny", "fail"))));
    // A hermetic failure on the same inputs is refuted and not run again; a
    // fetch reaches the network, so its failure is observed again.
    assert_eq!(failed["fmt-check"].0, "refuted");
    assert_eq!(failed["deny"].0, "observe");
    let parts: String = ["fmt-check", "model", "template-check", "self-model-check", "lint", "test", "deny", "app", "uorust"]
        .iter()
        .map(|name| receipt(name, "pass"))
        .collect();
    let green = routes(&planned(&format!("{TREE}{parts}")));
    assert_eq!(green["fmt-check"].0, "proved");
    assert_eq!(green["vv"].0, "observe");
    assert_eq!(green["certified"].0, "blocked", "the export waits for the verdicts it reads");
}

#[test]
fn with_every_receipt_the_plan_schedules_nothing() {
    let first = routes(&planned(TREE));
    let all: String = first.values().map(|(_, kappa)| format!("receipt {kappa} pass\n")).collect();
    let plan = planned(&format!("{TREE}{all}"));
    assert!(routes(&plan).values().all(|(route, _)| route == "proved"));
    assert!(!plan.lines().any(|line| line.starts_with("schedule ")));
    assert!(plan.contains("releasable yes"));
}

#[test]
fn only_observed_obligations_carry_their_files() {
    let plan = planned(TREE);
    assert!(plan.contains("file fmt-check rustfmt.toml\n"));
    assert!(!plan.contains("file fmt-check deny.toml\n"));
    assert!(!plan.contains("file fmt-check crates/uorust/core/frontier.toml\n"), "an exclusion keeps a register out");
    assert!(!plan.lines().any(|line| line.starts_with("file vv ")), "a blocked obligation carries no files");
}

#[test]
fn reweighing_raises_failures_and_lowers_passes() {
    let out = String::from_utf8(reweigh(b"weight test 4\noutcome test fail\noutcome lint pass\n").unwrap()).unwrap();
    assert!(out.contains("weight test 6\n"));
    assert!(out.contains("weight lint 3\n"));
    assert!(out.contains("weight vv 4\n"));
}

// Class: the factory's closure (refute-factory-6: UE-18 read the parent from
// history no identity covered). The base enters as `@base:` entries, and only
// the obligation that declares one reads it: `test`, for the register and the
// manifest UE-18 compares. One tree under two parents is two identities of
// `test` and one of everything else.
const CLOSURE_BASE: &str = "tree 100644:c001 @base:frontier.toml
tree 100644:c002 @base:crates/certified/manifest.toml
tree 100644:c003 @base:Cargo.toml
";

fn closure_with_base(base: &str) -> String {
    format!("{TREE}{base}")
}

/// The oracle's identities (tests/identity.py over `graph.py runs`) for a request's tree.
fn closure_oracle(tree: &str) -> BTreeMap<String, String> {
    let graph = Command::new("python3").args(["-I", "../graph.py", "runs"]).output().expect("python3");
    assert!(graph.status.success(), "{}", String::from_utf8_lossy(&graph.stderr));
    let runs = std::env::temp_dir().join(format!("prism-factory-closure-runs-{}.jsonl", std::process::id()));
    std::fs::write(&runs, &graph.stdout).unwrap();
    let mut oracle = Command::new("python3")
        .args(["-I", "tests/identity.py", runs.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("python3");
    oracle.stdin.take().unwrap().write_all(tree.as_bytes()).unwrap();
    let out = oracle.wait_with_output().unwrap();
    std::fs::remove_file(&runs).ok();
    assert!(out.status.success());
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|line| {
            let (name, kappa) = line.split_once(' ').unwrap();
            (name.to_string(), kappa.to_string())
        })
        .collect()
}

#[test]
fn the_parent_reaches_only_the_obligation_that_declares_its_entries() {
    let before = planned(&closure_with_base(CLOSURE_BASE));
    let register = planned(&closure_with_base(&CLOSURE_BASE.replace("c001", "d001")));
    let manifest = planned(&closure_with_base(&CLOSURE_BASE.replace("c002", "d002")));
    let other = planned(&closure_with_base(&CLOSURE_BASE.replace("c003", "d003")));
    assert_eq!(reached(&before, &register), names(&["test", "vv"]));
    assert_eq!(reached(&before, &manifest), names(&["test", "vv"]));
    assert!(reached(&before, &other).is_empty(), "a base entry no obligation declares reaches nothing");
    // Without a parent at all, test is another identity again (its closure lacks the base).
    assert_eq!(reached(&before, &planned(TREE)), names(&["test", "vv"]));
    assert!(before.contains("file test @base:frontier.toml\n"));
    assert!(before.contains("file test @base:crates/certified/manifest.toml\n"));
    assert!(
        !before.lines().any(|line| line.starts_with("file ") && line.contains(" @base:") && !line.starts_with("file test ")),
        "only test reads the base"
    );
}

#[test]
fn the_identities_with_a_base_match_an_independent_sha256() {
    let tree = closure_with_base(CLOSURE_BASE);
    let plan = routes(&planned(&tree));
    let expected = closure_oracle(&tree);
    assert_eq!(plan.len(), expected.len());
    for (name, kappa) in &expected {
        assert_eq!(&plan[name].1, kappa, "{name}");
    }
}
