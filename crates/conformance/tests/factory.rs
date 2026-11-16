//! The `factory` suite: one test per scenario in `features/suites/factory.feature`.
//!
//! The routing, the identity and the graph's theorems are the certified core's
//! and the Lean kernel's (`prism-factory/check.sh`, the `factory-model`
//! obligation, and the host's own tests against Python's `hashlib`). What this
//! test holds fixed is the claim around them: the component is upstream's at
//! the recorded revision, the model text is the one graph's, and the graph
//! declares what it reads. Each part is shown to refuse its planted defect.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/conformance is two below the root")
        .to_path_buf()
}

/// `graph.py check` run on the component at `root`: its exit and its words.
fn factory_check(root: &Path) -> (bool, String) {
    let out = Command::new("python3")
        .args(["-I", "prism-factory/graph.py", "check"])
        .current_dir(root)
        .output()
        .expect("python3 runs");
    let words = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), words)
}

/// A copy of the component and the Justfile its graph reads, in a fresh
/// directory of its own.
fn factory_copy(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uorust-ue20-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
    let status = Command::new("cp")
        .args(["-r", "prism-factory", "Justfile"])
        .arg(&dir)
        .current_dir(root())
        .status()
        .expect("cp runs");
    assert!(status.success());
    dir
}

fn factory_edit(path: &Path, from: &str, to: &str) {
    let text = std::fs::read_to_string(path).expect("readable");
    assert!(text.contains(from), "{} lacks {from:?}", path.display());
    std::fs::write(path, text.replacen(from, to, 1)).expect("writable");
}

/// `UE-20`: The gate, the writers and the certified export are obligations of the upstream Prism factory installed under prism-factory/: its engine, standard library and identity protocol are upstream's at the revision prism-factory/upstream.toml names and an installed file that differs from what that record states is refused, the graph's one source is prism-factory/graph.toml with the Justfile's recipe bodies and a model text or table that differs from what graph.py writes from them is refused, no obligation reads the whole tree, one run in the SDK declares the lock that pins the image and the launcher, the certified plan routes each obligation proved, observe, refuted or blocked from receipts of its identity, a test target that names or a compiled source that embeds a tracked file its obligation does not declare is refused before the plan, each obligation runs in a closure that is its own repository whose parent holds exactly the base entries it declares, under a work directory no cargo or rustup configuration sits above, a command stopped by a signal leaves no receipt, and one run at a time holds the receipt store
#[test]
fn factory_ue_20() {
    let (passed, words) = factory_check(&root());
    assert!(
        passed,
        "the committed component fails its own check:\n{words}"
    );
    let record =
        std::fs::read_to_string(root().join("prism-factory/upstream.toml")).expect("upstream.toml");
    assert!(record.contains("path = \"model/src/Factory/Engine.lex.tex\"\nhow = \"identical\""));
    assert!(record.contains("path = \"std/src/Sha256.lex.tex\"\nhow = \"identical\""));
    let model =
        std::fs::read_to_string(root().join("prism-factory/model/src/Factory/UORust.lex.tex"))
            .expect("model");
    for theorem in [
        "no_obligation_reads_the_whole_tree",
        "every_obligation_in_the_sdk_declares_its_image_and_its_launcher",
        "every_need_is_declared_before_it_is_needed",
    ] {
        assert!(
            model.contains(&format!("  name: {theorem},\n")),
            "{theorem}"
        );
    }
    let root_module =
        std::fs::read_to_string(root().join("prism-factory/model/src/Factory.lex.tex"))
            .expect("root");
    assert!(
        root_module.contains("(Factory.Engine::planAll (Factory.UORust::graph)"),
        "the plan root plans this graph"
    );

    // An installed upstream file drifts: refused, naming the file.
    let drifted = factory_copy("drift");
    factory_edit(
        &drifted.join("prism-factory/model/src/Factory/Engine.lex.tex"),
        "\"hermetic\"",
        "\"sealed\"",
    );
    let (passed, words) = factory_check(&drifted);
    assert!(
        !passed && words.contains("model/src/Factory/Engine.lex.tex: differs"),
        "{words}"
    );

    // A recipe the graph runs changes, the model text does not: refused.
    let stale = factory_copy("stale");
    factory_edit(
        &stale.join("Justfile"),
        "cargo deny --all-features check",
        "cargo deny check",
    );
    let (passed, words) = factory_check(&stale);
    assert!(
        !passed && words.contains("UORust.lex.tex is not what graph.py writes"),
        "{words}"
    );

    // An obligation that reads the whole tree: refused before anything is written.
    let whole = factory_copy("whole");
    factory_edit(
        &whole.join("prism-factory/graph.toml"),
        "inputs = [\"deny.toml\"]",
        "inputs = [\"deny.toml\", \"\"]",
    );
    let (passed, words) = factory_check(&whole);
    assert!(
        !passed && words.contains("deny reads the whole tree"),
        "{words}"
    );

    // An SDK obligation without the lock that pins the image: refused.
    let unpinned = factory_copy("unpinned");
    factory_edit(
        &unpinned.join("prism-factory/graph.toml"),
        "sdk = [\"prismpm.lock\", \"prism-factory/sdk.sh\"]",
        "sdk = [\"prism-factory/sdk.sh\"]",
    );
    let (passed, words) = factory_check(&unpinned);
    assert!(
        !passed && words.contains("without declaring prismpm.lock"),
        "{words}"
    );

    for dir in [drifted, stale, whole, unpinned] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

// Class: the factory's closure (refute-factory-6: `test` read prism-factory/
// and the brand's stylesheet without declaring them). `graph.py reads`, run
// by the entry on a checkout before it plans, refuses an obligation whose test
// target names, or whose compiled source embeds, a tracked file the
// obligation does not declare; declaring it lifts the refusal.

/// `graph.py reads` on the component at `root`: its exit and its words.
fn closure_reads(root: &Path) -> (bool, String) {
    let out = Command::new("python3")
        .args(["-I", "prism-factory/graph.py", "reads"])
        .current_dir(root)
        .output()
        .expect("python3 runs");
    let words = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), words)
}

