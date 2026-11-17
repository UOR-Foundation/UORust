//! UE-22: parameter macros preserve the surface lift and its source identity.
use std::path::PathBuf;
use uorust::{register::Status, Options};

#[test]
fn numeric_parameter_macros_lift_without_rewriting_the_sources_ue_22() {
    let root: PathBuf = std::env::temp_dir().join(format!("uorust-numeric-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let source = "const WIDTH: u8 = ::p::width!();\npub fn direct() -> u8 { ::p::width!() }\npub fn indirect() -> u8 { WIDTH }\npub fn unknown() -> u8 { ::p::unknown!() }\npub fn arguments() -> u8 { ::p::width!(9) }\npub fn conditional() -> u8 { ::p::conditional!() }\npub fn multiple() -> u8 { ::p::multiple!() }\n";
    for (path, text) in [
        ("Cargo.toml", "[workspace]\nmembers = [\"calc\", \"parameters\"]\nresolver = \"2\"\n"),
        ("calc/Cargo.toml", "[package]\nname = \"calc\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[dependencies]\np = { package = \"parameters\", path = \"../parameters\" }\n"),
        ("parameters/Cargo.toml", "[package]\nname = \"parameters\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        ("parameters/src/lib.rs", "#[macro_export]\nmacro_rules! width { () => { 8 }; }\n#[cfg(feature = \"alternate\")]\n#[macro_export]\nmacro_rules! conditional { () => { 9 }; }\n#[macro_export]\nmacro_rules! multiple { () => { 8 }; ($x:expr) => { $x }; }\n"),
        ("calc/src/lib.rs", source),
    ] {
        let to = root.join(path);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::write(to, text).unwrap();
    }
    let projected = uorust::project_workspace(
        &root,
        &Options {
            name: "numeric".into(),
            last_modified: "2026-11-11T00:00:00Z".into(),
            advisories: None,
            mir: None,
            limits: None,
            embedding: None,
        },
    )
    .expect("projects");
    let register = uorust::lift_register(&projected).expect("register");
    for name in [
        "direct",
        "indirect",
        "unknown",
        "arguments",
        "conditional",
        "multiple",
    ] {
        let row = register
            .rows
            .values()
            .find(|r| r.function == format!("calc::{name}"))
            .expect("row");
        assert_eq!(
            matches!(row.status, Status::Lifted(_)),
            matches!(name, "direct" | "indirect"),
            "{name}: {:?}",
            row.status
        );
    }
    assert_eq!(
        std::fs::read_to_string(root.join("calc/src/lib.rs")).unwrap(),
        source
    );
    std::fs::remove_dir_all(root).unwrap();
}
