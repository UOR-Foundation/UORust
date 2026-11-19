//! Same-compilation identities for actual compiler MIR dependencies.
use crate::error::io;
use crate::UorustError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Definition {
    pub(crate) mir: String,
    pub(crate) body_sha256: String,
    pub(crate) identity: String,
    pub(crate) parametric: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalogue {
    spec: String,
    toolchain: String,
    source_sha256: String,
    configuration_sha256: String,
    definition: Vec<Definition>,
}

pub(crate) fn sidecar(dumps: &Path) -> PathBuf {
    dumps.with_extension("compiler-definitions.toml")
}

fn bad(message: &str) -> UorustError {
    UorustError::Tool(format!("compiler definitions: {message}"))
}

fn graph_identity(text: &str) -> Option<(String, bool)> {
    let header = text
        .lines()
        .find_map(|line| line.trim().strip_prefix("label=<fn "))?;
    let header = header.split("<br ").next()?;
    let header = header
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&");
    let mut depth = 0usize;
    let mut prior = ' ';
    for (at, ch) in header.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' if prior != '-' => depth = depth.checked_sub(1)?,
            '(' if depth == 0 => {
                let identity = header[..at].trim().to_string();
                if !super::identity::qualified(&identity) {
                    return None;
                }
                let (_, owner, reference) = super::identity::parts(&identity)?;
                let parametric =
                    text.contains("/#") || owner.contains('<') || reference.contains('<');
                return Some((identity, parametric));
            }
            _ => {}
        }
        prior = ch;
    }
    None
}

fn target_crate(target: &str) -> Result<String, UorustError> {
    target
        .strip_prefix("lib-")
        .or_else(|| target.strip_prefix("bin-"))
        .filter(|s| !s.is_empty())
        .map(|s| s.replace('-', "_"))
        .ok_or_else(|| bad("unrecognized MIR target"))
}

/// Graph labels can name the parent of a constant. Only actual MIR source
/// definitions supply a method identity, using the common compiler classifier.
fn definition_key(krate: &str, identity: &str, text: &str) -> Option<crate::embedding::Key> {
    let name = super::identity::parts(identity)?.0;
    let mut bodies = super::parse::bodies(text)
        .into_iter()
        .filter_map(|body| crate::embedding::body_key(krate, &body.path))
        .filter(|key| key.name == name && key.impl_at.is_some());
    let key = bodies.next()?;
    bodies.next().is_none().then_some(key)
}

pub(crate) fn collect(
    target: &Path,
    dumps: &Path,
    records: &mut Vec<Definition>,
) -> Result<(), UorustError> {
    let krate = target_crate(
        target
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| bad("missing MIR target"))?,
    )?;
    let mut files: Vec<_> = std::fs::read_dir(target)
        .map_err(|e| io(target, e))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .collect();
    files.sort();
    for file in files {
        let Some(name) = file
            .file_name()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_suffix(".ElaborateBoxDerefs.before.dot"))
        else {
            continue;
        };
        let text = std::fs::read_to_string(&file).map_err(|e| io(&file, e))?;
        let Some((identity, parametric)) = graph_identity(&text) else {
            continue;
        };
        let mir = target.join(format!("{name}{}", super::DUMPED));
        let bytes = std::fs::read(&mir).map_err(|e| io(&mir, e))?;
        if definition_key(&krate, &identity, &String::from_utf8_lossy(&bytes)).is_none() {
            continue;
        }
        let relative = mir
            .strip_prefix(dumps)
            .map_err(|_| bad("MIR outside its capture root"))?;
        records.push(Definition {
            mir: relative.to_string_lossy().into_owned(),
            body_sha256: crate::project::sha(&bytes),
            identity,
            parametric,
        });
    }
    Ok(())
}

pub(crate) fn write(
    root: &Path,
    dumps: &Path,
    definitions: Vec<Definition>,
) -> Result<(), UorustError> {
    let configuration = dumps.join(crate::shard::CONFIGURATION);
    let configuration = std::fs::read(&configuration).map_err(|e| io(&configuration, e))?;
    let catalogue = Catalogue {
        spec: "uorust/compiler-definitions/1".into(),
        toolchain: super::TOOLCHAIN.into(),
        source_sha256: crate::shard::source_digest(root)?,
        configuration_sha256: crate::project::sha(&configuration),
        definition: definitions,
    };
    let file = sidecar(dumps);
    let text = toml::to_string(&catalogue).map_err(|e| bad(&e.to_string()))?;
    std::fs::write(&file, text).map_err(|e| io(&file, e))
}

