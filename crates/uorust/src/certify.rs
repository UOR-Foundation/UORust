//! The certified translator: the lifted translator, exported through
//! `lexlean production` (SPEC §24.7) as certified Rust, and the ratchet that
//! lets the hand-written translator only shrink.
//!
//! The translator is the code of [`SCOPE`]: the MIR path (`mir`), the
//! surface lift (`lift`) and the LexLean writer (`lex`). Each of its
//! functions the self register records as lifted has a LexLean definition in
//! the self projection's `Behavior` module, and every such definition is a
//! production root: [`production`] writes the projection again with each one
//! marked executable and the request that names them, so that `lexlean
//! production build` kernel-checks a Rust package per root. [`sync`] takes
//! the published bundle into the repository (`crates/certified`): each
//! package's source byte for byte, as the bundle's result records its
//! SHA-256, and the manifest that keys every certified root by the κ of the
//! Rust function it certifies.
//!
//! A function is certified when the manifest has a root of its κ: a changed
//! function is a new question and is uncertified until the bundle is rebuilt
//! from the projection that lifts it. [`ratchet`] refuses a register and a
//! manifest under which more translator functions are uncertified than under
//! the parent's, naming each new one with its cause.
//!
//! None of it is the translator's alone. A [`Profile`] says what an export
//! is of: the scope, the entry, the package prefix, the effects and the
//! target. The constants below are the translator's own profile, the default
//! of every step, so its gate reads as before; any other workspace states
//! its own (`--workspace`, the table of its Cargo manifest, the command's
//! options) and has every lifted function in its scope exported, taken into
//! a directory it names and held by the same ratchet. Its agreement is
//! [`cases`]: the closed assertions of its own tests, each a production
//! root of its own.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::error::io;
use crate::mir::ir::{IrAggregate, IrConstant, IrOperand, IrRvalue, IrStatement, IrTerminator};
use crate::mir::{Dumps, IrBody};
use crate::register::{Register, Status};
use crate::UorustError;

pub mod cases;
mod profile;

pub use profile::{Kind, Options, Profile, PROFILE};

/// The modules of the translator, by their crate paths.
pub const SCOPE: &[&str] = &["uorust::mir::", "uorust::lift::", "uorust::lex::"];

/// The translate entry: the function that translates the bodies of one
/// module of a library, from their MIR text to their LexLean and κ rows.
pub const ENTRY: &str = "uorust::mir::core::core_shard";

/// The schema of the manifest [`sync`] writes.
pub const SCHEMA: &str = "uorust/certified/1";

/// The prefix of every certified package's name.
pub const PACKAGE: &str = "uorust_certified_";

/// The version every certified package is requested at, and the crate that
/// holds them is written at.
const PACKAGE_VERSION: &str = "0.1.0";

/// The effects every root admits: the whole closed registry of SPEC §17.13.
/// The translator builds strings and lists (allocation), computes on
/// naturals of unbounded width (overflow, an explicit failure at the
/// target's 64 bits) and walks its inputs by structural recursion
/// (recursion); the compiler states which ones each package realizes.
pub const EFFECTS: &[&str] = &["allocation", "overflow", "recursion"];

/// The target every root is exported to: the translator runs where `std`
/// does.
pub const TARGET: &str = "rust-std";

/// Whether a function is translator code.
pub fn in_scope(function: &str) -> bool {
    SCOPE.iter().any(|s| function.starts_with(s))
}

/// A production root: a lifted translator function and its definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Root {
    /// The Rust function's crate path.
    pub function: String,
    /// Its κ in the register.
    pub kappa: String,
    /// The definition in the projection's `Behavior` module, unqualified.
    pub definition: String,
    /// The prefix of its package's name, the profile's.
    pub prefix: String,
}

/// The most characters a crate's name has, as the compiler states the rule
/// (`LLC0201`: a crate name is `[a-z][a-z0-9_]{0,63}`).
const CRATE_NAME: usize = 64;

/// The hexadecimal digits of the SHA-256 of a longer name that end its
/// shortened form: a quarter of the digest, 64 bits, which no two roots of a
/// workspace share by chance; two that did would be refused by name
/// ([`Profile::export_roots`]).
const NAME_DIGEST: usize = CRATE_NAME / 4;

impl Root {
    /// The certified package's name: the profile's prefix ([`PACKAGE`], for
    /// the translator) and the definition in snake case. A name longer than
    /// a crate's may be ([`CRATE_NAME`]) keeps its start and ends with the
    /// first [`NAME_DIGEST`] digits of its own SHA-256, so it is still the
    /// root's alone and the same at every export.
    pub fn package(&self) -> String {
        let mut out = self.prefix.clone();
        for c in self.definition.chars() {
            if c.is_ascii_uppercase() {
                out.push('_');
                out.push(c.to_ascii_lowercase());
            } else {
                out.push(c);
            }
        }
        if out.chars().count() <= CRATE_NAME {
            return out;
        }
        let digest = crate::project::sha(out.as_bytes());
        let kept: String = out.chars().take(CRATE_NAME - NAME_DIGEST - 1).collect();
        format!("{kept}_{}", &digest[..NAME_DIGEST])
    }
}

/// The production roots of a register: every lifted translator function, by
/// definition.
pub fn roots(register: &Register) -> Vec<Root> {
    Profile::default().roots(register)
}

impl Profile {
    /// The function roots of a register under this profile: every lifted
    /// function in its scope, by definition.
    pub fn roots(&self, register: &Register) -> Vec<Root> {
        let mut out: Vec<Root> = register
            .rows
            .iter()
            .filter(|(_, row)| self.in_scope(&row.function))
            .filter_map(|(kappa, row)| match &row.status {
                Status::Lifted(definition) => Some(Root {
                    function: row.function.clone(),
                    kappa: kappa.clone(),
                    definition: definition
                        .rsplit("::")
                        .next()
                        .unwrap_or(definition)
                        .to_string(),
                    prefix: self.package.clone(),
                }),
                _ => None,
            })
            .collect();
        out.sort_by(|a, b| a.definition.cmp(&b.definition));
        out
    }

    /// The roots of an export under this profile: its functions, or, of a
    /// cases export, the closed assertions the projection at `projection`
    /// states over them.
    pub fn export_roots(
        &self,
        projection: &Path,
        register: &Register,
    ) -> Result<Vec<Root>, UorustError> {
        let roots: Vec<Root> = match self.kind {
            Kind::Functions => self.roots(register),
            Kind::Cases => cases::cases(self, &modules(projection)?, register)
                .into_iter()
                .map(|c| c.root)
                .collect(),
        };
        // A package is one root's: two definitions that come to one name
        // would have one certify the other's function.
        let mut packages: BTreeMap<String, &str> = BTreeMap::new();
        for root in &roots {
            if let Some(other) = packages.insert(root.package(), &root.definition) {
                return Err(UorustError::Tool(format!(
                    "the definitions {other} and {} come to one package name, {}",
                    root.definition,
                    root.package()
                )));
            }
        }
        Ok(roots)
    }
}

/// The modules of a projection a root's closure lies in, by name, with
/// their texts: `Behavior`, and the `Prelude` it imports.
pub(crate) fn modules(projection: &Path) -> Result<BTreeMap<String, String>, UorustError> {
    let mut modules = BTreeMap::new();
    for module in [BEHAVIOR, "Prelude"] {
        let path = projection.join("src").join(format!("{module}.lex.tex"));
        let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
        modules.insert(module.to_string(), text);
    }
    Ok(modules)
}

/// Write the production project: the projection at `projection` with every
/// root's definition marked as a production root, and `request.json`, the
/// `lexlean/production-request/1` document naming one package per root at
/// the `rust-std` descriptor of `targets` (the output of `lexlean production
/// targets`), restricted to the definitions of `batch` when it is given.
/// Returns the roots the request names, by definition.
pub fn production(
    profile: &Profile,
    projection: &Path,
    out: &Path,
    register: &Register,
    targets: &str,
    compiler: &BTreeMap<String, String>,
    batch: Option<&BTreeSet<String>>,
) -> Result<Vec<String>, UorustError> {
    let bad = |path: &Path, reason: String| UorustError::Malformed {
        path: path.to_path_buf(),
        reason,
    };
    let digest = profile.descriptor(targets)?;
    let prefix = module_prefix(projection)?;
    copy_tree(projection, out)?;
    let roots = profile.export_roots(projection, register)?;
    let names: BTreeSet<&str> = roots.iter().map(|r| r.definition.as_str()).collect();
    let modules = modules(out)?;
    let exported: BTreeSet<&str> = names
        .iter()
        .copied()
        .filter(|n| !compiler.contains_key(*n))
        .collect();
    let (mut marked, found, mut refused) = profile.mark(&modules, &exported);
    // The compiler generates a package for every root the source marks, so
    // a batch marks only its own roots, and their closures executable.
    if let Some(batch) = batch {
        let own: BTreeSet<&str> = exported
            .iter()
            .copied()
            .filter(|n| batch.contains(*n))
            .collect();
        marked = profile.mark(&modules, &own).0;
    }
    refused.extend(compiler.iter().map(|(k, v)| (k.clone(), v.clone())));
    if let Some(missing) = names
        .iter()
        .find(|n| !found.contains(**n) && !refused.contains_key(**n))
    {
        return Err(bad(
            &out.join("src").join("Behavior.lex.tex"),
            format!("the register's definition {missing} is not in the projection"),
        ));
    }
    for (module, text) in marked {
        let path = out.join("src").join(format!("{module}.lex.tex"));
        std::fs::write(&path, text).map_err(|e| io(&path, e))?;
    }
    let refusals: String = roots
        .iter()
        .filter_map(|r| refused.get(&r.definition).map(|c| (r, c)))
        .map(|(r, c)| {
            format!(
                "[[refused]]\nkappa = \"{}\"\ncause = {}\n\n",
                r.kappa,
                toml_quoted(c)
            )
        })
        .collect();
    let path = out.join(REFUSALS);
    std::fs::write(&path, refusals).map_err(|e| io(&path, e))?;
    let named: Vec<String> = roots
        .iter()
        .filter(|r| !refused.contains_key(&r.definition))
        .filter(|r| batch.is_none_or(|b| b.contains(&r.definition)))
        .map(|r| r.definition.clone())
        .collect();
    let mut entries: Vec<(String, serde_json::Value)> = roots
        .iter()
        .filter(|r| named.contains(&r.definition))
        .map(|r| {
            let root = format!("{prefix}.Behavior.{}", r.definition);
            let entry = serde_json::json!({
                "package": {"name": r.package(), "version": PACKAGE_VERSION},
                "root": root,
                "targets": [{"digest": digest, "id": profile.target}],
            });
            (root, entry)
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let request = serde_json::json!({
        "roots": entries.into_iter().map(|(_, e)| e).collect::<Vec<_>>(),
        "spec": "lexlean/production-request/1",
    });
    let path = out.join("request.json");
    let body = serde_json::to_string_pretty(&request).unwrap_or_default() + "\n";
    std::fs::write(&path, body).map_err(|e| io(&path, e))?;
    Ok(named)
}

/// The limit of a `lexlean` project on the bytes one build generates over all
/// its roots (`[limits] max_total_source_bytes`): a build that names it is
/// over budget as a whole, not refused for any one root.
const TOTAL_BUDGET: &str = "max_total_source_bytes";

/// The build seconds per root an export without a measured figure plans
/// with: run 43653074598 built 80-root batches in 560 to 800 seconds.
const MEASURED_SECONDS_PER_ROOT: u64 = 10;

/// The builds of its batch a shard is sized to fit before the deadline: its
/// first and three more after roots the kernel refuses late.
const BUILDS_PER_SHARD: u64 = 4;

/// The most roots a shard holds, whatever the timing: the kernel refuses
/// roots late, one per build, so a shard of this many on its two workers
/// rebuilds batches of at most half of it, and its deadline holds as many
/// such rebuilds as it has roots.
const MAX_ROOTS_PER_SHARD: usize = 8;

/// The most shards an export plans: a matrix GitHub runs at once.
const MAX_SHARDS: usize = 64;

/// The plan of a sharded export: how many shards build the roots `prior`
/// did not refuse, each sized so that [`BUILDS_PER_SHARD`] builds of it fit
/// in `budget` seconds at the prior export's seconds per root, and the roots
/// whose refusal is the compiler's own (`LLB6005`, or `LLV7015` built alone), whose full
/// diagnostics the diagnose job keeps.
pub fn plan(register: &Register, prior: &Manifest, budget: u64) -> (usize, Vec<String>) {
    plan_of(&roots(register), prior, budget)
}

/// [`plan`] over the roots of any export.
pub fn plan_of(all: &[Root], prior: &Manifest, budget: u64) -> (usize, Vec<String>) {
    let refused = |r: &&Root| {
        prior.refused.contains_key(&r.kappa) && !prior.refused[&r.kappa].starts_with(UNBUILT)
    };
    let to_build = all.iter().filter(|r| !refused(r)).count();
    let per_root = prior
        .seconds_per_root
        .unwrap_or(MEASURED_SECONDS_PER_ROOT)
        .max(1);
    let per_shard = usize::try_from(budget / (per_root * BUILDS_PER_SHARD))
        .unwrap_or(usize::MAX)
        .clamp(1, MAX_ROOTS_PER_SHARD);
    let shards = to_build.div_ceil(per_shard).clamp(1, MAX_SHARDS);
    let diagnose = all
        .iter()
        .filter(|r| {
            prior
                .refused
                .get(&r.kappa)
                .is_some_and(|c| c.starts_with(ALONE) || c.starts_with("LLB6005"))
        })
        .map(|r| r.definition.clone())
        .collect();
    (shards, diagnose)
}

/// Build `root` alone in a production project at `out` and keep, in
/// `out/build.log`, everything `lexlean production build` prints in human
/// form: a refusal's whole diagnostic, which the JSON form cuts short.
pub fn diagnose(input: &Input<'_>, out: &Path, root: &str) -> Result<(), UorustError> {
    let Input {
        profile,
        projection,
        register,
        targets,
    } = *input;
    profile.descriptor(targets)?;
    if out.exists() {
        std::fs::remove_dir_all(out).map_err(|e| io(out, e))?;
    }
    let only: BTreeSet<String> = [root.to_string()].into_iter().collect();
    let named = production(
        profile,
        projection,
        out,
        register,
        targets,
        &BTreeMap::new(),
        Some(&only),
    )?;
    if named.is_empty() {
        return Err(UorustError::Tool(format!("{root} is not an eligible root")));
    }
    lexlean(out, &["lock"])?;
    let run = std::process::Command::new("lexlean")
        .args([
            "production",
            "build",
            "--request",
            "request.json",
            "--diagnostic-format",
            "human",
            "--color",
            "never",
        ])
        .current_dir(out)
        .output()
        .map_err(|e| UorustError::Tool(format!("lexlean production build: {e}")))?;
    let log = out.join("build.log");
    let mut text = run.stdout;
    text.extend(run.stderr);
    std::fs::write(&log, text).map_err(|e| io(&log, e))
}

/// What an export reads: its profile, the projection, the projection's
/// register, and the target descriptors (the output of `lexlean production
/// targets`).
#[derive(Clone, Copy, Debug)]
pub struct Input<'a> {
    /// What the export is of.
    pub profile: &'a Profile,
    /// The projection (`uorust project <workspace> --out ...`).
    pub projection: &'a Path,
    /// Its lift register.
    pub register: &'a Register,
    /// The target descriptors.
    pub targets: &'a str,
}

/// What a check of an export's roots decided, without a build.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Admission {
    /// The roots of the export.
    pub roots: usize,
    /// Those the compiler admits, by definition.
    pub admitted: Vec<String>,
    /// Those refused, by κ, with their causes.
    pub refused: BTreeMap<String, String>,
    /// The module prefix of the projection, which starts every name of its
    /// own a diagnostic quotes.
    pub prefix: String,
}

