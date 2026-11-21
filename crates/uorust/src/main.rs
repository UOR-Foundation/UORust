//! `uorust`: project a Cargo workspace into a Prism project.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use uorust::{Options, UorustError};

/// Project any Cargo workspace into a complete, checked Prism project.
#[derive(Parser)]
#[command(name = "uorust", version)]
struct Cli {
    /// Which translator runs the functions the certified translator covers:
    /// the certified one, or the hand-written one, the differential's second
    /// path.
    #[arg(long, global = true, value_enum, default_value_t = Translator::Certified)]
    translator: Translator,
    #[command(subcommand)]
    command: Command,
}

/// A path of the translator.
#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Translator {
    /// The certified packages crates/certified holds.
    Certified,
    /// The hand-written functions they certify.
    Hand,
}

/// What to do with the projection.
#[derive(Subcommand)]
enum Command {
    /// Write the projection of a workspace into a directory.
    Project(Args),
    /// Check that a directory holds exactly the projection of a workspace.
    Check(Args),
    /// Dump the MIR of every library and binary target of a workspace, for
    /// the MIR path of the lift (`--mir`).
    Mir(MirArgs),
    /// Decide every translation of the MIR path with `lexlean verify` and
    /// write the verdict register (`--verdicts`).
    Prove(ProveArgs),
    /// Translate every body of core's MIR, as `scripts/mir/dump-core.sh`
    /// dumps it one module at a time, and write the core register.
    Core(CoreArgs),
    /// Translate every body of std's MIR, as `scripts/mir/dump-lib.sh std`
    /// dumps it one module at a time, and write the std register.
    Std(CoreArgs),
    /// Embed every body of a library crate's MIR as a value of the MIR datatype of the
    /// LexLean semantics, and write the project, with the census of the
    /// constructs the evaluator does not define, for `lexlean verify`.
    Embed(EmbedArgs),
    /// The layouts rustc chose for the aggregates a library's dumps print,
    /// as the dump step asks for them (`scripts/mir/dump-lib.sh`).
    Layouts(LayoutsArgs),
    /// Dump the MIR of core, alloc or std of the pinned toolchain's
    /// rust-src, one directory per module, as `scripts/mir/dump-lib.sh`
    /// does, with the dump's key and the digest of what it wrote
    /// (`library-mir.toml`): what `uorust embed --link <crate>=<dir>` links.
    LibraryMir(LibraryMirArgs),
    /// Fetch a crate of crates.io, dump the MIR of its library, translate
    /// every body and write its register, `crates/uorust/crates/<name>`.
    Crate(CrateArgs),
    /// Merge the module files a sharded run wrote into a register, refused
    /// when a body it records as translated is residue in one, and rewrite
    /// its summary.
    Merge(MergeArgs),
    /// Verify the bridge's project, setting each theorem Lean refutes
    /// aside as pending until the rest verify, and stamp each translated
    /// row of the register's module files `bridged` or `pending`.
    Bridge(BridgeArgs),
    /// Carry a register's bridge stamps onto its census taken again on a
    /// newer head, each row's only while it is the row they were measured
    /// on.
    Carry(CarryArgs),
    /// Admit a lift register against its parent's (a register file or a
    /// `frontier.toml` each), by the register's law per function: a lifted
    /// function stays lifted, and an unchanged one stays under its construct
    /// until the candidate shows that construct no longer stops it (gone
    /// from the constructs its signature states, its callee lifted, or no
    /// effect reached); refused, naming each function re-decided.
    Admit(AdmitArgs),
    /// Give every numeric literal of a workspace a home in the model: the
    /// figure register, the generated package, the rewrite and the gate.
    #[command(subcommand)]
    Figures(Figures),
    /// Print the route table.
    Routes,
    /// The certified translator: export the lifted translator through
    /// `lexlean production`, take the bundle in, and hold the ratchet.
    #[command(subcommand)]
    Certify(Certify),
}

