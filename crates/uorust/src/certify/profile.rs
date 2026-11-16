//! What an export is of: the functions it certifies, where it starts, how
//! its packages are named, and what `lexlean production` is asked for.
//!
//! The translator's own export is the default ([`Profile::default`]): the
//! scope, the entry, the package prefix, the effects and the target the
//! module's constants name. Any other workspace has its own, from three
//! places, each over the one before: what its Cargo manifest and its
//! directory give ([`Profile::of_workspace`]), the table it declares in that
//! manifest (`[workspace.metadata.uorust.certify]`, or `[package.metadata..]`
//! of a manifest without a workspace), and the command's options
//! ([`Options`]). An export records its profile beside what it keeps, and a
//! manifest states the profile it was taken in under, so the steps after the
//! export never ask again.

use std::path::Path;

use super::{EFFECTS, ENTRY, PACKAGE, SCOPE, TARGET};
use crate::error::io;
use crate::UorustError;

/// The file of an export's kept output that records its profile.
pub const PROFILE: &str = "profile.toml";

/// The table of a workspace's Cargo manifest that declares its profile.
pub const TABLE: &str = "metadata.uorust.certify";

/// What every package prefix a workspace does not declare ends with.
const SUFFIX: &str = "_certified_";

/// What an export's roots are.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    /// The lifted functions in scope, each by its definition.
    #[default]
    Functions,
    /// The closed assertions of the workspace's own tests over them, each
    /// the Boolean definition the projection states it as.
    Cases,
}

impl Kind {
    /// The word a table states it by.
    pub fn word(self) -> &'static str {
        match self {
            Kind::Functions => "functions",
            Kind::Cases => "cases",
        }
    }
}

/// The profile of an export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    /// The crate-path prefixes of the functions measured; the empty prefix
    /// is every function of the register.
    pub scope: Vec<String>,
    /// The function the share's reachability starts at, when one is named.
    pub entry: Option<String>,
    /// The prefix of every certified package's name.
    pub package: String,
    /// The effects every root admits.
    pub effects: Vec<String>,
    /// The target every root is exported to.
    pub target: String,
    /// What the roots are.
    pub kind: Kind,
}

impl Default for Profile {
    /// The translator's own: the module's constants.
    fn default() -> Self {
        Profile {
            scope: SCOPE.iter().map(|s| (*s).to_string()).collect(),
            entry: Some(ENTRY.to_string()),
            package: PACKAGE.to_string(),
            effects: EFFECTS.iter().map(|s| (*s).to_string()).collect(),
            target: TARGET.to_string(),
            kind: Kind::Functions,
        }
    }
}

/// What a command's options say of the profile; each one given replaces
/// what the workspace says.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// The workspace whose manifest is read.
    pub workspace: Option<std::path::PathBuf>,
    /// The scope's prefixes.
    pub scope: Vec<String>,
    /// The entry.
    pub entry: Option<String>,
    /// The package prefix.
    pub package: Option<String>,
    /// The effects.
    pub effects: Vec<String>,
    /// The target.
    pub target: Option<String>,
    /// The roots are the cases.
    pub cases: bool,
}

impl Profile {
    /// Whether a function is measured.
    pub fn in_scope(&self, function: &str) -> bool {
        self.scope.iter().any(|s| function.starts_with(s.as_str()))
    }

    /// The profile a command runs under: the translator's own, or the
    /// workspace's when one is named, then every option given.
    pub fn resolve(options: &Options) -> Result<Profile, UorustError> {
        let mut profile = match &options.workspace {
            Some(workspace) => Profile::of_workspace(workspace)?,
            None => Profile::default(),
        };
        if !options.scope.is_empty() {
            profile.scope.clone_from(&options.scope);
        }
        if let Some(entry) = &options.entry {
            profile.entry = Some(entry.clone()).filter(|e| !e.is_empty());
        }
        if let Some(package) = &options.package {
            profile.package.clone_from(package);
        }
        if !options.effects.is_empty() {
            profile.effects.clone_from(&options.effects);
        }
        if let Some(target) = &options.target {
            profile.target.clone_from(target);
        }
        if options.cases {
            profile.kind = Kind::Cases;
        }
        profile.checked(Path::new("the command's options"))
    }

    /// The profile of the workspace at `workspace`: every function of its
    /// register in scope, no entry, packages named after its directory, every
    /// effect of the closed registry and the `std` target; then what the
    /// table of its Cargo manifest declares.
    pub fn of_workspace(workspace: &Path) -> Result<Profile, UorustError> {
        let manifest = workspace.join("Cargo.toml");
        let text = std::fs::read_to_string(&manifest).map_err(|e| io(&manifest, e))?;
        let doc: toml::Value = text.parse().map_err(|e| UorustError::Malformed {
            path: manifest.clone(),
            reason: format!("{e}"),
        })?;
        let derived = Profile {
            scope: vec![String::new()],
            entry: None,
            package: crate_name(&crate::default_name(workspace)?) + SUFFIX,
            ..Profile::default()
        };
        let declared = ["workspace", "package"].into_iter().find_map(|owner| {
            TABLE
                .split('.')
                .try_fold(doc.get(owner)?, |at, key| at.get(key))
        });
        match declared {
            Some(table) => Profile::parse(&manifest, table, derived),
            None => Ok(derived),
        }
    }