impl Admission {
    /// The causes of the refusals, each as its class ([`class`]), with the
    /// roots it refuses: the most first.
    pub fn causes(&self) -> Vec<(String, usize)> {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for cause in self.refused.values() {
            *counts.entry(class(cause, &self.prefix)).or_default() += 1;
        }
        let mut out: Vec<(String, usize)> = counts.into_iter().collect();
        out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        out
    }
}

/// The class of a refusal's cause: its first line in the compiler's own
/// terms, without what is the project's. Each quoted name of the project
/// (`<prefix>.<Module>.<name>`) and each definition named by its module
/// (`Module::name`) is `_`, and the closing parenthesis that locates the
/// violation in the project is left out.
pub(crate) fn class(cause: &str, prefix: &str) -> String {
    let line = cause.lines().next().unwrap_or_default();
    let line = match line.rfind(" (") {
        Some(at) if line.ends_with(')') => &line[..at],
        _ => line,
    };
    let own = format!("{prefix}.");
    let mut out = String::new();
    for (i, part) in line.split('`').enumerate() {
        if i % 2 == 1 {
            let name = if part.starts_with(&own) { "_" } else { part };
            out.push_str(&format!("`{name}`"));
            continue;
        }
        let words: Vec<&str> = part
            .split(' ')
            .map(|w| if w.contains("::") { "_" } else { w })
            .collect();
        out.push_str(&words.join(" "));
    }
    out
}

/// Decide which roots of an export the compiler admits, without a build:
/// the production project at `out` with every root marked is checked by
/// `lexlean check`, which types the project and decides each root's
/// eligibility (SPEC §17.13) and runs no Lean; each root a diagnostic names
/// is refused with it and the rest are checked again, until the check
/// passes. What is left are the roots a build is asked for: the kernel's
/// own refusals are a build's to find.
pub fn admit_roots(input: &Input<'_>, out: &Path) -> Result<Admission, UorustError> {
    let Input {
        profile,
        projection,
        register,
        targets,
    } = *input;
    let all = profile.export_roots(projection, register)?;
    let prefix = format!("{}.{BEHAVIOR}.", module_prefix(projection)?);
    let mut compiler: BTreeMap<String, String> = BTreeMap::new();
    let admitted = loop {
        if out.exists() {
            std::fs::remove_dir_all(out).map_err(|e| io(out, e))?;
        }
        let named = production(profile, projection, out, register, targets, &compiler, None)?;
        if named.is_empty() {
            break named;
        }
        lexlean(out, &["lock"])?;
        let result = lexlean(out, &["check"])?;
        if result.get("success").and_then(serde_json::Value::as_bool) == Some(true) {
            break named;
        }
        let messages = diagnostics(&result);
        let refused: BTreeMap<String, String> = messages
            .iter()
            .filter_map(|m| first_root(m, &prefix).map(|r| (r, m.clone())))
            .filter(|(r, _)| named.contains(r))
            .collect();
        if refused.is_empty() {
            return Err(UorustError::Tool(format!(
                "lexlean check: {}",
                messages
                    .first()
                    .map_or("refused without a diagnostic", String::as_str)
            )));
        }
        compiler.extend(refused);
    };
    let path = out.join(REFUSALS);
    let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
    let doc: toml::Value = text.parse().map_err(|e| UorustError::Malformed {
        path: path.clone(),
        reason: format!("{e}"),
    })?;
    let refused = doc
        .get("refused")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|r| {
            Some((
                r.get("kappa")?.as_str()?.to_string(),
                r.get("cause")?.as_str()?.to_string(),
            ))
        })
        .collect();
    Ok(Admission {
        roots: all.len(),
        admitted,
        refused,
        prefix: module_prefix(projection)?,
    })
}

impl Profile {
    /// The digest of the descriptor of the profile's target ([`TARGET`], for
    /// the translator) in `targets` (the output of `lexlean production
    /// targets`).
    fn descriptor(&self, targets: &str) -> Result<String, UorustError> {
        let bad = |reason: String| UorustError::Malformed {
            path: PathBuf::from("targets.json"),
            reason,
        };
        let descriptors: serde_json::Value =
            serde_json::from_str(targets).map_err(|e| bad(e.to_string()))?;
        descriptors
            .get("targets")
            .and_then(|t| t.as_array())
            .into_iter()
            .flatten()
            .find(|t| t.get("id").and_then(|i| i.as_str()) == Some(self.target.as_str()))
            .and_then(|t| t.get("digest"))
            .and_then(|d| d.as_str())
            .map(str::to_string)
            .ok_or_else(|| bad(format!("no {} descriptor", self.target)))
    }
}

/// The file of `kept` that records the build seconds of every pass and the
/// roots its bundles published, from which the next export sizes its shards.
pub const TIMING: &str = "timing.toml";

/// The directories of a shard's kept output under a merged `kept`.
pub const SHARD: &str = "shard-";

/// The kept outputs under `kept`: each `kept/shard-<i>` when the export was
/// sharded, else `kept` itself.
fn kept_roots(kept: &Path) -> Result<Vec<PathBuf>, UorustError> {
    let mut shards: Vec<PathBuf> = std::fs::read_dir(kept)
        .map_err(|e| io(kept, e))?
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with(SHARD))
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    shards.sort();
    Ok(if shards.is_empty() {
        vec![kept.to_path_buf()]
    } else {
        shards
    })
}

/// The file of `kept` that records the κ each root's package was exported
/// at, by package.
pub const KAPPAS: &str = "kappas.toml";

/// What part of the export one run builds, and for how long.
#[derive(Clone, Copy, Debug, Default)]
pub struct Scope {
    /// The time it may build for; the roots unbuilt then are refused.
    pub within: Option<std::time::Duration>,
    /// Shard I of N: every Nth eligible root, from the Ith.
    pub shard: Option<(usize, usize)>,
}

/// The code of the kernel's rejection of a certificate B. Run 43700278692's
/// diagnose job built roots alone that a batch had refused with it, and the
/// kernel accepted them.
const KERNEL: &str = "LLV7015";

/// What a refusal's cause starts with when the root was built alone.
const ALONE: &str = "built alone: ";

/// The cause of a root no build reached before the export's deadline.
const UNBUILT: &str = "not built before the export's deadline";

/// The cores one build occupies: `lexlean production build` and the `lean`
/// it runs beside itself, each a core at full load.
const CORES_PER_BUILD: usize = 2;

/// The work [`production_build`]'s workers share: the batches still to
/// build, the workers building one, the roots refused by the compiler, the
/// bundles kept, the passes run, and the first error.
#[derive(Default)]
struct Export {
    batches: Vec<Vec<String>>,
    busy: usize,
    compiler: BTreeMap<String, String>,
    bundles: usize,
    passes: usize,
    error: Option<UorustError>,
    late: bool,
    seconds: u64,
    published: usize,
}

/// What one build of a batch asks of the export next.
enum Outcome {
    /// It published the bundle at this path.
    Bundle(PathBuf),
    /// The compiler refused these roots, with their diagnostics.
    Refused(Vec<(String, String)>),
    /// The batch is over the project's total budget, with the diagnostic.
    Over(String),
    /// The export's deadline passed before the build ended.
    Late,
}

/// Export the translator: write the production project and build its roots
/// with `lexlean production build`, in batches on parallel workers, then
/// replay each bundle with `verify-artifact` and keep under `kept` what
/// [`sync`] reads of them: each bundle's result and package sources under
/// `kept/<n>`, and the refusals.
///
/// The roots are cut into one batch per worker, a worker per
/// [`CORES_PER_BUILD`] available cores, each built in its own project under
/// `out/<worker>` that marks only that batch's roots: a build's fixed cost,
/// verifying the marked project, is paid once per batch. A root the compiler finds ineligible is refused with
/// its diagnostic and only its batch is built again without it. A batch over
/// the project's total budget ([`TOTAL_BUDGET`]) is split in two, so no root
/// is refused for the size of the others. The roots `prior` (the manifest of
/// the last export) refused, at the same κ and by the same compiler, are
/// refused again before any build, so a steady export builds each batch
/// once. Each pass is printed to stderr. When `within` passes, the builds
/// still running are stopped and the roots not yet built are refused as
/// [`UNBUILT`], so the export ends with what it published. The project and
/// the build may live on a scratch volume.
pub fn production_build(
    input: &Input<'_>,
    out: &Path,
    kept: &Path,
    prior: &Manifest,
    scope: Scope,
) -> Result<(), UorustError> {
    let Input {
        profile,
        projection,
        register,
        targets,
    } = *input;
    let Scope { within, shard } = scope;
    if let Some((k, n)) = shard.filter(|(k, n)| k >= n) {
        return Err(UorustError::Tool(format!(
            "shard {k} of {n}: a shard is below the count"
        )));
    }
    let deadline = within.map(|w| std::time::Instant::now() + w);
    let prefix = format!("{}.{BEHAVIOR}.", module_prefix(projection)?);
    let all = profile.export_roots(projection, register)?;
    // Refused before anything is written: no descriptor to export to.
    profile.descriptor(targets)?;
    let mut compiler: BTreeMap<String, String> = BTreeMap::new();
    if !prior.compiler.is_empty() && prior.compiler == compiler_version()? {
        let by_kappa: BTreeMap<&str, &str> = all
            .iter()
            .map(|r| (r.kappa.as_str(), r.definition.as_str()))
            .collect();
        compiler.extend(prior.refused.iter().filter_map(|(kappa, cause)| {
            by_kappa
                .get(kappa.as_str())
                .filter(|_| cause.contains(&format!("`{prefix}")))
                // A kernel rejection is carried only when the root was built
                // alone: in a batch it can be the batch's, not the root's.
                .filter(|_| !cause.starts_with(KERNEL))
                .map(|d| ((*d).to_string(), cause.clone()))
        }));
    }
    for dir in [out, kept] {
        if dir.exists() {
            std::fs::remove_dir_all(dir).map_err(|e| io(dir, e))?;
        }
        std::fs::create_dir_all(dir).map_err(|e| io(dir, e))?;
    }
    // The κ each package is exported at, so that a sync on a newer commit
    // takes a package only while its function is the one exported.
    let measured: String = all
        .iter()
        .map(|r| format!("{} = \"{}\"\n", r.package(), r.kappa))
        .collect();
    let path = kept.join(KAPPAS);
    std::fs::write(&path, measured).map_err(|e| io(&path, e))?;
    // What the export is of, for the steps that take it in. The
    // translator's own is every step's default and is not restated.
    if *profile != Profile::default() {
        let path = kept.join(PROFILE);
        std::fs::write(&path, profile.rows()).map_err(|e| io(&path, e))?;
    }
    // Every root the request may name: the higher-order ones are refused by
    // `production` itself.
    let probe = out.join("probe");
    let admitted = production(
        profile, projection, &probe, register, targets, &compiler, None,
    )?;
    let eligible: Vec<String> = admitted
        .into_iter()
        .enumerate()
        .filter(|(i, _)| shard.is_none_or(|(k, n)| i % n == k))
        .map(|(_, r)| r)
        .collect();
    std::fs::remove_dir_all(&probe).map_err(|e| io(&probe, e))?;
    let cores = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    let workers = (cores / CORES_PER_BUILD).max(1);
    let count = workers.min(eligible.len()).max(1);
    let mut batches: Vec<Vec<String>> = vec![Vec::new(); count];
    for (i, root) in eligible.into_iter().enumerate() {
        batches[i % count].push(root);
    }
    batches.reverse();
    eprintln!(
        "certify production: {} batches on {workers} workers, {} roots refused before any build",
        batches.len(),
        compiler.len()
    );
    let state = std::sync::Mutex::new(Export {
        batches,
        compiler,
        ..Export::default()
    });
    let wake = std::sync::Condvar::new();
    let source = Source {
        profile,
        projection,
        register,
        targets,
        prefix: &prefix,
        deadline,
    };
    std::thread::scope(|scope| {
        for worker in 0..workers {
            let (state, wake, source) = (&state, &wake, &source);
            let dir = out.join(worker.to_string());
            scope.spawn(move || loop {
                let (batch, compiler, pass) = {
                    let mut s = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    loop {
                        if s.error.is_some() || s.late || (s.batches.is_empty() && s.busy == 0) {
                            wake.notify_all();
                            return;
                        }
                        if let Some(batch) = s.batches.pop() {
                            s.busy += 1;
                            s.passes += 1;
                            break (batch, s.compiler.clone(), s.passes);
                        }
                        s = wake.wait(s).unwrap_or_else(std::sync::PoisonError::into_inner);
                    }
                };
                let started = std::time::Instant::now();
                let outcome = build_batch(source, &dir, &compiler, &batch);
                let mut s = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                s.busy -= 1;
                let seconds = started.elapsed().as_secs();
                s.seconds += seconds;
                match outcome {
                    Ok((_, None)) => {}
                    Ok((named, Some(Outcome::Bundle(bundle)))) => {
                        s.bundles += 1;
                        s.published += named.len();
                        let to = kept.join(s.bundles.to_string());
                        eprintln!(
                            "certify production: pass {pass} (worker {worker}): {} roots published in {seconds}s",
                            named.len()
                        );
                        if let Err(e) = keep(&bundle, &to) {
                            s.error = Some(e);
                        }
                    }
                    Ok((named, Some(Outcome::Refused(refused)))) => {
                        let mut rest = named.clone();
                        for (root, cause) in refused {
                            // The kernel rejects some roots' certificates in a
                            // batch that it accepts alone: such a root is
                            // built alone before it is refused.
                            if cause.starts_with(KERNEL) && named.len() > 1 {
                                eprintln!(
                                    "certify production: pass {pass} (worker {worker}): {root} rejected in a batch after {seconds}s; built alone next"
                                );
                                rest.retain(|r| *r != root);
                                s.batches.push(vec![root]);
                                continue;
                            }
                            eprintln!(
                                "certify production: pass {pass} (worker {worker}): {root} refused after {seconds}s: {}",
                                cause.lines().next().unwrap_or_default()
                            );
                            let cause = if cause.starts_with(KERNEL) {
                                format!("{ALONE}{cause}")
                            } else {
                                cause
                            };
                            s.compiler.insert(root, cause);
                        }
                        if !rest.is_empty() {
                            s.batches.push(rest);
                        }
                    }
                    Ok((named, Some(Outcome::Over(cause)))) => {
                        eprintln!(
                            "certify production: pass {pass} (worker {worker}): {} roots over budget after {seconds}s: {}",
                            named.len(),
                            cause.lines().next().unwrap_or_default()
                        );
                        if let [root] = named.as_slice() {
                            s.compiler.insert(root.clone(), cause);
                        } else {
                            let (first, second) = named.split_at(named.len() / 2);
                            s.batches.push(second.to_vec());
                            s.batches.push(first.to_vec());
                        }
                    }
                    Ok((named, Some(Outcome::Late))) => {
                        eprintln!(
                            "certify production: pass {pass} (worker {worker}): {} roots unbuilt at the deadline, after {seconds}s",
                            named.len()
                        );
                        s.late = true;
                        s.batches.push(named);
                    }
                    Err(e) => s.error = Some(e),
                }
                wake.notify_all();
            });
        }
    });
    let mut s = state
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(e) = s.error {
        return Err(e);
    }
    // The roots no build reached by the deadline: uncertified at this
    // export, and built again by the next, since no compiler refused them.
    let unbuilt: Vec<String> = s.batches.drain(..).flatten().collect();
    for root in unbuilt {
        s.compiler
            .entry(root)
            .or_insert_with(|| UNBUILT.to_string());
    }
    // The refusals of every root refused by the end.
    let last = out.join("refusals");
    production(
        profile,
        projection,
        &last,
        register,
        targets,
        &s.compiler,
        None,
    )?;
    let refusals = last.join(REFUSALS);
    std::fs::copy(&refusals, kept.join(REFUSALS)).map_err(|e| io(&refusals, e))?;
    let timing = kept.join(TIMING);
    let text = format!("seconds = {}\nroots = {}\n", s.seconds, s.published);
    std::fs::write(&timing, text).map_err(|e| io(&timing, e))?;
    eprintln!(
        "certify production: {} bundles in {} passes, {} roots refused by the compiler",
        s.bundles,
        s.passes,
        s.compiler.len()
    );
    Ok(())
}