/// The steps of the certified translator (`just certified`).
#[derive(Subcommand)]
enum Certify {
    /// Export the lifted translator with `lexlean production build`: the
    /// self projection with every lifted translator definition a production
    /// root, refusing each the compiler finds ineligible, and the bundle
    /// replayed with `verify-artifact`.
    Production {
        /// The self projection (`uorust project . --out ...`).
        #[arg(long)]
        projection: PathBuf,
        /// The self register, `frontier.toml`.
        #[arg(long)]
        register: PathBuf,
        /// The output of `lexlean production targets`.
        #[arg(long)]
        targets: PathBuf,
        /// The directory the production project is written to.
        #[arg(long)]
        out: PathBuf,
        /// Where the bundles' results, package sources and refusals are
        /// kept for `certify sync`.
        #[arg(long)]
        kept: PathBuf,
        /// The manifest of the last export: the roots it refused at an
        /// unchanged κ and compiler are refused again without a build, and
        /// the roots it certified at an unchanged identity are carried.
        #[arg(long)]
        prior: PathBuf,
        /// A directory holding the prior export's package sources as a
        /// certified crate does (`roots/<package>.rs`), searched in order
        /// (repeatable); the prior manifest's own directory when none is
        /// given.
        #[arg(long)]
        carry: Vec<PathBuf>,
        /// Seconds the export may build for: the builds still running then
        /// are stopped, and the roots not built are refused for this export.
        #[arg(long)]
        within: Option<u64>,
        /// This export is shard I (from 0) of `--shards` N: it builds every
        /// Nth root from the Ith.
        #[arg(long, requires = "shards")]
        shard: Option<usize>,
        /// The shards of a sharded export.
        #[arg(long, requires = "shard")]
        shards: Option<usize>,
        #[command(flatten)]
        profile: ProfileArgs,
    },
    /// Plan a sharded export: print, as JSON, the shards that build the
    /// roots the prior manifest did not refuse within `budget` seconds, and
    /// the roots whose compiler refusal the diagnose job keeps in full.
    Plan {
        /// The self register, `frontier.toml`.
        #[arg(long)]
        register: PathBuf,
        /// The manifest of the last export.
        #[arg(long)]
        prior: PathBuf,
        /// The seconds each shard may build for.
        #[arg(long)]
        budget: u64,
        /// The projection, which the roots of a cases export are read from,
        /// and with `--targets` the roots' identities, so that the plan
        /// leaves out the roots the prior export carries.
        #[arg(long)]
        projection: Option<PathBuf>,
        /// The output of `lexlean production targets`.
        #[arg(long, requires = "projection")]
        targets: Option<PathBuf>,
        /// Where the prior export's package sources are (as `certify
        /// production --carry`).
        #[arg(long)]
        carry: Vec<PathBuf>,
        #[command(flatten)]
        profile: ProfileArgs,
    },
    /// Build one root alone and keep everything lexlean prints of it.
    Diagnose {
        /// The self projection.
        #[arg(long)]
        projection: PathBuf,
        /// The self register, `frontier.toml`.
        #[arg(long)]
        register: PathBuf,
        /// The output of `lexlean production targets`.
        #[arg(long)]
        targets: PathBuf,
        /// The root, by its definition.
        #[arg(long)]
        root: String,
        /// The production project, with `build.log`.
        #[arg(long)]
        out: PathBuf,
        #[command(flatten)]
        profile: ProfileArgs,
    },
    /// Decide, without a build, which roots of an export the compiler
    /// admits: the production project with every root marked is checked
    /// (`lexlean check`, which runs no Lean), each root it finds ineligible
    /// is refused with its diagnostic and the rest checked again. Prints the
    /// counts and each refusal, and writes `refusals.toml` in `out`.
    Roots {
        /// The projection (`uorust project <workspace> --out ...`).
        #[arg(long)]
        projection: PathBuf,
        /// The lift register of that projection.
        #[arg(long)]
        register: PathBuf,
        /// The output of `lexlean production targets`.
        #[arg(long)]
        targets: PathBuf,
        /// The directory the production project is written to.
        #[arg(long)]
        out: PathBuf,
        #[command(flatten)]
        profile: ProfileArgs,
    },
    /// Take a published bundle into the certified crate and write its
    /// manifest with the bootstrap's share, under the profile the export
    /// recorded in what it kept.
    Sync {
        /// What `certify production` kept of the bundle.
        #[arg(long)]
        kept: PathBuf,
        /// The register, `frontier.toml`.
        #[arg(long)]
        register: PathBuf,
        /// The workspace's MIR dumps (`uorust mir`): the adapters' types,
        /// and the share reachable from the profile's entry.
        #[arg(long, requires = "workspace")]
        mir: Option<PathBuf>,
        /// The workspace root the dumps are of.
        #[arg(long, requires = "mir")]
        workspace: Option<PathBuf>,
        /// The certified crate's directory.
        #[arg(long)]
        dir: PathBuf,
        /// The adapters' module that is written, of a workspace that runs
        /// its certified packages in place of its functions.
        #[arg(long, requires = "mir")]
        adapters: Option<PathBuf>,
        /// The projection of the register, which the roots of a cases
        /// export are read from.
        #[arg(long)]
        projection: Option<PathBuf>,
    },
    /// The differential and the fixpoint: every output directory under `dir`
    /// written by the certified translator (`<name>-certified`) has the κ of
    /// the one the hand-written translator wrote (`<name>-hand`); prints each
    /// output's κ, and is refused on any disagreement.
    ///
    /// With `--cases`, the agreement of any workspace: every certified case
    /// taken into that directory (`certify production --cases`, `certify
    /// sync`) answers true when rustc runs it, and the test of the workspace
    /// that states it passes; prints each case with the functions it asserts
    /// on, and is refused on any case that parts.
    Agree {
        /// The directory of the outputs.
        #[arg(long, required_unless_present = "cases")]
        dir: Option<PathBuf>,
        /// The certified crate of a cases export.
        #[arg(long, conflicts_with = "dir", requires = "workspace")]
        cases: Option<PathBuf>,
        /// The workspace whose tests state the cases.
        #[arg(long, requires = "cases")]
        workspace: Option<PathBuf>,
    },
    /// Refuse a head under which more translator functions are uncertified
    /// than under the base.
    Ratchet {
        /// The head's register.
        #[arg(long)]
        register: PathBuf,
        /// The head's manifest.
        #[arg(long)]
        manifest: PathBuf,
        /// The base's register.
        #[arg(long)]
        base_register: PathBuf,
        /// The base's manifest.
        #[arg(long)]
        base_manifest: PathBuf,
        #[command(flatten)]
        profile: ProfileArgs,
    },
}

/// What an export is of, when it is not the translator's own: each option
/// given replaces what the workspace declares, and with none of them the
/// export is the translator's (uorust::mir, lift and lex, from the translate
/// entry, as `uorust_certified_` packages).
#[derive(clap::Args, Default)]
struct ProfileArgs {
    /// The workspace the export is of: every function of its register is in
    /// scope and its packages are named after its directory, unless the
    /// `[workspace.metadata.uorust.certify]` table of its Cargo manifest
    /// (`scope`, `entry`, `package`, `effects`, `target`) says otherwise.
    #[arg(long)]
    workspace: Option<PathBuf>,
    /// A crate-path prefix of the functions in scope (repeatable).
    #[arg(long)]
    scope: Vec<String>,
    /// The function the share's reachability starts at.
    #[arg(long)]
    entry: Option<String>,
    /// The prefix of every certified package's name.
    #[arg(long)]
    package: Option<String>,
    /// An effect every root admits (repeatable).
    #[arg(long)]
    effect: Vec<String>,
    /// The target every root is exported to.
    #[arg(long)]
    target: Option<String>,
    /// Export the cases: the closed assertions of the workspace's tests
    /// over the functions in scope, each a root with no parameter.
    #[arg(long)]
    cases: bool,
}

impl ProfileArgs {
    fn options(&self) -> uorust::certify::Options {
        uorust::certify::Options {
            workspace: self.workspace.clone(),
            scope: self.scope.clone(),
            entry: self.entry.clone(),
            package: self.package.clone(),
            effects: self.effect.clone(),
            target: self.target.clone(),
            cases: self.cases,
        }
    }

    fn profile(&self) -> Result<uorust::certify::Profile, UorustError> {
        uorust::certify::Profile::resolve(&self.options())
    }
}

#[derive(clap::Args)]
struct MirArgs {
    /// The workspace root, the directory with the top `Cargo.toml`.
    workspace: PathBuf,
    /// The directory the dumps are written to.
    #[arg(long)]
    out: PathBuf,
    /// Build with these features enabled (repeatable, or comma-separated),
    /// as cargo takes them: a body under a feature the default build
    /// leaves off exists only in a dump that enables it.
    #[arg(long)]
    features: Vec<String>,
    /// Build without the default features.
    #[arg(long)]
    no_default_features: bool,
    /// Build with every feature.
    #[arg(long, conflicts_with_all = ["features", "no_default_features"])]
    all_features: bool,
    /// Build for this target triple; the host's when absent.
    #[arg(long)]
    target: Option<String>,
}

