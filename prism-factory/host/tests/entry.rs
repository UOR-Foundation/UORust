//! The host's `run` and the entry `factory.sh` on synthetic repositories.
//!
//! Class: the factory's closure (refute-factory-6). An obligation's command
//! stopped by a signal leaves no receipt; a work directory under a cargo or
//! rustup configuration is refused; a closure is its own repository whose
//! parent holds exactly the base entries the obligation declares, so one tree
//! under two parents is two identities of `test`; a second start is refused
//! while a run holds the store's lock; and the status says `interrupted` or
//! `stale`, never `done`, for a run that did not end by itself.
//!
//! The obligations here are the tool's graph (the certified core compiles it
//! in); their commands reach `prism-factory/sdk.sh`, which each repository
//! replaces with a stub that does what `STUB` says. The entry's tests run
//! `factory.sh` with `FACTORY_HOST`, a stub host, so they hold the entry's own
//! logic and nothing of the core.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

const HOST: &str = env!("CARGO_BIN_EXE_prism-factory");

/// The stub of `prism-factory/sdk.sh`: `STUB=kill` stops itself by SIGKILL,
/// `fail` fails, `base` passes only when the closure's parent holds exactly
/// `frontier.toml` (`parent`) and the closure's own `frontier.toml` is `head`.
const ENTRY_SDK_STUB: &str = r#"#!/usr/bin/env bash
case ${STUB:-pass} in
  kill) kill -KILL $$ ;;
  fail) exit 1 ;;
  base)
    test "$(git show HEAD~1:frontier.toml)" = parent
    test "$(git ls-tree -r --name-only HEAD~1)" = "$(printf 'crates/certified/manifest.toml\nfrontier.toml')"
    test "$(cat frontier.toml)" = head
    test "$(git show origin/main:crates/certified/manifest.toml)" = manifest
    ;;
  *) exit 0 ;;
esac
"#;

fn entry_scratch(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("prism-factory-entry-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
    dir
}

fn entry_git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00Z")
        .output()
        .expect("git runs");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn entry_write(dir: &Path, files: &[(&str, &str)]) {
    for (path, text) in files {
        let target = dir.join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, text).unwrap();
    }
}

/// A repository with the files the graph's SDK obligations declare first, and
/// the stub launcher; one commit, or two when `parent` is given (the parent's
/// `frontier.toml`), the head's `frontier.toml` being `head`.
fn entry_repository(label: &str, parent: Option<&str>, head: &str) -> PathBuf {
    let dir = entry_scratch(label).join("repository");
    std::fs::create_dir_all(&dir).unwrap();
    entry_git(&dir, &["init", "--quiet"]);
    entry_write(
        &dir,
        &[
            ("prismpm.lock", "{}\n"),
            ("prism-factory/sdk.sh", ENTRY_SDK_STUB),
            ("rustfmt.toml", "\n"),
            ("crates/certified/manifest.toml", "manifest"),
        ],
    );
    if let Some(parent) = parent {
        entry_write(&dir, &[("frontier.toml", parent)]);
        entry_git(&dir, &["add", "--all"]);
        entry_git(&dir, &["commit", "--quiet", "-m", "parent"]);
    }
    entry_write(&dir, &[("frontier.toml", head)]);
    entry_git(&dir, &["add", "--all"]);
    entry_git(&dir, &["commit", "--quiet", "-m", "head"]);
    dir
}

fn entry_host(repository: &Path, store: &Path, work: &Path, stub: &str, args: &[&str]) -> Output {
    Command::new(HOST)
        .args(args)
        .args(["--receipts", store.to_str().unwrap(), "--work", work.to_str().unwrap()])
        .current_dir(repository)
        .env("STUB", stub)
        .output()
        .expect("the host runs")
}