/// What every build of an export reads: the projection, the register, the
/// target descriptors, the roots' qualified prefix and the deadline.
struct Source<'a> {
    profile: &'a Profile,
    projection: &'a Path,
    register: &'a Register,
    targets: &'a str,
    prefix: &'a str,
    deadline: Option<std::time::Instant>,
}

/// Build one batch in the production project at `dir`: the roots the request
/// named (none when the compiler had refused them all) and what the build
/// asks next, its bundle replayed with `verify-artifact` when it published.
fn build_batch(
    source: &Source<'_>,
    dir: &Path,
    compiler: &BTreeMap<String, String>,
    batch: &[String],
) -> Result<(Vec<String>, Option<Outcome>), UorustError> {
    if dir.exists() {
        std::fs::remove_dir_all(dir).map_err(|e| io(dir, e))?;
    }
    let only: BTreeSet<String> = batch.iter().cloned().collect();
    let named = production(
        source.profile,
        source.projection,
        dir,
        source.register,
        source.targets,
        compiler,
        Some(&only),
    )?;
    if named.is_empty() {
        return Ok((named, None));
    }
    lexlean(dir, &["lock"])?;
    let Some(result) = lexlean_until(
        dir,
        &["production", "build", "--request", "request.json"],
        source.deadline,
    )?
    else {
        return Ok((named, Some(Outcome::Late)));
    };
    if result.get("success").and_then(serde_json::Value::as_bool) == Some(true) {
        // The artifact's path is the project's, relative to `dir`, where
        // lexlean runs.
        let artifact = result
            .get("artifacts")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|a| a.as_str())
            .ok_or_else(|| UorustError::Tool("lexlean production build: no bundle".into()))?;
        let bundle = dir.join(artifact);
        let replay = lexlean(dir, &["production", "verify-artifact", artifact])?;
        if replay.get("success").and_then(serde_json::Value::as_bool) != Some(true) {
            return Err(UorustError::Tool(format!(
                "lexlean production verify-artifact: {}",
                diagnostics(&replay).join("; ")
            )));
        }
        return Ok((named, Some(Outcome::Bundle(bundle))));
    }
    let messages = diagnostics(&result);
    let refused: Vec<(String, String)> = messages
        .iter()
        .filter(|m| !m.contains(TOTAL_BUDGET))
        .filter_map(|m| first_root(m, source.prefix).map(|r| (r, m.clone())))
        .filter(|(r, _)| !compiler.contains_key(r))
        .collect();
    if !refused.is_empty() {
        return Ok((named, Some(Outcome::Refused(refused))));
    }
    // A refusal that names no root (the total budget, or a limit of the
    // build's own, as run 43706236017's LLI9001 on a deep package manifest)
    // is the batch's: it is split until the refusal names its root or one
    // root holds it.
    match messages
        .iter()
        .find(|m| m.contains(TOTAL_BUDGET))
        .or(messages.first())
    {
        Some(m) => Ok((named, Some(Outcome::Over(m.clone())))),
        None => Err(UorustError::Tool(
            "lexlean production build: refused without a diagnostic".into(),
        )),
    }
}

/// The LexLean compiler's version, as `lexlean --version` states it.
fn compiler_version() -> Result<String, UorustError> {
    let run = std::process::Command::new("lexlean")
        .arg("--version")
        .output()
        .map_err(|e| UorustError::Tool(format!("lexlean --version: {e}")))?;
    Ok(String::from_utf8_lossy(&run.stdout)
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or_default()
        .to_string())
}

/// Keep what [`sync`] reads of a bundle under `to`: its `result.json` and
/// every package's `src/lib.rs` and Cargo manifest, at their paths in the
/// bundle.
fn keep(bundle: &Path, to: &Path) -> Result<std::path::PathBuf, UorustError> {
    if to.exists() {
        std::fs::remove_dir_all(to).map_err(|e| io(to, e))?;
    }
    std::fs::create_dir_all(to).map_err(|e| io(to, e))?;
    std::fs::copy(bundle.join("result.json"), to.join("result.json"))
        .map_err(|e| io(&bundle.join("result.json"), e))?;
    let packages = bundle.join("packages");
    for target in std::fs::read_dir(&packages).map_err(|e| io(&packages, e))? {
        let target = target.map_err(|e| io(&packages, e))?.path();
        for package in std::fs::read_dir(&target).map_err(|e| io(&target, e))? {
            let package = package.map_err(|e| io(&target, e))?.path();
            let rel = package.strip_prefix(bundle).unwrap_or(&package);
            let dest = to.join(rel).join("src");
            std::fs::create_dir_all(&dest).map_err(|e| io(&dest, e))?;
            let lib = package.join("src").join("lib.rs");
            std::fs::copy(&lib, dest.join("lib.rs")).map_err(|e| io(&lib, e))?;
            // The package's own manifest, whose edition a crate written
            // for the packages takes.
            let cargo = package.join("Cargo.toml");
            if cargo.is_file() {
                let to = to.join(rel).join("Cargo.toml");
                std::fs::copy(&cargo, to).map_err(|e| io(&cargo, e))?;
            }
        }
    }
    Ok(to.to_path_buf())
}

/// The diagnostics of a `lexlean` command result, each `code: message`.
fn diagnostics(result: &serde_json::Value) -> Vec<String> {
    result
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .into_iter()
        .flatten()
        .map(|d| {
            format!(
                "{} {}",
                d.get("code").and_then(|c| c.as_str()).unwrap_or_default(),
                d.get("message")
                    .and_then(|c| c.as_str())
                    .unwrap_or_default()
            )
        })
        .collect()
}

/// The first production root a diagnostic names, `<prefix><name>` in
/// backquotes, by its definition.
pub(crate) fn first_root(message: &str, prefix: &str) -> Option<String> {
    let at = message.find(&format!("`{prefix}"))? + 1 + prefix.len();
    let rest = &message[at..];
    let end = rest.find('`')?;
    Some(rest[..end].to_string())
}

/// Run `lexlean <args> --diagnostic-format json` in `dir`: its command
/// result, whatever its exit.
fn lexlean(dir: &Path, args: &[&str]) -> Result<serde_json::Value, UorustError> {
    lexlean_until(dir, args, None)?
        .ok_or_else(|| UorustError::Tool(format!("lexlean {}: no result", args.join(" "))))
}

/// [`lexlean`], stopped with every process it started when `deadline`
/// passes first: `None` then. The stop is coreutils' `timeout`, which runs
/// the command in its own process group and kills that whole group, the
/// `lean` lexlean runs among it, at the first whole second past `deadline`;
/// the `lake` and `lean` that left the group are killed by their build root.
fn lexlean_until(
    dir: &Path,
    args: &[&str],
    deadline: Option<std::time::Instant>,
) -> Result<Option<serde_json::Value>, UorustError> {
    let tool = |e: String| UorustError::Tool(format!("lexlean {}: {e}", args.join(" ")));
    let mut command = match deadline {
        Some(d) => {
            let left = d.saturating_duration_since(std::time::Instant::now());
            if left.is_zero() {
                return Ok(None);
            }
            let mut c = std::process::Command::new("timeout");
            c.args([
                "--signal=KILL",
                &format!("{}s", left.as_secs() + 1),
                "lexlean",
            ]);
            c
        }
        None => std::process::Command::new("lexlean"),
    };
    let run = command
        .args(args)
        .args(["--diagnostic-format", "json"])
        .current_dir(dir)
        .output()
        .map_err(|e| tool(e.to_string()))?;
    if deadline.is_some_and(|d| std::time::Instant::now() >= d) && !run.status.success() {
        // lake and lean leave the group: each names the build root.
        let _ = std::process::Command::new("pkill")
            .args(["-KILL", "-f"])
            .arg(dir.join(".lexlean").join(""))
            .status();
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&run.stdout);
    let last = text.lines().last().unwrap_or_default();
    serde_json::from_str(last)
        .map(Some)
        .map_err(|e| tool(format!("{e}: {}", String::from_utf8_lossy(&run.stderr))))
}

fn module_prefix(projection: &Path) -> Result<String, UorustError> {
    let config = projection.join("lexlean.toml");
    let text = std::fs::read_to_string(&config).map_err(|e| io(&config, e))?;
    text.lines()
        .find_map(|l| l.strip_prefix("module_prefix = "))
        .map(|v| v.trim().trim_matches('"').to_string())
        .ok_or_else(|| UorustError::Malformed {
            path: config,
            reason: "no module_prefix".to_string(),
        })
}

/// One top-level declaration of a module's text: a definition's name, when
/// it is one, and its lines.
struct Chunk {
    name: Option<String>,
    text: String,
}

/// A module's text cut into its top-level declarations, in order; joined
/// again they are the text.
fn chunks(source: &str) -> Vec<Chunk> {
    let mut out: Vec<Chunk> = Vec::new();
    let mut open = false;
    for line in source.lines() {
        // A definition on one line, as the MIR path writes its translations.
        if let Some(inner) = line.strip_prefix("definition{name: ") {
            let name = inner.split(',').next().unwrap_or_default().to_string();
            out.push(Chunk {
                name: Some(name),
                text: format!("{line}\n"),
            });
            continue;
        }
        if line == "definition{" {
            open = true;
            out.push(Chunk {
                name: None,
                text: String::new(),
            });
        } else if !open {
            out.push(Chunk {
                name: None,
                text: String::new(),
            });
        }
        let Some(chunk) = out.last_mut() else {
            continue;
        };
        if open && chunk.name.is_none() {
            if let Some(name) = line
                .strip_prefix("  name: ")
                .and_then(|n| n.strip_suffix(','))
            {
                chunk.name = Some(name.to_string());
            }
        }
        chunk.text.push_str(line);
        chunk.text.push('\n');
        if line == "}" {
            open = false;
        }
    }
    out
}

/// The definitions of a projection's modules, by qualified name
/// (`Module::name`), with their texts.
pub(crate) struct Definitions(BTreeMap<String, String>);

impl Definitions {
    /// The definitions of `modules`, by module name with their texts.
    pub(crate) fn of(modules: &BTreeMap<String, String>) -> Definitions {
        let mut defs = BTreeMap::new();
        for (module, text) in modules {
            for c in chunks(text) {
                if let Some(name) = c.name {
                    defs.insert(format!("{module}::{name}"), c.text);
                }
            }
        }
        Definitions(defs)
    }

    /// A definition's text.
    pub(crate) fn text(&self, def: &str) -> Option<&str> {
        self.0.get(def).map(String::as_str)
    }

    /// A definition's closure: itself and every definition its body
    /// reaches.
    pub(crate) fn closure(&self, start: &str) -> BTreeSet<String> {
        let mut reached: BTreeSet<String> = BTreeSet::new();
        let mut work = vec![start.to_string()];
        while let Some(def) = work.pop() {
            let Some(text) = self.0.get(&def) else {
                continue;
            };
            if !reached.insert(def.clone()) {
                continue;
            }
            let module = def.split("::").next().unwrap_or_default();
            for token in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':'))
            {
                let key = if token.contains("::") {
                    token.to_string()
                } else {
                    format!("{module}::{token}")
                };
                if self.0.contains_key(&key) && !reached.contains(&key) {
                    work.push(key);
                }
            }
        }
        reached
    }
}

/// The modules' texts, by module name, with every root (a definition of
/// `Behavior`) a production root and every definition a root reaches
/// executable, the roots found, and the roots refused with their causes.
#[cfg(test)]
pub(crate) fn mark(
    modules: &BTreeMap<String, String>,
    roots: &BTreeSet<&str>,
) -> (
    BTreeMap<String, String>,
    BTreeSet<String>,
    BTreeMap<String, String>,
) {
    Profile::default().mark(modules, roots)
}

impl Profile {
    /// The modules' texts, by module name, with every root (a definition
    /// of `Behavior`) a production root with this profile's effects and
    /// target, and every definition a root reaches executable; the roots
    /// found; and the roots refused with their causes.
    pub(crate) fn mark(
        &self,
        modules: &BTreeMap<String, String>,
        roots: &BTreeSet<&str>,
    ) -> (
        BTreeMap<String, String>,
        BTreeSet<String>,
        BTreeMap<String, String>,
    ) {
        let cut: BTreeMap<&String, Vec<Chunk>> =
            modules.iter().map(|(m, text)| (m, chunks(text))).collect();
        let definitions = Definitions::of(modules);
        let defs = &definitions.0;
        let closure = |start: &str| definitions.closure(start);
        let mut reached: BTreeSet<String> = BTreeSet::new();
        let mut refused: BTreeMap<String, String> = BTreeMap::new();
        for root in roots {
            let own = closure(&format!("{BEHAVIOR}::{root}"));
            // SPEC §17.13: an executable definition takes no function.
            if let Some(hof) = own
                .iter()
                .find(|d| defs.get(*d).is_some_and(|t| higher_order(t)))
            {
                refused.insert(
                (*root).to_string(),
                format!("reaches {hof}, a definition with a function parameter, which no production root may"),
            );
                continue;
            }
            reached.extend(own);
        }
        let mut found = BTreeSet::new();
        let mut out = BTreeMap::new();
        for (module, chunks) in cut {
            let mut text = String::new();
            for c in chunks {
                let key = c.name.as_ref().map(|n| format!("{module}::{n}"));
                let Some(key) = key.filter(|k| reached.contains(k)) else {
                    text.push_str(&c.text);
                    continue;
                };
                let root = module == BEHAVIOR
                    && c.name
                        .as_deref()
                        .is_some_and(|n| roots.contains(n) && !refused.contains_key(n));
                let mut fields = Vec::new();
                if !c.text.contains("executable: true") {
                    fields.push("executable: true".to_string());
                }
                if root {
                    fields.push(format!(
                        "production: {{effects: [{}], targets: [{}]}}",
                        self.effects.join(", "),
                        self.target
                    ));
                    found.insert(key.trim_start_matches(&format!("{BEHAVIOR}::")).to_string());
                }
                let body = c
                    .text
                    .trim_end_matches('\n')
                    .trim_end_matches('}')
                    .trim_end();
                if fields.is_empty() {
                    text.push_str(&c.text);
                } else if c.text.lines().count() == 1 {
                    text.push_str(&format!("{body}, {}}}\n", fields.join(", ")));
                } else {
                    text.push_str(&format!("{body},\n  {}\n}}\n", fields.join(",\n  ")));
                }
            }
            out.insert(module.clone(), text);
        }
        (out, found, refused)
    }
}