/// A checkout of the component with one workspace member whose test reads
/// `brand/planted.css` and whose library embeds `site/planted.txt`.
fn closure_checkout(label: &str) -> PathBuf {
    let dir = factory_copy(label);
    let files = [
        ("Cargo.toml", "[workspace]\nmembers = [\"crates/*\"]\n"),
        (
            "crates/probe/Cargo.toml",
            "[package]\nname = \"probe\"\nversion = \"0.0.0\"\n",
        ),
        (
            "crates/probe/src/lib.rs",
            "pub const PLANTED: &str = include_str!(\"../../../site/planted.txt\");\n",
        ),
        (
            "crates/probe/tests/probe.rs",
            "#[test]\nfn reads() {\n    let root = std::path::Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"../..\");\n    std::fs::read_to_string(root.join(\"brand/planted.css\")).unwrap();\n}\n",
        ),
        ("brand/planted.css", ":root {}\n"),
        ("site/planted.txt", "planted\n"),
    ];
    for (path, text) in files {
        let target = dir.join(path);
        std::fs::create_dir_all(target.parent().expect("a parent")).expect("scratch");
        std::fs::write(target, text).expect("writable");
    }
    for args in [vec!["init", "--quiet"], vec!["add", "--all"]] {
        let status = Command::new("git")
            .args(&args)
            .current_dir(&dir)
            .status()
            .expect("git runs");
        assert!(status.success(), "git {args:?}");
    }
    dir
}

#[test]
fn an_undeclared_read_of_a_test_or_an_embedded_file_is_refused_ue_20() {
    let planted = closure_checkout("reads");
    let (passed, words) = closure_reads(&planted);
    assert!(!passed, "{words}");
    assert!(
        words.contains("test reads brand/planted.css (crates/probe/tests/probe.rs:4)"),
        "{words}"
    );
    assert!(
        words.contains("lint embeds site/planted.txt (crates/probe/src/lib.rs:1)"),
        "{words}"
    );
    // Declared where it is read, nothing is refused.
    let graph = planted.join("prism-factory/graph.toml");
    factory_edit(
        &graph,
        "  \"brand/foundation/src/index.css\",\n",
        "  \"brand/foundation/src/index.css\",\n  \"brand/planted.css\",\n  \"site/planted.txt\",\n",
    );
    factory_edit(&graph, "rust = [\n", "rust = [\n  \"site/planted.txt\",\n");
    let (passed, words) = closure_reads(&planted);
    assert!(passed, "{words}");
    let _ = std::fs::remove_dir_all(planted);
}
