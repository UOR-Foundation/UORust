//! `prism-factory`: the command-line host of the Prism factory.
//!
//! The host reads a Git tree and a receipt store, asks the certified `plan`
//! root what to do, runs the obligations the plan routes to `observe` in this
//! environment, and records what happened. It never decides a route, an
//! identity, an order or releasability itself.
//!
//! This is upstream's host (PrismPM `prism-factory/host/src/main.rs` at the
//! revision `prism-factory/upstream.toml` names), adapted:
//! - the release and forge commands are left out, because the tool installs
//!   only the `plan` and `reweigh` roots;
//! - `run` gives every closure its base and its own repository the way
//!   upstream's `forge.rs` does for a lent runner (`base`, `entries_with_base`,
//!   `closure_repository`, `git_with_input`, `git_fixed`, `git_indexed` are
//!   upstream's code from there), so a ratchet's base is a declared input;
//! - two additions are the tool's own: `interrupted` (a command stopped by a
//!   signal reached no verdict and leaves no receipt) and
//!   `ancestor_configuration` (a work directory under a cargo or rustup
//!   configuration is refused, since every closure under it would read it).
//!
//! Every other function is upstream's.

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus, Stdio};

/// The exit of `run` when an obligation's command was stopped by a signal: no
/// receipt was written for it, and the next run runs it again.
const INTERRUPTED: u8 = 75;


const USAGE: &str = "usage: prism-factory <command> [options]

commands:
  request           print the plan request for a revision (tree, contexts, receipts, weights)
  plan              print the certified plan for a revision
  run               run every obligation the plan routes to `observe` on this host
                    (exit 1: a verdict failed; 75: a command was stopped by a signal, no receipt)
  plan-request      print the certified plan for a request on stdin (as `request` prints it)
  reweigh           print the next weights for `weight` and `outcome` lines on stdin

options:
  --rev REV          the revision to plan (default HEAD)
  --context KEY=VAL  a context input (`@KEY`) only the obligations that declare it read (repeatable)
  --receipts DIR     the receipt store (default target/factory/receipts)
  --only NAME        run only these obligations (repeatable)
  --rounds N         at most N plan/run rounds (default 32)
  --work DIR         scratch for closures, outside any repository (default $TMPDIR/prism-factory)";

pub(crate) struct Options {
    revision: String,
    contexts: BTreeMap<String, String>,
    receipts: PathBuf,
    only: Vec<String>,
    rounds: usize,
    work: PathBuf,
}

fn options(arguments: &[String]) -> Result<Options, String> {
    let mut parsed = Options {
        revision: "HEAD".into(),
        contexts: BTreeMap::new(),
        receipts: PathBuf::from("target/factory/receipts"),
        only: Vec::new(),
        rounds: 32,
        work: std::env::temp_dir().join("prism-factory"),
    };
    let mut items = arguments.iter();
    while let Some(flag) = items.next() {
        let mut value = || items.next().cloned().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--rev" => parsed.revision = value()?,
            "--context" => {
                let pair = value()?;
                let (key, val) = pair.split_once('=').ok_or("--context needs KEY=VALUE")?;
                let plain = |text: &str| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_graphic());
                if !plain(key) || !plain(val) || key.contains('/') {
                    return Err(format!("--context {pair}: key and value must be non-empty printable words"));
                }
                parsed.contexts.insert(key.into(), val.into());
            }
            "--receipts" => parsed.receipts = PathBuf::from(value()?),
            "--only" => parsed.only.push(value()?),
            "--rounds" => parsed.rounds = value()?.parse().map_err(|_| "--rounds needs a number")?,
            "--work" => parsed.work = PathBuf::from(value()?),
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(parsed)
}

pub(crate) fn git(arguments: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .args(arguments)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|error| format!("git: {error}"))?;
    if !out.status.success() {
        return Err(format!("git {} failed", arguments.join(" ")));
    }
    Ok(out.stdout)
}

/// One tracked entry: its path, and `mode:object` as its content address.
pub(crate) struct Entry {
    pub(crate) path: String,
    pub(crate) object: String,
}