/// Whether a definition's text takes a function: a parameter of a `Fn` type.
fn higher_order(text: &str) -> bool {
    let Some(at) = text.find("parameters: [") else {
        return false;
    };
    let rest = &text[at..];
    let end = rest.find("result:").unwrap_or(rest.len());
    rest[..end].contains("(Fn ")
}

/// The file of the production project that lists the roots it could not
/// export, by κ, with their causes.
pub const REFUSALS: &str = "refusals.toml";

fn toml_quoted(text: &str) -> String {
    toml::Value::String(text.to_string()).to_string()
}

/// The module whose definitions are the lifted functions.
const BEHAVIOR: &str = "Behavior";

fn copy_tree(from: &Path, to: &Path) -> Result<(), UorustError> {
    std::fs::create_dir_all(to).map_err(|e| io(to, e))?;
    let mut entries: Vec<_> = std::fs::read_dir(from)
        .map_err(|e| io(from, e))?
        .filter_map(Result::ok)
        .collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        // The build root is the verifier's, never the projection's.
        if name == ".lexlean" || name == ".lake" {
            continue;
        }
        let target = to.join(&name);
        if path.is_dir() {
            copy_tree(&path, &target)?;
        } else {
            std::fs::copy(&path, &target).map_err(|e| io(&path, e))?;
        }
    }
    Ok(())
}

/// One certified root of the manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Certified {
    /// The Rust function's crate path.
    pub function: String,
    /// Its κ.
    pub kappa: String,
    /// The definition it exports.
    pub definition: String,
    /// The package's name.
    pub package: String,
    /// The SHA-256 of the package's `src/lib.rs`, as the bundle records it.
    pub source: String,
    /// The kernel's status of its certificates.
    pub certification: String,
}

/// The certified translator: the bundle it came from, its roots, and the
/// figures of the bootstrap.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Manifest {
    /// The bundle's result id.
    pub result: String,
    /// The LexLean compiler's version.
    pub compiler: String,
    /// The certified roots, by package.
    pub roots: Vec<Certified>,
    /// The figures, as [`Share`] computes them.
    pub share: Share,
    /// The roots the production project could not export, by κ, with
    /// their causes.
    pub refused: BTreeMap<String, String>,
    /// The build seconds the export spent per root it published, over all
    /// its passes and shards: what the next export sizes its shards by.
    pub seconds_per_root: Option<u64>,
}

/// The figures of the bootstrap.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Share {
    /// Translator functions in the register.
    pub translator: usize,
    /// Those certified.
    pub certified: usize,
    /// Translator functions reachable from [`ENTRY`].
    pub reachable: usize,
    /// Those certified.
    pub reachable_certified: usize,
}

impl Manifest {
    /// The κ of every certified root.
    pub fn kappas(&self) -> BTreeSet<String> {
        self.roots.iter().map(|r| r.kappa.clone()).collect()
    }

    /// Read a manifest; an absent file is the empty manifest.
    pub fn read(path: &Path) -> Result<Manifest, UorustError> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Manifest::default()),
            Err(e) => return Err(io(path, e)),
        };
        Manifest::parse(path, &text)
    }

    /// Parse a manifest's text.
    pub fn parse(path: &Path, text: &str) -> Result<Manifest, UorustError> {
        let bad = |reason: String| UorustError::Malformed {
            path: path.to_path_buf(),
            reason,
        };
        let doc: toml::Value = text.parse().map_err(|e| bad(format!("{e}")))?;
        if doc.get("schema").and_then(toml::Value::as_str) != Some(SCHEMA) {
            return Err(bad(format!("not a {SCHEMA} manifest")));
        }
        let text_of = |v: &toml::Value, k: &str| {
            v.get(k)
                .and_then(toml::Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| bad(format!("no {k}")))
        };
        let count = |k: &str| {
            doc.get("share")
                .and_then(|s| s.get(k))
                .and_then(toml::Value::as_integer)
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(|| bad(format!("no share.{k}")))
        };
        let mut roots = Vec::new();
        for row in doc
            .get("root")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
        {
            roots.push(Certified {
                function: text_of(row, "function")?,
                kappa: text_of(row, "kappa")?,
                definition: text_of(row, "definition")?,
                package: text_of(row, "package")?,
                source: text_of(row, "source")?,
                certification: text_of(row, "certification")?,
            });
        }
        let mut refused = BTreeMap::new();
        for row in doc
            .get("refused")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
        {
            refused.insert(text_of(row, "kappa")?, text_of(row, "cause")?);
        }
        Ok(Manifest {
            seconds_per_root: doc
                .get("timing")
                .and_then(|t| t.get("seconds_per_root"))
                .and_then(toml::Value::as_integer)
                .and_then(|n| u64::try_from(n).ok()),
            refused,
            result: text_of(&doc, "result")?,
            compiler: text_of(&doc, "compiler")?,
            roots,
            share: Share {
                translator: count("translator")?,
                certified: count("certified")?,
                reachable: count("reachable")?,
                reachable_certified: count("reachable_certified")?,
            },
        })
    }

    /// The manifest's text, of the translator's own export.
    pub fn to_text(&self) -> String {
        self.text(&Profile::default())
    }

    /// The manifest's text, of an export under `profile`: the translator's
    /// own states no profile, any other states its own in `[profile]`.
    pub fn text(&self, profile: &Profile) -> String {
        let s = &self.share;
        let own = *profile == Profile::default();
        let mut out = String::new();
        out.push_str(match (own, profile.kind) {
            (true, _) => {
                "# The certified translator, written by `uorust certify sync` (just certified):\n\
                 # every lifted translator function exported by `lexlean production` and\n\
                 # kernel-checked, keyed by the κ of the Rust function it certifies.\n"
            }
            (false, Kind::Functions) => {
                "# The certified implementation, written by `uorust certify sync`: every\n\
                 # lifted function in scope exported by `lexlean production` and\n\
                 # kernel-checked, keyed by the κ of the Rust function it certifies.\n"
            }
            (false, Kind::Cases) => {
                "# The certified cases, written by `uorust certify sync`: every closed\n\
                 # assertion of the workspace's tests over a lifted function in scope,\n\
                 # exported by `lexlean production` and kernel-checked, keyed by the κ of\n\
                 # the definitions it runs.\n"
            }
        });
        out.push_str(&format!("schema = \"{SCHEMA}\"\n"));
        out.push_str(&format!("result = \"{}\"\n", self.result));
        out.push_str(&format!("compiler = \"{}\"\n", self.compiler));
        if let Some(entry) = &profile.entry {
            out.push_str(&format!("entry = \"{entry}\"\n"));
        }
        if !own {
            out.push_str("\n# What the export is of.\n[profile]\n");
            out.push_str(&profile.rows());
        }
        out.push_str(if own {
            "\n# The bootstrap: the share of the translator that runs certified.\n"
        } else {
            "\n# The share that is certified: `translator` counts what is in scope.\n"
        });
        out.push_str("[share]\n");
        out.push_str(&format!("translator = {}\n", s.translator));
        out.push_str(&format!("certified = {}\n", s.certified));
        out.push_str(&format!("reachable = {}\n", s.reachable));
        out.push_str(&format!(
            "reachable_certified = {}\n",
            s.reachable_certified
        ));
        if let Some(n) = self.seconds_per_root {
            out.push_str(
                "\n# The build seconds per published root, which sizes the next export's shards.\n",
            );
            out.push_str(&format!("[timing]\nseconds_per_root = {n}\n"));
        }
        for r in &self.roots {
            out.push_str("\n[[root]]\n");
            out.push_str(&format!("function = \"{}\"\n", r.function));
            out.push_str(&format!("kappa = \"{}\"\n", r.kappa));
            out.push_str(&format!("definition = \"{}\"\n", r.definition));
            out.push_str(&format!("package = \"{}\"\n", r.package));
            out.push_str(&format!("source = \"{}\"\n", r.source));
            out.push_str(&format!("certification = \"{}\"\n", r.certification));
        }
        for (kappa, cause) in &self.refused {
            out.push_str("\n[[refused]]\n");
            out.push_str(&format!("kappa = \"{kappa}\"\n"));
            out.push_str(&format!("cause = {}\n", toml_quoted(cause)));
        }
        out
    }
}

/// The bundles [`production_build`] kept under `kept`, in the order it
/// built them: each directory `kept/<n>` that holds a `result.json`.
fn bundles(kept: &Path) -> Result<Vec<PathBuf>, UorustError> {
    let mut out: Vec<(usize, PathBuf)> = std::fs::read_dir(kept)
        .map_err(|e| io(kept, e))?
        .filter_map(Result::ok)
        .filter_map(|e| {
            let n = e.file_name().to_str()?.parse::<usize>().ok()?;
            Some((n, e.path()))
        })
        .filter(|(_, p)| p.join("result.json").is_file())
        .collect();
    out.sort();
    Ok(out.into_iter().map(|(_, p)| p).collect())
}

/// What a sync takes in and where: the kept bundles, the directory of the
/// certified crate, the register its roots are current under, and what only
/// some exports have.
#[derive(Clone, Copy, Debug)]
pub struct Sync<'a> {
    /// What `certify production` kept, whole or by shard.
    pub kept: &'a Path,
    /// The certified crate's directory, which the caller names.
    pub dir: &'a Path,
    /// The register the roots are current under.
    pub register: &'a Register,
    /// The projection of that register: a cases export's roots are read
    /// from it.
    pub projection: Option<&'a Path>,
    /// The adapters' module that is written, when the workspace runs its
    /// certified packages in place of its functions.
    pub adapters: Option<&'a Path>,
    /// The workspace and its MIR dumps, for the adapters' types and for the
    /// share reachable from the profile's entry.
    pub mir: Option<(&'a Path, &'a Path)>,
}

/// Take the bundles kept under `kept` into `dir`: each certified root's
/// `src/lib.rs` as `dir/roots/<package>.rs` (checked against the SHA-256 its
/// bundle's result records), the crate that compiles them (with its Cargo
/// manifest, when the directory has none), and the manifest, with the share
/// over `register` and the call graph of `dumps`. A package is taken only
/// while its root's κ is the one it was exported at, so a function changed
/// since the export is uncertified until its package is rebuilt.
pub(crate) fn sync(
    profile: &Profile,
    sync: &Sync<'_>,
    roots: &[Root],
    dumps: Option<&Dumps>,
    compiler: &str,
) -> Result<Manifest, UorustError> {
    let Sync {
        kept,
        dir,
        register,
        adapters,
        ..
    } = *sync;
    let bad = |path: &Path, reason: String| UorustError::Malformed {
        path: path.to_path_buf(),
        reason,
    };
    // Refused before anything is written: not what `certify production`
    // keeps, which always holds the refusals and each bundle's result.json.
    let outputs = kept_roots(kept)?;
    if !outputs.iter().any(|k| k.join(REFUSALS).is_file()) {
        return Err(bad(
            kept,
            format!("no {REFUSALS} beside a bundle's result.json: not kept by certify production"),
        ));
    }
    let mut measured: Option<toml::Table> = None;
    let mut all = Vec::new();
    for k in &outputs {
        let kappas = k.join(KAPPAS);
        if let Ok(text) = std::fs::read_to_string(&kappas) {
            let table: toml::Table = text.parse().map_err(|e| bad(&kappas, format!("{e}")))?;
            measured.get_or_insert_with(toml::Table::new).extend(table);
        }
        all.extend(bundles(k)?);
    }
    let mut results = Vec::new();
    for bundle in all {
        let path = bundle.join("result.json");
        let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
        let result: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| bad(&path, e.to_string()))?;
        results.push((bundle, path, result));
    }
    let field = |v: &serde_json::Value, keys: &[&str]| {
        let mut at = v;
        for k in keys {
            at = at.get(k).unwrap_or(&serde_json::Value::Null);
        }
        at.as_str().unwrap_or_default().to_string()
    };
    let by_package: BTreeMap<String, &Root> = roots.iter().map(|r| (r.package(), r)).collect();
    let roots_dir = dir.join("roots");
    if roots_dir.is_dir() {
        std::fs::remove_dir_all(&roots_dir).map_err(|e| io(&roots_dir, e))?;
    }
    std::fs::create_dir_all(&roots_dir).map_err(|e| io(&roots_dir, e))?;
    let mut certified = Vec::new();
    let mut edition = None;
    for (bundle, path, result) in &results {
        let (bundle, path) = (bundle.as_path(), path.as_path());
        for root in result
            .get("roots")
            .and_then(|r| r.as_array())
            .into_iter()
            .flatten()
        {
            for output in root
                .get("outputs")
                .and_then(|o| o.as_array())
                .into_iter()
                .flatten()
            {
                let package = output.get("package").unwrap_or(&serde_json::Value::Null);
                let directory = field(package, &["directory"]);
                let name = directory.rsplit('/').next().unwrap_or_default().to_string();
                let Some(own) = by_package.get(&name) else {
                    if measured.is_some() {
                        continue;
                    }
                    return Err(bad(path, format!("{name} is no root of the register")));
                };
                // A function changed since the export is a new question.
                if measured.as_ref().is_some_and(|m| {
                    m.get(&name).and_then(toml::Value::as_str) != Some(own.kappa.as_str())
                }) {
                    continue;
                }
                let lib = format!("{directory}/src/lib.rs");
                let recorded = package
                    .get("files")
                    .and_then(|f| f.as_array())
                    .into_iter()
                    .flatten()
                    .find(|f| field(f, &["path"]) == lib)
                    .map(|f| field(f, &["sha256"]))
                    .ok_or_else(|| bad(path, format!("{lib} is not in the result")))?;
                let source_path = bundle.join(&lib);
                let bytes = std::fs::read(&source_path).map_err(|e| io(&source_path, e))?;
                let digest = crate::project::sha(&bytes);
                if digest != recorded {
                    return Err(bad(
                        &source_path,
                        format!("SHA-256 {digest}, the result records {recorded}"),
                    ));
                }
                let target = roots_dir.join(format!("{name}.rs"));
                std::fs::write(&target, &bytes).map_err(|e| io(&target, e))?;
                if edition.is_none() {
                    edition = package_edition(&bundle.join(&directory));
                }
                certified.push(Certified {
                    function: own.function.clone(),
                    kappa: own.kappa.clone(),
                    definition: own.definition.clone(),
                    package: name,
                    source: digest,
                    certification: field(output, &["certification", "status"]),
                });
            }
        }
    }
    certified.sort_by(|a, b| a.package.cmp(&b.package));
    let lib = profile.crate_source(&certified);
    let lib_path = dir.join("src").join("lib.rs");
    std::fs::create_dir_all(dir.join("src")).map_err(|e| io(dir, e))?;
    std::fs::write(&lib_path, lib).map_err(|e| io(&lib_path, e))?;
    // The crate's Cargo manifest is the caller's when the directory has
    // one (the translator's inherits its workspace's); else it is written,
    // a package of its own that any directory builds.
    let cargo = dir.join("Cargo.toml");
    if !cargo.exists() {
        let text = profile.cargo_manifest(edition.as_deref());
        std::fs::write(&cargo, text).map_err(|e| io(&cargo, e))?;
    }
    if let (Some(adapters), Some(dumps)) = (adapters, dumps) {
        let empty = BTreeMap::new();
        let mut wired = BTreeMap::new();
        for c in &certified {
            let source = std::fs::read_to_string(roots_dir.join(format!("{}.rs", c.package)))
                .map_err(|e| io(&roots_dir, e))?;
            let (krate, path) = c.function.split_once("::").unwrap_or(("", &c.function));
            let bodies = dumps.crates.get(krate).unwrap_or(&empty);
            let adapted = bodies
                .get(path)
                .and_then(|b| profile.adapter(c, b, &source));
            if let Some((name, code)) = adapted {
                wired.insert(name, code);
            }
        }
        let mut sites = BTreeMap::new();
        if let Some(src) = adapters.parent() {
            call_sites(src, adapters, &mut sites)?;
        }
        let text = adapters_source(&wired, &sites);
        std::fs::write(adapters, text).map_err(|e| io(adapters, e))?;
    }
    let kappas: BTreeSet<String> = certified.iter().map(|c| c.kappa.clone()).collect();
    let mut refused: BTreeMap<String, String> = BTreeMap::new();
    let (mut seconds, mut built) = (0u64, 0u64);
    for k in &outputs {
        let refusals = k.join(REFUSALS);
        let Ok(text) = std::fs::read_to_string(&refusals) else {
            continue;
        };
        let doc: toml::Value = text.parse().map_err(|e| bad(&refusals, format!("{e}")))?;
        refused.extend(
            doc.get("refused")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|r| {
                    Some((
                        r.get("kappa")?.as_str()?.to_string(),
                        r.get("cause")?.as_str()?.to_string(),
                    ))
                }),
        );
        let timing: toml::Table = std::fs::read_to_string(k.join(TIMING))
            .ok()
            .and_then(|t| t.parse().ok())
            .unwrap_or_default();
        let count = |key: &str| {
            timing
                .get(key)
                .and_then(toml::Value::as_integer)
                .and_then(|n| u64::try_from(n).ok())
                .unwrap_or_default()
        };
        seconds += count("seconds");
        built += count("roots");
    }
    // A root a shard built is not refused by another shard's view of it.
    refused.retain(|kappa, _| !kappas.contains(kappa));
    let manifest = Manifest {
        refused,
        result: results
            .iter()
            .map(|(_, _, r)| field(r, &["result_id"]))
            .collect::<Vec<_>>()
            .join(" "),
        // As `lexlean --version` states it, which the next export compares
        // its own against: a manifest of no bundle has it too.
        compiler: compiler.to_string(),
        seconds_per_root: seconds.checked_div(built),
        roots: certified,
        share: match profile.kind {
            Kind::Functions => profile.share(register, &kappas, dumps),
            // Of a cases export, the cases the projection states and those
            // whose package is current.
            Kind::Cases => Share {
                translator: roots.len(),
                certified: roots.iter().filter(|r| kappas.contains(&r.kappa)).count(),
                ..Share::default()
            },
        },
    };
    let manifest_path = dir.join("manifest.toml");
    std::fs::write(&manifest_path, manifest.text(profile)).map_err(|e| io(&manifest_path, e))?;
    Ok(manifest)
}