#[derive(clap::Args)]
struct CoreArgs {
    /// The module shards: one directory of core's dumps per module.
    #[arg(long)]
    mir: PathBuf,
    /// The rust-src `library` directory the dumps were built from.
    #[arg(long)]
    library: PathBuf,
    /// The register directory: `frontier.toml` and one file per module.
    #[arg(long)]
    register: PathBuf,
    /// Translate only this module (repeatable), linked against every
    /// module the dumps hold: a CI shard's census, whose rows are the
    /// whole census's.
    #[arg(long)]
    only: Vec<String>,
    /// Write each module's shallow translations here, `{module}.toml`,
    /// each translated row's κ with its rendered LexLean: what `uorust
    /// embed --shallow` bridges to the evaluator.
    #[arg(long)]
    lexlean: Option<PathBuf>,
}

/// The steps of the figures (`uorust::figures`).
#[derive(Subcommand)]
enum Figures {
    /// Write the figure register of a workspace: one row per counted
    /// numeric-literal site, and the figure rows the register already holds.
    Extract(FigureArgs),
    /// Write the generated package (and the LexLean module) from the
    /// register, or with `--check` list each file that differs.
    Write {
        #[command(flatten)]
        register: FigureFiles,
        /// The generated package's directory.
        #[arg(long)]
        out: PathBuf,
        /// The LexLean module that states the same figures.
        #[arg(long)]
        lexlean: Option<PathBuf>,
        /// Write nothing; fail on each file that differs.
        #[arg(long)]
        check: bool,
    },
    /// Replace each site that has a figure by its generated name, and list
    /// each site no rewrite can place with why.
    Rewrite(FigureArgs),
    /// The gate: no counted site without a figure, no figure without a
    /// home, the register and the generated files current.
    Check {
        #[command(flatten)]
        args: FigureArgs,
        /// The generated package's directory, checked current.
        #[arg(long)]
        package: Option<PathBuf>,
        /// The LexLean module, checked current.
        #[arg(long, requires = "package")]
        lexlean: Option<PathBuf>,
    },
}

#[derive(clap::Args)]
struct FigureFiles {
    /// The rule file (`uorust/figure-rules/1`).
    #[arg(long)]
    rules: PathBuf,
    /// The figure register (`uorust/figures/1`).
    #[arg(long)]
    register: PathBuf,
}

#[derive(clap::Args)]
struct FigureArgs {
    /// The workspace root.
    workspace: PathBuf,
    #[command(flatten)]
    files: FigureFiles,
}

#[derive(clap::Args)]
struct LibraryMirArgs {
    /// The library: `core`, `alloc` or `std`.
    #[arg(value_name = "CRATE")]
    krate: String,
    /// The directory the dumps are written to.
    #[arg(long, required_unless_present_any = ["key", "modules"])]
    out: Option<PathBuf>,
    /// Dump only this top-level module (repeatable); every module when none.
    #[arg(long)]
    only: Vec<String>,
    /// The target triple the library is built for; the host's when not
    /// given. It enters the key: a dump links only into a program of the
    /// same triple and data layout.
    #[arg(long)]
    target: Option<String>,
    /// Where rust-src's archive of the pinned toolchain is kept (taken only
    /// at the digest its channel manifest records) and unpacked anew for
    /// each run; `<out>/../rust-src-<toolchain>` when not given.
    #[arg(long)]
    source_cache: Option<PathBuf>,
    /// Print the dump's key (for the target and the modules asked), read
    /// from the toolchain alone, and dump nothing.
    #[arg(long, conflicts_with = "modules")]
    key: bool,
    /// Print the library's top-level modules as a JSON array and dump nothing.
    #[arg(long)]
    modules: bool,
}

#[derive(clap::Args)]
struct LayoutsArgs {
    /// The crate the dumps are of: `core`, `std`.
    #[arg(long = "crate")]
    krate: String,
    /// The dumps, one directory per module; `layouts.json` is written here.
    #[arg(long)]
    mir: PathBuf,
    /// A copy of the `library` directory the dumps were built from, which
    /// the aliases are written into.
    #[arg(long)]
    copy: PathBuf,
    /// The shell command that checks the copy, rustc's diagnostics on its
    /// stderr.
    #[arg(long)]
    check: String,
}

#[derive(clap::Args)]
struct EmbedArgs {
    /// The crate the dumps are of: `core`, `std`, or with `--workspace` a
    /// library crate of that workspace.
    #[arg(long = "crate")]
    krate: String,
    /// The workspace the crate is a library of: `--mir` is then the
    /// directory `uorust mir` dumped it into, whose `shards-<crate>` are
    /// embedded, with the crate's declarations and those of the crates it
    /// builds on mounted beside `--library`'s core, alloc and std.
    #[arg(long, conflicts_with_all = ["harness", "observed", "register", "shallow", "bridge"])]
    workspace: Option<PathBuf>,
    /// With `--workspace`: the directory `uorust crate --workspace
    /// --dumps` kept the build's registry crates in; the crate's program
    /// links each of them its build links, after every `--link`.
    #[arg(long, requires = "workspace")]
    links: Option<PathBuf>,
    /// With `--workspace`: one project per module of the crate,
    /// `<out>/<module>/project` (its configuration and record in
    /// `<out>/<module>`, outside the project), smallest module first, each
    /// linked against every module, as one `--only <module>` run each; with
    /// `--only`, one per module or part it names. The libraries are linked
    /// once for all.
    #[arg(long, requires = "workspace")]
    each_module: bool,
    /// A library crate to link into the program, `<crate>=<dumps>`, as
    /// `uorust crate --link` links one: core, alloc, or with `--workspace`
    /// another library of the workspace (`<mir>/shards-<lib>`). `<dumps>`
    /// may be a published archive of them, `<url>#sha256=<hex>`, fetched
    /// and taken only at that digest.
    #[arg(long = "link")]
    link: Vec<String>,
    /// The module shards: one directory of the crate's dumps per module.
    #[arg(long)]
    mir: PathBuf,
    /// The rust-src `library` directory the dumps were built from.
    #[arg(long)]
    library: PathBuf,
    /// The directory the LexLean project is written to.
    #[arg(long)]
    out: PathBuf,
    /// Embed only this module (repeatable), linked against every module
    /// the dumps hold: a CI shard's.
    #[arg(long)]
    only: Vec<String>,
    /// Write the differential check's harness crate here: a call of each
    /// sampled body's function, as rustc builds core, per case.
    #[arg(long)]
    harness: Option<PathBuf>,
    /// The harness's output: add the theorems that the evaluator returns
    /// what rustc's build returned, case by case.
    #[arg(long)]
    observed: Option<PathBuf>,
    /// A register's module files to stamp each row's embedding on, by κ:
    /// the census the merge sums is the evaluator's own.
    #[arg(long)]
    register: Option<PathBuf>,
    /// The shallow translations `uorust core --lexlean` wrote: with
    /// `--bridge`, the bridge's theorems that each refines the evaluator.
    #[arg(long, requires = "bridge")]
    shallow: Option<PathBuf>,
    /// The directory the bridge's LexLean project is written to.
    #[arg(long, requires = "shallow")]
    bridge: Option<PathBuf>,
}