pub(crate) fn tree(revision: &str) -> Result<Vec<Entry>, String> {
    let listing = git(&["ls-tree", "-r", "-z", "--full-tree", revision])?;
    let mut entries = Vec::new();
    for row in listing.split(|byte| *byte == 0).filter(|row| !row.is_empty()) {
        let row = std::str::from_utf8(row).map_err(|_| "a tracked path is not UTF-8")?;
        let (meta, path) = row.split_once('\t').ok_or("unexpected ls-tree row")?;
        let fields: Vec<&str> = meta.split(' ').collect();
        if fields.len() != 3 || path.contains('\n') {
            return Err(format!("unsupported tree entry {path:?}"));
        }
        entries.push(Entry { path: path.to_string(), object: format!("{}:{}", fields[0], fields[2]) });
    }
    Ok(entries)
}

fn receipt_lines(store: &Path) -> Result<String, String> {
    let mut out = String::new();
    if let Ok(listing) = fs::read_dir(store) {
        let mut names: Vec<PathBuf> = listing.filter_map(|item| item.ok().map(|item| item.path())).collect();
        names.sort();
        for path in names.iter().filter(|path| path.extension().is_some_and(|ext| ext == "receipt")) {
            let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
            if let Some(first) = text.lines().next().filter(|line| line.starts_with("receipt ")) {
                out.push_str(first);
                out.push('\n');
            }
        }
        if let Ok(weights) = fs::read_to_string(store.join("weights")) {
            for line in weights.lines().filter(|line| line.starts_with("weight ")) {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    Ok(out)
}

/// The base of a revision is its first parent: for a pull request's merge
/// commit, the tip of the branch it merges into; for a push, the previous tip.
/// A check that compares the tree with its base (a ratchet) declares
/// `@base:PATH`, and reads the base's entry at PATH and nothing else of it.
pub(crate) fn base(revision: &str) -> Result<Option<String>, String> {
    let commit = git(&["cat-file", "commit", revision])?;
    let header = String::from_utf8_lossy(&commit);
    let Some(parent) = header.lines().take_while(|line| !line.is_empty()).find_map(|line| line.strip_prefix("parent ")) else {
        return Ok(None);
    };
    let parent = parent.trim().to_string();
    if git(&["cat-file", "-e", &format!("{parent}^{{tree}}")]).is_err() {
        return Err(format!("the base {parent} of {revision} is not here; fetch it (git fetch --depth=1 origin {parent})"));
    }
    Ok(Some(parent))
}

/// Every entry of the tree, then every entry of its base under `@base:`.
fn entries_with_base(revision: &str) -> Result<Vec<Entry>, String> {
    let mut entries = tree(revision)?;
    if let Some(parent) = base(revision)? {
        for entry in tree(&parent)? {
            entries.push(Entry { path: format!("@base:{}", entry.path), object: entry.object });
        }
    }
    Ok(entries)
}

pub(crate) fn request(options: &Options) -> Result<(String, Vec<Entry>), String> {
    let entries = entries_with_base(&options.revision)?;
    let mut text = String::new();
    for entry in &entries {
        text.push_str(&format!("tree {} {}\n", entry.object, entry.path));
    }
    // A context is not a file: it enters only the obligations that declare
    // `@key` (the model never lets a plain prefix, even "", reach it).
    for (key, value) in &options.contexts {
        text.push_str(&format!("tree context:{value} @{key}\n"));
    }
    text.push_str(&receipt_lines(&options.receipts)?);
    Ok((text, entries))
}

pub(crate) fn plan_text(request: &str) -> Result<String, String> {
    let out = prism_factory_host::plan(request.as_bytes()).map_err(|_| "the plan root overflowed")?;
    String::from_utf8(out).map_err(|_| "the plan is not UTF-8".into())
}

/// The plan, read back into what the host needs to act on it.
#[derive(Default)]
pub(crate) struct Plan {
    pub(crate) routes: BTreeMap<String, (String, String)>,
    pub(crate) order: Vec<String>,
    environment: BTreeMap<String, String>,
    run: BTreeMap<String, String>,
    files: BTreeMap<String, Vec<String>>,
    schedule: Vec<String>,
    releasable: bool,
}

pub(crate) fn read_plan(text: &str) -> Plan {
    let mut plan = Plan::default();
    for line in text.lines() {
        let mut parts = line.splitn(3, ' ');
        let (tag, owner, rest) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""), parts.next().unwrap_or(""));
        match tag {
            "obligation" => {
                let (route, kappa) = rest.split_once(' ').unwrap_or((rest, ""));
                plan.routes.insert(owner.into(), (route.into(), kappa.into()));
                plan.order.push(owner.into());
            }
            "environment" => {
                plan.environment.insert(owner.into(), rest.into());
            }
            "run" => {
                plan.run.insert(owner.into(), rest.into());
            }
            "file" => plan.files.entry(owner.into()).or_default().push(rest.into()),
            "schedule" => plan.schedule.push(owner.into()),
            "releasable" => plan.releasable = owner == "yes",
            _ => {}
        }
    }
    plan
}

/// Writes exactly the obligation's closure into a fresh directory.
pub(crate) fn materialize(entries: &[Entry], files: &[String], destination: &Path) -> Result<(), String> {
    if destination.exists() {
        fs::remove_dir_all(destination).map_err(|error| format!("{}: {error}", destination.display()))?;
    }
    fs::create_dir_all(destination).map_err(|error| format!("{}: {error}", destination.display()))?;
    let by_path: BTreeMap<&str, &str> = entries.iter().map(|entry| (entry.path.as_str(), entry.object.as_str())).collect();
    let mut batch = Command::new("git")
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| format!("git cat-file: {error}"))?;
    let mut input = batch.stdin.take().ok_or("git cat-file stdin")?;
    let mut output = BufReader::new(batch.stdout.take().ok_or("git cat-file stdout")?);
    for path in files {
        let object = by_path.get(path.as_str()).ok_or(format!("{path} is not in the tree"))?;
        let (mode, oid) = object.split_once(':').ok_or("bad object")?;
        if mode == "160000" {
            continue;
        }
        writeln!(input, "{oid}").map_err(|error| error.to_string())?;
        input.flush().map_err(|error| error.to_string())?;
        let mut header = String::new();
        output.read_line(&mut header).map_err(|error| error.to_string())?;
        let size: usize = header.trim_end().rsplit(' ').next().and_then(|size| size.parse().ok()).ok_or(format!("git cat-file: {header}"))?;
        let mut content = vec![0; size + 1];
        output.read_exact(&mut content).map_err(|error| error.to_string())?;
        content.pop();
        let target = destination.join(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        if mode == "120000" {
            #[cfg(unix)]
            std::os::unix::fs::symlink(String::from_utf8_lossy(&content).as_ref(), &target).map_err(|error| error.to_string())?;
        } else {
            fs::write(&target, &content).map_err(|error| error.to_string())?;
            #[cfg(unix)]
            if mode == "100755" {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).map_err(|error| error.to_string())?;
            }
        }
    }
    drop(input);
    batch.wait().map_err(|error| error.to_string())?;
    Ok(())
}

