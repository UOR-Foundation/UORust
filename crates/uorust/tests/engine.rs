//! The engine against its fixture, against this workspace, and against planted
//! defects (`UE-01` .. `UE-17`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use uorust::{Options, UorustError};

const STAMP: &str = "2026-11-11T00:00:00Z";

type Projection = BTreeMap<String, Vec<u8>>;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/uorust is two below the root")
        .to_path_buf()
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/thermostat")
}

fn golden() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/thermostat-prism")
}

fn options(name: &str) -> Options {
    Options {
        name: name.to_string(),
        last_modified: STAMP.to_string(),
        advisories: None,
        mir: None,
        limits: None,
        embedding: None,
    }
}

/// A projection; this workspace's runs the MIR path on its committed verdict
/// register, as `just register` and `just uorust` do.
fn project(root: &Path, name: &str) -> BTreeMap<String, Vec<u8>> {
    // The acceptance run reads an immutable checkout. Share that expensive
    // self projection across independent assertions; determinism stays fresh.
    static SELF: std::sync::Mutex<BTreeMap<String, Projection>> =
        std::sync::Mutex::new(BTreeMap::new());
    if root != self::root() {
        return project_fresh(root, name);
    }
    let mut cache = SELF
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    cache
        .entry(name.to_string())
        .or_insert_with(|| project_fresh(root, name))
        .clone()
}

fn project_fresh(root: &Path, name: &str) -> BTreeMap<String, Vec<u8>> {
    let mut options = options(name);
    if root == self::root() {
        options.mir = Some(mir_options(
            dumped(root, name),
            verdicts(&root.join("lift-verdicts.toml")),
            uorust::mir::Trial::Off,
        ));
    }
    uorust::project_workspace(root, &options).expect("projects")
}

/// The MIR of the workspace at `root`, dumped once per test run.
fn dumped(root: &Path, name: &str) -> PathBuf {
    static DUMPS: std::sync::Mutex<BTreeMap<String, PathBuf>> =
        std::sync::Mutex::new(BTreeMap::new());
    let mut dumps = DUMPS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    dumps
        .entry(name.to_string())
        .or_insert_with(|| {
            let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("mir-{name}"));
            uorust::mir::dump(root, &out, &Default::default()).expect("the MIR dumps");
            out
        })
        .clone()
}

fn verdicts(path: &Path) -> uorust::mir::verdict::Verdicts {
    uorust::mir::verdict::Verdicts::read(path).expect("the verdict register parses")
}

fn mir_options(
    dumps: PathBuf,
    verdicts: uorust::mir::verdict::Verdicts,
    trial: uorust::mir::Trial,
) -> uorust::mir::MirOptions {
    uorust::mir::MirOptions {
        dumps,
        verdicts,
        trial,
        learned: BTreeMap::new(),
    }
}

/// The frontier without its lifted rows, which come last: what did not lift.
fn unlifted(files: &BTreeMap<String, Vec<u8>>) -> String {
    let frontier = text(files, "frontier.toml");
    frontier
        .split("\n[[lifted]]")
        .next()
        .unwrap_or_default()
        .to_string()
}

/// The rows of the array of tables `table` in a projection's frontier.
fn frontier_rows(files: &BTreeMap<String, Vec<u8>>, table: &str) -> Vec<toml::Value> {
    let frontier: toml::Value =
        toml::from_str(&text(files, "frontier.toml")).expect("frontier parses");
    frontier
        .get(table)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

/// The `[[refusal]]` rows of `reason` in a projection's frontier, and the
/// count the work queue's `[[reason]]` row of that reason carries: read as
/// rows, so a reason text in the queue never counts as a refusal and a
/// refusal row is never stood in for by the queue's summary.
fn refused(files: &BTreeMap<String, Vec<u8>>, reason: &str) -> (usize, Option<i64>) {
    let of = |row: &toml::Value| row.get("reason").and_then(|v| v.as_str()) == Some(reason);
    let rows = frontier_rows(files, "refusal")
        .iter()
        .filter(|r| of(r))
        .count();
    let queued: Vec<i64> = frontier_rows(files, "reason")
        .iter()
        .filter(|r| of(r))
        .map(|r| {
            r.get("refusals")
                .and_then(|v| v.as_integer())
                .expect("a queue row counts its refusals")
        })
        .collect();
    assert!(queued.len() <= 1, "{reason} is queued twice: {queued:?}");
    (rows, queued.first().copied())
}

/// The reasons of the `[[residue]]` rows of `function`.
fn residue_reasons(files: &BTreeMap<String, Vec<u8>>, function: &str) -> Vec<String> {
    frontier_rows(files, "residue")
        .iter()
        .filter(|r| r.get("function").and_then(|v| v.as_str()) == Some(function))
        .filter_map(|r| r.get("reason").and_then(|v| v.as_str()))
        .map(str::to_string)
        .collect()
}

fn text(files: &BTreeMap<String, Vec<u8>>, path: &str) -> String {
    String::from_utf8(files.get(path).cloned().unwrap_or_default()).expect("utf-8")
}

/// A scratch workspace, written fresh for one test.
fn scratch(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uorust-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, body) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        std::fs::write(path, body).expect("write");
    }
    dir
}

#[test]
fn the_fixture_projects_to_the_committed_project_ue_01() {
    let files = project(&fixture(), "thermostat");
    if let Err(error) = uorust::check(&golden(), &files) {
        panic!(
            "{error}\nregenerate with: cargo run -p uorust -- project \
             crates/uorust/tests/fixtures/thermostat --out \
             crates/uorust/tests/fixtures/thermostat-prism --name thermostat \
             --last-modified {STAMP}"
        );
    }
    // And nothing stale lingers beside it: every committed file is projected.
    let manifest = std::fs::read_to_string(golden().join(uorust::MANIFEST)).expect("manifest");
    assert_eq!(manifest.lines().count() + 1, files.len());
}

#[test]
fn projections_are_byte_identical_ue_02() {
    for (root, name) in [(fixture(), "thermostat"), (root(), "uorust")] {
        let first = project_fresh(&root, name);
        let second = project_fresh(&root, name);
        assert_eq!(first, second, "{name} projects differently twice");
        assert!(first.contains_key("src/Architecture.lex.tex"));
    }
}