#[derive(clap::Args)]
struct AdmitArgs {
    /// The parent's lift register: the incumbent every row is held against.
    #[arg(long)]
    parent: PathBuf,
    /// The candidate lift register.
    candidate: PathBuf,
}

#[derive(clap::Args)]
struct CarryArgs {
    /// The register's module files the stamps were measured in.
    #[arg(long)]
    from: PathBuf,
    /// The module files of the census taken again, which get them.
    #[arg(long)]
    register: PathBuf,
    /// The commit the stamps were measured at.
    #[arg(long)]
    measured: String,
}

#[derive(clap::Args)]
struct BridgeArgs {
    /// The bridge's project, as `uorust embed --bridge` wrote it.
    #[arg(long)]
    project: PathBuf,
    /// A register's module files to stamp each translated row's bridge on.
    #[arg(long)]
    register: Option<PathBuf>,
    /// When the bridge must be done, in seconds since the Unix epoch: work
    /// that could not finish by then is pending, not started.
    #[arg(long)]
    until: Option<u64>,
}

#[derive(clap::Args)]
struct CrateArgs {
    /// The crate and its version, `<name>@<version>`.
    #[arg(required_unless_present = "workspace", conflicts_with = "workspace")]
    spec: Option<String>,
    /// Every library of crates.io the build of this workspace links, each
    /// at the version and checksum its lock records and with the features
    /// the build enables for it (cargo's unit graph of the build), dumped,
    /// translated and embedded with those it builds on linked (after each
    /// `--link`); with `--dumps` and `--library`. Each register is the
    /// `<name>-<version>` directory of `--registers`.
    #[arg(long, requires_all = ["dumps", "library"])]
    workspace: Option<PathBuf>,
    /// With `--workspace`, only these packages and those they build on
    /// (repeatable).
    #[arg(long, requires = "workspace")]
    only: Vec<String>,
    /// The features to build with: the crate's own, exactly, in place of
    /// its defaults (with `<spec>`); the workspace build's, as cargo takes
    /// them (with `--workspace`).
    #[arg(long, value_delimiter = ',')]
    features: Vec<String>,
    /// Build without the default features (the crate's with `<spec>`, the
    /// workspace build's with `--workspace`).
    #[arg(long)]
    no_default_features: bool,
    /// With `--workspace`: build the workspace with every feature of its
    /// members, as `uorust mir --all-features` dumps it.
    #[arg(long, requires = "workspace", conflicts_with_all = ["features", "no_default_features"])]
    all_features: bool,
    /// Keep the crate's dumps (one directory per module, `target.toml`,
    /// the layouts, `dump.toml`) in this directory, as `--link
    /// <crate>=<dumps>` reads them; with `--workspace`, one directory per
    /// crate and `links.toml`.
    #[arg(long, requires = "library")]
    dumps: Option<PathBuf>,
    /// The directory the crates' registers are in; the register is its
    /// `<name>` directory.
    #[arg(long, default_value = "crates/uorust/crates")]
    registers: PathBuf,
    /// With `<spec>`, the register's directory itself, in place of
    /// `<registers>/<name>`.
    #[arg(long, conflicts_with = "workspace")]
    register: Option<PathBuf>,
    /// With `<spec>`, the checksum a consuming workspace's lock records for
    /// the crate: the archive is cargo's cached one where the cache holds
    /// it, and must have this digest.
    #[arg(long, conflicts_with = "workspace")]
    checksum: Option<String>,
    /// With `<spec>`, a consuming workspace's lock file: the crate's own
    /// dependencies are resolved at the versions it locks.
    #[arg(long, conflicts_with = "workspace")]
    lock: Option<PathBuf>,
    /// The rust-src `library` directory of the pinned toolchain: with it,
    /// the crate's bodies are embedded in LexLean too, and each row of its
    /// register stamped with its embedding.
    #[arg(long)]
    library: Option<PathBuf>,
    /// With `--library`, check the embedding against rustc's build of the
    /// crate: each of its functions over scalars run by both, a theorem per
    /// case that the evaluator returns what the build did, which `lexlean
    /// verify` decides; each sampled row is stamped with its theorems.
    #[arg(long, requires = "library")]
    differential: bool,
    /// A library crate to link into the crate's program, `<crate>=<dumps>`
    /// (`core=target/core-mir`, as `scripts/mir/dump-lib.sh` writes them):
    /// a call into it names its body, so a body that calls core is
    /// evaluated through core's. `<dumps>` may be a published archive of
    /// them, `<url>#sha256=<hex>`, fetched and taken only at that digest.
    #[arg(long = "link", requires = "library")]
    link: Vec<String>,
}

#[derive(clap::Args)]
struct MergeArgs {
    /// The crate the register is of: `core`, `std`, or `<name>@<version>`
    /// for a crate of crates.io.
    #[arg(long = "crate")]
    krate: String,
    /// The module files to merge.
    #[arg(long)]
    shards: PathBuf,
    /// The register directory.
    #[arg(long)]
    register: PathBuf,
}

#[derive(clap::Args)]
struct ProveArgs {
    #[command(flatten)]
    args: Args,
    /// The directory the trial projections are verified in.
    #[arg(long)]
    scratch: PathBuf,
}