fn git_with_input(arguments: &[&str], input: &[u8]) -> Result<Vec<u8>, String> {
    let mut child = Command::new("git").args(arguments).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().map_err(|error| format!("git: {error}"))?;
    child.stdin.take().ok_or("git stdin")?.write_all(input).map_err(|error| error.to_string())?;
    let out = child.wait_with_output().map_err(|error| error.to_string())?;
    if !out.status.success() {
        return Err(format!("git {} failed", arguments.join(" ")));
    }
    Ok(out.stdout)
}

/// A closure commit is a function of the closure alone: its dates are fixed, so
/// an image labelled with the closure's revision is reproducible from κ.
fn git_fixed(arguments: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .env("GIT_AUTHOR_DATE", "1970-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "1970-01-01T00:00:00Z")
        .args(arguments)
        .output()
        .map_err(|error| format!("git: {error}"))?;
    if !out.status.success() {
        return Err(format!("git {} failed: {}", arguments.join(" "), String::from_utf8_lossy(&out.stderr)));
    }
    Ok(out.stdout)
}

fn git_indexed(index: &Path, arguments: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git").env("GIT_INDEX_FILE", index).args(arguments).output().map_err(|error| format!("git: {error}"))?;
    if !out.status.success() {
        return Err(format!("git {} failed: {}", arguments.join(" "), String::from_utf8_lossy(&out.stderr)));
    }
    Ok(out.stdout)
}

/// The closure gets its own repository, whose one commit tracks exactly the
/// closure, so no Git command in it can reach the checkout around it and
/// `git ls-files` lists exactly what the identity covers. When the closure
/// reads its base, that commit's parent holds exactly the base entries it
/// declared, named `origin/main` as a full checkout names the base.
/// (Upstream's `forge.rs`, `prepare`, from `git init` to the closure commit.)
fn closure_repository(entries: &[Entry], based: &[String], work: &Path, kappa: &str) -> Result<(), String> {
    let at = work.to_string_lossy();
    git(&["init", "--quiet", &at])?;
    let identity = ["-c", "user.name=prism-factory", "-c", "user.email=prism-factory@users.noreply.github.com", "-c", "commit.gpgsign=false"];
    if !based.is_empty() {
        let by_path: BTreeMap<&str, &str> = entries.iter().map(|entry| (entry.path.as_str(), entry.object.as_str())).collect();
        // GIT_INDEX_FILE resolves against the directory `-C` enters, so name it absolutely.
        let index = fs::canonicalize(work).map_err(|error| format!("{}: {error}", work.display()))?.join(".git/prism-base-index");
        let _ = fs::remove_file(&index);
        for path in based {
            let object = by_path.get(path.as_str()).ok_or(format!("{path} is not in the base"))?;
            let (mode, oid) = object.split_once(':').ok_or("bad object")?;
            let blob = git(&["cat-file", "blob", oid])?;
            let stored = String::from_utf8(git_with_input(&["-C", &at, "hash-object", "-w", "--stdin"], &blob)?).map_err(|_| "git hash-object")?;
            git_indexed(&index, &["-C", &at, "update-index", "--add", "--cacheinfo", &format!("{mode},{},{}", stored.trim(), &path["@base:".len()..])])?;
        }
        let tree = String::from_utf8(git_indexed(&index, &["-C", &at, "write-tree"])?).map_err(|_| "git write-tree")?;
        let _ = fs::remove_file(&index);
        let mut arguments = vec!["-C", &*at];
        arguments.extend(identity);
        let message = format!("base of closure {kappa}");
        arguments.extend(["commit-tree", tree.trim(), "-m", &message]);
        let commit = String::from_utf8(git_fixed(&arguments)?).map_err(|_| "git commit-tree")?;
        git(&["-C", &at, "update-ref", "refs/remotes/origin/main", commit.trim()])?;
        git(&["-C", &at, "update-ref", "HEAD", commit.trim()])?;
    }
    git(&["-C", &at, "add", "--all", "--force"])?;
    let mut arguments = vec!["-C", &*at];
    arguments.extend(identity);
    let message = format!("closure {kappa}");
    arguments.extend(["commit", "--quiet", "--allow-empty", "-m", &message]);
    git_fixed(&arguments)?;
    Ok(())
}

/// The signal that stopped an obligation's command, if one did. The tool's own.
///
/// A command stopped by a signal (an OOM kill, a killed session, an operator's
/// interrupt) reached no verdict on its inputs, so it leaves no receipt: a
/// fail receipt of a hermetic obligation would refute its inputs for good, and
/// the obligation in flight is all an interruption may lose. `bash -c` reports
/// a child stopped by signal N as exit 128 + N, so those exits count too, for
/// the signals that stop a process from outside (HUP 1, INT 2, KILL 9, TERM
/// 15). A command that crashes by its own fault under cargo (a test's SIGSEGV
/// or SIGABRT) makes cargo exit 101, and that stays a verdict.
fn interrupted(status: &ExitStatus) -> Option<i32> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return Some(signal);
        }
    }
    match status.code() {
        Some(code @ (129 | 130 | 137 | 143)) => Some(code - 128),
        _ => None,
    }
}