fn entry_receipts(store: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(store)
        .map(|listing| listing.filter_map(|item| item.ok()).map(|item| item.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default();
    names.retain(|name| name.ends_with(".receipt"));
    names.sort();
    names
}

fn entry_kappa(plan: &str, name: &str) -> String {
    plan.lines()
        .find_map(|line| line.strip_prefix(&format!("obligation {name} ")))
        .and_then(|rest| rest.split(' ').nth(1))
        .expect("the obligation is planned")
        .to_string()
}

#[test]
fn a_command_stopped_by_a_signal_leaves_no_receipt_and_runs_again() {
    let repository = entry_repository("signal", None, "head");
    let scratch = repository.parent().unwrap().to_path_buf();
    let (store, work) = (scratch.join("store"), scratch.join("work"));
    let killed = entry_host(&repository, &store, &work, "kill", &["run", "--only", "fmt-check"]);
    let words = String::from_utf8_lossy(&killed.stderr);
    assert_eq!(killed.status.code(), Some(75), "{words}");
    assert!(words.contains("factory: fmt-check interrupted by signal 9"), "{words}");
    assert!(entry_receipts(&store).is_empty(), "an interruption is no verdict: {:?}", entry_receipts(&store));
    // The next run runs it again, and a verdict is recorded as ever.
    let failed = entry_host(&repository, &store, &work, "fail", &["run", "--only", "fmt-check"]);
    assert_eq!(failed.status.code(), Some(1), "{}", String::from_utf8_lossy(&failed.stderr));
    assert!(String::from_utf8_lossy(&failed.stderr).contains("running fmt-check"));
    let receipts = entry_receipts(&store);
    assert!(receipts.len() == 1 && receipts[0].ends_with("-fail.receipt"), "{receipts:?}");
    let _ = std::fs::remove_dir_all(scratch);
}

#[test]
fn a_work_directory_under_a_cargo_or_rustup_configuration_is_refused() {
    let repository = entry_repository("ancestor", None, "head");
    let scratch = repository.parent().unwrap().to_path_buf();
    let store = scratch.join("store");
    for planted in [".cargo/config.toml", "rust-toolchain.toml"] {
        let above = scratch.join("above");
        let _ = std::fs::remove_dir_all(&above);
        entry_write(&above, &[(planted, "[build]\n")]);
        let out = entry_host(&repository, &store, &above.join("work"), "pass", &["run", "--only", "fmt-check"]);
        let words = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(2), "{words}");
        assert!(words.contains(&format!("{planted} would be read by every obligation")), "{words}");
        assert!(entry_receipts(&store).is_empty());
    }
    let out = entry_host(&repository, &store, &scratch.join("work"), "pass", &["run", "--only", "fmt-check"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(entry_receipts(&store).len(), 1);
    let _ = std::fs::remove_dir_all(scratch);
}

#[test]
fn a_closure_holds_its_declared_base_and_one_tree_under_two_parents_is_two_identities() {
    let repository = entry_repository("base", Some("parent"), "head");
    let scratch = repository.parent().unwrap().to_path_buf();
    let (store, work) = (scratch.join("store"), scratch.join("work"));
    let out = entry_host(&repository, &store, &work, "base", &["run", "--only", "test"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(entry_receipts(&store)[0].ends_with("-pass.receipt"));
    // The same tree under another parent: another identity of test, the same of fmt-check.
    let tree = entry_git(&repository, &["rev-parse", "HEAD^{tree}"]);
    entry_write(&repository, &[("frontier.toml", "another parent")]);
    entry_git(&repository, &["add", "--all"]);
    let other_parent = entry_git(&repository, &["commit-tree", &entry_git(&repository, &["write-tree"]), "-m", "another parent"]);
    let twin = entry_git(&repository, &["commit-tree", &tree, "-p", &other_parent, "-m", "twin"]);
    let plan = |revision: &str| String::from_utf8(entry_host(&repository, &store, &work, "pass", &["plan", "--rev", revision]).stdout).unwrap();
    let (here, there) = (plan("HEAD"), plan(&twin));
    assert_ne!(entry_kappa(&here, "test"), entry_kappa(&there, "test"));
    assert_eq!(entry_kappa(&here, "fmt-check"), entry_kappa(&there, "fmt-check"));
    let _ = std::fs::remove_dir_all(scratch);
}

/// The stub host of the entry's tests: `plan` names one observed obligation;
/// `run` sleeps (`FAKE=sleep`), stops itself by SIGKILL (`FAKE=kill`) or passes.
const ENTRY_HOST_STUB: &str = r#"#!/usr/bin/env bash
case $1 in
  plan) echo "obligation fmt-check observe k" ;;
  run)
    case ${FAKE:-} in sleep) sleep 2 ;; kill) kill -KILL $$ ;; esac
    echo "factory: round 0: running fmt-check (k)" >&2
    echo "factory: fmt-check passed" >&2
    ;;
esac
"#;

/// A repository with the entry, its reads check and the graph it reads.
fn entry_checkout(label: &str) -> PathBuf {
    let component = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let dir = entry_scratch(label);
    let repository = dir.join("repository");
    std::fs::create_dir_all(repository.join("prism-factory")).unwrap();
    for name in ["factory.sh", "graph.py", "graph.toml"] {
        std::fs::copy(component.join(name), repository.join("prism-factory").join(name)).unwrap();
    }
    std::fs::copy(component.join("../Justfile"), repository.join("Justfile")).unwrap();
    entry_write(&repository, &[("Cargo.toml", "[workspace]\nmembers = []\n")]);
    entry_write(&dir, &[("host", ENTRY_HOST_STUB)]);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir.join("host"), std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    entry_git(&repository, &["init", "--quiet"]);
    entry_git(&repository, &["add", "--all"]);
    entry_git(&repository, &["commit", "--quiet", "-m", "entry"]);
    repository
}

fn entry_command(repository: &Path, fake: &str, args: &[&str]) -> Command {
    let dir = repository.parent().unwrap();
    let mut command = Command::new("bash");
    command
        .arg("prism-factory/factory.sh")
        .args(args)
        .current_dir(repository)
        .env("FACTORY_HOST", dir.join("host"))
        .env("FACTORY_RECEIPTS", dir.join("store"))
        .env("FACTORY_WORK", dir.join("work"))
        .env("FAKE", fake)
        .stdin(Stdio::null());
    command
}

/// The status line once the detached run has ended (the test waits; an agent does not).
fn entry_ended(repository: &Path) -> String {
    let status = repository.parent().unwrap().join("store/status");
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let line = std::fs::read_to_string(&status).unwrap_or_default();
        if !line.is_empty() && !line.starts_with("running ") {
            return line;
        }
        assert!(Instant::now() < deadline, "the detached run did not end: {line:?}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn two_starts_at_once_are_one_run_and_the_second_is_refused() {
    let repository = entry_checkout("starts");
    let first = entry_command(&repository, "sleep", &["start"]).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let second = entry_command(&repository, "sleep", &["start"]).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut codes = vec![first.wait_with_output().unwrap(), second.wait_with_output().unwrap()];
    codes.sort_by_key(|out| out.status.code());
    assert_eq!(codes[0].status.code(), Some(0), "{}", String::from_utf8_lossy(&codes[0].stderr));
    assert_eq!(codes[1].status.code(), Some(75), "{}", String::from_utf8_lossy(&codes[1].stderr));
    assert!(String::from_utf8_lossy(&codes[1].stderr).contains("a run is in progress"));
    // While the run holds the lock, a foreground run is refused too.
    let during = entry_command(&repository, "", &["run"]).output().unwrap();
    assert_eq!(during.status.code(), Some(75), "{}", String::from_utf8_lossy(&during.stderr));
    assert_eq!(entry_ended(&repository), "done 0 established 1 proved 0 refuted 0 blocked 0 observe 1\n");
    let log = std::fs::read_to_string(repository.parent().unwrap().join("store/factory.log")).unwrap();
    assert_eq!(log.matches("running fmt-check").count(), 1, "{log}");
    let _ = std::fs::remove_dir_all(repository.parent().unwrap());
}

#[test]
fn a_run_that_did_not_end_by_itself_is_interrupted_or_stale_never_done() {
    let repository = entry_checkout("ended");
    let started = entry_command(&repository, "kill", &["start"]).output().unwrap();
    assert!(started.status.success(), "{}", String::from_utf8_lossy(&started.stderr));
    let line = entry_ended(&repository);
    assert!(line.starts_with("interrupted 9 "), "{line}");
    // A `running` line whose process is gone is stale.
    let status = repository.parent().unwrap().join("store/status");
    std::fs::write(&status, "running 999999999 2000-01-01T00:00:00Z\n").unwrap();
    let out = entry_command(&repository, "", &["status"]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("stale running 999999999"), "{}", String::from_utf8_lossy(&out.stdout));
    let _ = std::fs::remove_dir_all(repository.parent().unwrap());
}