/// The edition a bundle's package declares in its Cargo manifest.
fn package_edition(package: &Path) -> Option<String> {
    let text = std::fs::read_to_string(package.join("Cargo.toml")).ok()?;
    let doc: toml::Value = text.parse().ok()?;
    doc.get("package")?
        .get("edition")?
        .as_str()
        .map(str::to_string)
}

/// The profile every kept output under `kept` recorded: one export has one.
pub fn kept_profile(kept: &Path) -> Result<Profile, UorustError> {
    let mut found: Option<Profile> = None;
    for k in kept_roots(kept)? {
        let profile = Profile::kept(&k)?;
        if found.as_ref().is_some_and(|f| *f != profile) {
            return Err(UorustError::Malformed {
                path: k.join(PROFILE),
                reason: "the shards were exported under different profiles".to_string(),
            });
        }
        found = Some(profile);
    }
    Ok(found.unwrap_or_default())
}

/// [`sync`] of the bundles [`production_build`] kept, under the profile the
/// export recorded there, with the dumps read when the sync names them.
/// Returns the profile and the manifest.
pub fn sync_paths(s: &Sync<'_>) -> Result<(Profile, Manifest), UorustError> {
    let profile = kept_profile(s.kept)?;
    let roots = match (profile.kind, s.projection) {
        (Kind::Functions, _) => profile.roots(s.register),
        (Kind::Cases, Some(projection)) => profile.export_roots(projection, s.register)?,
        (Kind::Cases, None) => {
            return Err(UorustError::Tool(
                "a cases export is taken in against the projection its cases are read from: pass --projection"
                    .to_string(),
            ))
        }
    };
    let dumps = match s.mir {
        Some((workspace, mir)) => Some(crate::mir::load(workspace, mir)?),
        None => None,
    };
    let manifest = sync(&profile, s, &roots, dumps.as_ref(), &compiler_version()?)?;
    Ok((profile, manifest))
}

/// The adapters' module, `crates/uorust/src/certified.rs`: the switch
/// between the certified and the hand-written translator, the macro a
/// hand-written function runs its certified realization through, one adapter
/// per certified root whose types convert, and a stub answering `None` for
/// each call site whose root has no adapter at this commit, so that a root
/// leaving the certified set never breaks the build.
pub(crate) fn adapters_source(
    adapters: &BTreeMap<String, String>,
    sites: &BTreeMap<String, usize>,
) -> String {
    let mut out = String::from(ADAPTERS_HEAD);
    for a in adapters.values() {
        out.push('\n');
        out.push_str(a);
    }
    for (name, arity) in sites {
        if adapters.contains_key(name) {
            continue;
        }
        let params: Vec<String> = (0..*arity).map(|i| format!("A{i}: ?Sized")).collect();
        let args: Vec<String> = (0..*arity).map(|i| format!("_: &A{i}")).collect();
        out.push_str(&format!(
            "\n/// Not certified at this commit: the hand-written body runs.\npub(crate) fn {name}<{}R>({}) -> Option<R> {{\n    None\n}}\n",
            params.iter().map(|p| format!("{p}, ")).collect::<String>(),
            args.join(", ")
        ));
    }
    out.push_str(ADAPTERS_TESTS);
    out
}

/// The `certified!(adapter, args..)` call sites under `dir`, by adapter,
/// with their argument counts; the adapters' module itself is skipped.
fn call_sites(
    dir: &Path,
    adapters: &Path,
    out: &mut BTreeMap<String, usize>,
) -> Result<(), UorustError> {
    const SITE: &str = "certified!(";
    let entries = std::fs::read_dir(dir).map_err(|e| io(dir, e))?;
    let mut paths: Vec<PathBuf> = entries
        .map(|e| e.map(|e| e.path()).map_err(|e| io(dir, e)))
        .collect::<Result<_, UorustError>>()?;
    paths.sort();
    for path in paths {
        if path.is_dir() {
            call_sites(&path, adapters, out)?;
            continue;
        }
        let own = path.file_name().and_then(|n| n.to_str()) == Some("certify.rs");
        if path == adapters || own || path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
        for (at, _) in text.match_indices(SITE) {
            let rest = &text[at + SITE.len()..];
            let mut depth = 1usize;
            let end = rest.char_indices().find_map(|(i, c)| {
                match c {
                    '(' | '[' => depth += 1,
                    ')' | ']' => depth -= 1,
                    _ => {}
                }
                (depth == 0).then_some(i)
            });
            let Some(end) = end else { continue };
            let parts: Vec<String> = split_top_level(&rest[..end])
                .into_iter()
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();
            if let Some((name, args)) = parts.split_first() {
                out.insert(name.clone(), args.len());
            }
        }
    }
    Ok(())
}

const ADAPTERS_HEAD: &str = r#"//! The certified translator's adapters, written by `uorust certify sync`
//! (`just certified`): for each certified root whose parameter and result
//! types convert, a function over references to the hand-written one's
//! arguments that runs the certified package (crates/certified) on them. A
//! hand-written function runs its certified realization through
//! [`certified!`], its first statement; a site whose root this commit does
//! not certify has a stub that answers `None`, so the hand-written body runs.
//! `--translator hand` runs every hand-written body, the differential's second
//! path. A certified root that overflows its target's width stops with a
//! panic, which the differential sees as a disagreement.

// The adapters are written from the certified packages' signatures: an
// adapter is named after its root's Rust path and wraps its arguments as the
// package's `run` takes them.
#![allow(dead_code, non_snake_case, unused_parens)]

use std::sync::atomic::{AtomicBool, Ordering};

static HAND: AtomicBool = AtomicBool::new(false);

/// Run the hand-written translator in place of the certified one: the
/// differential's second path.
pub fn use_hand(hand: bool) {
    HAND.store(hand, Ordering::Relaxed);
}

/// Whether the hand-written translator runs.
pub(crate) fn hand() -> bool {
    HAND.load(Ordering::Relaxed)
}

/// `certified!(adapter, args...)`, a hand-written function's first
/// statement under `#[cfg(not(uorust_lift))]`: return what the adapter
/// answers, unless the hand-written translator runs. The lift's dump sets
/// `uorust_lift`, so the self register lifts the hand-written body alone.
macro_rules! certified {
    ($adapter:ident $(, $arg:expr)* $(,)?) => {
        if !$crate::certified::hand() {
            if let Some(v) = $crate::certified::$adapter($(&$arg),*) {
                return v;
            }
        }
    };
}
pub(crate) use certified;
"#;

const ADAPTERS_TESTS: &str = r#"
#[cfg(test)]
mod tests {
    // The switch the differential runs the hand-written translator through.
    #[test]
    fn use_hand_selects_the_hand_written_translator() {
        super::use_hand(true);
        assert!(super::hand());
        super::use_hand(false);
        assert!(!super::hand());
    }
}
"#;

/// The adapter of a certified root over its hand-written function's body:
/// none when a parameter or the result has no conversion.
#[cfg(test)]
pub(crate) fn adapter(root: &Certified, body: &IrBody, source: &str) -> Option<(String, String)> {
    Profile::default().adapter(root, body, source)
}

impl Profile {
    /// The adapter of a root certified under this profile over its function's
    /// body, calling its module of the profile's certified crate and named by
    /// the function's path in its own crate: none when a parameter or the
    /// result has no conversion.
    pub(crate) fn adapter(
        &self,
        root: &Certified,
        body: &IrBody,
        source: &str,
    ) -> Option<(String, String)> {
        let (params, result) = run_signature(source)?;
        if params.len() != body.arity {
            return None;
        }
        let m = format!("{}::{}", self.crate_name(), self.module(&root.package));
        let mut args = Vec::new();
        let mut converted = Vec::new();
        for (i, cert) in params.iter().enumerate() {
            let rust = unlifetimed(body.locals.get(i + 1)?);
            // Each argument comes by reference, so the hand-written body still
            // owns it when the root is not certified.
            args.push(format!("p{i}: &{rust}"));
            converted.push(if rust.starts_with('&') {
                to_cert(&format!("(*p{i})"), &rust, cert, &m)?
            } else {
                to_cert(&format!("p{i}"), &format!("&{rust}"), cert, &m)?
            });
        }
        let rust_result = unlifetimed(body.locals.first()?);
        let call = format!("{m}::run({})", converted.join(", "));
        let value = match result.strip_prefix("R<").and_then(|r| r.strip_suffix('>')) {
            Some(inner) => {
                let overflow = format!(
                "match {call} {{ Ok(v) => v, Err(_) => panic!(\"{}: the certified root overflowed\") }}",
                root.function
            );
                from_cert(&overflow, inner, &rust_result)?
            }
            None => from_cert(&call, &result, &rust_result)?,
        };
        let name = root
            .function
            .split_once("::")
            .map_or(root.function.as_str(), |(_, path)| path)
            .replace("::", "_");
        let code = format!(
        "/// `{}`, certified as `Behavior::{}`.\n#[allow(clippy::all, non_snake_case, unused_parens)]\npub(crate) fn {name}({}) -> Option<{rust_result}> {{\n    Some({value})\n}}\n",
        root.function,
        root.definition,
        args.join(", ")
    );
        Some((name, code))
    }
}

/// A type as MIR prints it, without its lifetimes.
fn unlifetimed(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\'' {
            while chars
                .peek()
                .is_some_and(|n| n.is_alphanumeric() || *n == '_')
            {
                chars.next();
            }
            if chars.peek() == Some(&' ') {
                chars.next();
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// The parameter and result types of a package's `run`.
fn run_signature(source: &str) -> Option<(Vec<String>, String)> {
    let at = source.find("pub fn run(")? + "pub fn run(".len();
    let rest = &source[at..];
    let close = rest.find(") -> ")?;
    let params: Vec<String> = split_top_level(&rest[..close])
        .into_iter()
        .filter(|p| !p.trim().is_empty())
        .map(|p| {
            p.split_once(':')
                .map(|(_, t)| t.trim().to_string())
                .unwrap_or_default()
        })
        .collect();
    let result = rest[close + ") -> ".len()..]
        .split('{')
        .next()?
        .trim()
        .to_string();
    Some((params, result))
}

fn split_top_level(text: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut depth = 0usize;
    for c in text.chars() {
        match c {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                out.push(String::new());
                continue;
            }
            _ => {}
        }
        if let Some(last) = out.last_mut() {
            last.push(c);
        }
    }
    out
}

/// The generic argument of `outer<..>` in a type.
fn argument<'a>(text: &'a str, outer: &str) -> Option<&'a str> {
    text.strip_prefix(outer)?
        .strip_prefix('<')?
        .strip_suffix('>')
}

const NATURALS: &[&str] = &["u8", "u16", "u32", "u64", "usize"];

/// The expression converting the Rust value `e` of `rust` to the package's
/// `cert`.
fn to_cert(e: &str, rust: &str, cert: &str, m: &str) -> Option<String> {
    let rust = rust.trim();
    if let Some(inner) = rust.strip_prefix('&') {
        let inner = inner.trim_start_matches("mut ").trim();
        return match inner {
            "str" | "std::string::String" if cert == "Str" => Some(format!("{m}::Str::lit({e})")),
            n if NATURALS.contains(&n) && cert == "u64" => Some(format!(
                "u64::try_from(*{e}).expect(\"a natural of 64 bits\")"
            )),
            "bool" if cert == "bool" => Some(format!("*{e}")),
            _ => {
                let elem = inner
                    .strip_prefix('[')
                    .and_then(|t| t.strip_suffix(']'))
                    .or_else(|| argument(inner, "std::vec::Vec"))?;
                let item = argument(cert, "List")?;
                let each = to_cert("x", &format!("&{elem}"), item, m)?;
                Some(format!(
                    "{{ let mut l = {m}::List::nil(); for x in {e}.iter().rev() {{ l = {m}::List::cons({each}, l); }} l }}"
                ))
            }
        };
    }
    match rust {
        "std::string::String" if cert == "Str" => Some(format!("{m}::Str::lit(&{e})")),
        n if NATURALS.contains(&n) && cert == "u64" => Some(format!(
            "u64::try_from({e}).expect(\"a natural of 64 bits\")"
        )),
        "bool" if cert == "bool" => Some(e.to_string()),
        _ => {
            if let (Some(t), Some(c)) = (
                argument(rust, "std::option::Option"),
                argument(cert, "Option"),
            ) {
                let each = to_cert("x", t, c, m)?;
                return Some(format!("{e}.map(|x| {each})"));
            }
            let elem = argument(rust, "std::vec::Vec")?;
            let item = argument(cert, "List")?;
            let each = to_cert("x", elem, item, m)?;
            Some(format!(
                "{{ let mut l = {m}::List::nil(); for x in {e}.into_iter().rev() {{ l = {m}::List::cons({each}, l); }} l }}"
            ))
        }
    }
}