/// Files outside a closure that change what runs in it. The tool's own.
///
/// Cargo reads `.cargo/config.toml` (and the older `.cargo/config`) in every
/// ancestor of the directory it runs in, and rustup reads the nearest
/// `rust-toolchain.toml` (or `rust-toolchain`). Such a file above the work
/// directory would be read by every compiling obligation run under it, and no
/// identity declares it, so the work directory is refused before anything
/// runs and no receipt is written.
const ANCESTOR_CONFIGURATION: [&str; 4] = [".cargo/config.toml", ".cargo/config", "rust-toolchain.toml", "rust-toolchain"];

fn ancestor_configuration(work: &Path) -> Result<(), String> {
    fs::create_dir_all(work).map_err(|error| format!("{}: {error}", work.display()))?;
    let work = fs::canonicalize(work).map_err(|error| format!("{}: {error}", work.display()))?;
    for directory in work.ancestors() {
        for name in ANCESTOR_CONFIGURATION {
            let path = directory.join(name);
            if path.exists() {
                return Err(format!(
                    "{} would be read by every obligation run under {}, and no identity declares it; choose a --work without it",
                    path.display(),
                    work.display()
                ));
            }
        }
    }
    Ok(())
}

fn record(store: &Path, name: &str, kappa: &str, passed: bool, revision: &str) -> Result<(), String> {
    fs::create_dir_all(store).map_err(|error| format!("{}: {error}", store.display()))?;
    let verdict = if passed { "pass" } else { "fail" };
    let body = format!("receipt {kappa} {verdict}\nobligation {name}\nrevision {revision}\nenvironment host\nnetwork host\n");
    fs::write(store.join(format!("{kappa}-{verdict}.receipt")), body).map_err(|error| error.to_string())
}

