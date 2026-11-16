//! The one error type a run of the engine can end in.
//!
//! An ambiguity in the workspace is not an error: admission records it as a
//! typed refusal (an open claim in `frontier.toml`) and the run continues. An
//! error is a condition under which no projection exists at all.

use std::fmt;
use std::path::{Path, PathBuf};

/// Why no projection was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UorustError {
    /// `cargo metadata` could not be run or did not succeed.
    CargoMetadata(String),
    /// A manifest, lockfile or metadata document did not parse.
    Malformed {
        /// The document.
        path: PathBuf,
        /// What the parser reported.
        reason: String,
    },
    /// A file could not be read or written.
    Io {
        /// The file.
        path: PathBuf,
        /// What the operating system reported.
        reason: String,
    },
    /// The control set needs a `lastModified` instant and none was given: no
    /// `--last-modified`, and the workspace is not a git checkout with a commit.
    NoTimestamp,
    /// A `--last-modified` value is not `YYYY-MM-DDThh:mm:ssZ`.
    BadTimestamp(String),
    /// The output directory holds files this engine did not write.
    ForeignOutput(PathBuf),
    /// `uorust check` found the committed projection differs from a fresh one.
    Stale(Vec<String>),
    /// A function of the incumbent lift register is re-decided against the
    /// register's law, by κ: one it records as lifted is decided unlifted,
    /// or, against a parent's register, one it records as stopped by a
    /// construct is counted under another while that construct still stops
    /// a function.
    Regressed(Vec<String>),
    /// A module of the projection is larger than the `max_file_bytes` the
    /// projection's own `lexlean.toml` states, so LexLean would refuse the
    /// project it is part of.
    ModuleTooLarge {
        /// The module's path in the projection.
        module: String,
        /// Its size in bytes.
        bytes: usize,
        /// The limit the projection states.
        limit: usize,
    },
    /// A tool the engine runs could not be run or failed: `cargo rustc`
    /// dumping MIR, or `lexlean verify` deciding the MIR path's theorems.
    Tool(String),
    /// The ratchet: more translator functions are uncertified than in the
    /// parent commit; the first line counts both, each other names a new
    /// uncertified function and its cause.
    Grown(Vec<String>),
    /// The differential: an output of the certified translator differs from
    /// the hand-written one's, by κ; each names the output and both κ.
    Disagree(Vec<String>),
}

impl fmt::Display for UorustError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CargoMetadata(reason) => write!(f, "cargo metadata failed: {reason}"),
            Self::Malformed { path, reason } => {
                write!(f, "{} does not parse: {reason}", path.display())
            }
            Self::Io { path, reason } => write!(f, "{}: {reason}", path.display()),
            Self::NoTimestamp => write!(
                f,
                "no lastModified instant: pass --last-modified YYYY-MM-DDThh:mm:ssZ or run \
                 inside a git checkout with a commit"
            ),
            Self::BadTimestamp(value) => {
                write!(
                    f,
                    "{value} is not an instant of the form YYYY-MM-DDThh:mm:ssZ"
                )
            }
            Self::ForeignOutput(path) => write!(
                f,
                "{} holds files uorust did not write; choose an empty or uorust-owned directory",
                path.display()
            ),
            Self::Stale(paths) => write!(
                f,
                "the projection is stale; these files differ: {}",
                paths.join(", ")
            ),
            Self::Regressed(rows) => write!(
                f,
                "a function is re-decided: a lifted κ stays lifted, and against a parent's register \
                 an unchanged κ stays under its construct while that construct stops a function: \
                 {}",
                rows.join("; ")
            ),
            Self::ModuleTooLarge {
                module,
                bytes,
                limit,
            } => write!(
                f,
                "the module {module} is {bytes} bytes, beyond the max_file_bytes of {limit} the \
                 projection states in lexlean.toml"
            ),
            Self::Tool(reason) => write!(f, "{reason}"),
            Self::Disagree(rows) => write!(
                f,
                "the certified and the hand-written translator disagree: {}",
                rows.join("; ")
            ),
            Self::Grown(rows) => write!(
                f,
                "the hand-written translator may only shrink: {}",
                rows.join("\n  ")
            ),
        }
    }
}

impl std::error::Error for UorustError {}

pub(crate) fn io(path: &Path, error: std::io::Error) -> UorustError {
    UorustError::Io {
        path: path.to_path_buf(),
        reason: error.to_string(),
    }
}