/// The expression converting the package's value `e` of `cert` to the Rust
/// `rust`, which owns its value.
fn from_cert(e: &str, cert: &str, rust: &str) -> Option<String> {
    let rust = rust.trim();
    match rust {
        "std::string::String" if cert == "Str" => Some(format!("{e}.text().to_string()")),
        n if NATURALS.contains(&n) && cert == "u64" => Some(format!(
            "{n}::try_from({e}).expect(\"a natural the hand-written type holds\")"
        )),
        "bool" if cert == "bool" => Some(e.to_string()),
        _ => {
            if let (Some(t), Some(c)) = (
                argument(rust, "std::option::Option"),
                argument(cert, "Option"),
            ) {
                let each = from_cert("x", c, t)?;
                return Some(format!("{e}.map(|x| {each})"));
            }
            let elem = argument(rust, "std::vec::Vec")?;
            let item = argument(cert, "List")?;
            let each = from_cert("h", item, elem)?;
            Some(format!(
                "{{ let mut v = Vec::new(); let mut c = {e}; while let Some((h, t)) = c.uncons() {{ v.push({each}); c = t; }} v }}"
            ))
        }
    }
}

impl Profile {
    /// The certified crate's `lib.rs`: one module per root, its file the
    /// package source byte for byte.
    pub(crate) fn crate_source(&self, roots: &[Certified]) -> String {
        let mut out = String::from(match (*self == Profile::default(), self.kind) {
            (true, _) => {
                "//! The certified translator: the Rust `lexlean production` exported from the\n\
                 //! lifted translator's definitions, one module per production root, each file\n\
                 //! the certified package's `src/lib.rs` byte for byte. Written by `uorust\n\
                 //! certify sync`; `crates/certified/manifest.toml` keys each by the κ of the\n\
                 //! Rust function it certifies.\n\n\
                 #![forbid(unsafe_code)]\n"
            }
            (false, Kind::Functions) => {
                "//! The certified implementation: the Rust `lexlean production` exported from\n\
                 //! the lifted definitions of the functions in scope, one module per\n\
                 //! production root, each file the certified package's `src/lib.rs` byte for\n\
                 //! byte. Written by `uorust certify sync`; `manifest.toml` keys each by the κ\n\
                 //! of the Rust function it certifies.\n\n\
                 #![forbid(unsafe_code)]\n"
            }
            (false, Kind::Cases) => {
                "//! The certified cases: the Rust `lexlean production` exported from the\n\
                 //! closed assertions of the workspace's tests, one module per production\n\
                 //! root, each file the certified package's `src/lib.rs` byte for byte.\n\
                 //! Written by `uorust certify sync`; `manifest.toml` keys each by the κ of\n\
                 //! the definitions it runs.\n\n\
                 #![forbid(unsafe_code)]\n"
            }
        });
        for r in roots {
            let named = match self.kind {
                Kind::Functions => format!("`{}`, certified as", r.function),
                Kind::Cases => format!("A case of `{}`:", r.function),
            };
            out.push_str(&format!(
                "\n/// {named} `Behavior::{}`.\n#[rustfmt::skip]\n#[allow(missing_docs, dead_code, clippy::all)]\n#[path = \"../roots/{}.rs\"]\npub mod {};\n",
                r.definition,
                r.package,
                self.module(&r.package)
            ));
        }
        out
    }

    /// The module of the certified crate a package is, by its name: the
    /// name without the profile's prefix.
    pub(crate) fn module<'a>(&self, package: &'a str) -> &'a str {
        package
            .strip_prefix(self.package.as_str())
            .unwrap_or(package)
    }

    /// The certified crate's name: the package prefix without the
    /// separator it ends with.
    pub(crate) fn crate_name(&self) -> String {
        self.package.trim_end_matches('_').to_string()
    }

    /// The Cargo manifest of a certified crate whose directory has none: a
    /// package and a workspace of its own, at the edition the bundle's
    /// packages declare, which builds wherever the directory is.
    pub(crate) fn cargo_manifest(&self, edition: Option<&str>) -> String {
        let edition = edition.map_or_else(String::new, |e| format!("edition = \"{e}\"\n"));
        format!(
            "# Written by `uorust certify sync` into a directory that had no Cargo\n\
             # manifest: the crate of the certified packages `manifest.toml` lists.\n\
             [package]\nname = \"{}\"\nversion = \"{PACKAGE_VERSION}\"\n{edition}publish = false\n\n\
             [lib]\npath = \"src/lib.rs\"\n\n[workspace]\n",
            self.crate_name()
        )
    }

    /// The figures of the bootstrap over `register`, a function being
    /// certified when its κ is in `certified`; the reachable ones are
    /// counted when the profile names an entry and the dumps are given.
    pub(crate) fn share(
        &self,
        register: &Register,
        certified: &BTreeSet<String>,
        dumps: Option<&Dumps>,
    ) -> Share {
        let reach = match (&self.entry, dumps) {
            (Some(entry), Some(dumps)) => reachable_from(entry, dumps),
            _ => BTreeSet::new(),
        };
        // (reachable, certified) of each function in scope; a root
        // certifies its κ only while the register lifts it.
        let rows: Vec<(bool, bool)> = register
            .rows
            .iter()
            .filter(|(_, r)| self.in_scope(&r.function))
            .map(|(k, r)| {
                (
                    reach.contains(&r.function),
                    certified.contains(k) && matches!(r.status, Status::Lifted(_)),
                )
            })
            .collect();
        Share {
            translator: rows.len(),
            certified: rows.iter().filter(|(_, c)| *c).count(),
            reachable: rows.iter().filter(|(r, _)| *r).count(),
            reachable_certified: rows.iter().filter(|(r, c)| *r && *c).count(),
        }
    }
}

/// The functions of its crate an entry reaches, by crate path, over the
/// calls, the closures and the function items of their MIR.
pub(crate) fn reachable_from(entry: &str, dumps: &Dumps) -> BTreeSet<String> {
    let krate = entry.split("::").next().unwrap_or_default();
    let empty = BTreeMap::new();
    let bodies = dumps.crates.get(krate).unwrap_or(&empty);
    let closures = dumps.closures.get(krate).unwrap_or(&empty);
    let prefix = format!("{krate}::");
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut work: Vec<&IrBody> = Vec::new();
    let mut closures_seen: BTreeSet<&str> = BTreeSet::new();
    if let Some(body) = bodies.get(entry.strip_prefix(&prefix).unwrap_or(entry)) {
        seen.insert(entry.to_string());
        work.push(body);
    }
    while let Some(body) = work.pop() {
        let (calls, made) = body_mentions(body);
        for call in calls {
            let path = callee_path(&call);
            if let Some(next) = bodies.get(&path) {
                if seen.insert(format!("{prefix}{path}")) {
                    work.push(next);
                }
            }
        }
        for ty in made {
            if let Some((key, next)) = closures.get_key_value(&ty) {
                if closures_seen.insert(key.as_str()) {
                    work.push(next);
                }
            }
        }
    }
    seen
}

/// A callee as MIR prints it, as the path its body is keyed by: generic
/// arguments erased, `<T as Trait>::m` the method of `T`.
pub(crate) fn callee_path(callee: &str) -> String {
    let erased = crate::mir::lower::erase(callee);
    if let Some(rest) = erased.strip_prefix('<') {
        if let Some((ty, tail)) = rest.split_once(" as ") {
            if let Some((_, method)) = tail.rsplit_once(">::") {
                return format!("{ty}::{method}");
            }
        }
    }
    erased
}

/// The callees, and the closures built, of a body.
fn body_mentions(body: &IrBody) -> (Vec<String>, Vec<String>) {
    let mut calls = Vec::new();
    let mut made = Vec::new();
    let operand = |o: &IrOperand, calls: &mut Vec<String>| {
        if let IrOperand::Constant(IrConstant::Path(p)) = o {
            calls.push(p.clone());
        }
    };
    for block in &body.blocks {
        for statement in &block.statements {
            if let IrStatement::Assign(_, rvalue) = statement {
                match rvalue {
                    IrRvalue::Use(o) | IrRvalue::Unary(_, o) | IrRvalue::Cast(_, o, _) => {
                        operand(o, &mut calls);
                    }
                    IrRvalue::Binary(_, a, b) => {
                        operand(a, &mut calls);
                        operand(b, &mut calls);
                    }
                    IrRvalue::Aggregate(kind, ops) => {
                        if let IrAggregate::Closure(ty) = kind {
                            made.push(ty.clone());
                        }
                        for (_, o) in ops {
                            operand(o, &mut calls);
                        }
                    }
                    _ => {}
                }
            }
        }
        if let IrTerminator::Call { callee, args, .. } = &block.terminator {
            calls.push(callee.clone());
            for a in args {
                operand(a, &mut calls);
            }
        }
    }
    (calls, made)
}

/// The domain tag of an output tree's κ.
pub const TREE: &str = "uorust/certified-output/1";

/// The κ of an output tree: SHA-256 over [`TREE`] and each file's path and
/// bytes, in path order, so two trees have one κ exactly when they are byte
/// for byte the same.
pub fn tree_kappa(dir: &Path) -> Result<String, UorustError> {
    let mut files = Vec::new();
    tree_files(dir, dir, &mut files)?;
    files.sort();
    let mut text = Vec::new();
    text.extend_from_slice(TREE.as_bytes());
    for (rel, path) in files {
        let bytes = std::fs::read(&path).map_err(|e| io(&path, e))?;
        text.extend_from_slice(format!("\n{rel}\n{}\n", bytes.len()).as_bytes());
        text.extend_from_slice(&bytes);
    }
    Ok(crate::project::sha(&text))
}

fn tree_files(
    root: &Path,
    dir: &Path,
    out: &mut Vec<(String, std::path::PathBuf)>,
) -> Result<(), UorustError> {
    for entry in std::fs::read_dir(dir).map_err(|e| io(dir, e))? {
        let path = entry.map_err(|e| io(dir, e))?.path();
        if path.is_dir() {
            tree_files(root, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string();
            out.push((rel, path));
        }
    }
    Ok(())
}

/// The differential and the fixpoint over the outputs under `dir`: each
/// `<name>-certified` beside its `<name>-hand`. Returns one line per output,
/// `<name> <κ>`; refused, naming each, when a pair's κ differ or a pair is
/// incomplete.
pub fn agree(dir: &Path) -> Result<Vec<String>, UorustError> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    for entry in std::fs::read_dir(dir).map_err(|e| io(dir, e))? {
        let name = entry
            .map_err(|e| io(dir, e))?
            .file_name()
            .to_string_lossy()
            .into_owned();
        for suffix in ["-certified", "-hand"] {
            if let Some(base) = name.strip_suffix(suffix) {
                names.insert(base.to_string());
            }
        }
    }
    let mut lines = Vec::new();
    let mut disagree = Vec::new();
    for name in &names {
        let certified = dir.join(format!("{name}-certified"));
        let hand = dir.join(format!("{name}-hand"));
        if !certified.is_dir() || !hand.is_dir() {
            disagree.push(format!("{name}: one path wrote no output"));
            continue;
        }
        let (c, h) = (tree_kappa(&certified)?, tree_kappa(&hand)?);
        if c == h {
            lines.push(format!("{name} {c}"));
        } else {
            disagree.push(format!("{name}: certified κ {c}, hand-written κ {h}"));
        }
    }
    if names.is_empty() {
        disagree.push(format!("{}: no outputs", dir.display()));
    }
    if disagree.is_empty() {
        Ok(lines)
    } else {
        Err(UorustError::Disagree(disagree))
    }
}

/// The translator functions of `register` the manifest does not certify,
/// by function, each with its cause.
pub fn uncertified(register: &Register, manifest: &Manifest) -> BTreeMap<String, String> {
    Profile::default().uncertified(register, manifest)
}

/// The ratchet: the hand-written translator may only shrink. Refused when
/// more translator functions are uncertified under the head than under the
/// base, naming each that is uncertified under the head and not under the
/// base, with its cause. Returns the head's count.
pub fn ratchet(
    head: &Register,
    head_manifest: &Manifest,
    base: &Register,
    base_manifest: &Manifest,
) -> Result<usize, UorustError> {
    Profile::default().ratchet(head, head_manifest, base, base_manifest)
}

impl Profile {
    /// The functions of `register` in this profile's scope the manifest
    /// does not certify, by function, each with its cause.
    pub fn uncertified(
        &self,
        register: &Register,
        manifest: &Manifest,
    ) -> BTreeMap<String, String> {
        let certified = manifest.kappas();
        register
            .rows
            .iter()
            // A root certifies its κ only while the register lifts it.
            .filter(|(k, r)| {
                self.in_scope(&r.function)
                    && !(certified.contains(*k) && matches!(r.status, Status::Lifted(_)))
            })
            .map(|(k, r)| {
                let cause = match (&r.status, manifest.refused.get(k)) {
                    (Status::Lifted(d), Some(why)) => {
                        format!("lifted as {d}, not exported: {why}")
                    }
                    (Status::Lifted(d), None) => format!("lifted as {d}, not yet certified"),
                    (Status::Residue(reason, detail), _) => {
                        format!("residue ({reason}): {detail}")
                    }
                    (Status::Foreign(detail), _) => format!("foreign: {detail}"),
                    (Status::Effect(effect), _) => format!("effect: {effect}"),
                };
                (r.function.clone(), cause)
            })
            .collect()
    }

    /// The ratchet over this profile's scope: what is not certified may
    /// only shrink. Refused when more functions in scope are uncertified
    /// under the head than under the base, naming each that is uncertified
    /// under the head and not under the base, with its cause. Returns the
    /// head's count.
    pub fn ratchet(
        &self,
        head: &Register,
        head_manifest: &Manifest,
        base: &Register,
        base_manifest: &Manifest,
    ) -> Result<usize, UorustError> {
        let now = self.uncertified(head, head_manifest);
        let before = self.uncertified(base, base_manifest);
        if now.len() <= before.len() {
            return Ok(now.len());
        }
        let mut rows = vec![format!(
            "{} uncertified {}, {} in the parent",
            now.len(),
            self.measured(),
            before.len()
        )];
        rows.extend(
            now.iter()
                .filter(|(f, _)| !before.contains_key(*f))
                .map(|(f, cause)| format!("{f}: {cause}")),
        );
        Err(UorustError::Grown(rows))
    }