pub(crate) fn load(
    root: &Path,
    dumps: &Path,
) -> Result<BTreeMap<crate::embedding::Key, Definition>, UorustError> {
    let file = sidecar(dumps);
    if !file.exists() {
        return Ok(BTreeMap::new());
    }
    let text = std::fs::read_to_string(&file).map_err(|e| io(&file, e))?;
    let catalogue: Catalogue = toml::from_str(&text).map_err(|e| bad(&e.to_string()))?;
    let config = dumps.join(crate::shard::CONFIGURATION);
    let bytes = std::fs::read(&config).map_err(|e| io(&config, e))?;
    if catalogue.spec != "uorust/compiler-definitions/1"
        || catalogue.toolchain != super::TOOLCHAIN
        || catalogue.source_sha256 != crate::shard::source_digest(root)?
        || catalogue.configuration_sha256 != crate::project::sha(&bytes)
    {
        return Err(bad(
            "source, configuration or pinned compiler binding mismatch",
        ));
    }
    let mut result = BTreeMap::new();
    let mut ambiguous = std::collections::BTreeSet::new();
    for record in catalogue.definition {
        let path = Path::new(&record.mir);
        if path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(bad("non-relative MIR identity"));
        }
        let target = path
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
            .ok_or_else(|| bad("missing target"))?;
        let krate = target_crate(target)?;
        let path = dumps.join(path);
        let bytes = std::fs::read(&path).map_err(|e| io(&path, e))?;
        if crate::project::sha(&bytes) != record.body_sha256 {
            return Err(bad("MIR body binding mismatch"));
        }
        let key = definition_key(&krate, &record.identity, &String::from_utf8_lossy(&bytes))
            .ok_or_else(|| {
                bad(&format!(
                    "identity does not name one compiler body: {}",
                    record.mir
                ))
            })?;
        if result.insert(key.clone(), record).is_some() {
            ambiguous.insert(key.clone());
        }
    }
    for key in ambiguous {
        result.remove(&key);
    }
    Ok(result)
}

/// Add companion identities without changing any saved MIR or its producer.
pub fn augment(root: &Path, dumps: &Path) -> Result<(), UorustError> {
    let config_path = dumps.join(crate::shard::CONFIGURATION);
    let text = std::fs::read_to_string(&config_path).map_err(|e| io(&config_path, e))?;
    let doc: toml::Value = toml::from_str(&text).map_err(|e| bad(&e.to_string()))?;
    let config = crate::shard::Configuration {
        features: doc
            .get("features")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_str)
            .map(str::to_string)
            .collect(),
        no_default_features: !doc
            .get("default_features")
            .and_then(toml::Value::as_bool)
            .unwrap_or(true),
        all_features: doc
            .get("all_features")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false),
        target: doc
            .get("target")
            .and_then(toml::Value::as_str)
            .map(str::to_string),
    };
    let temp =
        std::env::temp_dir().join(format!("uorust-definition-capture-{}", std::process::id()));
    if temp.exists() {
        return Err(bad("capture scratch already exists"));
    }
    let outcome = (|| {
        super::dump_inner(root, &temp, &config, true)?;
        let held = sidecar(&temp);
        let text = std::fs::read_to_string(&held).map_err(|e| io(&held, e))?;
        let mut catalogue: Catalogue = toml::from_str(&text).map_err(|e| bad(&e.to_string()))?;
        for record in &catalogue.definition {
            let path = dumps.join(&record.mir);
            let bytes = std::fs::read(&path).map_err(|e| io(&path, e))?;
            if crate::project::sha(&bytes) != record.body_sha256 {
                return Err(bad(
                    "captured body differs from saved MIR; refresh its exact compiler inputs",
                ));
            }
        }
        catalogue.configuration_sha256 = crate::project::sha(text_bytes(&config_path)?.as_slice());
        let output = sidecar(dumps);
        let pending = output.with_extension("pending");
        std::fs::write(
            &pending,
            toml::to_string(&catalogue).map_err(|e| bad(&e.to_string()))?,
        )
        .map_err(|e| io(&pending, e))?;
        std::fs::rename(&pending, &output).map_err(|e| io(&output, e))
    })();
    if temp.exists() {
        std::fs::remove_dir_all(&temp).map_err(|e| io(&temp, e))?;
    }
    let held = sidecar(&temp);
    if held.exists() {
        std::fs::remove_file(&held).map_err(|e| io(&held, e))?;
    }
    outcome
}

