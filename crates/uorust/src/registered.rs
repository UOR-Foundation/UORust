//! Types of registry crates the lift reads, each as the crate's own source
//! defines it at the locked versions its reading was taken from.
//!
//! A registry crate is not assumed: a reading names the package, the path
//! of the type, and every version, with the checksum Cargo records for its
//! archive, whose source the reading was read from. A package of the
//! workspace reads the type only where it depends on that package under
//! its own name and `Cargo.lock` pins exactly one version of it, one the
//! reading records with the same checksum. Elsewhere the type is what it
//! was: a type of a crate with no LexLean model.

use std::collections::BTreeMap;

use crate::facts::{LockedPackage, Package};
use crate::lift::Ty;

/// The registry index whose packages a reading's checksums are of.
const CRATES_IO: &str = "registry+https://github.com/rust-lang/crates.io-index";

/// A registry crate's type and what the lift reads it as.
pub(crate) struct Reading {
    /// The package, as `Cargo.lock` names it.
    pub(crate) package: &'static str,
    /// The type's path from the crate's root, as the crate exports it.
    pub(crate) path: &'static str,
    /// The versions read, each with the SHA-256 of its archive.
    pub(crate) locked: &'static [(&'static str, &'static str)],
    /// The LexLean type the lift reads it as.
    pub(crate) ty: Ty,
    /// The other registry packages the type is made of, whose locked
    /// versions their own readings must record: the reading of a type
    /// holds only where those of what it holds do.
    pub(crate) within: &'static [&'static str],
}

/// Every reading.
///
/// `num_bigint::BigInt` (num-bigint 0.4.8, `src/bigint.rs`) is the struct
/// of a private `sign` and a private magnitude `data`, a `BigUint` whose
/// digits are kept without high zeros. Every constructor the crate exports
/// (`from_biguint`, `new`, the `From` conversions, the arithmetic) ends in
/// a value whose sign is `NoSign` exactly when the magnitude is zero, so a
/// value stands for exactly one mathematical integer and each integer for
/// exactly one value. `PartialEq`, `Ord` and `Hash` compare and hash the
/// sign and magnitude, and `Clone` copies them: equal values are equal
/// integers and the order is the integers' order. Its `+`, `-`, `*` and
/// unary `-` (`src/bigint/{addition,subtraction,multiplication}.rs`,
/// `impl Neg`) are exact, allocating as the value grows, and `/` and `%`
/// are `Integer::div_rem`'s quotient truncated toward zero and remainder of
/// the dividend's sign; `Default` is zero. The orphan rule leaves those
/// operator implementations between two values of the type to the crate,
/// so no workspace overrides them. LexLean's `Int` is the mathematical
/// integers with exactly those operations (its `quotient` and `remainder`
/// are `Int.tdiv` and `Int.tmod`), so a `BigInt` is held as an `Int`.
///
/// `num_rational::BigRational` (num-rational 0.4.2, `src/lib.rs`) is the
/// alias of `Ratio<BigInt>`, the struct of a private `numer` and `denom`
/// that derives `Clone` and nothing that compares. `new_raw` stores any
/// pair as given, and the rest of the crate keeps a stored pair until an
/// operation reduces it, so two values of one rational may store two pairs:
/// the crate's `PartialEq`, `Ord` and `Hash` compare values, not pairs.
/// It is held as the pair it stores, `(Int * Int)` of its two `BigInt`s,
/// which every accessor (`numer`, `denom`) reads exactly; the lift reads no
/// comparison of it (see [`Ty::Ratio`]).
pub(crate) fn readings() -> Vec<Reading> {
    vec![
        Reading {
            package: "num-bigint",
            path: "num_bigint::BigInt",
            locked: &[(
                "0.4.8",
                "c89e69e7e0f03bea5ef08013795c25018e101932225a656383bd384495ecc367",
            )],
            ty: Ty::Unbounded,
            within: &[],
        },
        Reading {
            package: "num-rational",
            path: "num_rational::BigRational",
            locked: &[(
                "0.4.2",
                "f83d14da390562dca69fc84082e73e548e1ad308d24accdedd2720017cb37824",
            )],
            ty: Ty::Ratio,
            within: &["num-bigint"],
        },
    ]
}

/// Whether the lock pins exactly one version of a package, one a reading
/// of it records with the same checksum.
fn pinned_as_read(package: &str, lock: &[LockedPackage]) -> bool {
    let pinned: Vec<&LockedPackage> = lock.iter().filter(|l| l.name == package).collect();
    match pinned.as_slice() {
        [one] => {
            one.source == CRATES_IO
                && readings().iter().any(|r| {
                    r.package == package
                        && r.locked
                            .iter()
                            .any(|(v, sum)| *v == one.version && *sum == one.checksum)
                })
        }
        _ => false,
    }
}