    /// The profile a table states over `base`: each key it holds replaces
    /// the base's. A key of another type, an effect outside the closed
    /// registry, a package prefix that is no crate name, a key the profile
    /// does not have: [`UorustError::Malformed`] at `path`.
    pub fn parse(path: &Path, table: &toml::Value, base: Profile) -> Result<Profile, UorustError> {
        let bad = |reason: String| UorustError::Malformed {
            path: path.to_path_buf(),
            reason,
        };
        let rows = table
            .as_table()
            .ok_or_else(|| bad(format!("{TABLE} is not a table")))?;
        let text = |key: &str, v: &toml::Value| {
            v.as_str()
                .map(str::to_string)
                .ok_or_else(|| bad(format!("{key} is not a string")))
        };
        let list = |key: &str, v: &toml::Value| {
            v.as_array()
                .ok_or_else(|| bad(format!("{key} is not a list")))?
                .iter()
                .map(|item| text(key, item))
                .collect::<Result<Vec<_>, UorustError>>()
        };
        let mut profile = base;
        for (key, value) in rows {
            match key.as_str() {
                "scope" => profile.scope = list(key, value)?,
                "entry" => profile.entry = Some(text(key, value)?).filter(|e| !e.is_empty()),
                "package" => profile.package = text(key, value)?,
                "effects" => profile.effects = list(key, value)?,
                "target" => profile.target = text(key, value)?,
                "roots" => {
                    let word = text(key, value)?;
                    profile.kind = [Kind::Functions, Kind::Cases]
                        .into_iter()
                        .find(|k| k.word() == word)
                        .ok_or_else(|| bad(format!("roots = {word:?}: functions or cases")))?;
                }
                other => return Err(bad(format!("{TABLE} has no key {other}"))),
            }
        }
        profile.checked(path)
    }

    /// The profile, when it is one an export can run under.
    fn checked(self, path: &Path) -> Result<Profile, UorustError> {
        let bad = |reason: String| UorustError::Malformed {
            path: path.to_path_buf(),
            reason,
        };
        if self.scope.is_empty() {
            return Err(bad("the scope names no prefix".to_string()));
        }
        if let Some(effect) = self.effects.iter().find(|e| !EFFECTS.contains(&e.as_str())) {
            return Err(bad(format!(
                "{effect} is no effect of the closed registry ({})",
                EFFECTS.join(", ")
            )));
        }
        let named = self.package.starts_with(|c: char| c.is_ascii_lowercase());
        if !named || crate_name(&self.package) != self.package {
            return Err(bad(format!(
                "the package prefix {:?} is not a crate name's",
                self.package
            )));
        }
        if self.target.is_empty() {
            return Err(bad("no target".to_string()));
        }
        Ok(self)
    }

    /// The profile as a table's rows, every key stated: what [`Profile::parse`]
    /// reads back whatever its base.
    pub fn rows(&self) -> String {
        let list = |items: &[String]| {
            let quoted: Vec<String> = items.iter().map(|i| quoted(i)).collect();
            format!("[{}]", quoted.join(", "))
        };
        format!(
            "scope = {}\nentry = {}\npackage = {}\neffects = {}\ntarget = {}\nroots = {}\n",
            list(&self.scope),
            quoted(self.entry.as_deref().unwrap_or_default()),
            quoted(&self.package),
            list(&self.effects),
            quoted(&self.target),
            quoted(self.kind.word())
        )
    }

    /// The profile an export recorded in its kept output at `kept`; the
    /// translator's own when it recorded none.
    pub fn kept(kept: &Path) -> Result<Profile, UorustError> {
        let path = kept.join(PROFILE);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Profile::default()),
            Err(e) => return Err(io(&path, e)),
        };
        let doc: toml::Value = text.parse().map_err(|e| UorustError::Malformed {
            path: path.clone(),
            reason: format!("{e}"),
        })?;
        Profile::parse(&path, &doc, Profile::default())
    }

    /// The profile a manifest's text states it was taken in under: its
    /// `[profile]` table, or the translator's own when it has none.
    pub fn recorded(path: &Path, text: &str) -> Result<Profile, UorustError> {
        let doc: toml::Value = text.parse().map_err(|e| UorustError::Malformed {
            path: path.to_path_buf(),
            reason: format!("{e}"),
        })?;
        match doc.get("profile") {
            Some(table) => Profile::parse(path, table, Profile::default()),
            None => Ok(Profile::default()),
        }
    }

    /// [`Profile::recorded`] of the manifest file at `path`; an absent file
    /// states none.
    pub fn recorded_at(path: &Path) -> Result<Option<Profile>, UorustError> {
        match std::fs::read_to_string(path) {
            Ok(text) => Profile::recorded(path, &text).map(Some),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(io(path, e)),
        }
    }

    /// The prefixes of `self`'s scope that `head`'s scope does not cover: a
    /// scope a head measures under may only widen.
    pub fn dropped_by(&self, head: &Profile) -> Vec<String> {
        self.scope
            .iter()
            .filter(|prefix| !head.scope.iter().any(|h| prefix.starts_with(h.as_str())))
            .cloned()
            .collect()
    }
}

fn quoted(text: &str) -> String {
    toml::Value::String(text.to_string()).to_string()
}

/// A name as a crate's: lower case, every character outside a crate name's
/// an underscore.
pub(crate) fn crate_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}