fn text_bytes(path: &Path) -> Result<Vec<u8>, UorustError> {
    std::fs::read(path).map_err(|e| io(path, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_parent_labels_cannot_promote_constants_to_methods_ue_85() {
        let root =
            std::env::temp_dir().join(format!("uorust-definition-kinds-{}", std::process::id()));
        let target = root.join("lib-neutral");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(root.join("lib.rs"), "#[derive(Debug,Default)] pub struct Mark {pub a:bool,pub b:bool,pub c:bool,pub d:bool,pub e:bool,pub f:bool}\n").unwrap();
        let output = std::process::Command::new("rustc")
            .args([
                "lib.rs",
                "--crate-type=lib",
                "--crate-name=neutral",
                "--edition=2021",
            ])
            .args(super::super::FLAGS)
            .arg(format!("-Zdump-mir-dir={}", target.display()))
            .env("RUSTC_BOOTSTRAP", "1")
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            std::fs::read_dir(&target)
                .unwrap()
                .flatten()
                .any(|e| e.file_name().to_string_lossy().contains("promoted")),
            "compiler fixture must contain an actual promoted body"
        );
        let mut records = Vec::new();
        collect(&target, &root, &mut records).unwrap();
        assert_eq!(
            records.len(),
            2,
            "parent graph labels imported constants as method definitions"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn graph_definitions_preserve_qualified_identity_and_open_parameters_ue_85() {
        let closed = "label=<fn &lt;nested::Value as std::clone::Clone&gt;::clone(_1: &amp;'{erased} nested::Value) -&gt; nested::Value<br align=\"left\"/>";
        assert_eq!(
            graph_identity(closed),
            Some(("<nested::Value as std::clone::Clone>::clone".into(), false))
        );
        let generic = "label=<fn &lt;Boxed&lt;U&gt; as std::clone::Clone&gt;::clone(_1: &amp;Boxed&lt;U/#0&gt;) -&gt; Boxed&lt;U/#0&gt;<br />";
        assert!(graph_identity(generic).unwrap().1);
        assert!(graph_identity("label=<fn clone(_1: bool) -&gt; bool<br />").is_none());
    }

    #[test]
    fn definition_catalogue_rejects_changed_body_source_configuration_and_paths_ue_85() {
        let root =
            std::env::temp_dir().join(format!("uorust-definition-bindings-{}", std::process::id()));
        let dumps = root.join("dumps");
        std::fs::create_dir_all(dumps.join("lib-neutral")).unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname=\"neutral\"\nversion=\"0.1.0\"\n",
        )
        .unwrap();
        let source = root.join("src/lib.rs");
        std::fs::write(
            &source,
            "#[derive(Default)]\npub struct Mark {pub flag:bool}\n",
        )
        .unwrap();
        let config = dumps.join(crate::shard::CONFIGURATION);
        std::fs::write(&config, "target=\"x86_64-unknown-linux-gnu\"\n").unwrap();
        let body = "fn <impl at src/lib.rs:1:10: 1:17>::default() -> Mark {\n    let mut _0: Mark;\n    bb0: {\n        _0 = Mark { flag: const false };\n        return;\n    }\n}\n";
        let mir = format!("lib-neutral/derived{}", super::super::DUMPED);
        std::fs::write(dumps.join(&mir), body).unwrap();
        let record = Definition {
            mir: mir.clone(),
            body_sha256: crate::project::sha(body.as_bytes()),
            identity: "<Mark as std::default::Default>::default".into(),
            parametric: false,
        };
        write(&root, &dumps, vec![record.clone()]).unwrap();
        assert_eq!(load(&root, &dumps).unwrap().len(), 1);
        std::fs::write(dumps.join(&mir), body.replace("false", "true")).unwrap();
        assert!(load(&root, &dumps)
            .unwrap_err()
            .to_string()
            .contains("body binding mismatch"));
        std::fs::write(dumps.join(&mir), body).unwrap();
        std::fs::write(&source, "pub struct Mark;\n").unwrap();
        assert!(load(&root, &dumps)
            .unwrap_err()
            .to_string()
            .contains("binding mismatch"));
        std::fs::write(
            &source,
            "#[derive(Default)]\npub struct Mark {pub flag:bool}\n",
        )
        .unwrap();
        std::fs::write(&config, "target=\"wasm32-unknown-unknown\"\n").unwrap();
        assert!(load(&root, &dumps)
            .unwrap_err()
            .to_string()
            .contains("binding mismatch"));
        write(&root, &dumps, vec![record.clone(), record.clone()]).unwrap();
        assert!(load(&root, &dumps).unwrap().is_empty());
        let mut escaped = record;
        escaped.mir = "../outside.mir".into();
        write(&root, &dumps, vec![escaped]).unwrap();
        assert!(load(&root, &dumps)
            .unwrap_err()
            .to_string()
            .contains("non-relative"));
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_file(sidecar(&dumps)).ok();
    }
}