#[derive(clap::Args)]
struct Args {
    /// The workspace root, the directory with the top `Cargo.toml`.
    workspace: PathBuf,
    /// The directory the Prism project is written to or checked in.
    #[arg(long)]
    out: PathBuf,
    /// The project name; when absent, the name of the package the
    /// workspace's root manifest declares (a virtual workspace declares
    /// none, and then the name must be given). Never the directory's name.
    #[arg(long)]
    name: Option<String>,
    /// The control set's lastModified, YYYY-MM-DDThh:mm:ssZ; the HEAD commit time when absent.
    #[arg(long)]
    last_modified: Option<String>,
    /// A RustSec advisory database checkout to check Cargo.lock against.
    #[arg(long)]
    advisories: Option<PathBuf>,
    /// The limits the projection's LexLean and PrismPM projects state: a
    /// TOML file with a [lexlean] and a [prismpm] table holding every key
    /// the SDK's project schema requires. Without it the SDK scaffold's
    /// limits are written, and the frontier's [limits] table says which.
    #[arg(long)]
    limits: Option<PathBuf>,
    /// A lift register file: the projection is admitted against it (no
    /// function it records as lifted may be decided otherwise), then
    /// `project` writes the projection's register there and `check` requires
    /// it to be current.
    #[arg(long)]
    register: Option<PathBuf>,
    /// The MIR dumps of the workspace (`uorust mir`): every function the
    /// surface lift leaves as residue is translated from its MIR.
    #[arg(long)]
    mir: Option<PathBuf>,
    /// The verdict register of the MIR path (`uorust prove`); a translation
    /// is admitted only on a proved verdict.
    #[arg(long, requires = "mir")]
    verdicts: Option<PathBuf>,
    /// The records `uorust embed --workspace` wrote (any depth under the
    /// directory), of this workspace's sources as they are now: a function
    /// row that did not lift is joined to its body in every declared
    /// `--configuration`, by the item the dump names, and is embedded-closed
    /// where every one is embedded, evaluable and reaches only linked
    /// bodies. A record of another source tree is refused.
    #[arg(long, requires = "configuration")]
    embedding: Option<PathBuf>,
    /// A configuration the workspace declares, by the label its records
    /// state, target included as `uorust mir` records it (`default features
    /// on x86_64-unknown-linux-gnu`, `all features on <target>`, `default
    /// features and a,b on <target>`; repeatable): a row is embedded-closed
    /// only where the records of every declared configuration cover its
    /// crate, so a label no record states leaves every row as it was.
    #[arg(long, requires = "embedding")]
    configuration: Vec<String>,
}

fn options(args: &Args) -> Result<Options, UorustError> {
    let name = match &args.name {
        Some(name) => name.clone(),
        None => uorust::declared_name(&args.workspace)?,
    };
    let last_modified = match &args.last_modified {
        Some(value) => value.clone(),
        None => uorust::commit_time(&args.workspace)?,
    };
    let mir = match &args.mir {
        Some(dumps) => Some(uorust::mir::MirOptions {
            dumps: dumps.clone(),
            verdicts: match &args.verdicts {
                Some(path) => uorust::mir::verdict::Verdicts::read(path)?,
                None => uorust::mir::verdict::Verdicts::default(),
            },
            trial: uorust::mir::Trial::Off,
            learned: std::collections::BTreeMap::new(),
        }),
        None => None,
    };
    Ok(Options {
        name,
        last_modified,
        advisories: args.advisories.clone(),
        mir,
        limits: args.limits.clone(),
        embedding: args.embedding.clone().map(|records| uorust::Embedding {
            records,
            configurations: args.configuration.clone(),
        }),
    })
}

