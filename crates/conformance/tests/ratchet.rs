//! UE-18: the hand-written translator may only shrink.
//!
//! The translator (uorust::mir, lift, lex) runs certified where `lexlean
//! production` exported its lifted definitions (crates/certified). A
//! translator function the manifest does not certify is hand-written
//! surface; a commit under which more of them are uncertified than under its
//! parent is refused, naming each new one with its cause. Both the register
//! and the manifest are written only by their generators (generate.yml's `just
//! register`, certified.yml's `just certified`), so their committed states are
//! generated states. The base is the parent commit, or HEAD when the work
//! tree changes either file (a commit in the making). There is no bypass.

use std::path::{Path, PathBuf};
use std::process::Command;

use uorust::certify::{ratchet, Manifest};
use uorust::register::Register;

const REGISTER: &str = "frontier.toml";
const MANIFEST: &str = "crates/certified/manifest.toml";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/conformance is two below the root")
        .to_path_buf()
}

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The base revision: HEAD when the work tree changes the register or the
/// manifest, else HEAD's first parent; none in a checkout without history.
fn base(root: &Path) -> Option<String> {
    let dirty = git(root, &["status", "--porcelain", "--", REGISTER, MANIFEST])?;
    let rev = if dirty.trim().is_empty() {
        "HEAD~1"
    } else {
        "HEAD"
    };
    git(root, &["rev-parse", "--verify", "--quiet", rev]).map(|r| r.trim().to_string())
}

fn manifest_at(root: &Path, text: Option<String>) -> Manifest {
    match text {
        Some(text) => Manifest::parse(&root.join(MANIFEST), &text).expect("the manifest parses"),
        None => Manifest::default(),
    }
}

/// Whether the base must be there. CI checks out the whole history
/// (fetch-depth: 0). The Prism factory runs this suite in a closure that is its
/// own repository, whose parent commit holds the base entries `test` declares
/// (`@base:frontier.toml`, `@base:crates/certified/manifest.toml`), and its
/// host names the obligation (`FACTORY_KAPPA`). In either place, a missing base
/// means the comparison cannot be made, so UE-18 fails instead of passing
/// undecided.
fn base_required() -> bool {
    std::env::var_os("CI").is_some() || std::env::var_os("FACTORY_KAPPA").is_some()
}

/// UE-18 at `root`: refused when more translator functions are uncertified
/// than under the base. Without a base it passes only where none is
/// `required` (a checkout without history, outside CI and the factory).
fn ue_18(root: &Path, required: bool) -> Result<(), String> {
    let head_text = std::fs::read_to_string(root.join(REGISTER)).expect("frontier.toml");
    let head = Register::parse(&root.join(REGISTER), &head_text).expect("the register parses");
    let head_manifest = manifest_at(root, std::fs::read_to_string(root.join(MANIFEST)).ok());
    let Some(rev) = base(root) else {
        if required {
            return Err(
                "UE-18 has no base to compare: CI must check out the parent commit, \
                 and a factory closure must declare @base:frontier.toml and \
                 @base:crates/certified/manifest.toml"
                    .into(),
            );
        }
        return Ok(());
    };
    let Some(base_text) = git(root, &["show", &format!("{rev}:{REGISTER}")]) else {
        if required {
            return Err(format!(
                "UE-18: the base {rev} holds no {REGISTER} to compare"
            ));
        }
        return Ok(());
    };
    let base = Register::parse(Path::new(REGISTER), &base_text).expect("the base register parses");
    let base_manifest = manifest_at(root, git(root, &["show", &format!("{rev}:{MANIFEST}")]));
    ratchet(&head, &head_manifest, &base, &base_manifest)
        .map(|_| ())
        .map_err(|refusal| format!("UE-18 against {rev}: {refusal}"))
}

#[test]
fn the_hand_written_translator_only_shrinks_ue_18() {
    if let Err(refusal) = ue_18(&root(), base_required()) {
        panic!("{refusal}");
    }
}