#[test]
fn ambiguity_is_refused_with_a_typed_open_claim_ue_03() {
    let dir = scratch(
        "ue03",
        &[
            (
                "Cargo.toml",
                "[package]\nname = \"amb\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
            ),
            (
                "src/lib.rs",
                "//! Planted defects.\npub mod a;\npub mod b;\npub use a::*;\n\n/// Calls go.\npub fn run() -> u8 {\n    go()\n}\n\n/// Picks an implementation by its argument's type.\npub fn pick(make: impl FnOnce() -> u8) -> a::Wide {\n    a::Wide::from(make())\n}\n",
            ),
            (
                "src/a.rs",
                "/// One.\npub fn go() -> u8 {\n    1\n}\n\n/// A byte.\npub struct Pair(pub u8);\n\n/// A wide value.\npub struct Wide(pub u32);\n\nimpl From<u8> for Wide {\n    fn from(x: u8) -> Wide {\n        Wide(u32::from(x))\n    }\n}\n\nimpl From<u16> for Wide {\n    fn from(x: u16) -> Wide {\n        Wide(u32::from(x) + 1)\n    }\n}\n\n/// Both a state and an error.\n#[derive(Debug)]\npub enum LoadState {\n    /// Idle.\n    Idle,\n    /// Done.\n    Done,\n}\n\nimpl std::fmt::Display for LoadState {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        write!(f, \"{self:?}\")\n    }\n}\n\nimpl std::error::Error for LoadState {}\n",
            ),
            ("src/b.rs", "/// Two.\npub fn go() -> u8 {\n    2\n}\n"),
        ],
    );
    let files = project(&dir, "amb");
    let frontier = text(&files, "frontier.toml");
    // Each reason from a refusal row of its own, an open claim; the queue's
    // [[reason]] counts alone do not show one.
    let refusals: Vec<&str> = frontier
        .split("\n[[")
        .filter_map(|row| row.strip_prefix("refusal]]\n"))
        .collect();
    for reason in [
        "ambiguous-call",
        "ambiguous-fact",
        "feature-without-acceptance",
        "no-msrv",
    ] {
        assert!(
            refusals.iter().any(
                |row| row.lines().any(|l| l == format!("reason = \"{reason}\""))
                    && row.lines().any(|l| l == "status = \"open\"")
            ),
            "{reason} missing:\n{frontier}"
        );
    }
    // The resolver now reads tuple fields. A callback's result keeps this
    // valid Rust call outside its type-reading floor, with both `From`
    // implementations still possible to the resolver.
    let ambiguous = refusals
        .iter()
        .find(|row| row.contains("reason = \"ambiguous-call\""))
        .expect("an ambiguous-call row");
    assert!(
        ambiguous.contains("span = \"src/lib.rs:12\"")
            && ambiguous.contains("Wide::from")
            && ambiguous.contains("src/a.rs:13")
            && ambiguous.contains("src/a.rs:19"),
        "{ambiguous}"
    );
    let interface = text(&files, "src/Interface.lex.tex");
    assert!(
        !interface.contains("Foundation.Arch::Feature {"),
        "an unaccepted feature was emitted:\n{interface}"
    );
    let behavior = text(&files, "src/Behavior.lex.tex");
    assert!(!behavior.contains("LoadStateFailure"), "{behavior}");
    assert!(!text(&files, "src/Design.lex.tex").contains("LoadState"));
    let quality = text(&files, "src/Quality.lex.tex");
    assert!(!quality.contains("MinimumRustMinor"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_call_admission_resolves_lifts_as_the_function_it_resolves_to() {
    // `go` of the root module is `a::go`, through the glob import: one
    // function by the language's name resolution, which admission and the
    // lift both read it by.
    let dir = scratch(
        "glob-call",
        &[
            (
                "Cargo.toml",
                "[package]\nname = \"amb\"\nversion = \"0.1.0\"\nedition = \"2021\"\nrust-version = \"1.70\"\n\n[workspace]\n",
            ),
            (
                "src/lib.rs",
                "//! A glob import.\npub mod a;\npub mod b;\npub use a::*;\n\n/// Calls go.\npub fn run() -> u8 {\n    go()\n}\n",
            ),
            ("src/a.rs", "/// One.\npub fn go() -> u8 {\n    1\n}\n"),
            ("src/b.rs", "/// Two.\npub fn go() -> u8 {\n    2\n}\n"),
        ],
    );
    let files = project(&dir, "amb");
    let _ = std::fs::remove_dir_all(&dir);
    let frontier = text(&files, "frontier.toml");
    assert!(!frontier.contains("ambiguous-call"), "{frontier}");
    assert!(
        frontier.contains("[lift]\nfunctions = 3\nlifted = 3\nresidue = 0\n"),
        "{frontier}"
    );
    let behavior = text(&files, "src/Behavior.lex.tex");
    let run = behavior
        .split("definition{")
        .find(|d| d.contains("name: run,"))
        .unwrap_or_else(|| panic!("run is not lifted:\n{behavior}"));
    assert!(run.contains("body: (aGo)}"), "run calls another go:\n{run}");
}

#[test]
fn the_floor_lifts_and_the_rest_is_typed_residue_ue_04() {
    let files = project(&fixture(), "thermostat");
    let behavior = text(&files, "src/Behavior.lex.tex");
    for name in ["shouldHeat", "withinBand", "nextMode", "halt"] {
        assert!(
            behavior.contains(&format!("name: {name},")),
            "{name}:\n{behavior}"
        );
    }
    assert!(behavior.contains("statement: ((shouldHeat 18 21) = true),"));
    assert!(behavior.contains("statement: ((withinBand 10 5 9) = false),"));
    assert!(behavior.contains("proof: decide{}"));
    assert!(behavior.contains("(prim quotient (low + high) 2 0 : Nat)"));
    let frontier = text(&files, "frontier.toml");
    // Arithmetic whose meaning is a build setting stays typed residue.
    let planted = scratch(
        "arith",
        &[
            (
                "Cargo.toml",
                "[package]\nname = \"arith\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            (
                "src/lib.rs",
                "/// Overflows.\npub fn mid(a: u32, b: u32) -> u32 {\n    (a + b) / 2\n}\n",
            ),
        ],
    );
    let planted_files = project(&planted, "arith");
    assert_eq!(
        residue_reasons(&planted_files, "arith::mid"),
        ["arithmetic"],
        "{}",
        text(&planted_files, "frontier.toml")
    );
    let _ = std::fs::remove_dir_all(planted);
    // The I/O shell is an effect row, not residue: `load_target` reads a file
    // and `target_from_env` reads the environment, directly or through it.
    let effect = |function: &str, needle: &str| {
        frontier
            .split("[[effect]]")
            .skip(1)
            .any(|row| row.contains(&format!("function = \"{function}\"")) && row.contains(needle))
    };
    assert!(
        effect(
            "thermo_core::config::load_target",
            "Fs effect: fs::read_to_string"
        ),
        "{frontier}"
    );
    assert!(
        residue_reasons(&files, "thermo_core::config::load_target").is_empty(),
        "{frontier}"
    );
}

#[test]
fn every_failure_is_a_typed_variant_ue_06() {
    let missing = std::env::temp_dir().join("uorust-ue06-does-not-exist");
    assert!(matches!(
        uorust::project_workspace(&missing, &options("x")),
        Err(UorustError::Io { .. })
    ));

    let empty = scratch("ue06-empty", &[("README", "no manifest\n")]);
    assert!(matches!(
        uorust::project_workspace(&empty, &options("x")),
        Err(UorustError::CargoMetadata(_))
    ));
    assert!(matches!(
        uorust::commit_time(&empty),
        Err(UorustError::NoTimestamp)
    ));

    let bad_lock = scratch(
        "ue06-lock",
        &[
            (
                "Cargo.toml",
                "[package]\nname = \"l\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
            ),
            ("src/lib.rs", "//! L.\n"),
            ("Cargo.lock", "this is = = not toml\n"),
        ],
    );
    assert!(matches!(
        uorust::project_workspace(&bad_lock, &options("x")),
        Err(UorustError::Malformed { .. })
    ));

    let bad_stamp = Options {
        name: "x".into(),
        last_modified: "yesterday".into(),
        advisories: None,
        mir: None,
        limits: None,
        embedding: None,
    };
    assert!(matches!(
        uorust::project_workspace(&fixture(), &bad_stamp),
        Err(UorustError::BadTimestamp(_))
    ));

    let foreign = scratch("ue06-foreign", &[("notes.txt", "mine\n")]);
    let files = project(&fixture(), "thermostat");
    assert!(matches!(
        uorust::write(&foreign, &files),
        Err(UorustError::ForeignOutput(_))
    ));

    let out = std::env::temp_dir().join(format!("uorust-ue06-out-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    uorust::write(&out, &files).expect("writes into an absent directory");
    uorust::write(&out, &files).expect("rewrites its own projection");
    uorust::check(&out, &files).expect("a fresh write is current");
    std::fs::write(out.join("src/Behavior.lex.tex"), "planted\n").expect("plant");
    match uorust::check(&out, &files) {
        Err(UorustError::Stale(paths)) => assert_eq!(paths, vec!["src/Behavior.lex.tex"]),
        other => panic!("expected Stale, got {other:?}"),
    }
    for dir in [empty, bad_lock, foreign, out] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[test]
fn vv_runs_the_sdk_checks_on_both_projections_ue_07() {
    let justfile = std::fs::read_to_string(root().join("Justfile")).expect("Justfile");
    let vv = justfile
        .lines()
        .find(|l| l.starts_with("vv:"))
        .expect("a vv recipe");
    assert!(vv.split_whitespace().any(|w| w == "uorust"), "{vv}");
    let recipe: String = justfile
        .split("\nuorust:\n")
        .nth(1)
        .expect("a uorust recipe")
        .lines()
        .take_while(|l| l.starts_with("    ") || l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    for needle in [
        "check crates/uorust/tests/fixtures/thermostat",
        "cd crates/uorust/tests/fixtures/thermostat-prism && lexlean fmt --check && lexlean lock --check && lexlean verify && prismpm check",
        "thermostat-prism/library/*/; do (cd \"$lib\" && lexlean fmt --check && lexlean lock --check && prismpm check && prismpm verify) || exit 1; done",
        "project . --out target/uorust-self",
        "cd target/uorust-self && lexlean fmt --check && lexlean lock --check && lexlean verify && prismpm check",
        "--advisories crates/uorust/tests/fixtures/advised/advisory-db",
        "cd target/uorust-advised && lexlean fmt --check && lexlean lock --check && lexlean verify && prismpm check",
    ] {
        assert!(recipe.contains(needle), "the uorust recipe lacks `{needle}`:\n{recipe}");
    }
}

#[test]
fn pub_trait_methods_are_interface_calls_ue_08() {
    let files = project(&fixture(), "thermostat");
    let interface = text(&files, "src/Interface.lex.tex");
    for call in ["Read", "Calibrated"] {
        assert!(
            interface.contains(&format!("name: callThermoCoreSensorSensor{call},")),
            "{interface}"
        );
    }
    let feature = interface
        .split("definition{")
        .find(|d| d.contains("name: featureThermoCoreSensor,"))
        .expect("the sensor module is a feature");
    assert!(feature.contains("\"Interface::callThermoCoreSensorSensorRead\""));
    assert!(feature.contains("\"Interface::callThermoCoreSensorSensorCalibrated\""));
    assert!(
        feature
            .contains("\"Interface::acThermoCoreThermoCoreSensorTestsAFixedSensorReadsItsValue\""),
        "{feature}"
    );
}

#[test]
fn each_module_is_an_opm_view_ue_09() {
    let files = project(&fixture(), "thermostat");
    let design = text(&files, "src/Design.lex.tex");
    for view in [
        "opmModuleLibThermoCore",
        "opmModuleLibThermoCoreConfig",
        "opmModuleLibThermoCoreSensor",
        "opmModuleBinThermo",
    ] {
        assert!(design.contains(&format!("name: {view},")), "{view}");
    }
    let zoom = design
        .split("definition{")
        .find(|d| d.contains("name: opmModuleLibThermoCoreConfigZoom"))
        .expect("load_target zooms into parse_target");
    assert!(zoom.contains("process := \"Design::processThermoCoreConfigLoadTarget\""));
    assert!(zoom.contains("subprocess := \"Design::processThermoCoreConfigParseTarget\""));
    assert!(design.contains("name: opmModuleLibThermoCoreConfigInvokesThermoCoreConfigLoadTargetThermoCoreConfigParseTarget,"));
}

#[test]
fn engine_modules_have_their_own_titles_ue_10() {
    let files = project(&fixture(), "thermostat");
    for (module, title) in [
        ("src/Behavior.lex.tex", "Lifted Rust behavior"),
        ("src/SupplyChain.lex.tex", "Locked supply chain"),
    ] {
        let source = text(&files, module);
        assert!(source.contains("\\useglossary{uorust.titles@1.0.0}"));
        assert!(source.contains(&format!("\\title{{{title}}}")), "{source}");
    }
    for path in ["lexlean.toml", "lexlean.lock"] {
        assert!(text(&files, path).contains("language/uorust.titles"));
    }
    let surfaces: String = ["behavior", "supply-chain"]
        .iter()
        .map(|e| text(&files, &format!("language/uorust.titles/entries/{e}.toml")))
        .collect();
    assert!(surfaces.contains("surface = \"Lifted Rust behavior\""));
    assert!(surfaces.contains("surface = \"Locked supply chain\""));
}

#[test]
fn the_widened_lift_covers_the_fixture_ue_11() {
    let files = project(&fixture(), "thermostat");
    let behavior = text(&files, "src/Behavior.lex.tex");
    for (name, needle) in [
        (
            "width",
            "(prim subtract (project band high) (project band low) : Nat)",
        ),
        ("checkedMidpoint", "<? 4294967296)"),
        ("tick", "(prim remainder (counter + 1) 256 0 : Nat)"),
        (
            "shortfall",
            "(prim subtract (project band low) current : Nat)",
        ),
        ("freezing", "(prim less_than (0 : Int64) outdoor : Bool)"),
        (
            "delta",
            "(prim checked_subtract after before : (Option Int64))",
        ),
        ("checkedValue", "(record TupleBandBool {item0 := band"),
        ("isCelsius", "(prim equal unit \"C\" : Bool)"),
        ("holdsOff", "mutual: holdsOffMutual"),
        (
            "steady",
            "(if coldArg then (if occupied then false else true)",
        ),
        ("hasLine", "(let lines : (List String) := (nil String);"),
        ("phasesOff", "measure: (size_of_list_Phase phases)"),
        (
            "phaseOff",
            "(match phase | Phase.mk label step => (holdsOff step))",
        ),
        (
            "clamped",
            "(let state1 : TupleBoolNat := (if (value <? (project band low))",
        ),
        ("above", "recursive_argument: readings"),
        ("firstReading", "(prim index readings 0 : (Option Nat))"),
        ("targetOr", "| Option.some"),
        ("heatFor", "(ctor Result.error [Bool, ConfigError]"),
        ("fanLabel", "(if (speed =? 0) then \"off\""),
        (
            "labelled",
            "(prim join (prim append (cons value (nil String)) (cons \" \" (cons unit (nil String))) : (List String)) \"\" : String)",
        ),
    ] {
        let decl = behavior
            .split("definition{")
            .find(|d| d.contains(&format!("name: {name},")))
            .unwrap_or_else(|| panic!("{name} is lifted:\n{behavior}"));
        assert!(decl.contains(needle), "{name} lacks {needle}:\n{decl}");
    }
    assert!(behavior.contains("name: Band,"));
    assert!(behavior.contains("statement: ((freezing (-3 : Int64)) = true),"));
    assert!(behavior.contains("axioms: [propext]"));
    let frontier = unlifted(&files);
    assert!(!frontier.contains("function = \"thermo_core::midpoint\""));
    assert!(
        !frontier.contains("function = \"thermo_core::band::"),
        "{frontier}"
    );
}

#[test]
fn iterator_chains_methods_and_maps_lift_ue_15() {
    let files = project(&fixture(), "thermostat");
    let behavior = text(&files, "src/Behavior.lex.tex");
    let decl = |name: &str| -> String {
        behavior
            .split("definition{")
            .find(|d| d.contains(&format!("name: {name},")))
            .unwrap_or_else(|| panic!("{name} is lifted:\n{behavior}"))
            .to_string()
    };
    for (name, needle) in [
        ("roomIsCold", "parameters: [selfValue : Room, floor : Nat]"),
        ("roomReread", "result: Room,"),
        ("roomLabel", "then \"unnamed\" else"),
        ("namesMap1", "recursive_argument: items"),
        ("names", "(namesMap1 rooms)"),
        ("coldFilter1", "(roomIsCold room floor)"),
        ("anyColdAny1", "then true else"),
        ("allWithinAll1", "else false"),
        ("firstColdFind1", "(ctor Option.some [Room] room)"),
        ("highestFold1", "| List.nil => best"),
        (
            "coldCount",
            "(prim length (coldCountFilter1 rooms floor) : Nat)",
        ),
        ("scheduledContains1", "(prim equal item"),
        ("roll", "(prim join (names rooms) \", \" : String)"),
        ("pairs", "axioms: [Classical.choice, Quot.sound, propext]"),
        ("targetOf", "| Option.none => 18"),
        ("roomsOf", "(List TupleStringNat)"),
        ("hasTarget", "(hasTargetHas1 targets room)"),
        ("closed", "(closedContains1 rooms room)"),
        ("labelWidth", "(prim length label : Nat)"),
        ("clamp", "(let low : Nat := (project pair"),
        ("blank", "body: \"\""),
        ("nextMode", "coldArg : Bool"),
        ("coldNamesLoop1", "recursive_argument: items"),
        ("coldNames", "(coldNamesLoop1 rooms"),
        ("reportLoop1", "(record TupleStringNat {item0 := "),
        ("report", "(reportLoop1 rooms (record TupleStringNat"),
        ("readingsOfLoop1", "(prim append state"),
        // A mutual family: a struct joins its enum as an inductive of one
        // constructor, read through accessors, and recursion over it is
        // well-founded on its size.
        ("phaseLabel", "(field_Phase_label phase)"),
        ("hold", "(ctor Phase.mk label (ctor Step.Hold target))"),
        ("size_of_Step", "mutual: SizeOfPhase"),
        ("occupiedTarget", "measure: (size_of_Step step)"),
        ("isFinal", "| Menu.Choose"),
        // A lifetime parameter is erased.
        ("labelOr", "parameters: [phase : Phase, fallback : String]"),
        // An ordered set or map is built by a sorted insert.
        ("labels", "(labelsLoop1 phases out)"),
        (
            "labelsInsert1",
            "then (cons head1 rest1) else (if (prim less_than item1 head1 : Bool)",
        ),
        ("heldTargetsInsert1", "then (cons item1 rest1) else"),
        // format! holes of naturals, booleans and constants.
        ("describe", "(Prelude::natDecimal target)"),
        ("describe", "(if sequence then \"true\" else \"false\")"),
        ("describe", "(cons (heldUnit) "),
        // A while loop is a walk measured by its bound less its cursor,
        // with the theorem that each step decreases it.
        (
            "everyOtherLoop1",
            "measure: (prim subtract (prim length rooms : Nat) at1 : Nat)",
        ),
        (
            "everyOtherLoop1",
            "(everyOtherLoop1 (at1 + 2) (everyOtherStep1 at1 state1 rooms) rooms)",
        ),
        ("firstRoomsLoop1", "evidence: [decreases_firstRoomsLoop1_0]"),
        // A family held in an option of a struct of a list is measured through it.
        ("size_of_option_Floor", "(1 + (size_of_Floor x0))"),
        ("annexedRoom", "measure: (size_of_Plan plan)"),
        // A function parameter is applied; a closure passed for one is its
        // lambda, with the captures it reads.
        ("renamedMap1", "(apply name room)"),
        ("prefixed", "(fun [prefix] (room : String) =>"),
        // `m.entry(k).or_default().push(v)` reads the entry, updates it
        // and inserts it back.
        (
            "byReadingPart1",
            "(match (byReadingGet1 state1 (project room reading))",
        ),
        // A struct written through the crate's modules is the crate's.
        (
            "openingBand",
            "(record Band {low := (project room reading), high := target})",
        ),
        // `()` is LexLean's Unit.
        ("validate", "(ctor Result.ok [Unit, String] ())"),
    ] {
        let d = decl(name);
        assert!(d.contains(needle), "{name} lacks {needle}:\n{d}");
    }
    let frontier = unlifted(&files);
    assert!(!frontier.contains("thermo_core::roster::"), "{frontier}");
    assert!(!frontier.contains("thermo_core::schedule::"), "{frontier}");
    // A pair that names a data enum is a product, which the enum may hold.
    assert!(behavior.contains("{name: Submenu, fields: [(List (String * Menu))]}"));
    assert!(behavior.contains("statement: ((step = (ctor Step.Either occupied rest)) -> ((size_of_Step occupied) < (size_of_Step step)))"));
}

#[test]
fn advisories_are_risks_on_the_supply_chain_ue_12() {
    let advised = root().join("crates/uorust/tests/fixtures/advised");
    let db = advised.join("advisory-db");
    let with_db = Options {
        advisories: Some(db.clone()),
        ..options("advised")
    };
    let files = uorust::project_workspace(&advised, &with_db).expect("projects");
    let security = text(&files, "src/Security.lex.tex");
    for id in ["20180018", "20190009", "20190012", "20210003"] {
        assert!(
            security.contains(&format!("name: riskAdvisoryRustsec{id}V0p6p9,")),
            "{id}:\n{security}"
        );
    }
    // 0.6.9 is patched against RUSTSEC-2018-0003.
    assert!(!security.contains("Rustsec20180003"));
    // deny.toml ignores RUSTSEC-2018-0018 with a reason: an accepted risk, no control.
    assert!(security.contains("name: acceptedThreatAdvisoryRustsec20180018V0p6p9,"));
    assert!(!security.contains("name: controlAdvisoryRustsec20180018V0p6p9,"));
    // The others are answered by the advisory gate, which fails today.
    let measurement = security
        .split("definition{")
        .find(|d| d.contains("name: measurementAdvisoryRustsec20190012V0p6p9,"))
        .expect("a measurement");
    assert!(measurement.contains("passed := false"), "{measurement}");
    let frontier = text(&files, "frontier.toml");
    assert!(!frontier.contains("advisories-unconsulted"), "{frontier}");

    // Without a database, the frontier says no advisory was consulted.
    let files = project(&advised, "advised");
    assert_eq!(
        refused(&files, "advisories-unconsulted"),
        (1, Some(1)),
        "{}",
        text(&files, "frontier.toml")
    );
    assert!(!text(&files, "src/Security.lex.tex").contains("Advisory"));

    // Without deny.toml, an advisory nothing answers is refused, not modeled.
    let copy = scratch(
        "ue12",
        &[
            (
                "Cargo.toml",
                &std::fs::read_to_string(advised.join("Cargo.toml")).expect("manifest"),
            ),
            (
                "Cargo.lock",
                &std::fs::read_to_string(advised.join("Cargo.lock")).expect("lock"),
            ),
            (
                "src/lib.rs",
                &std::fs::read_to_string(advised.join("src/lib.rs")).expect("lib"),
            ),
        ],
    );
    let bare = Options {
        advisories: Some(db),
        ..options("advised")
    };
    let files = uorust::project_workspace(&copy, &bare).expect("projects");
    let frontier = text(&files, "frontier.toml");
    // Four refusals, one per advisory, and the work queue counts them.
    assert_eq!(
        refused(&files, "unanswered-advisory"),
        (4, Some(4)),
        "{frontier}"
    );
    assert!(!text(&files, "src/Security.lex.tex").contains("Advisory"));
    let _ = std::fs::remove_dir_all(copy);
}

#[test]
fn every_feature_of_this_workspace_is_accepted_ue_13() {
    let files = project(&root(), "uorust");
    let frontier = text(&files, "frontier.toml");
    assert!(
        !frontier.contains("feature-without-acceptance"),
        "{frontier}"
    );
    let interface = text(&files, "src/Interface.lex.tex");
    // `uorust mir` and the module `uorust::mir` are two features, told apart.
    for feature in [
        "featureRepoModelCodegen",
        "featureXtaskMain",
        "featureUorustMir",
        "featureUorustCommandMir",
    ] {
        assert!(
            interface.contains(&format!("name: {feature},")),
            "{feature}"
        );
    }
}

/// The `definition{...}` records of the given module texts, by name.
fn records(files: impl IntoIterator<Item = String>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for body in files {
        for chunk in body.split("definition{").skip(1) {
            let first = chunk.lines().next().unwrap_or_default();
            let record = if first.contains("body:") {
                first
            } else {
                chunk.split("\n}").next().unwrap_or_default()
            };
            let Some(name) = record.split("name: ").nth(1) else {
                continue;
            };
            let name: String = name
                .chars()
                .take_while(char::is_ascii_alphanumeric)
                .collect();
            out.insert(name, record.to_string());
        }
    }
    out
}

/// A `key := "value"` field of a rendered record.
fn field<'a>(record: &'a str, key: &str) -> &'a str {
    let start = record
        .find(&format!("{key} := \""))
        .unwrap_or_else(|| panic!("no {key} in {record}"))
        + key.len()
        + 5;
    let end = record[start..].find('"').expect("closing quote");
    &record[start..start + end]
}

#[test]
fn the_self_projection_reproduces_the_hand_model_ue_14() {
    let app = root().join("app/src");
    let hand_files: Vec<String> = std::fs::read_dir(&app)
        .expect("app/src")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.to_string_lossy().ends_with(".lex.tex"))
        .map(|path| std::fs::read_to_string(path).expect("hand module"))
        .collect();
    let hand = records(hand_files);
    let files = project(&root(), "uorust");
    let derived = records(
        files
            .iter()
            .filter(|(path, _)| path.starts_with("src/") && path.ends_with(".lex.tex"))
            .map(|(_, bytes)| String::from_utf8(bytes.clone()).expect("utf-8")),
    );

    let table: toml::Value = toml::from_str(
        &std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixpoint.toml"),
        )
        .expect("fixpoint.toml"),
    )
    .expect("fixpoint.toml parses");
    let rows = |key: &str| {
        table
            .get(key)
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
    };
    let get = |row: &toml::Value, key: &str| -> String {
        row.get(key)
            .and_then(|v| v.as_str())
            .expect(key)
            .to_string()
    };

    // Each pair whose two records do not both state the fact is drift.
    let mut drift = Vec::new();
    let mut accounted = std::collections::BTreeSet::new();
    for row in rows("pair") {
        let (h, d, fact) = (get(&row, "hand"), get(&row, "derived"), get(&row, "fact"));
        accounted.insert(h.clone());
        for (side, name, set) in [("hand", &h, &hand), ("derived", &d, &derived)] {
            match set.get(name) {
                None => drift.push(format!("{side} record {name} is missing")),
                Some(body) if !body.contains(&fact) => {
                    drift.push(format!("{side} record {name} does not state `{fact}`"));
                }
                Some(_) => {}
            }
        }
    }
    for row in rows("outside") {
        let h = get(&row, "hand");
        assert!(
            hand.contains_key(&h),
            "outside row names no hand record: {h}"
        );
        assert!(
            !get(&row, "reason").is_empty(),
            "{h} is outside without a reason"
        );
        // The reason is typed, and what each kind claims is checked.
        let body = &hand[&h];
        match get(&row, "kind").as_str() {
            "external-system" => assert!(
                body.contains("ComponentKind.externalSystem"),
                "{h} is outside as an external system but is not one"
            ),
            "private-stage" => {
                let module = get(&row, "module");
                let lib = std::fs::read_to_string(root().join("crates/uorust/src/lib.rs"))
                    .expect("lib.rs");
                assert!(
                    lib.lines().any(|l| l == format!("mod {module};")),
                    "{h} names {module}, which is not a private module of the library"
                );
            }
            "outside-rust" => {
                let path = root().join(get(&row, "path"));
                assert!(path.exists(), "{h} names a path that does not exist");
                let mut stack = vec![path];
                while let Some(at) = stack.pop() {
                    if at.is_dir() {
                        if at.ends_with("target")
                            || at.file_name().is_some_and(|n| {
                                n.to_string_lossy().starts_with('.') && n != ".github"
                            })
                        {
                            continue;
                        }
                        for entry in std::fs::read_dir(&at).expect("dir") {
                            stack.push(entry.expect("entry").path());
                        }
                    } else {
                        assert!(
                            at.extension().is_none_or(|e| e != "rs"),
                            "{h} is outside the Rust code, but {} is Rust",
                            at.display()
                        );
                    }
                }
            }
            "run-time" => {
                let gate = get(&row, "gate");
                let ids = std::fs::read_to_string(root().join("model/ids.toml")).expect("ids");
                let just = std::fs::read_to_string(root().join("Justfile")).expect("Justfile");
                assert!(
                    ids.contains(&format!("id = \"{gate}\""))
                        || just.lines().any(|l| l.starts_with(&format!("{gate}:"))),
                    "{h} names a gate, {gate}, that is neither a register ID nor a recipe"
                );
            }
            "reviewer-judgement" => assert!(
                h.starts_with("modeledThreat") || h.starts_with("modeledControl"),
                "{h} is a reviewer's judgement but not a modeled threat or control"
            ),
            "this-gate" => assert_eq!(h, "measureSelfModelDrift", "{h} is not this gate's measure"),
            other => panic!("{h} is outside with an untyped kind {other}"),
        }
        assert!(
            accounted.insert(h.clone()),
            "{h} is both paired and outside"
        );
    }
    // Every hand record of a checked kind is accounted for, one way or the other.
    let checked: Vec<String> = rows("checked")
        .iter()
        .map(|v| v.as_str().expect("prefix").to_string())
        .collect();
    for name in hand.keys() {
        let kind = checked.iter().find(|prefix| {
            name.strip_prefix(prefix.as_str())
                .and_then(|rest| rest.chars().next())
                .is_some_and(|c| c.is_ascii_uppercase())
        });
        if kind.is_some() && !accounted.contains(name) {
            drift.push(format!("hand record {name} is neither paired nor outside"));
        }
    }

    // The route table: the hand model's `routes`, in order, is uorust::ROUTES.
    let routes = &hand.get("routes").expect("hand routes")[..];
    let hand_routes: Vec<[String; 4]> = routes
        .split("record Route {")
        .skip(1)
        .map(|r| {
            let r = r.split('}').next().unwrap_or_default();
            ["id", "rustFact", "prismRecord", "facet"].map(|k| field(r, k).to_string())
        })
        .collect();
    let code_routes: Vec<[String; 4]> = uorust::ROUTES
        .iter()
        .map(|r| [r.id, r.rust_fact, r.prism_record, r.facet].map(str::to_string))
        .collect();
    if hand_routes != code_routes {
        drift.push(format!(
            "route table differs:\nhand {hand_routes:?}\ncode {code_routes:?}"
        ));
    }

    assert!(
        hand.get("measureSelfModelDrift")
            .is_some_and(|m| m.contains("threshold := 0")),
        "the hand model's drift threshold is 0"
    );
    assert!(
        drift.is_empty(),
        "self-model drift {}:\n{}",
        drift.len(),
        drift.join("\n")
    );
}

#[test]
fn no_lifted_kappa_turns_unlifted_ue_16() {
    use uorust::register::{admit, Register};
    // The incumbent registers: the committed fixture frontier and this
    // workspace's committed lift register.
    for (root, name, incumbent) in [
        (root(), "uorust", root().join("frontier.toml")),
        (fixture(), "thermostat", golden().join("frontier.toml")),
    ] {
        let files = project(&root, name);
        // Typed, keyed by κ, every function once.
        let candidate = uorust::lift_register(&files).expect("the frontier is a register");
        let committed = std::fs::read_to_string(&incumbent).expect("the committed register");
        let incumbent =
            Register::parse(&incumbent, &committed).expect("the committed register parses");
        let regressed = admit(&incumbent, &candidate);
        assert!(
            regressed.is_empty(),
            "{name}: lifted κ decided otherwise:\n{}",
            regressed.join("\n")
        );
        if name == "uorust" {
            assert!(
                committed == candidate.to_text(),
                "frontier.toml is not this workspace's lift register; run `just register`"
            );
        }
        let frontier: toml::Value =
            toml::from_str(&text(&files, "frontier.toml")).expect("frontier parses");
        let rows = |table: &str| {
            frontier
                .get(table)
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
        };
        for row in &rows("residue") {
            for field in ["function", "span", "reason", "detail"] {
                assert!(row.get(field).is_some(), "{name}: untyped residue {row}");
            }
        }
        // A divergence is a lifted row's alone, and it says where the
        // definition parts from the Rust function; the register keeps it.
        for table in ["residue", "foreign", "effect"] {
            for row in rows(table) {
                assert!(row.get("divergence").is_none(), "{name}: {row}");
            }
        }
        for row in rows("lifted") {
            if let Some(divergence) = row.get("divergence") {
                assert!(
                    divergence.as_str().is_some_and(|d| !d.trim().is_empty()),
                    "{name}: an empty divergence {row}"
                );
                let function = row.get("function").and_then(|v| v.as_str());
                assert!(
                    candidate
                        .rows
                        .values()
                        .any(|r| Some(r.function.as_str()) == function
                            && r.divergence.as_deref() == divergence.as_str()),
                    "{name}: the register drops the divergence of {row}"
                );
            }
        }
        for row in &rows("foreign") {
            let detail = row
                .get("detail")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            assert!(
                detail.contains(" crate with no LexLean model"),
                "{name}: {row}"
            );
        }
        // An effect row names the effect it performs or reaches.
        for row in rows("effect") {
            let effect = row
                .get("effect")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            assert!(
                effect.contains(" effect: ") || effect.contains("reaches an effect"),
                "{name}: an effect row without its effect: {row}"
            );
        }
    }
}

/// Every function of product code, counted from the source apart from the
/// engine: free functions, inherent and trait-impl methods and trait methods
/// at item level, outside `#[cfg(test)]` modules (inline, or out of line in
/// their own file or directory) and not `#[test]`, in the `src/` tree of
/// every package of the workspace at `root` ([`members`]).
fn product_functions(root: &Path) -> Vec<String> {
    fn cfg_test(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|a| {
            a.path().is_ident("cfg")
                && quote::ToTokens::to_token_stream(a)
                    .to_string()
                    .contains("test")
        })
    }
    fn is_test(attrs: &[syn::Attribute]) -> bool {
        attrs
            .iter()
            .any(|a| a.path().segments.last().is_some_and(|s| s.ident == "test"))
    }
    /// The functions of `list`, and in `tests` the file and the directory of
    /// each out-of-line `#[cfg(test)] mod name;`, which lie in `dir`.
    fn items(
        list: &[syn::Item],
        at: &str,
        dir: &Path,
        out: &mut Vec<String>,
        tests: &mut Vec<PathBuf>,
    ) {
        for item in list {
            match item {
                syn::Item::Fn(f) if !is_test(&f.attrs) => out.push(format!("{at}{}", f.sig.ident)),
                syn::Item::Impl(i) => {
                    for m in &i.items {
                        if let syn::ImplItem::Fn(f) = m {
                            if !is_test(&f.attrs) {
                                out.push(format!("{at}impl {}", f.sig.ident));
                            }
                        }
                    }
                }
                syn::Item::Trait(t) => {
                    for m in &t.items {
                        if let syn::TraitItem::Fn(f) = m {
                            out.push(format!("{at}trait {}", f.sig.ident));
                        }
                    }
                }
                syn::Item::Mod(m) if !cfg_test(&m.attrs) => {
                    if let Some((_, inner)) = &m.content {
                        items(inner, at, &dir.join(m.ident.to_string()), out, tests);
                    }
                }
                syn::Item::Mod(m) if m.content.is_none() => {
                    assert!(
                        !m.attrs.iter().any(|a| a.path().is_ident("path")),
                        "{at}a #[path] test module this walker does not resolve"
                    );
                    let name = m.ident.to_string();
                    tests.push(dir.join(format!("{name}.rs")));
                    tests.push(dir.join(name));
                }
                _ => {}
            }
        }
    }
    fn walk(dir: &Path, top: bool, out: &mut Vec<PathBuf>) {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .expect("dir")
            .map(|e| e.expect("entry").path())
            .collect();
        entries.sort();
        if !top && entries.iter().any(|p| p.ends_with("Cargo.toml")) {
            return;
        }
        for path in entries {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() {
                if !matches!(name, "target" | "fixtures") {
                    walk(&path, false, out);
                }
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    for package in members(root) {
        assert!(
            package.join("src").is_dir(),
            "{}: a member with no src/ tree",
            package.display()
        );
        let mut files = Vec::new();
        walk(&package.join("src"), true, &mut files);
        let (mut found, mut tests) = (Vec::new(), Vec::new());
        for path in &files {
            let text = std::fs::read_to_string(path).expect("source");
            let file = syn::parse_file(&text).expect("parses");
            // A module's children lie beside a crate root or a `mod.rs`, and
            // in the directory named after any other file.
            let parent = path.parent().expect("a directory");
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let root_file = matches!(stem, "lib" | "main" | "mod") || parent.ends_with("src/bin");
            let dir = if root_file {
                parent.to_path_buf()
            } else {
                parent.join(stem)
            };
            let mut functions = Vec::new();
            items(
                &file.items,
                &format!("{}: ", path.display()),
                &dir,
                &mut functions,
                &mut tests,
            );
            found.push((path, functions));
        }
        for (path, functions) in found {
            if !tests.iter().any(|t| path.starts_with(t)) {
                out.extend(functions);
            }
        }
    }
    out
}

/// The packages of the workspace whose root is `root`, as cargo decides
/// them and read apart from the engine (which asks `cargo metadata`): the
/// root itself when its manifest has a `[package]`, and every directory a
/// `workspace.members` entry names or matches, less `workspace.exclude`. A
/// package below `root` that is not a member is not this workspace's.
fn members(root: &Path) -> Vec<PathBuf> {
    let manifest: toml::Value = toml::from_str(
        &std::fs::read_to_string(root.join("Cargo.toml")).expect("the root manifest"),
    )
    .expect("the root manifest parses");
    let list = |key: &str| -> Vec<String> {
        manifest
            .get("workspace")
            .and_then(|w| w.get(key))
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .map(|v| v.as_str().expect("a path").to_string())
            .collect()
    };
    let excluded: Vec<PathBuf> = list("exclude").iter().map(|e| root.join(e)).collect();
    let mut out = Vec::new();
    if manifest.get("package").is_some() {
        out.push(root.to_path_buf());
    }
    for pattern in list("members") {
        let mut found = vec![root.to_path_buf()];
        for part in pattern.split('/').filter(|p| !p.is_empty() && *p != ".") {
            assert!(
                !part.contains("**") && !part.contains('['),
                "{pattern}: a member glob this reader does not expand"
            );
            let mut next = Vec::new();
            for dir in found {
                if !part.contains(['*', '?']) {
                    next.push(dir.join(part));
                    continue;
                }
                let mut matched: Vec<PathBuf> = std::fs::read_dir(&dir)
                    .expect("a member's parent")
                    .map(|e| e.expect("entry").path())
                    .filter(|p| p.is_dir())
                    .filter(|p| {
                        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        glob_matches(part, name)
                    })
                    .collect();
                matched.sort();
                next.extend(matched);
            }
            found = next;
        }
        for dir in found {
            assert!(
                dir.join("Cargo.toml").is_file(),
                "{pattern}: {} is a member with no manifest",
                dir.display()
            );
            if !excluded.contains(&dir) && !out.contains(&dir) {
                out.push(dir);
            }
        }
    }
    out
}

/// Whether one path component `name` matches the glob component `pattern`
/// (`*` any run of characters, `?` one character), as cargo's member globs.
fn glob_matches(pattern: &str, name: &str) -> bool {
    fn at(p: &[char], n: &[char]) -> bool {
        match p.split_first() {
            None => n.is_empty(),
            Some(('*', rest)) => (0..=n.len()).any(|i| at(rest, &n[i..])),
            Some(('?', rest)) => !n.is_empty() && at(rest, &n[1..]),
            Some((c, rest)) => n.first() == Some(c) && at(rest, &n[1..]),
        }
    }
    let (p, n): (Vec<char>, Vec<char>) = (pattern.chars().collect(), name.chars().collect());
    at(&p, &n)
}

/// Product code is what cargo makes a member of the workspace: the walker
/// reads the root manifest's `workspace.members` and `exclude` itself, so a
/// member with no row fails this test whatever its directory is called, and a
/// package that is not a member is not counted.
///
/// Left outside by that rule today, as an open row for a later projection of
/// that workspace under its own profile: the three packages the installed
/// Prism factory brings, none a member here. `prism-factory/host` declares
/// its own `[workspace]` and is upstream's factory host, held to upstream's
/// text by `prism-factory/upstream.toml` and `graph.py`; its one adapted file
/// is covered by the factory's own gate (UE-20), not by this lift register.
/// `prism-factory/certified/packages/rust-std/factory_plan` and
/// `factory_reweigh` are Rust that a LexLean production generated and that
/// `verify-artifact` replays byte for byte.
#[test]
fn every_function_of_product_code_has_exactly_one_row_ue_16() {
    // The walker's own rule, on a planted workspace: members by path and by
    // glob count, an excluded directory, a package that is not a member (its
    // own `[workspace]`, beside the members) and an out-of-line test module
    // do not.
    let package = |name: &str| format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n");
    let planted = scratch(
        "ue16-members",
        &[
            (
                "Cargo.toml",
                "[workspace]\nmembers = [\"crates/*\", \"tool\"]\nexclude = [\"crates/skipped\"]\n",
            ),
            ("crates/alpha/Cargo.toml", &package("alpha")),
            (
                "crates/alpha/src/lib.rs",
                "/// A.\npub fn alpha() {}\n/// T.\npub struct T;\nimpl T {\n    /// M.\n    pub fn method(&self) {}\n}\n",
            ),
            ("crates/beta/Cargo.toml", &package("beta")),
            (
                "crates/beta/src/lib.rs",
                "/// B.\npub fn beta() {}\nmod inner;\n#[cfg(test)]\nmod tests;\n",
            ),
            ("crates/beta/src/inner.rs", "/// I.\npub fn inner() {}\n"),
            ("crates/beta/src/tests.rs", "/// Only a test's.\npub fn helper() {}\n"),
            ("crates/beta/src/tests/deep.rs", "/// A test's too.\npub fn deep() {}\n"),
            ("crates/skipped/Cargo.toml", &package("skipped")),
            ("crates/skipped/src/lib.rs", "/// S.\npub fn skipped() {}\n"),
            ("tool/Cargo.toml", &package("tool")),
            ("tool/src/main.rs", "fn main() {}\n"),
            (
                "outside/host/Cargo.toml",
                &format!("{}\n[workspace]\n", package("host")),
            ),
            ("outside/host/src/lib.rs", "/// H.\npub fn host() {}\n"),
        ],
    );
    let names: Vec<String> = product_functions(&planted)
        .iter()
        .map(|f| f.rsplit(": ").next().unwrap_or_default().to_string())
        .collect();
    assert_eq!(
        names,
        ["alpha", "impl method", "inner", "beta", "main"],
        "{names:?}"
    );
    let _ = std::fs::remove_dir_all(planted);

    for (root, name) in [(root(), "uorust"), (fixture(), "thermostat")] {
        let files = project(&root, name);
        let frontier: toml::Value =
            toml::from_str(&text(&files, "frontier.toml")).expect("frontier parses");
        let mut rows: BTreeMap<String, Vec<&str>> = BTreeMap::new();
        for table in ["residue", "foreign", "effect", "lifted"] {
            for row in frontier
                .get(table)
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
            {
                let function = row
                    .get("function")
                    .and_then(|v| v.as_str())
                    .expect("a function")
                    .to_string();
                rows.entry(function).or_default().push(table);
            }
        }
        for (function, tables) in &rows {
            assert_eq!(tables.len(), 1, "{name}: {function} has rows in {tables:?}");
        }
        let source = product_functions(&root);
        assert_eq!(
            rows.len(),
            source.len(),
            "{name}: {} functions of product code, {} with a row",
            source.len(),
            rows.len()
        );
    }
}

/// The planted defect: a function the incumbent register records as lifted,
/// whose own source is unchanged (so its κ is the same question), is decided
/// unlifted because a function it calls stopped lifting. The gate refuses the
/// projection, by the register and by `uorust::write` into the old output,
/// and leaves the old projection in place.
#[test]
fn a_lifted_kappa_that_turns_unlifted_is_refused_ue_16() {
    use uorust::register::{admit, Status};
    let manifest =
        "[package]\nname = \"ratchet\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n";
    let caller = "/// Calls the test.\npub fn ready(x: u8) -> bool {\n    over(x)\n}\n";
    let before = scratch(
        "ue16-before",
        &[
            ("Cargo.toml", manifest),
            (
                "src/lib.rs",
                &format!("//! R.\n\n/// The test.\npub fn over(x: u8) -> bool {{\n    x > 1\n}}\n\n{caller}"),
            ),
        ],
    );
    let after = scratch(
        "ue16-after",
        &[
            ("Cargo.toml", manifest),
            (
                "src/lib.rs",
                &format!("//! R.\n\n/// The test.\npub fn over(x: u8) -> bool {{\n    x + 1 > 2\n}}\n\n{caller}"),
            ),
        ],
    );
    let old = project(&before, "ratchet");
    let new = project(&after, "ratchet");
    let incumbent = uorust::lift_register(&old).expect("a register");
    let candidate = uorust::lift_register(&new).expect("a register");
    let (kappa, row) = incumbent
        .rows
        .iter()
        .find(|(_, r)| r.function == "ratchet::ready")
        .expect("ready has a row");
    assert!(matches!(row.status, Status::Lifted(_)), "{row:?}");
    // The same question, now decided otherwise.
    assert!(
        !matches!(candidate.rows[kappa].status, Status::Lifted(_)),
        "{:?}",
        candidate.rows[kappa]
    );
    let refused = admit(&incumbent, &candidate);
    assert!(
        refused.len() == 1 && refused[0].contains(kappa.as_str()),
        "{refused:?}"
    );
    // The changed function is a new question: no regression of its own.
    assert!(refused.iter().all(|r| !r.starts_with("ratchet::over")));
    // Projecting into the incumbent's output is refused, and writes nothing.
    let out = std::env::temp_dir().join(format!("uorust-ue16-out-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    uorust::write(&out, &old).expect("a first projection");
    match uorust::write(&out, &new) {
        Err(UorustError::Regressed(rows)) => assert_eq!(rows, refused),
        other => panic!("expected Regressed, got {other:?}"),
    }
    assert!(matches!(
        uorust::check(&out, &new),
        Err(UorustError::Regressed(_))
    ));
    assert_eq!(
        std::fs::read(out.join("frontier.toml")).expect("kept"),
        old["frontier.toml"]
    );
    for dir in [before, after, out] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// The law per function, on a register the tool wrote: two functions of a
/// synthetic workspace stopped by one construct of their signatures, which
/// the register records among every construct their signatures state. The
/// planted defect moves one of them, unchanged, under another construct
/// while the register still records the first for it: `uorust::admit_register`
/// (the `uorust admit --parent` command) refuses it, naming the function,
/// whether or not the other moves too. Moving both on where the candidate's
/// signatures no longer state the construct (its blocker shown gone) raises
/// the other cause and is admitted.
#[test]
fn a_function_stays_under_its_construct_while_it_stops_one_ue_16() {
    use uorust::register::{Register, Status};
    let manifest =
        "[package]\nname = \"moves\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n";
    let lib = "//! M.\n\nuse std::cell::RefCell;\n\n\
               /// One.\npub fn one(x: &RefCell<u8>) -> u8 {\n    *x.borrow()\n}\n\n\
               /// Two.\npub fn two(x: &RefCell<u8>) -> u8 {\n    *x.borrow() + 1\n}\n";
    let root = scratch(
        "ue16-moves",
        &[("Cargo.toml", manifest), ("src/lib.rs", lib)],
    );
    let parent = uorust::lift_register(&project(&root, "moves")).expect("a register");
    let stopped = |name: &str| {
        parent
            .rows
            .iter()
            .find(|(_, r)| r.function == format!("moves::{name}"))
            .map(|(k, r)| (k.clone(), r.status.clone()))
            .expect("a row")
    };
    let ((k1, s1), (k2, s2)) = (stopped("one"), stopped("two"));
    let construct = s1.construct().map(|(c, d)| (c, d.to_string()));
    assert!(
        construct.is_some() && construct == s2.construct().map(|(c, d)| (c, d.to_string())),
        "both functions are stopped by one construct: {s1:?} {s2:?}"
    );
    let recorded = |k: &String| {
        let stops = parent.rows[k].stops.clone().unwrap_or_default();
        let (cause, detail) = s1.construct().expect("stopped");
        stops.contains(&(cause, detail.to_string()))
    };
    assert!(recorded(&k1) && recorded(&k2), "{parent:?}");
    let next = Status::Residue("form".into(), "a construct planted as the next".into());
    let with = |moved: &[&String], gone: bool| {
        let mut r: Register = parent.clone();
        for k in moved {
            let row = r.rows.get_mut(*k).expect("a row");
            row.status = next.clone();
            if gone {
                row.stops = Some(Vec::new());
            }
        }
        r
    };
    let dir = scratch("ue16-moves-registers", &[]);
    std::fs::create_dir_all(&dir).expect("dir");
    let file = |name: &str, r: &Register| {
        let path = dir.join(name);
        std::fs::write(&path, r.to_text()).expect("write");
        path
    };
    let at = file("parent.toml", &parent);
    // Both moved on, the construct gone from both signatures: the cause
    // `form` rises, admitted.
    let moved = with(&[&k1, &k2], true);
    assert!(uorust::admit_register(&at, &file("moved.toml", &moved)).is_ok());
    // One moved while its signature still states the construct: refused,
    // whether the other stays or moves too.
    for planted in [with(&[&k1], false), {
        let mut both = with(&[&k1, &k2], false);
        both.rows.get_mut(&k2).expect("two").stops = Some(Vec::new());
        both
    }] {
        match uorust::admit_register(&at, &file("planted.toml", &planted)) {
            Err(UorustError::Regressed(rows)) => {
                assert!(
                    rows.len() == 1 && rows[0].starts_with("moves::one "),
                    "{rows:?}"
                );
            }
            other => panic!("expected Regressed, got {other:?}"),
        }
    }
    for dir in [root, dir] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

fn mir_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mir-path")
}

/// The fixture projected through the MIR path on `verdicts`.
fn walk(
    verdicts: uorust::mir::verdict::Verdicts,
    trial: uorust::mir::Trial,
) -> BTreeMap<String, Vec<u8>> {
    let root = mir_fixture();
    let mut options = options("walk");
    options.mir = Some(mir_options(dumped(&root, "walk"), verdicts, trial));
    uorust::project_workspace(&root, &options).expect("projects")
}

/// The detail of the residue row of `function` in a projection's frontier.
fn residue_detail(files: &BTreeMap<String, Vec<u8>>, function: &str) -> Option<String> {
    let frontier: toml::Value = toml::from_str(&text(files, "frontier.toml")).expect("frontier");
    frontier
        .get("residue")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .find(|r| r.get("function").and_then(|v| v.as_str()) == Some(function))
        .and_then(|r| r.get("detail").and_then(|v| v.as_str()))
        .map(str::to_string)
}

fn lifted_functions(files: &BTreeMap<String, Vec<u8>>) -> Vec<String> {
    let frontier: toml::Value = toml::from_str(&text(files, "frontier.toml")).expect("frontier");
    frontier
        .get("lifted")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|r| r.get("function").and_then(|v| v.as_str()))
        .map(str::to_string)
        .collect()
}

#[test]
fn every_mir_kind_of_the_pinned_toolchain_has_a_coverage_row_ue_17() {
    let toolchain = std::fs::read_to_string(root().join("rust-toolchain.toml")).expect("toolchain");
    let source =
        std::fs::read_to_string(root().join("crates/uorust/mir/SOURCE.toml")).expect("source");
    let channel = |text: &str, key: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(&format!("{key} = \"")))
            .and_then(|v| v.strip_suffix('"'))
            .unwrap_or_default()
            .to_string()
    };
    assert_eq!(
        channel(&source, "toolchain"),
        channel(&toolchain, "channel")
    );
    let report = uorust::mir::coverage::report();
    assert!(report.contains(&format!("at toolchain {}", channel(&toolchain, "channel"))));
    for kind in [
        "StatementKind::Assign: lowered",
        "TerminatorKind::SwitchInt: lowered",
        "TerminatorKind::Assert: lowered",
        "Rvalue::BinaryOp: lowered",
        "BinOp::AddWithOverflow: lowered",
        "Rvalue::RawPtr: residue",
        "TerminatorKind::InlineAsm: residue",
    ] {
        assert!(report.contains(kind), "no coverage row {kind}");
    }
}

#[test]
fn the_mir_path_translates_and_admits_on_verdicts_ue_17() {
    use uorust::mir::verdict::Decision;
    use uorust::mir::Trial;
    // The dump: one file per body of the library.
    let fresh = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("mir-walk-fresh");
    uorust::mir::dump(&mir_fixture(), &fresh, &Default::default()).expect("the MIR dumps");
    assert!(std::fs::read_dir(fresh.join("lib-walk"))
        .expect("the library's dumps")
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .any(|n| n.starts_with("walk.count.")));
    // The trial run: every translation is admitted, with its loop helper,
    // the helper's measure and decrease theorems, and a theorem per
    // obligation; the prover reads the listing of candidates.
    let trial = walk(Default::default(), Trial::All);
    let behavior = text(&trial, "src/Behavior.lex.tex");
    let rows = frontier_rows(&trial, "lifted");
    let name = |function: &str| -> String {
        rows.iter()
            .find(|r| r["function"].as_str() == Some(function))
            .and_then(|r| r["definition"].as_str())
            .and_then(|d| d.strip_prefix("Behavior::"))
            .unwrap_or_else(|| panic!("no lifted trial declaration for {function}"))
            .to_string()
    };
    let capped = name("walk::capped_sum");
    let count = name("walk::count");
    let above = name("walk::above");
    let rise = name("walk::first_rise");
    for decl in [
        format!("name: {capped}Loop1,"),
        format!("name: decreases_{capped}Loop1_0,"),
        format!("name: holds_{capped}_0,"),
        format!("name: {capped},"),
        format!("name: {count},"),
        format!("name: {above},"),
        format!("name: {rise}Loop2,"),
    ] {
        assert!(
            behavior.contains(&decl),
            "the trial projection lacks {decl}"
        );
    }
    for decl in [
        "termination: {",
        // std's own `Option::unwrap_or`, lifted from its vendored MIR at
        // `u32`, and a promoted constant lowered from its own body.
        "name: stdOptionOptionUnwrapOrU32,",
        // `BTreeSet::insert` by its model, a backward function of the
        // mutable borrow; `str::trim` by its model over the prelude.
        "name: stdInsertString,",
        "(stdInsertString (stdInsertString (nil String) a) b)",
        "(Prelude::textTrim ",
        // A `for` loop over a slice: its iterator a cursor, the loop's
        // measure the distance between the cursor's positions, which its
        // guarded `next` is `Some` within.
        "measure: (prim subtract (project (project loopState item1) item2) (project (project loopState item1) item1) : Nat)",
        "name: stdAtNat,",
        // A closure: a definition over the value of its captures, called
        // from std's `find`, which is lifted from std's own MIR.
        "name: closureFirstNamed0,",
        "name: stdSliceIterIterFindClosure",
    ] {
        assert!(behavior.contains(decl), "the trial projection lacks {decl}");
    }
    let listing = text(&trial, uorust::mir::CANDIDATES);
    for function in ["walk::capped_sum", "walk::count"] {
        assert!(
            listing.contains(&format!("function = \"{function}\"\ndigest = ")),
            "{function} is not a candidate"
        );
    }
    // A kind or type the translator does not lower is a residue row that
    // names it.
    assert_eq!(
        residue_detail(&trial, "walk::bump").as_deref(),
        Some("mir::Ty: std::cell::Cell<u32>")
    );

    // The committed verdicts: the proved translation lifts, the refused one
    // is residue naming the verifier's diagnostic.
    let committed = verdicts(&mir_fixture().join("lift-verdicts.toml"));
    let decided = walk(committed.clone(), Trial::Off);
    for f in [
        "walk::capped_sum",
        "walk::distinct",
        "walk::first_blank",
        "walk::above",
        // `for` loops over a slice and a set, `break`, and `?` on an option.
        "walk::first_rise",
        "walk::sum_until",
        "walk::second_known",
        // A closure passed to std's `find`.
        "walk::first_named",
        // Invariant closure captures remain available in the loop state.
        "walk::any_above",
        // Signed integers compared as their bit patterns.
        "walk::first_negative",
    ] {
        assert!(
            lifted_functions(&decided).contains(&f.to_string()),
            "{f} does not lift"
        );
    }
    assert!(text(&decided, "src/Behavior.lex.tex").contains(&format!("name: {capped}Loop1,")));
    assert!(!decided.contains_key(uorust::mir::CANDIDATES));
    let count = residue_detail(&decided, "walk::count").expect("count is residue");
    assert!(count.starts_with("mir::Verdict: LLV"), "{count}");
    // The MIR path only adds: whatever lifts without it lifts with it.
    let surface = project(&mir_fixture(), "walk");
    for f in lifted_functions(&surface) {
        assert!(
            lifted_functions(&decided).contains(&f),
            "{f} stopped lifting"
        );
    }

    // A verdict of another digest admits nothing: a changed translation is a
    // new question.
    let mut stale = committed;
    for v in stale.rows.values_mut() {
        if v.status == Decision::Proved {
            v.digest = "0".repeat(64);
        }
    }
    let planted = walk(stale, Trial::Off);
    assert!(!lifted_functions(&planted).contains(&"walk::capped_sum".to_string()));
    assert_eq!(
        residue_detail(&planted, "walk::capped_sum").as_deref(),
        Some("mir::Verdict: a translation the verifier has not decided yet")
    );
    assert!(!text(&planted, "src/Behavior.lex.tex").contains("name: walkCappedSum,"));
}

/// `prove` on a register that decides every translation runs no verifier and
/// writes the same register back.
#[test]
fn the_prover_keeps_a_decided_register_ue_17() {
    let root = mir_fixture();
    let committed = std::fs::read_to_string(root.join("lift-verdicts.toml")).expect("verdicts");
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("mir-walk-prove");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
    let verdicts = dir.join("lift-verdicts.toml");
    std::fs::write(&verdicts, &committed).expect("verdicts");
    let summary = uorust::mir::prove(
        &root,
        &options("walk"),
        &dumped(&root, "walk"),
        &verdicts,
        &dir.join("scratch"),
    )
    .expect("proves");
    assert_eq!((summary.proved, summary.refused, summary.runs), (0, 0, 0));
    assert_eq!(
        std::fs::read_to_string(&verdicts).expect("verdicts"),
        committed
    );
}

/// A planted defect: a shard whose rust-src is absent. Its census refuses,
/// so the shard and the run are red, rather than translating every body
/// against no declarations and recording what it got.
#[test]
fn a_census_without_its_library_sources_refuses() {
    use uorust::mir::core;
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/core");
    let missing = fixture.join("absent/library");
    assert!(!missing.exists());
    assert!(
        core::core_census(&missing, &fixture.join("mir")).is_err(),
        "a census with no rust-src must refuse"
    );
}

/// Each library workflow's shards read rust-src before translating, and
/// its merge writes the register only when every shard succeeded.
#[test]
fn library_shards_read_their_sources_and_merge_only_whole() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for workflow in ["core.yml", "std.yml"] {
        let text = std::fs::read_to_string(root.join(".github/workflows").join(workflow))
            .expect("workflow");
        let translate = text
            .lines()
            .find(|l| l.contains("--only '$MODULE'"))
            .expect("the shard's census");
        let fetch = translate
            .find("scripts/mir/dump-lib.sh")
            .expect("rust-src fetched");
        assert!(fetch < translate.find("target/bin/uorust").expect("census"));
        let merge = &text[text.find("\n  merge:").expect("merge job")..];
        let refuse = merge
            .find("Refuse a partial set of shards")
            .expect("refusal");
        assert!(merge[refuse..].contains("needs.shard.result != 'success'"));
        assert!(refuse < merge.find("uorust merge").expect("merge step"));
        // The register lands through land-register.sh, which pushes it
        // with push.sh: both after the refusal.
        let land = merge
            .find("scripts/ci/land-register.sh")
            .expect("landing step");
        assert!(refuse < land);
        let script = std::fs::read_to_string(root.join("scripts/ci/land-register.sh"))
            .expect("land-register.sh");
        assert!(script.contains("scripts/ci/push.sh"));
    }
}

/// The witness of `crates/uorust/core/retracted.toml`'s row: a switch on
/// `cmp::Ordering`'s discriminant reads the discriminants core declares
/// (-1, 0, 1; a switch prints -1 as its `i8` bits, 255), not the variants'
/// indices. Read by index, the arm on 255 matched no variant and its call
/// was never lowered, so the body translated as if `Less` took the
/// otherwise arm (and `then_with`'s arm on 0 ran its closure on `Less`).
#[test]
fn a_switch_on_ordering_reads_its_declared_discriminants() {
    use uorust::mir::core;
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ordering");
    let shards = core::core_census(&fixture.join("library"), &fixture.join("mir")).expect("census");
    let row = shards
        .iter()
        .flat_map(|s| &s.rows)
        .find(|r| r.path == "core::cmp::less_reaches")
        .expect("less_reaches");
    // `Less` reaches the call this fixture leaves undefined.
    assert!(row.digest.is_none(), "{row:?}");
    let cause = row.cause.as_deref().unwrap_or_default();
    assert!(cause.contains("absent"), "{cause}");
    let row = |path: &str| {
        shards
            .iter()
            .flat_map(|s| &s.rows)
            .find(|r| r.path == path)
            .unwrap_or_else(|| panic!("{path}"))
    };
    // An arm on each declared discriminant leaves the otherwise arm
    // unreached; read by index, `Greater` (2) fell to it.
    let every = row("core::cmp::every_arm");
    assert!(every.digest.is_some(), "{every:?}");
    // `Less`, the first variant, takes the otherwise arm, not the arm on 0.
    let equal = row("core::cmp::equal_arm");
    let cause = equal.cause.as_deref().unwrap_or_default();
    assert!(cause.contains("on_other"), "{cause}");
}

/// The witness of `crates/uorust/core/retracted.toml`'s `int_sqrt` rows:
/// rustc prints two constant items of `u32_stages`, `middle_stage!`'s
/// `HALF_BITS` (`u16::BITS >> 1`) and `last_stage!`'s (`u32::BITS >> 1`),
/// and every use of either, under one path; only the dumps' file names
/// tell them apart (`HALF_BITS#1`). The census took the first dump it read
/// for the path, `last_stage!`'s, so `middle_stage!`'s `u16`
/// `LOWER_HALF_1_BITS` read 16 where its own item is 8. A use of such a
/// path is now residue.
#[test]
fn a_constant_path_rustc_prints_for_two_items_is_refused() {
    use uorust::mir::core;
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/twins");
    // The two items have other values: `u32`'s `BITS` and `u16`'s.
    let mut names: Vec<PathBuf> = std::fs::read_dir(fixture.join("mir/num"))
        .expect("dumps")
        .map(|e| e.expect("dump").path())
        .filter(|p| p.to_string_lossy().contains("-HALF_BITS"))
        .collect();
    names.sort();
    let texts: Vec<String> = names
        .iter()
        .map(|p| std::fs::read_to_string(p).expect("dump"))
        .collect();
    assert_eq!(texts.len(), 2);
    // The first read, the one the old census took, is `u32`'s; the `u16`
    // constant that uses it is `middle_stage!`'s, whose own is `u16`'s.
    assert!(
        texts[0].contains("const num::<impl u32>::BITS"),
        "{}",
        texts[0]
    );
    assert!(
        texts[1].contains("const num::<impl u16>::BITS"),
        "{}",
        texts[1]
    );
    let lower = std::fs::read_to_string(
        fixture.join("mir/num/core.num-imp-int_sqrt-u32_stages-LOWER_HALF_1_BITS.2-2-012.ElaborateBoxDerefs.before.mir"),
    )
    .expect("lower");
    assert!(lower.contains("const u32_stages::LOWER_HALF_1_BITS: u16"));
    let shards = core::core_census(&fixture.join("library"), &fixture.join("mir")).expect("census");
    let rows: Vec<&core::CoreRow> = shards.iter().flat_map(|s| &s.rows).collect();
    let half: Vec<&&core::CoreRow> = rows
        .iter()
        .filter(|r| r.path.contains("::HALF_BITS"))
        .collect();
    assert_eq!(half.len(), 2, "{rows:?}");
    assert!(half.iter().all(|r| r.digest.is_some()), "{half:?}");
    assert_ne!(
        half[0].digest, half[1].digest,
        "the two items' values differ"
    );
    let user = rows
        .iter()
        .find(|r| r.path.ends_with("::LOWER_HALF_1_BITS"))
        .expect("LOWER_HALF_1_BITS");
    let cause = user.cause.as_deref().unwrap_or_default();
    assert!(cause.contains("more than one constant item"), "{user:?}");
}

/// Every retraction of a library register names its body by a κ and path
/// the register holds, and its witness is a test of these tests, which
/// `cargo test` runs: a retraction whose witness fails is a red run.
#[test]
fn every_retraction_is_witnessed() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut tests = String::new();
    for entry in std::fs::read_dir(crate_dir.join("tests")).expect("tests") {
        let p = entry.expect("entry").path();
        if p.extension().is_some_and(|x| x == "rs") {
            tests.push_str(&std::fs::read_to_string(&p).expect("test source"));
        }
    }
    for register in ["core", "std"] {
        let dir = crate_dir.join(register);
        for r in uorust::queue::retractions(&dir).expect("retractions") {
            let test = format!("#[test]\nfn {}() {{", r.witness);
            assert!(tests.contains(&test), "{}: no test {}", r.path, r.witness);
            let held = std::fs::read_dir(&dir)
                .expect("register")
                .filter_map(Result::ok)
                .filter(|e| {
                    e.path()
                        .file_stem()
                        .is_some_and(|n| n != uorust::queue::RETRACTED)
                })
                .filter_map(|e| std::fs::read_to_string(e.path()).ok())
                .any(|t| t.contains(&format!("path = {:?}\nkappa = \"{}\"", r.path, r.kappa)));
            assert!(held, "{} is no row of the {register} register", r.path);
        }
    }
}

/// Core's register: each body of a module shard translated at the canonical
/// instantiation and recorded by κ; a body recorded as translated that is
/// residue later is a regression the register refuses.
#[test]
fn core_bodies_are_translated_into_a_monotone_register() {
    use uorust::mir::core;
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/core");
    let shards = core::core_census(&fixture.join("library"), &fixture.join("mir")).expect("census");
    assert_eq!(shards.len(), 1);
    let bool_shard = &shards[0];
    assert_eq!(bool_shard.module, "bool");
    let status = |path: &str| {
        bool_shard
            .rows
            .iter()
            .find(|r| r.path == path)
            .map(|r| r.digest.is_some())
    };
    // The impl is named by the type its source implements; a generic body
    // is translated with its parameters at the canonical type.
    assert_eq!(status("core::bool::bool::then_some"), Some(true));
    assert_eq!(status("core::bool::bool::ok_or"), Some(true));
    // `then` calls its closure parameter, which the canonical type is not.
    assert_eq!(status("core::bool::bool::then"), Some(false));
    let row = bool_shard
        .rows
        .iter()
        .find(|r| r.path == "core::bool::bool::then_some")
        .expect("then_some");
    assert_eq!(row.at, format!("T={}", core::CANONICAL));

    let dir = std::env::temp_dir().join(format!("uorust-core-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    core::write_core_register(&dir, &shards).expect("writes");
    let summary = std::fs::read_to_string(dir.join("frontier.toml")).expect("summary");
    assert!(summary.contains("[module.bool]\nbodies = 4\ntranslated = 2\nresidue = 2"));
    assert!(core::core_regressions(&dir, &shards).is_empty());
    // A register that records `then` as translated refuses the census that
    // leaves it residue.
    let rows = std::fs::read_to_string(dir.join("bool.toml")).expect("rows");
    let then = rows
        .split("\n\n")
        .find(|r| r.contains("path = \"core::bool::bool::then\""))
        .expect("then's row");
    let forged = then
        .lines()
        .take_while(|l| !l.starts_with("status"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\nstatus = \"translated\"\ndigest = \"0\"";
    std::fs::write(dir.join("bool.toml"), rows.replace(then, &forged)).expect("forges");
    let regressed = core::core_regressions(&dir, &shards);
    assert_eq!(regressed.len(), 1, "{regressed:?}");
    assert!(regressed[0].starts_with("core::bool::bool::then:"));
    // Retracted by its κ and path, it may be residue; under another path,
    // or with no witness, it still refuses.
    let kappa = then
        .lines()
        .find_map(|l| l.strip_prefix("kappa = "))
        .expect("then's kappa");
    let retract = |path: &str, witness: &str| {
        std::fs::write(
            dir.join("retracted.toml"),
            format!("[[retracted]]\npath = {path:?}\nkappa = {kappa}\nreason = \"r\"\nwitness = {witness:?}\n"),
        )
        .expect("retracts");
        core::core_regressions(&dir, &shards)
    };
    assert!(retract("core::bool::bool::then", "w").is_empty());
    assert_eq!(retract("core::bool::bool::then_some", "w").len(), 1);
    assert_eq!(retract("core::bool::bool::then", "").len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Every body of the core fixture is embedded as a value of the MIR
/// datatype beside the semantics, with its census; dumps that do not say
/// the target they were built for are refused, since an integer's width
/// depends on it.
#[test]
fn every_core_body_is_embedded_ue_19() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/core");
    let work = std::env::temp_dir().join(format!("uorust-embed-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    let out = work.join("out");
    let refused = uorust::embed::embed(
        "core",
        &fixture.join("library"),
        &fixture.join("mir"),
        &out,
        &[],
        &uorust::embed::Outputs::default(),
    );
    assert!(
        matches!(refused, Err(UorustError::Malformed { .. })),
        "{refused:?}"
    );
    // The fixture's dumps with the target the bodies were built for.
    let dumps = work.join("mir");
    std::fs::create_dir_all(dumps.join("bool")).expect("the dumps");
    for entry in std::fs::read_dir(fixture.join("mir/bool")).expect("the fixture") {
        let entry = entry.expect("a dump");
        std::fs::copy(entry.path(), dumps.join("bool").join(entry.file_name())).expect("copied");
    }
    std::fs::write(
        dumps.join("target.toml"),
        "triple = \"x86_64-unknown-linux-gnu\"\npointer_width = 64\n",
    )
    .expect("the target");
    let harness = work.join("harness");
    let census = uorust::embed::embed(
        "core",
        &fixture.join("library"),
        &dumps,
        &out,
        &[],
        &uorust::embed::Outputs {
            harness: Some(&harness),
            ..Default::default()
        },
    )
    .expect("the embedding");
    assert_eq!(census.bodies, 4);
    assert!(census.evaluable <= census.bodies);
    let rendered = census.render();
    assert!(rendered.contains("bodies = 4"), "{rendered}");
    let part =
        std::fs::read_to_string(out.join("src/CoreMir/BoolPart0.lex.tex")).expect("the bodies");
    assert_eq!(part.matches("result: Mir::Body").count(), 4);
    assert!(part.contains("\\importmodule{Mir}"));
    assert!(out.join("src/Mir.lex.tex").exists());
    let config = std::fs::read_to_string(out.join("lexlean.toml")).expect("the project");
    assert!(config.contains("\"src/CoreMir/BoolPart0.lex.tex\", \"src/Mir.lex.tex\""));
    assert_eq!(
        std::fs::read_to_string(out.join("census.toml")).expect("the census"),
        rendered
    );
    // No body of `bool` is a scalar method of an integer, so there is
    // nothing for a harness to call.
    assert!(!harness.exists());
    let _ = std::fs::remove_dir_all(&work);
}

/// Compiler inputs shared by the method-identity and bounded-proof regressions.
fn mir_method_fixture(case: &str) -> (PathBuf, PathBuf, PathBuf) {
    let work =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("mir-method-homonyms-{case}"));
    let _ = std::fs::remove_dir_all(&work);
    let source = work.join("source");
    std::fs::create_dir_all(source.join("src")).expect("source");
    std::fs::write(
        source.join("Cargo.toml"),
        r#"[package]
name = "owners"
version = "0.0.0"
edition = "2021"
[workspace]
"#,
    )
    .expect("manifest");
    let mut code = String::new();
    for module in ["a", "b", "c"] {
        code.push_str(&format!("pub mod {module} {{ pub struct Token; impl Token {{ pub fn convert(mut value: u32) -> u32 {{ while value > 0 {{ value -= 1; }} value }} }} }}\n"));
    }
    std::fs::write(source.join("src/lib.rs"), code).expect("source bytes");
    let lock = std::process::Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&source)
        .output()
        .expect("lock fixture");
    assert!(
        lock.status.success(),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
    let dumps = work.join("all");
    uorust::mir::dump(&source, &dumps, &Default::default()).expect("actual compiler MIR");
    (work, source, dumps)
}

/// UE-28: every homonym retains its source-qualified name.
#[test]
fn mir_method_homonyms_have_stable_names_ue_28() {
    let (work, source, dumps) = mir_method_fixture("names");
    let candidates = |at: &Path| {
        let mut opts = options("owners");
        opts.mir = Some(mir_options(
            at.to_path_buf(),
            Default::default(),
            uorust::mir::Trial::All,
        ));
        let files = uorust::project_workspace(&source, &opts).expect("trial projection");
        let listing: toml::Value = text(&files, uorust::mir::CANDIDATES)
            .parse()
            .expect("candidates");
        listing
            .get("candidate")
            .and_then(toml::Value::as_array)
            .unwrap_or_else(|| panic!("no MIR candidates: {}", text(&files, "frontier.toml")))
            .iter()
            .map(|row| {
                (
                    row["function"].as_str().expect("function").to_string(),
                    row["decls"]
                        .as_array()
                        .expect("declarations")
                        .last()
                        .expect("body")
                        .as_str()
                        .expect("name")
                        .to_string(),
                )
            })
            .collect::<BTreeMap<_, _>>()
    };
    let all = candidates(&dumps);
    assert_eq!(all.len(), 3, "a conversion method was lost: {all:?}");
    assert_eq!(
        all.values()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
    let selected = work.join("selected");
    std::fs::create_dir_all(selected.join("lib-owners")).expect("selected view");
    for name in ["configuration.toml", "layouts.toml"] {
        std::fs::copy(dumps.join(name), selected.join(name)).expect("compiler metadata");
    }
    for file in std::fs::read_dir(dumps.join("lib-owners")).expect("compiler bodies") {
        let file = file.expect("file").path();
        if file.extension().is_some_and(|e| e == "mir")
            && std::fs::read_to_string(&file)
                .expect("MIR")
                .contains("fn c::<impl at ")
        {
            std::fs::copy(
                &file,
                selected
                    .join("lib-owners")
                    .join(file.file_name().expect("name")),
            )
            .expect("exact compiler body");
        }
    }
    let one = candidates(&selected);
    assert_eq!(one.len(), 1, "the selected compiler view: {one:?}");
    assert_eq!(
        one["owners::c::Token::convert"],
        all["owners::c::Token::convert"]
    );
    std::fs::remove_dir_all(work).expect("cleanup");
}

/// UE-29: bounded plans preserve all work and dependency order.
#[test]
fn mir_proof_batches_keep_dependencies_and_refuse_invalid_budgets_ue_29() {
    use std::collections::BTreeSet;
    let graph = BTreeMap::from([
        ("leaf".to_string(), BTreeSet::new()),
        ("middle".to_string(), BTreeSet::from(["leaf".to_string()])),
        ("top".to_string(), BTreeSet::from(["middle".to_string()])),
        (
            "cycle-a".to_string(),
            BTreeSet::from(["cycle-b".to_string()]),
        ),
        (
            "cycle-b".to_string(),
            BTreeSet::from(["cycle-a".to_string()]),
        ),
    ]);
    let plan = uorust::mir::proof_batches(&graph, 2).expect("bounded plan");
    let mut done = BTreeSet::new();
    for batch in &plan {
        assert!(!batch.is_empty() && batch.len() <= 2);
        assert!(batch.is_disjoint(&done), "work was repeated");
        for node in batch {
            assert!(
                graph[node]
                    .iter()
                    .all(|dependency| done.contains(dependency) || batch.contains(dependency)),
                "a dependency was omitted"
            );
        }
        done.extend(batch.iter().cloned());
    }
    assert_eq!(done, graph.keys().cloned().collect());
    assert!(plan
        .iter()
        .any(|batch| batch.contains("cycle-a") && batch.contains("cycle-b")));
    for maximum in [0, 1] {
        assert!(matches!(
            uorust::mir::proof_batches(&graph, maximum),
            Err(UorustError::Tool(_))
        ));
    }
    let missing = BTreeMap::from([("caller".to_string(), BTreeSet::from(["absent".to_string()]))]);
    assert!(matches!(
        uorust::mir::proof_batches(&missing, 2),
        Err(UorustError::Tool(_))
    ));
    assert!(uorust::mir::proof_batches(&BTreeMap::new(), 2)
        .expect("no translations")
        .is_empty());
    let (work, source, dumps) = mir_method_fixture("batches");
    let verdicts = work.join("verdicts.toml");
    let scratch = work.join("trial");
    let proved =
        uorust::mir::prove_bounded(&source, &options("owners"), &dumps, &verdicts, &scratch, 1)
            .expect("actual SDK proofs");
    assert_eq!(proved.proved, 3);
    assert_eq!(proved.refused, 0);
    // Each single-function trial discovers its actual axiom set, then verifies it.
    assert_eq!(proved.runs, 6);
    let reused =
        uorust::mir::prove_bounded(&source, &options("owners"), &dumps, &verdicts, &scratch, 1)
            .expect("reuse verified digests");
    assert_eq!(reused.runs, 0, "unchanged proofs were repeated");
    std::fs::remove_dir_all(work).expect("cleanup");
}

/// UE-30: registry alias discovery cannot erase an admitted number model.
#[test]
fn registered_number_alias_keeps_its_admitted_model_ue_30() {
    let dir = scratch("registered-number-alias", &[
        ("Cargo.toml", "[package]\nname = \"number_alias\"\nversion = \"0.1.0\"\nedition = \"2021\"\nrust-version = \"1.97\"\n[dependencies]\nnum-bigint = \"=0.4.8\"\nnum-rational = \"=0.4.2\"\n"),
        ("src/lib.rs", "use num_rational::BigRational;\npub fn keep(r: BigRational) -> BigRational { r }\n"),
    ]);
    let locked = std::process::Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dir)
        .output()
        .expect("Cargo lock");
    assert!(
        locked.status.success(),
        "{}",
        String::from_utf8_lossy(&locked.stderr)
    );
    let lifted = |files: &BTreeMap<String, Vec<u8>>| {
        frontier_rows(files, "lifted")
            .iter()
            .any(|row| row["function"].as_str() == Some("number_alias::keep"))
    };
    assert!(
        lifted(&project(&dir, "number_alias")),
        "the registered alias lost its type model"
    );
    let lock = dir.join("Cargo.lock");
    let original = std::fs::read_to_string(&lock).expect("lock");
    let checksum = "f83d14da390562dca69fc84082e73e548e1ad308d24accdedd2720017cb37824";
    assert!(
        original.contains(checksum),
        "the fixture did not lock the registered archive"
    );
    std::fs::write(
        &lock,
        original.replace(checksum, &"0".repeat(checksum.len())),
    )
    .expect("planted wrong checksum");
    assert!(
        !lifted(&project(&dir, "number_alias")),
        "a different archive acquired the registered model"
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-33: transport is modeled without assuming JSON storage equality.
#[test]
fn locked_json_carriers_preserve_data_and_refuse_unmodeled_equality_ue_33() {
    let dir = scratch("registered-json-carrier", &[
        ("Cargo.toml", "[package]\nname = \"json_carrier\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[dependencies]\nserde_json = \"=1.0.114\"\n"),
        ("src/lib.rs", "use serde_json::{Value, Number};\n#[derive(PartialEq)] pub struct Held { pub payload: Value, pub id: u8 }\npub fn keep(v: Value) -> Value { v }\npub fn number(n: Number) -> Number { n }\npub fn pair(v: Value, n: u8) -> (Value, u8) { (v, n) }\npub fn id(h: &Held) -> u8 { h.id }\npub fn compared(a: Value, b: Value) -> bool { a == b }\npub fn numbers_equal(a: Number, b: Number) -> bool { a == b }\npub fn held_equal(a: Held, b: Held) -> bool { a == b }\n"),
    ]);
    let locked = std::process::Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dir)
        .output()
        .expect("lock");
    assert!(
        locked.status.success(),
        "{}",
        String::from_utf8_lossy(&locked.stderr)
    );
    let files = project(&dir, "json_carrier");
    let functions = |files: &BTreeMap<String, Vec<u8>>| -> Vec<String> {
        frontier_rows(files, "lifted")
            .iter()
            .filter_map(|row| row["function"].as_str().map(str::to_string))
            .collect()
    };
    let lifted = functions(&files);
    for function in [
        "json_carrier::keep",
        "json_carrier::number",
        "json_carrier::pair",
        "json_carrier::id",
    ] {
        assert!(
            lifted.iter().any(|name| name == function),
            "carrier function did not lift: {function}"
        );
    }
    assert!(
        !lifted.iter().any(|name| name == "json_carrier::compared"),
        "Storage equality was substituted for JSON equality"
    );
    for function in ["json_carrier::numbers_equal", "json_carrier::held_equal"] {
        assert!(
            !lifted.iter().any(|name| name == function),
            "Unmodeled JSON equality lifted: {function}"
        );
    }
    let out = dir.join("projection");
    uorust::write(&out, &files).expect("write JSON projection");
    let kernel = std::process::Command::new("lexlean")
        .args([
            "verify",
            "src/Behavior.lex.tex",
            "--diagnostic-format",
            "json",
        ])
        .current_dir(&out)
        .output()
        .expect("actual SDK verifier");
    assert!(
        kernel.status.success(),
        "{}{}",
        String::from_utf8_lossy(&kernel.stdout),
        String::from_utf8_lossy(&kernel.stderr)
    );
    let lock = dir.join("Cargo.lock");
    let original = std::fs::read_to_string(&lock).expect("lock");
    let checksum = "c5f09b1bd632ef549eaa9f60a1f8de742bdbc698e6cee2095fc84dde5f549ae0";
    assert!(original.contains(checksum));
    std::fs::write(&lock, original.replace(checksum, &"0".repeat(64))).expect("wrong checksum");
    assert!(
        functions(&project(&dir, "json_carrier")).is_empty(),
        "An unregistered archive acquired the JSON model"
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-34: an alias transports its reference, never a frozen cell value.
#[test]
fn shared_cell_references_transport_without_snapshotting_contents_ue_34() {
    let dir = scratch("shared-cell-references", &[
        ("Cargo.toml", "[package]\nname = \"shared_cells\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        ("src/lib.rs", "use std::{cell::{Cell, RefCell}, rc::Rc, sync::Arc};\n#[derive(PartialEq)] pub struct Held { pub first: Rc<RefCell<u8>>, pub second: Rc<RefCell<u8>>, pub id: u8 }\npub fn keep(x: Rc<RefCell<u8>>) -> Rc<RefCell<u8>> { x }\npub fn cloned(x: &Rc<RefCell<u8>>) -> Rc<RefCell<u8>> { Rc::clone(x) }\npub fn pair(x: Rc<RefCell<u8>>) -> (Rc<RefCell<u8>>, Rc<RefCell<u8>>) { (x.clone(), x) }\npub fn field(h: &Held) -> Rc<RefCell<u8>> { h.first.clone() }\npub fn id(h: &Held) -> u8 { h.id }\npub fn arc_keep(x: Arc<Cell<u8>>) -> Arc<Cell<u8>> { x }\npub fn read(x: &Rc<RefCell<u8>>) -> u8 { *x.borrow() }\npub fn write(x: &Rc<RefCell<u8>>, value: u8) { *x.borrow_mut() = value; }\npub fn make(value: u8) -> Rc<RefCell<u8>> { Rc::new(RefCell::new(value)) }\npub fn counts(x: &Rc<RefCell<u8>>) -> usize { Rc::strong_count(x) }\npub fn equal(x: Rc<RefCell<u8>>, y: Rc<RefCell<u8>>) -> bool { x == y }\npub fn held_equal(x: Held, y: Held) -> bool { x == y }\n"),
    ]);
    let files = project(&dir, "shared_cells");
    let lifted: Vec<String> = frontier_rows(&files, "lifted")
        .iter()
        .filter_map(|row| row["function"].as_str().map(str::to_string))
        .collect();
    for name in ["keep", "cloned", "pair", "field", "id", "arc_keep"] {
        assert!(
            lifted.contains(&format!("shared_cells::{name}")),
            "reference transport did not lift: {name}"
        );
    }
    for name in ["read", "write", "make", "counts", "equal", "held_equal"] {
        assert!(
            !lifted.contains(&format!("shared_cells::{name}")),
            "unmodeled shared heap operation lifted: {name}"
        );
    }
    let out = dir.join("projection");
    uorust::write(&out, &files).expect("write reference projection");
    let kernel = std::process::Command::new("lexlean")
        .args([
            "verify",
            "src/Behavior.lex.tex",
            "--diagnostic-format",
            "json",
        ])
        .current_dir(&out)
        .output()
        .expect("actual SDK verifier");
    assert!(
        kernel.status.success(),
        "{}{}",
        String::from_utf8_lossy(&kernel.stdout),
        String::from_utf8_lossy(&kernel.stderr)
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-35: wide field and recursive-proof records must survive SDK formatting.
#[test]
fn long_type_declarations_are_canonical_without_rewriting_ue_35() {
    let dir = scratch("wide-type-layout", &[
        ("Cargo.toml", "[package]\nname = \"wide_types\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        ("src/lib.rs", "pub struct PayloadWithAnIntentionallyLongQualifiedNameForCanonicalLayout { pub value: u8 }\npub struct Holder { pub overridesWithAnIntentionallyWideFieldName: Vec<PayloadWithAnIntentionallyLongQualifiedNameForCanonicalLayout> }\npub enum RecursiveTreeWithAnIntentionallyLongQualifiedNameForCanonicalLayout { End, Next(Box<RecursiveTreeWithAnIntentionallyLongQualifiedNameForCanonicalLayout>) }\npub fn keep(h: Holder) -> Holder { h }\npub fn depth(t: &RecursiveTreeWithAnIntentionallyLongQualifiedNameForCanonicalLayout) -> u32 { match t { RecursiveTreeWithAnIntentionallyLongQualifiedNameForCanonicalLayout::End => 0, RecursiveTreeWithAnIntentionallyLongQualifiedNameForCanonicalLayout::Next(rest) => depth(rest).saturating_add(1) } }\n"),
    ]);
    let files = project(&dir, "wide_types");
    let out = dir.join("projection");
    uorust::write(&out, &files).expect("write wide declarations");
    let check = std::process::Command::new("lexlean")
        .args(["fmt", "--check"])
        .current_dir(&out)
        .output()
        .expect("actual SDK formatter");
    assert!(
        check.status.success(),
        "{}{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-36: an immutable slice owner retains the whole ordered sequence.
#[test]
fn owned_slices_retain_sequences_without_snapshotting_cells_ue_36() {
    let dir = scratch("owned-slices", &[
        ("Cargo.toml", "[package]\nname = \"owned_slices\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        ("src/lib.rs", "use std::{cell::Cell, rc::Rc, sync::Arc};\npub struct Held { pub order: Rc<[u8]> }\npub fn keep(x: Box<[u8]>) -> Box<[u8]> { x }\npub fn arc_keep(x: Arc<[u8]>) -> Arc<[u8]> { x }\npub fn len(x: &Held) -> usize { x.order.len() }\npub fn cloned(x: &Held) -> Rc<[u8]> { x.order.clone() }\npub fn counts(x: &Held) -> usize { Rc::strong_count(&x.order) }\npub fn cells(x: Rc<[Cell<u8>]>) -> Rc<[Cell<u8>]> { x }\n"),
    ]);
    let files = project(&dir, "owned_slices");
    let lifted: Vec<String> = frontier_rows(&files, "lifted")
        .iter()
        .filter_map(|r| r["function"].as_str().map(str::to_string))
        .collect();
    for name in ["keep", "arc_keep", "len", "cloned"] {
        assert!(
            lifted.contains(&format!("owned_slices::{name}")),
            "owned slice did not lift: {name}"
        );
    }
    for name in ["counts", "cells"] {
        assert!(
            !lifted.contains(&format!("owned_slices::{name}")),
            "unmodeled owner operation lifted: {name}"
        );
    }
    let out = dir.join("projection");
    uorust::write(&out, &files).expect("write owned slices");
    let checked = std::process::Command::new("lexlean")
        .args([
            "verify",
            "src/Behavior.lex.tex",
            "--diagnostic-format",
            "json",
        ])
        .current_dir(&out)
        .output()
        .expect("SDK verifier");
    assert!(
        checked.status.success(),
        "{}{}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr)
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-37: default BitVec's logical sequence excludes capacity and dead bits.
#[test]
fn locked_bit_vectors_retain_logical_bits_without_raw_storage_ue_37() {
    let dir=scratch("locked-bit-vector", &[
        ("Cargo.toml","[package]\nname = \"logical_bits\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[dependencies]\nbitvec = \"=1.0.1\"\n"),
        ("src/lib.rs","use bitvec::prelude::BitVec;\npub fn keep(x: BitVec) -> BitVec { x }\npub fn len(x: &BitVec) -> usize { x.len() }\npub fn capacity(x: &BitVec) -> usize { x.capacity() }\npub fn raw(x: &BitVec) -> &[usize] { x.as_raw_slice() }\n"),
    ]);
    let locked = std::process::Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dir)
        .output()
        .expect("lock bitvec");
    assert!(
        locked.status.success(),
        "{}",
        String::from_utf8_lossy(&locked.stderr)
    );
    let files = project(&dir, "logical_bits");
    let functions = |files: &BTreeMap<String, Vec<u8>>| -> Vec<String> {
        frontier_rows(files, "lifted")
            .iter()
            .filter_map(|r| r["function"].as_str().map(str::to_string))
            .collect()
    };
    let lifted = functions(&files);
    for name in ["keep", "len"] {
        assert!(
            lifted.contains(&format!("logical_bits::{name}")),
            "bit vector did not lift: {name}"
        );
    }
    for name in ["capacity", "raw"] {
        assert!(
            !lifted.contains(&format!("logical_bits::{name}")),
            "unmodeled storage operation lifted: {name}"
        );
    }
    let out = dir.join("projection");
    uorust::write(&out, &files).expect("write bit projection");
    let checked = std::process::Command::new("lexlean")
        .args([
            "verify",
            "src/Behavior.lex.tex",
            "--diagnostic-format",
            "json",
        ])
        .current_dir(&out)
        .output()
        .expect("SDK verifier");
    assert!(
        checked.status.success(),
        "{}{}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr)
    );
    let lock = dir.join("Cargo.lock");
    let source = std::fs::read_to_string(&lock).expect("lock text");
    let checksum = "1bc2832c24239b0141d5674bb9174f9d68a8b5b3f2753311927c172ca46f7e9c";
    assert!(source.contains(checksum));
    std::fs::write(&lock, source.replace(checksum, &"0".repeat(64))).expect("wrong checksum");
    assert!(
        functions(&project(&dir, "logical_bits")).is_empty(),
        "another archive acquired the bit-vector model"
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-38: stale bytes never inherit proof, while observed policies save discovery.
#[test]
fn native_proof_refresh_reuses_policy_hints_but_verifies_new_bytes_ue_38() {
    let (work, source, dumps) = mir_method_fixture("policy-cache");
    let verdicts = work.join("verdicts.toml");
    let scratch = work.join("trial");
    let opts = options("owners");
    let first = uorust::mir::prove_bounded(&source, &opts, &dumps, &verdicts, &scratch, 3)
        .expect("initial SDK proofs");
    assert_eq!(first.proved, 3);
    assert_eq!(first.refused, 0);
    let mut stale = uorust::mir::verdict::Verdicts::read(&verdicts).expect("actual verdicts");
    let key = stale.rows.keys().next().expect("proved row").clone();
    let actual = stale.rows[&key].clone();
    assert!(
        !actual.axioms.is_empty(),
        "fixture has no observed policies"
    );
    stale.rows.get_mut(&key).expect("row").digest = "0".repeat(64);
    std::fs::write(&verdicts, stale.to_text()).expect("stale digest");
    let refreshed = uorust::mir::prove_bounded(&source, &opts, &dumps, &verdicts, &scratch, 3)
        .expect("SDK refresh");
    assert_eq!(refreshed.proved, 1);
    assert_eq!(refreshed.refused, 0);
    assert_eq!(
        refreshed.runs, 1,
        "the observed policies were needlessly rediscovered"
    );
    let fresh = uorust::mir::verdict::Verdicts::read(&verdicts).expect("fresh verdict");
    assert_eq!(fresh.rows[&key], actual);
    let replay =
        uorust::mir::prove_bounded(&source, &opts, &dumps, &verdicts, &scratch, 3).expect("replay");
    assert_eq!(replay.runs, 0);
    let mut wrong = fresh;
    let row = wrong.rows.get_mut(&key).expect("row");
    row.digest = "0".repeat(64);
    for policy in row.axioms.values_mut() {
        policy.clear();
    }
    std::fs::write(&verdicts, wrong.to_text()).expect("incorrect hints");
    let corrected = uorust::mir::prove_bounded(&source, &opts, &dumps, &verdicts, &scratch, 3)
        .expect("SDK correction");
    assert_eq!(corrected.proved, 1);
    assert_eq!(corrected.refused, 0);
    assert_eq!(corrected.runs, 2);
    let checked = uorust::mir::verdict::Verdicts::read(&verdicts).expect("corrected verdict");
    assert_eq!(checked.rows[&key], actual);
    std::fs::remove_dir_all(work).expect("cleanup");
}

/// UE-39: expanding an alias must not rebind its element names at the use site.
#[test]
fn aliases_keep_their_declaring_scope_through_nested_types_ue_39() {
    let dir=scratch("scoped-aliases", &[
        ("Cargo.toml","[package]\nname = \"scoped_aliases\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        ("src/lib.rs","pub mod left; pub mod right; pub mod holder;\n"),
        ("src/left.rs","use std::collections::BTreeMap;\npub struct Node { pub value: u8 }\npub type Pool = BTreeMap<String, Vec<(String, Vec<Node>)>>;\npub type Items<Node> = Vec<Node>;\npub type Nested = Pool;\npub struct Local { pub pool: Pool }\npub fn keep(x: Local) -> Local { x }\n"),
        ("src/right.rs","pub struct Node { pub flag: bool }\npub type Pool = Vec<Node>;\n"),
        ("src/holder.rs","use crate::left::{Nested as Pool, Items};\nuse crate::right::Node;\npub struct Held { pub pool: Pool, pub flags: Items<Node>, pub others: crate::right::Pool }\npub fn keep(x: Held) -> Held { x }\npub fn retained(x: Held) -> crate::left::Nested { x.pool }\npub fn unread(x: std::cell::UnsafeCell<u8>) -> std::cell::UnsafeCell<u8> { x }\n"),
    ]);
    let rust = std::process::Command::new("cargo")
        .args(["check", "--offline"])
        .env("CARGO_TARGET_DIR", dir.join("rust-check"))
        .current_dir(&dir)
        .output()
        .expect("check source fixture");
    assert!(
        rust.status.success(),
        "{}",
        String::from_utf8_lossy(&rust.stderr)
    );
    let files = project(&dir, "scoped_aliases");
    let lifted: Vec<String> = frontier_rows(&files, "lifted")
        .iter()
        .filter_map(|r| r["function"].as_str().map(str::to_string))
        .collect();
    for name in [
        "scoped_aliases::left::keep",
        "scoped_aliases::holder::keep",
        "scoped_aliases::holder::retained",
    ] {
        assert!(
            lifted.contains(&name.to_string()),
            "alias scope did not lift: {name}"
        );
    }
    assert!(!lifted.contains(&"scoped_aliases::holder::unread".to_string()));
    let behavior = text(&files, "src/Behavior.lex.tex");
    assert!(behavior.contains("LeftNode"), "{behavior}");
    assert!(behavior.contains("RightNode"), "{behavior}");
    assert!(
        behavior.contains("{name: flags, type: (List ScopedAliasesRightNode)}"),
        "generic alias parameter was rebound: {behavior}"
    );
    let out = dir.join("projection");
    uorust::write(&out, &files).expect("write aliases");
    let checked = std::process::Command::new("lexlean")
        .args([
            "verify",
            "src/Behavior.lex.tex",
            "--diagnostic-format",
            "json",
        ])
        .current_dir(&out)
        .output()
        .expect("SDK verifier");
    assert!(
        checked.status.success(),
        "{}{}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr)
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-52: native callers cannot reuse a proof of a different source callee.
#[test]
fn native_proof_cache_binds_source_dependencies_ue_52() {
    let first="pub fn flag(x:bool)->bool { x }\npub fn caller(mut n:u8,x:bool)->bool {while n>0 {n-=1;} flag(x)}\n";
    let dir=scratch("native-context-cache",&[("Cargo.toml","[package]\nname=\"context_cache\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[workspace]\n"),("src/lib.rs",first)]);
    let lock = std::process::Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dir)
        .output()
        .expect("lock");
    assert!(lock.status.success());
    let dumps = dumped(&dir, "native-context-cache");
    let register = dir.join("verdicts.toml");
    let prove = || {
        uorust::mir::prove_bounded(
            &dir,
            &options("context_cache"),
            &dumps,
            &register,
            &dir.join("trial"),
            1,
        )
        .expect("actual context proof")
    };
    let before = prove();
    assert_eq!((before.proved, before.refused), (1, 0));
    let original = verdicts(&register);
    std::fs::write(dir.join("src/lib.rs"), first.replace("{ x }", "{ !x }"))
        .expect("changed callee only");
    let after = prove();
    assert!(
        after.runs > 0,
        "changed source dependency reused an old SDK decision"
    );
    assert_eq!((after.proved, after.refused), (1, 0));
    assert_ne!(
        original.rows.values().next().expect("old").digest,
        verdicts(&register)
            .rows
            .values()
            .next()
            .expect("fresh")
            .digest
    );
    let replay = prove();
    assert_eq!((replay.proved, replay.refused, replay.runs), (0, 0, 0));
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-51: one binary conversion rule covers all primitive word widths.
#[test]
fn numeric_word_constructors_follow_widths_ue_51() {
    let types = [
        ("u8", u8::BITS, false),
        ("u16", u16::BITS, false),
        ("u32", u32::BITS, false),
        ("u64", u64::BITS, false),
        ("u128", u128::BITS, false),
        ("usize", usize::BITS, false),
        ("i8", i8::BITS, true),
        ("i16", i16::BITS, true),
        ("i32", i32::BITS, true),
        ("i64", i64::BITS, true),
        ("i128", i128::BITS, true),
        ("isize", isize::BITS, true),
    ];
    let mut code="use num_bigint::BigInt;\nuse num_rational::BigRational;\npub fn source_unsigned(x:u64)->BigInt {BigInt::from(x)}\n".to_string();
    for (ty, _, _) in types {
        code.push_str(&format!(
            "pub fn from_{ty}(mut n:u8,x:{ty})->BigInt {{while n>0 {{n-=1;}} BigInt::from(x)}}\n"
        ));
    }
    code.push_str("pub fn ratio(mut n:u8,x:BigInt)->BigRational {while n>0 {n-=1;} BigRational::from_integer(x)}\n");
    code.push_str("#[test] fn reference_boundaries() {\n");
    for (ty, _, _) in types {
        code.push_str(&format!("assert_eq!(from_{ty}(0,{ty}::MIN).to_string(),{ty}::MIN.to_string()); assert_eq!(from_{ty}(0,{ty}::MAX).to_string(),{ty}::MAX.to_string());\n"));
    }
    code.push_str("}\n");
    let dir=scratch("word-constructors",&[("Cargo.toml","[package]\nname=\"word_constructors\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[workspace]\n[dependencies]\nnum-bigint=\"=0.4.8\"\nnum-rational=\"=0.4.2\"\n"),("src/lib.rs",&code)]);
    let reference = std::process::Command::new("cargo")
        .args(["test", "--offline"])
        .current_dir(&dir)
        .output()
        .expect("Rust word boundary vectors");
    assert!(
        reference.status.success(),
        "{}",
        String::from_utf8_lossy(&reference.stderr)
    );
    let files = uorust::project_workspace(&dir, &options("word_constructors"))
        .expect("source word construction");
    assert!(
        frontier_rows(&files, "lifted")
            .iter()
            .any(|r| r["function"].as_str() == Some("word_constructors::source_unsigned")),
        "source unsigned construction is not lifted"
    );
    let dumps = dumped(&dir, "word-constructors");
    let mut opts = options("word_constructors");
    opts.mir = Some(mir_options(
        dumps.clone(),
        Default::default(),
        uorust::mir::Trial::All,
    ));
    let files = uorust::project_workspace(&dir, &opts).expect("native constructors");
    let listing = text(&files, uorust::mir::CANDIDATES);
    for (ty, _, _) in types {
        assert!(
            listing.contains(&format!("word_constructors::from_{ty}\"")),
            "missing width {ty}: {listing}"
        );
    }
    assert!(listing.contains("word_constructors::ratio\""));
    let register =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("word-constructor-verdicts.toml");
    let summary = uorust::mir::prove_bounded(
        &dir,
        &options("word_constructors"),
        &dumps,
        &register,
        &dir.join("trial"),
        8,
    )
    .expect("actual word proofs");
    assert_eq!(summary.refused, 0);
    let decisions = verdicts(&register);
    opts.mir = Some(mir_options(
        dumps,
        decisions.clone(),
        uorust::mir::Trial::Off,
    ));
    let files = uorust::project_workspace(&dir, &opts).expect("proved word constructors");
    let rows = frontier_rows(&files, "lifted");
    let name = |f: &str| {
        rows.iter()
            .find(|r| r["function"].as_str() == Some(&format!("word_constructors::{f}")))
            .and_then(|r| r["definition"].as_str())
            .expect("proved constructor")
            .to_string()
    };
    let mut vectors = Vec::new();
    for (ty, width, signed) in types {
        let mask = u128::MAX >> (u128::BITS - width);
        let half = 1u128 << (width - 1);
        let f = format!("from_{ty}");
        if signed {
            vectors.push((
                f.clone(),
                format!("0 {half}"),
                format!("(prim negate ({half} : Int) : Int)"),
            ));
            vectors.push((
                f.clone(),
                format!("0 {mask}"),
                "(prim negate (1 : Int) : Int)".to_string(),
            ));
            vectors.push((
                f,
                format!("0 {}", half - 1),
                format!("({} : Int)", half - 1),
            ));
        } else {
            vectors.push((f.clone(), "0 0".to_string(), "(0 : Int)".to_string()));
            vectors.push((f, format!("0 {mask}"), format!("({mask} : Int)")));
        }
    }
    vectors.push((
        "source_unsigned".into(),
        u64::MAX.to_string(),
        format!("({} : Int)", u64::MAX),
    ));
    vectors.push((
        "ratio".into(),
        "0 (prim negate (2 : Int) : Int)".into(),
        "((prim negate (2 : Int) : Int), (1 : Int))".into(),
    ));
    let mut model="\\begin{lexlean}{WordVectors}\n\\useglossary{uorust.titles@1.0.0}\n\\importmodule{Behavior}\n\\title{Lifted Rust behavior}\n\\begin{semanticmodule}\n".to_string();
    for (i, (f, args, expected)) in vectors.iter().enumerate() {
        let def = name(f);
        let decl = def.strip_prefix("Behavior::").expect("namespace");
        let policy = decisions
            .rows
            .values()
            .find(|r| r.function == format!("word_constructors::{f}"))
            .and_then(|r| r.axioms.get(decl))
            .cloned()
            .unwrap_or_default();
        model.push_str(&format!("theorem{{name: word{i}, parameters: [], statement: (({def} {args}) = {expected}), proof: decide{{}}, axioms: [{}]}}\n",policy.join(", ")));
    }
    model.push_str("\\end{semanticmodule}\n\\end{lexlean}\n");
    let out = dir.join("projection");
    uorust::write(&out, &files).expect("write constructors");
    std::fs::write(out.join("src/WordVectors.lex.tex"), model).expect("word vectors");
    let checked = std::process::Command::new("lexlean")
        .args([
            "verify",
            "src/WordVectors.lex.tex",
            "--diagnostic-format",
            "json",
        ])
        .current_dir(&out)
        .output()
        .expect("SDK word boundary proof");
    assert!(
        checked.status.success(),
        "{}{}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr)
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-49: numeric predicates and accessors retain the locked crates' stored-value semantics.
#[test]
fn native_numeric_observations_follow_locked_values_ue_49() {
    let dir = scratch("native-numeric-observations", &[
        ("Cargo.toml", "[package]\nname = \"numeric_observations\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n[dependencies]\nnum-bigint = \"=0.4.8\"\nnum-rational = \"=0.4.2\"\nnum-traits = \"=0.2.19\"\n"),
        ("src/lib.rs", r#"
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Zero, One, Signed};
pub fn iz(mut n:u8,x:&BigInt)->bool { while n>0 {n-=1;} x.is_zero() }
pub fn io(mut n:u8,x:&BigInt)->bool { while n>0 {n-=1;} x.is_one() }
pub fn ip(mut n:u8,x:&BigInt)->bool { while n>0 {n-=1;} x.is_positive() }
pub fn ine(mut n:u8,x:&BigInt)->bool { while n>0 {n-=1;} x.is_negative() }
pub fn ia(mut n:u8,x:&BigInt)->BigInt { while n>0 {n-=1;} x.abs() }
pub fn isig(mut n:u8,x:&BigInt)->BigInt { while n>0 {n-=1;} x.signum() }
pub fn rz(mut n:u8,x:&BigRational)->bool { while n>0 {n-=1;} x.is_zero() }
pub fn ro(mut n:u8,x:&BigRational)->bool { while n>0 {n-=1;} x.is_one() }
pub fn rp(mut n:u8,x:&BigRational)->bool { while n>0 {n-=1;} x.is_positive() }
pub fn rn(mut n:u8,x:&BigRational)->bool { while n>0 {n-=1;} x.is_negative() }
pub fn ra(mut n:u8,x:&BigRational)->BigRational { while n>0 {n-=1;} x.abs() }
pub fn ri(mut n:u8,x:&BigRational)->bool { while n>0 {n-=1;} x.is_integer() }
pub fn numer(mut n:u8,x:&BigRational)->&BigInt { while n>0 {n-=1;} x.numer() }
pub fn denom(mut n:u8,x:&BigRational)->&BigInt { while n>0 {n-=1;} x.denom() }
pub fn izero(mut n:u8)->BigInt { while n>0 {n-=1;} BigInt::zero() }
pub fn ione(mut n:u8)->BigInt { while n>0 {n-=1;} BigInt::one() }
pub fn rzero(mut n:u8)->BigRational { while n>0 {n-=1;} BigRational::zero() }
pub fn rone(mut n:u8)->BigRational { while n>0 {n-=1;} BigRational::one() }
pub fn unsupported(mut n:u8,x:&BigInt)->u64 { while n>0 {n-=1;} x.bits() }
#[test] fn reference_raw_ratios() {
    let raw=BigRational::new_raw(BigInt::zero(),BigInt::zero()); assert!(raw.is_zero() && raw.is_one());
    let q=BigRational::new_raw(BigInt::from(2),BigInt::from(-2)); assert!(q.is_negative());
    assert_eq!((q.abs().numer().clone(),q.abs().denom().clone()),(BigInt::from(-2),BigInt::from(-2)));
}
"#),
    ]);
    let reference = std::process::Command::new("cargo")
        .args(["test", "--offline"])
        .current_dir(&dir)
        .output()
        .expect("locked Rust numeric vectors");
    assert!(
        reference.status.success(),
        "{}",
        String::from_utf8_lossy(&reference.stderr)
    );
    let dumps = dumped(&dir, "native-numeric-observations");
    let mut opts = options("numeric_observations");
    opts.mir = Some(mir_options(
        dumps.clone(),
        Default::default(),
        uorust::mir::Trial::All,
    ));
    let files = uorust::project_workspace(&dir, &opts).expect("numeric observations");
    let listing = text(&files, uorust::mir::CANDIDATES);
    let names = [
        "iz", "io", "ip", "ine", "ia", "isig", "rz", "ro", "rp", "rn", "ra", "ri", "numer",
        "denom", "izero", "ione", "rzero", "rone",
    ];
    for name in names {
        assert!(
            listing.contains(&format!("numeric_observations::{name}\"")),
            "missing numeric observation {name}: {listing}"
        );
    }
    assert!(!listing.contains("numeric_observations::unsupported\""));
    // Reuse actual digest-bound SDK decisions when only the surrounding
    // acceptance vectors change. UORust invalidates changed translations.
    let register =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("numeric-observation-verdicts.toml");
    let proof = uorust::mir::prove_bounded(
        &dir,
        &options("numeric_observations"),
        &dumps,
        &register,
        &dir.join("trial"),
        8,
    )
    .expect("actual numeric SDK proofs");
    assert_eq!(proof.refused, 0);
    opts.mir = Some(mir_options(
        dumps.clone(),
        verdicts(&register),
        uorust::mir::Trial::Off,
    ));
    let files = uorust::project_workspace(&dir, &opts).expect("proved observations");
    let out = dir.join("projection");
    uorust::write(&out, &files).expect("write observations");
    let rows = frontier_rows(&files, "lifted");
    let named = |function: &str| {
        rows.iter()
            .find(|r| r["function"].as_str() == Some(&format!("numeric_observations::{function}")))
            .and_then(|r| r["definition"].as_str())
            .expect("proved definition")
            .to_string()
    };
    let vectors = [
        ("iz", "0 (0 : Int)", "true"),
        ("iz", "0 (2 : Int)", "false"),
        ("io", "0 (1 : Int)", "true"),
        ("ip", "0 (0 : Int)", "false"),
        ("ine", "0 (prim negate (2 : Int) : Int)", "true"),
        ("ia", "0 (prim negate (2 : Int) : Int)", "(2 : Int)"),
        (
            "isig",
            "0 (prim negate (2 : Int) : Int)",
            "(prim negate (1 : Int) : Int)",
        ),
        ("rz", "0 ((0 : Int), (0 : Int))", "true"),
        ("ro", "0 ((0 : Int), (0 : Int))", "true"),
        ("rn", "0 ((2 : Int), (prim negate (2 : Int) : Int))", "true"),
        (
            "rp",
            "0 ((prim negate (2 : Int) : Int), (prim negate (2 : Int) : Int))",
            "true",
        ),
        (
            "ra",
            "0 ((2 : Int), (prim negate (2 : Int) : Int))",
            "((prim negate (2 : Int) : Int), (prim negate (2 : Int) : Int))",
        ),
        ("ri", "0 ((2 : Int), (1 : Int))", "true"),
        ("numer", "0 ((2 : Int), (0 : Int))", "(2 : Int)"),
        ("denom", "0 ((2 : Int), (0 : Int))", "(0 : Int)"),
        ("izero", "0", "(0 : Int)"),
        ("ione", "0", "(1 : Int)"),
        ("rzero", "0", "((0 : Int), (1 : Int))"),
        ("rone", "0", "((1 : Int), (1 : Int))"),
    ];
    let mut model =
        "\\begin{lexlean}{NumericVectors}\n\\useglossary{uorust.titles@1.0.0}\n\\importmodule{Behavior}\n\\title{Lifted Rust behavior}\n\\begin{semanticmodule}\n"
            .to_string();
    let decisions = verdicts(&register);
    for (i, (f, args, expected)) in vectors.iter().enumerate() {
        let definition = named(f);
        let declaration = definition
            .strip_prefix("Behavior::")
            .expect("native namespace");
        let policy = decisions
            .rows
            .values()
            .find(|r| r.function == format!("numeric_observations::{f}"))
            .and_then(|r| r.axioms.get(declaration))
            .cloned()
            .unwrap_or_default();
        model.push_str(&format!("theorem{{ name: vector{i}, parameters: [], statement: (({definition} {args}) = {expected}), proof: decide{{}}, axioms: [{}] }}\n",policy.join(", ")));
    }
    model.push_str("\\end{semanticmodule}\n\\end{lexlean}\n");
    std::fs::write(out.join("src/NumericVectors.lex.tex"), model).expect("closed numeric vectors");
    let checked = std::process::Command::new("lexlean")
        .args([
            "verify",
            "src/NumericVectors.lex.tex",
            "--diagnostic-format",
            "json",
        ])
        .current_dir(&out)
        .output()
        .expect("actual SDK numeric vectors");
    assert!(
        checked.status.success(),
        "{}{}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr)
    );
    let lock = dir.join("Cargo.lock");
    let original = std::fs::read_to_string(&lock).expect("lock");
    std::fs::write(
        &lock,
        original.replace(
            "c89e69e7e0f03bea5ef08013795c25018e101932225a656383bd384495ecc367",
            &"0".repeat(64),
        ),
    )
    .expect("wrong checksum");
    let refused = uorust::project_workspace(&dir, &opts).expect("checksum refusal");
    assert!(!frontier_rows(&refused, "lifted")
        .iter()
        .any(|r| r["function"].as_str() == Some("numeric_observations::iz")));
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-48: early exits retain the updates already performed through mutable parameters.
#[test]
fn source_propagation_preserves_mutable_state_ue_48() {
    let dir = scratch("threaded-propagation", &[
        ("Cargo.toml", "[package]\nname = \"propagate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n"),
        ("src/lib.rs", r#"
pub struct State { pub flag: bool }
pub fn option(s: &mut State, x: Option<bool>) -> Option<bool> {
    s.flag = true; let y = x?; s.flag = y; Some(y)
}
pub fn result(s: &mut State, x: Result<bool, bool>) -> Result<bool, bool> {
    s.flag = true; let y = x?; s.flag = y; Ok(y)
}
pub fn callee(s: &mut State) -> Result<bool, bool> { s.flag = true; Err(false) }
pub fn caller(s: &mut State) -> Result<bool, bool> {
    let y = callee(s)?; s.flag = y; Ok(y)
}
pub fn both(a: &mut State, b: &mut State, x: Option<bool>) -> Option<bool> {
    a.flag = true; b.flag = true; let y = x?; a.flag = y; Some(y)
}
pub fn alias(s: &mut State, x: Option<bool>) -> Option<bool> {
    let other = s; other.flag = true; let y = x?; Some(y)
}
pub fn conversion(s: &mut State, x: Result<bool, bool>) -> Result<bool, u8> {
    s.flag = true; let y = x?; Ok(y)
}
pub fn option_failure() -> bool {
    let mut s = State { flag: false }; let r = option(&mut s, None); r == None && s.flag
}
pub fn option_success() -> bool {
    let mut s = State { flag: true }; let r = option(&mut s, Some(false)); r == Some(false) && !s.flag
}
pub fn result_failure() -> bool {
    let mut s = State { flag: false }; let r = result(&mut s, Err(false)); r == Err(false) && s.flag
}
pub fn result_success() -> bool {
    let mut s = State { flag: true }; let r = result(&mut s, Ok(false)); r == Ok(false) && !s.flag
}
pub fn callee_failure() -> bool {
    let mut s = State { flag: false }; let r = caller(&mut s); r == Err(false) && s.flag
}
pub fn two_states() -> bool {
    let mut a = State { flag: false }; let mut b = State { flag: false };
    let r = both(&mut a, &mut b, None); r == None && a.flag && b.flag
}
#[test] fn option_failure_vector() { assert!(option_failure()); }
#[test] fn option_success_vector() { assert!(option_success()); }
#[test] fn result_failure_vector() { assert!(result_failure()); }
#[test] fn result_success_vector() { assert!(result_success()); }
#[test] fn callee_failure_vector() { assert!(callee_failure()); }
#[test] fn two_states_vector() { assert!(two_states()); }
"#),
    ]);
    let reference = std::process::Command::new("cargo")
        .args(["test", "--offline"])
        .current_dir(&dir)
        .output()
        .expect("reference Rust vectors");
    assert!(
        reference.status.success(),
        "{}",
        String::from_utf8_lossy(&reference.stderr)
    );
    let files = uorust::project_workspace(&dir, &options("propagate")).expect("source projection");
    let rows = frontier_rows(&files, "lifted");
    for name in [
        "option",
        "result",
        "callee",
        "caller",
        "both",
        "option_failure",
        "option_success",
        "result_failure",
        "result_success",
        "callee_failure",
        "two_states",
    ] {
        assert!(
            rows.iter()
                .any(|r| r["function"].as_str() == Some(&format!("propagate::{name}"))),
            "missing threaded propagation {name}: {}",
            text(&files, "frontier.toml")
        );
    }
    assert!(!rows
        .iter()
        .any(|r| r["function"].as_str() == Some("propagate::alias")));
    assert!(!rows
        .iter()
        .any(|r| r["function"].as_str() == Some("propagate::conversion")));
    let behavior = text(&files, "src/Behavior.lex.tex");
    assert!(
        behavior.matches("proof: decide").count() >= 6,
        "missing executable acceptance roots: {behavior}"
    );
    let out = dir.join("projection");
    uorust::write(&out, &files).expect("write propagation projection");
    let checked = std::process::Command::new("lexlean")
        .args([
            "verify",
            "src/Behavior.lex.tex",
            "--diagnostic-format",
            "json",
        ])
        .current_dir(&out)
        .output()
        .expect("actual SDK propagation proof");
    assert!(
        checked.status.success(),
        "{}{}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr)
    );
    // Plant loss of the update on the failure path. The same acceptance
    // roots must reject it; a merely well-typed tuple is not evidence.
    let marker = "item0 := (ctor Option.none [Bool]), item1 := s1";
    assert!(behavior.contains(marker), "missing failure-state boundary");
    let mut wrong = files.clone();
    wrong.insert(
        "src/Behavior.lex.tex".into(),
        behavior
            .replacen(marker, "item0 := (ctor Option.none [Bool]), item1 := s", 1)
            .into_bytes(),
    );
    let mutant = dir.join("lost-state");
    uorust::write(&mutant, &wrong).expect("write planted state-loss defect");
    let rejected = std::process::Command::new("lexlean")
        .args([
            "verify",
            "src/Behavior.lex.tex",
            "--diagnostic-format",
            "json",
        ])
        .current_dir(&mutant)
        .output()
        .expect("SDK rejects lost state");
    let diagnostic = String::from_utf8_lossy(&rejected.stdout);
    assert!(
        !rejected.status.success()
            && diagnostic.contains("optionFailure")
            && diagnostic.contains("is false"),
        "{diagnostic}"
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-47: native workspaces use the same core-derived IEEE formats as source lifting.
#[test]
fn native_workspace_floats_preserve_root_names_ue_47() {
    let dir = scratch("native-root-float-name", &[
        ("Cargo.toml", "[package]\nname = \"float_shadow\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n"),
        ("src/lib.rs", "#[allow(non_camel_case_types)] pub struct f64 { pub flag: bool }\npub fn flag(mut n: u8, x: f64) -> bool { while n > 0 { n -= 1; } x.flag }\n"),
    ]);
    let lock = std::process::Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dir)
        .output()
        .expect("lock");
    assert!(lock.status.success());
    let mut opts = options("float_shadow");
    opts.mir = Some(mir_options(
        dumped(&dir, "native-root-float-name"),
        Default::default(),
        uorust::mir::Trial::All,
    ));
    let files = uorust::project_workspace(&dir, &opts).expect("root primitive-name declaration");
    assert!(
        text(&files, uorust::mir::CANDIDATES).contains("float_shadow::flag\""),
        "the root declaration was replaced by a primitive"
    );
    let out = dir.join("projection");
    uorust::write(&out, &files).expect("write root declaration");
    let checked = std::process::Command::new("lexlean")
        .args([
            "check",
            "src/Behavior.lex.tex",
            "--diagnostic-format",
            "json",
        ])
        .current_dir(&out)
        .output()
        .expect("SDK type check");
    assert!(
        checked.status.success(),
        "{}{}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr)
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-47: native workspaces use the same core-derived IEEE formats as source lifting.
#[test]
fn native_workspace_floats_share_source_readings_ue_47() {
    let dir = scratch("native-workspace-floats", &[
        ("Cargo.toml", "[package]\nname = \"native_floats\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n"),
        ("src/lib.rs", "pub fn id32(mut n: u8, x: f32) -> f32 { while n > 0 { n -= 1; } x }\npub fn id64(mut n: u8, x: f64) -> f64 { while n > 0 { n -= 1; } x }\npub fn bits32(mut n: u8, x: f32) -> u32 { while n > 0 { n -= 1; } x.to_bits() }\npub fn arithmetic64(mut n: u8, x: f64, y: f64) -> f64 { while n > 0 { n -= 1; } (x + y).abs() }\npub fn rebuilt64(mut n: u8, x: u64) -> f64 { while n > 0 { n -= 1; } f64::from_bits(x) }\npub fn negative_zero(mut n: u8) -> u64 { while n > 0 { n -= 1; } (-0.0f64).to_bits() }\npub fn finite64(mut n: u8, x: f64) -> bool { while n > 0 { n -= 1; } x.is_finite() }\npub fn unmodeled(mut n: u8, x: f64) -> f64 { while n > 0 { n -= 1; } x.sin() }\npub mod shadow { #[allow(non_camel_case_types)] pub struct f64 { pub flag: bool } pub fn transport(mut n: u8, x: f64) -> f64 { while n > 0 { n -= 1; } x } }\n"),
    ]);
    let lock = std::process::Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dir)
        .output()
        .expect("lock");
    assert!(lock.status.success());
    let dumps = dumped(&dir, "native-workspace-floats");
    let mut opts = options("native_floats");
    opts.mir = Some(mir_options(
        dumps.clone(),
        Default::default(),
        uorust::mir::Trial::All,
    ));
    let files = uorust::project_workspace(&dir, &opts).expect("native float projection");
    let listing = text(&files, uorust::mir::CANDIDATES);
    for name in [
        "id32",
        "id64",
        "bits32",
        "arithmetic64",
        "rebuilt64",
        "negative_zero",
        "finite64",
        "shadow::transport",
    ] {
        assert!(
            listing.contains(&format!("native_floats::{name}\"")),
            "missing native float {name}: {listing}"
        );
    }
    assert!(!listing.contains("native_floats::unmodeled\""));
    let behavior = text(&files, "src/Behavior.lex.tex");
    let rows = frontier_rows(&files, "lifted");
    for function in ["native_floats::id32", "native_floats::id64"] {
        let definition = rows
            .iter()
            .find(|r| r["function"].as_str() == Some(function))
            .and_then(|r| r["definition"].as_str())
            .and_then(|d| d.strip_prefix("Behavior::"))
            .expect("primitive definition");
        let start = behavior
            .find(&format!("name: {definition},"))
            .expect("primitive header");
        let header = behavior[start..].split("body:").next().expect("header");
        assert!(
            header.contains("x : Nat") && header.contains("result: Nat"),
            "primitive was read as a workspace homonym: {header}"
        );
    }
    let proof = uorust::mir::prove_bounded(
        &dir,
        &options("native_floats"),
        &dumps,
        &dir.join("verdicts.toml"),
        &dir.join("trial"),
        8,
    )
    .expect("actual IEEE proofs");
    assert_eq!((proof.proved, proof.refused), (8, 0));
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-46: narrowing the compiler view must not destroy reusable SDK decisions.
#[test]
fn native_verdicts_survive_compiler_input_selections_ue_46() {
    let source = mir_fixture();
    let all = dumped(&source, "walk");
    let work = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("native-selection-history");
    let _ = std::fs::remove_dir_all(&work);
    let selected = work.join("selected");
    std::fs::create_dir_all(selected.join("lib-walk")).expect("selected input");
    for name in ["configuration.toml", "layouts.toml"] {
        std::fs::copy(all.join(name), selected.join(name)).expect("compiler metadata");
    }
    let mut count = 0;
    for entry in std::fs::read_dir(all.join("lib-walk")).expect("actual compiler bodies") {
        let path = entry.expect("entry").path();
        if path.extension().is_some_and(|x| x == "mir")
            && std::fs::read_to_string(&path)
                .expect("body")
                .contains("fn through_raw(")
        {
            std::fs::copy(
                &path,
                selected
                    .join("lib-walk")
                    .join(path.file_name().expect("name")),
            )
            .expect("exact compiler bytes");
            count += 1;
        }
    }
    assert_eq!(count, 1);
    let register = work.join("verdicts.toml");
    std::fs::copy(source.join("lift-verdicts.toml"), &register)
        .expect("actual committed SDK verdicts");
    let before = verdicts(&register);
    assert!(before
        .rows
        .values()
        .any(|r| matches!(r.status, uorust::mir::verdict::Decision::Refused(_))));
    for inputs in [&all, &selected, &all] {
        let replay = uorust::mir::prove_bounded(
            &source,
            &options("walk"),
            inputs,
            &register,
            &work.join("trial"),
            16,
        )
        .expect("cached selection replay");
        assert_eq!((replay.proved, replay.refused, replay.runs), (0, 0, 0));
        assert_eq!(
            verdicts(&register).rows,
            before.rows,
            "selection discarded reusable proof history"
        );
    }
    std::fs::remove_dir_all(work).expect("cleanup");
}

/// UE-45: transport does not inspect a hash builder or invent iteration order.
#[test]
fn native_hash_collections_reuse_unordered_carriers_ue_45() {
    let dir = scratch("native-hash-carriers", &[
        ("Cargo.toml", "[package]\nname = \"hash_carriers\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n"),
        ("src/lib.rs", "use std::collections::{HashMap, HashSet};\npub struct Seed { pub state: u64 }\npub fn map(mut n: u8, x: HashMap<u8, u16>) -> HashMap<u8, u16> { while n > 0 { n -= 1; } x }\npub fn set(mut n: u8, x: HashSet<u8>) -> HashSet<u8> { while n > 0 { n -= 1; } x }\npub fn custom_map(mut n: u8, x: HashMap<u8, u16, Seed>) -> HashMap<u8, u16, Seed> { while n > 0 { n -= 1; } x }\npub fn custom_set(mut n: u8, x: HashSet<u8, Seed>) -> HashSet<u8, Seed> { while n > 0 { n -= 1; } x }\npub fn inspect(mut n: u8, x: HashMap<u8, u16, Seed>) -> u64 { while n > 0 { n -= 1; } x.hasher().state }\npub fn ordered(mut n: u8, x: HashSet<u8>) -> Vec<u8> { while n > 0 { n -= 1; } x.into_iter().collect() }\npub fn unread(mut n: u8, x: HashMap<u8, std::cell::UnsafeCell<u8>, Seed>) -> HashMap<u8, std::cell::UnsafeCell<u8>, Seed> { while n > 0 { n -= 1; } x }\npub mod shadow { pub struct HashSet { pub flag: bool } pub fn transport(mut n: u8, x: HashSet) -> HashSet { while n > 0 { n -= 1; } x } }\n"),
    ]);
    let lock = std::process::Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dir)
        .output()
        .expect("lock source");
    assert!(lock.status.success());
    let dumps = dumped(&dir, "native-hash-carriers");
    let mut opts = options("hash_carriers");
    opts.mir = Some(mir_options(
        dumps.clone(),
        Default::default(),
        uorust::mir::Trial::All,
    ));
    let files = uorust::project_workspace(&dir, &opts).expect("collection projection");
    let listing = text(&files, uorust::mir::CANDIDATES);
    for name in [
        "map",
        "set",
        "custom_map",
        "custom_set",
        "shadow::transport",
    ] {
        assert!(
            listing.contains(&format!("hash_carriers::{name}\"")),
            "missing native collection {name}: {listing}"
        );
    }
    for name in ["inspect", "ordered", "unread"] {
        assert!(
            !listing.contains(&format!("hash_carriers::{name}\"")),
            "unmodeled observation was admitted: {name}"
        );
    }
    let proof = uorust::mir::prove_bounded(
        &dir,
        &options("hash_carriers"),
        &dumps,
        &dir.join("verdicts.toml"),
        &dir.join("trial"),
        5,
    )
    .expect("actual collection proofs");
    assert_eq!((proof.proved, proof.refused), (5, 0));
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-44: one changing state keeps loop invariants in the termination context.
#[test]
fn native_loop_state_preserves_invariants_in_evidence_ue_44() {
    let dir = scratch("loop-evidence-scope", &[
        ("Cargo.toml", "[package]\nname = \"loop_scope\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n"),
        ("src/lib.rs", "pub fn carried(mut n: u8, x: bool) -> bool { while n > 0 { n -= 1; } x }\n"),
    ]);
    let lock = std::process::Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dir)
        .output()
        .expect("lock");
    assert!(lock.status.success());
    let dumps = dumped(&dir, "loop-evidence-scope");
    let result = uorust::mir::prove_bounded(
        &dir,
        &options("loop_scope"),
        &dumps,
        &dir.join("verdicts.toml"),
        &dir.join("trial"),
        1,
    )
    .expect("SDK proof");
    assert_eq!((result.proved, result.refused), (1, 0));
    let behavior =
        std::fs::read_to_string(dir.join("trial/src/Behavior.lex.tex")).expect("actual trial");
    let start = behavior
        .find("name: decreases_")
        .expect("decrease evidence");
    let parameters = behavior[start..]
        .split("statement:")
        .next()
        .expect("evidence header");
    assert!(
        !parameters.contains(": Bool"),
        "invariant was left as a fixed argument: {parameters}"
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-43: the proof feedback loop checks only its closure and preserves decisions.
#[test]
fn native_verification_scopes_feedback_and_reuses_decisions_ue_43() {
    let (work, source, _) = mir_method_fixture("scoped-feedback");
    let path = source.join("src/lib.rs");
    let mut code = std::fs::read_to_string(&path).expect("source");
    code.push_str(
        "pub fn overflow(x: u8) -> u8 { let mut n = 1u8; while n > 0 { n -= 1; } x + 1 }\n",
    );
    std::fs::write(&path, code).expect("overflow case");
    let dumps = work.join("with-overflow");
    uorust::mir::dump(&source, &dumps, &Default::default()).expect("actual compiler");
    let verdict_path = work.join("verdicts.toml");
    let trial = work.join("trial");
    let summary = uorust::mir::prove_bounded(
        &source,
        &options("owners"),
        &dumps,
        &verdict_path,
        &trial,
        4,
    )
    .expect("actual SDK decisions");
    assert_eq!((summary.proved, summary.refused), (3, 1));
    let result: serde_json::Value = serde_json::from_slice(
        &std::fs::read(trial.join("uorust-proof-result.json"))
            .expect("machine-readable SDK feedback"),
    )
    .expect("SDK JSON");
    let modules = result["modules"].as_array().expect("verified modules");
    assert_eq!(
        modules.len(),
        2,
        "native trial verified unrelated modules: {result}"
    );
    let before = std::fs::read(&verdict_path).expect("actual verdicts");
    let replay = uorust::mir::prove_bounded(
        &source,
        &options("owners"),
        &dumps,
        &verdict_path,
        &trial,
        4,
    )
    .expect("unchanged replay");
    assert_eq!((replay.proved, replay.refused, replay.runs), (0, 0, 0));
    assert_eq!(
        std::fs::read(&verdict_path).expect("cached verdicts"),
        before
    );
    std::fs::remove_dir_all(work).expect("cleanup");
}

/// UE-42: native transport reads the same lock-admitted types as source transport.
#[test]
fn native_registry_carriers_require_locked_readings_ue_42() {
    let dir = scratch("native-registry-carriers", &[
        ("Cargo.toml", "[package]\nname = \"native_carriers\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n[dependencies]\nnum-bigint = \"=0.4.8\"\nnum-rational = \"=0.4.2\"\nserde_json = \"=1.0.114\"\nbitvec = \"=1.0.1\"\n"),
        ("src/lib.rs", "use num_bigint::BigInt;\nuse num_rational::BigRational;\nuse serde_json::{Value, Number};\nuse bitvec::vec::BitVec;\npub fn big(x: BigInt) -> BigInt { let mut n = 1u8; while n > 0 { n -= 1; } x }\npub fn ratio(x: BigRational) -> BigRational { let mut n = 1u8; while n > 0 { n -= 1; } x }\npub fn json(x: Value) -> Value { let mut n = 1u8; while n > 0 { n -= 1; } x }\npub fn number(x: Number) -> Number { let mut n = 1u8; while n > 0 { n -= 1; } x }\npub fn bits(x: BitVec) -> BitVec { let mut n = 1u8; while n > 0 { n -= 1; } x }\npub fn equal(a: Value, b: Value) -> bool { let mut n = 1u8; while n > 0 { n -= 1; } a == b }\n"),
    ]);
    let lock = std::process::Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dir)
        .output()
        .expect("lock native carriers");
    assert!(
        lock.status.success(),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
    let dumps = dumped(&dir, "native-registry-carriers");
    let mut opts = options("native_carriers");
    opts.mir = Some(mir_options(
        dumps.clone(),
        Default::default(),
        uorust::mir::Trial::All,
    ));
    let files = uorust::project_workspace(&dir, &opts).expect("native carriers");
    let listing = text(&files, uorust::mir::CANDIDATES);
    for name in ["big", "ratio", "json", "number", "bits"] {
        assert!(
            listing.contains(&format!("native_carriers::{name}\"")),
            "missing native carrier {name}: {listing}"
        );
    }
    assert!(
        !listing.contains("native_carriers::equal\""),
        "unmodeled JSON equality was admitted"
    );
    let out = dir.join("projection");
    uorust::write(&out, &files).expect("write native carriers");
    let checked = std::process::Command::new("lexlean")
        .args([
            "check",
            "src/Behavior.lex.tex",
            "--diagnostic-format",
            "json",
        ])
        .current_dir(&out)
        .output()
        .expect("SDK verifies native carriers");
    assert!(
        checked.status.success(),
        "{}{}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr)
    );
    let proof = uorust::mir::prove_bounded(
        &dir,
        &options("native_carriers"),
        &dumps,
        &dir.join("verdicts.toml"),
        &dir.join("proof"),
        5,
    )
    .expect("actual native carrier proofs");
    assert_eq!((proof.proved, proof.refused), (5, 0));
    let lock_path = dir.join("Cargo.lock");
    let original = std::fs::read_to_string(&lock_path).expect("actual lock");
    let checksum = "c89e69e7e0f03bea5ef08013795c25018e101932225a656383bd384495ecc367";
    assert!(original.contains(checksum));
    std::fs::write(&lock_path, original.replace(checksum, &"0".repeat(64)))
        .expect("wrong checksum");
    let wrong = uorust::project_workspace(&dir, &opts).expect("refused carrier projection");
    assert!(
        !text(&wrong, uorust::mir::CANDIDATES).contains("native_carriers::big\""),
        "wrong checksum admitted native BigInt"
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-41: source integer fields cannot silently become native bit patterns.
#[test]
fn native_aggregates_check_field_representations_ue_41() {
    let dir = scratch("aggregate-representations", &[
        ("Cargo.toml", "[package]\nname = \"aggregate_representations\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n"),
        ("src/lib.rs", "pub struct Signed { pub x: i32 }\npub enum Payload { Number(i32), Empty }\npub fn variant(n: i32) -> Payload { let mut count = 1u8; while count > 0 { count -= 1; } Payload::Number(n) }\npub fn array(s: Signed) -> [i32; 2] { let mut n = 1u8; while n > 0 { n -= 1; } [s.x, s.x] }\npub fn optional(s: Signed) -> Option<i32> { let mut n = 1u8; while n > 0 { n -= 1; } Some(s.x) }\npub fn unpack(s: Signed) -> (i32, u8, u8) { let mut n = 1u8; while n > 0 { n -= 1; } (s.x, n, 0) }\npub fn pack(n: i32) -> Signed { let mut count = 1u8; while count > 0 { count -= 1; } Signed { x: n } }\npub fn good(mut n: u8) -> (u8, u8, u8) { while n > 0 { n -= 1; } (n, 1, 2) }\n"),
    ]);
    let lock = std::process::Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dir)
        .output()
        .expect("lock source fixture");
    assert!(
        lock.status.success(),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
    let dumps = dumped(&dir, "aggregate-representations");
    let mut opts = options("aggregate_representations");
    opts.mir = Some(mir_options(
        dumps,
        Default::default(),
        uorust::mir::Trial::All,
    ));
    let files = uorust::project_workspace(&dir, &opts).expect("native aggregate projection");
    for name in ["unpack", "pack", "variant", "array", "optional"] {
        let detail = residue_detail(&files, &format!("aggregate_representations::{name}"))
            .unwrap_or_else(|| {
                panic!(
                    "incompatible aggregate was admitted: {name}: {}",
                    text(&files, "frontier.toml")
                )
            });
        assert!(detail.contains("field representation"), "{detail}");
    }
    let listing = text(&files, uorust::mir::CANDIDATES);
    assert!(
        listing.contains("aggregate_representations::good"),
        "{listing}"
    );
    let out = dir.join("projection");
    uorust::write(&out, &files).expect("write aggregates");
    let checked = std::process::Command::new("lexlean")
        .args([
            "check",
            "src/Behavior.lex.tex",
            "--diagnostic-format",
            "json",
        ])
        .current_dir(&out)
        .output()
        .expect("SDK type checker");
    assert!(
        checked.status.success(),
        "{}{}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr)
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}

/// UE-40: imported names are resolved, while function type parameters shadow them.
#[test]
fn renamed_unique_types_preserve_imports_and_generic_shadowing_ue_40() {
    let dir=scratch("renamed-unique-types", &[
        ("Cargo.toml","[package]\nname = \"renamed_types\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        ("src/lib.rs","pub mod data; pub mod holder;\n"),
        ("src/data.rs","pub struct Payload { pub value: u8 }\n"),
        ("src/holder.rs","use crate::data::Payload as Renamed;\nuse crate::data::Payload as T;\npub struct Held { pub payload: Renamed }\npub fn keep(x: Held) -> Held { x }\npub fn get(x: &Held) -> u8 { x.payload.value }\npub fn direct(x: Renamed) -> Renamed { x }\npub trait Read { fn read(&self) -> u8; }\npub fn hold_generic<T: Read>(x: &T) -> u8 { x.read() }\n"),
    ]);
    let files = project(&dir, "renamed_types");
    let lifted: Vec<String> = frontier_rows(&files, "lifted")
        .iter()
        .filter_map(|r| r["function"].as_str().map(str::to_string))
        .collect();
    for name in ["keep", "get", "direct", "hold_generic"] {
        assert!(
            lifted.contains(&format!("renamed_types::holder::{name}")),
            "renamed type function did not lift: {name}"
        );
    }
    let behavior = text(&files, "src/Behavior.lex.tex");
    let start = behavior
        .find("name: holdGeneric,")
        .expect("generic definition");
    let header = behavior[start..].split("body:").next().expect("header");
    assert!(
        header.contains("type_parameters: [T]") && header.contains("x : 'T"),
        "generic parameter was rebound to the imported concrete type: {header}"
    );
    let out = dir.join("projection");
    uorust::write(&out, &files).expect("write renamed types");
    let checked = std::process::Command::new("lexlean")
        .args([
            "verify",
            "src/Behavior.lex.tex",
            "--diagnostic-format",
            "json",
        ])
        .current_dir(&out)
        .output()
        .expect("SDK verifier");
    assert!(
        checked.status.success(),
        "{}{}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr)
    );
    std::fs::remove_dir_all(dir).expect("cleanup");
}