fn run(cli: Cli) -> Result<(), UorustError> {
    uorust::certified::use_hand(cli.translator == Translator::Hand);
    match cli.command {
        Command::Project(args) => {
            let files = uorust::project_workspace(&args.workspace, &options(&args)?)?;
            if let Some(register) = &args.register {
                uorust::admit(register, &files)?;
            }
            uorust::write(&args.out, &files)?;
            if let Some(register) = &args.register {
                uorust::write_register(register, &files)?;
            }
            println!("wrote {} files to {}", files.len(), args.out.display());
            Ok(())
        }
        Command::Check(args) => {
            let files = uorust::project_workspace(&args.workspace, &options(&args)?)?;
            if let Some(register) = &args.register {
                uorust::check_register(register, &files)?;
            }
            uorust::check(&args.out, &files)?;
            println!("{} is the current projection", args.out.display());
            Ok(())
        }
        Command::Mir(args) => {
            let config = uorust::shard::Configuration {
                features: args
                    .features
                    .iter()
                    .flat_map(|f| f.split(','))
                    .map(str::trim)
                    .filter(|f| !f.is_empty())
                    .map(str::to_string)
                    .collect(),
                no_default_features: args.no_default_features,
                all_features: args.all_features,
                target: args.target.clone(),
            };
            uorust::mir::dump(&args.workspace, &args.out, &config)?;
            println!(
                "dumped the MIR of {} into {}",
                args.workspace.display(),
                args.out.display()
            );
            Ok(())
        }
        Command::Prove(prove) => {
            let args = &prove.args;
            let (Some(dumps), Some(verdicts)) = (&args.mir, &args.verdicts) else {
                return Err(UorustError::Tool(
                    "prove needs --mir and --verdicts".to_string(),
                ));
            };
            let summary = uorust::mir::prove(
                &args.workspace,
                &options(args)?,
                dumps,
                verdicts,
                &prove.scratch,
            )?;
            println!(
                "proved {}, refused {} in {} verifier runs; verdicts in {}",
                summary.proved,
                summary.refused,
                summary.runs,
                verdicts.display()
            );
            Ok(())
        }
        Command::Core(args) => library("core", &args),
        Command::LibraryMir(args) => {
            let here = std::path::Path::new(".");
            if args.key {
                let inputs = uorust::library::inputs(&args.krate, here, args.target.as_deref())?;
                println!("{}", uorust::library::key(&inputs, &args.only));
                return Ok(());
            }
            let toolchain_cache =
                |base: &std::path::Path| base.join(format!("rust-src-{}", uorust::mir::TOOLCHAIN));
            let base = args
                .out
                .as_deref()
                .and_then(std::path::Path::parent)
                .unwrap_or(here)
                .to_path_buf();
            let cache = args
                .source_cache
                .clone()
                .unwrap_or_else(|| toolchain_cache(&base));
            if args.modules {
                let inputs = uorust::library::inputs(&args.krate, here, args.target.as_deref())?;
                let library = uorust::library::source(&inputs, &cache)?;
                let found = uorust::library::modules(&library, &args.krate)?;
                println!("{}", serde_json::to_string(&found).unwrap_or_default());
                return Ok(());
            }
            let out = args.out.as_deref().unwrap_or(here);
            let work = base.join(format!(
                ".library-mir-{}-{}",
                args.krate,
                std::process::id()
            ));
            let done = uorust::library::dump(
                &args.krate,
                args.target.as_deref(),
                out,
                &args.only,
                &cache,
                &work,
            )?;
            for k in &done.kept {
                println!("{}: {} bodies, {} bytes", k.module, k.files, k.bytes);
            }
            println!(
                "library-mir {}: {} modules, {} bodies, {} bytes, {} layouts; key {}; digest {}; library {}",
                args.krate,
                done.kept.len(),
                done.kept.iter().map(|k| k.files).sum::<usize>(),
                done.kept.iter().map(|k| k.bytes).sum::<u64>(),
                done.layouts,
                done.key,
                done.digest,
                done.library.display()
            );
            Ok(())
        }
        Command::Layouts(args) => {
            let n = uorust::layouts::run(&args.krate, &args.mir, &args.copy, &args.check)?;
            eprintln!("layouts: {n} types laid out");
            Ok(())
        }
        Command::Embed(args) if args.workspace.is_some() => {
            let workspace = args
                .workspace
                .as_deref()
                .unwrap_or(std::path::Path::new("."));
            let cache = args
                .out
                .parent()
                .map_or_else(std::env::temp_dir, std::path::Path::to_path_buf)
                .join("uorust-links");
            let mut link = args
                .link
                .iter()
                .map(|l| {
                    let (k, d) = l.split_once('=').ok_or_else(|| {
                        uorust::UorustError::Tool(format!("--link {l}: not <crate>=<dumps>"))
                    })?;
                    Ok((k.to_string(), uorust::registry::link_dumps(d, &cache)?))
                })
                .collect::<Result<Vec<_>, UorustError>>()?;
            if let Some(dir) = &args.links {
                link.extend(uorust::shard::member_links(dir, &args.krate)?);
            }
            let done = uorust::shard::embed_workspace_crate(&uorust::shard::WorkspaceEmbed {
                workspace,
                krate: &args.krate,
                dumps: &args.mir,
                library: &args.library,
                out: &args.out,
                only: &args.only,
                each_module: args.each_module,
                link: &link,
            })?;
            for d in &done {
                let c = &d.census;
                match &d.module {
                    Some(m) => println!(
                        "embed: {} ({} files, {} bytes): {} bodies embedded, {} of them evaluable, {} bound; calls {} linked, {} unlinked; project in {}",
                        m.name,
                        m.files,
                        m.bytes,
                        c.bodies,
                        c.evaluable,
                        c.bound,
                        c.linked.values().sum::<usize>(),
                        c.unlinked.values().sum::<usize>(),
                        args.out.join(&m.name).join(uorust::shard::PROJECT).display()
                    ),
                    None => println!(
                        "embed: {} bodies embedded, {} of them evaluable, {} bound; calls {} linked, {} unlinked; project in {}",
                        c.bodies,
                        c.evaluable,
                        c.bound,
                        c.linked.values().sum::<usize>(),
                        c.unlinked.values().sum::<usize>(),
                        args.out.join(uorust::shard::PROJECT).display()
                    ),
                }
            }
            Ok(())
        }
        Command::Embed(args) => {
            let link = args
                .link
                .iter()
                .map(|l| {
                    l.split_once('=')
                        .map(|(k, d)| (k.to_string(), PathBuf::from(d)))
                        .ok_or_else(|| {
                            uorust::UorustError::Tool(format!("--link {l}: not <crate>=<dumps>"))
                        })
                })
                .collect::<Result<Vec<_>, UorustError>>()?;
            let link: Vec<(&str, &std::path::Path)> = link
                .iter()
                .map(|(k, d)| (k.as_str(), d.as_path()))
                .collect();
            let census = uorust::embed::embed(
                &args.krate,
                &args.library,
                &args.mir,
                &args.out,
                &args.only,
                &uorust::embed::Outputs {
                    harness: args.harness.as_deref(),
                    observed: args.observed.as_deref(),
                    register: args.register.as_deref(),
                    bridge: args.shallow.as_deref().zip(args.bridge.as_deref()),
                    differential: None,
                    link: &link,
                },
            )?;
            println!(
                "embed: {} bodies embedded, {} of them evaluable; project in {}",
                census.bodies,
                census.evaluable,
                args.out.display()
            );
            Ok(())
        }
        Command::Std(args) => library("std", &args),
        Command::Crate(args) => {
            // A published link's dumps are kept by their digest, beside
            // the kept dumps where there are some.
            let cache = args
                .dumps
                .as_deref()
                .and_then(std::path::Path::parent)
                .map_or_else(std::env::temp_dir, std::path::Path::to_path_buf)
                .join("uorust-links");
            let link = args
                .link
                .iter()
                .map(|l| {
                    let (k, d) = l.split_once('=').ok_or_else(|| {
                        uorust::UorustError::Tool(format!("--link {l}: not <crate>=<dumps>"))
                    })?;
                    Ok((k.to_string(), uorust::registry::link_dumps(d, &cache)?))
                })
                .collect::<Result<Vec<_>, UorustError>>()?;
            if let (Some(root), Some(dumps), Some(library)) =
                (&args.workspace, &args.dumps, &args.library)
            {
                let mut cargo_args = Vec::new();
                if !args.features.is_empty() {
                    cargo_args.push(format!("--features={}", args.features.join(",")));
                }
                if args.no_default_features {
                    cargo_args.push("--no-default-features".to_string());
                }
                if args.all_features {
                    cargo_args.push("--all-features".to_string());
                }
                let (graph, outcomes) =
                    uorust::registry::lift_workspace(&uorust::registry::Workspace {
                        root,
                        cargo_args: &cargo_args,
                        dumps,
                        registers: &args.registers,
                        library,
                        only: &args.only,
                        link: &link,
                    })?;
                let lifted = outcomes
                    .iter()
                    .filter(|o| matches!(o, uorust::registry::Outcome::Lifted(_)))
                    .count();
                println!(
                    "{} of {} libraries of crates.io the build links lifted, {} refused; index in {}",
                    lifted,
                    graph.locked.len(),
                    outcomes.len() - lifted,
                    dumps.join(uorust::registry::LINKS).display()
                );
                return Ok(());
            }
            let spec = args.spec.as_deref().unwrap_or_default();
            let (name, version) = uorust::registry::parse_spec(spec)?;
            let register = args
                .register
                .clone()
                .unwrap_or_else(|| args.registers.join(&name));
            let lifted = uorust::registry::lift_crate(
                &name,
                &version,
                &register,
                &uorust::registry::Options {
                    library: args.library.as_deref(),
                    differential: args.differential,
                    link: &link,
                    features: &args.features,
                    no_default_features: args.no_default_features,
                    checksum: args.checksum.as_deref(),
                    lock: args.lock.as_deref(),
                    dumps: args.dumps.as_deref(),
                },
            )?;
            println!(
                "{name}@{version}: {} of {} bodies translated in {} modules; register in {}",
                lifted.translated,
                lifted.bodies,
                lifted.modules,
                register.display()
            );
            if let Some((bodies, evaluable, bound)) = lifted.embedding {
                println!(
                    "{name}@{version}: {bodies} bodies embedded, {evaluable} of them evaluable, {bound} leave a callee unlinked"
                );
            }
            if let Some((bodies, theorems)) = lifted.differential {
                println!(
                    "{name}@{version}: {theorems} differential theorems verified over {bodies} bodies"
                );
            }
            Ok(())
        }
        Command::Merge(args) => {
            let scope = match uorust::registry::parse_spec(&args.krate) {
                Ok((name, version)) => uorust::mir::core::Scope::registry(&name, &version),
                Err(_) => uorust::mir::core::Scope::library(&args.krate, uorust::mir::TOOLCHAIN),
            };
            let merged = uorust::queue::merge(&scope, &args.register, &args.shards)?;
            println!(
                "{}: merged {} modules into {}",
                scope.krate,
                merged.len(),
                args.register.display()
            );
            Ok(())
        }
        Command::Bridge(args) => {
            let b = uorust::bridge::verify(
                &args.project,
                args.register.as_deref(),
                args.until
                    .map(|t| std::time::UNIX_EPOCH + std::time::Duration::from_secs(t)),
            )?;
            println!(
                "bridge: {} of {} translated bodies bridged, {} pending",
                b.bridged,
                b.bridged + b.pending,
                b.pending
            );
            Ok(())
        }
        Command::Carry(args) => {
            let (kept, dropped) =
                uorust::bridge::carry(&args.from, &args.register, &args.measured)?;
            println!(
                "carry: {kept} bridge stamps carried from {}, {dropped} rows to measure again",
                args.measured
            );
            Ok(())
        }
        Command::Certify(step) => certify(step),
        Command::Admit(args) => {
            let totals = uorust::admit_register(&args.parent, &args.candidate)?;
            println!(
                "{} admitted against {}: {} of {} functions lifted",
                args.candidate.display(),
                args.parent.display(),
                totals.lifted,
                totals.functions
            );
            Ok(())
        }
        Command::Figures(step) => figures(step),
        Command::Routes => {
            for route in uorust::ROUTES {
                println!(
                    "{}\t{}\t{}\t{}",
                    route.id, route.rust_fact, route.prism_record, route.facet
                );
            }
            Ok(())
        }
    }
}

