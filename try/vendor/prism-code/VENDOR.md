# Vendored `prism-code`

This directory is a byte-for-byte copy of the browser-relevant files from the
published `prism-code@0.9.1` npm package. Nothing here is rebuilt.

- Package: `prism-code@0.9.1`
- Source: <https://registry.npmjs.org/prism-code/-/prism-code-0.9.1.tgz>
- Tarball shasum (SHA-1): `36fe09ce09dcf3f393ab3bd68c350ce2e965ff07`
- Tarball integrity:
  `sha512-AecO+1zwolME+fCdLRD8IS/1j/4lneAmnqhll2ConDHJ2OG+dEiPbl94/nB4BdlWwp5vAT1AffOuT0SYECrxOg==`
- Files taken: `src/`, `fixtures/Calculator.holo`, `LICENSE`.
- `package.json` here is a minimal replacement that keeps `version` and
  `type: module` (the published manifest is not needed for relative imports).
- `fixtures/Calculator.holo`: 24,801 bytes, SHA-256
  `84371aaac82af6b2a81a25a81d43d4097892985f55ae4919cfdcfa8b6e3486e7`.

The page imports `src/browser.mjs` directly, so there is no bundler and no
external request at runtime. `src/node.mjs` and `src/docker.mjs` are unused in
the browser and are kept only so the vendored tree matches the published `src/`.

## Local patch

`src/core.mjs` differs from the published 0.9.1 file in one place: `compileGuest`
also accepts a target/2 guest's immutable `__stack_low` and `__stack_high`
global exports (PrismPM `SPEC.md`, application V2), both or neither. The fix
is proposed upstream as
[UOR-Foundation/PrismPM#640](https://github.com/UOR-Foundation/PrismPM/pull/640).
When a prism-code release carries it, re-vendor that release and drop this
section.
