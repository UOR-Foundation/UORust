//! C1: verify the complete candidate tree before a factory test closure is built.
//! Checking only materialized files would allow undeclared test suites to vanish
//! from both discovery and qualification.

use std::collections::BTreeSet;

/// The full committed revision must contain every script test the certified
/// planner includes for its `test` obligation, whether it runs or reuses a
/// receipt. A missing suite invalidates the closure before materialization.
pub(crate) fn verify<'a, I>(
    complete_revision: I,
    test_closure: Option<&[String]>,
) -> Result<usize, String>
where
    I: IntoIterator<Item = &'a str>,
{
    let suites: BTreeSet<&str> = complete_revision
        .into_iter()
        .filter(|path| path.starts_with("scripts/"))
        .filter(|path| {
            let name = path.rsplit('/').next().unwrap_or("");
            (name.starts_with("test_") && name.ends_with(".py"))
                || (name.starts_with("test-") && name.ends_with(".sh"))
        })
        .collect();
    if suites.is_empty() {
        return Err("complete revision contains no script test suites".into());
    }
    let included = test_closure
        .ok_or("factory plan omitted the test obligation's materialized file set")?;
    let missing: Vec<&str> = suites
        .iter()
        .copied()
        .filter(|suite| !included.iter().any(|path| path == suite))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "factory test closure omits {} suite(s) from complete revision: {}",
            missing.len(),
            missing.join(", ")
        ));
    }
    Ok(suites.len())
}