/// A step of the figures.
fn figures(step: Figures) -> Result<(), UorustError> {
    use uorust::figures;
    let declared = |l: &figures::Locations| {
        println!(
            "figures: the rules declare columns = \"{}\": {}",
            l.columns, l.reason
        );
    };
    let counts = |c: &figures::Counts| {
        println!(
            "figures: {} counted sites, {} with a figure, {} without; {} rewritten; {} figures, \
             {} without a home, {} whose home names no check",
            c.sites, c.figured, c.free, c.rewritten, c.figures, c.unhomed, c.unchecked
        );
        for (why, n) in &c.refusals {
            println!("figures: {n} sites no rewrite can place: {why}");
        }
        if let Some(l) = &c.columns {
            declared(l);
            println!(
                "figures: {} rewritten sites moved columns under the declaration",
                c.moved
            );
        }
    };
    match step {
        Figures::Extract(a) => {
            let done = figures::extract(&a.workspace, &a.files.rules, &a.files.register)?;
            counts(&done.counts);
            for v in &done.vanished {
                println!("figures: a listed site is gone: {v}");
            }
            Ok(())
        }
        Figures::Write {
            register,
            out,
            lexlean,
            check,
        } => {
            let w = figures::write(
                &register.rules,
                &register.register,
                &out,
                lexlean.as_deref(),
                check,
            )?;
            if check {
                if !w.files.is_empty() {
                    return Err(UorustError::Figures(
                        w.files
                            .iter()
                            .map(|f| format!("{f} is not what the writer writes now"))
                            .collect(),
                    ));
                }
                println!("figures: the generated files are current");
            } else {
                println!("figures: wrote {}", w.files.join(", "));
            }
            Ok(())
        }
        Figures::Rewrite(a) => {
            let r = figures::rewrite(&a.workspace, &a.files.rules, &a.files.register)?;
            println!(
                "figures: {} sites replaced in {} files; {} refused",
                r.replaced,
                r.files.len(),
                r.refused.len()
            );
            for x in &r.refused {
                println!("figures: refused {}: {}", x.key, x.why);
            }
            if let Some(l) = &r.columns {
                declared(l);
            }
            for m in &r.moved {
                println!(
                    "figures: placed {} at {}:{}: the columns from {} on move by {}",
                    m.key, m.file, m.line, m.column, m.by
                );
            }
            Ok(())
        }
        Figures::Check {
            args,
            package,
            lexlean,
        } => {
            let c = figures::check(
                &args.workspace,
                &args.files.rules,
                &args.files.register,
                package.as_deref(),
                lexlean.as_deref(),
            )?;
            counts(&c.counts);
            if c.findings.is_empty() {
                Ok(())
            } else {
                Err(UorustError::Figures(c.findings))
            }
        }
    }
}

/// Translate the module shards of `krate`, a crate of the pinned
/// toolchain's rust-src, and write its register.
fn library(krate: &str, args: &CoreArgs) -> Result<(), UorustError> {
    let scope = uorust::mir::core::Scope::library(krate, uorust::mir::TOOLCHAIN);
    // The dumps name sources `library/<crate>/src/..`.
    let root = args.library.parent().unwrap_or(&args.library);
    let shards = uorust::mir::core::census(&scope, root, &args.mir, &args.only)?;
    let regressed = uorust::mir::core::core_regressions(&args.register, &shards);
    if !regressed.is_empty() {
        return Err(UorustError::Tool(format!(
            "a body the {krate} register records as translated is now residue: {}",
            regressed.join("; ")
        )));
    }
    uorust::mir::core::write_register(&scope, &args.register, &shards)?;
    if let Some(dir) = &args.lexlean {
        uorust::bridge::write_shallow(dir, &shards)?;
    }
    let (all, done) = shards.iter().fold((0, 0), |(a, d), s| {
        (a + s.rows.len(), d + s.translated_count())
    });
    println!(
        "{krate}: {done} of {all} bodies translated in {} modules; register in {}",
        shards.len(),
        args.register.display()
    );
    Ok(())
}