// The planted defect: the head's register with one more uncertified
// translator function than the base's is refused, the new row named with its
// cause; the base against itself passes.
#[test]
fn a_grown_translator_is_refused_ue_18() {
    let root = root();
    let text = std::fs::read_to_string(root.join(REGISTER)).expect("frontier.toml");
    let base = Register::parse(&root.join(REGISTER), &text).expect("the register parses");
    let manifest = manifest_at(&root, std::fs::read_to_string(root.join(MANIFEST)).ok());
    assert!(ratchet(&base, &manifest, &base, &manifest).is_ok());
    let planted = format!(
        "{text}\n[[residue]]\nkappa = \"{}\"\nfunction = \"uorust::mir::lower::planted\"\nreason = \"mir\"\ndetail = \"mir::Rvalue::Use\"\n",
        "0".repeat(64)
    );
    let grown =
        Register::parse(Path::new(REGISTER), &planted).expect("the planted register parses");
    let refusal =
        ratchet(&grown, &manifest, &base, &manifest).expect_err("a grown translator is refused");
    let message = refusal.to_string();
    assert!(
        message.contains("uorust::mir::lower::planted: residue (mir): mir::Rvalue::Use"),
        "{message}"
    );
}

// Class: the factory's closure (refute-factory-6, UE-18 read the parent from
// history the identity did not cover). In a closure the base is a declared
// input; a closure without it fails UE-18 instead of skipping it, and a closure
// whose parent commit holds the base decides against it.

/// A fresh directory holding the head's register and manifest; with `parent`,
/// a repository of two commits whose first holds `parent` as the register.
fn closure_ue_18(label: &str, parent: Option<&str>, head: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uorust-ue18-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("crates/certified")).expect("scratch");
    let manifest = std::fs::read_to_string(root().join(MANIFEST)).unwrap_or_default();
    std::fs::write(dir.join(MANIFEST), &manifest).expect("manifest");
    let commit = |message: &str| {
        for args in [
            vec!["add", "--all"],
            vec![
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--quiet",
                "--allow-empty",
                "-m",
                message,
            ],
        ] {
            assert!(git(&dir, &args).is_some(), "git {args:?}");
        }
    };
    if let Some(parent) = parent {
        assert!(git(&dir, &["init", "--quiet"]).is_some());
        std::fs::write(dir.join(REGISTER), parent).expect("base register");
        commit("base");
        std::fs::write(dir.join(REGISTER), head).expect("register");
        commit("closure");
    } else {
        std::fs::write(dir.join(REGISTER), head).expect("register");
    }
    dir
}

#[test]
fn a_closure_without_its_base_fails_ue_18_and_one_with_it_decides() {
    let text = std::fs::read_to_string(root().join(REGISTER)).expect("frontier.toml");
    let grown = format!(
        "{text}\n[[residue]]\nkappa = \"{}\"\nfunction = \"uorust::mir::lower::planted\"\nreason = \"mir\"\ndetail = \"mir::Rvalue::Use\"\n",
        "0".repeat(64)
    );
    // No history: undecided, so it fails where a base is required and passes elsewhere.
    let bare = closure_ue_18("bare", None, &grown);
    let refusal = ue_18(&bare, true).expect_err("a required base that is absent fails");
    assert!(refusal.contains("@base:frontier.toml"), "{refusal}");
    assert!(ue_18(&bare, false).is_ok());
    // The parent holds the base: the same grown head is refused against it,
    // and an unchanged head passes.
    let grew = closure_ue_18("grew", Some(&text), &grown);
    let refusal = ue_18(&grew, true).expect_err("a grown translator is refused against the base");
    assert!(refusal.contains("uorust::mir::lower::planted"), "{refusal}");
    let same = closure_ue_18("same", Some(&text), &text);
    assert!(ue_18(&same, true).is_ok());
    for dir in [bare, grew, same] {
        let _ = std::fs::remove_dir_all(dir);
    }
}