    /// What the ratchet counts, in words: the translator's functions, or
    /// the functions in another workspace's scope.
    pub fn measured(&self) -> &'static str {
        if *self == Profile::default() {
            "translator functions"
        } else {
            "functions in scope"
        }
    }

    /// [`Profile::ratchet`] of a head measured under this profile against a
    /// base that was measured under `base_profile`, when its manifest
    /// states one: refused, before any count, when this scope leaves out a
    /// prefix the base measured, since a function taken out of the scope is
    /// hand-written and no longer counted.
    pub fn ratchet_from(
        &self,
        base_profile: Option<&Profile>,
        head: &Register,
        head_manifest: &Manifest,
        base: &Register,
        base_manifest: &Manifest,
    ) -> Result<usize, UorustError> {
        if self.kind != Kind::Functions {
            return Err(UorustError::Tool(
                "the ratchet measures functions: the manifest is of a cases export".to_string(),
            ));
        }
        let dropped = base_profile.map(|b| b.dropped_by(self)).unwrap_or_default();
        if !dropped.is_empty() {
            let mut rows = vec![format!(
                "the scope measures less than the base's: {} of its prefixes are left out",
                dropped.len()
            )];
            rows.extend(dropped.into_iter().map(|p| {
                let prefix = if p.is_empty() { "every function" } else { &p };
                format!("{prefix}: no longer in scope")
            }));
            return Err(UorustError::Grown(rows));
        }
        self.ratchet(head, head_manifest, base, base_manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn register(rows: &[(&str, &str, Status)]) -> Register {
        let mut r = Register::default();
        for (kappa, function, status) in rows {
            r.rows.insert(
                (*kappa).to_string(),
                crate::register::Row {
                    function: (*function).to_string(),
                    status: status.clone(),
                    divergence: None,
                },
            );
        }
        r
    }

    #[test]
    fn a_root_is_a_lifted_translator_function() {
        let r = register(&[
            (
                "a",
                "uorust::mir::coverage::binop",
                Status::Lifted("Behavior::binop".into()),
            ),
            (
                "b",
                "uorust::project::x",
                Status::Lifted("Behavior::x".into()),
            ),
            (
                "c",
                "uorust::lift::y",
                Status::Residue("mir".into(), "d".into()),
            ),
        ]);
        let roots = roots(&r);
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].definition, "binop");
        assert_eq!(roots[0].package(), "uorust_certified_binop");
        assert_eq!(
            Root {
                definition: "mutTargetPart1".into(),
                ..roots[0].clone()
            }
            .package(),
            "uorust_certified_mut_target_part1"
        );
    }

    #[test]
    fn marking_makes_the_roots_production_and_their_callees_executable() {
        let behavior = "\\begin{lexlean}{Behavior}\ndefinition{\n  name: a,\n  body: x\n}\n\ndefinition{\n  name: b,\n  body: (c (Prelude::p 1)),\n  axioms: []\n}\ndefinition{name: c, parameters: [], body: 1}\n";
        let prelude = "definition{\n  name: p,\n  body: (q 1)\n}\ndefinition{\n  name: q,\n  body: 1\n}\ndefinition{\n  name: r,\n  body: 1\n}\n";
        let modules: BTreeMap<String, String> = [
            ("Behavior".to_string(), behavior.to_string()),
            ("Prelude".to_string(), prelude.to_string()),
        ]
        .into_iter()
        .collect();
        let names: BTreeSet<&str> = ["b"].into_iter().collect();
        let (out, found, refused) = mark(&modules, &names);
        assert!(refused.is_empty());
        assert!(found.contains("b"));
        let b = &out["Behavior"];
        assert!(
            b.starts_with("\\begin{lexlean}{Behavior}\ndefinition{\n  name: a,\n  body: x\n}\n")
        );
        assert!(b.contains("  axioms: [],\n  executable: true,\n  production: {effects: [allocation, overflow, recursion], targets: [rust-std]}\n}\n"));
        assert!(b.contains("definition{name: c, parameters: [], body: 1, executable: true}\n"));
        let p = &out["Prelude"];
        assert!(p.contains("  body: (q 1),\n  executable: true\n}"));
        assert!(p.contains("  name: q,\n  body: 1,\n  executable: true\n}"));
        assert!(p.contains("  name: r,\n  body: 1\n}"));
    }

    #[test]
    fn an_adapter_converts_strings_lists_and_naturals() {
        let root = Certified {
            function: "uorust::mir::lower::implication".into(),
            kappa: "k".into(),
            definition: "implication".into(),
            package: "uorust_certified_implication".into(),
            source: "s".into(),
            certification: "kernel-checked".into(),
        };
        let body = IrBody {
            arity: 2,
            locals: vec![
                "std::string::String".into(),
                "&'a [std::string::String]".into(),
                "&str".into(),
            ],
            ..IrBody::default()
        };
        let source = "pub fn run(p0: List<Str>, p1: Str) -> Str {\n    f0(p0, p1)\n}\n";
        let (name, code) = adapter(&root, &body, source).expect("an adapter");
        assert_eq!(name, "mir_lower_implication");
        assert!(code.contains("pub(crate) fn mir_lower_implication(p0: &&[std::string::String], p1: &&str) -> Option<std::string::String>"), "{code}");
        assert!(code.contains("uorust_certified::implication::run("));
        assert!(code.contains(".text().to_string()"));
        // A type with no conversion has no adapter.
        let opaque = IrBody {
            arity: 1,
            locals: vec!["bool".into(), "&mir::lower::Cx<'_>".into()],
            ..IrBody::default()
        };
        assert_eq!(
            adapter(&root, &opaque, "pub fn run(p0: Adt3) -> bool {\n}\n"),
            None
        );
        // An overflowing root's result is unwrapped.
        let nat = IrBody {
            arity: 1,
            locals: vec!["u32".into(), "u32".into()],
            ..IrBody::default()
        };
        let (_, code) =
            adapter(&root, &nat, "pub fn run(p0: u64) -> R<u64> {\n}\n").expect("an adapter");
        assert!(code.contains("the certified root overflowed"));
    }

    // The planted defect of the differential: one byte apart is refused.
    #[test]
    fn the_differential_refuses_one_byte_apart() {
        let dir = std::env::temp_dir().join(format!("uorust-agree-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (name, text) in [
            ("core-certified", "a"),
            ("core-hand", "a"),
            ("self-certified", "b"),
            ("self-hand", "c"),
        ] {
            std::fs::create_dir_all(dir.join(name)).unwrap();
            std::fs::write(dir.join(name).join("frontier.toml"), text).unwrap();
        }
        match agree(&dir) {
            Err(UorustError::Disagree(rows)) => {
                assert_eq!(rows.len(), 1);
                assert!(rows[0].starts_with("self: certified κ"));
            }
            other => panic!("not refused: {other:?}"),
        }
        std::fs::write(dir.join("self-hand").join("frontier.toml"), "b").unwrap();
        let lines = agree(&dir).expect("agrees");
        assert_eq!(lines.len(), 2);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_diagnostic_names_its_root() {
        let m = "production root `Uorust.Behavior.exprIsFlat` is not eligible";
        assert_eq!(
            first_root(m, "Uorust.Behavior."),
            Some("exprIsFlat".to_string())
        );
        assert_eq!(
            first_root("root `Uorust.Prelude.x`", "Uorust.Behavior."),
            None
        );
    }

    #[test]
    fn a_trait_call_is_the_method_of_its_type() {
        assert_eq!(
            callee_path("<lift::Types as std::default::Default>::default"),
            "lift::Types::default"
        );
        assert_eq!(callee_path("mir::parse::bodies"), "mir::parse::bodies");
    }

    // The planted defect: a new translator function that does not lift, and a
    // certified one whose κ changed, grow the uncertified surface; the
    // ratchet refuses both, naming each with its cause.
    #[test]
    fn the_ratchet_refuses_a_grown_surface_naming_each_new_row() {
        let base = register(&[
            (
                "k1",
                "uorust::mir::lower::a",
                Status::Lifted("Behavior::a".into()),
            ),
            (
                "k2",
                "uorust::mir::lower::b",
                Status::Residue("mir".into(), "mir::Ty: x".into()),
            ),
        ]);
        let certified = Manifest {
            roots: vec![Certified {
                function: "uorust::mir::lower::a".into(),
                kappa: "k1".into(),
                definition: "a".into(),
                package: "uorust_certified_a".into(),
                source: "s".into(),
                certification: "kernel-checked".into(),
            }],
            ..Manifest::default()
        };
        assert_eq!(ratchet(&base, &certified, &base, &certified), Ok(1));
        let grown = register(&[
            (
                "k1",
                "uorust::mir::lower::a",
                Status::Lifted("Behavior::a".into()),
            ),
            (
                "k2",
                "uorust::mir::lower::b",
                Status::Residue("mir".into(), "mir::Ty: x".into()),
            ),
            (
                "k3",
                "uorust::mir::lower::c",
                Status::Residue("mir".into(), "mir::Rvalue::Use".into()),
            ),
        ]);
        match ratchet(&grown, &certified, &base, &certified) {
            Err(UorustError::Grown(rows)) => {
                assert_eq!(rows.len(), 2);
                assert!(rows[1].contains("uorust::mir::lower::c: residue (mir): mir::Rvalue::Use"));
            }
            other => panic!("not refused: {other:?}"),
        }
        let edited = register(&[
            (
                "k9",
                "uorust::mir::lower::a",
                Status::Lifted("Behavior::a".into()),
            ),
            (
                "k2",
                "uorust::mir::lower::b",
                Status::Residue("mir".into(), "mir::Ty: x".into()),
            ),
        ]);
        assert!(matches!(
            ratchet(&edited, &certified, &base, &certified),
            Err(UorustError::Grown(_))
        ));
        // Code outside the translator is not measured.
        let outside = register(&[
            (
                "k1",
                "uorust::mir::lower::a",
                Status::Lifted("Behavior::a".into()),
            ),
            (
                "k2",
                "uorust::mir::lower::b",
                Status::Residue("mir".into(), "mir::Ty: x".into()),
            ),
            (
                "k4",
                "uorust::project::d",
                Status::Residue("mir".into(), "e".into()),
            ),
        ]);
        assert_eq!(ratchet(&outside, &certified, &base, &certified), Ok(1));
    }

    #[test]
    fn the_manifest_round_trips() {
        let m = Manifest {
            result: "r".into(),
            compiler: "0.4.0".into(),
            roots: vec![Certified {
                function: "uorust::mir::coverage::binop".into(),
                kappa: "k".into(),
                definition: "binop".into(),
                package: "uorust_certified_binop".into(),
                source: "s".into(),
                certification: "kernel-checked".into(),
            }],
            share: Share {
                translator: 3,
                certified: 1,
                reachable: 2,
                reachable_certified: 1,
            },
            seconds_per_root: Some(9),
            refused: [(
                "k2".to_string(),
                "reaches Behavior::walk, a \"function\"".to_string(),
            )]
            .into_iter()
            .collect(),
        };
        assert_eq!(Manifest::parse(Path::new("m"), &m.to_text()), Ok(m));
    }

    #[test]
    fn a_plan_sizes_its_shards_and_names_the_compiler_refusals() {
        let rows: Vec<(String, String, Status)> = (0..20)
            .map(|i| {
                (
                    format!("k{i}"),
                    format!("uorust::mir::f{i}"),
                    Status::Lifted(format!("Behavior::f{i}")),
                )
            })
            .collect();
        let reg = register(
            &rows
                .iter()
                .map(|(k, f, s)| (k.as_str(), f.as_str(), s.clone()))
                .collect::<Vec<_>>(),
        );
        let mut prior = Manifest::default();
        prior
            .refused
            .insert("k0".into(), "built alone: LLV7015 certificate B: x".into());
        prior.refused.insert("k1".into(), UNBUILT.into());
        // Nineteen to build, at most eight a shard however long the budget.
        let (shards, diagnose) = plan(&reg, &prior, u64::MAX / 2);
        assert_eq!((shards, diagnose), (3, vec!["f0".to_string()]));
        // A budget of one build of two roots: ten shards.
        prior.seconds_per_root = Some(1);
        assert_eq!(plan(&reg, &prior, 2 * BUILDS_PER_SHARD).0, 10);
    }

    #[test]
    fn the_kept_bundles_are_read_in_build_order() {
        let kept = std::env::temp_dir().join(format!("uorust-kept-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&kept);
        for n in ["1", "2", "10", "notes"] {
            std::fs::create_dir_all(kept.join(n)).expect("a bundle directory");
            std::fs::write(kept.join(n).join("result.json"), "{}").expect("a result");
        }
        std::fs::create_dir_all(kept.join("3")).expect("an empty directory");
        let found = bundles(&kept).expect("the bundles");
        let _ = std::fs::remove_dir_all(&kept);
        let names: Vec<_> = found
            .iter()
            .filter_map(|p| p.file_name()?.to_str().map(str::to_string))
            .collect();
        assert_eq!(names, ["1", "2", "10"]);
    }

    // Class: the implementation produced from the model, for any workspace.

    /// A workspace that is not the translator's: a meter with two crates.
    fn meter() -> Profile {
        Profile {
            scope: vec!["meter::".to_string()],
            entry: None,
            package: "meter_certified_".to_string(),
            effects: vec!["overflow".to_string()],
            target: "rust-core".to_string(),
            kind: Kind::Functions,
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("uorust-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        dir
    }

    // The default of every step is the translator's own export, so its gate
    // reads as before: the constants, the committed manifest byte for byte,
    // and the committed crate of its roots byte for byte.
    #[test]
    fn the_default_profile_is_the_translators_own() {
        let own = Profile::default();
        assert_eq!(own.scope, SCOPE);
        assert_eq!(own.entry.as_deref(), Some(ENTRY));
        assert_eq!(own.package, PACKAGE);
        assert_eq!(own.effects, EFFECTS);
        assert_eq!(own.target, TARGET);
        assert_eq!(own.kind, Kind::Functions);
        assert_eq!(Profile::resolve(&Options::default()), Ok(own.clone()));
        assert!(SCOPE.iter().all(|s| in_scope(&format!("{s}f"))));
        let certified = Path::new(env!("CARGO_MANIFEST_DIR")).join("../certified");
        let path = certified.join("manifest.toml");
        let text = std::fs::read_to_string(&path).expect("the committed manifest");
        let manifest = Manifest::parse(&path, &text).expect("it parses");
        assert_eq!(manifest.to_text(), text);
        assert_eq!(manifest.text(&own), text);
        assert_eq!(Profile::recorded(&path, &text), Ok(own.clone()));
        let lib = std::fs::read_to_string(certified.join("src/lib.rs")).expect("its crate");
        assert_eq!(own.crate_source(&manifest.roots), lib);
    }

    // Step 1, the export. The planted defect: roots read through the
    // translator's constants give this workspace none.
    #[test]
    fn another_workspace_has_its_own_roots_packages_effects_and_target() {
        let r = register(&[
            ("a", "meter::read", Status::Lifted("Behavior::read".into())),
            (
                "b",
                "gauge::show",
                Status::Lifted("Behavior::showAll".into()),
            ),
            (
                "c",
                "meter::write",
                Status::Residue("form".into(), "d".into()),
            ),
        ]);
        assert!(roots(&r).is_empty());
        let meter = meter();
        let found = meter.roots(&r);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].function, "meter::read");
        assert_eq!(found[0].package(), "meter_certified_read");
        // Every function of the register, under the empty prefix.
        let every = Profile {
            scope: vec![String::new()],
            ..meter.clone()
        };
        let packages: Vec<String> = every.roots(&r).iter().map(Root::package).collect();
        assert_eq!(
            packages,
            ["meter_certified_read", "meter_certified_show_all"]
        );
        let modules: BTreeMap<String, String> = [
            (
                "Behavior".to_string(),
                "definition{\n  name: read,\n  body: 1\n}\n".to_string(),
            ),
            ("Prelude".to_string(), String::new()),
        ]
        .into_iter()
        .collect();
        let names: BTreeSet<&str> = ["read"].into_iter().collect();
        let (out, found, refused) = meter.mark(&modules, &names);
        assert!(refused.is_empty() && found.contains("read"));
        assert!(out["Behavior"].contains(
            "  executable: true,\n  production: {effects: [overflow], targets: [rust-core]}\n}\n"
        ));
        // An adapter calls the workspace's own certified crate and is named
        // by the function's path in its own crate.
        let certified = Certified {
            function: "meter::dial::read".into(),
            kappa: "a".into(),
            definition: "read".into(),
            package: "meter_certified_read".into(),
            source: "s".into(),
            certification: "kernel-checked".into(),
        };
        let body = IrBody {
            arity: 2,
            locals: vec!["u32".into(), "&[u32]".into(), "u32".into()],
            ..IrBody::default()
        };
        let source = "pub fn run(p0: List<u64>, p1: u64) -> R<u64> {\n}\n";
        let (name, code) = meter
            .adapter(&certified, &body, source)
            .expect("an adapter");
        assert_eq!(name, "dial_read");
        assert!(code.contains("pub(crate) fn dial_read(p0: &&[u32], p1: &u32) -> Option<u32>"));
        assert!(code.contains("meter_certified::read::run("), "{code}");
        // The request's descriptor is the profile's target's.
        let targets =
            r#"{"targets":[{"id":"rust-core","digest":"c0"},{"id":"rust-std","digest":"5d"}]}"#;
        assert_eq!(meter.descriptor(targets), Ok("c0".to_string()));
        assert_eq!(Profile::default().descriptor(targets), Ok("5d".to_string()));
        let none = Profile {
            target: "rust-wasm".to_string(),
            ..meter
        };
        assert!(matches!(
            none.descriptor(targets),
            Err(UorustError::Malformed { .. })
        ));
    }

    #[test]
    fn a_workspace_declares_its_profile_in_its_cargo_manifest() {
        let dir = scratch("profile").join("meter-ws");
        std::fs::create_dir_all(&dir).expect("the workspace");
        let cargo = dir.join("Cargo.toml");
        std::fs::write(&cargo, "[workspace]\nmembers = [\"meter\"]\n").expect("write");
        // What the workspace gives without a table: everything in scope,
        // packages named after its directory, every effect, the std target.
        let derived = Profile::of_workspace(&dir).expect("a profile");
        assert_eq!(derived.scope, [""]);
        assert!(derived.in_scope("meter::read") && derived.in_scope("gauge::show"));
        assert_eq!(derived.entry, None);
        assert_eq!(derived.package, "meter_ws_certified_");
        assert_eq!(derived.effects, EFFECTS);
        assert_eq!(derived.target, TARGET);
        std::fs::write(
            &cargo,
            "[workspace]\nmembers = [\"meter\"]\n\n[workspace.metadata.uorust.certify]\nscope = [\"meter::\"]\nentry = \"meter::run\"\neffects = [\"overflow\"]\ntarget = \"rust-core\"\n",
        )
        .expect("write");
        let declared = Profile::of_workspace(&dir).expect("a profile");
        assert_eq!(declared.scope, ["meter::"]);
        assert_eq!(declared.entry.as_deref(), Some("meter::run"));
        assert_eq!(declared.package, "meter_ws_certified_");
        assert_eq!(declared.effects, ["overflow"]);
        assert_eq!(declared.target, "rust-core");
        // An option replaces what the workspace declares.
        let options = Options {
            workspace: Some(dir.clone()),
            package: Some("m_".to_string()),
            cases: true,
            ..Options::default()
        };
        let resolved = Profile::resolve(&options).expect("a profile");
        assert_eq!(
            (resolved.package.as_str(), resolved.kind),
            ("m_", Kind::Cases)
        );
        assert_eq!(resolved.scope, ["meter::"]);
        // What an export records is read back whole, whatever the base.
        let table: toml::Value = resolved.rows().parse().expect("rows");
        assert_eq!(
            Profile::parse(&cargo, &table, Profile::default()),
            Ok(resolved)
        );
        for bad in [
            "effects = [\"io\"]\n",
            "roots = \"tests\"\n",
            "scopes = []\n",
            "package = \"Meter-\"\n",
            "package = \"9meter_\"\n",
            "scope = []\n",
        ] {
            std::fs::write(&cargo, format!("[package.metadata.uorust.certify]\n{bad}"))
                .expect("write");
            assert!(
                matches!(
                    Profile::of_workspace(&dir),
                    Err(UorustError::Malformed { .. })
                ),
                "{bad}"
            );
        }
        let _ = std::fs::remove_dir_all(dir.parent().expect("the scratch"));
    }

    /// What `certify production` keeps of one bundle of one package, under
    /// `kept`: the result, the package's source and Cargo manifest, the κ it
    /// was exported at, the refusals, the timing and the profile.
    fn kept_bundle(kept: &Path, profile: &Profile, root: &Root, source: &str) {
        let package = root.package();
        let directory = format!("packages/{}/{package}", profile.target);
        let bundle = kept.join("1");
        std::fs::create_dir_all(bundle.join(&directory).join("src")).expect("the package");
        std::fs::write(bundle.join(&directory).join("src/lib.rs"), source).expect("write");
        std::fs::write(
            bundle.join(&directory).join("Cargo.toml"),
            format!("[package]\nname = \"{package}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        )
        .expect("write");
        let result = serde_json::json!({
            "result_id": "r1",
            "roots": [{"outputs": [{
                "certification": {"status": "kernel-checked"},
                "package": {
                    "directory": directory,
                    "files": [{
                        "path": format!("{directory}/src/lib.rs"),
                        "sha256": crate::project::sha(source.as_bytes()),
                    }],
                },
            }]}],
        });
        std::fs::write(bundle.join("result.json"), result.to_string()).expect("write");
        std::fs::write(
            kept.join(KAPPAS),
            format!("{package} = \"{}\"\n", root.kappa),
        )
        .expect("write");
        std::fs::write(
            kept.join(REFUSALS),
            "[[refused]]\nkappa = \"k2\"\ncause = \"LLT4005 not eligible\"\n",
        )
        .expect("write");
        std::fs::write(kept.join(TIMING), "seconds = 12\nroots = 1\n").expect("write");
        std::fs::write(kept.join(PROFILE), profile.rows()).expect("write");
    }

    // Step 2, taking the bundle in. The planted defects: a package taken
    // whatever its function's κ, and a source that is not the one the
    // bundle's result records.
    #[test]
    fn a_sync_takes_the_packages_into_the_named_directory_keyed_by_kappa() {
        let tmp = scratch("sync");
        let (kept, dir) = (tmp.join("kept"), tmp.join("meter-certified"));
        let profile = Profile {
            target: TARGET.to_string(),
            ..meter()
        };
        let rows = |kappa: &'static str| {
            register(&[
                (
                    kappa,
                    "meter::read",
                    Status::Lifted("Behavior::read".into()),
                ),
                (
                    "k2",
                    "meter::scale",
                    Status::Lifted("Behavior::scale".into()),
                ),
                ("k3", "gauge::show", Status::Lifted("Behavior::show".into())),
            ])
        };
        let exported = rows("k1");
        let source = "pub fn run(p0: u64) -> u64 {\n    p0\n}\n";
        kept_bundle(&kept, &profile, &profile.roots(&exported)[0], source);
        assert_eq!(kept_profile(&kept), Ok(profile.clone()));
        let take = |register: &Register| {
            let s = Sync {
                kept: &kept,
                dir: &dir,
                register,
                projection: None,
                adapters: None,
                mir: None,
            };
            sync(&profile, &s, &profile.roots(register), None, "0.4.0")
        };
        let manifest = take(&exported).expect("a sync");
        assert_eq!(manifest.kappas(), ["k1".to_string()].into_iter().collect());
        assert_eq!(manifest.roots[0].function, "meter::read");
        assert_eq!(manifest.roots[0].package, "meter_certified_read");
        assert_eq!(manifest.refused["k2"], "LLT4005 not eligible");
        assert_eq!(manifest.seconds_per_root, Some(12));
        // Two functions in scope, one certified; the gauge is not measured.
        assert_eq!(
            (manifest.share.translator, manifest.share.certified),
            (2, 1)
        );
        let read = |rel: &str| std::fs::read_to_string(dir.join(rel)).expect(rel);
        assert_eq!(read("roots/meter_certified_read.rs"), source);
        assert!(read("src/lib.rs")
            .contains("#[path = \"../roots/meter_certified_read.rs\"]\npub mod read;\n"));
        let cargo = read("Cargo.toml");
        assert!(cargo.contains("name = \"meter_certified\"\n"), "{cargo}");
        assert!(cargo.contains("edition = \"2021\"\n") && cargo.contains("[workspace]\n"));
        // The manifest states the profile it was taken in under.
        let text = read("manifest.toml");
        let path = dir.join("manifest.toml");
        assert_eq!(Profile::recorded(&path, &text), Ok(profile.clone()));
        assert_eq!(Manifest::parse(&path, &text), Ok(manifest.clone()));
        assert!(!text.contains("entry = \"uorust"));
        // A caller's own Cargo manifest is kept.
        std::fs::write(dir.join("Cargo.toml"), "# the caller's\n").expect("write");
        // The function changed since the export: a new question, whose
        // package is not taken, so it is uncertified until rebuilt.
        let changed = rows("k9");
        let later = take(&changed).expect("a sync");
        assert!(later.roots.is_empty());
        assert!(!dir.join("roots/meter_certified_read.rs").exists());
        assert_eq!(read("Cargo.toml"), "# the caller's\n");
        assert_eq!(
            profile.uncertified(&changed, &later)["meter::read"],
            "lifted as Behavior::read, not yet certified"
        );
        assert_eq!(
            profile
                .uncertified(&exported, &manifest)
                .keys()
                .collect::<Vec<_>>(),
            ["meter::scale"]
        );
        // A source that is not the one the result records is refused.
        std::fs::write(
            kept.join("1/packages/rust-std/meter_certified_read/src/lib.rs"),
            "pub fn run(p0: u64) -> u64 {\n    0\n}\n",
        )
        .expect("plant");
        match take(&exported) {
            Err(UorustError::Malformed { reason, .. }) => {
                assert!(reason.contains("the result records"), "{reason}");
            }
            other => panic!("not refused: {other:?}"),
        }
        let _ = std::fs::remove_dir_all(tmp);
    }

    // Step 3, the ratchet. The planted defect: a head measured through the
    // translator's scope sees none of this workspace's functions and passes.
    #[test]
    fn the_ratchet_holds_any_scope_and_a_scope_only_widens() {
        let meter = meter();
        let base = register(&[
            ("k1", "meter::read", Status::Lifted("Behavior::read".into())),
            (
                "k2",
                "meter::scale",
                Status::Residue("form".into(), "x".into()),
            ),
        ]);
        let certified = Manifest {
            roots: vec![Certified {
                function: "meter::read".into(),
                kappa: "k1".into(),
                definition: "read".into(),
                package: "meter_certified_read".into(),
                source: "s".into(),
                certification: "kernel-checked".into(),
            }],
            ..Manifest::default()
        };
        assert_eq!(meter.ratchet(&base, &certified, &base, &certified), Ok(1));
        let grown = register(&[
            ("k9", "meter::read", Status::Lifted("Behavior::read".into())),
            (
                "k2",
                "meter::scale",
                Status::Residue("form".into(), "x".into()),
            ),
            (
                "k3",
                "meter::sum",
                Status::Effect("Fs effect: fs::read".into()),
            ),
            (
                "k4",
                "gauge::show",
                Status::Residue("form".into(), "y".into()),
            ),
        ]);
        // The translator's scope measures nothing here.
        assert_eq!(ratchet(&grown, &certified, &base, &certified), Ok(0));
        match meter.ratchet(&grown, &certified, &base, &certified) {
            Err(UorustError::Grown(rows)) => assert_eq!(
                rows,
                [
                    "3 uncertified functions in scope, 1 in the parent",
                    "meter::read: lifted as Behavior::read, not yet certified",
                    "meter::sum: effect: Fs effect: fs::read"
                ]
            ),
            other => panic!("not refused: {other:?}"),
        }
        // A head that measures less than its base is refused before any
        // count; one that measures more is counted under its own scope.
        let wide = Profile {
            scope: vec![String::new()],
            ..meter.clone()
        };
        match meter.ratchet_from(Some(&wide), &base, &certified, &base, &certified) {
            Err(UorustError::Grown(rows)) => {
                assert_eq!(rows[1], "every function: no longer in scope");
            }
            other => panic!("not refused: {other:?}"),
        }
        assert_eq!(
            wide.ratchet_from(Some(&meter), &base, &certified, &base, &certified),
            Ok(1)
        );
        assert_eq!(
            meter.ratchet_from(None, &base, &certified, &base, &certified),
            Ok(1)
        );
        let cases = Profile {
            kind: Kind::Cases,
            ..meter
        };
        assert!(matches!(
            cases.ratchet_from(None, &base, &certified, &base, &certified),
            Err(UorustError::Tool(_))
        ));
    }

    // Step 4, the agreement's export: the roots of a cases export are the
    // cases the projection states, read from its modules.
    #[test]
    fn the_roots_of_a_cases_export_are_the_projections_cases() {
        let tmp = scratch("cases");
        std::fs::create_dir_all(tmp.join("src")).expect("the sources");
        std::fs::write(
            tmp.join("src/Behavior.lex.tex"),
            "definition{\n  name: read,\n  parameters: [n : Nat],\n  result: Nat,\n  body: n\n}\n\ndefinition{\n  name: acceptMeterReads0Root,\n  parameters: [],\n  result: Bool,\n  body: ((read 2) =? 2)\n}\n\ntheorem{\n  name: acceptMeterReads0RootHolds,\n  parameters: [],\n  statement: ((acceptMeterReads0Root) = true),\n  proof: decide{}\n}\n",
        )
        .expect("write");
        std::fs::write(tmp.join("src/Prelude.lex.tex"), "").expect("write");
        let r = register(&[("k1", "meter::read", Status::Lifted("Behavior::read".into()))]);
        let functions = meter();
        let cases = Profile {
            kind: Kind::Cases,
            ..functions.clone()
        };
        let of = |p: &Profile| -> Vec<String> {
            let roots = p.export_roots(&tmp, &r).expect("the roots");
            roots.iter().map(Root::package).collect()
        };
        assert_eq!(of(&functions), ["meter_certified_read"]);
        // A name longer than a crate's may be is shortened to the bound,
        // the same at every export and each root's alone.
        let long = |end: &str| Root {
            definition: format!(
                "acceptMeterReadsEveryRoomOfTheRosterInItsOwnOrderAndKeepsThem{end}"
            ),
            ..functions.roots(&r)[0].clone()
        };
        let (first, second) = (long("0Root").package(), long("1Root").package());
        assert_eq!((first.len(), second.len()), (CRATE_NAME, CRATE_NAME));
        assert!(
            first.starts_with("meter_certified_accept_meter_reads_every_room_") && first != second
        );
        assert_eq!(first, long("0Root").package());
        assert_eq!(of(&cases), ["meter_certified_accept_meter_reads0_root"]);
        let found = cases.export_roots(&tmp, &r).expect("the roots");
        assert_eq!(found[0].function, "meter::read");
        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn a_refusals_class_is_its_cause_without_the_projects_names() {
        let admission = Admission {
            roots: 4,
            admitted: vec!["a".into()],
            refused: [
                ("k1", "LLT4005 phase production: production root `Meter.Behavior.read` is not eligible for target `rust-std`: effect mismatch: construct `prim append` realizes effect `allocation`, which the root does not admit (in `Meter.Behavior.read`, reached by Meter.Behavior.read; 1 violation(s) in total)"),
                ("k2", "LLT4005 phase production: production root `Meter.Behavior.scale` is not eligible for target `rust-std`: effect mismatch: construct `prim append` realizes effect `allocation`, which the root does not admit (in `Meter.Behavior.scaleLoop1`, reached by Meter.Behavior.scale -> Meter.Behavior.scaleLoop1; 2 violation(s) in total)"),
                ("k3", "reaches Behavior::walk, a definition with a function parameter, which no production root may"),
            ]
            .into_iter()
            .map(|(k, c)| (k.to_string(), c.to_string()))
            .collect(),
            prefix: "Meter".to_string(),
        };
        assert_eq!(
            admission.causes(),
            [
                ("LLT4005 phase production: production root `_` is not eligible for target `rust-std`: effect mismatch: construct `prim append` realizes effect `allocation`, which the root does not admit".to_string(), 2),
                ("reaches _ a definition with a function parameter, which no production root may".to_string(), 1),
            ]
        );
    }
}
