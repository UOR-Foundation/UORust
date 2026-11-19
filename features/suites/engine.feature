Feature: engine

  uorust reads a Cargo workspace and writes a Prism project: router proposals,
  admission that refuses ambiguity with typed refusals, and a byte-deterministic
  projection the PrismPM SDK checks.

  @UE-01 @build
  Scenario: uorust projects the fixture workspace into exactly the committed Prism project, which lexlean fmt, lexlean lock and prismpm check accept
    Given the fixture workspace in crates/uorust/tests/fixtures/thermostat
    When the suite exercises UE-01
    Then the fresh projection equals crates/uorust/tests/fixtures/thermostat-prism byte for byte

  @UE-02 @build
  Scenario: Two projections of the same workspace are byte-identical, for the fixture and for this workspace
    Given the fixture workspace and this workspace
    When the suite exercises UE-02
    Then each workspace projects to the same files with the same bytes twice

  @UE-03 @build
  Scenario: Admission refuses an ambiguous call, a feature without acceptance and a missing MSRV with typed refusals recorded as open claims, and emits no record for them
    Given a workspace whose call is ambiguous, whose enum is both an error and a state, whose functions are untested and which declares no rust-version
    When the suite exercises UE-03
    Then frontier.toml records ambiguous-call, ambiguous-fact, feature-without-acceptance and no-msrv refusals with status open
    And the Interface module declares no feature

  @UE-04 @build
  Scenario: Total functions over bool, unsigned comparisons and unit enums lift to LexLean definitions, closed assertions over them become decide theorems, and every other function is residue with a typed reason
    Given the fixture workspace
    When the suite exercises UE-04
    Then Behavior defines shouldHeat, withinBand, nextMode and halt with decide theorems from the fixture's assertions
    And frontier.toml records midpoint as arithmetic residue and parse_target as type residue

  @UE-05 @build
  Scenario: The engine's router equals model/routes.toml route for route and answers every router vector of that table exactly
    Given the route table in model/routes.toml
    When the suite exercises UE-05
    Then uorust::ROUTES lists the same routes in the same order
    And uorust::respond answers each non-workbench vector with its response

  @UE-06 @build
  Scenario: Every way a run can end without a projection is a typed UorustError variant with a negative test
    Given a missing workspace, a directory with no manifest, a malformed lockfile, a bad timestamp, a foreign output directory and a stale projection
    When the suite exercises UE-06
    Then each ends in its own UorustError variant

  @UE-07 @build
  Scenario: The gate checks the committed fixture projection and a fresh projection of this workspace with lexlean and prismpm
    Given the Justfile
    When the suite exercises UE-07
    Then vv runs the uorust recipe, which runs the three SDK checks on both projections

  @UE-08 @build
  Scenario: A public trait's methods are interface calls its module's feature names, and a method call through the trait resolves to the trait's declaration
    Given the fixture workspace, whose sensor module declares pub trait Sensor implemented by Fixed
    When the suite exercises UE-08
    Then Interface declares callThermoCoreSensorSensorRead and callThermoCoreSensorSensorCalibrated, and featureThermoCoreSensor names both and is accepted by the test that calls Fixed(20).read()

  @UE-09 @build
  Scenario: Each module with processes is its own OPM view beside its component's: the component is the object, the module's functions are its processes, and each process zooms into the processes it calls in the module
    Given the fixture workspace, whose config module's load_target calls parse_target
    When the suite exercises UE-09
    Then Design declares the view opmModuleLibThermoCoreConfig with an invocation and a zoom from load_target to parse_target, and one view per other module

  @UE-10 @build
  Scenario: The Behavior and SupplyChain modules take their titles from uorust.titles, a glossary package the projection carries and locks
    Given the fixture workspace
    When the suite exercises UE-10
    Then Behavior is titled Lifted Rust behavior and SupplyChain Locked supply chain, both use uorust.titles@1.0.0, and lexlean.toml and lexlean.lock name language/uorust.titles

  @UE-11 @build
  Scenario: Unsigned arithmetic whose overflow the code names, signed integers, strings, structs, tuples, Option, Result with ?, slices with structural recursion, integer matches and string format! lift to LexLean definitions that lexlean verify accepts, with the Lean axioms they need declared
    Given the fixture workspace, whose band module uses checked, wrapping and saturating arithmetic, i64, &str, a struct, a tuple, Option, Result, a slice and format!
    When the suite exercises UE-11
    Then every band function is a Behavior definition, plain + in midpoint stays arithmetic residue, and the uorust recipe runs lexlean verify on both projections

  @UE-12 @build
  Scenario: A RustSec advisory that affects a locked package's version is a Risk on the supply chain, answered by the advisory gate or accepted where deny.toml ignores it with its reason; one nothing answers is refused, and a run without a database records that none was consulted
    Given the advised fixture, whose Cargo.lock pins smallvec 0.6.9, with five copied RustSec advisories and a deny.toml that ignores RUSTSEC-2018-0018
    When the suite exercises UE-12
    Then Security holds a Risk for each of the four advisories that affect 0.6.9, an AcceptedRisk for the ignored one, failing gate measurements for the rest, and the SDK accepts the projection

  @UE-13 @build
  Scenario: Every feature of this workspace's own projection is admitted: each library module and binary entry point is exercised by a test, doctest or example, directly or through a helper of the test's file
    Given this workspace, whose repo-model codegen and xtask binary have their own tests
    When the suite exercises UE-13
    Then frontier.toml records no feature-without-acceptance refusal, and Interface declares featureRepoModelCodegen and featureXtaskMain

  @UE-14 @build
  Scenario: uorust run on its own source reproduces its hand model: every hand component, call, threat, control and quality measure in app/src is paired with the derived record that states the same fact or listed as outside the code with a reason, the hand route table equals uorust::ROUTES, and the drift is 0
    Given this workspace and the hand model in app/src, with crates/uorust/tests/fixpoint.toml pairing their records
    When the suite exercises UE-14
    Then every paired record on both sides states its fact, every checked hand record is paired or outside, the route tables agree, and the drift is 0

  @UE-15 @build
  Scenario: Iterator chains, inherent methods, read-only maps, early returns and for loops lift
    Given the thermostat fixture, whose roster module uses iterator chains with closures, methods taking self, BTreeMap, HashMap and BTreeSet reads, early returns, let else, tuple lets and for loops that push onto locals
    When the suite exercises UE-15
    Then every roster function and method is a LexLean definition, each chain a helper with recursive_argument, no roster function is residue, and lexlean verify passes on the committed projection

  @UE-16 @build
  Scenario: A lifted function never turns unlifted
    Given this workspace, the thermostat fixture and their committed lift registers
    When the suite exercises UE-16
    Then every function of product code counted from the source has exactly one typed row keyed by its κ, a divergence appears on lifted rows only and the register keeps it, no κ a committed register records as lifted is decided otherwise, this workspace's register is current, a planted function whose callee stops lifting is refused by the register, by uorust write and by uorust check, and a planted unchanged function counted under another construct while its signature still states its own is refused by uorust admit against its parent's register whether or not another function still states it, while moving functions on whose signatures no longer state it, or whose callee lifted, is admitted, an inline module's homonym of a function is a row of its own, and the frontier's type construct row names the file and line of the declaration each name of its chain means where it is written

  @UE-17 @build
  Scenario: The lift translates MIR mechanically
    Given the mir-path fixture, its committed verdict register and the vendored MIR definition of the pinned toolchain
    When the suite exercises UE-17
    Then every MIR kind has a coverage row, the trial projection lists a loop helper with its measure and decrease theorems and a theorem per obligation, std's own MIR instantiated at a call's types, a std model from the table with a mutable borrow returned by its backward function, and a promoted constant lowered from its own body, the mutable borrow of a parameter is a residue row naming its MIR cause, the proved translation lifts, the refused one is residue naming the verifier's diagnostic, and a verdict of another digest admits nothing

  @UE-18 @build
  Scenario: The translator certifies itself and only shrinks by hand
    Given this workspace's lift register and the certified translator's manifest, and the parent commit's
    When the suite exercises UE-18
    Then each lifted translator function is a production root, a root the compiler finds ineligible is refused with its diagnostic, every certified source matches the SHA-256 its bundle records, the share counts the translator functions reachable from the translate entry and those certified, and a commit with more uncertified translator functions than its parent is refused naming each new one and its cause

  @UE-19 @build
  Scenario: Every body of core is embedded and its evaluator checked against rustc
    Given core's MIR dumps, rust-src at the pinned toolchain and the MIR semantics module
    When the suite exercises UE-19
    Then every body is a value of the MIR datatype whose kinds are rustc's, a long list nests no deeper than its root, the constructs the evaluator does not define are read from its source, each register row records its embedding and what blocks it, and each sampled case is a theorem that the evaluator returns what rustc's build returned

  @UE-21 @build
  Scenario: Every numeric literal of a workspace gets a home by extraction and rewrite
    Given small synthetic workspaces and figure registers written for them
    When the suite exercises UE-21
    Then every numeric literal is a site at its place with a key an unrelated edit keeps, the writer expands each figure to its own token, a rewrite names the figure and a second changes nothing, what cannot be placed is refused with its reason, and the gate holds the workspace to its register

  @UE-22 @build
  Scenario: Unconditional exported numeric parameter macros of workspace libraries preserve the surface lift: an absolute invocation through a declared Cargo dependency alias with no arguments and one empty matcher expanding to one numeric literal is read as that literal, with the source facts and function identities unchanged; conditional, ambiguous, argument-taking and unknown macros remain residue
    Given a workspace library and its renamed parameter dependency
    When the suite exercises UE-22
    Then the direct and constant-mediated literal macros lift and other macro forms stay residue

  @UE-23 @build
  Scenario: A library embedding resolves crate-relative implementation spans against the defining crate
    Given two libraries with different implementation headers at the same relative source path and a conflicting root file
    When the suite exercises UE-23
    Then each inherent method resolves only through its own library's source and a missing own source stays unresolved

  @UE-24 @build
  Scenario: An invalid recursive-descent parameter is refused without a panic
    Given a caller with two parameters and descent evidence naming parameter index two
    When the suite exercises UE-24
    Then the result is typed form residue and valid single-parameter evidence still names its own parameter

  @UE-25 @build
  Scenario: A cross-file constant is emitted before its caller
    Given a caller preceding its constant and a mutual group whose member needs a later definition
    When the suite exercises UE-25
    Then dependencies precede their callers and the mutual group remains together without treating string contents as calls

  @UE-26 @build
  Scenario: Inert dependency string data does not taint figure rewrites
    Given a dependency resource that is one string literal and mentions a tracked callee
    When the suite exercises UE-26
    Then inert data introduces no location reader while malformed source remains unknown and real readers remain detected

  @UE-27 @build
  Scenario: Qualified associated functions retain their acceptance roots
    Given two owners with an associated function of the same short name
    When the suite exercises UE-27
    Then each qualified assertion binds its own lifted function and an unknown owner is refused

  @UE-28 @build
  Scenario: MIR method homonyms keep stable qualified names
    Given three modules with equally named conversion methods and their actual compiler MIR
    When every body or only the last module's method is translated on trial
    Then every method has its own candidate and the last method keeps the same declaration name in both views

  @UE-29 @build
  Scenario: Native MIR proof work is bounded by dependency components
    Given dependent translations, mutually recursive translations and a positive batch budget
    When the prover plans and executes its trials
    Then every eligible translation is scheduled once after its dependencies, recursive components remain together, and unchanged proved verdicts need no verifier rerun
    And a verifier diagnostic on a theorem opening names that theorem, never the preceding function
    But zero budgets, dangling dependencies and a recursive component larger than the budget are refused before verification

  @UE-30 @build
  Scenario: An admitted registered alias keeps its model through registry expansion
    Given a package using the locked BigRational alias and its registered BigInt dependency
    When its source is projected with registry aliases available
    Then its identity function lifts as the stored integer pair
    And a mismatched rational archive checksum leaves that function unlifted

  @UE-31 @build
  Scenario: Qualified field types retain their declaring scope
    Given two modules declaring different types with the same short name
    And a holder that uses a qualified type and a conflicting bare imported type
    When the workspace is projected
    Then both field getters lift with the types their written paths identify
    And unresolved or ambiguous paths remain refused

  @UE-32 @build
  Scenario: MIR collection avoids native object generation without losing bodies
    Given private, generic, closure and impl functions compiled by the pinned Rust compiler
    When native emission and forced metadata MIR emission collect the same lowering pass
    Then their body inventories and bytes are identical and nonempty
    But metadata emission without forced MIR encoding is rejected for missing bodies

  @UE-33 @build
  Scenario: Locked JSON data can pass through a formal function without losing its representation
    Given serde_json 1.0.114 pinned to its registered archive checksum
    When identity, tuple and enclosing-field functions are projected
    Then their JSON values retain the six source variants and numeric storage alternatives
    And comparisons without a faithful equality model remain unlifted
    And the actual SDK verifies the carrier definitions and their function bodies
    And recursive JSON native exports are refused individually under the pinned library profile
    And replacing the archive checksum refuses the carrier reading

  @UE-34 @build
  Scenario: Shared mutable owners retain references instead of frozen contents
    Given Rc and Arc owners of RefCell and Cell, including two fields that may alias
    When identity, owner cloning, tuple and enclosing-field functions are projected
    Then their models transport the same allocation references
    And the actual SDK verifies their carrier definitions and function bodies
    But heap allocation, reads, writes, owner counts and value equality stay unlifted
    And native roots requiring these reference carriers are refused individually

  @UE-35 @build
  Scenario: Long type names generate canonical declarations
    Given a structure field and recursive tree with long qualified type names
    When their functions are projected
    Then the pinned SDK accepts the generated modules without rewriting their layout

  @UE-36 @build
  Scenario: Owned slices retain their ordered elements
    Given Box, Rc and Arc slices, including an Rc slice field in a structure
    When identity, length and owner-clone functions are projected
    Then the actual SDK verifies their sequence models
    But reference counts and slices of mutable cells remain unlifted

  @UE-37 @build
  Scenario: A locked bit vector retains logical bits rather than storage words
    Given default bitvec 1.0.1 BitVec pinned to its registered checksum
    When identity and length functions are projected
    Then the actual SDK verifies their Boolean sequence model
    But raw storage and allocation capacity remain unlifted
    And a wrong archive checksum refuses the model

  @UE-38 @build
  Scenario: Native proof refresh reuses observations without trusting stale proofs
    Given SDK-proved translations whose prior records contain observed axiom policies
    When one translation digest is made stale
    Then the actual SDK verifies its refreshed proof in one run using the prior policies as hints
    And unchanged replay uses zero verifier calls
    But incorrect policy hints require corrective SDK diagnostics and another verification

  @UE-39 @build
  Scenario: An alias expands in its defining scope
    Given an alias of nested tuples whose element type has homonyms in other modules
    When the alias is used from a module importing another homonym
    Then its fields retain the original element type and the SDK verifies the model
    And alias type parameters retain their own binding
    But an alias of an unread interior-mutable type stays unlifted

  @UE-40 @build
  Scenario: A renamed import retains its declared type
    Given a unique workspace type imported under another name
    When fields and function signatures use the renamed import
    Then their functions retain the original data type and the actual SDK verifies them
    And a generic type parameter shadowing an import remains generic

  @UE-41 @build
  Scenario: Native aggregates retain their field representations
    Given source structures and native tuples with different signed integer representations
    When their compiler bodies are translated together with an unsigned tuple function
    Then the incompatible aggregates remain typed MIR residue
    And the compatible tuple is translated and its generated fields pass SDK type checking

  @UE-42 @build
  Scenario: Native bodies reuse admitted registry carriers
    Given locked numeric, JSON and default bit-vector dependencies
    When compiler bodies transport their values through loops
    Then their native translations retain the registered representations and the SDK verifies them
    But JSON equality without an operation model is refused
    And a wrong archive checksum prevents the native carrier reading

  @UE-43 @build
  Scenario: Native verification produces scoped reusable feedback
    Given native candidates including an overflowing function
    When the prover submits their trials to the actual SDK
    Then compatible candidates are proved and the overflowing candidate is refused
    And the structured SDK result names only Behavior and its dependency closure
    And unchanged replay performs no new verification and retains the same verdicts

  @UE-44 @build
  Scenario: Loop evidence omits unused invariant arguments
    Given a native countdown loop transporting an invariant Boolean
    When the prover generates its decrease evidence
    Then its live values share one changing loop-state parameter
    And the actual SDK proves the loop without an unbound invariant argument

  @UE-45 @build
  Scenario: Native hash collections reuse unordered carriers
    Given standard maps and sets with default and explicit hasher types
    When compiler-derived countdown loops transport their values
    Then the SDK proves their unordered collection models and a workspace homonym retains its own type
    But hasher inspection, iteration order and unsupported element types stay unlifted

  @UE-46 @build
  Scenario: Compiler-input selection preserves proof history
    Given an SDK-decided register with proved and refused translations
    When the prover replays one compiler body and then all compiler bodies
    Then it retains the other selection's actual verdicts and performs zero verifier runs
    But a live translation with a stale digest still needs actual verification

  @UE-47 @build
  Scenario: Workspace native bodies share primitive IEEE models
    Given ordinary f32 and f64 functions without a core-library scope
    When compiler-derived loops transport floats, compute supported operations and convert their bits
    Then the actual SDK proves their existing IEEE models including negative zero
    And a workspace declaration under a primitive name retains its own type
    But an operation outside the IEEE model stays unlifted

  @UE-48 @build
  Scenario: Early failure preserves mutable state
    Given functions that update mutable parameters before Result or Option propagation
    When propagation succeeds or exits early, including after a state-changing callee
    Then the SDK proves the returned values and every resulting state against executable Rust vectors
    But a mutable alias still refuses the lift

  @UE-49 @build
  Scenario: Native numeric observations use locked stored values
    Given checksum-admitted BigInt and BigRational values
    When compiler-derived bodies observe their signs, identities and stored ratio components
    Then the actual SDK verifies the translations and closed numeric acceptance vectors
    And raw negative or zero denominators retain the locked implementation's behavior
    But a wrong archive checksum and an unsupported operation stay unlifted

  @UE-50 @build
  Scenario: Proof campaigns publish their own measured completion position
    Given compiler inputs and an existing actual SDK verdict register
    When prove writes a requested machine report
    Then it writes the final projection and reports its census, remaining functions and blockers with exact byte bindings
    And unchanged proofs require zero verifier calls without claiming full-project SDK acceptance
    But a foreign projection directory is a typed refusal and its files are preserved

  @UE-51 @build
  Scenario: Numeric constructors derive their reading from word widths
    Given locked numeric crates and compiler-derived primitive integer types
    When constructors convert unsigned values and signed two's-complement words
    Then the SDK verifies every primitive width and sign boundary using bounded binary conversion
    And rational construction preserves the integer as numerator and one as denominator

  @UE-52 @build
  Scenario: Proof decisions follow their semantic dependency context
    Given an actual SDK proof of a native function calling a source-lifted helper
    When that helper changes without changing the caller's own tokens
    Then UORust verifies the changed proof context and reuses its unchanged replay with zero verifier calls
    But a broken shared semantic context is a typed campaign failure rather than per-function refusal