/// The type a reading of `path` gives, whatever the package: the lift
/// reads it only where [`of_package`] admitted the path into its scope.
pub(crate) fn reading(path: &str) -> Option<Ty> {
    readings()
        .into_iter()
        .find(|r| r.path == path)
        .map(|r| r.ty)
}

/// The paths of the readings that hold for a package: it depends on the
/// reading's package by that package's own name, as a normal dependency on
/// every platform (a build or dev dependency is not in the library's crate
/// graph, and another normal dependency whose library takes the name then
/// is what the name means; two of them the compiler refuses, E0464), no
/// other dependency is renamed to the crate's name, and the lock pins one
/// version of it, and of each package the type is made of, that a reading
/// records with the same checksum. A checksum identifies the source only
/// where Cargo builds from the registry's archive: where a configuration
/// file replaces the source or overrides a path (`replaced`), Cargo builds
/// what that names with the lock unchanged, and no reading holds.
pub(crate) fn of_package(
    package: &Package,
    lock: &[LockedPackage],
    replaced: &[String],
) -> Vec<String> {
    if !replaced.is_empty() {
        return Vec::new();
    }
    readings()
        .into_iter()
        .filter(|r| {
            let krate = r.package.replace('-', "_");
            let depends = package.deps.iter().any(|d| {
                d.name == r.package
                    && d.path.is_none()
                    && d.rename.is_none()
                    && d.kind == "normal"
                    && d.platform.is_none()
            });
            let shadowed = package
                .deps
                .iter()
                .any(|d| d.rename.as_deref().map(|n| n.replace('-', "_")) == Some(krate.clone()));
            let read = std::iter::once(&r.package)
                .chain(r.within)
                .all(|p| pinned_as_read(p, lock));
            depends && !shadowed && read
        })
        .map(|r| r.path.to_string())
        .collect()
}