pub(crate) fn reweigh(store: &Path, outcomes: &[(String, bool)]) -> Result<(), String> {
    let mut request = fs::read_to_string(store.join("weights")).unwrap_or_default();
    for (name, passed) in outcomes {
        request.push_str(&format!("outcome {name} {}\n", if *passed { "pass" } else { "fail" }));
    }
    let next = prism_factory_host::reweigh(request.as_bytes()).map_err(|_| "the reweigh root overflowed")?;
    fs::write(store.join("weights"), next).map_err(|error| error.to_string())
}

/// How a run ended: every obligation it ran reached a verdict (and whether one
/// failed), or one was stopped by a signal and left no receipt.
pub(crate) enum Finish {
    Verdicts { failed: bool },
    Interrupted,
}

fn run(options: &Options) -> Result<Finish, String> {
    let revision = String::from_utf8(git(&["rev-parse", "--verify", &format!("{}^{{commit}}", options.revision)])?)
        .map_err(|_| "revision")?
        .trim()
        .to_string();
    ancestor_configuration(&options.work)?;
    let mut attempted = std::collections::BTreeSet::new();
    let mut failed = false;
    for round in 0..options.rounds {
        let (text, entries) = request(&Options { revision: revision.clone(), ..clone(options) })?;
        let plan = read_plan(&plan_text(&text)?);
        let runnable: Vec<String> = plan
            .schedule
            .iter()
            .filter(|name| plan.environment.get(*name).map(String::as_str) == Some("host"))
            .filter(|name| options.only.is_empty() || options.only.contains(name))
            .filter(|name| !attempted.contains(*name))
            .cloned()
            .collect();
        if runnable.is_empty() {
            for (name, (route, _)) in &plan.routes {
                if route != "proved" {
                    let place = plan.environment.get(name).map(String::as_str).unwrap_or("");
                    eprintln!("factory: {name} is {route} (environment {place})");
                }
            }
            eprintln!("factory: releasable {}", if plan.releasable { "yes" } else { "no" });
            return Ok(Finish::Verdicts { failed });
        }
        let mut outcomes = Vec::new();
        for name in runnable {
            attempted.insert(name.clone());
            let kappa = plan.routes[&name].1.clone();
            let work = options.work.join(&name);
            if git(&["-C", &options.work.to_string_lossy(), "rev-parse", "--show-toplevel"]).is_ok() {
                return Err(format!("{} is inside a Git repository; a closure must not see anything else", options.work.display()));
            }
            let (based, files): (Vec<String>, Vec<String>) =
                plan.files.get(&name).cloned().unwrap_or_default().into_iter().partition(|path| path.starts_with("@base:"));
            // A context (`@key`) is part of the identity, not a file.
            let files: Vec<String> = files.into_iter().filter(|path| !path.starts_with('@')).collect();
            materialize(&entries, &files, &work)?;
            closure_repository(&entries, &based, &work, &kappa)?;
            eprintln!("factory: round {round}: running {name} ({kappa})");
            let status = Command::new("bash")
                .args(["-euo", "pipefail", "-c", &plan.run[&name]])
                .current_dir(&work)
                .env("FACTORY_OBLIGATION", &name)
                .env("FACTORY_KAPPA", &kappa)
                .env("FACTORY_REVISION", &revision)
                .status()
                .map_err(|error| format!("{name}: {error}"))?;
            if let Some(signal) = interrupted(&status) {
                eprintln!("factory: {name} interrupted by signal {signal}; no receipt is written, so the next run runs it again");
                if !outcomes.is_empty() {
                    reweigh(&options.receipts, &outcomes)?;
                }
                return Ok(Finish::Interrupted);
            }
            let passed = status.success();
            failed |= !passed;
            eprintln!("factory: {name} {}", if passed { "passed" } else { "failed" });
            record(&options.receipts, &name, &kappa, passed, &revision)?;
            outcomes.push((name, passed));
        }
        reweigh(&options.receipts, &outcomes)?;
    }
    Err(format!("no fixed point within {} rounds", options.rounds))
}