fn read_register(path: &std::path::Path) -> Result<uorust::register::Register, UorustError> {
    let text = std::fs::read_to_string(path).map_err(|e| UorustError::Io {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    uorust::register::Register::read(path, &text)
}

/// Where an export takes the prior export's package sources from: the
/// directories given, else the prior manifest's own.
fn carried_from(prior: &std::path::Path, given: Vec<PathBuf>) -> Vec<PathBuf> {
    if !given.is_empty() {
        return given;
    }
    prior
        .parent()
        .map(std::path::Path::to_path_buf)
        .into_iter()
        .collect()
}

fn certify(step: Certify) -> Result<(), UorustError> {
    use uorust::certify;
    let read = |path: &PathBuf| {
        std::fs::read_to_string(path).map_err(|e| UorustError::Io {
            path: path.clone(),
            reason: e.to_string(),
        })
    };
    match step {
        Certify::Production {
            projection,
            register,
            targets,
            out,
            kept,
            prior,
            carry,
            within,
            shard,
            shards,
            profile,
        } => {
            let manifest = certify::Manifest::read(&prior)?;
            let sources = carried_from(&prior, carry);
            let done = certify::production_build(
                &certify::Input {
                    profile: &profile.profile()?,
                    projection: &projection,
                    register: &read_register(&register)?,
                    targets: &read(&targets)?,
                },
                &out,
                &kept,
                &certify::Prior {
                    manifest: &manifest,
                    sources: &sources,
                    verify: certify::verify_project,
                },
                &certify::Compiler::installed()?,
                certify::Scope {
                    within: within.map(std::time::Duration::from_secs),
                    shard: shard.zip(shards),
                },
            )?;
            println!(
                "carried {}, built {} in {} builds, refused {}; certified bundle kept in {}",
                done.carried,
                done.built,
                done.builds,
                done.refused,
                kept.display()
            );
        }
        Certify::Sync {
            kept,
            register,
            mir,
            workspace,
            dir,
            adapters,
            projection,
        } => {
            let (profile, m) = certify::sync_paths(&certify::Sync {
                kept: &kept,
                dir: &dir,
                register: &read_register(&register)?,
                projection: projection.as_deref(),
                adapters: adapters.as_deref(),
                mir: workspace.as_deref().zip(mir.as_deref()),
            })?;
            let s = &m.share;
            match (profile.kind, &profile.entry) {
                (certify::Kind::Cases, _) => println!(
                    "{} certified roots; {} of {} cases certified, over {} functions",
                    m.roots.len(),
                    s.certified,
                    s.translator,
                    certify::cases::covered(&m).len()
                ),
                (certify::Kind::Functions, Some(entry)) => println!(
                    "{} certified roots; {} of {} translator functions certified, {} of {} reachable from {entry}",
                    m.roots.len(),
                    s.certified,
                    s.translator,
                    s.reachable_certified,
                    s.reachable
                ),
                (certify::Kind::Functions, None) => println!(
                    "{} certified roots; {} of {} functions in scope certified",
                    m.roots.len(),
                    s.certified,
                    s.translator
                ),
            }
        }
        Certify::Plan {
            register,
            prior,
            budget,
            projection,
            targets,
            carry,
            profile,
        } => {
            let profile = profile.profile()?;
            let register = read_register(&register)?;
            let roots = match (profile.kind, &projection) {
                (certify::Kind::Cases, None) => {
                    return Err(UorustError::Tool(
                        "the cases are read from the projection: pass --projection".to_string(),
                    ))
                }
                (certify::Kind::Cases, Some(projection)) => {
                    profile.export_roots(projection, &register)?
                }
                (certify::Kind::Functions, _) => profile.roots(&register),
            };
            let manifest = certify::Manifest::read(&prior)?;
            let carried = match (&projection, &targets) {
                (Some(projection), Some(targets)) => {
                    let input = certify::Input {
                        profile: &profile,
                        projection,
                        register: &register,
                        targets: &read(targets)?,
                    };
                    let sources = carried_from(&prior, carry);
                    let prior = certify::Prior {
                        manifest: &manifest,
                        sources: &sources,
                        verify: certify::verify_project,
                    };
                    let decided =
                        certify::carry(&input, &roots, &certify::Compiler::installed()?, &prior);
                    Some(decided?)
                }
                _ => None,
            };
            let plan = certify::plan_keyed(&roots, carried.as_ref(), &manifest, budget);
            let shards: Vec<String> = (0..plan.shards)
                .map(|i| format!("{i}-{}", plan.shards))
                .collect();
            println!(
                "{}",
                serde_json::json!({
                    "shards": shards,
                    "diagnose": plan.diagnose,
                    "carried": plan.carried,
                    "build": plan.build,
                    "refused": plan.refused,
                })
            );
        }
        Certify::Diagnose {
            projection,
            register,
            targets,
            root,
            out,
            profile,
        } => {
            certify::diagnose(
                &certify::Input {
                    profile: &profile.profile()?,
                    projection: &projection,
                    register: &read_register(&register)?,
                    targets: &read(&targets)?,
                },
                &out,
                &root,
            )?;
            println!("{}", out.join("build.log").display());
        }
        Certify::Roots {
            projection,
            register,
            targets,
            out,
            profile,
        } => {
            let admission = certify::admit_roots(
                &certify::Input {
                    profile: &profile.profile()?,
                    projection: &projection,
                    register: &read_register(&register)?,
                    targets: &read(&targets)?,
                },
                &out,
            )?;
            println!(
                "{} roots: {} admitted, {} refused",
                admission.roots,
                admission.admitted.len(),
                admission.refused.len()
            );
            for (cause, count) in admission.causes() {
                println!("{count}\t{cause}");
            }
        }
        Certify::Agree {
            dir,
            cases,
            workspace,
        } => match (dir, cases, workspace) {
            (_, Some(cases), Some(workspace)) => {
                let lines = certify::cases::agree(&cases, &workspace)?;
                for line in &lines {
                    println!("{line}");
                }
                println!("{} cases agree", lines.len());
            }
            (Some(dir), ..) => {
                for line in certify::agree(&dir)? {
                    println!("{line}");
                }
            }
            _ => {
                return Err(UorustError::Tool(
                    "agree needs --dir, or --cases and --workspace".to_string(),
                ))
            }
        },
        Certify::Ratchet {
            register,
            manifest,
            base_register,
            base_manifest,
            profile,
        } => {
            // The scope the head is measured under: the options', else the
            // one its manifest states, else the translator's own.
            let stated = profile.options() != certify::Options::default();
            let head_profile = match certify::Profile::recorded_at(&manifest)? {
                Some(recorded) if !stated => recorded,
                _ => profile.profile()?,
            };
            let n = head_profile.ratchet_from(
                certify::Profile::recorded_at(&base_manifest)?.as_ref(),
                &read_register(&register)?,
                &certify::Manifest::read(&manifest)?,
                &read_register(&base_register)?,
                &certify::Manifest::read(&base_manifest)?,
            )?;
            println!(
                "{n} {} uncertified, none more than in the base",
                head_profile.measured()
            );
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("uorust: {error}");
            ExitCode::FAILURE
        }
    }
}