/// The bare names of a file the lift reads as a read path: those the name
/// resolver binds to it in every module of the file
/// (`names::Resolver::registry_bindings`, the one reading of a `use`), less
/// any name a type of the package declares, since the lift's own tables
/// read a declared type's bare name across the package. A package that
/// invokes a macro in item position may declare a type the facts do not
/// hold, so none of its names is read.
pub(crate) fn bindings(
    package: &Package,
    mut bound: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    if package.files.iter().any(|f| f.item_macros) {
        return BTreeMap::new();
    }
    let declared = |name: &str| {
        package.files.iter().any(|f| {
            f.structs.iter().any(|s| s.name == name)
                || f.enums.iter().any(|e| e.name == name)
                || f.unread.iter().any(|(n, _)| n == name)
        })
    };
    bound.retain(|name, _| !declared(name));
    bound
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::Dep;

    // Registry number types.

    const BIGINT_SUM: &str = "c89e69e7e0f03bea5ef08013795c25018e101932225a656383bd384495ecc367";

    fn registered_package(deps: Vec<Dep>, source: &str) -> Package {
        let (file, _) = crate::ingest::read_text(source);
        Package {
            name: "w".into(),
            version: "0.1.0".into(),
            description: String::new(),
            repository: String::new(),
            homepage: String::new(),
            license: String::new(),
            rust_version: None,
            authors: Vec::new(),
            dir: String::new(),
            targets: Vec::new(),
            deps,
            lints: BTreeMap::new(),
            files: vec![file],
        }
    }

    fn registered_dep(name: &str, rename: Option<&str>) -> Dep {
        Dep {
            name: name.into(),
            rename: rename.map(str::to_string),
            kind: "normal".into(),
            path: None,
            inside: false,
            platform: None,
        }
    }

    fn registered_lock(version: &str, checksum: &str) -> LockedPackage {
        LockedPackage {
            name: "num-bigint".into(),
            version: version.into(),
            source: CRATES_IO.into(),
            checksum: checksum.into(),
        }
    }

    #[test]
    fn a_reading_holds_only_at_a_locked_version_it_was_read_from() {
        let package = registered_package(vec![registered_dep("num-bigint", None)], "");
        let read = vec!["num_bigint::BigInt".to_string()];
        assert_eq!(
            of_package(&package, &[registered_lock("0.4.8", BIGINT_SUM)], &[]),
            read
        );
        // Another version, the same version from another archive, two
        // versions at once, or none: the type is not read.
        assert!(of_package(&package, &[registered_lock("0.4.9", BIGINT_SUM)], &[]).is_empty());
        assert!(of_package(&package, &[registered_lock("0.4.8", &"0".repeat(64))], &[]).is_empty());
        let both = [
            registered_lock("0.4.8", BIGINT_SUM),
            registered_lock("0.3.3", BIGINT_SUM),
        ];
        assert!(of_package(&package, &both, &[]).is_empty());
        assert!(of_package(&package, &[], &[]).is_empty());
    }

    #[test]
    fn a_reading_holds_only_where_the_crate_has_its_own_name() {
        let lock = [registered_lock("0.4.8", BIGINT_SUM)];
        // The package not a dependency, renamed, or its name taken by
        // another package's rename.
        let none = registered_package(Vec::new(), "");
        assert!(of_package(&none, &lock, &[]).is_empty());
        let renamed = registered_package(vec![registered_dep("num-bigint", Some("big"))], "");
        assert!(of_package(&renamed, &lock, &[]).is_empty());
        let taken = registered_package(
            vec![
                registered_dep("num-bigint", None),
                registered_dep("other", Some("num_bigint")),
            ],
            "",
        );
        assert!(of_package(&taken, &lock, &[]).is_empty());
    }

    #[test]
    fn a_use_binds_a_registered_type_where_nothing_else_takes_the_name() {
        // What a `use` binds is the resolver's
        // (`names::tests::registered_bindings_*`); a type of the package of
        // the name keeps it from being read as the crate's.
        let bound = || {
            BTreeMap::from([
                ("Big".to_string(), "num_bigint::BigInt".to_string()),
                ("BigInt".to_string(), "num_bigint::BigInt".to_string()),
            ])
        };
        let source = "use num_bigint::BigInt;\nuse ::num_bigint::BigInt as Big;\n";
        let package = registered_package(Vec::new(), source);
        assert_eq!(bindings(&package, bound()), bound());
        let declared = "use num_bigint::BigInt as Big;\npub struct Big(u8);\n";
        let package = registered_package(Vec::new(), declared);
        assert_eq!(
            bindings(&package, bound()),
            BTreeMap::from([("BigInt".to_string(), "num_bigint::BigInt".to_string())])
        );
    }

    #[test]
    fn a_reading_of_a_ratio_holds_only_where_its_integers_are_read() {
        const RATIO_SUM: &str = "f83d14da390562dca69fc84082e73e548e1ad308d24accdedd2720017cb37824";
        let package = registered_package(vec![registered_dep("num-rational", None)], "");
        let ratio = LockedPackage {
            name: "num-rational".into(),
            version: "0.4.2".into(),
            source: CRATES_IO.into(),
            checksum: RATIO_SUM.into(),
        };
        assert_eq!(
            of_package(
                &package,
                &[ratio.clone(), registered_lock("0.4.8", BIGINT_SUM)],
                &[]
            ),
            ["num_rational::BigRational".to_string()]
        );
        assert!(of_package(&package, std::slice::from_ref(&ratio), &[]).is_empty());
        assert!(of_package(
            &package,
            &[ratio, registered_lock("0.4.7", BIGINT_SUM)],
            &[]
        )
        .is_empty());
    }

    // Class: refuted readings (repair of round 4's confirmed counterexamples).

    fn refuted_dep(kind: &str, platform: Option<&str>) -> Dep {
        Dep {
            kind: kind.into(),
            platform: platform.map(str::to_string),
            ..registered_dep("num-bigint", None)
        }
    }

    #[test]
    fn a_reading_holds_only_for_a_normal_dependency_built_from_its_archive() {
        let lock = [registered_lock("0.4.8", BIGINT_SUM)];
        let normal = registered_package(vec![refuted_dep("normal", None)], "");
        assert_eq!(of_package(&normal, &lock, &[]).len(), 1);
        // A build or dev dependency is not in the library's crate graph;
        // a platform's dependency is not on every platform.
        for dep in [
            refuted_dep("build", None),
            refuted_dep("dev", None),
            refuted_dep("normal", Some("cfg(windows)")),
        ] {
            let package = registered_package(vec![dep], "");
            assert!(of_package(&package, &lock, &[]).is_empty());
        }
        // A configuration that replaces the source: the lock no longer
        // names what Cargo builds.
        let replaced = [".cargo/config.toml".to_string()];
        assert!(of_package(&normal, &lock, &replaced).is_empty());
    }

    #[test]
    fn a_package_that_makes_items_by_a_macro_binds_no_registered_name() {
        let bound = || BTreeMap::from([("BigInt".to_string(), "num_bigint::BigInt".to_string())]);
        let plain = registered_package(Vec::new(), "use num_bigint::BigInt;\n");
        assert_eq!(bindings(&plain, bound()), bound());
        let made = registered_package(
            Vec::new(),
            "use num_bigint::BigInt;\nmacro_rules! m { () => {} }\nmod inner { m!(); }\n",
        );
        assert!(made.files[0].item_macros);
        assert!(bindings(&made, bound()).is_empty());
        // A macro's definition alone makes no item.
        let defined = registered_package(Vec::new(), "macro_rules! m { () => {} }\n");
        assert!(!defined.files[0].item_macros);
    }
}