pub(crate) fn clone(options: &Options) -> Options {
    Options {
        revision: options.revision.clone(),
        contexts: options.contexts.clone(),
        receipts: options.receipts.clone(),
        only: options.only.clone(),
        rounds: options.rounds,
        work: options.work.clone(),
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = arguments.first() else {
        eprintln!("{USAGE}");
        return ExitCode::from(64);
    };
    let options = match options(&arguments[1..]) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("prism-factory: {error}\n{USAGE}");
            return ExitCode::from(64);
        }
    };
    let result: Result<bool, String> = match command.as_str() {
        "request" => request(&options).map(|(text, _)| {
            print!("{text}");
            true
        }),
        "plan" => request(&options).and_then(|(text, _)| plan_text(&text)).map(|plan| {
            print!("{plan}");
            true
        }),
        "run" => match run(&options) {
            Ok(Finish::Verdicts { failed }) => Ok(!failed),
            Ok(Finish::Interrupted) => return ExitCode::from(INTERRUPTED),
            Err(error) => Err(error),
        },
        "plan-request" => {
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text).map_err(|error| error.to_string()).and_then(|_| plan_text(&text)).map(|plan| {
                print!("{plan}");
                true
            })
        }
        "reweigh" => {
            let mut request = Vec::new();
            std::io::stdin().read_to_end(&mut request).map_err(|error| error.to_string()).and_then(|_| {
                let next = prism_factory_host::reweigh(&request).map_err(|_| "the reweigh root overflowed".to_string())?;
                std::io::stdout().write_all(&next).map(|_| true).map_err(|error| error.to_string())
            })
        }
        _ => Err(format!("unknown command {command}")),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(error) => {
            eprintln!("prism-factory: {error}");
            ExitCode::from(2)
        }
    }
}
