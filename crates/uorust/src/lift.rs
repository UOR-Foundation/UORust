//! Function bodies to LexLean definitions, where the lift is mechanical.
//!
//! The lift covers the total, pure fragment whose meaning does not depend on
//! the machine: `bool` logic; unsigned integers with comparisons, division and
//! remainder by a non-zero literal, and the `checked_*`, `wrapping_*` and
//! `saturating_*` operations whose overflow behaviour the method names; `i8` to
//! `i64` with comparisons and, for `i64`, the checked operations; strings
//! compared for equality; unit-variant enums; structs and tuples of liftable
//! types; `Option` and `Result` of liftable types, with `?` on a `let`; slices
//! and `Vec` with their length, indexing and structural recursion over a slice
//! pattern; `let`, `if`, `if let`, `match` and calls among lifted functions.
//! Anything else (`+`, `-` and `*` without a named overflow behaviour,
//! references to mutable state, loops, closures, other methods) is not guessed
//! at: the function becomes a residue row with a typed reason.

use std::collections::{BTreeMap, BTreeSet};

/// A lift step: the value, or the residue that stops the lift.
pub(crate) type Lift<T> = Result<T, Residue>;

use crate::ast as rs;
use crate::lex::{camel, Arm, Decl, Expr};
// Imported by name: the lift reads a type two modules name `Refusal` by
// its file's import, never by a path written out.
use crate::mir::layout::Refusal;
use crate::order::ORDERING;

/// Why a function stays in the frontier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Residue {
    /// The function is generic or async, or takes `self`.
    Signature(String),
    /// A parameter or the result has a type outside the lift.
    Type(String),
    /// The body has statements other than `let` before its value.
    Statements,
    /// Integer arithmetic whose overflow behaviour is a build setting.
    Arithmetic(String),
    /// A `usize` operator whose value leaves the width of these targets,
    /// by pointer width: the operator, and the targets of each width.
    Overflow(String, Vec<(u32, Vec<String>)>),
    /// Integer arithmetic whose meaning depends on the width of the type
    /// (`usize`, or a signed type narrower than `i64`).
    Width(String),
    /// An expression form outside the lift.
    Form(String),
    /// A call to a function that is not lifted.
    Call(String),
    /// Another function of the crate lifts to the same LexLean name.
    Collision(String),
    /// A kind of MIR the translator does not lower (`Rvalue::Ref: Mut`), or
    /// an obligation of its translation the verifier did not discharge.
    Mir(String),
    /// A layout the targets' data layouts do not give (see
    /// `mir::layout::Refusal`).
    Layout(Refusal),
}

impl Residue {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::Signature(_) => "signature",
            Self::Type(_) => "type",
            Self::Statements => "statements",
            Self::Arithmetic(_) | Self::Overflow(..) => "arithmetic",
            Self::Width(_) => "width",
            Self::Form(_) => "form",
            Self::Call(_) => "call",
            Self::Collision(_) => "collision",
            Self::Mir(_) | Self::Layout(_) => "mir",
        }
    }

    pub(crate) fn detail(&self) -> String {
        match self {
            Self::Signature(s) => format!("signature outside the lift: {s}"),
            Self::Type(t) => format!("type outside the lift: {t}"),
            Self::Statements => "the body has statements other than let".to_string(),
            Self::Arithmetic(op) => {
                format!("integer operator {op} wraps or panics depending on the build profile")
            }
            Self::Overflow(op, widths) => {
                let at: Vec<String> = widths
                    .iter()
                    .map(|(bits, targets)| format!("past 2^{bits} on {}", targets.join(" and ")))
                    .collect();
                format!(
                    "usize {op} wraps or panics where its value leaves the pointer width: {}",
                    at.join(", ")
                )
            }
            Self::Width(op) => {
                format!("{op} depends on an integer width LexLean does not fix")
            }
            Self::Form(f) => format!("expression form outside the lift: {f}"),
            Self::Call(c) => format!("calls {c}, which is not lifted"),
            Self::Collision(n) => format!("another function of the crate is also named {n}"),
            Self::Mir(cause) => format!("mir::{cause}"),
            Self::Layout(refusal) => format!("mir::Memory: {}", refusal.text()),
        }
    }
}

/// A Rust type the lift can name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Ty {
    /// `()`: LexLean's `Unit`, whose one value is `()`.
    Unit,
    Bool,
    /// An unsigned integer of this many bits; `None` for `usize`.
    Nat(Option<u32>),
    /// A signed integer of this many bits, held as `Int64`.
    Int(u32),
    /// An IEEE 754 binary floating-point number, held as its bit pattern
    /// (a `Nat`): the format's exponent and fraction widths, as core's own
    /// constants give them (see `mir::float`).
    Float(crate::mir::float::Format),
    /// A signed integer the 64-bit family does not hold: `i128`, of 128
    /// bits, or `isize` (`None`), as wide as the target's pointer. Held as
    /// LexLean's mathematical `Int`, as `u128` and `usize` are held as
    /// `Nat`: the width says what an operation may return, and is no part
    /// of how a value is held.
    Integer(Option<u32>),
    /// An integer of a registered crate whose values are exactly the
    /// mathematical integers (`num_bigint::BigInt`, see
    /// [`crate::registered::readings`]): LexLean's `Int`, with no width
    /// an operation could overflow.
    Unbounded,
    /// A ratio of two integers of no width as a registered crate stores it
    /// (`num_rational::BigRational`, see [`crate::registered::readings`]):
    /// the pair of its numerator and denominator, Lean's `(Int * Int)`. The
    /// pair is the stored one, not a reduced one, so the lift reads no
    /// equality or order of it: the crate's own compares values.
    Ratio,
    /// The MIR path's signed integer of this many bits (`None` for
    /// `isize`), held as its two's-complement bit pattern in `Nat`: LexLean
    /// converts no `Nat` to a signed family, so a cast between the two
    /// closes only where both are `Nat`.
    Signed(Option<u32>),
    Str,
    /// A `char`, held as the one-character string it is.
    Char,
    /// The MIR path's `char`, held as its Unicode scalar value in `Nat`:
    /// at most `0x10FFFF` and outside the surrogates `0xD800..=0xDFFF`, as
    /// core's own `char` methods compute with it.
    Scalar,
    /// The error of an integer `parse`: the prelude's `ParseIntError`.
    ParseError,
    /// A `PathBuf` or `Path`: the prelude's owned path record, its
    /// components and whether it is absolute.
    Path,
    /// A unit-variant enum or a struct.
    Named(String),
    Option(Box<Ty>),
    /// `Result` of a value and an error.
    Outcome(Box<Ty>, Box<Ty>),
    List(Box<Ty>),
    Tuple(Vec<Ty>),
    /// A pair that names a data-carrying enum: Lean's product `(A * B)`,
    /// which an inductive may hold itself in (`Vec<(String, Self)>`), where
    /// a generated tuple structure would make the two mutually recursive.
    Pair(Vec<Ty>),
    /// A map, held as the list of its entries; `true` when its iteration
    /// order is its key order (`BTreeMap`), which `insert` keeps.
    Map(Box<Ty>, Box<Ty>, bool),
    /// A set, held as the list of its members; `true` when its iteration
    /// order is its member order (`BTreeSet`), which `insert` keeps.
    Set(Box<Ty>, bool),
    /// A pure function of its inputs: `&dyn Fn(A) -> R`, `impl Fn(A) -> R`.
    Fn(Vec<Ty>, Box<Ty>),
    /// A type parameter of the definition being lifted, by its LexLean
    /// name: LexLean's `'T`, which the definition binds and each call
    /// instantiates explicitly.
    Param(String),
}

impl Ty {
    /// The LexLean type.
    pub(crate) fn render(&self) -> String {
        match self {
            Self::Unit => "Unit".into(),
            Self::Bool => "Bool".into(),
            Self::Nat(_) | Self::Float(_) | Self::Signed(_) | Self::Scalar => "Nat".into(),
            Self::Int(_) => "Int64".into(),
            Self::Integer(_) | Self::Unbounded => "Int".into(),
            Self::Ratio => "(Int * Int)".into(),
            Self::Str | Self::Char => "String".into(),
            Self::ParseError => format!("{PRELUDE}::ParseIntError"),
            Self::Path => format!("{PRELUDE}::FilePath"),
            Self::Named(n) => n.clone(),
            Self::Option(t) => format!("(Option {})", t.render()),
            Self::Outcome(t, e) => format!("(Result {} {})", t.render(), e.render()),
            Self::List(t) | Self::Set(t, _) => format!("(List {})", t.render()),
            Self::Tuple(ts) => format!("Tuple{}", ts.iter().map(|t| t.tag()).collect::<String>()),
            Self::Pair(ts) => product_types(ts),
            // The list of its entries, each the tuple of a key and a value.
            Self::Map(..) => format!("(List {})", self.entry().render()),
            Self::Fn(ins, out) => format!(
                "(Fn ({}) {})",
                ins.iter()
                    .map(Ty::render)
                    .collect::<Vec<String>>()
                    .join(" "),
                out.render()
            ),
            Self::Param(n) => format!("'{n}"),
        }
    }

    /// A map's entry: the tuple of its key and value.
    pub(crate) fn entry(&self) -> Ty {
        match self {
            Self::Map(k, v, _) => {
                let fields = vec![(**k).clone(), (**v).clone()];
                if fields.iter().any(|t| !t.parameters().is_empty()) {
                    Ty::Pair(fields)
                } else {
                    Ty::Tuple(fields)
                }
            }
            other => other.clone(),
        }
    }

    pub(crate) fn tag(&self) -> String {
        match self {
            Self::Unit => "Unit".into(),
            Self::Bool => "Bool".into(),
            Self::Nat(_) | Self::Float(_) | Self::Signed(_) | Self::Scalar => "Nat".into(),
            Self::Int(_) => "Int".into(),
            // `Int` is the tag of `Int64`; a name of the prefix no
            // workspace type lifts under tells the two apart.
            Self::Integer(_) | Self::Unbounded => INTEGER_TAG.into(),
            // The tag of the pair of two `Int`s, which it is.
            Self::Ratio => format!("Pair{INTEGER_TAG}{INTEGER_TAG}"),
            Self::Str | Self::Char => "String".into(),
            Self::ParseError => "ParseIntError".into(),
            Self::Path => "FilePath".into(),
            Self::Named(n) if n.contains("::") => format!(
                "__Qualified{}",
                n.as_bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            ),
            Self::Named(n) => n.clone(),
            Self::Option(t) => format!("Option{}", t.tag()),
            Self::Outcome(t, e) => format!("Result{}{}", t.tag(), e.tag()),
            Self::List(t) | Self::Set(t, _) => format!("List{}", t.tag()),
            Self::Tuple(ts) => format!("Tuple{}", ts.iter().map(|t| t.tag()).collect::<String>()),
            Self::Pair(ts) => format!("Pair{}", ts.iter().map(|t| t.tag()).collect::<String>()),
            Self::Map(..) => format!("List{}", self.entry().tag()),
            Self::Fn(ins, out) => format!(
                "Fn{}To{}",
                ins.iter().map(Ty::tag).collect::<String>(),
                out.tag()
            ),
            Self::Param(n) => n.clone(),
        }
    }

    /// The same LexLean type, and the same IEEE 754 format at every place
    /// either holds a float: a float is held as its bit pattern, a `Nat`,
    /// so its LexLean type alone does not tell a binary32 from a binary64
    /// or from an integer, and a value of one format where Rust has
    /// another is another number.
    pub(crate) fn same(&self, other: &Self) -> bool {
        self.render() == other.render() && self.formats() == other.formats()
    }

    /// The float format at each place of the type that is held as a `Nat`
    /// (`None` where that place is an integer), in order.
    fn formats(&self) -> Vec<Option<crate::mir::float::Format>> {
        match self {
            Self::Float(f) => vec![Some(*f)],
            Self::Nat(_) | Self::Signed(_) | Self::Scalar => vec![None],
            Self::Option(t) | Self::List(t) | Self::Set(t, _) => t.formats(),
            Self::Outcome(t, e) => [t.formats(), e.formats()].concat(),
            Self::Map(k, v, _) => [k.formats(), v.formats()].concat(),
            Self::Tuple(ts) | Self::Pair(ts) => ts.iter().flat_map(Ty::formats).collect(),
            Self::Fn(ins, out) => ins
                .iter()
                .flat_map(Ty::formats)
                .chain(out.formats())
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Every named type and tuple this type mentions, itself included.
    pub(crate) fn mentions(&self) -> BTreeSet<Ty> {
        match self {
            Self::Named(_) => BTreeSet::from([self.clone()]),
            Self::Tuple(ts) => {
                let mut out = BTreeSet::from([self.clone()]);
                for t in ts {
                    out.extend(t.mentions());
                }
                out
            }
            Self::Pair(ts) => {
                let mut out = BTreeSet::new();
                for t in ts {
                    out.extend(t.mentions());
                }
                out
            }
            Self::Option(t) | Self::List(t) | Self::Set(t, _) => t.mentions(),
            // Its entry: the tuple of a key and a value.
            Self::Map(..) => self.entry().mentions(),
            Self::Outcome(t, e) => {
                let mut out = t.mentions();
                out.extend(e.mentions());
                out
            }
            Self::Fn(ins, result) => {
                let mut out = result.mentions();
                for t in ins {
                    out.extend(t.mentions());
                }
                out
            }
            _ => BTreeSet::new(),
        }
    }

    /// Free type parameters, including those inside containers and callbacks.
    pub(crate) fn parameters(&self) -> BTreeSet<String> {
        match self {
            Self::Param(n) => BTreeSet::from([n.clone()]),
            Self::Option(t) | Self::List(t) | Self::Set(t, _) => t.parameters(),
            Self::Outcome(a, b) | Self::Map(a, b, _) => {
                a.parameters().union(&b.parameters()).cloned().collect()
            }
            Self::Tuple(ts) | Self::Pair(ts) => ts.iter().flat_map(Self::parameters).collect(),
            Self::Fn(ins, out) => ins
                .iter()
                .chain(std::iter::once(out.as_ref()))
                .flat_map(Self::parameters)
                .collect(),
            _ => BTreeSet::new(),
        }
    }
}

fn product_types(ts: &[Ty]) -> String {
    match ts {
        [] => "Unit".into(),
        [t] => t.render(),
        [first, rest @ ..] => format!("({} * {})", first.render(), product_types(rest)),
    }
}

/// The names of the enums and structs among the types a type mentions.
fn named_in(types: &BTreeSet<Ty>) -> BTreeSet<String> {
    types
        .iter()
        .filter_map(|t| match t {
            Ty::Named(n) => Some(n.clone()),
            _ => None,
        })
        .collect()
}

/// A struct value from its fields in declaration order: a record, or the
/// constructor `mk` of a struct of a recursive family.
pub(crate) fn struct_value(name: &str, fields: &[(&String, String)], types: &Types) -> String {
    if fields.is_empty()
        && types
            .enums
            .get(name)
            .is_some_and(|variants| variants.as_slice() == ["mk"])
    {
        return format!("(ctor {name}.mk)");
    }
    if types.boxed(name) {
        let args = fields
            .iter()
            .map(|(_, v)| format!(" {v}"))
            .collect::<String>();
        format!("(ctor {name}.mk{args})")
    } else {
        let parts: Vec<String> = fields.iter().map(|(f, v)| format!("{f} := {v}")).collect();
        format!("(record {name} {{{}}})", parts.join(", "))
    }
}

/// The `i`-th item of a tuple value `whole` of type `ty`.
pub(crate) fn tuple_item(ty: &Ty, whole: &str, i: usize) -> String {
    match ty {
        Ty::Pair(ts) => {
            let rest = (0..i).fold(whole.to_string(), |v, _| format!("(second {v})"));
            if i + 1 == ts.len() {
                rest
            } else {
                format!("(first {rest})")
            }
        }
        _ => format!("(project {whole} item{i})"),
    }
}

/// A tuple value of type `ty` from its items.
pub(crate) fn tuple_value(ty: &Ty, items: &[String]) -> String {
    match ty {
        Ty::Pair(_) => items
            .iter()
            .rev()
            .cloned()
            .reduce(|rest, first| format!("({first}, {rest})"))
            .unwrap_or_else(|| "()".into()),
        _ => {
            let parts: Vec<String> = items
                .iter()
                .enumerate()
                .map(|(i, v)| format!("item{i} := {v}"))
                .collect();
            format!("(record {} {{{}}})", ty.render(), parts.join(", "))
        }
    }
}

/// The unit-variant enums and the structs the lift can name.
#[derive(Clone, Debug, Default)]
pub(crate) struct Types {
    pub(crate) enums: BTreeMap<String, Vec<String>>,
    /// Struct name -> fields `(LexLean name, type)` in declaration order.
    pub(crate) structs: BTreeMap<String, Vec<(String, Ty)>>,
    /// A Rust type name -> the LexLean name it lifts to, where the crate's
    /// name qualifies it apart from another crate's type of the same name.
    pub(crate) aliases: BTreeMap<String, String>,
    /// Enums whose variants carry data -> each variant and its fields:
    /// the Rust field name of a struct variant, and the type.
    pub(crate) datas: BTreeMap<String, Vec<(String, Vec<DataField>)>>,
    /// The recursive families: the enums and structs that name themselves
    /// or each other, each one inductive group.
    pub(crate) families: Vec<BTreeSet<String>>,
    /// The traits each type derives (`Ord`, `PartialEq`, ...), by LexLean
    /// name: a derived order or equality is the structural one.
    pub(crate) derives: BTreeMap<String, BTreeSet<String>>,
    /// The MIR path's closures, by their type as MIR prints it: the types
    /// of their captures, as printed.
    pub(crate) closures: BTreeMap<String, Vec<String>>,
    /// Whether this is core's own scope, where MIR prints core's paths
    /// without the crate: `ops::ControlFlow` is `core::ops::ControlFlow`.
    pub(crate) core: bool,
    /// The floating-point types the scope reads, by name (`f32`), each the
    /// format core's own constants define for it.
    pub(crate) floats: BTreeMap<String, crate::mir::float::Format>,
    /// Each struct whose one field is a pattern type (`(u32) is 1..`), by
    /// LexLean name: the pattern's base type and the pattern, as written.
    pub(crate) valid: BTreeMap<String, (String, String)>,
    /// The structs that hold a mutable borrow, by LexLean name: each field
    /// that is a `&mut`, by position, with its type as printed. The lift
    /// holds the borrowed value there, threaded as the borrow is.
    pub(crate) borrowing: BTreeMap<String, Vec<(usize, String)>>,
    /// The discriminants an enum's source sets, one per variant in order,
    /// by LexLean name, where they are not the variants' indices alone.
    /// Held as `i64`, the width the lift lifts, and widened where read.
    pub(crate) discriminants: BTreeMap<String, Vec<i64>>,
    /// The workspace's structs and enums outside the lift, by LexLean name:
    /// the chain from each down to the leaf that keeps it out
    /// (`Mid.leaf: Leaf.x: f64`), so that a residue row states its cause
    /// and not only the nearest type the cause is inside.
    pub(crate) outside: BTreeMap<String, String>,
    /// Whether a registered crate's type by its path (`num_bigint::BigInt`,
    /// or a bare name a `use` binds to it) is not read in this scope: a
    /// macro the package invokes in item position may declare a module of
    /// the crate's name that a relative path here names first.
    pub(crate) paths_open: bool,
    /// The type parameters of the signature in hand, by Rust name: each
    /// the type it is read as (a LexLean type parameter, or the function
    /// type its one `Fn` bound names). A type parameter's name shadows a
    /// type of the same name, as Rust's own scoping has it.
    pub(crate) generic: BTreeMap<String, Ty>,
    /// Native-only views of complete source structs. Unread fields remain
    /// abstract values; this supplies no operation on their contents.
    pub(crate) native_views: BTreeMap<String, Ty>,
    /// The method each type's impl of a trait of the crate gives: (the
    /// trait, as a dictionary entry names it; the type, by LexLean name;
    /// the method) -> the key the impl's method lifts under.
    pub(crate) impls: BTreeMap<(String, String, String), String>,
    /// The fields of the lifted structs that are cells (see [`CellKind`]),
    /// by struct and field LexLean name: each holds its content, and is
    /// read only as [`CellKind`] says.
    pub(crate) cells: BTreeMap<String, BTreeMap<String, CellKind>>,
}

/// A standard cell a struct's field may be: a value whose content changes
/// behind a shared borrow. Each holds one `T` (`RefCell` beside a borrow
/// flag, `Cell` as `UnsafeCell<T>` alone, `Mutex` and `RwLock` beside a
/// lock and a poison flag), so the struct's value holds the content. The
/// flags are not part of what the lift reads: an operation that consults
/// one panics or blocks where it is set, and the lifted definition agrees
/// with Rust wherever Rust returns, as `default_of` does for a panic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CellKind {
    RefCell,
    Cell,
    Mutex,
    RwLock,
}

impl CellKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::RefCell => "RefCell",
            Self::Cell => "Cell",
            Self::Mutex => "Mutex",
            Self::RwLock => "RwLock",
        }
    }

    fn named(name: &str) -> Option<CellKind> {
        [Self::RefCell, Self::Cell, Self::Mutex, Self::RwLock]
            .into_iter()
            .find(|k| k.name() == name)
    }

    /// The method that reads the content where the read is the content as
    /// the function was called with it: `RefCell::borrow` and `Cell::get`.
    /// Neither cell is `Sync`, so no other thread reaches it, and no lifted
    /// code changes a cell (every change is a typed row), so the content
    /// stays what it was at the call. A `Mutex` or `RwLock` is `Sync`:
    /// another thread may change it between two locks, so its content is
    /// not read.
    fn read(self) -> Option<&'static str> {
        match self {
            Self::RefCell => Some("borrow"),
            Self::Cell => Some("get"),
            Self::Mutex | Self::RwLock => None,
        }
    }
}

/// What a function's own module scope lets a name of its body mean, beyond
/// the crate's tables: Rust resolves a `use` and a trait in scope per
/// module, and an inline module inherits no name of the module around it.
#[derive(Clone, Debug, Default)]
pub(crate) struct Seen {
    /// Each name a `use` item of the function's own module scope binds,
    /// with the path it names.
    pub(crate) uses: BTreeMap<String, String>,
    /// The names the scope itself binds (its modules, types, functions and
    /// `use` names) and those the body binds anew: a path whose head is one
    /// of them starts there, not at a crate of that name.
    pub(crate) local: BTreeSet<String>,
    /// The names of the methods and associated functions declared by the
    /// workspace's traits that may be in the scope: one it declares, one a
    /// `use` of it binds or imports by a path ending in the trait's name,
    /// and every one where it has a glob import. A call of such a name may
    /// reach the trait's method before the inherent one the lift reads.
    pub(crate) trait_methods: BTreeSet<String>,
    /// Whether a trait whose methods the workspace does not declare may be
    /// in scope: the scope imports a path of a crate outside the workspace
    /// and the standard library, or has a glob import of anything but the
    /// standard library.
    pub(crate) outside_traits: bool,
    /// Whether an item a macro makes in item position (a type, a trait, a
    /// module), which no declaration the tool reads shows, may be in the
    /// scope: the file invokes such a macro, or the package does and the
    /// scope imports by a glob or by a path inside the crate.
    pub(crate) macro_items: bool,
    /// Whether such an item may be what a path of the function's file
    /// names (`Types::paths_open`).
    pub(crate) paths_open: bool,
}

/// The language's primitive types a path of one segment may name.
pub(crate) const PRIMITIVE_TYPES: [&str; 17] = [
    "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
    "i128", "isize", "f32", "f64",
];

/// The tag of the mathematical `Int` in a generated name.
const INTEGER_TAG: &str = "LexLeanInt";

/// The one constructor of a struct that declares no field (`struct S;`):
/// LexLean's structure holds at least one field, so such a struct is the
/// inductive of one constructor, named as Lean names a structure's own.
const SOLE: &str = "mk";

/// The LexLean name of a struct's field: a tuple struct's, which Rust
/// names by position, as a generated tuple's item is named. A position is
/// read as a `u32`, a width fixed on every target: rustc numbers a
/// struct's fields by `FieldIdx`, a 32-bit index, so every position Rust
/// can write fits it, and the reading holds whatever `usize` is.
fn field_name(rust: &str) -> String {
    match rust.parse::<u32>() {
        Ok(position) => format!("item{position}"),
        Err(_) => local_name(rust),
    }
}

/// A type whose name LexLean or the projection writes for a type of its own.
const TAKEN_NAME: &str = "a type named as LexLean or the projection names one of its own";

/// A variant whose name is a word LexLean or Lean reserves.
const TAKEN_VARIANT: &str = "a variant named as a word LexLean reserves";

/// A recursive occurrence Lean does not nest an inductive under.
const NOT_NESTED: &str =
    "a recursion through a tuple, a map or a set, where Lean nests no inductive";

/// A struct with a type or const parameter: one declaration, a type per
/// instance.
pub(crate) const PARAMETRIC_STRUCT: &str = "a struct with type or const parameters";

/// A name two declarations of one module share: one type per build
/// configuration, or an item of an inline module read at its file's.
pub(crate) const DECLARED_TWICE: &str = "a type declared twice in one module";

/// A name several types of a crate share, where the file that writes it
/// neither declares nor imports one of them.
pub(crate) const SHARED_NAME: &str =
    "one of several types of this name in its crate, which the file does not tell apart";

/// A struct the reader of structs does not read as fields, or a name that
/// reads as no one type, registered so that the type side names it: under
/// no field name, with its shape where a field's type stands. A shape is
/// no type, so the name stays outside the lift and is itself the leaf of
/// every chain through it.
pub(crate) fn unread_shape(shape: &str) -> (String, rs::RustType) {
    let shape = rs::RustType {
        text: shape.to_string(),
        kind: rs::TypeKind::Other,
    };
    (String::new(), shape)
}

/// Where the standard library declares each type the lift reads that a
/// source may name by its full path: the module, under `std` (or `core`
/// or `alloc`, which `std` re-exports them from), and the names.
const LIBRARY: [(&str, &[&str]); 10] = [
    ("boxed", &["Box"]),
    (
        "collections",
        &["BTreeMap", "BTreeSet", "HashMap", "HashSet", "VecDeque"],
    ),
    ("ops", &["Range"]),
    ("option", &["Option"]),
    ("path", &["Path", "PathBuf"]),
    ("rc", &["Rc"]),
    ("result", &["Result"]),
    ("string", &["String"]),
    ("sync", &["Arc"]),
    ("vec", &["Vec"]),
];

/// Whether a path is the standard library's own full path of a type the
/// lift reads: `std::collections::BTreeMap`, `std::rc::Rc`.
fn std_path(path: &rs::Path) -> bool {
    let [root, module, last] = path.segments.as_slice() else {
        return false;
    };
    let plain = |s: &rs::PathSegment| matches!(s.arguments, rs::PathArguments::None);
    matches!(root.ident.as_str(), "std" | "core" | "alloc")
        && plain(root)
        && plain(module)
        && LIBRARY
            .iter()
            .any(|(m, names)| *m == module.ident && names.contains(&last.ident.as_str()))
}

/// Whether a generic argument, as written, is a lifetime (`'a`, `'_`): a
/// character literal, a const argument, closes its quote.
fn is_lifetime(argument: &str) -> bool {
    match argument.trim().strip_prefix('\'') {
        Some(name) => !name.contains('\''),
        None => false,
    }
}

/// A field of a variant as Rust writes its place: `Enum::Variant.name`, or
/// `Enum::Variant.0` by position.
fn variant_field(owner: &str, variant: &str, field: Option<&str>, at: usize) -> String {
    match field {
        Some(f) => format!("{owner}::{variant}.{f}"),
        None => format!("{owner}::{variant}.{at}"),
    }
}

/// What keeps a field of a type of the workspace outside the lift.
enum Stop {
    /// A leaf, by its text: a type no part of the workspace's types.
    Leaf(String),
    /// Another type of the workspace that is itself outside, by name.
    Type(String),
}

/// Marks, in the text of a type residue, a type of the workspace whose
/// chain is still to be built: a character no type's text holds.
const PENDING: char = '\u{1}';

/// The strongly connected components of a graph, each one set.
pub(crate) fn strong_components(
    graph: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<BTreeSet<String>> {
    let reach = |start: &String| {
        let mut seen = BTreeSet::new();
        let mut stack: Vec<&String> = graph.get(start).into_iter().flatten().collect();
        while let Some(n) = stack.pop() {
            if seen.insert(n.clone()) {
                stack.extend(graph.get(n).into_iter().flatten());
            }
        }
        seen
    };
    let reaches: BTreeMap<&String, BTreeSet<String>> =
        graph.keys().map(|n| (n, reach(n))).collect();
    let mut out: Vec<BTreeSet<String>> = Vec::new();
    for n in graph.keys() {
        if out.iter().any(|c| c.contains(n)) {
            continue;
        }
        let mut family: BTreeSet<String> = reaches[n]
            .iter()
            .filter(|m| reaches.get(m).is_some_and(|r| r.contains(n)))
            .cloned()
            .collect();
        family.insert(n.clone());
        out.push(family);
    }
    out
}

/// Whether every occurrence of a member of `family` in `ty` is strictly
/// positive and nested only where Lean nests an inductive: directly, or
/// under `Option`, `List`, `Result` or a product.
fn positive(ty: &Ty, family: &BTreeSet<String>) -> bool {
    match ty {
        Ty::Option(t) | Ty::List(t) => positive(t, family),
        Ty::Outcome(t, e) => positive(t, family) && positive(e, family),
        Ty::Pair(ts) => ts.iter().all(|t| positive(t, family)),
        Ty::Tuple(_) | Ty::Map(..) | Ty::Set(..) => {
            let inner = ty.mentions();
            named_in(&inner).is_disjoint(family)
        }
        _ => true,
    }
}

/// A field of a data-carrying variant: its Rust name, if any, and its type.
pub(crate) type DataField = (Option<String>, Ty);

/// A variant of a data-carrying enum and its fields, as Rust writes them.
type RawVariant = (String, Vec<(Option<String>, rs::RustType)>);

/// The fields of each variant of a data-carrying enum, as Rust writes them.
pub(crate) type RawVariants = Vec<RawVariant>;

/// Whether `ty` names `me` only where Lean accepts a nested occurrence:
/// directly, or inside `Option`, `List` or `Result`.
fn nested_ok(ty: &Ty, me: &str) -> bool {
    match ty {
        Ty::Option(t) | Ty::List(t) => nested_ok(t, me),
        Ty::Outcome(t, e) => nested_ok(t, me) && nested_ok(e, me),
        Ty::Pair(ts) => ts.iter().all(|t| nested_ok(t, me)),
        Ty::Tuple(_) | Ty::Map(..) | Ty::Set(..) => {
            let inner = ty.mentions();
            !inner.contains(&Ty::Named(me.to_string()))
        }
        _ => true,
    }
}

/// The suffix of the size function of a type of a nested family: the
/// member's name, behind `list_` or `option_` for a container around it.
fn size_suffix(ty: &Ty, family: &BTreeSet<String>) -> Option<String> {
    match ty {
        Ty::Named(n) if family.contains(n) => Some(n.clone()),
        // LexLean measures a member, or a list or an option of one: a
        // container of a container is outside every recursive family.
        Ty::List(t) => match &**t {
            Ty::Named(n) if family.contains(n) => Some(format!("list_{n}")),
            _ => None,
        },
        Ty::Option(t) => match &**t {
            Ty::Named(n) if family.contains(n) => Some(format!("option_{n}")),
            _ => None,
        },
        _ => None,
    }
}

/// The size domains with a field type's added: the containers inside it
/// first, then itself, where it is a container that has a size.
fn with_domains(ty: &Ty, family: &BTreeSet<String>, domains: Vec<Ty>) -> Vec<Ty> {
    let domains = match ty {
        Ty::List(t) | Ty::Option(t) => with_domains(t, family, domains),
        _ => domains,
    };
    if matches!(ty, Ty::Named(_)) || size_suffix(ty, family).is_none() || domains.contains(ty) {
        return domains;
    }
    let mut domains = domains;
    domains.push(ty.clone());
    domains
}

/// Whether LexLean or the projection writes this name for a type of its
/// own, so that no type of the workspace lifts under it. `declared` is a
/// struct or a data-carrying enum, whose declaration may not be named as
/// the tuple structures the projection generates are. The projection lifts
/// such a type under its crate's name and its own.
pub(crate) fn name_taken(name: &str, declared: bool) -> bool {
    let generated = declared && name.starts_with("Tuple");
    BARE_TYPES.contains(&name) || name.starts_with("LexLean") || generated
}

/// Why a type is outside the lift whatever it holds, known before any type
/// is read: the chain that names it, where one does. Its own name, or a
/// variant's, is one LexLean writes; or the reader of structs did not read
/// it as fields (see [`unread_shape`]), and the struct is the leaf.
fn barred(
    name: &str,
    declared: bool,
    variants: &[&str],
    fields: &[(String, rs::RustType)],
) -> Option<String> {
    if let [(unnamed, shape)] = fields {
        if unnamed.is_empty() {
            return Some(format!("{name}: {}", shape.text));
        }
    }
    if name_taken(name, declared) {
        return Some(format!("{name}: {TAKEN_NAME}"));
    }
    variants
        .iter()
        .find(|v| RESERVED.contains(v))
        .map(|v| format!("{name}::{v}: {TAKEN_VARIANT}"))
}

impl Types {
    /// The lifted types with data-carrying enums among them: every type to
    /// the fixpoint where each names only types that lift, and no two name
    /// each other (a data enum may name itself). A type the fixpoint drops
    /// is recorded in `outside` with the chain down to its leaf (see
    /// [`Types::chains`]).
    pub(crate) fn with_data(
        enums: BTreeMap<String, Vec<String>>,
        datas: &BTreeMap<String, RawVariants>,
        structs: &BTreeMap<String, Vec<(String, rs::RustType)>>,
        aliases: BTreeMap<String, String>,
    ) -> Self {
        let base = Self::new(enums, structs, aliases);
        // What a name or a variant alone keeps out, whatever the type
        // holds. The structs' chains of `new` are not taken: it read them
        // without the data-carrying enums.
        let mut outside: BTreeMap<String, String> = base
            .outside
            .iter()
            .filter(|(n, _)| !structs.contains_key(*n))
            .map(|(n, chain)| (n.clone(), chain.clone()))
            .collect();
        for (n, variants) in datas {
            let known = base.enums.contains_key(n) || base.structs.contains_key(n);
            let named: Vec<&str> = variants.iter().map(|(v, _)| v.as_str()).collect();
            if let (false, Some(why)) = (known, barred(n, true, &named, &[])) {
                outside.insert(n.clone(), why);
            }
        }
        for (n, fields) in structs {
            let barred = barred(n, true, &[], fields);
            if let (false, Some(why)) = (base.enums.contains_key(n), barred) {
                outside.insert(n.clone(), why);
            }
        }
        let mut names: BTreeSet<String> = datas
            .keys()
            .filter(|n| {
                !base.enums.contains_key(*n)
                    && !base.structs.contains_key(*n)
                    && !outside.contains_key(*n)
                    && !datas[*n].is_empty()
            })
            .cloned()
            .collect();
        // The structs `new` may name: not an enum, and not a name generated
        // Lean writes bare or that the projection generates.
        let mut struct_names: BTreeSet<String> = structs
            .keys()
            .filter(|n| !base.enums.contains_key(*n) && !outside.contains_key(*n))
            .cloned()
            .collect();
        let candidates: BTreeSet<String> = names.union(&struct_names).cloned().collect();
        loop {
            let probe = Self {
                datas: names.iter().map(|n| (n.clone(), Vec::new())).collect(),
                structs: struct_names
                    .iter()
                    .filter(|n| !base.enums.contains_key(*n))
                    .map(|n| (n.clone(), Vec::new()))
                    .collect(),
                outside: outside.clone(),
                ..base.clone()
            };
            // The families this round refuses, each member with its chain.
            let mut refused: BTreeMap<String, String> = BTreeMap::new();
            let mut lifted = BTreeMap::new();
            for name in &names {
                if let Some(variants) = probe.variants_read(name, &datas[name]) {
                    lifted.insert(name.clone(), variants);
                }
            }
            let mut fields_of = BTreeMap::new();
            for name in &struct_names {
                if let Some(fields) = structs.get(name).and_then(|raw| probe.fields_read(raw)) {
                    fields_of.insert(name.clone(), fields);
                }
            }
            // The named types each type mentions, and those on a cycle
            // through another type, or a structure through itself.
            let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            for (name, variants) in &lifted {
                let mut out = BTreeSet::new();
                for (_, fields) in variants {
                    for (_, t) in fields {
                        out.extend(t.mentions());
                    }
                }
                graph.insert(name.clone(), named_in(&out));
            }
            for (name, fields) in &fields_of {
                let mut out = BTreeSet::new();
                for (_, t) in fields {
                    out.extend(t.mentions());
                }
                graph.insert(name.clone(), named_in(&out));
            }
            // A family of types that name each other is one mutual group,
            // where every occurrence of a member is strictly positive: held
            // directly or under `Option`, `Vec` or `Result`. One through a
            // generated tuple structure, a map or a set is refused.
            let mut cyclic = BTreeSet::new();
            let mut families = Vec::new();
            for family in strong_components(&graph) {
                let recursive = family.len() > 1 || family.iter().all(|n| graph[n].contains(n));
                if !recursive {
                    continue;
                }
                // Each member's fields: where Rust writes it, as written
                // and as read, in declaration order.
                let places = |n: &String| {
                    let of_variants = lifted
                        .get(n)
                        .into_iter()
                        .flatten()
                        .zip(datas.get(n).into_iter().flatten())
                        .flat_map(|((v, tys), (_, raws))| {
                            tys.iter()
                                .zip(raws)
                                .enumerate()
                                .map(|(i, ((_, ty), (f, raw)))| {
                                    (variant_field(n, v, f.as_deref(), i), raw, ty.clone())
                                })
                        });
                    let of_fields = fields_of
                        .get(n)
                        .into_iter()
                        .flatten()
                        .zip(structs.get(n).into_iter().flatten())
                        .map(|((_, ty), (f, raw))| (format!("{n}.{f}"), raw, ty.clone()));
                    of_variants.chain(of_fields).collect::<Vec<_>>()
                };
                let held = family
                    .iter()
                    .flat_map(places)
                    .find(|(_, _, ty)| !positive(ty, &family));
                match held {
                    None => families.push(family),
                    Some((at, raw, _)) => {
                        let chain = format!("{at}: {}: {NOT_NESTED}", raw.text);
                        for member in &family {
                            // The member that holds the occurrence states
                            // it; the others are in its family.
                            let own = at.starts_with(&format!("{member}."))
                                || at.starts_with(&format!("{member}::"));
                            let why = if own {
                                chain.clone()
                            } else {
                                format!("{member}: in one recursive family with {chain}")
                            };
                            refused.insert(member.clone(), why);
                        }
                        cyclic.extend(family);
                    }
                }
            }
            lifted.retain(|n, _| !cyclic.contains(n));
            fields_of.retain(|n, _| !cyclic.contains(n));
            if lifted.len() == names.len() && fields_of.len() == struct_names.len() {
                // This round's probe names exactly the types that lift.
                let pending = candidates
                    .iter()
                    .filter(|n| !probe.datas.contains_key(*n) && !probe.structs.contains_key(*n))
                    .filter(|n| !outside.contains_key(*n))
                    .cloned()
                    .collect();
                outside.extend(probe.chains(&pending, datas, structs));
                let cells = probe.cells_of(&fields_of, structs);
                return Self {
                    datas: lifted,
                    structs: fields_of,
                    families,
                    outside,
                    cells,
                    ..base
                };
            }
            outside.extend(refused);
            names = lifted.into_keys().collect();
            struct_names = fields_of.into_keys().collect();
        }
    }

    /// The chain of each type the fixpoint dropped, down to its leaf. From
    /// the type, the first field in declaration order that is outside the
    /// lift and under which a leaf lies without passing through a type
    /// already on the chain; then the same from the type that field names,
    /// to the leaf. A chain so never leads back into a type it explains:
    /// of two types each outside through the other, each names what keeps
    /// it out on its own account. `self` names the types that lift, and
    /// holds in `outside` the chains already known; `pending` are the
    /// dropped types.
    fn chains(
        &self,
        pending: &BTreeSet<String>,
        datas: &BTreeMap<String, RawVariants>,
        structs: &BTreeMap<String, Vec<(String, rs::RustType)>>,
    ) -> BTreeMap<String, String> {
        let mut probe = self.clone();
        for n in pending {
            probe.outside.insert(n.clone(), format!("{PENDING}{n}"));
        }
        let stop = |at: String, read: Lift<Ty>, written: &rs::RustType, owner: &str| {
            let stop = match read {
                Ok(ty) if nested_ok(&ty, owner) => return None,
                Ok(_) => Stop::Leaf(format!("{}: {NOT_NESTED}", written.text)),
                Err(Residue::Type(text)) => match text.strip_prefix(PENDING) {
                    Some(name) => Stop::Type(name.to_string()),
                    None => Stop::Leaf(text),
                },
                Err(other) => Stop::Leaf(other.detail()),
            };
            Some((at, stop))
        };
        // Each dropped type's fields that are outside, in declaration
        // order: where Rust writes the field, and what stops it. The type
        // itself is read as known there: a field that holds it is outside
        // on the type's own account only where Lean nests no inductive.
        let stops: BTreeMap<&String, Vec<(String, Stop)>> = pending
            .iter()
            .map(|n| {
                let mut own = probe.clone();
                own.outside.remove(n);
                if datas.contains_key(n) {
                    own.datas.insert(n.clone(), Vec::new());
                } else {
                    own.structs.insert(n.clone(), Vec::new());
                }
                let of_variants = datas.get(n).into_iter().flatten().flat_map(|(v, fields)| {
                    let own = &own;
                    fields.iter().enumerate().filter_map(move |(i, (f, t))| {
                        stop(variant_field(n, v, f.as_deref(), i), own.ty(t), t, n)
                    })
                });
                let of_fields = structs
                    .get(n)
                    .into_iter()
                    .flatten()
                    .filter_map(|(f, t)| stop(format!("{n}.{f}"), own.field_ty(t), t, ""));
                (n, of_variants.chain(of_fields).collect())
            })
            .collect();
        // Whether a leaf lies under `from` without passing through `on`.
        let grounded = |from: &String, on: &BTreeSet<&String>| {
            let mut seen: BTreeSet<&String> = on.clone();
            let mut next = vec![from];
            while let Some(n) = next.pop() {
                if !seen.insert(n) {
                    continue;
                }
                for (_, stop) in stops.get(n).into_iter().flatten() {
                    match stop {
                        Stop::Leaf(_) => return true,
                        Stop::Type(inner) => next.push(inner),
                    }
                }
            }
            false
        };
        let mut chains = BTreeMap::new();
        for start in pending {
            let mut on: BTreeSet<&String> = BTreeSet::from([start]);
            let mut at = start;
            let mut chain = String::new();
            loop {
                let held = stops.get(at).into_iter().flatten().find(|(_, s)| match s {
                    Stop::Leaf(_) => true,
                    Stop::Type(inner) => !on.contains(inner) && grounded(inner, &on),
                });
                match held {
                    Some((place, Stop::Leaf(leaf))) => {
                        chain.push_str(&format!("{place}: {leaf}"));
                        break;
                    }
                    Some((place, Stop::Type(inner))) => {
                        chain.push_str(&format!("{place}: "));
                        on.insert(inner);
                        at = inner;
                    }
                    // Every dropped type has a leaf under it: the round
                    // that dropped it read one, or a type dropped before.
                    None => {
                        chain.push_str(at);
                        break;
                    }
                }
            }
            chains.insert(start.clone(), chain);
        }
        chains
    }

    /// A struct's fields as the lift names them, where every one lifts. A
    /// tuple struct's field is named by its position.
    fn fields_read(&self, raw: &[(String, rs::RustType)]) -> Option<Vec<(String, Ty)>> {
        raw.iter()
            .map(|(field, t)| Some((field_name(field), self.field_ty(t).ok()?)))
            .collect()
    }

    /// The cell a written type is, of the standard library (by its full
    /// path, or by its name where the workspace declares none), and the
    /// type it holds.
    fn cell_of<'t>(&self, t: &'t rs::RustType) -> Option<(CellKind, &'t rs::RustType)> {
        let rs::TypeKind::Path { qself: false, path } = &t.kind else {
            return None;
        };
        let last = path.segments.last()?;
        let kind = CellKind::named(&last.ident)?;
        let module = match kind {
            CellKind::RefCell | CellKind::Cell => "cell",
            CellKind::Mutex | CellKind::RwLock => "sync",
        };
        let full = match path.segments.as_slice() {
            [root, m, _] => {
                matches!(root.ident.as_str(), "std" | "core" | "alloc")
                    && m.ident == module
                    && root.arguments == rs::PathArguments::None
                    && m.arguments == rs::PathArguments::None
            }
            _ => false,
        };
        let library = full || (path.segments.len() == 1 && !self.declares(&last.ident));
        let rs::PathArguments::AngleBracketed(args) = &last.arguments else {
            return None;
        };
        match args.as_slice() {
            [rs::GenericArgument::Type(inner)] if library => Some((kind, inner)),
            _ => None,
        }
    }

    /// The type of a struct's field: a cell's is the content it holds.
    fn field_ty(&self, t: &rs::RustType) -> Lift<Ty> {
        match self.cell_of(t) {
            Some((_, inner)) => self.ty(inner),
            None => self.ty(t),
        }
    }

    /// The cell fields of these lifted structs (see [`Types::cells`]).
    fn cells_of(
        &self,
        lifted: &BTreeMap<String, Vec<(String, Ty)>>,
        structs: &BTreeMap<String, Vec<(String, rs::RustType)>>,
    ) -> BTreeMap<String, BTreeMap<String, CellKind>> {
        lifted
            .keys()
            .filter_map(|n| {
                let cells: BTreeMap<String, CellKind> = structs
                    .get(n)?
                    .iter()
                    .filter_map(|(f, t)| Some((field_name(f), self.cell_of(t)?.0)))
                    .collect();
                (!cells.is_empty()).then(|| (n.clone(), cells))
            })
            .collect()
    }

    /// The cell a lifted struct's field is, if any.
    pub(crate) fn cell(&self, name: &str, field: &str) -> Option<CellKind> {
        self.cells.get(name)?.get(field).copied()
    }

    /// A data-carrying enum's variants as the lift names them, where every
    /// field lifts and holds the enum itself only where Lean nests an
    /// inductive.
    fn variants_read(
        &self,
        name: &str,
        raw: &[RawVariant],
    ) -> Option<Vec<(String, Vec<DataField>)>> {
        let read = raw
            .iter()
            .filter_map(|(variant, fields)| {
                let tys: Vec<(Option<String>, Ty)> = fields
                    .iter()
                    .filter_map(|(field, t)| {
                        let ty = self.ty(t).ok().filter(|ty| nested_ok(ty, name));
                        ty.map(|ty| (field.clone(), ty))
                    })
                    .collect();
                (tys.len() == fields.len()).then_some((variant.clone(), tys))
            })
            .collect::<Vec<_>>();
        (read.len() == raw.len()).then_some(read)
    }

    /// The structs whose every field lifts, to the fixpoint where a struct
    /// only names enums and structs that lift.
    /// Rust type names `aliases` maps to the LexLean names the maps are
    /// keyed by: a crate's own names, apart from another crate's.
    pub(crate) fn new(
        enums: BTreeMap<String, Vec<String>>,
        structs: &BTreeMap<String, Vec<(String, rs::RustType)>>,
        aliases: BTreeMap<String, String>,
    ) -> Self {
        let mut outside: BTreeMap<String, String> = enums
            .iter()
            .filter_map(|(n, vs)| {
                let named: Vec<&str> = vs.iter().map(String::as_str).collect();
                Some((n.clone(), barred(n, false, &named, &[])?))
            })
            .collect();
        let mut enums: BTreeMap<String, Vec<String>> = enums
            .into_iter()
            .filter(|(n, _)| !outside.contains_key(n))
            .collect();
        let mut names = BTreeSet::new();
        for (n, fields) in structs {
            if enums.contains_key(n) {
                continue;
            }
            match barred(n, true, &[], fields) {
                Some(why) => {
                    outside.insert(n.clone(), why);
                }
                // A struct of no fields has one value: the inductive of
                // one constructor.
                None if fields.is_empty() => {
                    enums.insert(n.clone(), vec![SOLE.to_string()]);
                }
                None => {
                    names.insert(n.clone());
                }
            }
        }
        let candidates = names.clone();
        loop {
            let probe = Self {
                enums: enums.clone(),
                structs: names.iter().map(|n| (n.clone(), Vec::new())).collect(),
                aliases: aliases.clone(),
                outside: outside.clone(),
                ..Self::default()
            };
            let mut lifted = BTreeMap::new();
            for name in &names {
                if let Some(fields) = probe.fields_read(&structs[name]) {
                    lifted.insert(name.clone(), fields);
                }
            }
            if lifted.len() == names.len() {
                let pending = candidates.difference(&names).cloned().collect();
                outside.extend(probe.chains(&pending, &BTreeMap::new(), structs));
                let cells = probe.cells_of(&lifted, structs);
                return Self {
                    enums,
                    structs: lifted,
                    aliases,
                    outside,
                    cells,
                    ..Self::default()
                };
            }
            names = lifted.into_keys().collect();
        }
    }

    /// The type of a tuple of these items: a product when it is a pair that
    /// names a data-carrying enum, a generated structure otherwise.
    pub(crate) fn tuple(&self, ts: Vec<Ty>) -> Ty {
        let mut inner = BTreeSet::new();
        for t in &ts {
            inner.extend(t.mentions());
        }
        let data = named_in(&inner).iter().any(|n| self.datas.contains_key(n));
        if (ts.len() == 2 && data) || ts.iter().any(|t| !t.parameters().is_empty()) {
            Ty::Pair(ts)
        } else {
            Ty::Tuple(ts)
        }
    }

    /// The recursive family a named type belongs to.
    pub(crate) fn family_of(&self, name: &str) -> Option<&BTreeSet<String>> {
        self.families.iter().find(|f| f.contains(name))
    }

    /// A struct of a recursive family: declared as an inductive of one
    /// constructor `mk`, its fields read through generated accessors, since
    /// a structure cannot join a mutual group.
    pub(crate) fn boxed(&self, name: &str) -> bool {
        self.structs.contains_key(name) && self.family_of(name).is_some()
    }

    /// The field types of a member of a family, over all its constructors.
    pub(crate) fn member_fields(&self, name: &str) -> Vec<Ty> {
        let data = self.datas.get(name).into_iter().flatten();
        let mut out: Vec<Ty> = data
            .flat_map(|(_, fs)| fs.iter().map(|(_, t)| t.clone()))
            .collect();
        out.extend(
            self.structs
                .get(name)
                .into_iter()
                .flatten()
                .map(|(_, t)| t.clone()),
        );
        out
    }

    /// A type Lean calls nested or mutual: its family has more than one
    /// member, or a member occurs under a container. A function recursing
    /// on one is well-founded on its size, not structural.
    pub(crate) fn nested(&self, name: &str) -> bool {
        let Some(family) = self.family_of(name) else {
            return false;
        };
        family.len() > 1
            || family.iter().any(|m| {
                self.member_fields(m).iter().any(|t| {
                    !matches!(t, Ty::Named(_)) && {
                        let inner = t.mentions();
                        !named_in(&inner).is_disjoint(family)
                    }
                })
            })
    }

    /// The size function of a type of a nested family: the members, and
    /// lists and options of types that have one. `None` for any other type.
    pub(crate) fn size_name(&self, ty: &Ty, family: &BTreeSet<String>) -> Option<String> {
        size_suffix(ty, family).map(|s| format!("size_of_{s}"))
    }

    /// The types the sizes of a family range over: the members, then each
    /// list or option the members hold whose items have a size, after the
    /// ones inside it.
    pub(crate) fn size_domains(&self, family: &BTreeSet<String>) -> Vec<Ty> {
        let fields: Vec<Ty> = family.iter().flat_map(|m| self.member_fields(m)).collect();
        let members: Vec<Ty> = family.iter().map(|m| Ty::Named(m.clone())).collect();
        fields
            .iter()
            .fold(members, |domains, t| with_domains(t, family, domains))
    }

    /// Whether a nested family has a structural size: every occurrence of a
    /// member is direct, or the whole element of a list or an option.
    pub(crate) fn sizable(&self, family: &BTreeSet<String>) -> bool {
        family.iter().all(|m| {
            self.member_fields(m).iter().all(|t| {
                let inner = t.mentions();
                named_in(&inner).is_disjoint(family) || self.size_name(t, family).is_some()
            })
        })
    }

    /// The accessor of a field of a struct of a recursive family.
    pub(crate) fn accessor(name: &str, field: &str) -> String {
        format!("field_{name}_{field}")
    }

    /// The LexLean name of a Rust type name. A lifetime argument is erased,
    /// as where a type is read: the owner `Reader<'a>` of a method names
    /// the struct `Reader`.
    pub(crate) fn resolve(&self, rust: &str) -> String {
        let bare = match rust.split_once('<') {
            Some((head, arguments)) => match arguments.strip_suffix('>') {
                Some(list) if list.split(',').all(is_lifetime) => head.trim_end(),
                _ => rust,
            },
            None => rust,
        };
        self.aliases
            .get(bare)
            .cloned()
            .unwrap_or_else(|| bare.to_string())
    }

    /// Resolve model dependencies against declarations, never a rendered name
    /// or a namespace prefix. Fixed prelude bytes are already content-bound in
    /// the projection's semantic context and remain subject to SDK acceptance.
    pub(crate) fn model_declared(&self, ty: &Ty) -> bool {
        static PRELUDE_TYPES: std::sync::OnceLock<Option<BTreeSet<String>>> =
            std::sync::OnceLock::new();
        let prelude = PRELUDE_TYPES.get_or_init(|| {
            crate::lex::type_names(PRELUDE_SOURCE).ok().map(|names| {
                names
                    .into_iter()
                    .map(|name| format!("{PRELUDE}::{name}"))
                    .collect()
            })
        });
        ty.mentions().iter().all(|mentioned| match mentioned {
            Ty::Named(name) => {
                self.structs.contains_key(name)
                    || self.enums.contains_key(name)
                    || self.datas.contains_key(name)
                    || prelude.as_ref().is_some_and(|names| names.contains(name))
            }
            _ => true,
        })
    }

    /// Whether the workspace declares a type of this Rust name, inside the
    /// lift or outside it: a name of the standard library that is not in
    /// the prelude (`Rc`, `Range`) is the library's only where it does not.
    fn declares(&self, rust: &str) -> bool {
        let lex = self.resolve(rust);
        self.enums.contains_key(&lex)
            || self.structs.contains_key(&lex)
            || self.datas.contains_key(&lex)
            || self.outside.contains_key(&lex)
    }

    /// The residue of a type the lift does not name, by its Rust name: the
    /// chain down to its leaf where the workspace declares it, the name as
    /// written otherwise.
    pub(crate) fn unnamed(&self, rust: &str) -> Residue {
        let chain = self.outside.get(&self.resolve(rust));
        Residue::Type(chain.cloned().unwrap_or_else(|| rust.to_string()))
    }

    pub(crate) fn ty(&self, t: &rs::RustType) -> Lift<Ty> {
        let bad = || Err(Residue::Type(t.text.clone()));
        match &t.kind {
            rs::TypeKind::Reference {
                mutable: false,
                elem,
            } => match &elem.kind {
                rs::TypeKind::Path { qself: false, path } if path.is_ident("str") => Ok(Ty::Str),
                rs::TypeKind::Slice(s) => Ok(Ty::List(Box::new(self.ty(s)?))),
                _ => self.ty(elem),
            },
            rs::TypeKind::Paren(p) => self.ty(p),
            rs::TypeKind::Fn(ins, out) => {
                let mut inputs = Vec::new();
                for t in ins {
                    inputs.push(self.ty(t)?);
                }
                Ok(Ty::Fn(inputs, Box::new(self.ty(out)?)))
            }
            // An array is the list of its items: its length is the
            // literal's, which the value fixes.
            rs::TypeKind::Array(t, _) => Ok(Ty::List(Box::new(self.ty(t)?))),
            // Slice storage is its ordered sequence, including under an
            // owning pointer. Mutable cell elements still fail their own
            // type reading instead of becoming frozen snapshots.
            rs::TypeKind::Slice(t) => Ok(Ty::List(Box::new(self.ty(t)?))),
            rs::TypeKind::Tuple(elems) if elems.is_empty() => Ok(Ty::Unit),
            rs::TypeKind::Tuple(elems) if elems.len() >= 2 => {
                let mut ts = Vec::new();
                for e in elems {
                    ts.push(self.ty(e)?);
                }
                Ok(self.tuple(ts))
            }
            // A type of the standard library written by its full path
            // (`std::collections::BTreeMap<K, V>`): the library's, whatever
            // the workspace declares under the name.
            rs::TypeKind::Path { qself: false, path } if std_path(path) => {
                match path.segments.last() {
                    Some(last) => self.named(t, last, true),
                    None => bad(),
                }
            }
            rs::TypeKind::Path { qself: false, path } if path.segments.len() == 1 => {
                self.named(t, &path.segments[0], false)
            }
            // A registered crate's type by its path (`num_bigint::BigInt`).
            rs::TypeKind::Path { qself: false, path }
                if path
                    .segments
                    .iter()
                    .all(|s| s.arguments == rs::PathArguments::None) =>
            {
                let written: Vec<&str> = path.segments.iter().map(|s| s.ident.as_str()).collect();
                let joined = written.join("::");
                if self.paths_open && !joined.starts_with("::") {
                    return bad();
                }
                self.registered(joined.strip_prefix("::").unwrap_or(&joined))
                    .map_or_else(bad, Ok)
            }
            _ => bad(),
        }
    }

    /// The type a registered crate's path reads as, where the scope admits
    /// the reading: the package's lock pins a version it was read from
    /// (see [`crate::registered::of_package`]), which puts the path among
    /// the scope's names.
    pub(crate) fn float(&self, name: &str) -> Option<crate::mir::float::Format> {
        use crate::mir::float::Format;
        // The width of a float is that of the unsigned integer its
        // `to_bits` returns; its digits and largest exponent are core's
        // `MANTISSA_DIGITS` and `MAX_EXP`, as the core the tool is built
        // with states them. The language fixes both types as IEEE 754's
        // binary32 and binary64, so no build of core states others.
        let host = |bytes: usize, digits: u32, max_exp: i32| {
            let bits = u8::BITS * u32::try_from(bytes).ok()?;
            Format::from_core(bits, digits, i64::from(max_exp))
        };
        match self.floats.get(name) {
            Some(f) => Some(*f),
            None => match name {
                "f32" => host(
                    std::mem::size_of::<f32>(),
                    f32::MANTISSA_DIGITS,
                    f32::MAX_EXP,
                ),
                "f64" => host(
                    std::mem::size_of::<f64>(),
                    f64::MANTISSA_DIGITS,
                    f64::MAX_EXP,
                ),
                _ => None,
            },
        }
    }

    /// The type a registered crate's path reads as (see below).
    pub(crate) fn registered(&self, path: &str) -> Option<Ty> {
        self.aliases
            .get(path)
            .filter(|named| *named == path)
            .and_then(|_| crate::registered::reading(path))
    }

    /// The type one path segment names, with its arguments: a type of the
    /// standard library the lift reads, or one of the workspace. `library`
    /// is a segment reached by the library's own full path.
    fn named(&self, t: &rs::RustType, seg: &rs::PathSegment, library: bool) -> Lift<Ty> {
        let bad = || Err(Residue::Type(t.text.clone()));
        let name = seg.ident.clone();
        // The type arguments as written, a lifetime erased.
        let mut written = Vec::new();
        match &seg.arguments {
            rs::PathArguments::None => {}
            rs::PathArguments::AngleBracketed(a) => {
                for arg in a {
                    match arg {
                        rs::GenericArgument::Type(t) => written.push(t),
                        rs::GenericArgument::Other(l) if is_lifetime(l) => {}
                        rs::GenericArgument::Other(_) => return bad(),
                    }
                }
            }
            rs::PathArguments::Parenthesized => return bad(),
        }
        // A name of the library outside the prelude (`Rc`, `Range`) is the
        // library's where its full path says so, or where the workspace
        // declares no type of the name.
        let imported = library || !self.declares(&name);
        // A primitive type's name that a type of the crate also takes
        // (`struct f32` in one module) names that type in its module and
        // the primitive elsewhere: the crate-wide tables cannot tell which.
        if PRIMITIVE_TYPES.contains(&name.as_str()) && written.is_empty() && !imported {
            return bad();
        }
        // The head is read before its arguments: a head the lift does not
        // read is the leaf whatever it holds (`RefCell<T>` is outside for
        // every `T`), and only a head it reads passes the question on to
        // what it holds.
        let arg = |i: usize| self.ty(written[i]).map(Box::new);
        if let (Some(t), 0, false) = (self.generic.get(&name), written.len(), library) {
            return Ok(t.clone());
        }
        let ty = match (name.as_str(), written.len()) {
            ("bool", 0) => Ty::Bool,
            ("u8", 0) => Ty::Nat(Some(8)),
            ("u16", 0) => Ty::Nat(Some(16)),
            ("u32", 0) => Ty::Nat(Some(32)),
            ("u64", 0) => Ty::Nat(Some(64)),
            ("u128", 0) => Ty::Nat(Some(128)),
            ("usize", 0) => Ty::Nat(None),
            ("i8", 0) => Ty::Int(8),
            ("i16", 0) => Ty::Int(16),
            ("i32", 0) => Ty::Int(32),
            ("i64", 0) => Ty::Int(64),
            ("i128", 0) => Ty::Integer(Some(128)),
            ("isize", 0) => Ty::Integer(None),
            ("String", 0) => Ty::Str,
            ("char", 0) => Ty::Char,
            // A float of the language: IEEE 754's binary format of the
            // widths core's own constants state (see `Types::float`).
            ("f32" | "f64", 0) if imported => match self.float(&name) {
                Some(f) => Ty::Float(f),
                None => return bad(),
            },
            ("Option", 1) => Ty::Option(arg(0)?),
            ("Result", 2) => Ty::Outcome(arg(0)?, arg(1)?),
            ("Vec", 1) => Ty::List(arg(0)?),
            // An owning pointer reads as what it owns. `Rc` and `Arc`
            // share that ownership, and through either a function reaches
            // only `&T`: what it reads is the content, the same from every
            // owner for as long as any holds it, so a pure function of the
            // pointer is a function of the content. What tells two owners
            // of equal content apart (`ptr_eq`, the counts, `Weak`) is no
            // operation the lift reads. A cell that changes behind a
            // shared borrow (`RefCell`, `Cell`, `Mutex`, `RwLock`, an
            // atomic) is not read: it stays outside, the leaf of every
            // chain through it.
            ("Box", 1) => *arg(0)?,
            // A shared cell is an allocation reference, never its current
            // contents. Transport and owner clones preserve this identity;
            // no heap read, write, allocation or equality is read here.
            ("Rc" | "Arc", 1)
                if imported
                    && self.cell_of(written[0]).is_some_and(|(kind, _)| {
                        matches!(kind, CellKind::RefCell | CellKind::Cell)
                    }) =>
            {
                Ty::Named(format!("Prelude::{name}CellRef"))
            }
            // Concurrent locks still require a synchronization model.
            ("Rc" | "Arc", 1) if imported && self.cell_of(written[0]).is_some() => {
                let kind = self.cell_of(written[0]).map_or("cell", |(k, _)| k.name());
                return Err(Residue::Type(format!(
                    "{}: a {kind} that another owner shares, which sees every change",
                    t.text
                )));
            }
            ("Rc" | "Arc", 1) if imported => *arg(0)?,
            ("BTreeMap", 2) => Ty::Map(arg(0)?, arg(1)?, true),
            ("HashMap", 2) => Ty::Map(arg(0)?, arg(1)?, false),
            // `HashMap<K, V, S>` and `HashSet<T, S>`: the standard map and
            // set with the hasher `S` (the standard library declares both
            // with `S = RandomState`). Which keys a map holds and the value
            // at each is fixed by `K`'s `Eq` alone: the library requires of
            // every `BuildHasher` that the hashers one value builds hash
            // equal input equally, so the hasher changes where an entry is
            // stored and the order iteration visits them, never what
            // `get`, `contains_key`, `contains` or `len` return. The map is
            // read as the unordered one, whose forms the lift takes only
            // where they do not depend on that order, whatever `S` is.
            ("HashMap", 3) => Ty::Map(arg(0)?, arg(1)?, false),
            ("BTreeSet", 1) => Ty::Set(arg(0)?, true),
            ("HashSet", 1) => Ty::Set(arg(0)?, false),
            ("HashSet", 2) => Ty::Set(arg(0)?, false),
            // A double-ended queue is the sequence of its items, front to
            // back, as a vector is.
            ("VecDeque", 1) if imported => Ty::List(arg(0)?),
            // `a..b` as a value: the struct of its two bounds, which the
            // language declares with both public.
            ("Range", 1) if imported => {
                let bound = *arg(0)?;
                self.tuple(vec![bound.clone(), bound])
            }
            // A registered crate's type, where the package's lock admits
            // the reading: the scope names its path.
            (n, 0) if !library && self.registered(&self.resolve(n)).is_some() => {
                self.registered(&self.resolve(n)).unwrap_or(Ty::Unit)
            }
            (n, 0)
                if !library
                    && (self.enums.contains_key(&self.resolve(n))
                        || self.structs.contains_key(&self.resolve(n))
                        || self.datas.contains_key(&self.resolve(n))) =>
            {
                Ty::Named(self.resolve(n))
            }
            // The standard path, where the crate declares no type of the
            // name.
            ("Path" | "PathBuf", 0) => Ty::Path,
            // A type of the workspace outside the lift: the chain down to
            // its leaf. Any other name is itself the leaf.
            (n, _) => {
                return match self.outside.get(&self.resolve(n)).filter(|_| !library) {
                    Some(chain) => Err(Residue::Type(chain.clone())),
                    None => bad(),
                }
            }
        };
        Ok(ty)
    }

    /// The LexLean signature of a candidate, or why it is not one. `owner`
    /// says what its `self` and `Self` name, where it is a method.
    pub(crate) fn signature(
        &self,
        sig: &rs::Signature,
        owner: Option<&Owner<'_>>,
    ) -> Lift<(Vec<Param>, Ty)> {
        // A lifetime parameter says how long a borrow lives, which a value
        // of LexLean does not have: it is erased. A type or const parameter
        // is not.
        if let Some(what) = unread_signature(sig) {
            return Err(Residue::Signature(what));
        }
        // The type `Self` names: none in a default method, where it is each
        // implementing type.
        let named = match owner {
            Some(Owner::Type { name, .. }) => Some(*name),
            Some(Owner::Implementor) | None => None,
        };
        let mut params = Vec::new();
        // The types of the `&mut` parameters, which the definition gives
        // back after its result (see [`threads`]).
        let mut states = Vec::new();
        for input in &sig.inputs {
            match input {
                rs::FnArg::Receiver { plain } => match owner {
                    // A default method that lifts reads nothing of `self`,
                    // which is each implementing type's value: the
                    // definition does not take it.
                    Some(Owner::Implementor) => {}
                    Some(Owner::Type { name, receiver })
                        if self.enums.contains_key(*name)
                            || self.structs.contains_key(*name)
                            || self.datas.contains_key(*name) =>
                    {
                        match (*plain, receiver.map(Receiver::of)) {
                            (true, _) | (false, Some(Receiver::Owned)) => {}
                            (false, Some(Receiver::Unique)) => {
                                states.push(Ty::Named((*name).to_string()));
                            }
                            _ => {
                                return Err(Residue::Signature(format!(
                                    "takes {} of {name}: the lift reads self, mut self, &self \
                                     and &mut self, and no other receiver",
                                    receiver.unwrap_or("self")
                                )));
                            }
                        }
                        params.push((
                            "self".to_string(),
                            local_name("self"),
                            Ty::Named((*name).to_string()),
                        ));
                    }
                    Some(Owner::Type { name, .. }) => {
                        return Err(Residue::Signature(format!(
                            "takes self of {name}, a type outside the lift"
                        )));
                    }
                    None => {
                        return Err(Residue::Signature("takes self outside a type".to_string()));
                    }
                },
                rs::FnArg::Typed(pat) => {
                    let rs::Pat::Ident(name) = &*pat.pat else {
                        return Err(Residue::Signature("pattern parameter".to_string()));
                    };
                    let rust = name.ident.clone();
                    let ty = match unique_target(&pat.ty) {
                        Some(target) => {
                            let ty = self.ty_in(target, named)?;
                            states.push(ty.clone());
                            ty
                        }
                        None => self.ty_in(&pat.ty, named)?,
                    };
                    params.push((rust.clone(), local_name(&rust), ty));
                }
            }
        }
        let result = match &sig.output {
            None => Ty::Unit,
            Some(t) => self.ty_in(t, named)?,
        };
        Ok((
            params,
            crate::mir::lower::threaded_result(&result, &states, self),
        ))
    }

    /// A type where `Self` names the method's owner.
    fn ty_in(&self, t: &rs::RustType, owner: Option<&str>) -> Lift<Ty> {
        let plain = match &t.kind {
            rs::TypeKind::Reference {
                mutable: false,
                elem,
            } => elem,
            _ => t,
        };
        match (&plain.kind, owner) {
            (rs::TypeKind::Path { qself: false, path }, Some(o)) if path.is_ident("Self") => {
                Ok(Ty::Named(o.to_string()))
            }
            _ => self.ty(t),
        }
    }
}

/// Words a LexLean or Lean name may not be: the semantic notation's keywords
/// and Lean's.
const RESERVED: &[&str] = &[
    "at",
    "by",
    "calc",
    "def",
    "do",
    "else",
    "end",
    "example",
    "fun",
    "have",
    "if",
    "in",
    "instance",
    "let",
    "match",
    "mut",
    "nomatch",
    "open",
    "prop",
    "return",
    "show",
    "then",
    "theorem",
    "type",
    "where",
    "with",
    "from",
    "for",
    "try",
    "catch",
    "break",
    "continue",
    "unless",
    "structure",
    "inductive",
    "class",
    "namespace",
    "section",
    "local",
    "private",
    "protected",
    "partial",
    "unsafe",
    "macro",
    "syntax",
    "import",
    "export",
    "axiom",
    "abbrev",
    "opaque",
    "mutual",
    "variable",
    "universe",
    "deriving",
    "extends",
    "suffices",
    "obtain",
    "record",
    "ctor",
    "prim",
    "project",
    "cons",
    "nil",
    "not",
    "true",
    "false",
    "forall",
    "first",
    "second",
    "apply",
    "ref",
    "checked",
    "self",
    "noncomputable",
    "termination_by",
    // Lean names generated code writes bare.
    "and_congr",
    "and_true",
    "beq_iff_eq",
    "congr",
    "decide",
    "decide_eq_false_iff_not",
    "decide_eq_true_eq",
    "id",
    "reduceIte",
    "rfl",
    "true_and",
    // Constructors Lean resolves from the root in a pattern.
    "isFalse",
    "isTrue",
    "none",
    "some",
    // Members Lean declares for every structure.
    "below",
    "brecOn",
    "casesOn",
    "ctorElim",
    "ctorElimType",
    "ctorIdx",
    "mk",
    "noConfusion",
    "noConfusionType",
    "rec",
    "recOn",
    "toCtorIdx",
];

/// Type names generated Lean writes bare: a struct or enum may not take one.
pub(crate) const BARE_TYPES: &[&str] = &[
    "And",
    "BitVec",
    "Bool",
    "ByteArray",
    "ContractViolation",
    "Except",
    "Iff",
    "Int",
    "Int16",
    "Int32",
    "Int64",
    "Int8",
    "List",
    "Nat",
    "Option",
    "Ordering",
    "Prod",
    "Prop",
    "ReasoningFailure",
    "Result",
    "String",
    "Type",
    "UInt16",
    "UInt32",
    "UInt64",
    "UInt8",
    "Unit",
];

/// The LexLean name of a Rust local, parameter or field.
pub(crate) fn local_name(ident: &str) -> String {
    let name = camel(&[ident.trim_start_matches("r#")]);
    if name.is_empty() || RESERVED.contains(&name.as_str()) {
        format!("{name}Value")
    } else {
        name
    }
}

/// A parameter: its Rust name, its LexLean name and its type.
pub(crate) type Param = (String, String, Ty);

/// What `self` and `Self` name in a method's signature.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Owner<'a> {
    /// The type the method is of, by its LexLean name, and the receiver as
    /// the method writes it (`&mut self`), where it takes one.
    Type {
        name: &'a str,
        receiver: Option<&'a str>,
    },
    /// Each type that implements the trait: a default method the trait
    /// declares.
    Implementor,
}

/// A receiver that is not `self` or `&self`, as the method writes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Receiver {
    /// `mut self`: the method owns the value, and its binding is mutable.
    /// Nothing else reaches the value, so a change to it is a new binding
    /// that the method's own later statements read, as for any local.
    Owned,
    /// `&mut self` (`&'a mut self`): a borrow nothing else may use while it
    /// lives, which Rust's borrow checker guarantees of every program. The
    /// place it lends holds, when the method returns, the value the
    /// method's last statement leaves in it, and no one reads the place in
    /// between: the method is the function from the old value to its
    /// result and the new value (see [`threads`]).
    Unique,
    /// `self: T` and any other written receiver.
    Other,
}

impl Receiver {
    pub(crate) fn of(written: &str) -> Receiver {
        let words: Vec<&str> = written.split_whitespace().collect();
        match words.as_slice() {
            ["mut", "self"] => Receiver::Owned,
            [borrow, "mut", "self"] if borrow.starts_with('&') => Receiver::Unique,
            ["&mut", "self"] => Receiver::Unique,
            _ => Receiver::Other,
        }
    }
}

/// The type a `&mut T` parameter lends, where the lift threads it: any `T`
/// but a slice or `str`, whose length the borrow fixes and whose items it
/// changes in place.
fn unique_target(t: &rs::RustType) -> Option<&rs::RustType> {
    match &t.kind {
        rs::TypeKind::Reference {
            mutable: true,
            elem,
        } => match &elem.kind {
            rs::TypeKind::Slice(_) => None,
            rs::TypeKind::Path { qself: false, path } if path.is_ident("str") => None,
            _ => Some(elem),
        },
        _ => None,
    }
}

/// The positions, among a candidate's parameters, of those the definition
/// threads: a `&mut self` receiver and each `&mut T` parameter. As
/// `mir::lower::threaded` does for a body's locals, the definition takes
/// the borrowed value and returns the tuple of its result and, after it,
/// the value each such parameter's place holds when the function returns
/// (`mir::lower::threaded_result`); a caller writes each back into the
/// place it lent. A `&mut` borrow is the only way to the place while it
/// lives, so no other reader sees the place between the call and the
/// write.
pub(crate) fn threads(sig: &rs::Signature, owner: Option<&Owner<'_>>) -> Vec<usize> {
    let mut out = Vec::new();
    let mut at = 0;
    for input in &sig.inputs {
        match input {
            rs::FnArg::Receiver { plain } => {
                if let Some(Owner::Type { receiver, .. }) = owner {
                    if !*plain && receiver.map(Receiver::of) == Some(Receiver::Unique) {
                        out.push(at);
                    }
                    at += 1;
                }
            }
            rs::FnArg::Typed(pat) => {
                if unique_target(&pat.ty).is_some() {
                    out.push(at);
                }
                at += 1;
            }
        }
    }
    out
}

/// A lifted function's signature.
#[derive(Clone, Debug)]
pub(crate) struct Sig {
    pub(crate) name: String,
    pub(crate) params: Vec<Ty>,
    pub(crate) result: Ty,
    /// Its type parameters and its dictionary, where it is generic.
    pub(crate) generic: Option<Generic>,
    /// The positions of the parameters it threads (see [`threads`]):
    /// `result` is then the tuple of its own result and their values.
    pub(crate) threads: Vec<usize>,
}

/// What a function generic over type parameters takes besides its own
/// parameters: each type parameter, and for each method of a trait that
/// bounds one and that its body calls, the function the implementing
/// type supplies (its dictionary).
///
/// Rust compiles a generic function once per instantiation, and in each
/// a call `x.m(..)` on a value of a type parameter `T: Tr` is the method
/// `m` of the one impl of `Tr` for the type that takes `T`'s place
/// (coherence admits no second impl of a trait without parameters for a
/// type). The function of the method's receiver and arguments that impl
/// gives is therefore an argument the instantiation fixes: the lifted
/// definition takes it as a parameter and applies it where the body calls
/// `m`, and each call passes the impl's method of the type its arguments
/// fix. A body that reaches a value of `T` other than through the
/// dictionary can only move it, since `T` has no other bound (other
/// bounds keep their row): a definition over `'T` does the same for every
/// type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Generic {
    /// The type parameters, by LexLean name, in order.
    pub(crate) types: Vec<String>,
    /// Each type parameter by its Rust name -> what the body reads it as:
    /// its LexLean type parameter, or the function type of its `Fn` bound.
    pub(crate) reads: BTreeMap<String, Ty>,
    /// The dictionary, in the order the definition takes it.
    pub(crate) dictionary: Vec<Entry>,
}

/// One method in a dictionary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The type parameter it is the method of, by LexLean name.
    pub(crate) param: String,
    /// The trait that declares it, by its name in its crate.
    pub(crate) declarer: String,
    pub(crate) method: String,
    /// The LexLean parameter that holds it.
    pub(crate) name: String,
    /// Its function type: the receiver first, `Self` read as the type
    /// parameter.
    pub(crate) ty: Ty,
}

/// What the lift reads and does not change while it lifts a crate: the
/// types it names and the candidate functions' signatures.
pub(crate) struct Env {
    /// Imported compiler models may not replace a Rust panic with a default value.
    total: bool,
    pub(crate) types: Types,
    /// Candidate functions: Rust name -> signature.
    pub(crate) fns: BTreeMap<String, Sig>,
    /// The pointer widths `usize` takes, each with the targets it is the
    /// width of.
    pub(crate) widths: Vec<(u32, Vec<String>)>,
    /// The LexLean names of every crate's candidates, which share one
    /// module: a binder takes none of them.
    pub(crate) names: BTreeSet<String>,
}

/// The scope an expression or statement lifts in.
#[derive(Clone)]
struct Ctx {
    /// The Rust name of the function being lifted.
    current: String,
    /// Its parameters' LexLean names, in order.
    params: Vec<String>,
    /// Its result type.
    result: Ty,
    /// Mutable Rust parameters whose current values accompany an early result.
    return_states: Vec<String>,
    /// The LexLean name of the function being lifted, which its helpers extend.
    own: String,
    /// Rust local -> (LexLean name, type).
    locals: BTreeMap<String, (String, Ty)>,
    /// LexLean local bound to the tail of a slice parameter -> that parameter.
    tails: BTreeMap<String, String>,
    /// The statements in hand are the function's own, in tail position, so
    /// an early `return` leaves the function.
    top: bool,
    /// The locals of the loop whose step the statements in hand are: a
    /// `continue` there is the step's value, a `break` its last.
    looping: Option<Vec<String>>,
    /// Binders still in scope whose Rust local a later binding renamed.
    hidden: Vec<String>,
    /// The `use` imports of the function's file: name -> the path it names.
    imports: BTreeMap<String, String>,
    /// What the function's own module scope binds (see [`Seen`]).
    seen: Seen,
    /// The closures `let` binds, by Rust name: a call of one is its body
    /// with its parameters bound to the arguments.
    closures: BTreeMap<String, Closure>,
    /// The parameter a list in hand is a sub-term of, where a closure
    /// walking it binds each element: a recursive call on one descends.
    elements: Option<String>,
    /// A parameter a match split -> its constructor applied to the arm's
    /// binders, which names it there.
    rebuilt: BTreeMap<String, String>,
    /// Natural-number terms -> a lower bound the conditions in scope
    /// establish: `n` at least 2 after `if n < 2 { return .. }`.
    bounds: BTreeMap<String, u128>,
    /// Natural-number terms the conditions in scope hold below a `usize`
    /// -> the largest constant that added to one stays in range: one below
    /// any `usize`, half the narrowest width below a collection's length,
    /// which is at most `isize::MAX`.
    capped: BTreeMap<String, u128>,
    /// The state type of the loop step the statements in hand are, where
    /// they are the function's own: a `return` or `?` there leaves the walk
    /// with the function's value.
    exits: Option<Ty>,
    /// The formatter parameter of a `Display` impl's `fmt`, which `write!`
    /// writes the function's text into.
    formatter: Option<String>,
    /// The dictionary of the generic function being lifted.
    dictionary: Vec<Entry>,
    /// The Rust locals bound outside the expression in hand where its value
    /// is an operand (a `let`'s value, an argument, a closure's body): the
    /// statements after it are not part of it, so an update of one there
    /// would be lost to them, and is refused.
    sealed: BTreeSet<String>,
    /// The Rust names a `ref` or `ref mut` pattern of the body binds: such a
    /// binder is a borrow of the place it matched, so an update of it is
    /// one of that place, which no rebinding of the binder writes back.
    borrowed: BTreeSet<String>,
}

/// A closure a `let` binds: its parameters, its body, and the locals in
/// scope where it is written, which its body reads.
#[derive(Clone)]
struct Closure {
    inputs: Vec<rs::Pat>,
    body: rs::RustExpr,
    locals: BTreeMap<String, (String, Ty)>,
}

/// What lifting a body gathers as it goes. Each step takes it and gives
/// back the state after it, so the steps of a body are a fold over it.
#[derive(Clone, Debug, Default)]
struct Acc {
    /// The parameter positions recursive calls descend on.
    descends: BTreeSet<usize>,
    /// The other lifted functions the body calls.
    calls: BTreeSet<String>,
    /// The fresh binders the body has taken, in order.
    given: Vec<String>,
    /// Every type an expression of the body has.
    seen: BTreeSet<Ty>,
    /// The Lean axioms the body's primitives elaborate with.
    axioms: BTreeSet<String>,
    /// The list helpers the body's iterator chains lift to.
    helpers: Vec<Helper>,
    /// The types whose derived order the body compares by.
    ordered: BTreeSet<Ty>,
}

impl Acc {
    /// The state with `name` called.
    fn called(self, name: String) -> Acc {
        let mut calls = self.calls;
        calls.insert(name);
        Acc { calls, ..self }
    }

    /// The state with every one of `names` called.
    fn called_all(self, names: BTreeSet<String>) -> Acc {
        let mut calls = self.calls;
        calls.extend(names);
        Acc { calls, ..self }
    }

    /// The state with the axiom `name`.
    fn axiom(self, name: &str) -> Acc {
        let mut axioms = self.axioms;
        axioms.insert(name.to_string());
        Acc { axioms, ..self }
    }

    /// The state with every one of `names`.
    fn axioms_all(self, names: BTreeSet<String>) -> Acc {
        let mut axioms = self.axioms;
        axioms.extend(names);
        Acc { axioms, ..self }
    }

    /// The state with the axioms of the runtime's string length and decimal
    /// formatting.
    fn all_axioms(self) -> Acc {
        self.axioms_all(ALL_AXIOMS.iter().map(|a| a.to_string()).collect())
    }

    /// The state with every type `ty` mentions seen.
    fn saw(self, ty: &Ty) -> Acc {
        let mut seen = self.seen;
        seen.extend(ty.mentions());
        Acc { seen, ..self }
    }

    /// The state comparing values of `ty` by its derived order: the
    /// comparison's axioms, and `Ordering`, which it results in.
    fn ordered(self, ty: &Ty, types: &Types) -> Acc {
        let mut ordered = self.ordered;
        ordered.insert(crate::order::normal(ty));
        let acc = Acc { ordered, ..self }.saw(&Ty::Named(ORDERING.into()));
        acc.axioms_all(crate::order::axioms(ty, types))
    }

    /// The state with a recursive call descending on parameter `at`.
    fn descended(self, at: usize) -> Acc {
        let mut descends = self.descends;
        descends.insert(at);
        Acc { descends, ..self }
    }

    /// The state with one more helper.
    fn helper(self, helper: Helper) -> Acc {
        let mut helpers = self.helpers;
        helpers.push(helper);
        Acc { helpers, ..self }
    }

    /// The state of a scope of its own, apart from the calls, axioms and
    /// descents gathered so far, which `rejoin` puts back.
    fn apart(self) -> (Acc, Gathered) {
        let gathered = Gathered {
            calls: self.calls,
            axioms: self.axioms,
            descends: self.descends,
        };
        let scope = Acc {
            calls: BTreeSet::new(),
            axioms: BTreeSet::new(),
            descends: BTreeSet::new(),
            ..self
        };
        (scope, gathered)
    }

    /// The state after a scope of its own: the scope's fresh names, types
    /// and helpers, with the calls, axioms and descents gathered before it,
    /// and the scope's own apart.
    fn rejoin(self, before: Gathered) -> (Acc, Gathered) {
        let own = Gathered {
            calls: self.calls,
            axioms: self.axioms,
            descends: self.descends,
        };
        let acc = Acc {
            calls: before.calls,
            axioms: before.axioms,
            descends: before.descends,
            ..self
        };
        (acc, own)
    }

    /// A call of a prelude definition, with the axioms it brings.
    fn prelude(self, name: &str, args: &[&str]) -> (String, Acc) {
        let acc = self.axioms_all(prelude_axioms(name).into_iter().collect());
        (format!("({PRELUDE}::{name} {})", args.join(" ")), acc)
    }
}

/// The calls, axioms and descents of a scope, held apart from another's.
#[derive(Clone, Debug, Default)]
struct Gathered {
    calls: BTreeSet<String>,
    axioms: BTreeSet<String>,
    descends: BTreeSet<usize>,
}

/// A structurally recursive definition an iterator chain lifts to: it walks
/// its first parameter, a list, and takes the closure's captures after it.
#[derive(Clone, Debug)]
pub(crate) struct Helper {
    pub(crate) name: String,
    pub(crate) params: Vec<(String, String)>,
    pub(crate) result: String,
    pub(crate) body: Expr,
    /// The lifted functions the closure calls.
    pub(crate) calls: BTreeSet<String>,
    /// The Lean axioms the closure's primitives elaborate with.
    pub(crate) axioms: BTreeSet<String>,
    /// It recurses on its first parameter; a hoisted operand does not.
    pub(crate) recursive: bool,
    /// Its closure calls the function being lifted on an element of the
    /// list it walks, a sub-term of the function's recursive parameter:
    /// the two are one mutual group, each structural on its own.
    pub(crate) entwined: bool,
    /// A walk that is not structural: its measure and the theorems that
    /// each recursive call decreases it, which precede it.
    pub(crate) termination: Option<(String, Vec<Decl>)>,
}

/// A name `base<k>` that `taken` does not hold: one past the largest `k`
/// any taken `base<k>` spells, so a single pass over `taken` decides it
/// (searching `base1`, `base2`, .. against every taken name cost the
/// square of a function's helpers). A `k` past `u32::MAX` is passed over:
/// the name made has a smaller number, so it is never that one; and `u32`
/// keeps the successor's bound a literal the production target represents.
/// Only a taken name of `u32::MAX` leaves no successor, and then the first
/// free of `base1`, `base2`, .. is taken: each taken name rules out at most
/// one of the first `taken.len() + 1`.
fn first_free(base: &str, taken: &[String]) -> String {
    #[cfg(not(uorust_lift))]
    crate::certified::certified!(lift_first_free, base, taken);
    let top = taken
        .iter()
        .filter_map(|name| name.strip_prefix(base))
        .filter_map(|rest| rest.parse::<u32>().ok())
        .fold(0_u32, |most, k| most.max(k));
    match top.checked_add(1) {
        Some(next) => format!("{base}{next}"),
        None => (1..taken.len() + 2)
            .map(|k| format!("{base}{k}"))
            .find(|name| !taken.contains(name))
            .unwrap_or_default(),
    }
}

impl Ctx {
    /// A fresh name for a helper of the function being lifted.
    fn helper_name(&self, env: &Env, acc: &Acc, kind: &str) -> String {
        let base = format!("{}{kind}", self.own);
        let helpers: Vec<String> = acc.helpers.iter().map(|h| h.name.clone()).collect();
        first_free(&base, &self.names_from(env, acc, &base, helpers))
    }

    /// The names in scope, given or in `more` that start with `base`.
    fn names_from(&self, env: &Env, acc: &Acc, base: &str, more: Vec<String>) -> Vec<String> {
        self.locals
            .values()
            .map(|(n, _)| n.clone())
            .chain(self.hidden.iter().cloned())
            .chain(env.fns.values().map(|f| f.name.clone()))
            .chain(env.names.iter().cloned())
            .chain(acc.given.iter().cloned())
            .chain(more)
            .filter(|n| n.starts_with(base))
            .collect()
    }

    fn taken(&self, env: &Env, name: &str) -> bool {
        self.locals.values().any(|(n, _)| n == name)
            || self.hidden.iter().any(|n| n == name)
            || env.fns.values().any(|f| f.name == name)
            || env.names.contains(name)
    }

    /// A LexLean binder for a Rust name: a binder may not shadow another.
    fn bind(&self, env: &Env, acc: Acc, ident: &str, ty: Ty) -> (Ctx, String, Acc) {
        let name = local_name(ident);
        let (name, acc) = if self.taken(env, &name) {
            self.fresh(env, acc, &name)
        } else {
            (name, acc)
        };
        let mut next = self.clone();
        if let Some((old, _)) = next.locals.insert(ident.to_string(), (name.clone(), ty)) {
            next.hidden.push(old);
        }
        // A local bound inside an operand is the operand's own.
        next.sealed.remove(ident);
        (next, name, acc)
    }

    /// The locals an operand of the expression in hand may not update:
    /// every one bound so far.
    fn sealing(&self) -> BTreeSet<String> {
        let mut sealed = self.sealed.clone();
        sealed.extend(self.locals.keys().cloned());
        sealed
    }

    /// Why an update of `target` here is refused, where it is one an
    /// operand makes of a local bound outside it.
    fn unsealed(&self, target: &str) -> Lift<()> {
        if self.borrowed.contains(target) {
            return Err(Residue::Form(format!(
                "an update of {target}, which a ref pattern binds to the place it borrows"
            )));
        }
        if self.sealed.contains(target) {
            return Err(Residue::Form(format!(
                "an update of {target} inside an operand, which the statements after it do not see"
            )));
        }
        Ok(())
    }

    /// A binder no name in scope and no name given before takes: `base`
    /// and a number.
    fn fresh(&self, env: &Env, acc: Acc, base: &str) -> (String, Acc) {
        let name = first_free(base, &self.names_from(env, &acc, base, Vec::new()));
        let mut given = acc.given;
        given.push(name.clone());
        (name, Acc { given, ..acc })
    }

    /// The same scope where `cond` is `holds`: with the lower bound it
    /// gives a natural-number term, from `x >= k`, `x > k`, `!(x < k)`,
    /// `!(x <= k)` or `x != 0`, either way round.
    fn assuming(&self, cond: &rs::RustExpr, holds: bool, env: &Env, acc: &Acc) -> Ctx {
        use rs::BinOp as B;
        let next = self.clone();
        let b = match cond {
            rs::RustExpr::Unary(u) if matches!(u.op, rs::UnOp::Not) => {
                return self.assuming(&u.expr, !holds, env, acc);
            }
            rs::RustExpr::Paren(p) => return self.assuming(p, holds, env, acc),
            rs::RustExpr::Binary(b) => b,
            _ => return next,
        };
        // The bound on the term, with the term on the left.
        let (term, op, k) = match (int_literal(&b.left), int_literal(&b.right)) {
            (None, Some(k)) => (&*b.left, b.op, k),
            (Some(k), None) => {
                let flipped = match b.op {
                    B::Lt => B::Gt,
                    B::Le => B::Ge,
                    B::Gt => B::Lt,
                    B::Ge => B::Le,
                    other => other,
                };
                (&*b.right, flipped, k)
            }
            _ => return next,
        };
        let at_least = match (op, holds) {
            (B::Ge, true) | (B::Lt, false) => Some(k),
            (B::Gt, true) | (B::Le, false) => k.checked_add(1),
            (B::Ne, true) | (B::Eq, false) if k == 0 => Some(1),
            _ => None,
        };
        let Some(at_least) = at_least else {
            return next;
        };
        let Ok(((t, Ty::Nat(_)), _)) = flat_of(term, env, &self.below(), acc.clone(), None) else {
            return next;
        };
        let mut next = next;
        let known = next.bounds.get(&t).copied().unwrap_or(0);
        next.bounds.insert(t, known.max(at_least));
        next
    }

    /// The same scope, below the function's own statements: a `return`
    /// there is not an early exit the lift can follow.
    fn below(&self) -> Ctx {
        let mut next = self.clone();
        next.top = false;
        next
    }

    /// A value the function returns, from where the statements in hand
    /// stand: itself, or in a loop step, the mark that leaves the walk.
    fn returning(&self, value: (Expr, Ty)) -> (Expr, Ty) {
        match &self.exits {
            Some(state) => (
                map_tails(value.0, &|s| format!("{LEAVE}{s}")),
                state.clone(),
            ),
            None => value,
        }
    }
}

fn pow2(bits: u32) -> &'static str {
    match bits {
        8 => "256",
        16 => "65536",
        32 => "4294967296",
        64 => "18446744073709551616",
        _ => "340282366920938463463374607431768211456",
    }
}

fn max_of(bits: u32) -> &'static str {
    match bits {
        8 => "255",
        16 => "65535",
        32 => "4294967295",
        64 => "18446744073709551615",
        _ => "340282366920938463463374607431768211455",
    }
}

/// The single text of an expression that must stay on one line.
fn flat_text(e: Expr) -> Lift<String> {
    match e {
        Expr::Flat(s) => Ok(s),
        Expr::If { .. } if !e.flat().contains('\n') && !e.flat().contains("(match") => Ok(e.flat()),
        _ => Err(Residue::Form("match or let inside an operand".to_string())),
    }
}

/// An operand on one line. A `match` or `let` there becomes a definition of
/// its own, a helper that takes the locals it names, called in its place.
fn flat_of(
    e: &rs::RustExpr,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((String, Ty), Acc)> {
    let scope = Ctx {
        top: false,
        looping: None,
        hidden: Vec::new(),
        sealed: ctx.sealing(),
        ..ctx.clone()
    };
    let (inner, before) = acc.apart();
    let ((x, t), inner) = expr(e, env, &scope, inner, want)?;
    let (acc, own) = inner.rejoin(before);
    if let Ok(text) = flat_text(x.clone()) {
        let acc = acc.called_all(own.calls).axioms_all(own.axioms);
        let mut descends = acc.descends;
        descends.extend(own.descends);
        return Ok(((text, t), Acc { descends, ..acc }));
    }
    if !own.descends.is_empty() {
        return Err(Residue::Form(
            "recursion inside a match or let in an operand".to_string(),
        ));
    }
    let caps = captures(ctx, &[x.flat()], &[]);
    let name = ctx.helper_name(env, &acc, "Part");
    let mut parts = vec![name.clone()];
    parts.extend(caps.iter().map(|(c, _)| c.clone()));
    let acc = acc.helper(Helper {
        name: name.clone(),
        params: caps
            .iter()
            .map(|(c, ty)| (c.clone(), ty.render()))
            .collect(),
        result: t.render(),
        body: x,
        calls: own.calls,
        axioms: own.axioms,
        recursive: false,
        entwined: false,
        termination: None,
    });
    let acc = acc.called(format!("#{name}"));
    Ok(((format!("({})", parts.join(" ")), t), acc))
}

/// The pointer width of a target, from the architecture its triple names
/// first, as rustc's target specifications set `target_pointer_width`.
pub(crate) fn pointer_width(triple: &str) -> Option<u32> {
    #[cfg(not(uorust_lift))]
    crate::certified::certified!(lift_pointer_width, triple);
    let arch = triple.split('-').next()?;
    let wide = [
        "x86_64",
        "aarch64",
        "arm64",
        "riscv64",
        "powerpc64",
        "powerpc64le",
        "s390x",
        "sparc64",
        "mips64",
        "mips64el",
        "loongarch64",
        "wasm64",
        "bpfel",
        "bpfeb",
    ];
    let narrow = [
        "i386",
        "i586",
        "i686",
        "x86",
        "arm",
        "armv7",
        "armv7a",
        "armv7r",
        "armebv7r",
        "armv5te",
        "armv4t",
        "armv6",
        "thumbv6m",
        "thumbv7m",
        "thumbv7em",
        "thumbv7neon",
        "thumbv8m.base",
        "thumbv8m.main",
        "riscv32i",
        "riscv32im",
        "riscv32imc",
        "riscv32imac",
        "riscv32imafc",
        "powerpc",
        "mips",
        "mipsel",
        "sparc",
        "wasm32",
        "hexagon",
        "csky",
        "xtensa",
        "m68k",
    ];
    if wide.contains(&arch) {
        Some(64)
    } else if narrow.contains(&arch) || arch.starts_with("thumbv") || arch.starts_with("armv") {
        Some(32)
    } else if matches!(arch, "avr" | "msp430") {
        Some(16)
    } else {
        None
    }
}

/// The widths `usize` takes on the declared targets, narrowest first, each
/// with its targets. A target whose width the triple does not name, and a
/// workspace that declares none, leave every width the language allows.
pub(crate) fn pointer_widths(targets: &[String]) -> Vec<(u32, Vec<String>)> {
    let mut by: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    let mut open = targets.is_empty();
    for t in targets {
        match pointer_width(t) {
            Some(w) => by.entry(w).or_default().push(t.clone()),
            None => open = true,
        }
    }
    if open {
        for w in [16, 32, 64] {
            by.entry(w).or_default().push(format!("any {w}-bit target"));
        }
    }
    by.into_iter().collect()
}

/// `a + b` on `usize`, which agrees with the sum of naturals on every
/// declared target where the declared limits bound it: a collection's
/// length is at most `isize::MAX`, `2^(w-1) - 1` on a target of width `w`,
/// so a length plus a constant `k <= 2^(w-1)` stays below `2^w` on all of
/// them. Otherwise the residue names the targets, by width, where the sum
/// can leave the width.
fn usize_sum(
    left: &rs::RustExpr,
    right: &rs::RustExpr,
    l: &str,
    r: &str,
    env: &Env,
    capped: &BTreeMap<String, u128>,
) -> Lift<String> {
    let is_len = |e: &rs::RustExpr| matches!(e, rs::RustExpr::MethodCall(m) if m.method == "len" && m.args.is_empty());
    let narrowest = env.widths.first().map(|(w, _)| *w);
    let half = narrowest.and_then(|w| 1u128.checked_shl(w - 1));
    let bounded = |len: &rs::RustExpr, text: &str, k: &rs::RustExpr| match int_literal(k) {
        Some(k) if is_len(len) => half.is_some_and(|h| k <= h),
        Some(k) => capped.get(text).is_some_and(|m| k <= *m),
        None => false,
    };
    if bounded(left, l, right) || bounded(right, r, left) {
        return Ok(format!("({l} + {r})"));
    }
    Err(Residue::Overflow("+".to_string(), env.widths.clone()))
}

fn int_literal(e: &rs::RustExpr) -> Option<u128> {
    match e {
        rs::RustExpr::Lit(rs::Lit::Int { value, .. }) => *value,
        rs::RustExpr::Paren(p) => int_literal(p),
        _ => None,
    }
}

/// A number literal without a suffix, negated, in parentheses or in
/// arithmetic with others of its kind: its type is whatever inference
/// gives it.
fn untyped_number(e: &rs::RustExpr) -> bool {
    match e {
        rs::RustExpr::Lit(rs::Lit::Int { suffix, .. }) => suffix.is_empty(),
        rs::RustExpr::Lit(rs::Lit::Other) => true,
        rs::RustExpr::Unary(u) => {
            matches!(u.op, rs::UnOp::Neg | rs::UnOp::Not) && untyped_number(&u.expr)
        }
        rs::RustExpr::Paren(p) => untyped_number(p),
        // Arithmetic of such literals has their one inferred type; a shift
        // has its left operand's.
        rs::RustExpr::Binary(b) => match b.op {
            rs::BinOp::Add
            | rs::BinOp::Sub
            | rs::BinOp::Mul
            | rs::BinOp::Div
            | rs::BinOp::Rem
            | rs::BinOp::BitXor
            | rs::BinOp::BitAnd
            | rs::BinOp::BitOr => untyped_number(&b.left) && untyped_number(&b.right),
            rs::BinOp::Shl | rs::BinOp::Shr => untyped_number(&b.left),
            _ => false,
        },
        _ => false,
    }
}

fn is_literal(e: &rs::RustExpr) -> bool {
    match e {
        rs::RustExpr::Lit(_) => true,
        rs::RustExpr::Unary(u) => u.op == rs::UnOp::Neg && is_literal(&u.expr),
        rs::RustExpr::Paren(p) => is_literal(p),
        _ => false,
    }
}

/// A LexLean string literal holding exactly `text`, control characters
/// escaped; `None` when LexLean source cannot write it.
pub(crate) fn exact_string(text: &str) -> Option<String> {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            // A raw percent sign would open a TeX comment, which LexLean
            // source forbids, and its canonical form writes every string
            // with its characters raw: no spelling of it checks.
            '%' => return None,
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    Some(out)
}

/// Why a string literal does not lift.
const PERCENT: &str = "a string with a percent sign, which LexLean source cannot write";

fn literal_of(lit: &rs::Lit, want: Option<&Ty>, negative: bool) -> Lift<(Expr, Ty)> {
    let flat = |s: String, t: Ty| Ok((Expr::Flat(s), t));
    match lit {
        rs::Lit::Bool(b) if !negative => flat(b.to_string(), Ty::Bool),
        rs::Lit::Int { value, suffix } => {
            let Some(v) = *value else {
                return Err(Residue::Form("integer literal".to_string()));
            };
            let ty = match suffix.as_str() {
                "u8" => Ty::Nat(Some(8)),
                "u16" => Ty::Nat(Some(16)),
                "u32" => Ty::Nat(Some(32)),
                "u64" => Ty::Nat(Some(64)),
                "u128" => Ty::Nat(Some(128)),
                "usize" => Ty::Nat(None),
                "i8" => Ty::Int(8),
                "i16" => Ty::Int(16),
                "i32" => Ty::Int(32),
                "i64" => Ty::Int(64),
                "i128" => Ty::Integer(Some(128)),
                "isize" => Ty::Integer(None),
                "" => match want {
                    Some(t @ (Ty::Nat(_) | Ty::Int(_) | Ty::Integer(_) | Ty::Unbounded)) => {
                        t.clone()
                    }
                    _ if negative => Ty::Int(32),
                    _ => Ty::Nat(Some(32)),
                },
                _ => return Err(Residue::Form("integer literal".to_string())),
            };
            match ty {
                Ty::Int(_) if v <= i64::MAX as u128 + u128::from(negative) => {
                    let sign = if negative { "-" } else { "" };
                    flat(format!("({sign}{v} : Int64)"), ty)
                }
                // The widest signed type bounds the literal Rust accepts;
                // the mathematical `Int` holds every one of them.
                Ty::Integer(_) | Ty::Unbounded if v <= i128::MAX as u128 + u128::from(negative) => {
                    let sign = if negative { "-" } else { "" };
                    flat(format!("({sign}{v} : Int)"), ty)
                }
                Ty::Nat(_) if !negative => flat(v.to_string(), ty),
                _ => Err(Residue::Form("integer literal".to_string())),
            }
        }
        rs::Lit::Str(s) if !negative => match exact_string(s) {
            Some(text) => flat(text, Ty::Str),
            None => Err(Residue::Form(PERCENT.to_string())),
        },
        rs::Lit::Char(c) if !negative => match exact_string(&c.to_string()) {
            Some(text) => flat(text, Ty::Char),
            None => Err(Residue::Form(PERCENT.to_string())),
        },
        rs::Lit::Byte(b) if !negative => flat(b.to_string(), Ty::Nat(Some(8))),
        // A float literal: the format's value nearest the decimal, ties to
        // even, as Rust rounds one. Unsuffixed, its type is what its use
        // infers (`f64` only where nothing constrains it, which a later use
        // the lift has not reached yet may still do), so it is read only
        // where the use in hand asks for a float of one format.
        rs::Lit::Float { digits, suffix } => {
            let named = match (suffix.as_str(), want) {
                ("f32" | "f64", _) => suffix.as_str(),
                ("", Some(Ty::Float(f))) => return float_literal(*f, digits, negative),
                ("", _) => {
                    return Err(Residue::Form(
                        "a float literal whose format its use does not fix".to_string(),
                    ))
                }
                _ => return Err(Residue::Form("float literal".to_string())),
            };
            match Types::default().float(named) {
                Some(f) => float_literal(f, digits, negative),
                None => Err(Residue::Form("float literal".to_string())),
            }
        }
        rs::Lit::ByteStr(b) if !negative => {
            let bytes: Vec<String> = b.iter().map(u8::to_string).collect();
            flat(
                crate::embed::term_list(&bytes, "Nat"),
                Ty::List(Box::new(Ty::Nat(Some(8)))),
            )
        }
        _ => Err(Residue::Form("literal".to_string())),
    }
}

fn ctor(path: &rs::Path, types: &Types) -> Option<(String, String)> {
    let n = path.segments.len();
    // `S` of `struct S;`: the one value of its type. No variant of an
    // enum lifts under the constructor's name, so the name alone tells.
    if n == 1 {
        let owner = types.resolve(&path.segments[0].ident);
        let sole = types
            .enums
            .get(&owner)
            .is_some_and(|vs| vs.len() == 1 && vs.iter().all(|v| v == SOLE));
        return sole.then(|| (owner, SOLE.to_string()));
    }
    if n < 2 {
        return None;
    }
    let owner = types.resolve(&path.segments[n - 2].ident.to_string());
    let variant = path.segments[n - 1].ident.to_string();
    let unit = types
        .enums
        .get(&owner)
        .is_some_and(|vs| vs.contains(&variant))
        || types
            .datas
            .get(&owner)
            .is_some_and(|vs| vs.iter().any(|(v, fs)| *v == variant && fs.is_empty()));
    unit.then_some((owner, variant))
}

/// Whether a type is a data-carrying enum.
fn types_has_data(t: &Ty, types: &Types) -> bool {
    matches!(t, Ty::Named(n) if types.datas.contains_key(n))
}

/// A variant of a data-carrying enum a path names, and its fields.
fn data_variant<'t>(
    path: &rs::Path,
    types: &'t Types,
) -> Option<(String, String, &'t [DataField])> {
    let n = path.segments.len();
    if n < 2 {
        return None;
    }
    let owner = types.resolve(&path.segments[n - 2].ident.to_string());
    let variant = path.segments[n - 1].ident.to_string();
    let (_, fields) = types
        .datas
        .get(&owner)?
        .iter()
        .find(|(v, _)| *v == variant)?;
    Some((owner, variant, fields))
}

/// `None`, `Some`, `Ok` or `Err`, written bare or through `Option::`/`Result::`.
fn wrapper(path: &rs::Path) -> Option<&'static str> {
    let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    let (last, owner) = match segs.as_slice() {
        [last] => (last.as_str(), None),
        [owner, last] => (last.as_str(), Some(owner.as_str())),
        _ => return None,
    };
    match (owner, last) {
        (None | Some("Option"), "None") => Some("None"),
        (None | Some("Option"), "Some") => Some("Some"),
        (None | Some("Result"), "Ok") => Some("Ok"),
        (None | Some("Result"), "Err") => Some("Err"),
        _ => None,
    }
}

fn expr(
    e: &rs::RustExpr,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let ((x, t), acc) = expr_inner(e, env, ctx, acc, want)?;
    let acc = acc.saw(&t);
    Ok(((x, t), acc))
}

fn expr_inner(
    e: &rs::RustExpr,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    // The forms a nested expression recurses through, each in a function
    // of its own, so this frame stays small however deep they nest.
    match e {
        rs::RustExpr::Paren(p) => expr(p, env, ctx, acc, want),
        rs::RustExpr::Block(b) if !b.label => block(&b.block.stmts, env, ctx, acc, want),
        rs::RustExpr::Reference(r) if !r.mutable => expr(&r.expr, env, ctx, acc, want),
        rs::RustExpr::Binary(b) => binary(b, env, ctx, acc, want),
        rs::RustExpr::Call(c) => call(c, env, ctx, acc, want),
        rs::RustExpr::MethodCall(m) => method(m, env, ctx, acc, want),
        rs::RustExpr::Macro(m) if m.path.is_ident("format") => format_macro(m, env, ctx, acc),
        rs::RustExpr::Macro(m) if m.path.is_ident("write") => write_macro(m, env, ctx, acc),
        rs::RustExpr::Closure(_) if matches!(want, Some(Ty::Fn(..))) => {
            function_value(e, env, ctx, acc, want)
        }
        // A closure a `let` binds, where a function is wanted: its lambda,
        // in the scope it was written in.
        rs::RustExpr::Path(p)
            if matches!(want, Some(Ty::Fn(..)))
                && p.get_ident().is_some_and(|i| ctx.closures.contains_key(i)) =>
        {
            let Some(closure) = p.get_ident().and_then(|i| ctx.closures.get(i)) else {
                return Err(Residue::Form("a closure out of scope".to_string()));
            };
            let written = rs::RustExpr::Closure(rs::ExprClosure {
                inputs: closure.inputs.clone(),
                body: Box::new(closure.body.clone()),
                asyncness: false,
            });
            let scope = Ctx {
                locals: closure.locals.clone(),
                ..ctx.clone()
            };
            function_value(&written, env, &scope, acc, want)
        }
        _ => expr_form(e, env, ctx, acc, want),
    }
}

/// A closure where a function is wanted: the lambda of its body over its
/// parameters, with the captures it reads.
fn function_value(
    e: &rs::RustExpr,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let Some(f @ Ty::Fn(inputs, output)) = want else {
        return Err(Residue::Form(
            "a closure where no function is wanted".to_string(),
        ));
    };
    let (lam, acc) = lambda(e, inputs, env, ctx, acc, Some(output))?;
    if !lam.ty.same(output) {
        return Err(Residue::Form("a closure of another type".to_string()));
    }
    if !lam.descends.is_empty() {
        return Err(Residue::Form("recursion inside a closure".to_string()));
    }
    let body = lam.wrap(lam.core.clone()).flat();
    let texts = vec![body.clone()];
    let caps = captures(ctx, &texts, &lam.binders);
    let held = match caps.as_slice() {
        [] => String::new(),
        some => format!(
            "[{}] ",
            some.iter()
                .map(|(c, _)| c.clone())
                .collect::<Vec<String>>()
                .join(", ")
        ),
    };
    let params: Vec<String> = lam
        .binders
        .iter()
        .zip(inputs)
        .map(|(b, t)| format!("({b} : {})", t.render()))
        .collect();
    let acc = acc.called_all(lam.calls).axioms_all(lam.axioms);
    Ok((
        (
            Expr::Flat(format!("(fun {held}{} => {body})", params.join(" "))),
            f.clone(),
        ),
        acc,
    ))
}

/// A value with nothing left to lift: its term and type.
fn flat_lift(s: String, t: Ty, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    Ok(((Expr::Flat(s), t), acc))
}

/// The other forms of [`expr_inner`].
#[inline(never)]
fn expr_form(
    e: &rs::RustExpr,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    match e {
        rs::RustExpr::Lit(lit) => literal_of(lit, want, false).map(|v| (v, acc)),
        rs::RustExpr::Path(p) => path_value(p, env, ctx, acc, want),
        rs::RustExpr::Unary(u) => unary_value(u, env, ctx, acc, want),
        rs::RustExpr::If(i) => if_value(i, env, ctx, acc, want),
        rs::RustExpr::Match(m) => match_value(m, env, ctx, acc, want),
        rs::RustExpr::Field(f) => field_value(f, env, ctx, acc),
        // `E::V { a, b }`: a struct variant, its fields in declaration order.
        rs::RustExpr::Struct(s)
            if s.rest.is_none() && !s.qself && data_variant(&s.path, &env.types).is_some() =>
        {
            variant_value(s, env, ctx, acc)
        }
        // `S { a: x, .. }`, or `S { a: x, ..base }` where `base` gives the
        // fields not written.
        rs::RustExpr::Struct(s) if !s.qself => struct_literal(s, env, ctx, acc),
        rs::RustExpr::Tuple(t) if t.is_empty() => flat_lift("()".into(), Ty::Unit, acc),
        rs::RustExpr::Tuple(t) if t.len() >= 2 => tuple_literal(t, env, ctx, acc, want),
        // `a..b`, `a..=b`: the naturals from `a`, as many as the range
        // holds; none where it is empty.
        rs::RustExpr::Other(o) if range_bounds(e).is_some() => range_value(o, e, env, ctx, acc),
        rs::RustExpr::Cast(c) => cast_value(c, env, ctx, acc),
        // `xs[i]`: the element, and where Rust's bounds check panics, the
        // type's default, as Lean's `xs[i]!` is.
        rs::RustExpr::Index(ix) => index_value(ix, env, ctx, acc),
        rs::RustExpr::Continue(label) => loop_exit(env, ctx, acc, *label, false),
        rs::RustExpr::Break(b) if b.expr.is_none() => loop_exit(env, ctx, acc, b.label, true),
        rs::RustExpr::Try(_) => Err(Residue::Form("? outside a let".to_string())),
        // A `return` in tail position: the function's value.
        rs::RustExpr::Return(Some(v)) if ctx.top => early_return(Some(v), env, ctx, acc),
        // `matches!(e, P)`: a match with `true` for `P` and `false` otherwise.
        rs::RustExpr::Macro(m) if m.path.is_ident("matches") => matches_value(m, env, ctx, acc),
        rs::RustExpr::Macro(m) if m.path.is_ident("vec") => {
            let Some(rs::MacroArgs { items }) = &m.args else {
                return Err(Residue::Form("vec! of a repeated value".to_string()));
            };
            list_of(items, env, ctx, acc, want)
        }
        rs::RustExpr::Array(items) => list_of(items, env, ctx, acc, want),
        _ => Err(Residue::Form(e.kind())),
    }
}

/// `!b`, `-lit` and `*e`.
fn unary_value(
    u: &rs::ExprUnary,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    match u.op {
        rs::UnOp::Not => {
            let ((x, t), acc) = flat_of(&u.expr, env, ctx, acc, Some(&Ty::Bool))?;
            if t != Ty::Bool {
                return Err(Residue::Form("bitwise not".to_string()));
            }
            flat_lift(format!("(not {x})"), Ty::Bool, acc)
        }
        rs::UnOp::Neg => match &*u.expr {
            rs::RustExpr::Lit(lit) => literal_of(lit, want, true).map(|v| (v, acc)),
            // A float negates its sign, a NaN's too.
            e if float_operand(e, env, ctx, &acc) => {
                let ((x, xt), acc) = flat_of(e, env, ctx, acc, None)?;
                let Ty::Float(f) = xt else {
                    return Err(Residue::Arithmetic("-".to_string()));
                };
                let (term, acc) = float_term(f, "Neg", &[&x], acc);
                flat_lift(term, xt, acc)
            }
            // An integer of no width negates exactly (see
            // `unbounded_arithmetic`).
            e if unbounded_operand(e, env, ctx, &acc) => {
                let ((x, _), acc) = flat_of(e, env, ctx, acc, Some(&Ty::Unbounded))?;
                flat_lift(format!("(prim negate {x} : Int)"), Ty::Unbounded, acc)
            }
            _ => Err(Residue::Arithmetic("-".to_string())),
        },
        rs::UnOp::Deref => expr(&u.expr, env, ctx, acc, want),
    }
}

/// A path: a local, a constant, `None`, or a unit variant.
fn path_value(
    p: &rs::Path,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    if let Some(ident) = p.get_ident() {
        if let Some((name, ty)) = ctx.locals.get(&ident.to_string()) {
            return flat_lift(name.clone(), ty.clone(), acc);
        }
    }
    // A constant: a definition of no parameters.
    if let Some(ident) = p.get_ident() {
        let key = ident.to_string();
        if let Some(sig) = env.fns.get(&key).filter(|s| s.params.is_empty()) {
            if key == ctx.current {
                return Err(Residue::Form("a constant that names itself".to_string()));
            }
            let acc = acc.called(key);
            return flat_lift(format!("({})", sig.name), sig.result.clone(), acc);
        }
    }
    if wrapper(p) == Some("None") {
        return match want {
            Some(t @ Ty::Option(inner)) => flat_lift(
                format!("(ctor Option.none [{}])", inner.render()),
                t.clone(),
                acc,
            ),
            _ => Err(Residue::Form("None of an unknown type".to_string())),
        };
    }
    match ctor(p, &env.types) {
        Some((owner, variant)) => {
            flat_lift(format!("(ctor {owner}.{variant})"), Ty::Named(owner), acc)
        }
        None => match p.segments.as_slice() {
            [owner, f] if owner.ident == "char" && unicode_property(&f.ident).is_some() => Err(
                Residue::Form(unicode_property(&f.ident).unwrap_or_default()),
            ),
            _ => Err(Residue::Form(format!("free name {}", p.text))),
        },
    }
}

/// `if c { a } else { b }` and `if let p = e { a } else { b }`.
fn if_value(
    i: &rs::ExprIf,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let Some(otherwise) = &i.else_branch else {
        return Err(Residue::Form("if without else".to_string()));
    };
    if let rs::RustExpr::Let(l) = &*i.cond {
        let wild = rs::Pat::Wild;
        let arms = vec![
            (&l.pat, Body::Block(&i.then_branch)),
            (&wild, Body::Expr(otherwise)),
        ];
        return matching(&l.expr, &arms, env, ctx, acc, want);
    }
    let ((c, ct), acc) = flat_of(&i.cond, env, &ctx.below(), acc, Some(&Ty::Bool))?;
    if ct != Ty::Bool {
        return Err(Residue::Form("non-boolean condition".to_string()));
    }
    let yes = ctx.assuming(&i.cond, true, env, &acc);
    let no = ctx.assuming(&i.cond, false, env, &acc);
    let ((then, tt), (other, ot), acc) =
        match block(&i.then_branch.stmts, env, &yes, acc.clone(), want) {
            // A branch whose type only the other says: that one first.
            Err(r) if want.is_none() && untyped(&r) => {
                let ((other, ot), acc) = expr(otherwise, env, &no, acc, None)?;
                let (then, acc) = block(&i.then_branch.stmts, env, &yes, acc, Some(&ot))?;
                (then, (other, ot), acc)
            }
            lifted => {
                let ((then, tt), acc) = lifted?;
                let (other, acc) = expr(otherwise, env, &no, acc, Some(want.unwrap_or(&tt)))?;
                ((then, tt), other, acc)
            }
        };
    if !tt.same(&ot) {
        return Err(Residue::Form("branches of different types".to_string()));
    }
    Ok((
        (
            Expr::If {
                condition: Box::new(Expr::Flat(c)),
                then: Box::new(then),
                otherwise: Box::new(other),
            },
            tt,
        ),
        acc,
    ))
}

/// `match e { .. }`: the match compiler over its arms.
fn match_value(
    m: &rs::ExprMatch,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let arms: Vec<(&rs::Pat, Body<'_>)> = m
        .arms
        .iter()
        .map(|a| match &a.pat {
            rs::Pat::Guard(g) => (&*g.pat, Body::Guarded(&g.guard, &a.body)),
            p => (p, Body::Expr(&a.body)),
        })
        .collect();
    matching(&m.expr, &arms, env, ctx, acc, want)
}

/// `e.f` and `e.0`: a field of a structure or an item of a tuple.
fn field_value(f: &rs::ExprField, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let ((base, bt), acc) = flat_of(&f.base, env, ctx, acc, None)?;
    match (&f.member, &bt) {
        (member, Ty::Named(s)) => {
            // A tuple struct's field is named by its position.
            let name = match member {
                rs::Member::Named(field) => local_name(field),
                rs::Member::Unnamed(position) => field_name(&position.to_string()),
            };
            let Some((_, ty)) = env
                .types
                .structs
                .get(s)
                .and_then(|fs| fs.iter().find(|(n, _)| *n == name))
            else {
                return Err(Residue::Form("field of an unknown struct".to_string()));
            };
            if let Some(kind) = env.types.cell(s, &name) {
                return Err(cell_unread(kind, s, &name, None));
            }
            if env.types.boxed(s) {
                flat_lift(
                    format!("({} {base})", Types::accessor(s, &name)),
                    ty.clone(),
                    acc,
                )
            } else {
                flat_lift(format!("(project {base} {name})"), ty.clone(), acc)
            }
        }
        (rs::Member::Unnamed(i), Ty::Tuple(ts) | Ty::Pair(ts)) => match ts.get(*i as usize) {
            Some(ty) => flat_lift(tuple_item(&bt, &base, *i as usize), ty.clone(), acc),
            None => Err(Residue::Form("tuple field".to_string())),
        },
        _ => Err(Residue::Form("field access".to_string())),
    }
}

/// `E::V { a, b }`: a struct variant, its fields in declaration order.
fn variant_value(s: &rs::ExprStruct, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let Some((owner, variant, fields)) = data_variant(&s.path, &env.types) else {
        return Err(Residue::Form("struct variant".to_string()));
    };
    let mut given = BTreeMap::new();
    for fv in &s.fields {
        let rs::Member::Named(id) = &fv.member else {
            return Err(Residue::Form("struct variant field".to_string()));
        };
        given.insert(id.to_string(), &fv.expr);
    }
    if given.len() != fields.len() {
        return Err(Residue::Form("struct variant literal".to_string()));
    }
    let mut parts = vec![format!("ctor {owner}.{variant}")];
    let mut acc = acc;
    for (name, ty) in fields {
        let Some(value) = name.as_ref().and_then(|n| given.get(n)) else {
            return Err(Residue::Form("struct variant literal".to_string()));
        };
        let ((v, vt), next) = flat_of(value, env, ctx, acc, Some(ty))?;
        acc = next;
        if !vt.same(ty) {
            return Err(Residue::Form("struct variant field of another type".into()));
        }
        parts.push(v);
    }
    flat_lift(format!("({})", parts.join(" ")), Ty::Named(owner), acc)
}

/// `S { a: x, .. }`, or `S { a: x, ..base }` where `base` gives the
/// fields not written.
fn struct_literal(s: &rs::ExprStruct, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let Some(written) = s.path.segments.last().map(|seg| seg.ident.clone()) else {
        return Err(Residue::Form("struct path".to_string()));
    };
    let owner = ctx.current.split_once("::").map(|(o, _)| o.to_string());
    let name = match (written.as_str(), owner) {
        ("Self", Some(o)) => env.types.resolve(&o),
        _ => env.types.resolve(&written),
    };
    // `S`, `Self`, or `m::S` through modules of this crate, which
    // declares a type name once.
    let through_modules = s.path.segments.split_last().is_some_and(|(_, prefix)| {
        prefix
            .first()
            .is_some_and(|h| rs::crate_module(&h.ident, &ctx.imports))
            && prefix
                .iter()
                .all(|seg| seg.arguments == rs::PathArguments::None)
    });
    let local = s.path.segments.len() == 1 || written == "Self" || through_modules;
    let Some(fields) = env.types.structs.get(&name).filter(|_| local) else {
        return Err(Residue::Form(format!("struct {name}")));
    };
    let mut given = BTreeMap::new();
    for fv in &s.fields {
        let rs::Member::Named(id) = &fv.member else {
            return Err(Residue::Form("struct field".to_string()));
        };
        given.insert(local_name(id), &fv.expr);
    }
    let mut acc = acc;
    let base = match &s.rest {
        Some(b) => {
            let ((bv, bt), next) = flat_of(b, env, ctx, acc, Some(&Ty::Named(name.clone())))?;
            acc = next;
            if bt != Ty::Named(name.clone()) {
                return Err(Residue::Form("struct update of another type".to_string()));
            }
            Some(bv)
        }
        None if given.len() != fields.len() => {
            return Err(Residue::Form("struct literal".to_string()));
        }
        None => None,
    };
    let mut parts = Vec::new();
    for (field, ty) in fields {
        let v = match (given.get(field), &base) {
            (Some(value), _) => {
                let value = match env.types.cell(&name, field) {
                    Some(kind) => cell_made(value, kind, &name, field)?,
                    None => *value,
                };
                let ((v, vt), next) = flat_of(value, env, ctx, acc, Some(ty))?;
                acc = next;
                if !vt.same(ty) {
                    return Err(Residue::Form("struct field of another type".to_string()));
                }
                v
            }
            (None, Some(b)) if env.types.boxed(&name) => {
                format!("({} {b})", Types::accessor(&name, field))
            }
            (None, Some(b)) => format!("(project {b} {field})"),
            (None, None) => return Err(Residue::Form("struct literal".to_string())),
        };
        parts.push((field, v));
    }
    flat_lift(
        struct_value(&name, &parts, &env.types),
        Ty::Named(name),
        acc,
    )
}

/// `(a, b, ..)`: a tuple of its items.
fn tuple_literal(
    t: &[rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let wants: Vec<Option<&Ty>> = match want {
        Some(Ty::Tuple(ts) | Ty::Pair(ts)) if ts.len() == t.len() => ts.iter().map(Some).collect(),
        _ => vec![None; t.len()],
    };
    let mut parts = Vec::new();
    let mut tys = Vec::new();
    let mut acc = acc;
    for (e, w) in t.iter().zip(wants) {
        let ((v, vt), next) = flat_of(e, env, ctx, acc, w)?;
        acc = next;
        parts.push(v);
        tys.push(vt);
    }
    let ty = env.types.tuple(tys);
    flat_lift(tuple_value(&ty, &parts), ty, acc)
}

/// `a..b`, `a..=b`: the naturals from `a`, as many as the range holds.
fn range_value(
    o: &rs::ExprOther,
    e: &rs::RustExpr,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let Some((lo, hi, closed)) = range_bounds(e) else {
        return Err(Residue::Form(o.kind.clone()));
    };
    let ((l, lt), acc) = flat_of(lo, env, ctx, acc, Some(&Ty::Nat(None)))?;
    let ((h, ht), acc) = flat_of(hi, env, ctx, acc, Some(&lt))?;
    if !matches!(lt, Ty::Nat(_)) || !ht.same(&lt) {
        return Err(Residue::Form(
            "a range of something other than naturals".into(),
        ));
    }
    let end = if closed { format!("({h} + 1)") } else { h };
    let count = format!("(prim subtract {end} {l} : Nat)");
    let (list, acc) = acc.prelude("natRange", &[&l, &count]);
    flat_lift(list, Ty::List(Box::new(lt)), acc)
}

/// `e as T`.
fn cast_value(c: &rs::ExprCast, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let ((v, vt), acc) = flat_of(&c.expr, env, ctx, acc, None)?;
    let target = match env.types.ty(&c.ty.localized(&ctx.imports)) {
        Ok(t) => t,
        Err(_) => return Err(Residue::Form("cast".to_string())),
    };
    // `usize` is as wide as the narrowest declared target's pointer.
    let narrowest = env.widths.first().map(|(w, _)| *w);
    // `isize` is at most as wide as the widest one's.
    let widest = env.widths.last().map(|(w, _)| *w);
    match (&vt, &target) {
        (Ty::Nat(Some(a)), Ty::Nat(Some(b))) if a <= b => flat_lift(v, target, acc),
        (Ty::Nat(Some(a)), Ty::Nat(None)) if narrowest.is_some_and(|w| *a <= w) => {
            flat_lift(v, target, acc)
        }
        // A cast to a narrower unsigned type keeps the low bits: the
        // value modulo 2^bits, whatever the source's width.
        (Ty::Nat(_), Ty::Nat(Some(b))) => flat_lift(
            format!("(prim remainder {v} {} 0 : Nat)", pow2(*b)),
            target,
            acc,
        ),
        (Ty::Int(a), Ty::Int(b)) if a <= b => flat_lift(v, target, acc),
        // A signed value widens to `i128` as itself; to `isize` where the
        // narrowest declared pointer holds its type, as `usize` does.
        (Ty::Int(a), Ty::Integer(wide))
            if wide.map_or(narrowest, Some).is_some_and(|w| *a <= w) =>
        {
            // The conversion of a fixed width to `Int` elaborates through
            // these two.
            let acc = acc.axiom("Quot.sound").axiom("propext");
            flat_lift(format!("(prim to_int {v} : Int)"), target, acc)
        }
        (Ty::Integer(a), Ty::Integer(b))
            if a == b || matches!((a.map_or(widest, Some), b), (Some(w), Some(b)) if w <= *b) =>
        {
            flat_lift(v, target, acc)
        }
        (Ty::Nat(_), Ty::Int(_)) => Err(Residue::Form(
            "a cast from an unsigned to a signed integer, where LexLean binds no \
             conversion from Nat to Int64"
                .to_string(),
        )),
        _ => Err(Residue::Form("narrowing or sign-changing cast".to_string())),
    }
}

/// `xs[i]`.
fn index_value(ix: &rs::ExprIndex, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let ((base, bt), acc) = flat_of(&ix.expr, env, ctx, acc, None)?;
    let Ty::List(elem) = &bt else {
        return Err(Residue::Form(format!("index of {}", bt.render())));
    };
    let ((i, it), acc) = flat_of(&ix.index, env, ctx, acc, Some(&Ty::Nat(None)))?;
    if !matches!(it, Ty::Nat(_)) {
        return Err(Residue::Form("index by a range".to_string()));
    }
    let Some(d) = default_of(elem, &env.types) else {
        return Err(Residue::Form("index of a type with no value".to_string()));
    };
    let (v, acc) = ctx.fresh(env, acc, "item");
    Ok((
        (
            Expr::Match {
                scrutinee: format!("(prim index {base} {i} : (Option {}))", elem.render()),
                arms: vec![
                    Arm {
                        pattern: "Option.none".into(),
                        body: Expr::Flat(d),
                    },
                    Arm {
                        pattern: format!("Option.some {v}"),
                        body: Expr::Flat(v),
                    },
                ],
            },
            (**elem).clone(),
        ),
        acc,
    ))
}

/// `matches!(e, P)`: a match with `true` for `P` and `false` otherwise.
fn matches_value(m: &rs::ExprMacro, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let Some(rs::MacroMatch {
        expr: scrutinee,
        pat,
    }) = &m.matches
    else {
        return Err(Residue::Form("matches! of another form".to_string()));
    };
    let yes = rs::RustExpr::Lit(rs::Lit::Bool(true));
    let no = rs::RustExpr::Lit(rs::Lit::Bool(false));
    let wild = rs::Pat::Wild;
    let first = match &**pat {
        rs::Pat::Guard(g) => (&*g.pat, Body::Guarded(&g.guard, &yes)),
        p => (p, Body::Expr(&yes)),
    };
    let arms = vec![first, (&wild, Body::Expr(&no))];
    matching(scrutinee, &arms, env, ctx, acc, Some(&Ty::Bool))
}

/// An array or `vec!` of items, as the list of them.
fn list_of(
    items: &[rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let mut elem = match want {
        Some(Ty::List(t) | Ty::Set(t, _)) => Some((**t).clone()),
        _ => None,
    };
    let mut parts = Vec::new();
    let mut acc = acc;
    for item in items {
        let ((v, vt), next) = flat_of(item, env, ctx, acc, elem.as_ref())?;
        acc = next;
        match &elem {
            Some(t) if !t.same(&vt) => {
                return Err(Residue::Form(
                    "a list of items of different types".to_string(),
                ))
            }
            Some(_) => {}
            None => elem = Some(vt),
        }
        parts.push(v);
    }
    let Some(elem) = elem else {
        return Err(Residue::Form(
            "an empty list of an unknown type".to_string(),
        ));
    };
    // A literal can be as long as its author likes, and the linker reads a
    // term to a bounded depth: the list is written as balanced halves.
    let list = crate::embed::term_list(&parts, &elem.render());
    Ok(((Expr::Flat(list), Ty::List(Box::new(elem))), acc))
}

/// Whether a format hole names a value: an identifier, without a format
/// specification.
fn is_identifier(hole: &str) -> bool {
    let mut chars = hole.chars();
    chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
        && hole != "_"
}

/// `write!(f, ..)` into the formatter of a `Display` impl: the text the
/// same `format!` makes, which the impl's lifted function returns.
fn write_macro(mac: &rs::ExprMacro, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let into_formatter = |e: &rs::RustExpr| match (e, &ctx.formatter) {
        (rs::RustExpr::Path(p), Some(f)) => p.is_ident(f),
        _ => false,
    };
    match mac.args.as_ref().map(|a| a.items.as_slice()) {
        Some([f, rest @ ..]) if into_formatter(f) => {
            let text = rs::ExprMacro {
                path: mac.path.clone(),
                args: Some(rs::MacroArgs {
                    items: rest.to_vec(),
                }),
                matches: None,
            };
            format_macro(&text, env, ctx, acc)
        }
        _ => Err(Residue::Form(
            "write! into something other than the formatter".to_string(),
        )),
    }
}

/// `format!` whose holes are all strings: the pieces joined.
fn format_macro(mac: &rs::ExprMacro, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let bad = |what: &str| Err(Residue::Form(format!("format! {what}")));
    let Some(rs::MacroArgs { items: args }) = &mac.args else {
        return bad("arguments");
    };
    let mut args = args.iter();
    let Some(rs::RustExpr::Lit(rs::Lit::Str(template))) = args.next() else {
        return bad("template");
    };
    let mut positional = Vec::new();
    let mut named = BTreeMap::new();
    for arg in args {
        match arg {
            rs::RustExpr::Assign(a) => {
                let rs::RustExpr::Path(p) = &*a.left else {
                    return bad("argument");
                };
                let Some(id) = p.get_ident() else {
                    return bad("argument");
                };
                named.insert(id.to_string(), &*a.right);
            }
            other => positional.push(other),
        }
    }
    let mut pieces: Vec<String> = Vec::new();
    let mut text = String::new();
    let mut next = 0;
    let mut acc = acc;
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                text.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                text.push('}');
            }
            '{' => {
                let mut hole = String::new();
                for h in chars.by_ref() {
                    if h == '}' {
                        break;
                    }
                    hole.push(h);
                }
                // `{b:02x}`: a byte as two lowercase hex digits, the only
                // format spec the lift reads.
                let (hole, hex) = match hole.split_once(':') {
                    Some((name, "02x")) => (name.to_string(), true),
                    Some(_) => return bad("hole"),
                    None => (hole, false),
                };
                if !text.is_empty() {
                    let Some(piece) = exact_string(&text) else {
                        return bad("text");
                    };
                    pieces.push(piece);
                    text.clear();
                }
                let (value, after) = if hole.is_empty() {
                    let Some(e) = positional.get(next) else {
                        return bad("hole");
                    };
                    next += 1;
                    flat_of(e, env, ctx, acc, Some(&Ty::Str))?
                } else if let Some(e) = named.get(&hole) {
                    flat_of(e, env, ctx, acc, Some(&Ty::Str))?
                } else if let Some((name, ty)) = ctx.locals.get(&hole) {
                    ((name.clone(), ty.clone()), acc)
                } else if is_identifier(&hole) {
                    // A constant named in the hole.
                    let path = rs::RustExpr::Path(rs::Path::ident(&hole));
                    flat_of(&path, env, ctx, acc, Some(&Ty::Str))?
                } else {
                    return bad("hole");
                };
                acc = after;
                match value.1 {
                    Ty::Nat(Some(8)) if hex => {
                        let (piece, after) = acc.prelude("hexByte", &[&value.0]);
                        acc = after;
                        pieces.push(piece);
                    }
                    _ if hex => return bad("hex of another type"),
                    Ty::Str | Ty::Char => pieces.push(value.0),
                    Ty::Bool => {
                        pieces.push(format!("(if {} then \"true\" else \"false\")", value.0))
                    }
                    Ty::Nat(_) => {
                        // A natural has as many digits as steps of division
                        // by ten, never more than itself and one: the fuel.
                        let (piece, after) = acc.prelude("natDecimal", &[&value.0]);
                        acc = after;
                        pieces.push(piece);
                    }
                    Ty::Int(_) => {
                        acc = acc.all_axioms();
                        pieces.push(format!("(prim format_decimal {} : String)", value.0));
                    }
                    _ => return bad("hole of another type"),
                }
            }
            '}' => return bad("template"),
            c => text.push(c),
        }
    }
    if !text.is_empty() {
        let Some(piece) = exact_string(&text) else {
            return bad("text");
        };
        pieces.push(piece);
    }
    // Joining strings elaborates through propositional extensionality.
    let acc = acc.axiom("propext");
    let list = crate::embed::term_list(&pieces, "String");
    Ok((
        (
            Expr::Flat(format!("(prim join {list} \"\" : String)")),
            Ty::Str,
        ),
        acc,
    ))
}

/// A comparison `==`, `!=`, `<`, `<=`, `>` or `>=`, apart from [`binary`]
/// so the frame of each stays small where expressions nest deeply.
#[inline(never)]
fn comparison(b: &rs::ExprBinary, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    use rs::BinOp as B;
    let flat = |s: String, t: Ty, acc: Acc| Ok(((Expr::Flat(s), t), acc));
    let op_text = || b.op.text().to_string();
    let ((l, lt), (r, _), acc) = if is_literal(&b.left) {
        let (right, acc) = flat_of(&b.right, env, ctx, acc, None)?;
        let (left, acc) = flat_of(&b.left, env, ctx, acc, Some(&right.1))?;
        (left, right, acc)
    } else {
        let (left, acc) = flat_of(&b.left, env, ctx, acc, None)?;
        let (right, acc) = flat_of(&b.right, env, ctx, acc, Some(&left.1))?;
        (left, right, acc)
    };
    let (text, acc) = match (&lt, b.op) {
        (Ty::Nat(_), B::Eq) => (format!("({l} =? {r})"), acc),
        (Ty::Nat(_), B::Ne) => (format!("(not ({l} =? {r}))"), acc),
        (Ty::Nat(_), B::Lt) => (format!("({l} <? {r})"), acc),
        (Ty::Nat(_), B::Le) => (format!("({l} <=? {r})"), acc),
        (Ty::Nat(_), B::Gt) => (format!("({r} <? {l})"), acc),
        (Ty::Nat(_), B::Ge) => (format!("({r} <=? {l})"), acc),
        (Ty::Bool | Ty::Named(_), B::Eq) if is_enum_or_bool(&lt, env) => {
            (same_constructor(&lt, &l, &r), acc)
        }
        (Ty::Bool | Ty::Named(_), B::Ne) if is_enum_or_bool(&lt, env) => {
            (format!("(not {})", same_constructor(&lt, &l, &r)), acc)
        }
        (Ty::Int(_) | Ty::Str | Ty::Char, B::Eq) => (format!("(prim equal {l} {r} : Bool)"), acc),
        (Ty::Int(_) | Ty::Str | Ty::Char, B::Ne) => {
            (format!("(not (prim equal {l} {r} : Bool))"), acc)
        }
        // The mathematical `Int` has an order and no `equal`: two values
        // are the same where neither is below the other.
        // IEEE 754's comparisons, a NaN unordered: the prelude's model.
        (Ty::Float(f), _) => {
            let op = match b.op {
                B::Eq => "Eq",
                B::Ne => "Ne",
                B::Lt => "Lt",
                B::Le => "Le",
                B::Gt => "Gt",
                _ => "Ge",
            };
            float_term(*f, op, &[&l, &r], acc)
        }
        (Ty::Integer(_) | Ty::Unbounded, _) => {
            let acc = acc.axiom("propext");
            let below = |x: &str, y: &str| format!("(prim less_than {x} {y} : Bool)");
            let text = match b.op {
                B::Lt => below(&l, &r),
                B::Gt => below(&r, &l),
                B::Le => format!("(not {})", below(&r, &l)),
                B::Ge => format!("(not {})", below(&l, &r)),
                B::Eq => format!("(not ({} || {}))", below(&l, &r), below(&r, &l)),
                _ => format!("({} || {})", below(&l, &r), below(&r, &l)),
            };
            (text, acc)
        }
        // Strings order by their scalar values, as Rust's `str` and
        // `char` do.
        (Ty::Int(_) | Ty::Str | Ty::Char, B::Lt | B::Gt | B::Le | B::Ge) => {
            // Int64's order is decided through propositional
            // extensionality; a string's through its scalars.
            let acc = acc.axiom("propext");
            let acc = if matches!(lt, Ty::Int(_)) {
                acc
            } else {
                acc.all_axioms()
            };
            let (x, y) = match b.op {
                B::Lt | B::Ge => (&l, &r),
                _ => (&r, &l),
            };
            let less = format!("(prim less_than {x} {y} : Bool)");
            if matches!(b.op, B::Lt | B::Gt) {
                (less, acc)
            } else {
                (format!("(not {less})"), acc)
            }
        }
        (Ty::Option(inner), B::Eq | B::Ne) => {
            let (x, acc) = ctx.fresh(env, acc, "left");
            let (y, acc) = ctx.fresh(env, acc, "right");
            let (left, acc) = ctx.fresh(env, acc, "leftOption");
            let (right, acc) = ctx.fresh(env, acc, "rightOption");
            let Some((same, acc)) = same_value(inner, &x, &y, env, acc) else {
                return Err(Residue::Form(format!(
                    "comparison {} of {}",
                    op_text(),
                    lt.render()
                )));
            };
            let eq = matches!(b.op, B::Eq);
            let (yes, no) = if eq {
                ("true", "false")
            } else {
                ("false", "true")
            };
            let same = if eq { same } else { format!("(not {same})") };
            let none_r = Expr::Match {
                scrutinee: right.clone(),
                arms: vec![
                    Arm {
                        pattern: "Option.none".into(),
                        body: Expr::Flat(yes.into()),
                    },
                    Arm {
                        pattern: format!("Option.some {y}"),
                        body: Expr::Flat(no.into()),
                    },
                ],
            };
            let some_r = Expr::Match {
                scrutinee: right.clone(),
                arms: vec![
                    Arm {
                        pattern: "Option.none".into(),
                        body: Expr::Flat(no.into()),
                    },
                    Arm {
                        pattern: format!("Option.some {y}"),
                        body: Expr::Flat(same),
                    },
                ],
            };
            return Ok((
                (
                    wrap_lets(
                        vec![(left.clone(), lt.render(), l), (right, lt.render(), r)],
                        Expr::Match {
                            scrutinee: left,
                            arms: vec![
                                Arm {
                                    pattern: "Option.none".into(),
                                    body: none_r,
                                },
                                Arm {
                                    pattern: format!("Option.some {x}"),
                                    body: some_r,
                                },
                            ],
                        },
                    ),
                    Ty::Bool,
                ),
                acc,
            ));
        }
        // Any other type compares by its derived order.
        _ => match crate::order::operator(&op_text(), &lt, &l, &r) {
            Some(test) if crate::order::derived(&lt, &env.types) => {
                (test, acc.ordered(&lt, &env.types))
            }
            _ => {
                return Err(Residue::Form(format!(
                    "comparison {} of {}",
                    op_text(),
                    lt.render()
                )))
            }
        },
    };
    flat(text, Ty::Bool, acc)
}

fn binary(
    b: &rs::ExprBinary,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    use rs::BinOp as B;
    match b.op {
        B::And | B::Or => logic(b, env, ctx, acc),
        B::Eq | B::Ne | B::Lt | B::Le | B::Gt | B::Ge => comparison(b, env, ctx, acc),
        // `s + t` of a `String` and a `&str`: the two joined.
        B::Add | B::Sub | B::Mul | B::Div | B::Rem
            if float_operand(&b.left, env, ctx, &acc)
                || float_operand(&b.right, env, ctx, &acc)
                || (matches!(want, Some(Ty::Float(_)))
                    && (float_literal_operand(&b.left) || float_literal_operand(&b.right))) =>
        {
            float_arithmetic(b, env, ctx, acc, want)
        }
        B::Add | B::Sub | B::Mul | B::Div | B::Rem
            if unbounded_operand(&b.left, env, ctx, &acc)
                || unbounded_operand(&b.right, env, ctx, &acc) =>
        {
            unbounded_arithmetic(b, env, ctx, acc)
        }
        B::Add if text_operand(&b.left, env, ctx, &acc) => text_join(b, env, ctx, acc),
        B::Add => nat_sum(b, env, ctx, acc),
        // `a - k` where the conditions in scope bound `a` below by `k` or
        // more: it does not underflow, so it is the natural difference.
        B::Sub => nat_difference(b, env, ctx, acc),
        B::Div | B::Rem => nat_division(b, env, ctx, acc),
        _ => Err(Residue::Arithmetic(b.op.text().to_string())),
    }
}

/// `a && b` and `a || b`.
fn logic(b: &rs::ExprBinary, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let ((l, lt), acc) = flat_of(&b.left, env, ctx, acc, Some(&Ty::Bool))?;
    let ((r, rt), acc) = flat_of(&b.right, env, ctx, acc, Some(&Ty::Bool))?;
    if lt != Ty::Bool || rt != Ty::Bool {
        return Err(Residue::Form("logic on non-booleans".to_string()));
    }
    let op = if matches!(b.op, rs::BinOp::And) {
        "&&"
    } else {
        "||"
    };
    flat_lift(format!("({l} {op} {r})"), Ty::Bool, acc)
}

/// `s + t` of a `String` and a `&str`: the two joined.
fn text_join(b: &rs::ExprBinary, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let ((l, _), acc) = flat_of(&b.left, env, ctx, acc, Some(&Ty::Str))?;
    let ((r, rt), acc) = flat_of(&b.right, env, ctx, acc, Some(&Ty::Str))?;
    if rt != Ty::Str {
        return Err(Residue::Form("+ of a string and another type".to_string()));
    }
    let acc = acc.axiom("propext");
    flat_lift(
        format!("(prim join (cons {l} (cons {r} (nil String))) \"\" : String)"),
        Ty::Str,
        acc,
    )
}

/// `a + b` of naturals the conditions in scope keep in range.
fn nat_sum(b: &rs::ExprBinary, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let want = Ty::Nat(None);
    let ((l, lt), acc) = flat_of(&b.left, env, ctx, acc, Some(&want))?;
    let ((r, rt), acc) = flat_of(&b.right, env, ctx, acc, Some(&want))?;
    if lt != want || rt != want {
        return Err(Residue::Arithmetic(b.op.text().to_string()));
    }
    flat_lift(
        usize_sum(&b.left, &b.right, &l, &r, env, &ctx.capped)?,
        want,
        acc,
    )
}

/// `a - k` where the conditions in scope bound `a` below by `k` or
/// more: it does not underflow, so it is the natural difference.
fn nat_difference(b: &rs::ExprBinary, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let ((l, lt), acc) = flat_of(&b.left, env, ctx, acc, Some(&Ty::Nat(None)))?;
    match (&lt, int_literal(&b.right)) {
        (Ty::Nat(_), Some(k)) if ctx.bounds.get(&l).is_some_and(|m| *m >= k) => {
            flat_lift(format!("(prim subtract {l} {k} : Nat)"), lt, acc)
        }
        _ => Err(Residue::Arithmetic(b.op.text().to_string())),
    }
}

/// Whether an operand is an integer of no width (see [`Ty::Unbounded`]).
fn unbounded_operand(e: &rs::RustExpr, env: &Env, ctx: &Ctx, acc: &Acc) -> bool {
    matches!(
        flat_of(e, env, ctx, acc.clone(), None),
        Ok(((_, Ty::Unbounded), _))
    )
}

/// `a + b`, `a - b`, `a * b`, `a / b` and `a % b` of integers of no width:
/// the crate's operators are exact (see [`crate::registered::readings`]),
/// and a literal operand, whichever scalar type Rust infers for it, is
/// promoted to the same integer. `/` and `%` truncate toward zero, as
/// LexLean's `Int` quotient and remainder do, and are read only by a
/// divisor that is a literal other than zero: the crate panics on a zero
/// divisor (num-bigint 0.4.8, `src/biguint/division.rs`), where LexLean's
/// total quotient returns its default, so a divisor that may be zero
/// stays outside.
fn unbounded_arithmetic(
    b: &rs::ExprBinary,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    use rs::BinOp as B;
    let want = Ty::Unbounded;
    let ((l, lt), acc) = flat_of(&b.left, env, ctx, acc, Some(&want))?;
    let ((r, rt), acc) = flat_of(&b.right, env, ctx, acc, Some(&want))?;
    if lt != want || rt != want {
        return Err(Residue::Arithmetic(b.op.text().to_string()));
    }
    if matches!(b.op, B::Div | B::Rem) && !int_literal(&b.right).is_some_and(|k| k > 0) {
        return Err(Residue::Form(format!(
            "{} of an integer of no width by a divisor that may be zero, where Rust panics",
            b.op.text()
        )));
    }
    let text = match b.op {
        B::Add => format!("(prim subtract {l} (prim negate {r} : Int) : Int)"),
        B::Sub => format!("(prim subtract {l} {r} : Int)"),
        B::Mul => format!("(prim multiply {l} {r} : Int)"),
        B::Div => format!("(prim quotient {l} {r} (0 : Int) : Int)"),
        _ => format!("(prim remainder {l} {r} (0 : Int) : Int)"),
    };
    flat_lift(text, want, acc)
}

/// Whether an operand is a float literal without a suffix, whose format
/// its use decides.
fn float_literal_operand(e: &rs::RustExpr) -> bool {
    match e {
        rs::RustExpr::Lit(rs::Lit::Float { suffix, .. }) => suffix.is_empty(),
        rs::RustExpr::Paren(p) => float_literal_operand(p),
        rs::RustExpr::Unary(u) => u.op == rs::UnOp::Neg && float_literal_operand(&u.expr),
        _ => false,
    }
}

/// Whether an expression is a float literal whose value is not zero in any
/// format: its magnitude is at least the least normal binary32, core's
/// `f32::MIN_POSITIVE`, which every float type of the language holds as a
/// nonzero value.
fn nonzero_float_literal(e: &rs::RustExpr) -> bool {
    match e {
        rs::RustExpr::Lit(rs::Lit::Float { digits, .. }) => digits
            .replace('_', "")
            .parse::<f64>()
            .is_ok_and(|v| v.abs() >= f64::from(f32::MIN_POSITIVE)),
        rs::RustExpr::Paren(p) => nonzero_float_literal(p),
        rs::RustExpr::Unary(u) => u.op == rs::UnOp::Neg && nonzero_float_literal(&u.expr),
        _ => false,
    }
}

/// Whether an operand is a float of the language.
fn float_operand(e: &rs::RustExpr, env: &Env, ctx: &Ctx, acc: &Acc) -> bool {
    matches!(
        flat_of(e, env, ctx, acc.clone(), None),
        Ok(((_, Ty::Float(_)), _))
    )
}

/// The prelude's IEEE 754 operation `op` of a format (see
/// [`crate::mir::float`]), with the axioms its definition states.
fn float_term(f: crate::mir::float::Format, op: &str, args: &[&str], acc: Acc) -> (String, Acc) {
    let axioms = prelude_axioms(&format!("float{op}"));
    (
        f.term(op, args),
        acc.axioms_all(axioms.into_iter().collect()),
    )
}

/// A float literal of a format: the prelude's value of its decimal digits
/// (`crate::mir::float::literal`), negated where written so.
fn float_literal(f: crate::mir::float::Format, digits: &str, negative: bool) -> Lift<(Expr, Ty)> {
    let text = if negative {
        format!("-{digits}")
    } else {
        digits.to_string()
    };
    match crate::mir::float::literal(f, &text) {
        Some(term) => Ok((Expr::Flat(term), Ty::Float(f))),
        None => Err(Residue::Form("float literal".to_string())),
    }
}

/// `a + b`, `a - b`, `a * b`, `a / b` and `a % b` of floats of one
/// format: IEEE 754's correctly rounded operations, to nearest with ties
/// to even, which Rust's are, and `%` the remainder truncated toward zero
/// that Rust's `%` on floats is; the prelude's model (see
/// [`crate::mir::float`]), the MIR path's too. The format is an operand's
/// that has one, else the one the expression's use asks for (`let y: f32 =
/// 0.1 + 0.2`): an unsuffixed literal takes its type from where it is
/// used, as Rust's inference gives it.
fn float_arithmetic(
    b: &rs::ExprBinary,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    used: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    use rs::BinOp as B;
    let typed = [&b.left, &b.right]
        .into_iter()
        .find(|e| float_operand(e, env, ctx, &acc))
        .and_then(|e| flat_of(e, env, ctx, acc.clone(), None).ok())
        .map(|((_, t), _)| t);
    let Some(want) = typed.or_else(|| used.filter(|t| matches!(t, Ty::Float(_))).cloned()) else {
        return Err(Residue::Arithmetic(b.op.text().to_string()));
    };
    let ((l, lt), acc) = flat_of(&b.left, env, ctx, acc, Some(&want))?;
    let ((r, rt), acc) = flat_of(&b.right, env, ctx, acc, Some(&want))?;
    let Ty::Float(f) = want else {
        return Err(Residue::Arithmetic(b.op.text().to_string()));
    };
    if lt != want || rt != want {
        return Err(Residue::Arithmetic(b.op.text().to_string()));
    }
    let op = match b.op {
        B::Add => "Add",
        B::Sub => "Sub",
        B::Mul => "Mul",
        B::Div => "Div",
        _ => "Rem",
    };
    let (term, acc) = float_term(f, op, &[&l, &r], acc);
    flat_lift(term, want, acc)
}

/// `a / k` and `a % k` by a divisor that is not zero.
fn nat_division(b: &rs::ExprBinary, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let ((l, lt), acc) = flat_of(&b.left, env, ctx, acc, None)?;
    match (&lt, int_literal(&b.right)) {
        (Ty::Nat(_), Some(k)) if k > 0 => {
            let prim = if matches!(b.op, rs::BinOp::Div) {
                "quotient"
            } else {
                "remainder"
            };
            flat_lift(format!("(prim {prim} {l} {k} 0 : Nat)"), lt, acc)
        }
        // A divisor the conditions in scope bound away from zero.
        (Ty::Nat(_), None) => {
            let ((r, rt), acc) = flat_of(&b.right, env, ctx, acc, Some(&lt))?;
            if !rt.same(&lt) || !ctx.bounds.get(&r).is_some_and(|m| *m > 0) {
                return Err(Residue::Arithmetic(b.op.text().to_string()));
            }
            let prim = if matches!(b.op, rs::BinOp::Div) {
                "quotient"
            } else {
                "remainder"
            };
            flat_lift(format!("(prim {prim} {l} {r} 0 : Nat)"), lt, acc)
        }
        _ => Err(Residue::Arithmetic(b.op.text().to_string())),
    }
}

/// Whether an operand lifts to text: the left side of a `String` `+`.
fn text_operand(e: &rs::RustExpr, env: &Env, ctx: &Ctx, acc: &Acc) -> bool {
    matches!(
        flat_of(e, env, ctx, acc.clone(), None),
        Ok(((_, Ty::Str), _))
    )
}

/// Equality of two values of a unit-variant enum, through the enum's own
/// equality definition (`=?` is `Nat`'s), or of two `Bool`s, by cases on the
/// left.
fn same_constructor(ty: &Ty, l: &str, r: &str) -> String {
    match ty {
        Ty::Named(n) => format!("({} {l} {r})", equality_name(n)),
        _ => format!("(if {l} then {r} else (not {r}))"),
    }
}

/// The definition that decides equality of a unit-variant enum's values,
/// which the projection emits after the enum for every enum a lifted body
/// compares.
pub(crate) fn equality_name(enum_name: &str) -> String {
    #[cfg(not(uorust_lift))]
    crate::certified::certified!(lift_equality_name, enum_name);
    format!("equal{enum_name}")
}

fn is_enum_or_bool(t: &Ty, env: &Env) -> bool {
    match t {
        Ty::Bool => true,
        Ty::Named(n) => env.types.enums.contains_key(n),
        _ => false,
    }
}

/// A call of a closure a `let` bound: its body, in the scope it was
/// written in, with each parameter bound to its argument.
fn applied_closure(
    closure: &Closure,
    args: &[rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    if closure.inputs.len() != args.len() {
        return Err(Residue::Form("a closure call of another arity".to_string()));
    }
    let mut scope = ctx.clone();
    scope.locals = closure.locals.clone();
    // The binders of the call's scope stay taken, so a parameter's binder
    // does not shadow one an argument names.
    scope
        .hidden
        .extend(ctx.locals.values().map(|(name, _)| name.clone()));
    let mut lets = Vec::new();
    let mut acc = acc;
    for (input, arg) in closure.inputs.iter().zip(args) {
        let (pat, declared) = match input {
            rs::Pat::Type(pt) => (
                &*pt.pat,
                Some(env.types.ty(&pt.ty.localized(&ctx.imports))?),
            ),
            p => (p, None),
        };
        let ((value, ty), after) = flat_of(arg, env, &ctx.below(), acc, declared.as_ref())?;
        acc = after;
        if declared.as_ref().is_some_and(|d| !d.same(&ty)) {
            return Err(Residue::Form(
                "a closure argument of another type".to_string(),
            ));
        }
        let rust = match pat {
            rs::Pat::Ident(i) if i.subpat.is_none() => i.ident.clone(),
            rs::Pat::Reference(r) => match &**r {
                rs::Pat::Ident(i) if i.subpat.is_none() => i.ident.clone(),
                _ => return Err(Residue::Form("closure pattern".to_string())),
            },
            rs::Pat::Wild => continue,
            _ => return Err(Residue::Form("closure pattern".to_string())),
        };
        let (next, name, after) = scope.bind(env, acc, &rust, ty.clone());
        scope = next;
        acc = after;
        lets.push((name, ty.render(), value));
    }
    let ((e, t), acc) = expr(&closure.body, env, &scope.below(), acc, want)?;
    Ok(((wrap_lets(lets, e), t), acc))
}

fn call(
    c: &rs::ExprCall,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let rs::RustExpr::Path(p) = &*c.func else {
        return Err(Residue::Form("call of an expression".to_string()));
    };
    if let Some(closure) = p.get_ident().and_then(|i| ctx.closures.get(i)) {
        return applied_closure(closure, &c.args, env, ctx, acc, want);
    }
    // `f(x)` of a function parameter: its application.
    if let Some((f, Ty::Fn(inputs, output))) = p.get_ident().and_then(|i| ctx.locals.get(i)) {
        return applied_param(c, f, inputs, output, env, ctx, acc);
    }
    if let (Some(w), [arg]) = (wrapper(p), c.args.as_slice()) {
        return wrapped_value(w, arg, env, ctx, acc, want);
    }
    // `E::V(a, b)`: a variant of a data-carrying enum.
    if let Some((owner, variant, fields)) = data_variant(p, &env.types) {
        return variant_call(c, owner, &variant, fields, env, ctx, acc);
    }
    if let Some((name, fields)) = tuple_struct(p, c.args.len(), env, ctx) {
        return tuple_struct_value(c, &name, fields, env, ctx, acc);
    }
    path_call(c, p, env, ctx, acc, want)
}

/// The tuple struct a call's path names, `S(a, b)` or `Self(a, b)`, with
/// its fields: a struct of as many fields as the call has arguments, each
/// named by its position. Rust reads the name as the struct's constructor
/// before any function, since both live in the one value namespace.
fn tuple_struct<'e>(
    p: &rs::Path,
    arity: usize,
    env: &'e Env,
    ctx: &Ctx,
) -> Option<(String, &'e [(String, Ty)])> {
    let written = p.get_ident()?;
    let owner = ctx.current.split_once("::").map(|(o, _)| o);
    let name = match (written.as_str(), owner) {
        ("Self", Some(o)) => env.types.resolve(o),
        _ => env.types.resolve(written),
    };
    let fields = env.types.structs.get(&name)?;
    let positional = fields
        .iter()
        .enumerate()
        .all(|(i, (f, _))| *f == field_name(&format!("{i}")));
    (positional && fields.len() == arity).then_some((name, fields.as_slice()))
}

/// `S(a, b)`: a value of a tuple struct, its fields in order.
fn tuple_struct_value(
    c: &rs::ExprCall,
    name: &str,
    fields: &[(String, Ty)],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let mut acc = acc;
    let mut values = Vec::new();
    for (arg, (field, ty)) in c.args.iter().zip(fields) {
        let arg = match env.types.cell(name, field) {
            Some(kind) => cell_made(arg, kind, name, field)?,
            None => arg,
        };
        let ((v, vt), next) = flat_of(arg, env, ctx, acc, Some(ty))?;
        if !vt.same(ty) {
            return Err(Residue::Form(format!("{name} of another type")));
        }
        acc = next;
        values.push(v);
    }
    let parts: Vec<(&String, String)> = fields.iter().map(|(f, _)| f).zip(values).collect();
    flat_lift(
        struct_value(name, &parts, &env.types),
        Ty::Named(name.to_string()),
        acc,
    )
}

/// `f(x)` of a function parameter `f`: its application.
fn applied_param(
    c: &rs::ExprCall,
    f: &str,
    inputs: &[Ty],
    output: &Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    if inputs.len() != c.args.len() {
        return Err(Residue::Form(
            "a function applied to another arity".to_string(),
        ));
    }
    let mut parts = vec!["apply".to_string(), f.to_string()];
    let mut acc = acc;
    for (arg, ty) in c.args.iter().zip(inputs) {
        let ((v, vt), next) = flat_of(arg, env, &ctx.below(), acc, Some(ty))?;
        if !vt.same(ty) {
            return Err(Residue::Form(
                "a function applied to another type".to_string(),
            ));
        }
        acc = next;
        parts.push(v);
    }
    flat_lift(format!("({})", parts.join(" ")), output.clone(), acc)
}

/// `Some(v)`, `Ok(v)` and `Err(e)`.
fn wrapped_value(
    w: &str,
    arg: &rs::RustExpr,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    match (w, want) {
        ("Some", _) => {
            let inner = match want {
                Some(Ty::Option(t)) => Some(&**t),
                _ => None,
            };
            let ((v, vt), acc) = flat_of(arg, env, ctx, acc, inner)?;
            flat_lift(
                format!("(ctor Option.some [{}] {v})", vt.render()),
                Ty::Option(Box::new(vt)),
                acc,
            )
        }
        ("Ok", Some(t @ Ty::Outcome(ok, err))) | ("Err", Some(t @ Ty::Outcome(ok, err))) => {
            let (ctor, inner) = if w == "Ok" {
                ("ok", &**ok)
            } else {
                ("error", &**err)
            };
            let ((v, vt), acc) = flat_of(arg, env, ctx, acc, Some(inner))?;
            if !vt.same(inner) {
                return Err(Residue::Form("result of another type".to_string()));
            }
            flat_lift(
                format!(
                    "(ctor Result.{ctor} [{}, {}] {v})",
                    ok.render(),
                    err.render()
                ),
                t.clone(),
                acc,
            )
        }
        _ => Err(Residue::Form("result of an unknown type".to_string())),
    }
}

/// `E::V(a, b)`: a variant of a data-carrying enum.
fn variant_call(
    c: &rs::ExprCall,
    owner: String,
    variant: &str,
    fields: &[DataField],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    if fields.len() != c.args.len() {
        return Err(Residue::Form(format!(
            "{owner}::{variant} of another arity"
        )));
    }
    let mut parts = vec![format!("ctor {owner}.{variant}")];
    let mut acc = acc;
    for (arg, (_, ty)) in c.args.iter().zip(fields) {
        let ((v, vt), next) = flat_of(arg, env, ctx, acc, Some(ty))?;
        acc = next;
        if !vt.same(ty) {
            return Err(Residue::Form(format!("{owner}::{variant} of another type")));
        }
        parts.push(v);
    }
    flat_lift(format!("({})", parts.join(" ")), Ty::Named(owner), acc)
}

/// A call of a path: a constructor of the standard library the lift
/// knows, or a lifted function.
fn path_call(
    c: &rs::ExprCall,
    p: &rs::Path,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let segments: Vec<String> = p.segments.iter().map(|s| s.ident.to_string()).collect();
    let call_args: Vec<&rs::RustExpr> = c.args.iter().collect();
    // `Sha256::digest(bytes)`: the SHA-256 of a byte list, which the prelude
    // computes.
    if let ([.., owner, method], [bytes]) = (segments.as_slice(), call_args.as_slice()) {
        let byte_list = Ty::List(Box::new(Ty::Nat(Some(8))));
        let named = match segments.len() {
            2 => ctx.imports.get(owner).cloned(),
            _ => Some(segments[..segments.len() - 1].join("::")),
        };
        if method == "digest" && named.as_deref() == Some("sha2::Sha256") {
            let ((v, vt), acc) = flat_of(bytes, env, ctx, acc, Some(&byte_list))?;
            if vt != byte_list {
                return Err(Residue::Form(
                    "a digest of something other than bytes".into(),
                ));
            }
            let (s, acc) = acc.prelude("sha256", &[&v]);
            return flat_lift(s, byte_list, acc);
        }
    }
    let (known, acc) = std_call(&segments, &call_args, env, ctx, acc, want)?;
    if let Some(lifted) = known {
        return Ok((lifted, acc));
    }
    let (known, acc) = unbounded_call(&segments, &call_args, env, ctx, acc)?;
    if let Some(lifted) = known {
        return Ok((lifted, acc));
    }
    fn_call(c, &segments, env, ctx, acc)
}

/// The traits of `num-traits` whose associated function a call through an
/// integer of no width names, each with the paths a `use` may bind it by
/// (the crate's root re-exports its `identities` module's traits).
const UNBOUNDED_IDENTITIES: &[(&str, &str, &[&str])] = &[
    (
        "zero",
        "(0 : Int)",
        &["num_traits::Zero", "num_traits::identities::Zero"],
    ),
    (
        "one",
        "(1 : Int)",
        &["num_traits::One", "num_traits::identities::One"],
    ),
];

/// A constructor of an integer of no width, by the type's path
/// (`BigInt::from(n)`): the reading's crate defines each (see
/// [`crate::registered::readings`]). `from` of a signed integer of the
/// language is that integer, exactly, as the crate's `From` of every
/// signed width is, and of the type itself is the value; `default` is
/// zero. `zero` and `one` are the `num-traits` identities, which name the
/// crate's own implementation where the file's `use` brings the trait into
/// scope: the type declares no associated function of either name, so a
/// call that compiles names the trait in scope, and Rust refuses one where
/// another trait in scope gives the name too.
fn unbounded_call(
    segments: &[String],
    call_args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let [owner, function] = segments else {
        return Ok((None, acc));
    };
    match env.types.registered(&env.types.resolve(owner)) {
        Some(Ty::Unbounded) => {}
        Some(Ty::Ratio) => return ratio_call(owner, function, call_args, env, ctx, acc),
        _ => return Ok((None, acc)),
    }
    let want = Ty::Unbounded;
    match (function.as_str(), call_args) {
        ("from", [a]) => {
            let ((v, vt), acc) = flat_of(a, env, ctx, acc, None)?;
            match vt {
                Ty::Unbounded | Ty::Integer(_) => flat_some(v, want, acc),
                Ty::Nat(_) => {
                    let word = crate::mir::int::Int::of(&vt, &env.widths).ok_or_else(|| {
                        Residue::Form("unsigned constructor without a word width".into())
                    })?;
                    flat_some(
                        crate::registered::word_integer(&v, word.held, false),
                        want,
                        acc,
                    )
                }
                // The conversion of a fixed width to `Int` elaborates
                // through these two.
                Ty::Int(_) => {
                    let acc = acc.axiom("Quot.sound").axiom("propext");
                    flat_some(format!("(prim to_int {v} : Int)"), want, acc)
                }
                other => Err(Residue::Form(format!(
                    "{owner}::from of {}, where LexLean binds no conversion to Int",
                    other.render()
                ))),
            }
        }
        ("default", []) => flat_some("(0 : Int)".into(), want, acc),
        (name, []) => match identity(name, ctx) {
            Some(value) => flat_some(value.into(), want, acc),
            None => Ok((None, acc)),
        },
        _ => Ok((None, acc)),
    }
}

/// The value of a `num-traits` identity (`zero`, `one`) a `use` of the
/// function's own module scope brings into scope. The path is the crate's
/// only where the scope binds no item of the path's head itself (a local
/// `mod num_traits` is what a relative path names first); and the call is
/// the trait's only where no trait of the workspace declares a function
/// of the name, which a call of a trait in scope could reach instead, and
/// no macro of the package makes items the tool does not read.
fn identity(name: &str, ctx: &Ctx) -> Option<&'static str> {
    if ctx.seen.trait_methods.contains(name) || ctx.seen.macro_items {
        return None;
    }
    // A path from the crate graph's root (`::num_traits`) names the crate
    // whatever the scope binds.
    let crate_path = |p: &str| match p.strip_prefix("::") {
        Some(rooted) => (rooted.to_string(), true),
        None => {
            let head = p.split("::").next().unwrap_or(p);
            (p.to_string(), !ctx.seen.local.contains(head))
        }
    };
    UNBOUNDED_IDENTITIES
        .iter()
        .find(|(n, _, _)| *n == name)
        .filter(|(_, _, paths)| {
            ctx.seen.uses.values().any(|p| {
                let (p, from_crate) = crate_path(p);
                from_crate && paths.contains(&p.as_str())
            })
        })
        .map(|(_, value, _)| *value)
}

/// A constructor of a ratio by the type's path, as num-rational 0.4.2's
/// `src/lib.rs` defines each on the stored pair: `new_raw(n, d)` stores
/// the pair as given; `from_integer(n)` and `from(n)` of an integer are
/// `new_raw(n, 1)`; `zero()` is `new_raw(0, 1)`, `one()` is
/// `new_raw(1, 1)`, and `default()` is `zero()`. `new` and `from` of a pair
/// reduce by the greatest common divisor, which LexLean's `Int` gives no
/// operation for: they stay outside.
fn ratio_call(
    owner: &str,
    function: &str,
    call_args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let integer = Ty::Unbounded;
    let pair = |n: &str, d: &str| format!("({n}, {d})");
    match (function, call_args) {
        ("new_raw", [n, d]) => {
            let ((n, nt), acc) = flat_of(n, env, ctx, acc, Some(&integer))?;
            let ((d, dt), acc) = flat_of(d, env, ctx, acc, Some(&integer))?;
            if nt != integer || dt != integer {
                return Err(Residue::Form(format!("{owner}::new_raw of another type")));
            }
            flat_some(pair(&n, &d), Ty::Ratio, acc)
        }
        ("from_integer" | "from", [n]) => {
            let ((n, nt), acc) = flat_of(n, env, ctx, acc, Some(&integer))?;
            if nt != integer {
                return Err(Residue::Form(format!(
                    "{owner}::{function} of {}, which reduces by a greatest common divisor \
                     LexLean's Int gives no operation for",
                    nt.render()
                )));
            }
            flat_some(pair(&n, "(1 : Int)"), Ty::Ratio, acc)
        }
        ("default", []) => flat_some(pair("(0 : Int)", "(1 : Int)"), Ty::Ratio, acc),
        (name, []) => match identity(name, ctx) {
            Some(value) => flat_some(pair(value, "(1 : Int)"), Ty::Ratio, acc),
            None => Ok((None, acc)),
        },
        ("new", [_, _]) => Err(Residue::Form(format!(
            "{owner}::new, which reduces by a greatest common divisor LexLean's Int gives \
             no operation for"
        ))),
        _ => Ok((None, acc)),
    }
}

/// A constructor of the standard library the lift knows, by its path's
/// segments: `String::new()`, `Vec::new()`, `Box::new(v)` and the like.
fn std_call(
    segments: &[String],
    call_args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    match (segments, call_args, want) {
        ([s, n], [], _) if s == "String" && n == "new" => flat_some("\"\"".into(), Ty::Str, acc),
        ([s, n], [], Some(t @ Ty::List(inner))) if s == "Vec" && n == "new" => {
            flat_some(format!("(nil {})", inner.render()), t.clone(), acc)
        }
        // An empty ordered set or map: the empty list of its members or of
        // its entries, which `insert` keeps sorted and free of duplicates.
        ([s, n], [], Some(t @ Ty::Set(inner, true))) if s == "BTreeSet" && n == "new" => {
            flat_some(format!("(nil {})", inner.render()), t.clone(), acc)
        }
        ([s, n], [], Some(t @ Ty::Map(_, _, true))) if s == "BTreeMap" && n == "new" => {
            flat_some(format!("(nil {})", t.entry().render()), t.clone(), acc)
        }
        // `std::slice::from_ref(&x)`: the list of the one value.
        ([.., s, n], [a], _) if s == "slice" && n == "from_ref" => {
            let elem = match want {
                Some(Ty::List(t)) => Some(&**t),
                _ => None,
            };
            let ((v, vt), acc) = flat_of(a, env, ctx, acc, elem)?;
            let list = Ty::List(Box::new(vt.clone()));
            flat_some(format!("(cons {v} (nil {}))", vt.render()), list, acc)
        }
        // A capacity is a hint to the allocator: the value is the empty list.
        ([s, n], [_], Some(t @ Ty::List(inner))) if s == "Vec" && n == "with_capacity" => {
            flat_some(format!("(nil {})", inner.render()), t.clone(), acc)
        }
        // A box is its value: LexLean holds every value by reference.
        // `BTreeSet::from([..])`: each member inserted in turn, from empty.
        ([s, n], [items @ rs::RustExpr::Array(_)], _) if s == "BTreeSet" && n == "from" => {
            let ((v, vt), acc) = flat_of(items, env, ctx, acc, want)?;
            let Ty::List(elem) = &vt else {
                return Err(Residue::Form("BTreeSet::from of another type".into()));
            };
            let set = Ty::Set(elem.clone(), true);
            let (insert, acc) = sorted_insert(&set, env, ctx, acc)?;
            let (helper, acc) = insert_all(&set, &vt, &insert, env, ctx, acc);
            flat_some(format!("({helper} {v} (nil {}))", elem.render()), set, acc)
        }
        // `Path::new(s)`, `PathBuf::from(s)`: the path the text names.
        ([s, n], [a], _) | ([_, _, s, n], [a], _)
            if (s == "Path" && n == "new" || s == "PathBuf" && n == "from")
                && (segments.len() == 2 || segments[..2] == ["std", "path"])
                && !env.types.structs.contains_key(s.as_str()) =>
        {
            let ((v, vt), acc) = flat_of(a, env, ctx, acc, Some(&Ty::Str))?;
            if vt != Ty::Str {
                return Err(Residue::Form("a path of something other than text".into()));
            }
            let (path, acc) = acc.prelude("pathOf", &[&v]);
            flat_some(path, Ty::Path, acc)
        }
        // An owning pointer is what it owns (see `Types::ty`): building one
        // is its content, and so is another owner of the same content.
        ([s, n], [a], _)
            if matches!(s.as_str(), "Box" | "Rc" | "Arc") && matches!(n.as_str(), "new") =>
        {
            let ((v, vt), acc) = flat_of(a, env, ctx, acc, want)?;
            flat_some(v, vt, acc)
        }
        ([s, n], [a], _)
            if matches!(s.as_str(), "Rc" | "Arc") && n == "clone" && !env.types.declares(s) =>
        {
            let ((v, vt), acc) = flat_of(a, env, ctx, acc, want)?;
            flat_some(v, vt, acc)
        }
        ([s, n], [a], _) if s == "String" && n == "from" => {
            let ((v, vt), acc) = flat_of(a, env, ctx, acc, Some(&Ty::Str))?;
            if vt == Ty::Str {
                return flat_some(v, Ty::Str, acc);
            }
            Err(Residue::Form("String::from of another type".to_string()))
        }
        _ => Ok((None, acc)),
    }
}

/// A call of a lifted function, or of the function itself where the call
/// descends.
fn fn_call(
    c: &rs::ExprCall,
    segments: &[String],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    // `T::m(x, ..)` of a type parameter `T`: the dictionary's method.
    if let [owner, method] = segments {
        let held = match env.types.generic.get(owner) {
            Some(Ty::Param(p)) => ctx
                .dictionary
                .iter()
                .find(|e| &e.param == p && e.method == *method),
            _ => None,
        };
        if let Some(entry) = held {
            let args: Vec<&rs::RustExpr> = c.args.iter().collect();
            return dictionary_call(entry, None, &args, env, ctx, acc);
        }
    }
    let mut name = segments.last().cloned().unwrap_or_default();
    if let [.., owner, method] = segments {
        let owner = if owner == "Self" {
            ctx.current
                .split_once("::")
                .map_or(owner.as_str(), |(o, _)| o)
        } else {
            owner.as_str()
        };
        let key = format!("{}::{method}", env.types.resolve(owner));
        if env.fns.contains_key(&key) {
            name = key;
        }
    }
    let Some(sig) = env.fns.get(&name) else {
        return Err(Residue::Call(name));
    };
    if !sig.threads.is_empty() {
        return Err(threaded_operand(&name));
    }
    if sig.params.len() != c.args.len() {
        return Err(Residue::Call(name));
    }
    if let Some(generic) = &sig.generic {
        return generic_call(c, &name, sig, generic, env, ctx, acc);
    }
    let mut parts = vec![sig.name.clone()];
    let mut descends = None;
    let mut acc = acc;
    for (i, (arg, ty)) in c.args.iter().zip(&sig.params).enumerate() {
        let ((v, vt), next) = flat_of(arg, env, ctx, acc, Some(ty))?;
        acc = next;
        if !vt.same(ty) {
            return Err(Residue::Form(format!("argument of another type to {name}")));
        }
        if name == ctx.current
            && ctx
                .params
                .get(i)
                .is_some_and(|parameter| ctx.tails.get(&v) == Some(parameter))
            && descends.is_none()
        {
            descends = Some(i);
        }
        parts.push(v);
    }
    let acc = if name == ctx.current {
        let Some(i) = descends else {
            return Err(Residue::Form(
                "recursion that does not descend on a slice".to_string(),
            ));
        };
        acc.descended(i)
    } else {
        acc.called(name)
    };
    flat_lift(format!("({})", parts.join(" ")), sig.result.clone(), acc)
}

/// A dictionary's method applied: to the receiver and the arguments of a
/// method call, or to the arguments of a path call `T::m(x, ..)`, which
/// pass the receiver first.
fn dictionary_call(
    entry: &Entry,
    receiver: Option<String>,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let Ty::Fn(inputs, output) = &entry.ty else {
        return Err(Residue::Form(
            "a dictionary method of no function type".to_string(),
        ));
    };
    let mut parts = vec!["apply".to_string(), entry.name.clone()];
    let rest = match &receiver {
        Some(r) => {
            parts.push(r.clone());
            &inputs[1..]
        }
        None => &inputs[..],
    };
    if rest.len() != args.len() {
        return Err(Residue::Form(
            "a dictionary method applied to another arity".to_string(),
        ));
    }
    let mut acc = acc;
    for (arg, ty) in args.iter().zip(rest) {
        let ((v, vt), next) = flat_of(arg, env, &ctx.below(), acc, Some(ty))?;
        if !vt.same(ty) {
            return Err(Residue::Form(
                "a dictionary method applied to another type".to_string(),
            ));
        }
        acc = next;
        parts.push(v);
    }
    flat_lift(format!("({})", parts.join(" ")), (**output).clone(), acc)
}

/// The instantiation `bind` extends so that `pattern` (a type that may
/// name type parameters) is `actual`, or `false` where none does.
fn instantiate(pattern: &Ty, actual: &Ty, bind: &mut BTreeMap<String, Ty>) -> bool {
    match (pattern, actual) {
        // One type argument is one Rust type: widths and orders count,
        // though `Nat` renders every unsigned width alike.
        (Ty::Param(p), _) => match bind.get(p) {
            Some(held) => held == actual,
            None => {
                bind.insert(p.clone(), actual.clone());
                true
            }
        },
        (Ty::Option(a), Ty::Option(b))
        | (Ty::List(a), Ty::List(b))
        | (Ty::Set(a, _), Ty::Set(b, _)) => instantiate(a, b, bind),
        (Ty::Outcome(a, e), Ty::Outcome(b, f)) | (Ty::Map(a, e, _), Ty::Map(b, f, _)) => {
            instantiate(a, b, bind) && instantiate(e, f, bind)
        }
        (Ty::Fn(xs, r), Ty::Fn(ys, s)) => {
            xs.len() == ys.len()
                && xs.iter().zip(ys).all(|(x, y)| instantiate(x, y, bind))
                && instantiate(r, s, bind)
        }
        (Ty::Tuple(xs), Ty::Tuple(ys)) | (Ty::Pair(xs), Ty::Pair(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys).all(|(a, b)| instantiate(a, b, bind))
        }
        _ => pattern.same(actual) && !pattern.render().contains('\''),
    }
}

/// `pattern` with each type parameter `bind` fixes put in its place.
pub(crate) fn instantiated(pattern: &Ty, bind: &BTreeMap<String, Ty>) -> Ty {
    let go = |t: &Ty| Box::new(instantiated(t, bind));
    match pattern {
        Ty::Param(p) => bind.get(p).cloned().unwrap_or_else(|| pattern.clone()),
        Ty::Option(t) => Ty::Option(go(t)),
        Ty::List(t) => Ty::List(go(t)),
        Ty::Set(t, o) => Ty::Set(go(t), *o),
        Ty::Outcome(t, e) => Ty::Outcome(go(t), go(e)),
        Ty::Map(k, v, o) => Ty::Map(go(k), go(v), *o),
        Ty::Fn(ins, out) => Ty::Fn(ins.iter().map(|t| instantiated(t, bind)).collect(), go(out)),
        Ty::Tuple(ts) => Ty::Tuple(ts.iter().map(|t| instantiated(t, bind)).collect()),
        Ty::Pair(ts) => Ty::Pair(ts.iter().map(|t| instantiated(t, bind)).collect()),
        other => other.clone(),
    }
}

/// The declared generic carrier after substitution, retaining its product
/// choices. A bound opaque parameter keeps its own admitted representation.
fn model_carrier(ty: &Ty, bind: &BTreeMap<String, Ty>) -> Ty {
    match ty {
        Ty::Map(..) => Ty::List(Box::new(model_carrier(&ty.entry(), bind))),
        Ty::List(t) => Ty::List(Box::new(model_carrier(t, bind))),
        Ty::Option(t) => Ty::Option(Box::new(model_carrier(t, bind))),
        Ty::Outcome(t, e) => Ty::Outcome(
            Box::new(model_carrier(t, bind)),
            Box::new(model_carrier(e, bind)),
        ),
        Ty::Tuple(ts) => Ty::Tuple(ts.iter().map(|t| model_carrier(t, bind)).collect()),
        Ty::Pair(ts) => Ty::Pair(ts.iter().map(|t| model_carrier(t, bind)).collect()),
        Ty::Fn(ts, r) => Ty::Fn(
            ts.iter().map(|t| model_carrier(t, bind)).collect(),
            Box::new(model_carrier(r, bind)),
        ),
        _ => instantiated(ty, bind),
    }
}

/// A structural transport between admitted carriers. It changes no key, value
/// or ordering; unsupported callable/state conversions remain residue.
fn carrier_transport(
    value: &str,
    from: &Ty,
    to: &Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(String, Acc)> {
    if from.same(to) {
        return Ok((value.into(), acc));
    }
    let from = match from {
        Ty::Map(..) => Ty::List(Box::new(from.entry())),
        _ => from.clone(),
    };
    let to = match to {
        Ty::Map(..) => Ty::List(Box::new(to.entry())),
        _ => to.clone(),
    };
    let mut acc = acc.saw(&from).saw(&to);
    let (input, next) = ctx.fresh(env, acc, "carrierValue");
    acc = next;
    let before = acc.helpers.len();
    let mut recursive = false;
    let mut list_tail = None;
    let body = match (&from, &to) {
        (Ty::Tuple(fs) | Ty::Pair(fs), Ty::Tuple(ts) | Ty::Pair(ts)) if fs.len() == ts.len() => {
            let mut values = Vec::new();
            for (i, (f, t)) in fs.iter().zip(ts).enumerate() {
                let (v, next) =
                    carrier_transport(&tuple_item(&from, &input, i), f, t, env, ctx, acc)?;
                acc = next;
                values.push(v);
            }
            Expr::Flat(tuple_value(&to, &values))
        }
        (Ty::List(f), Ty::List(t)) => {
            let (head, next) = ctx.fresh(env, acc, "carrierHead");
            acc = next;
            let (tail, next) = ctx.fresh(env, acc, "carrierTail");
            acc = next;
            let (converted, next) = carrier_transport(&head, f, t, env, ctx, acc)?;
            acc = next;
            recursive = true;
            list_tail = Some(tail.clone());
            Expr::Match {
                scrutinee: input.clone(),
                arms: vec![
                    Arm {
                        pattern: "List.nil".into(),
                        body: Expr::Flat(format!("(nil {})", t.render())),
                    },
                    Arm {
                        pattern: format!("List.cons {head} {tail}"),
                        body: Expr::Flat(converted),
                    },
                ],
            }
        }
        (Ty::Option(f), Ty::Option(t)) => {
            let (head, next) = ctx.fresh(env, acc, "carrierSome");
            acc = next;
            let (converted, next) = carrier_transport(&head, f, t, env, ctx, acc)?;
            acc = next;
            Expr::Match {
                scrutinee: input.clone(),
                arms: vec![
                    Arm {
                        pattern: "Option.none".into(),
                        body: Expr::Flat(format!("(ctor Option.none [{}])", t.render())),
                    },
                    Arm {
                        pattern: format!("Option.some {head}"),
                        body: Expr::Flat(format!(
                            "(ctor Option.some [{}] {converted})",
                            t.render()
                        )),
                    },
                ],
            }
        }
        (Ty::Outcome(f, fe), Ty::Outcome(t, te)) => {
            let (head, next) = ctx.fresh(env, acc, "carrierResult");
            acc = next;
            let (ok, next) = carrier_transport(&head, f, t, env, ctx, acc)?;
            acc = next;
            let (err, next) = carrier_transport(&head, fe, te, env, ctx, acc)?;
            acc = next;
            let params = format!("[{}, {}]", t.render(), te.render());
            Expr::Match {
                scrutinee: input.clone(),
                arms: vec![
                    Arm {
                        pattern: format!("Result.ok {head}"),
                        body: Expr::Flat(format!("(ctor Result.ok {params} {ok})")),
                    },
                    Arm {
                        pattern: format!("Result.error {head}"),
                        body: Expr::Flat(format!("(ctor Result.error {params} {err})")),
                    },
                ],
            }
        }
        _ => {
            return Err(Residue::Form(format!(
                "no structural carrier transport from {} to {}",
                from.render(),
                to.render()
            )))
        }
    };
    let identity = crate::project::sha(format!("{from:?}\0{to:?}").as_bytes());
    let name = ctx.helper_name(env, &acc, &format!("Carrier{identity}"));
    let body = match (body, list_tail) {
        (
            Expr::Match {
                scrutinee,
                mut arms,
            },
            Some(tail),
        ) => {
            let Expr::Flat(head) = &arms[1].body else {
                return Err(Residue::Form("a non-value carrier element".into()));
            };
            arms[1].body = Expr::Flat(format!("(cons {head} ({name} {tail}))"));
            Expr::Match { scrutinee, arms }
        }
        (body, _) => body,
    };
    let calls = acc.helpers[before..]
        .iter()
        .map(|h| format!("#{}", h.name))
        .collect();
    let acc = acc
        .helper(Helper {
            name: name.clone(),
            params: vec![(input, from.render())],
            result: to.render(),
            body,
            calls,
            axioms: BTreeSet::new(),
            recursive,
            entwined: false,
            termination: None,
        })
        .called(format!("#{name}"));
    Ok((format!("({name} {value})"), acc))
}

/// A call of a generic function: the type arguments its arguments fix, in
/// the order of its type parameters, then for each method of its
/// dictionary the one the instantiating type supplies, then its own
/// arguments. A type parameter of the caller passes on the caller's own
/// dictionary method; a type of the crate passes the method its impl of
/// the trait gives, the one impl coherence admits. Any other type, a
/// method an impl leaves to the trait's default body, and a type
/// parameter no argument fixes stop the call.
///
/// An unsuffixed number literal has no type of its own: Rust gives it the
/// type the call's other arguments fix (`first(200, x)` with `x: u8` makes
/// `200` a `u8`). Such arguments are therefore lifted after the others,
/// against the type those fixed; a type parameter only such literals fix
/// takes its type from the call's context, which the lift does not read,
/// and stops the call.
fn generic_call(
    c: &rs::ExprCall,
    name: &str,
    sig: &Sig,
    generic: &Generic,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let mut bind: BTreeMap<String, Ty> = BTreeMap::new();
    let mut lifted: Vec<Option<(String, Ty)>> = vec![None; c.args.len().min(sig.params.len())];
    let mut descends = None;
    let mut acc = acc;
    let pairs: Vec<(usize, (&rs::RustExpr, &Ty))> =
        c.args.iter().zip(&sig.params).enumerate().collect();
    let (inferred, fixed): (Vec<_>, Vec<_>) = pairs
        .into_iter()
        .partition(|(_, (arg, ty))| untyped_number(arg) && ty.render().contains('\''));
    for (i, (arg, ty)) in fixed.into_iter().chain(inferred) {
        let wanted = instantiated(ty, &bind);
        let typed = !wanted.render().contains('\'');
        if !typed && untyped_number(arg) {
            return Err(Residue::Form(format!(
                "a call of {name} whose type parameter only an unsuffixed number literal fixes"
            )));
        }
        let want = typed.then_some(&wanted);
        let ((v, vt), next) = flat_of(arg, env, ctx, acc, want)?;
        acc = next;
        if !instantiate(ty, &vt, &mut bind) {
            return Err(Residue::Form(format!("argument of another type to {name}")));
        }
        if name == ctx.current && ctx.tails.get(&v) == ctx.params.get(i) && descends.is_none() {
            descends = Some(i);
        }
        lifted[i] = Some((v, vt));
    }
    let mut args = Vec::new();
    for ((value, actual), formal) in lifted.into_iter().flatten().zip(&sig.params) {
        let target = model_carrier(formal, &bind);
        let (value, next) = carrier_transport(&value, &actual, &target, env, ctx, acc)?;
        acc = next;
        args.push(value);
    }
    let mut types = Vec::new();
    for p in &generic.types {
        let Some(t) = bind.get(p) else {
            return Err(Residue::Form(format!(
                "a call of {name} whose arguments do not fix its type parameter {p}"
            )));
        };
        types.push(t.render());
    }
    if name == ctx.current {
        // No polymorphic recursion: a recursive call passes the
        // definition's own type parameters and dictionary.
        let own = generic
            .types
            .iter()
            .all(|p| bind.get(p) == Some(&Ty::Param(p.clone())));
        let Some(i) = descends.filter(|_| own) else {
            return Err(Residue::Form(
                "recursion that does not descend on a slice".to_string(),
            ));
        };
        acc = acc.descended(i);
    } else {
        acc = acc.called(name.to_string());
    }
    let mut dictionary = Vec::new();
    for e in &generic.dictionary {
        let wanted = instantiated(&e.ty, &bind);
        match &bind[&e.param] {
            Ty::Param(q) => {
                let Some(own) = ctx
                    .dictionary
                    .iter()
                    .find(|d| &d.param == q && d.declarer == e.declarer && d.method == e.method)
                else {
                    return Err(Residue::Call(format!(
                        "{}::{} of a type parameter whose dictionary does not give it",
                        e.declarer, e.method
                    )));
                };
                dictionary.push(own.name.clone());
            }
            Ty::Named(o) => {
                let key = (e.declarer.clone(), o.clone(), e.method.clone());
                let Some((rust, method)) = env
                    .types
                    .impls
                    .get(&key)
                    .and_then(|k| env.fns.get(k).map(|s| (k, s)))
                else {
                    return Err(Residue::Call(format!(
                        "{}::{} of {o}, which no impl method of {o} that lifts gives",
                        e.declarer, e.method
                    )));
                };
                let given = Ty::Fn(method.params.clone(), Box::new(method.result.clone()));
                if method.generic.is_some() || !given.same(&wanted) {
                    return Err(Residue::Form(format!(
                        "{}::{} of {o} of another type",
                        e.declarer, e.method
                    )));
                }
                acc = acc.called(rust.clone());
                dictionary.push(format!("(ref {})", method.name));
            }
            other => {
                return Err(Residue::Call(format!(
                    "{}::{} of {}, a type no impl of the crate is read for",
                    e.declarer,
                    e.method,
                    other.render()
                )))
            }
        }
    }
    let mut parts = vec![sig.name.clone()];
    if !types.is_empty() {
        parts.push(format!("[{}]", types.join(", ")));
    }
    parts.extend(dictionary);
    parts.extend(args);
    let result = instantiated(&sig.result, &bind);
    let modeled = model_carrier(&sig.result, &bind);
    let (term, acc) = carrier_transport(
        &format!("({})", parts.join(" ")),
        &modeled,
        &result,
        env,
        ctx,
        acc,
    )?;
    flat_lift(term, result, acc)
}

/// A generic model term, result, external dependencies and structural declarations.
pub(crate) type ModelBinding = (String, Ty, BTreeSet<String>, Vec<Decl>);

/// Bind compiler values with the same generic argument and coherent dictionary
/// mechanism used by source calls. This imports an existing definition; it
/// neither supplies a missing body nor assumes a trait method's behavior.
pub(crate) fn bind_model_call(
    sig: &Sig,
    args: &[(String, Ty)],
    types: &Types,
    fns: &BTreeMap<String, Sig>,
    widths: &[(u32, Vec<String>)],
    names: &BTreeSet<String>,
) -> Lift<ModelBinding> {
    let generic = sig
        .generic
        .as_ref()
        .ok_or_else(|| Residue::Form("a compiler generic call with no generic signature".into()))?;
    if !sig.threads.is_empty() || sig.params.len() != args.len() {
        return Err(Residue::Form(
            "a compiler generic call of another interface".into(),
        ));
    }
    let locals: BTreeMap<String, (String, Ty)> = args
        .iter()
        .enumerate()
        .map(|(i, arg)| (format!("modelArg{i}"), arg.clone()))
        .collect();
    let call: syn::ExprCall = syn::parse_str(&format!(
        "compilerDependency({})",
        (0..args.len())
            .map(|i| format!("modelArg{i}"))
            .collect::<Vec<_>>()
            .join(", ")
    ))
    .map_err(|_| Residue::Form("a compiler generic call whose arguments cannot be bound".into()))?;
    let converted = crate::convert::expr(&syn::Expr::Call(call));
    let rs::RustExpr::Call(call) = &converted else {
        return Err(Residue::Form(
            "a compiler generic call of no call expression".into(),
        ));
    };
    let env = Env {
        total: true,
        types: types.clone(),
        fns: fns.clone(),
        widths: widths.to_vec(),
        names: names.clone(),
    };
    let ctx = Ctx {
        current: String::new(),
        params: Vec::new(),
        result: sig.result.clone(),
        return_states: Vec::new(),
        own: "compilerDependency".into(),
        locals,
        tails: BTreeMap::new(),
        top: false,
        looping: None,
        hidden: Vec::new(),
        imports: BTreeMap::new(),
        seen: Seen::default(),
        closures: BTreeMap::new(),
        elements: None,
        rebuilt: BTreeMap::new(),
        bounds: BTreeMap::new(),
        capped: BTreeMap::new(),
        exits: None,
        formatter: None,
        dictionary: Vec::new(),
        sealed: BTreeSet::new(),
        borrowed: BTreeSet::new(),
    };
    let ((term, result), acc) = generic_call(
        call,
        "compilerDependency",
        sig,
        generic,
        &env,
        &ctx,
        Acc::default(),
    )?;
    if !acc.axioms.is_empty()
        || acc.helpers.iter().any(|h| {
            !h.axioms.is_empty()
                || h.entwined
                || h.termination.is_some()
                || h.calls.iter().any(|c| !c.starts_with('#'))
        })
    {
        return Err(Residue::Form(
            "a compiler generic binding needing additional declarations".into(),
        ));
    }
    let calls = acc
        .calls
        .iter()
        .filter(|key| !key.starts_with('#'))
        .map(|key| {
            if key == "compilerDependency" {
                Ok(sig.name.clone())
            } else {
                fns.get(key)
                    .map(|s| s.name.clone())
                    .ok_or_else(|| Residue::Call(key.clone()))
            }
        })
        .collect::<Lift<BTreeSet<_>>>()?;
    let declarations = acc
        .helpers
        .into_iter()
        .map(|h| Decl::Definition {
            type_parameters: Vec::new(),
            recursive: h.recursive.then(|| h.params[0].0.clone()),
            name: h.name,
            parameters: h.params,
            result: h.result,
            body: h.body,
            axioms: Vec::new(),
            termination: None,
            mutual: None,
        })
        .collect();
    Ok((flat_text(term)?, result, calls, declarations))
}

/// The module of the byte-level text prelude every projection carries.
pub(crate) const PRELUDE: &str = "Prelude";

/// The prelude's source: the definitions string methods lift to. Its SHA-256
/// is ported from UOR-Foundation/compression, `src/Uorc/Integrity/Sha256.lex.tex`,
/// under the Apache License 2.0: the round functions, constants and word
/// loading are that module's, renamed; the padding, block walk and digest
/// read a whole message at once.
pub(crate) const PRELUDE_SOURCE: &str = include_str!("../assets/project/src/Prelude.lex.tex");

/// The prelude definitions the kernel cannot compute by unfolding: those
/// that reach a primitive Lean implements opaquely, directly or through
/// another prelude definition.
fn prelude_opaque() -> BTreeSet<String> {
    let decls: Vec<(&str, &str)> = PRELUDE_SOURCE
        .split("\ndefinition{\n  name: ")
        .skip(1)
        .filter_map(|decl| {
            let (name, rest) = decl.split_once(",\n")?;
            Some((name, rest.split("\n}\n").next().unwrap_or(rest)))
        })
        .collect();
    let mut opaque: BTreeSet<String> = decls
        .iter()
        .filter(|(_, body)| KERNEL_OPAQUE.iter().any(|p| body.contains(p)))
        .map(|(name, _)| (*name).to_string())
        .collect();
    loop {
        let more: Vec<String> = decls
            .iter()
            .filter(|(name, body)| {
                !opaque.contains(*name)
                    && body
                        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                        .any(|token| opaque.contains(token))
            })
            .map(|(name, _)| (*name).to_string())
            .collect();
        if more.is_empty() {
            return opaque;
        }
        opaque.extend(more);
    }
}

/// The prelude definitions a lifted body calls.
fn prelude_calls(text: &str) -> Vec<String> {
    let marker = format!("{PRELUDE}::");
    text.split(marker.as_str())
        .skip(1)
        .map(|rest| {
            rest.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .next()
                .unwrap_or("")
                .to_string()
        })
        .collect()
}

/// The Lean axioms a prelude definition elaborates with, as its source
/// declares them.
fn prelude_axioms(name: &str) -> Vec<String> {
    let Some(start) = PRELUDE_SOURCE.find(&format!("  name: {name},\n")) else {
        return Vec::new();
    };
    let decl = &PRELUDE_SOURCE[start..];
    let decl = &decl[..decl.find("\n}\n").unwrap_or(decl.len())];
    decl.find("  axioms: [")
        .and_then(|i| {
            let list = &decl[i + "  axioms: [".len()..];
            list.find(']').map(|j| &list[..j])
        })
        .map(|list| {
            list.split(',')
                .map(|a| a.trim().to_string())
                .filter(|a| !a.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// The byte list `bytes` of `bytes.iter().map(|b| format!("{b:02x}"))`.
fn hex_of(e: &rs::RustExpr) -> Option<&rs::RustExpr> {
    let rs::RustExpr::MethodCall(map) = e else {
        return None;
    };
    let [rs::RustExpr::Closure(f)] = map.args.as_slice() else {
        return None;
    };
    let rs::RustExpr::MethodCall(iter) = &*map.receiver else {
        return None;
    };
    if map.method != "map" || iter.method != "iter" || !iter.args.is_empty() {
        return None;
    }
    let [rs::Pat::Ident(b)] = f.inputs.as_slice() else {
        return None;
    };
    let rs::RustExpr::Macro(mac) = &*f.body else {
        return None;
    };
    let hole = format!("{{{}:02x}}", b.ident);
    let named = match mac.args.as_ref().map(|a| a.items.as_slice()) {
        Some([rs::RustExpr::Lit(rs::Lit::Str(t))]) => *t == hole,
        Some([rs::RustExpr::Lit(rs::Lit::Str(t)), rs::RustExpr::Path(p)]) => {
            t == "{:02x}" && p.is_ident(&b.ident)
        }
        _ => false,
    };
    (mac.path.is_ident("format") && named && b.subpat.is_none() && !b.by_ref)
        .then_some(&*iter.receiver)
}

/// The methods of `str` and `char` over the prelude: `None` when the method
/// is none of them.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn text_method(
    m: &rs::ExprMethodCall,
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let flat = |s: String, t: Ty, acc: Acc| Ok((Some((Expr::Flat(s), t)), acc));
    // A pattern: a string or a character.
    let pattern = |a: &rs::RustExpr, acc: Acc| -> Lift<(String, Acc)> {
        let ((v, vt), acc) = flat_of(a, env, ctx, acc, Some(&Ty::Str))?;
        match vt {
            Ty::Str | Ty::Char => Ok((v, acc)),
            _ => Err(Residue::Form(format!("a .{name} pattern that is not text"))),
        }
    };
    let opt = |t: Ty| Ty::Option(Box::new(t));
    let list = |t: Ty| Ty::List(Box::new(t));
    let chars = rt == &Ty::Char;
    match (name, args) {
        ("len", []) if !chars => {
            let (s, acc) = acc.prelude("textLength", &[r]);
            flat(s, Ty::Nat(None), acc)
        }
        ("as_str" | "as_ref" | "into" | "to_owned" | "borrow", []) => {
            flat(r.to_string(), rt.clone(), acc)
        }
        ("to_string", []) => flat(r.to_string(), Ty::Str, acc),
        ("trim", []) => {
            let (s, acc) = acc.prelude("textTrim", &[r]);
            flat(s, Ty::Str, acc)
        }
        ("trim_start", []) => {
            let (s, acc) = acc.prelude("textTrimStart", &[r]);
            flat(s, Ty::Str, acc)
        }
        ("trim_end", []) => {
            let (s, acc) = acc.prelude("textTrimEnd", &[r]);
            flat(s, Ty::Str, acc)
        }
        // A predicate on the first or last character.
        ("starts_with" | "ends_with", [f]) if !chars && matches!(f, rs::RustExpr::Closure(_)) => {
            let (lam, acc) = lambda(f, &[Ty::Char], env, ctx, acc, Some(&Ty::Bool))?;
            if lam.ty != Ty::Bool {
                return Err(Residue::Form(format!(
                    "a .{name} predicate of another type"
                )));
            }
            let acc = acc.called_all(lam.calls.clone());
            let acc = acc.axioms_all(lam.axioms.clone());
            let (all, acc) = acc.prelude("textChars", &[r]);
            let at = if name == "starts_with" {
                "0".to_string()
            } else {
                format!("(prim subtract (prim length {all} : Nat) 1 : Nat)")
            };
            Ok((
                Some((
                    Expr::Match {
                        scrutinee: format!("(prim index {all} {at} : (Option String))"),
                        arms: vec![
                            Arm {
                                pattern: "Option.none".into(),
                                body: Expr::Flat("false".into()),
                            },
                            Arm {
                                pattern: format!("Option.some {}", lam.binders[0]),
                                body: lam.wrap(lam.core.clone()),
                            },
                        ],
                    },
                    Ty::Bool,
                )),
                acc,
            ))
        }
        ("starts_with" | "ends_with" | "contains", [p]) if !chars => {
            let (p, acc) = pattern(p, acc)?;
            let f = match name {
                "starts_with" => "textStartsWith",
                "ends_with" => "textEndsWith",
                _ => "textContains",
            };
            {
                let (s, acc) = acc.prelude(f, &[r, &p]);
                flat(s, Ty::Bool, acc)
            }
        }
        ("strip_prefix" | "strip_suffix", [p]) if !chars => {
            let (p, acc) = pattern(p, acc)?;
            let f = if name == "strip_prefix" {
                "textStripPrefix"
            } else {
                "textStripSuffix"
            };
            {
                let (s, acc) = acc.prelude(f, &[r, &p]);
                flat(s, opt(Ty::Str), acc)
            }
        }
        ("find" | "rfind", [p]) if !chars => {
            let (p, acc) = pattern(p, acc)?;
            let f = if name == "find" {
                "textFirst"
            } else {
                "textLast"
            };
            {
                let (s, acc) = acc.prelude(f, &[r, &p]);
                flat(s, opt(Ty::Nat(None)), acc)
            }
        }
        ("split" | "rsplit", [p]) if !chars => {
            let (p, acc) = pattern(p, acc)?;
            let f = if name == "split" {
                "textSplit"
            } else {
                "textRsplit"
            };
            {
                let (s, acc) = acc.prelude(f, &[r, &p]);
                flat(s, list(Ty::Str), acc)
            }
        }
        ("split_once" | "rsplit_once", [p]) if !chars => {
            let (p, acc) = pattern(p, acc)?;
            let f = if name == "split_once" {
                "textFirst"
            } else {
                "textLast"
            };
            let pair = Ty::Tuple(vec![Ty::Str, Ty::Str]);
            let acc = acc.saw(&pair);
            let (at, acc) = ctx.fresh(env, acc, "at");
            let (whole, acc) = acc.prelude("textLength", &[r]);
            let (width, acc) = acc.prelude("textLength", &[&p]);
            let (before, acc) = acc.prelude("textSlice", &[r, "0", &at]);
            let (after, acc) = acc.prelude("textSlice", &[r, &format!("({at} + {width})"), &whole]);
            let (scrutinee, acc) = acc.prelude(f, &[r, &p]);
            let body = Expr::Match {
                scrutinee,
                arms: vec![Arm { pattern: "Option.none".into(), body: Expr::Flat(format!("(ctor Option.none [{}])", pair.render())) }, Arm { pattern: format!("Option.some {at}"), body: Expr::Flat(format!(
                            "(ctor Option.some [{0}] (record {0} {{item0 := {before}, item1 := {after}}}))",
                            pair.render()
                        )) }],
            };
            Ok((Some((body, opt(pair))), acc))
        }
        ("replace", [from, to]) if !chars => {
            let (from, acc) = pattern(from, acc)?;
            let ((to, tt), acc) = flat_of(to, env, ctx, acc, Some(&Ty::Str))?;
            if !matches!(tt, Ty::Str | Ty::Char) {
                return Err(Residue::Form("a replacement that is not text".into()));
            }
            {
                let (s, acc) = acc.prelude("textReplace", &[r, &from, &to]);
                flat(s, Ty::Str, acc)
            }
        }
        ("trim_start_matches" | "trim_end_matches" | "trim_matches", [p]) if !chars => {
            let (p, acc) = pattern(p, acc)?;
            let f = match name {
                "trim_start_matches" => "textTrimStartMatches",
                "trim_end_matches" => "textTrimEndMatches",
                _ => "textTrimMatches",
            };
            {
                let (s, acc) = acc.prelude(f, &[r, &p]);
                flat(s, Ty::Str, acc)
            }
        }
        ("lines", []) if !chars => {
            let (s, acc) = acc.prelude("textLines", &[r]);
            flat(s, list(Ty::Str), acc)
        }
        ("split_whitespace", []) if !chars => {
            let (s, acc) = acc.prelude("textWords", &[r]);
            flat(s, list(Ty::Str), acc)
        }
        ("chars", []) if !chars => {
            let (s, acc) = acc.prelude("textChars", &[r]);
            flat(s, list(Ty::Char), acc)
        }
        ("bytes" | "as_bytes", []) if !chars => {
            let (s, acc) = acc.prelude("textBytes", &[r]);
            flat(s, list(Ty::Nat(Some(8))), acc)
        }
        ("repeat", [n]) if !chars => {
            let ((n, nt), acc) = flat_of(n, env, ctx, acc, Some(&Ty::Nat(None)))?;
            if !matches!(nt, Ty::Nat(_)) {
                return Err(Residue::Form("a repeat count that is not a number".into()));
            }
            {
                let (s, acc) = acc.prelude("textRepeat", &[&n, r]);
                flat(s, Ty::Str, acc)
            }
        }
        ("to_ascii_lowercase", []) => {
            let (s, acc) = acc.prelude("textAsciiLower", &[r]);
            flat(s, rt.clone(), acc)
        }
        // No Unicode case table is bound in LexLean or the SDK: the ASCII
        // letters lower as Rust lowers them, and the definition records
        // that it diverges on the others.
        ("to_lowercase", []) => {
            let (s, acc) = acc.prelude("textAsciiLower", &[r]);
            flat(s, rt.clone(), acc.axiom(&format!("{DIVERGES}{LOWERCASE}")))
        }
        ("to_ascii_uppercase", []) => {
            let (s, acc) = acc.prelude("textAsciiUpper", &[r]);
            flat(s, rt.clone(), acc)
        }
        ("eq_ignore_ascii_case", [other]) => {
            let ((o, ot), acc) = flat_of(other, env, ctx, acc, Some(rt))?;
            if !ot.same(rt) {
                return Err(Residue::Form("eq_ignore_ascii_case of another type".into()));
            }
            let (a, acc) = acc.prelude("textAsciiLower", &[r]);
            let (b, acc) = acc.prelude("textAsciiLower", &[&o]);
            flat(format!("(prim equal {a} {b} : Bool)"), Ty::Bool, acc)
        }
        ("is_ascii", []) => {
            let (s, acc) = acc.prelude("textIsAscii", &[r]);
            flat(s, Ty::Bool, acc)
        }
        (
            "is_ascii_digit"
            | "is_ascii_uppercase"
            | "is_ascii_lowercase"
            | "is_ascii_alphabetic"
            | "is_ascii_alphanumeric"
            | "is_ascii_hexdigit"
            | "is_ascii_whitespace"
            | "is_ascii_punctuation"
            | "is_ascii_control"
            | "is_whitespace"
            | "is_control",
            [],
        ) if chars => {
            let f = match name {
                "is_ascii_digit" => "textIsAsciiDigit",
                "is_ascii_uppercase" => "textIsAsciiUppercase",
                "is_ascii_lowercase" => "textIsAsciiLowercase",
                "is_ascii_alphabetic" => "textIsAsciiAlphabetic",
                "is_ascii_alphanumeric" => "textIsAsciiAlphanumeric",
                "is_ascii_hexdigit" => "textIsAsciiHexdigit",
                "is_ascii_whitespace" => "textIsAsciiWhitespace",
                "is_ascii_punctuation" => "textIsAsciiPunctuation",
                "is_ascii_control" => "textIsAsciiControl",
                "is_whitespace" => "textIsWhitespace",
                _ => "textIsControl",
            };
            {
                let (s, acc) = acc.prelude(f, &[r]);
                flat(s, Ty::Bool, acc)
            }
        }
        ("parse", []) if !chars => {
            let target = match &m.turbofish {
                Some(t) => match t.as_slice() {
                    [rs::GenericArgument::Type(t)] => {
                        Some(env.types.ty(&t.localized(&ctx.imports))?)
                    }
                    _ => None,
                },
                None => match want {
                    Some(Ty::Outcome(t, _) | Ty::Option(t)) => Some((**t).clone()),
                    _ => None,
                },
            };
            let bits = match target {
                Some(Ty::Nat(Some(bits))) if bits <= 64 => bits,
                Some(Ty::Nat(None)) => return Err(Residue::Width(".parse into usize".to_string())),
                _ => return Err(Residue::Form("parse into a type outside the lift".into())),
            };
            let (parsed, acc) = acc.prelude("textParseNat", &[r, max_of(bits)]);
            flat(
                parsed,
                Ty::Outcome(Box::new(Ty::Nat(Some(bits))), Box::new(Ty::ParseError)),
                acc,
            )
        }
        _ => Ok((None, acc)),
    }
}

/// The mark of a divergence among a definition's axioms: what follows it
/// says where the definition and the Rust function part, and it leaves the
/// axioms for the register row.
pub(crate) const DIVERGES: &str = "#diverges ";

/// Where a lifted `to_lowercase` parts from Rust's.
pub(crate) const LOWERCASE: &str = "to_lowercase lowers every letter the Unicode case table maps; \
     the lift lowers the ASCII letters only, since no Unicode case table is bound in LexLean or \
     the SDK, so the two differ on text holding a non-ASCII letter that has a lowercase form";

/// The axioms of the runtime's string length and decimal formatting.
const ALL_AXIOMS: [&str; 3] = ["Classical.choice", "Quot.sound", "propext"];

fn method(
    m: &rs::ExprMethodCall,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let name = m.method.to_string();
    if let Some(read) = cell_method(m, env, ctx, &acc) {
        return read(acc);
    }
    let (shaped, acc) = receiver_shape(m, &name, env, ctx, acc, want)?;
    if let Some(lifted) = shaped {
        return Ok((lifted, acc));
    }
    // What the receiver must be for the result to be what is wanted: a
    // `parse` takes its target from the `Option` or `Result` around it.
    let receiver_want = match (name.as_str(), want) {
        ("ok", Some(Ty::Option(t))) | ("map_err", Some(Ty::Outcome(t, _))) => {
            Some(Ty::Outcome(t.clone(), Box::new(Ty::ParseError)))
        }
        ("unwrap_or" | "unwrap_or_default" | "unwrap" | "expect", Some(t)) => {
            Some(Ty::Option(Box::new(t.clone())))
        }
        _ => None,
    };
    let ((r, rt), acc) = flat_of(&m.receiver, env, ctx, acc, receiver_want.as_ref())?;
    method_on(m, &name, r, rt, env, ctx, acc, want)
}

/// A method call on the receiver's lifted value `r` of type `rt`.
#[allow(clippy::too_many_arguments)]
fn method_on(
    m: &rs::ExprMethodCall,
    name: &str,
    r: String,
    rt: Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let mut acc = acc;
    let args: Vec<&rs::RustExpr> = m.args.iter().collect();
    // A value of a type parameter has the methods of its dictionary and
    // no other: Rust finds a method of a bound before any trait's that
    // needs more than the bound gives.
    if let Ty::Param(p) = &rt {
        let Some(entry) = ctx
            .dictionary
            .iter()
            .find(|e| &e.param == p && e.method == name)
        else {
            return Err(Residue::Form(format!(
                "a method {name} of a value of a type parameter, which its dictionary does not \
                 give"
            )));
        };
        return dictionary_call(entry, Some(r), &args, env, ctx, acc);
    }
    // A path's text, and the path itself where Rust borrows or copies it.
    if rt == Ty::Path && args.is_empty() {
        match name {
            "display" => {
                let (text, acc) = acc.prelude("pathDisplay", &[&r]);
                return flat_lift(text, Ty::Str, acc);
            }
            "to_path_buf" | "clone" | "to_owned" => return flat_lift(r, Ty::Path, acc),
            _ => {}
        }
    }
    // `p.join(q)`: the path of `q` where it is absolute, else `q` pushed
    // onto `p` after a separator.
    if let (Ty::Path, "join", [piece]) = (&rt, name, args.as_slice()) {
        let ((v, vt), acc) = flat_of(piece, env, ctx, acc, Some(&Ty::Str))?;
        let (text, acc) = match vt {
            Ty::Str => (v, acc),
            Ty::Path => acc.prelude("pathDisplay", &[&v]),
            _ => {
                return Err(Residue::Form(
                    "a join of something other than a path".into(),
                ))
            }
        };
        let (joined, acc) = acc.prelude("pathJoin", &[&r, &text]);
        return flat_lift(joined, Ty::Path, acc);
    }
    if let Ty::Named(owner) = &rt {
        let key = format!("{owner}::{name}");
        if let Some(sig) = env.fns.get(&key) {
            if !sig.threads.is_empty() {
                return Err(threaded_operand(&key));
            }
            if key == ctx.current {
                // Recursion on a sub-term of `self` descends; any other does not.
                if ctx.tails.get(&r) != ctx.params.first() {
                    return Err(Residue::Form("a method that calls itself".to_string()));
                }
                acc = acc.descended(0);
            }
            if sig.params.len() != args.len() + 1 {
                return Err(Residue::Call(key));
            }
            let mut parts = vec![sig.name.clone(), r];
            for (a, ty) in args.iter().zip(&sig.params[1..]) {
                let ((v, vt), next) = flat_of(a, env, ctx, acc, Some(ty))?;
                acc = next;
                if !vt.same(ty) {
                    return Err(Residue::Form(format!(
                        "argument of another type to .{name}"
                    )));
                }
                parts.push(v);
            }
            let acc = if key == ctx.current {
                acc
            } else {
                acc.called(key)
            };
            return flat_lift(format!("({})", parts.join(" ")), sig.result.clone(), acc);
        }
    }
    if matches!(rt, Ty::Map(..) | Ty::Set(..)) {
        return collection(name, &r, &rt, &args, env, ctx, acc);
    }
    if matches!(rt, Ty::Str | Ty::Char) {
        let (lifted, after) = text_method(m, name, &r, &rt, &args, env, ctx, acc, want)?;
        if let Some(lifted) = lifted {
            return Ok((lifted, after));
        }
        acc = after;
    }
    if let Ty::List(elem) = &rt {
        let (lifted, after) = combinator(m, name, &r, elem, &args, env, ctx, acc)?;
        if let Some(lifted) = lifted {
            return Ok((lifted, after));
        }
        acc = after;
    }
    if rt == Ty::Nat(Some(8)) && args.is_empty() {
        if let Some(test) = ascii_byte(name, &r) {
            return flat_lift(test, Ty::Bool, acc);
        }
    }
    method_typed(name, &r, &rt, &args, env, ctx, acc, want)
}
/// A method call whose receiver is itself a call the lift reads as a
/// whole: `bytes..collect()` into hex, `s.chars().count()`,
/// `s.matches(p).count()` and `(a..b).contains(&x)`. Each shape is tried
/// in turn, and the first that applies gives the value.
fn receiver_shape(
    m: &rs::ExprMethodCall,
    name: &str,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let (lifted, acc) = hex_collect(m, name, env, ctx, acc, want)?;
    if lifted.is_some() {
        return Ok((lifted, acc));
    }
    let (lifted, acc) = chars_count(m, name, env, ctx, acc)?;
    if lifted.is_some() {
        return Ok((lifted, acc));
    }
    let (lifted, acc) = matches_count(m, name, env, ctx, acc)?;
    if lifted.is_some() {
        return Ok((lifted, acc));
    }
    range_contains(m, name, env, ctx, acc)
}

/// `bytes.iter().map(|b| format!("{b:02x}")).collect()` into a `String`:
/// the bytes as lowercase hex.
fn hex_collect(
    m: &rs::ExprMethodCall,
    name: &str,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let mut acc = acc;
    let into_string = want == Some(&Ty::Str)
        || matches!(
            m.turbofish.as_deref(),
            Some([rs::GenericArgument::Type(t)]) if t.text == "String"
        );
    if name == "collect" && m.args.is_empty() && into_string {
        if let Some(bytes) = hex_of(&m.receiver) {
            let byte_list = Ty::List(Box::new(Ty::Nat(Some(8))));
            let ((v, vt), after) = flat_of(bytes, env, ctx, acc, Some(&byte_list))?;
            acc = after;
            if vt == byte_list {
                let (s, acc) = acc.prelude("hexBytes", &[&v]);
                return flat_some(s, Ty::Str, acc);
            }
        }
    }
    Ok((None, acc))
}

/// `s.chars().count()`: the number of characters, which is LexLean's
/// string length.
fn chars_count(
    m: &rs::ExprMethodCall,
    name: &str,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let mut acc = acc;
    if let rs::RustExpr::MethodCall(inner) = &*m.receiver {
        if name == "count" && m.args.is_empty() && inner.method == "chars" && inner.args.is_empty()
        {
            let ((s, st), after) = flat_of(&inner.receiver, env, ctx, acc, Some(&Ty::Str))?;
            acc = after;
            if st == Ty::Str {
                let acc = acc.all_axioms();
                return flat_some(format!("(prim length {s} : Nat)"), Ty::Nat(None), acc);
            }
        }
    }
    Ok((None, acc))
}

/// `s.matches(p).count()`: the pieces `s.split(p)` cuts, less one.
fn matches_count(
    m: &rs::ExprMethodCall,
    name: &str,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    if let rs::RustExpr::MethodCall(inner) = &*m.receiver {
        if let ("count", true, "matches", [p]) = (
            name,
            m.args.is_empty(),
            inner.method.as_str(),
            inner.args.as_slice(),
        ) {
            let ((s, st), after) = flat_of(&inner.receiver, env, ctx, acc, Some(&Ty::Str))?;
            let ((p, pt), after) = flat_of(p, env, ctx, after, Some(&Ty::Str))?;
            if st != Ty::Str || !matches!(pt, Ty::Str | Ty::Char) {
                return Err(Residue::Form("a .matches pattern that is not text".into()));
            }
            let (pieces, after) = after.prelude("textSplit", &[&s, &p]);
            return flat_some(
                format!("(prim subtract (prim length {pieces} : Nat) 1 : Nat)"),
                Ty::Nat(None),
                after,
            );
        }
    }
    Ok((None, acc))
}

/// `(a..b).contains(&x)`: `a <= x` and `x` below the end, or at it for
/// `a..=b`.
fn range_contains(
    m: &rs::ExprMethodCall,
    name: &str,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    if let (Some((lo, hi, closed)), "contains", [x]) =
        (range_bounds(&m.receiver), name, m.args.as_slice())
    {
        let ((l, lt), acc) = flat_of(lo, env, ctx, acc, Some(&Ty::Nat(None)))?;
        let ((h, ht), acc) = flat_of(hi, env, ctx, acc, Some(&lt))?;
        let x = match x {
            rs::RustExpr::Reference(r) => &*r.expr,
            other => other,
        };
        let ((v, vt), acc) = flat_of(x, env, ctx, acc, Some(&lt))?;
        if !matches!(lt, Ty::Nat(_)) || !ht.same(&lt) || !vt.same(&lt) {
            return Err(Residue::Form(
                "a range of something other than naturals".into(),
            ));
        }
        let below = if closed { "<=?" } else { "<?" };
        return flat_some(
            format!("(({l} <=? {v}) && ({v} {below} {h}))"),
            Ty::Bool,
            acc,
        );
    }
    Ok((None, acc))
}

/// A method of a receiver of a type the lift knows, by its type and name.
#[allow(clippy::too_many_arguments)]
fn method_typed(
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    // A trait of the workspace that declares a method of the name may be
    // what the call reaches (see `inherent_reached`).
    if ctx.seen.trait_methods.contains(name) || ctx.seen.macro_items {
        return Err(Residue::Form(format!(
            "method call .{name}, which a trait of the workspace in scope may declare"
        )));
    }
    match (rt, name) {
        // `a.checked_div(b)`, `a.checked_rem(b)`: none for a zero divisor.
        (Ty::Nat(_), "checked_div" | "checked_rem") if args.len() == 1 => {
            method_nat_checked_div(name, r, rt, args, env, ctx, acc)
        }
        (Ty::Nat(w), "checked_add" | "checked_mul") => {
            method_nat_checked_add(w, name, r, rt, args, env, ctx, acc)
        }
        (Ty::Nat(_), "checked_sub") => {
            method_nat_checked_sub(name, r, rt, args, env, ctx, acc)
        }
        (Ty::Nat(_), "checked_div" | "checked_rem") => {
            method_nat_checked_div_args(name, r, rt, args, env, ctx, acc)
        }
        (Ty::Nat(w), "wrapping_add" | "wrapping_mul" | "wrapping_sub") => {
            method_nat_wrapping_add(w, name, r, rt, args, env, ctx, acc)
        }
        (Ty::Nat(w), "saturating_add" | "saturating_mul") => {
            method_nat_saturating_add(w, name, r, rt, args, env, ctx, acc)
        }
        (Ty::Nat(_), "saturating_sub") => {
            method_nat_saturating_sub(name, r, rt, args, env, ctx, acc)
        }
        (Ty::Nat(_), "abs_diff") => {
            method_nat_abs_diff(name, r, rt, args, env, ctx, acc)
        }
        (Ty::Nat(_), "midpoint") => {
            method_nat_midpoint(name, r, rt, args, env, ctx, acc)
        }
        (Ty::Nat(_), "min" | "max") => {
            method_nat_min(name, r, rt, args, env, ctx, acc)
        }
        (Ty::Int(bits), "checked_add" | "checked_sub" | "checked_mul" | "checked_div") => {
            method_int_checked_add(bits, name, r, rt, args, env, ctx, acc)
        }
        (Ty::Int(bits), "checked_neg") if args.is_empty() => {
            method_int_checked_neg(bits, name, r, rt, acc)
        }
        (Ty::Int(_), "min" | "max") => {
            method_int_min(name, r, rt, args, env, ctx, acc)
        }
        (
            Ty::Integer(bits),
            "checked_add" | "checked_sub" | "checked_mul" | "checked_div" | "checked_neg",
        ) => method_integer_checked(bits, name, r, rt, args, env, ctx, acc),
        // A float's inherent methods, where no trait in scope may take the
        // name first: the prelude's model, by the table the MIR path reads.
        (Ty::Float(f), _) if float_method(name, args.len()).is_some() => {
            inherent_reached(name, ctx)?;
            method_float(*f, name, r, rt, args, env, ctx, acc)
        }
        (Ty::Ratio, "numer" | "denom" | "is_integer")
            if args.is_empty() && ctx.seen.outside_traits =>
        {
            Err(trait_in_scope(name))
        }
        // A ratio's inherent accessors (num-rational 0.4.2, `src/lib.rs`),
        // where no trait in scope may take the name first: the stored
        // numerator and denominator, and whether the denominator is one.
        // The reciprocal panics on a zero numerator, which nothing the lift
        // reads rules out: it stays outside.
        (Ty::Ratio, "recip") if args.is_empty() => Err(Residue::Form(
            "recip of a ratio whose numerator may be zero, where Rust panics".to_string(),
        )),
        (Ty::Ratio, "numer" | "denom" | "is_integer") if args.is_empty() => {
            let Some((term, result))=crate::registered::numeric_observation(rt,name,r) else {
                return Err(Residue::Form(format!("unmodeled ratio observation {name}")));
            };
            let acc=if name=="is_integer" {acc.axiom("propext")} else {acc};
            flat_lift(term,result,acc)
        }
        (Ty::List(_), "len") if args.is_empty() => {
            method_list_len(r, acc)
        }
        (Ty::List(_), "is_empty") if args.is_empty() => {
            method_list_is_empty(r, acc)
        }
        (Ty::List(t), "first") if args.is_empty() => flat_lift(
            format!("(prim index {r} 0 : (Option {}))", t.render()),
            opt_of(t),
            acc,
        ),
        (Ty::List(t), "get") => {
            method_list_get(t, name, r, args, env, ctx, acc)
        }
        (Ty::Str, "is_empty") if args.is_empty() => {
            method_str_is_empty(r, acc)
        }
        (Ty::Str, "to_string" | "to_owned")
        | (Ty::List(_), "as_slice")
        | (Ty::Option(_), "as_deref")
        // An option of a borrow, or a borrowed slice: the same value, since
        // LexLean holds every value by reference.
        | (Ty::Option(_) | Ty::List(_), "as_ref")
        | (_, "clone" | "copied" | "cloned" | "to_vec")
            if args.is_empty() =>
        {
            method_str_to_string(r, rt, acc)
        }
        // `b.then_some(v)`: `Some(v)` where `b` holds.
        // `c.then(|| v)`: the value where the test holds. A lifted value
        // is pure and total, so building it eagerly changes nothing.
        (Ty::Bool, "then") if matches!(args, [rs::RustExpr::Closure(c)] if c.inputs.is_empty()) =>
        {
            method_bool_then(r, args, env, ctx, acc, want)
        }
        (Ty::Bool, "then_some") if args.len() == 1 => {
            method_bool_then_some(r, args, env, ctx, acc)
        }
        (Ty::Option(_), "is_some" | "is_none") if args.is_empty() => {
            method_option_is_some(name, r, env, ctx, acc)
        }
        (Ty::Option(t), "unwrap_or") => {
            method_option_unwrap_or(t, name, r, args, env, ctx, acc)
        }
        (Ty::Outcome(_, _), "is_ok" | "is_err") if args.is_empty() => {
            method_outcome_is_ok(name, r, env, ctx, acc)
        }
        (Ty::Outcome(t, _), "ok") if args.is_empty() => {
            method_outcome_ok(t, r, env, ctx, acc)
        }
        (Ty::Option(_) | Ty::Outcome(..), _) => {
            method_option_any(name, r, rt, args, env, ctx, acc)
        }
        (Ty::Char, _) if unicode_property(name).is_some() => {
            method_char_any(name)
        }
        _ => Err(Residue::Form(format!("method call .{name}"))),
    }
}

/// `.checked_div` on a receiver of type `Nat`.
fn method_nat_checked_div(
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let ((d, dt), acc) = flat_of(args[0], env, ctx, acc, Some(rt))?;
    if !dt.same(rt) {
        return Err(Residue::Form(format!(
            "argument of another type to .{name}"
        )));
    }
    let prim = if name == "checked_div" {
        "quotient"
    } else {
        "remainder"
    };
    flat_lift(
        format!(
            "(if ({d} =? 0) then {} else {})",
            none_of(rt),
            some_of(rt, &format!("(prim {prim} {r} {d} 0 : Nat)"))
        ),
        opt_of(rt),
        acc,
    )
}

/// `.checked_add` on a receiver of type `Nat`.
#[allow(clippy::too_many_arguments)]
fn method_nat_checked_add(
    w: &Option<u32>,
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let Some(bits) = w else {
        return Err(Residue::Width(format!(".{name} on usize")));
    };
    let (a, acc) = one_arg(args, name, env, ctx, rt, acc)?;
    let v = if name == "checked_add" {
        format!("({r} + {a})")
    } else {
        format!("(prim multiply {r} {a} : Nat)")
    };
    flat_lift(
        format!(
            "(if ({v} <? {}) then {} else {})",
            pow2(*bits),
            some_of(rt, &v),
            none_of(rt)
        ),
        opt_of(rt),
        acc,
    )
}

/// `.checked_sub` on a receiver of type `Nat`.
fn method_nat_checked_sub(
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let (a, acc) = one_arg(args, name, env, ctx, rt, acc)?;
    flat_lift(
        format!(
            "(if ({a} <=? {r}) then {} else {})",
            some_of(rt, &format!("(prim subtract {r} {a} : Nat)")),
            none_of(rt)
        ),
        opt_of(rt),
        acc,
    )
}

/// `.checked_div` on a receiver of type `Nat`.
fn method_nat_checked_div_args(
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let (a, acc) = one_arg(args, name, env, ctx, rt, acc)?;
    let prim = if name == "checked_div" {
        "quotient"
    } else {
        "remainder"
    };
    flat_lift(
        format!(
            "(if ({a} =? 0) then {} else {})",
            none_of(rt),
            some_of(rt, &format!("(prim {prim} {r} {a} 0 : Nat)"))
        ),
        opt_of(rt),
        acc,
    )
}

/// `.wrapping_add` on a receiver of type `Nat`.
#[allow(clippy::too_many_arguments)]
fn method_nat_wrapping_add(
    w: &Option<u32>,
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let Some(bits) = w else {
        return Err(Residue::Width(format!(".{name} on usize")));
    };
    let (a, acc) = one_arg(args, name, env, ctx, rt, acc)?;
    let m = pow2(*bits);
    let v = match name {
        "wrapping_add" => format!("({r} + {a})"),
        "wrapping_mul" => format!("(prim multiply {r} {a} : Nat)"),
        _ => format!("(prim subtract ({r} + {m}) {a} : Nat)"),
    };
    flat_lift(format!("(prim remainder {v} {m} 0 : Nat)"), rt.clone(), acc)
}

/// `.saturating_add` on a receiver of type `Nat`.
#[allow(clippy::too_many_arguments)]
fn method_nat_saturating_add(
    w: &Option<u32>,
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let Some(bits) = w else {
        return Err(Residue::Width(format!(".{name} on usize")));
    };
    let (a, acc) = one_arg(args, name, env, ctx, rt, acc)?;
    let v = if name == "saturating_add" {
        format!("({r} + {a})")
    } else {
        format!("(prim multiply {r} {a} : Nat)")
    };
    flat_lift(
        format!(
            "(if ({v} <? {}) then {v} else {})",
            pow2(*bits),
            max_of(*bits)
        ),
        rt.clone(),
        acc,
    )
}

/// `.saturating_sub` on a receiver of type `Nat`.
fn method_nat_saturating_sub(
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let (a, acc) = one_arg(args, name, env, ctx, rt, acc)?;
    flat_lift(format!("(prim subtract {r} {a} : Nat)"), rt.clone(), acc)
}

/// `.abs_diff` on a receiver of type `Nat`.
fn method_nat_abs_diff(
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let (a, acc) = one_arg(args, name, env, ctx, rt, acc)?;
    flat_lift(
        format!("(if ({r} <=? {a}) then (prim subtract {a} {r} : Nat) else (prim subtract {r} {a} : Nat))"),
        rt.clone(), acc)
}

/// `.midpoint` on a receiver of type `Nat`.
fn method_nat_midpoint(
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let (a, acc) = one_arg(args, name, env, ctx, rt, acc)?;
    flat_lift(
        format!("(prim quotient ({r} + {a}) 2 0 : Nat)"),
        rt.clone(),
        acc,
    )
}

/// `.min` on a receiver of type `Nat`.
fn method_nat_min(
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let (a, acc) = one_arg(args, name, env, ctx, rt, acc)?;
    let (x, y) = if name == "min" {
        (r, a.as_str())
    } else {
        (a.as_str(), r)
    };
    flat_lift(
        format!("(if ({r} <=? {a}) then {x} else {y})"),
        rt.clone(),
        acc,
    )
}

/// `.checked_add` on a receiver of type `Int`.
#[allow(clippy::too_many_arguments)]
fn method_int_checked_add(
    bits: &u32,
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    if *bits != 64 {
        return Err(Residue::Width(format!(".{name} on i{bits}")));
    }
    let (a, acc) = one_arg(args, name, env, ctx, rt, acc)?;
    let prim = match name {
        "checked_add" => "checked_add",
        "checked_sub" => "checked_subtract",
        "checked_mul" => "checked_multiply",
        _ => "checked_quotient",
    };
    flat_lift(
        format!("(prim {prim} {r} {a} : (Option Int64))"),
        opt_of(rt),
        acc,
    )
}

/// `.checked_neg` on a receiver of type `Int`.
fn method_int_checked_neg(
    bits: &u32,
    name: &str,
    r: &str,
    rt: &Ty,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    if *bits != 64 {
        return Err(Residue::Width(format!(".{name} on i{bits}")));
    }
    flat_lift(
        format!("(prim checked_negate {r} : (Option Int64))"),
        opt_of(rt),
        acc,
    )
}

/// A checked operation on a receiver of type `Integer`: the mathematical
/// result where the type's width holds it. `i128` holds `-2^127` to
/// `2^127 - 1`; what `isize` holds is its target's, so the operation is
/// residue there, as it is on `usize`.
#[allow(clippy::too_many_arguments)]
fn method_integer_checked(
    bits: &Option<u32>,
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let Some(bits) = bits else {
        return Err(Residue::Width(format!(".{name} on isize")));
    };
    let half = 1u128 << (bits - 1);
    let lowest = format!("(-{half} : Int)");
    let highest = format!("({} : Int)", half - 1);
    let below = |x: &str, y: &str| format!("(prim less_than {x} {y} : Bool)");
    let none = "(ctor Option.none [Int])";
    let acc = acc.axiom("propext");
    let (value, zero, acc) = if name == "checked_neg" {
        if !args.is_empty() {
            return Err(Residue::Form(format!("method call .{name}")));
        }
        (format!("(prim negate {r} : Int)"), None, acc)
    } else {
        let (a, acc) = one_arg(args, name, env, ctx, rt, acc)?;
        match name {
            // LexLean adds no two of `Int`: a sum is the difference from
            // the negation.
            "checked_add" => (
                format!("(prim subtract {r} (prim negate {a} : Int) : Int)"),
                None,
                acc,
            ),
            "checked_sub" => (format!("(prim subtract {r} {a} : Int)"), None, acc),
            "checked_mul" => (format!("(prim multiply {r} {a} : Int)"), None, acc),
            // `Int`'s quotient truncates toward zero, as Rust's does; a
            // zero divisor has no quotient.
            _ => {
                let zero = "(0 : Int)";
                let is_zero = format!("(not ({} || {}))", below(&a, zero), below(zero, &a));
                (
                    format!("(prim quotient {r} {a} {zero} : Int)"),
                    Some(is_zero),
                    acc,
                )
            }
        }
    };
    let (held, acc) = ctx.fresh(env, acc, "wide");
    let out_of_range = format!("({} || {})", below(&held, &lowest), below(&highest, &held));
    let checked = format!(
        "(let {held} : Int := {value}; (if {out_of_range} then {none} else (ctor Option.some [Int] {held})))"
    );
    let text = match zero {
        Some(is_zero) => format!("(if {is_zero} then {none} else {checked})"),
        None => checked,
    };
    flat_lift(text, opt_of(rt), acc)
}

/// The residue of a method call a trait in scope may take before the
/// inherent method of the name.
fn trait_in_scope(name: &str) -> Residue {
    Residue::Form(format!(
        "method call .{name}, which a trait in scope from outside the workspace may declare"
    ))
}

/// Whether the inherent method `name` of a float or a ratio is what a call
/// reaches. Method lookup tries each receiver type in turn (the value's,
/// then `&` and `&mut` of it, then the same after each dereference) and
/// prefers an inherent method only within one of those steps (the Rust
/// reference, "Method call expressions"), so a trait method whose receiver
/// matches an earlier step wins: `Top::numer(self)` on a ratio value over
/// `Ratio::numer(&self)`, `Odd::abs(&self)` on an `&f64` over
/// `f64::abs(self)`. A trait's method is a candidate only where the trait
/// is in scope, so the inherent method is the one reached where no trait of
/// the workspace in scope declares the name (checked in [`method_typed`],
/// [`Seen::trait_methods`]) and the
/// scope brings in no trait from outside the workspace and the standard
/// library, whose prelude declares none of these names for a float or a
/// ratio.
fn inherent_reached(name: &str, ctx: &Ctx) -> Lift<()> {
    match ctx.seen.outside_traits {
        true => Err(trait_in_scope(name)),
        false => Ok(()),
    }
}

/// The prelude's operation and kind of a float's inherent method of this
/// many arguments besides `self`, by the table the MIR path reads
/// ([`crate::mir::float::METHODS`]).
fn float_method(name: &str, args: usize) -> Option<(&'static str, crate::mir::float::Kind)> {
    use crate::mir::float::Kind;
    crate::mir::float::METHODS
        .iter()
        .find(|(t, m, _, n, kind)| {
            t.is_empty()
                && *m == name
                && *n == args + 1
                && (matches!(kind, Kind::Value | Kind::Test) || *m == "to_bits")
        })
        .map(|(_, _, op, _, kind)| (*op, *kind))
}

/// The float methods whose value depends on the sign or payload of a NaN.
const NAN_OBSERVERS: [&str; 4] = [
    "to_bits",
    "is_sign_negative",
    "is_sign_positive",
    "copysign",
];

/// `x.abs()`, `x.is_nan()`, `x.to_bits()` and the other inherent methods
/// of a float the prelude models, each on operands of the receiver's
/// format. `to_bits` is the bit pattern a float is held as.
#[allow(clippy::too_many_arguments)]
fn method_float(
    f: crate::mir::float::Format,
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    use crate::mir::float::Kind;
    let Some((op, kind)) = float_method(name, args.len()) else {
        return Err(Residue::Form(format!("method call .{name}")));
    };
    // Rust leaves the sign and payload of a NaN an operation returns
    // unspecified (the standard library's `f64` documentation, "NaN bit
    // patterns"), where the prelude's model fixes one: what reads them of
    // a value that may be such a NaN is not the model's.
    if NAN_OBSERVERS.contains(&name) {
        return Err(Residue::Form(format!(
            ".{name} of a float, which reads the sign or payload of a NaN Rust leaves unspecified"
        )));
    }
    // `min` and `max` of two zeros may return either (the standard
    // library's `f64::min`: "either may be returned"), so the operands
    // must not both be zeros: one is a literal that is not zero.
    if matches!(name, "min" | "max" | "minimum" | "maximum")
        && !args.iter().any(|a| nonzero_float_literal(a))
    {
        return Err(Residue::Form(format!(
            ".{name} of floats that may both be zeros, whose sign Rust leaves unspecified"
        )));
    }
    if name == "to_bits" {
        return flat_lift(r.to_string(), Ty::Nat(Some(1 + f.exp + f.frac)), acc);
    }
    let mut terms = vec![r.to_string()];
    let mut acc = acc;
    for a in args {
        let ((v, vt), next) = flat_of(a, env, ctx, acc, Some(rt))?;
        if vt != *rt {
            return Err(Residue::Form(format!(
                "method call .{name} of another type"
            )));
        }
        terms.push(v);
        acc = next;
    }
    let refs: Vec<&str> = terms.iter().map(String::as_str).collect();
    let (term, acc) = float_term(f, op, &refs, acc);
    let ty = match kind {
        Kind::Test => Ty::Bool,
        _ => rt.clone(),
    };
    flat_lift(term, ty, acc)
}

/// `.min` on a receiver of type `Int`.
fn method_int_min(
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let acc = acc.axiom("propext");
    let (a, acc) = one_arg(args, name, env, ctx, rt, acc)?;
    let (x, y) = if name == "min" {
        (r, a.as_str())
    } else {
        (a.as_str(), r)
    };
    flat_lift(
        format!("(if (prim less_than {a} {r} : Bool) then {y} else {x})"),
        rt.clone(),
        acc,
    )
}

/// `.len` on a receiver of type `List`.
fn method_list_len(r: &str, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    flat_lift(format!("(prim length {r} : Nat)"), Ty::Nat(None), acc)
}

/// `.is_empty` on a receiver of type `List`.
fn method_list_is_empty(r: &str, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    flat_lift(format!("((prim length {r} : Nat) =? 0)"), Ty::Bool, acc)
}

/// `.get` on a receiver of type `List`.
fn method_list_get(
    t: &Ty,
    name: &str,
    r: &str,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let (i, acc) = one_arg(args, name, env, ctx, &Ty::Nat(None), acc)?;
    flat_lift(
        format!("(prim index {r} {i} : (Option {}))", t.render()),
        opt_of(t),
        acc,
    )
}

/// `.is_empty` on a receiver of type `Str`.
fn method_str_is_empty(r: &str, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    flat_lift(format!("(prim equal {r} \"\" : Bool)"), Ty::Bool, acc)
}

/// `.to_string` on a receiver of type `Str`.
fn method_str_to_string(r: &str, rt: &Ty, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    flat_lift(r.to_string(), rt.clone(), acc)
}

/// `.then` on a receiver of type `Bool`.
fn method_bool_then(
    r: &str,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let [rs::RustExpr::Closure(c)] = args else {
        return Err(Residue::Form(".then of another argument".into()));
    };
    let inner = match want {
        Some(Ty::Option(t)) => Some(&**t),
        _ => None,
    };
    let ((v, vt), acc) = flat_of(&c.body, env, &ctx.below(), acc, inner)?;
    flat_lift(
        format!("(if {r} then {} else {})", some_of(&vt, &v), none_of(&vt)),
        opt_of(&vt),
        acc,
    )
}

/// `.then_some` on a receiver of type `Bool`.
fn method_bool_then_some(
    r: &str,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let ((v, vt), acc) = flat_of(args[0], env, ctx, acc, None)?;
    flat_lift(
        format!("(if {r} then {} else {})", some_of(&vt, &v), none_of(&vt)),
        opt_of(&vt),
        acc,
    )
}

/// `.is_some` on a receiver of type `Option`.
fn method_option_is_some(
    name: &str,
    r: &str,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let (v, acc) = ctx.fresh(env, acc, "value");
    let (n, s) = if name == "is_some" {
        ("false", "true")
    } else {
        ("true", "false")
    };
    Ok((
        (
            Expr::Match {
                scrutinee: r.to_string(),
                arms: vec![
                    Arm {
                        pattern: "Option.none".into(),
                        body: Expr::Flat(n.into()),
                    },
                    Arm {
                        pattern: format!("Option.some {v}"),
                        body: Expr::Flat(s.into()),
                    },
                ],
            },
            Ty::Bool,
        ),
        acc,
    ))
}

/// `.unwrap_or` on a receiver of type `Option`.
fn method_option_unwrap_or(
    t: &Ty,
    name: &str,
    r: &str,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let (d, acc) = one_arg(args, name, env, ctx, t, acc)?;
    let (v, acc) = ctx.fresh(env, acc, "value");
    Ok((
        (
            Expr::Match {
                scrutinee: r.to_string(),
                arms: vec![
                    Arm {
                        pattern: "Option.none".into(),
                        body: Expr::Flat(d),
                    },
                    Arm {
                        pattern: format!("Option.some {v}"),
                        body: Expr::Flat(v.clone()),
                    },
                ],
            },
            t.clone(),
        ),
        acc,
    ))
}

/// `.is_ok` on a receiver of type `Outcome`.
fn method_outcome_is_ok(
    name: &str,
    r: &str,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let (v, acc) = ctx.fresh(env, acc, "value");
    let (e, acc) = ctx.fresh(env, acc, "error");
    let (o, x) = if name == "is_ok" {
        ("true", "false")
    } else {
        ("false", "true")
    };
    Ok((
        (
            Expr::Match {
                scrutinee: r.to_string(),
                arms: vec![
                    Arm {
                        pattern: format!("Result.ok {v}"),
                        body: Expr::Flat(o.into()),
                    },
                    Arm {
                        pattern: format!("Result.error {e}"),
                        body: Expr::Flat(x.into()),
                    },
                ],
            },
            Ty::Bool,
        ),
        acc,
    ))
}

/// `.ok` on a receiver of type `Outcome`.
fn method_outcome_ok(t: &Ty, r: &str, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<((Expr, Ty), Acc)> {
    let (v, acc) = ctx.fresh(env, acc, "value");
    let (e, acc) = ctx.fresh(env, acc, "error");
    Ok((
        (
            Expr::Match {
                scrutinee: r.to_string(),
                arms: vec![
                    Arm {
                        pattern: format!("Result.ok {v}"),
                        body: Expr::Flat(some_of(t, &v)),
                    },
                    Arm {
                        pattern: format!("Result.error {e}"),
                        body: Expr::Flat(none_of(t)),
                    },
                ],
            },
            opt_of(t),
        ),
        acc,
    ))
}

/// `._` on a receiver of type `Option`.
fn method_option_any(
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    wrapped_method(name, r, rt, args, env, ctx, acc)
}

/// `._` on a receiver of type `Char`.
fn method_char_any(name: &str) -> Lift<((Expr, Ty), Acc)> {
    Err(Residue::Form(unicode_property(name).unwrap_or_default()))
}

/// `Some(v)` of type `t`.
fn some_of(t: &Ty, v: &str) -> String {
    format!("(ctor Option.some [{}] {v})", t.render())
}

/// `None` of type `t`.
fn none_of(t: &Ty) -> String {
    format!("(ctor Option.none [{}])", t.render())
}

/// `Option<t>`.
fn opt_of(t: &Ty) -> Ty {
    Ty::Option(Box::new(t.clone()))
}

/// The one argument of a method call, of the type it must have.
fn one_arg(
    args: &[&rs::RustExpr],
    name: &str,
    env: &Env,
    ctx: &Ctx,
    want: &Ty,
    acc: Acc,
) -> Lift<(String, Acc)> {
    let [a] = args else {
        return Err(Residue::Form(format!("method call .{name}")));
    };
    let ((v, vt), acc) = flat_of(a, env, ctx, acc, Some(want))?;
    if vt.same(want) {
        Ok((v, acc))
    } else {
        Err(Residue::Form(format!(
            "argument of another type to .{name}"
        )))
    }
}

/// The bounds of `a..b` or `a..=b`, in parentheses or not, and whether the
/// range holds its end.
fn range_bounds(e: &rs::RustExpr) -> Option<(&rs::RustExpr, &rs::RustExpr, bool)> {
    let bare = match e {
        rs::RustExpr::Paren(inner) => &**inner,
        other => other,
    };
    match bare {
        rs::RustExpr::Other(o) => match (o.kind.as_str(), o.exprs.as_slice()) {
            ("expr::range", [lo, hi]) => Some((lo, hi, false)),
            ("expr::range_inclusive", [lo, hi]) => Some((lo, hi, true)),
            _ => None,
        },
        _ => None,
    }
}

/// The gap a Unicode property of a `char` names: the property tests the
/// Unicode Character Database, and no table of it is bound in LexLean or
/// the SDK. The ASCII forms (`is_ascii_alphabetic`, ..) lift.
fn unicode_property(name: &str) -> Option<String> {
    #[cfg(not(uorust_lift))]
    crate::certified::certified!(lift_unicode_property, name);
    let property = match name {
        "is_alphabetic" => "Alphabetic",
        "is_alphanumeric" => "Alphabetic or Numeric",
        "is_numeric" => "Numeric",
        "is_uppercase" => "Uppercase",
        "is_lowercase" => "Lowercase",
        _ => return None,
    };
    Some(format!(
        "char::{name} tests the Unicode {property} property, and no Unicode table is bound \
         in LexLean or the SDK"
    ))
}

/// A value of a type, where the lift needs one that nothing observes: what
/// a Rust `unwrap` or index panics to. Lean's own `get!` returns the same
/// `default`, so the definition agrees with Rust wherever Rust returns.
pub(crate) fn default_of(ty: &Ty, types: &Types) -> Option<String> {
    default_in(ty, types, 0)
}

fn default_in(ty: &Ty, types: &Types, depth: usize) -> Option<String> {
    if depth > 16 {
        return None;
    }
    Some(match ty {
        Ty::Unit => "()".into(),
        Ty::Bool => "false".into(),
        Ty::Nat(_) | Ty::Float(_) | Ty::Signed(_) | Ty::Scalar => "0".into(),
        Ty::Int(_) => "(0 : Int64)".into(),
        Ty::Integer(_) | Ty::Unbounded => "(0 : Int)".into(),
        Ty::Ratio => "((0 : Int), (1 : Int))".into(),
        Ty::Str | Ty::Char => "\"\"".into(),
        Ty::ParseError => format!("(ctor {PRELUDE}::ParseIntError.InvalidDigit)"),
        Ty::Option(t) => format!("(ctor Option.none [{}])", t.render()),
        // `Ok` of the success type's default, or `Err` of the error's.
        Ty::Outcome(ok, err) => match default_in(ok, types, depth + 1) {
            Some(v) => format!("(ctor Result.ok [{}, {}] {v})", ok.render(), err.render()),
            None => format!(
                "(ctor Result.error [{}, {}] {})",
                ok.render(),
                err.render(),
                default_in(err, types, depth + 1)?
            ),
        },
        // A type parameter has no default its definition could name.
        Ty::Path | Ty::Param(_) => return None,
        // A function: the one that gives every input its result's default.
        Ty::Fn(ins, out) => {
            let params: Vec<String> = ins
                .iter()
                .enumerate()
                .map(|(i, t)| format!("(input{i} : {})", t.render()))
                .collect();
            format!(
                "(fun {} => {})",
                params.join(" "),
                default_in(out, types, depth + 1)?
            )
        }
        Ty::List(t) | Ty::Set(t, _) => format!("(nil {})", t.render()),
        Ty::Map(..) => format!("(nil {})", ty.entry().render()),
        Ty::Tuple(ts) | Ty::Pair(ts) => {
            let mut parts = Vec::new();
            for t in ts {
                parts.push(default_in(t, types, depth + 1)?);
            }
            tuple_value(ty, &parts)
        }
        Ty::Named(n) => {
            if let Some(vs) = types.enums.get(n) {
                format!("(ctor {n}.{})", vs.first()?)
            } else if let Some(vs) = types.datas.get(n) {
                // The first variant whose fields all have a value.
                vs.iter().find_map(|(v, fields)| {
                    let mut parts = vec![format!("ctor {n}.{v}")];
                    for (_, t) in fields {
                        parts.push(default_in(t, types, depth + 1)?);
                    }
                    Some(format!("({})", parts.join(" ")))
                })?
            } else {
                let fields = types.structs.get(n)?;
                let mut parts = Vec::new();
                for (f, t) in fields {
                    parts.push((f, default_in(t, types, depth + 1)?));
                }
                struct_value(n, &parts, types)
            }
        }
    })
}

/// The combinators of `Option` and `Result`, each a match on the receiver:
/// `map`, `and_then`, `map_err`, `filter`, `is_some_and`, `is_none_or`,
/// `is_ok_and`, `ok_or`, `err`, `unwrap_or`, `unwrap_or_else`,
/// `unwrap_or_default`, `unwrap` and `expect`.
#[allow(clippy::too_many_lines)]
fn wrapped_method(
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let bad = || Err(Residue::Form(format!("method call .{name}")));
    let some = |t: &Ty, v: &str| format!("(ctor Option.some [{}] {v})", t.render());
    let none = |t: &Ty| format!("(ctor Option.none [{}])", t.render());
    let opt = |t: &Ty| Ty::Option(Box::new(t.clone()));
    // The receiver's value, its failure, and the constructor patterns.
    let (value, failure) = match rt {
        Ty::Option(t) => ((**t).clone(), None),
        Ty::Outcome(t, e) => ((**t).clone(), Some((**e).clone())),
        _ => return bad(),
    };
    let ok_pattern = |v: &str| {
        if failure.is_some() {
            format!("Result.ok {v}")
        } else {
            format!("Option.some {v}")
        }
    };
    let fail_pattern = |e: &str| {
        if failure.is_some() {
            format!("Result.error {e}")
        } else {
            "Option.none".to_string()
        }
    };
    let result_of = |t: &Ty, e: &Ty| format!("[{}, {}]", t.render(), e.render());
    let two = |ok_v: &str, ok: Expr, fail_e: &str, fail: Expr| Expr::Match {
        scrutinee: r.to_string(),
        arms: if failure.is_some() {
            vec![
                Arm {
                    pattern: ok_pattern(ok_v),
                    body: ok,
                },
                Arm {
                    pattern: fail_pattern(fail_e),
                    body: fail,
                },
            ]
        } else {
            vec![
                Arm {
                    pattern: fail_pattern(fail_e),
                    body: fail,
                },
                Arm {
                    pattern: ok_pattern(ok_v),
                    body: ok,
                },
            ]
        },
    };
    let closure =
        |f: &rs::RustExpr, params: &[Ty], acc: Acc, want: Option<&Ty>| -> Lift<(Lambda, Acc)> {
            let (lam, acc) = lambda(f, params, env, ctx, acc, want)?;
            let acc = acc.called_all(lam.calls.clone());
            let acc = acc.axioms_all(lam.axioms.clone());
            Ok((lam, acc))
        };
    let (fresh_e, acc) = ctx.fresh(env, acc, "error");
    match (name, args) {
        ("map", [f]) => {
            let (lam, acc) = closure(f, std::slice::from_ref(&value), acc, None)?;
            let v = lam.binders[0].clone();
            let body = lam.wrap(lam.core.clone());
            match &failure {
                None => Ok((
                    (
                        two(&v, wrap_some(&lam.ty, body), "", Expr::Flat(none(&lam.ty))),
                        opt(&lam.ty),
                    ),
                    acc,
                )),
                Some(e) => Ok((
                    (
                        two(
                            &v,
                            wrap_ctor(&format!("Result.ok {}", result_of(&lam.ty, e)), body),
                            &fresh_e,
                            Expr::Flat(format!(
                                "(ctor Result.error {} {fresh_e})",
                                result_of(&lam.ty, e)
                            )),
                        ),
                        Ty::Outcome(Box::new(lam.ty.clone()), Box::new(e.clone())),
                    ),
                    acc,
                )),
            }
        }
        ("map_or", [d, f]) => {
            let (lam, acc) = closure(f, std::slice::from_ref(&value), acc, None)?;
            let ((dv, dt), acc) = flat_of(d, env, ctx, acc, Some(&lam.ty))?;
            if !dt.same(&lam.ty) {
                return Err(Residue::Form("map_or of another type".into()));
            }
            let v = lam.binders[0].clone();
            let body = lam.wrap(lam.core.clone());
            Ok((
                (two(&v, body, &fresh_e, Expr::Flat(dv)), lam.ty.clone()),
                acc,
            ))
        }
        ("map_err", [f]) => {
            let Some(e) = &failure else { return bad() };
            let (lam, acc) = closure(f, std::slice::from_ref(e), acc, None)?;
            let (v, acc) = ctx.fresh(env, acc, "value");
            let x = lam.binders[0].clone();
            let body = lam.wrap(lam.core.clone());
            let types = result_of(&value, &lam.ty);
            Ok((
                (
                    two(
                        &v,
                        Expr::Flat(format!("(ctor Result.ok {types} {v})")),
                        &x,
                        wrap_ctor(&format!("Result.error {types}"), body),
                    ),
                    Ty::Outcome(Box::new(value.clone()), Box::new(lam.ty.clone())),
                ),
                acc,
            ))
        }
        ("and_then", [f]) => {
            let (lam, acc) = closure(f, std::slice::from_ref(&value), acc, None)?;
            let v = lam.binders[0].clone();
            let body = lam.wrap(lam.core.clone());
            let fail = match (&failure, &lam.ty) {
                (None, Ty::Option(u)) => none(u),
                (Some(e), Ty::Outcome(u, e2)) if e.same(e2) => {
                    format!("(ctor Result.error {} {fresh_e})", result_of(u, e))
                }
                _ => return Err(Residue::Form("and_then of another type".into())),
            };
            Ok((
                (two(&v, body, &fresh_e, Expr::Flat(fail)), lam.ty.clone()),
                acc,
            ))
        }
        ("filter", [f]) if failure.is_none() => {
            let (lam, acc) = closure(f, std::slice::from_ref(&value), acc, Some(&Ty::Bool))?;
            if lam.ty != Ty::Bool {
                return bad();
            }
            let v = lam.binders[0].clone();
            let test = lam.wrap(format!(
                "(if {} then {} else {})",
                lam.core,
                some(&value, &v),
                none(&value)
            ));
            Ok((
                (two(&v, test, "", Expr::Flat(none(&value))), rt.clone()),
                acc,
            ))
        }
        ("is_some_and" | "is_none_or" | "is_ok_and" | "is_err_and", [f]) => {
            let on_err = name == "is_err_and";
            let param = if on_err {
                match &failure {
                    Some(e) => e.clone(),
                    None => return bad(),
                }
            } else {
                value.clone()
            };
            let (lam, acc) = closure(f, std::slice::from_ref(&param), acc, Some(&Ty::Bool))?;
            if lam.ty != Ty::Bool {
                return bad();
            }
            let b = lam.binders[0].clone();
            let test = lam.wrap(lam.core.clone());
            let otherwise = Expr::Flat((name == "is_none_or").to_string());
            if on_err {
                let (v, acc) = ctx.fresh(env, acc, "value");
                return Ok(((two(&v, otherwise, &b, test), Ty::Bool), acc));
            }
            Ok(((two(&b, test, &fresh_e, otherwise), Ty::Bool), acc))
        }
        ("ok_or", [e]) if failure.is_none() => {
            let ((ev, et), acc) = flat_of(e, env, ctx, acc, None)?;
            let (v, acc) = ctx.fresh(env, acc, "value");
            let types = result_of(&value, &et);
            Ok((
                (
                    two(
                        &v,
                        Expr::Flat(format!("(ctor Result.ok {types} {v})")),
                        "",
                        Expr::Flat(format!("(ctor Result.error {types} {ev})")),
                    ),
                    Ty::Outcome(Box::new(value.clone()), Box::new(et)),
                ),
                acc,
            ))
        }
        ("err", []) => {
            let Some(e) = &failure else { return bad() };
            let (v, acc) = ctx.fresh(env, acc, "value");
            Ok((
                (
                    two(
                        &v,
                        Expr::Flat(none(e)),
                        &fresh_e,
                        Expr::Flat(some(e, &fresh_e)),
                    ),
                    opt(e),
                ),
                acc,
            ))
        }
        ("unwrap_or", [d]) => {
            let ((dv, dt), acc) = flat_of(d, env, ctx, acc, Some(&value))?;
            if !dt.same(&value) {
                return Err(Residue::Form("unwrap_or of another type".into()));
            }
            let (v, acc) = ctx.fresh(env, acc, "value");
            Ok((
                (
                    two(&v, Expr::Flat(v.clone()), &fresh_e, Expr::Flat(dv)),
                    value,
                ),
                acc,
            ))
        }
        // `o.or(p)`, `o.or_else(|| p)`: the option where it holds a value,
        // else the other.
        ("or" | "or_else", [d]) if failure.is_none() => {
            let whole = opt(&value);
            let (other, acc) = if name == "or" {
                let ((dv, dt), acc) = flat_of(d, env, ctx, acc, Some(&whole))?;
                if !dt.same(&whole) {
                    return Err(Residue::Form(format!("{name} of another type")));
                }
                (Expr::Flat(dv), acc)
            } else {
                let (lam, acc) = closure(d, &[], acc, Some(&whole))?;
                if !lam.ty.same(&whole) {
                    return Err(Residue::Form(format!("{name} of another type")));
                }
                (lam.wrap(lam.core.clone()), acc)
            };
            let (v, acc) = ctx.fresh(env, acc, "value");
            let kept = Expr::Flat(some(&value, &v));
            Ok(((two(&v, kept, &fresh_e, other), whole), acc))
        }
        ("unwrap_or_else", [f]) => {
            let params: Vec<Ty> = failure.iter().cloned().collect();
            let (lam, acc) = closure(f, &params, acc, Some(&value))?;
            if !lam.ty.same(&value) {
                return Err(Residue::Form("unwrap_or_else of another type".into()));
            }
            let e = lam
                .binders
                .first()
                .cloned()
                .unwrap_or_else(|| fresh_e.clone());
            let (v, acc) = ctx.fresh(env, acc, "value");
            let fallback = lam.wrap(lam.core.clone());
            Ok(((two(&v, Expr::Flat(v.clone()), &e, fallback), value), acc))
        }
        ("unwrap_or_default" | "unwrap" | "expect", _) => {
            let rust_default = name == "unwrap_or_default";
            if env.total && !rust_default {
                return Err(Residue::Form(
                    "a partial source model cannot discharge a native panic obligation".into(),
                ));
            }
            if rust_default && matches!(value, Ty::Named(_)) {
                return Err(Residue::Form("unwrap_or_default of a named type".into()));
            }
            let Some(d) = default_of(&value, &env.types) else {
                return Err(Residue::Form(format!(".{name} of a type with no value")));
            };
            let (v, acc) = ctx.fresh(env, acc, "value");
            Ok((
                (
                    two(&v, Expr::Flat(v.clone()), &fresh_e, Expr::Flat(d)),
                    value,
                ),
                acc,
            ))
        }
        _ => bad(),
    }
}

/// `(ctor Option.some [T] body)` around a body that may hold lets.
fn wrap_some(t: &Ty, body: Expr) -> Expr {
    wrap_ctor(&format!("Option.some [{}]", t.render()), body)
}

/// A constructor applied to the tail of a body: lets stay outside.
fn wrap_ctor(ctor: &str, body: Expr) -> Expr {
    map_tails(body, &|tail| format!("(ctor {ctor} {tail})"))
}

/// The ASCII class tests of a byte, as ranges of its value.
fn ascii_byte(name: &str, b: &str) -> Option<String> {
    let range = |lo: u8, hi: u8| format!("(({lo} <=? {b}) && ({b} <=? {hi}))");
    let upper = range(b'A', b'Z');
    let lower = range(b'a', b'z');
    let digit = range(b'0', b'9');
    Some(match name {
        "is_ascii" => format!("({b} <? 128)"),
        "is_ascii_digit" => digit,
        "is_ascii_uppercase" => upper,
        "is_ascii_lowercase" => lower,
        "is_ascii_alphabetic" => format!("({upper} || {lower})"),
        "is_ascii_alphanumeric" => format!("({upper} || ({lower} || {digit}))"),
        "is_ascii_hexdigit" => format!(
            "({digit} || ({} || {}))",
            range(b'A', b'F'),
            range(b'a', b'f')
        ),
        "is_ascii_whitespace" => format!(
            "(({b} =? 32) || (({b} =? 9) || (({b} =? 10) || (({b} =? 12) || ({b} =? 13)))))"
        ),
        "is_ascii_punctuation" => format!(
            "({} || ({} || ({} || {})))",
            range(33, 47),
            range(58, 64),
            range(91, 96),
            range(123, 126)
        ),
        "is_ascii_control" => format!("(({b} <? 32) || ({b} =? 127))"),
        "is_ascii_graphic" => range(33, 126),
        _ => return None,
    })
}

/// A closure lifted in a scope of its own.
struct Lambda {
    /// The LexLean binder of each closure parameter.
    binders: Vec<String>,
    /// Projections a tuple parameter's pattern binds: `(name, type, value)`.
    lets: Vec<(String, String, String)>,
    /// The body, on one line.
    core: String,
    ty: Ty,
    calls: BTreeSet<String>,
    axioms: BTreeSet<String>,
    /// The parameters of the function being lifted that the body's
    /// recursive calls descend on: through an element of a list that is a
    /// sub-term of one, so the helper walking the list and the function
    /// are one mutual group.
    descends: BTreeSet<usize>,
}

impl Lambda {
    /// The body's projections around a term that uses its binders.
    fn wrap(&self, term: String) -> Expr {
        self.wrap_expr(Expr::Flat(term))
    }

    /// The body's projections around an expression that uses its binders.
    fn wrap_expr(&self, term: Expr) -> Expr {
        self.lets
            .iter()
            .rev()
            .fold(term, |body, (name, ty, value)| Expr::Let {
                name: name.clone(),
                ty: ty.clone(),
                value: Box::new(Expr::Flat(value.clone())),
                body: Box::new(body),
            })
    }
}

/// Bind a closure parameter's pattern: a name, `_`, `&p`, `p: T`, or a tuple
/// of those over a tuple element.
/// The projections bound so far come in and go out with the new ones.
type Lets = Vec<(String, String, String)>;

/// A tuple or struct pattern: the whole bound to a fresh binder, and each
/// part's pattern bound to its item or field of it, in turn, so a part's
/// own parts follow it.
fn bind_parts(
    parts: &[(&rs::Pat, Ty, String)],
    ty: &Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    lets: Lets,
) -> Lift<(Ctx, String, Acc, Lets)> {
    let (whole, mut acc) = ctx.fresh(env, acc, "pair");
    let mut scope = ctx.clone();
    let mut lets = lets;
    for (pat, t, part) in parts {
        let (next, name, after, inner) = bind_pat(pat, t, env, &scope, acc, Vec::new())?;
        scope = next;
        acc = after;
        let value = match (ty, part.parse::<usize>()) {
            (Ty::Tuple(_) | Ty::Pair(_), Ok(i)) => tuple_item(ty, &whole, i),
            (Ty::Named(n), _) if env.types.boxed(n) => {
                format!("({} {whole})", Types::accessor(n, part))
            }
            _ => format!("(project {whole} {part})"),
        };
        lets.push((name, t.render(), value));
        lets.extend(inner);
    }
    Ok((scope, whole, acc, lets))
}

fn bind_pat(
    p: &rs::Pat,
    ty: &Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    lets: Lets,
) -> Lift<(Ctx, String, Acc, Lets)> {
    match (p, ty) {
        (rs::Pat::Ident(i), _) if i.subpat.is_none() && !i.by_ref => {
            let (scope, name, acc) = ctx.bind(env, acc, &i.ident, ty.clone());
            Ok((scope, name, acc, lets))
        }
        (rs::Pat::Wild, _) => {
            let (name, acc) = ctx.fresh(env, acc, "item");
            Ok((ctx.clone(), name, acc, lets))
        }
        (rs::Pat::Reference(r), _) => bind_pat(r, ty, env, ctx, acc, lets),
        (rs::Pat::Type(pt), _) => bind_pat(&pt.pat, ty, env, ctx, acc, lets),
        (rs::Pat::Tuple(tp), Ty::Tuple(ts) | Ty::Pair(ts)) if tp.len() == ts.len() => {
            let parts: Vec<(&rs::Pat, Ty, String)> = tp
                .iter()
                .zip(ts)
                .enumerate()
                .map(|(i, (e, t))| (e, t.clone(), format!("{i}")))
                .collect();
            bind_parts(&parts, ty, env, ctx, acc, lets)
        }
        // `S { a, b: p, .. }`: each named field's pattern, from the value.
        (rs::Pat::Struct(ps), Ty::Named(n)) if !ps.path.segments.is_empty() => {
            let fields = env
                .types
                .structs
                .get(n)
                .ok_or_else(|| Residue::Form("closure pattern".to_string()))?;
            let last = ps.path.segments.last().map(|s| env.types.resolve(&s.ident));
            if last.as_deref() != Some(n.as_str()) || (!ps.rest && ps.fields.len() != fields.len())
            {
                return Err(Residue::Form("closure pattern".to_string()));
            }
            let mut parts = Vec::new();
            for f in &ps.fields {
                let rs::Member::Named(m) = &f.member else {
                    return Err(Residue::Form("closure pattern".to_string()));
                };
                let field = local_name(m);
                let Some((_, t)) = fields.iter().find(|(name, _)| *name == field) else {
                    return Err(Residue::Form("closure pattern".to_string()));
                };
                if let Some(kind) = env.types.cell(n, &field) {
                    if !matches!(f.pat, rs::Pat::Wild) {
                        return Err(cell_unread(kind, n, &field, None));
                    }
                }
                parts.push((&f.pat, t.clone(), field));
            }
            bind_parts(&parts, ty, env, ctx, acc, lets)
        }
        _ => Err(Residue::Form("closure pattern".to_string())),
    }
}

/// A closure argument over parameters of these types.
/// The closure `|a0, ..| f(a0, ..)` of a function path `f` of `arity`
/// parameters; for a path `T::m` that names no lifted function, a method
/// of another type, `|a0, ..| a0.m(..)`.
fn applying(f: &rs::Path, arity: usize, env: &Env) -> rs::RustExpr {
    let names: Vec<String> = (0..arity).map(|i| format!("argument{i}")).collect();
    let local = |n: &String| {
        rs::RustExpr::Path(rs::Path {
            segments: vec![rs::PathSegment {
                ident: n.clone(),
                arguments: rs::PathArguments::None,
            }],
            text: n.clone(),
        })
    };
    rs::RustExpr::Closure(rs::ExprClosure {
        inputs: names
            .iter()
            .map(|n| {
                rs::Pat::Ident(rs::PatIdent {
                    ident: n.clone(),
                    by_ref: false,
                    subpat: None,
                })
            })
            .collect(),
        body: Box::new(applied_path(f, names.iter().map(local).collect(), env)),
        asyncness: false,
    })
}

/// A call of the function path `f` on `args`: [`applying`]'s body.
fn applied_path(f: &rs::Path, args: Vec<rs::RustExpr>, env: &Env) -> rs::RustExpr {
    let names: Vec<&String> = f.segments.iter().map(|s| &s.ident).collect();
    let lifted = match names.as_slice() {
        [.., owner, method] => env
            .fns
            .contains_key(&format!("{}::{method}", env.types.resolve(owner))),
        _ => true,
    };
    // `T::from` converts its argument: a call, not a method of it.
    let converts = names.last().is_some_and(|m| *m == "from");
    match (lifted || converts, names.last(), args.split_first()) {
        (false, Some(method), Some((receiver, rest))) => {
            rs::RustExpr::MethodCall(rs::ExprMethodCall {
                receiver: Box::new(receiver.clone()),
                method: (*method).clone(),
                turbofish: None,
                args: rest.to_vec(),
            })
        }
        _ => rs::RustExpr::Call(rs::ExprCall {
            func: Box::new(rs::RustExpr::Path(f.clone())),
            args,
        }),
    }
}

fn lambda(
    e: &rs::RustExpr,
    params: &[Ty],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<(Lambda, Acc)> {
    // A function given by its path is the closure that calls it.
    let named = match e {
        rs::RustExpr::Path(p) => Some(applying(p, params.len(), env)),
        _ => None,
    };
    let Some(rs::RustExpr::Closure(c)) = named.as_ref().or(Some(e)) else {
        return Err(Residue::Form(
            "an iterator adapter given a function, not a closure".to_string(),
        ));
    };
    if c.inputs.len() != params.len() || c.asyncness {
        return Err(Residue::Form("closure".to_string()));
    }
    let (mut acc, before) = acc.apart();
    let mut scope = Ctx {
        tails: BTreeMap::new(),
        top: false,
        looping: None,
        hidden: Vec::new(),
        elements: None,
        sealed: ctx.sealing(),
        ..ctx.clone()
    };
    let mut binders = Vec::new();
    let mut lets = Vec::new();
    for (p, ty) in c.inputs.iter().zip(params) {
        let (mut next, name, after, more) = bind_pat(p, ty, env, &scope, acc, lets)?;
        // An element of a list that is a sub-term of a parameter is one.
        if let (Some(param), true) = (&ctx.elements, binders.is_empty()) {
            next.tails.insert(name.clone(), param.clone());
        }
        scope = next;
        acc = after;
        lets = more;
        binders.push(name);
    }
    // A `?` in a closure that gives an option leaves the closure with
    // `None`: the body is the match it stands for.
    let body = match want {
        Some(Ty::Option(_)) => closure_tries(&c.body, acc.given.len()),
        _ => (*c.body).clone(),
    };
    let ((core, ty), acc) = flat_of(&body, env, &scope, acc, want)?;
    let (acc, own) = acc.rejoin(before);
    if !own.descends.is_empty() && ctx.elements.is_none() {
        return Err(Residue::Form("recursion inside a closure".to_string()));
    }
    Ok((
        Lambda {
            binders,
            lets,
            core,
            ty,
            calls: own.calls,
            axioms: own.axioms,
            descends: own.descends,
        },
        acc,
    ))
}

/// A closure body giving an option, each `?` it evaluates unconditionally
/// written as the match that leaves with `None`, numbered from `n`.
fn closure_tries(body: &rs::RustExpr, n: usize) -> rs::RustExpr {
    let var = format!("tried{}", n + 1);
    let Some((found, rest)) = take_try(body, &var) else {
        return body.clone();
    };
    let held = rs::Pat::TupleStruct(rs::PatTupleStruct {
        path: rs::Path::ident("Some"),
        elems: vec![rs::Pat::Ident(rs::PatIdent {
            ident: var,
            by_ref: false,
            subpat: None,
        })],
    });
    let none = rs::Path::ident("None");
    rs::RustExpr::Match(rs::ExprMatch {
        expr: Box::new(found),
        arms: vec![
            rs::MatchArm {
                pat: held,
                body: Box::new(closure_tries(&rest, n + 1)),
            },
            rs::MatchArm {
                pat: rs::Pat::Path(none.clone()),
                body: Box::new(rs::RustExpr::Path(none)),
            },
        ],
    })
}

/// The outer locals a helper's body names: they become its parameters.
fn captures(ctx: &Ctx, texts: &[String], bound: &[String]) -> Vec<(String, Ty)> {
    // A dictionary's method is a parameter of the definition, captured
    // as a local is.
    let dictionary = ctx
        .dictionary
        .iter()
        .map(|e| (e.name.clone(), e.ty.clone()));
    let found: BTreeSet<(String, Ty)> = ctx
        .locals
        .values()
        .cloned()
        .chain(dictionary)
        .filter(|(n, _)| !bound.contains(n) && texts.iter().any(|t| mentions_word(t, n)))
        .collect();
    found.into_iter().collect()
}

/// Whether a character continues a LexLean word.
fn word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Whether `text` holds `word` as a whole word: an occurrence that no word
/// character continues on either side.
fn mentions_word(text: &str, word: &str) -> bool {
    #[cfg(not(uorust_lift))]
    crate::certified::certified!(lift_mentions_word, text, word);
    !word.is_empty()
        && word.chars().all(word_char)
        && text.contains(word)
        && apart(&text.split(word).collect::<Vec<&str>>())
}

/// Whether two consecutive pieces of a split meet where neither continues
/// a word: the separator between them stands alone.
fn apart(pieces: &[&str]) -> bool {
    pieces
        .iter()
        .zip(pieces.iter().skip(1))
        .any(|(before, after)| {
            !before.chars().last().is_some_and(word_char)
                && !after.chars().next().is_some_and(word_char)
        })
}

/// The recursive call of a walk, given the values it threads.
type Recur<'a> = &'a dyn Fn(&[String]) -> String;
/// A walk's step from its recursive call.
type WalkStep<'a> = &'a dyn Fn(Recur<'_>, &str) -> Expr;

/// A walk over a list: a helper that matches its first parameter and calls
/// itself on the tail. `lead` are parameters after the list that the walk
/// threads (a fold's accumulator); `cons` builds the step from the head's
/// binder and the recursive call's text given the lead values.
#[allow(clippy::too_many_arguments)]
fn walk(
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    kind: &str,
    list: &str,
    elem: &Ty,
    lam: &Lambda,
    head: &str,
    lead: &[(String, Ty, String)],
    result: &Ty,
    nil: String,
    cons: WalkStep<'_>,
) -> ((Expr, Ty), Acc) {
    let name = ctx.helper_name(env, &acc, kind);
    let (items, acc) = ctx.fresh(env, acc, "items");
    let (rest, acc) = ctx.fresh(env, acc, "rest");
    let mut bound: Vec<String> = lam.binders.clone();
    bound.extend(lam.lets.iter().map(|(n, _, _)| n.clone()));
    bound.extend(lead.iter().map(|(n, _, _)| n.clone()));
    let lets_text: Vec<String> = lam.lets.iter().map(|(_, _, v)| v.clone()).collect();
    let mut texts = vec![lam.core.clone()];
    texts.extend(lets_text);
    let caps = captures(ctx, &texts, &bound);
    let cap_names: Vec<String> = caps.iter().map(|(n, _)| n.clone()).collect();
    let recur = |values: &[String]| {
        let mut parts = vec![name.clone(), rest.clone()];
        parts.extend(values.iter().cloned());
        parts.extend(cap_names.iter().cloned());
        format!("({})", parts.join(" "))
    };
    let step = cons(&recur, &rest);
    let mut params = vec![(items.clone(), Ty::List(Box::new(elem.clone())).render())];
    params.extend(lead.iter().map(|(n, t, _)| (n.clone(), t.render())));
    params.extend(caps.iter().map(|(n, t)| (n.clone(), t.render())));
    let acc = acc.helper(Helper {
        name: name.clone(),
        params,
        result: result.render(),
        body: Expr::Match {
            scrutinee: items,
            arms: vec![
                Arm {
                    pattern: "List.nil".to_string(),
                    body: Expr::Flat(nil),
                },
                Arm {
                    pattern: format!("List.cons {head} {rest}"),
                    body: lam.wrap_expr(step),
                },
            ],
        },
        calls: lam.calls.clone(),
        axioms: lam.axioms.clone(),
        recursive: true,
        entwined: !lam.descends.is_empty(),
        termination: None,
    });
    let acc = lam
        .descends
        .iter()
        .fold(acc.called(format!("#{name}")), |acc, i| acc.descended(*i));
    let mut parts = vec![name, list.to_string()];
    parts.extend(lead.iter().map(|(_, _, v)| v.clone()));
    parts.extend(cap_names);
    (
        (Expr::Flat(format!("({})", parts.join(" "))), result.clone()),
        acc,
    )
}

/// A lifted value and the state after it, as the value of an optional lift.
fn some<T>((value, acc): (T, Acc)) -> (Option<T>, Acc) {
    (Some(value), acc)
}

/// Equality of two values of a type the lift compares.
fn equality(ty: &Ty, a: &str, b: &str) -> Option<String> {
    match ty {
        Ty::Nat(_) => Some(format!("({a} =? {b})")),
        Ty::Str | Ty::Char | Ty::Int(_) => Some(format!("(prim equal {a} {b} : Bool)")),
        _ => None,
    }
}

/// Equality of two values of `ty`: a primitive's, a unit-variant enum's
/// constructors, or else its derived order's `Equal`.
fn same_value(ty: &Ty, a: &str, b: &str, env: &Env, acc: Acc) -> Option<(String, Acc)> {
    if let Some(e) = equality(ty, a, b) {
        return Some((e, acc));
    }
    if is_enum_or_bool(ty, env) {
        return Some((same_constructor(ty, a, b), acc));
    }
    let test = crate::order::operator("==", ty, a, b)?;
    crate::order::derived(ty, &env.types).then(|| (test, acc.ordered(ty, &env.types)))
}

/// A search of a list for a needle: `contains` on a list or set (`entry`
/// `None`), or a map's `contains_key` and `get` over its entries (`entry`
/// the value type, `get` when `kind` is `Get`).
#[allow(clippy::too_many_arguments)]
fn lookup(
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    kind: &str,
    r: &str,
    elem: &Ty,
    value: Option<&Ty>,
    needle: &rs::RustExpr,
) -> Lift<((Expr, Ty), Acc)> {
    let key_ty = match (elem, value) {
        (Ty::Tuple(kv) | Ty::Pair(kv), Some(_)) => kv[0].clone(),
        _ => elem.clone(),
    };
    let ((v, vt), acc) = flat_of(needle, env, ctx, acc, Some(&key_ty))?;
    if !vt.same(&key_ty) {
        return Err(Residue::Form(format!("{kind} of another type")));
    }
    let (head, acc) = ctx.fresh(env, acc, "item");
    let (probe, acc) = ctx.fresh(env, acc, "needle");
    let subject = if value.is_some() {
        tuple_item(elem, &head, 0)
    } else {
        head.clone()
    };
    let ordered = equality(&key_ty, &subject, &probe).is_none() && !is_enum_or_bool(&key_ty, env);
    let Some((test, acc)) = same_value(&key_ty, &subject, &probe, env, acc) else {
        return Err(Residue::Form(format!("{kind} over keys of this type")));
    };
    let lam = Lambda {
        binders: vec![head.clone()],
        lets: Vec::new(),
        core: test.clone(),
        ty: Ty::Bool,
        calls: BTreeSet::new(),
        axioms: if ordered {
            crate::order::axioms(&key_ty, &env.types)
        } else {
            BTreeSet::new()
        },
        descends: BTreeSet::new(),
    };
    let (result, nil, hit) = match value {
        Some(vt) if kind == "Get" => (
            Ty::Option(Box::new(vt.clone())),
            format!("(ctor Option.none [{}])", vt.render()),
            format!(
                "(ctor Option.some [{}] {})",
                vt.render(),
                tuple_item(elem, &head, 1)
            ),
        ),
        _ => (Ty::Bool, "false".to_string(), "true".to_string()),
    };
    let probe_arg = probe.clone();
    let cons = |rec: Recur<'_>, _: &str| {
        Expr::Flat(format!(
            "(if {test} then {hit} else {})",
            rec(std::slice::from_ref(&probe_arg))
        ))
    };
    Ok(walk(
        env,
        ctx,
        acc,
        kind,
        r,
        elem,
        &lam,
        &head,
        &[(probe, key_ty, v)],
        &result,
        nil,
        &cons,
    ))
}

/// The Rust name the lift binds a member under while it searches another
/// collection for it: no Rust identifier spells it.
const PROBE: &str = "#probe";

/// `a.is_disjoint(b)`: no member of `a` is a member of `b`, which must be
/// a local the search captures.
fn disjoint(
    r: &str,
    elem: &Ty,
    other: &rs::RustExpr,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let set = Ty::Set(Box::new(elem.clone()), true);
    let ((o, ot), acc) = flat_of(other, env, ctx, acc, Some(&set))?;
    let local = ctx.locals.values().any(|(n, _)| *n == o);
    if !local || !matches!(&ot, Ty::Set(t, _) | Ty::List(t) if t.same(elem)) {
        return Err(Residue::Form(".is_disjoint of another set".into()));
    }
    let (head, acc) = ctx.fresh(env, acc, "member");
    let mut probing = ctx.clone();
    probing
        .locals
        .insert(PROBE.into(), (head.clone(), elem.clone()));
    let probe = rs::RustExpr::Path(rs::Path {
        segments: vec![rs::PathSegment {
            ident: PROBE.into(),
            arguments: rs::PathArguments::None,
        }],
        text: PROBE.into(),
    });
    let before = acc.calls.clone();
    let ((test, _), acc) = lookup(env, &probing, acc, "Contains", &o, elem, None, &probe)?;
    let calls: BTreeSet<String> = acc.calls.difference(&before).cloned().collect();
    let axioms: BTreeSet<String> = acc
        .helpers
        .iter()
        .filter(|h| calls.contains(&format!("#{}", h.name)))
        .flat_map(|h| h.axioms.iter().cloned())
        .collect();
    let core = test.flat();
    let lam = Lambda {
        binders: vec![head.clone()],
        lets: Vec::new(),
        core: core.clone(),
        ty: Ty::Bool,
        calls,
        axioms,
        descends: BTreeSet::new(),
    };
    let cons =
        |rec: Recur<'_>, _: &str| Expr::Flat(format!("(if {core} then false else {})", rec(&[])));
    Ok(walk(
        env,
        ctx,
        acc,
        "Disjoint",
        r,
        elem,
        &lam,
        &head,
        &[],
        &Ty::Bool,
        "true".into(),
        &cons,
    ))
}

/// Reads of a map or a set, held as the list of its entries or members:
/// `get`, `contains_key`, `contains`, `len` and `is_empty`, and, where the
/// order is the key order, `iter`, `keys` and `values`.
fn collection(
    name: &str,
    r: &str,
    rt: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    let flat = |s: String, t: Ty, acc: Acc| Ok(((Expr::Flat(s), t), acc));
    let unordered = || {
        Err(Residue::Form(format!(
            ".{name} on a hash collection, whose order is unspecified"
        )))
    };
    match (rt, name, args) {
        (_, "clone", []) => flat(r.to_string(), rt.clone(), acc),
        (_, "len", []) => flat(format!("(prim length {r} : Nat)"), Ty::Nat(None), acc),
        (_, "is_empty", []) => flat(format!("((prim length {r} : Nat) =? 0)"), Ty::Bool, acc),
        (Ty::Map(_, v, _), "get", [k]) => lookup(env, ctx, acc, "Get", r, &rt.entry(), Some(v), k),
        (Ty::Map(_, v, _), "contains_key", [k]) => {
            lookup(env, ctx, acc, "Has", r, &rt.entry(), Some(v), k)
        }
        (Ty::Set(t, _), "contains", [k]) => lookup(env, ctx, acc, "Contains", r, t, None, k),
        (Ty::Set(t, _), "is_disjoint", [other]) => disjoint(r, t, other, env, ctx, acc),
        (Ty::Map(..), "iter" | "into_iter", []) => match rt {
            Ty::Map(_, _, true) => flat(r.to_string(), Ty::List(Box::new(rt.entry())), acc),
            _ => unordered(),
        },
        (Ty::Set(t, sorted), "iter" | "into_iter", []) => {
            if *sorted {
                flat(r.to_string(), Ty::List(t.clone()), acc)
            } else {
                unordered()
            }
        }
        (Ty::Map(k, v, sorted), "keys" | "values", []) => {
            if !*sorted {
                return unordered();
            }
            let (field, ty) = if name == "keys" { (0, k) } else { (1, v) };
            let (head, acc) = ctx.fresh(env, acc, "entry");
            let core = tuple_item(&rt.entry(), &head, field);
            let lam = Lambda {
                binders: vec![head.clone()],
                lets: Vec::new(),
                core: core.clone(),
                ty: (**ty).clone(),
                calls: BTreeSet::new(),
                axioms: BTreeSet::new(),
                descends: BTreeSet::new(),
            };
            let cons = |rec: Recur<'_>, _: &str| Expr::Flat(format!("(cons {core} {})", rec(&[])));
            let kind = if name == "keys" { "Keys" } else { "Values" };
            Ok(walk(
                env,
                ctx,
                acc,
                kind,
                r,
                &rt.entry(),
                &lam,
                &head,
                &[],
                &Ty::List(ty.clone()),
                format!("(nil {})", ty.render()),
                &cons,
            ))
        }
        _ => Err(Residue::Form(format!("method call .{name}"))),
    }
}

/// A list combinator's value with nothing left to lift: its term and type.
fn flat_some(s: String, t: Ty, acc: Acc) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    Ok((Some((Expr::Flat(s), t)), acc))
}

/// Iterator adapters and consumers over a list, with pure closures:
/// `iter`, `into_iter`, `map`, `filter`, `any`, `all`, `find`, `fold`,
/// `count`, `contains`, `collect` and `join`. `None` when the method is none
/// of them.
#[allow(clippy::too_many_arguments)]
fn combinator(
    m: &rs::ExprMethodCall,
    name: &str,
    r: &str,
    elem: &Ty,
    args: &[&rs::RustExpr],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let list = Ty::List(Box::new(elem.clone()));
    let flat = |s: String, t: Ty, acc: Acc| Ok((Some((Expr::Flat(s), t)), acc));
    match (name, args) {
        ("iter" | "into_iter", []) => flat(r.to_string(), list, acc),
        ("collect", []) => comb_collect(m, r, elem, &list, acc),
        ("count", []) => flat(format!("(prim length {r} : Nat)"), Ty::Nat(None), acc),
        ("join", [sep]) if *elem == Ty::Str => comb_join(sep, r, env, ctx, acc),
        // `a.zip(b)`: the pairs of items at the same place, as long as the
        // shorter.
        ("zip", [other]) => comb_zip(other, r, elem, &list, env, ctx, acc),
        ("contains", [needle]) => Ok(some(lookup(
            env, ctx, acc, "Contains", r, elem, None, needle,
        )?)),
        ("map" | "filter" | "any" | "all" | "find", [f]) => {
            comb_map(f, name, r, elem, &list, env, ctx, acc)
        }
        ("fold", [init, f]) => comb_fold(init, f, r, elem, env, ctx, acc),
        ("next" | "first", []) => flat(
            format!("(prim index {r} 0 : (Option {}))", elem.render()),
            Ty::Option(Box::new(elem.clone())),
            acc,
        ),
        ("last", []) => flat(
            format!(
                "(prim index {r} (prim subtract (prim length {r} : Nat) 1 : Nat) : (Option {}))",
                elem.render()
            ),
            Ty::Option(Box::new(elem.clone())),
            acc,
        ),
        ("nth", [n]) => comb_nth(n, r, elem, env, ctx, acc),
        ("chain", [other]) => comb_chain(other, r, &list, env, ctx, acc),
        ("rev" | "enumerate", []) => comb_rev(name, r, elem, &list, env, ctx, acc),
        ("skip" | "take", [n]) => comb_skip(n, name, r, elem, &list, env, ctx, acc),
        ("take_while" | "skip_while" | "position", [f]) => {
            comb_while(f, name, r, elem, &list, env, ctx, acc)
        }
        ("filter_map" | "find_map" | "flat_map", [f]) => {
            comb_filter_map(f, name, r, elem, env, ctx, acc)
        }
        _ => Ok((None, acc)),
    }
}

/// `.collect()`: the list itself, or, into a `String`, its pieces joined.
fn comb_collect(
    m: &rs::ExprMethodCall,
    r: &str,
    elem: &Ty,
    list: &Ty,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let to_string = matches!(
        m.turbofish.as_deref(),
        Some([rs::GenericArgument::Type(t)]) if t.text == "String"
    );
    if to_string {
        // A `char` is held as the one-character string it is.
        if !matches!(elem, Ty::Str | Ty::Char) {
            return Err(Residue::Form("collect of non-strings into a String".into()));
        }
        let acc = acc.axiom("propext");
        return flat_some(format!("(prim join {r} \"\" : String)"), Ty::Str, acc);
    }
    flat_some(r.to_string(), list.clone(), acc)
}

/// `.join(sep)` of strings.
fn comb_join(
    sep: &rs::RustExpr,
    r: &str,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let ((s, st), acc) = flat_of(sep, env, ctx, acc, Some(&Ty::Str))?;
    if st != Ty::Str {
        return Err(Residue::Form(
            "join with a separator of another type".into(),
        ));
    }
    let acc = acc.axiom("propext");
    flat_some(format!("(prim join {r} {s} : String)"), Ty::Str, acc)
}

/// `a.zip(b)`: the pairs of items at the same place, as long as the shorter.
fn comb_zip(
    other: &rs::RustExpr,
    r: &str,
    elem: &Ty,
    list: &Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let ((o, ot), acc) = flat_of(other, env, ctx, acc, None)?;
    let oe = match &ot {
        Ty::List(t) | Ty::Set(t, true) => (**t).clone(),
        Ty::Map(_, _, true) => ot.entry(),
        _ => {
            return Err(Residue::Form(
                ".zip with something other than a list".into(),
            ))
        }
    };
    let pair = env.types.tuple(vec![elem.clone(), oe.clone()]);
    let pairs = Ty::List(Box::new(pair.clone()));
    let name = ctx.helper_name(env, &acc, "Zip");
    let (items, acc) = ctx.fresh(env, acc, "items");
    let (others, acc) = ctx.fresh(env, acc, "others");
    let (head, acc) = ctx.fresh(env, acc, "item");
    let (rest, acc) = ctx.fresh(env, acc, "rest");
    let (partner, acc) = ctx.fresh(env, acc, "partner");
    let (more, acc) = ctx.fresh(env, acc, "more");
    let value = tuple_value(&pair, &[head.clone(), partner.clone()]);
    let body = Expr::Match {
        scrutinee: items.clone(),
        arms: vec![
            Arm {
                pattern: "List.nil".into(),
                body: Expr::Flat(format!("(nil {})", pair.render())),
            },
            Arm {
                pattern: format!("List.cons {head} {rest}"),
                body: Expr::Match {
                    scrutinee: others.clone(),
                    arms: vec![
                        Arm {
                            pattern: "List.nil".into(),
                            body: Expr::Flat(format!("(nil {})", pair.render())),
                        },
                        Arm {
                            pattern: format!("List.cons {partner} {more}"),
                            body: Expr::Flat(format!("(cons {value} ({name} {rest} {more}))")),
                        },
                    ],
                },
            },
        ],
    };
    let acc = acc.saw(&pair).helper(Helper {
        name: name.clone(),
        params: vec![
            (items, list.render()),
            (others, Ty::List(Box::new(oe)).render()),
        ],
        result: pairs.render(),
        body,
        calls: BTreeSet::new(),
        axioms: BTreeSet::new(),
        recursive: true,
        entwined: false,
        termination: None,
    });
    let acc = acc.called(format!("#{name}"));
    flat_some(format!("({name} {r} {o})"), pairs, acc)
}

/// `.map(f)`, `.filter(p)`, `.any(p)`, `.all(p)` and `.find(p)`.
#[allow(clippy::too_many_arguments)]
fn comb_map(
    f: &rs::RustExpr,
    name: &str,
    r: &str,
    elem: &Ty,
    list: &Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let want = if name == "map" { None } else { Some(&Ty::Bool) };
    // A list that is a sub-term of a parameter: the closure may
    // recurse on its elements.
    let ranged = Ctx {
        elements: ctx.tails.get(r).cloned(),
        ..ctx.clone()
    };
    let (lam, acc) = lambda(f, std::slice::from_ref(elem), env, &ranged, acc, want)?;
    if name != "map" && lam.ty != Ty::Bool {
        return Err(Residue::Form(format!(
            "a .{name} predicate of another type"
        )));
    }
    let head = lam.binders[0].clone();
    let core = lam.core.clone();
    let (kind, result, nil): (&str, Ty, String) = match name {
        "map" => (
            "Map",
            Ty::List(Box::new(lam.ty.clone())),
            format!("(nil {})", lam.ty.render()),
        ),
        "filter" => ("Filter", list.clone(), format!("(nil {})", elem.render())),
        "any" => ("Any", Ty::Bool, "false".into()),
        "all" => ("All", Ty::Bool, "true".into()),
        _ => (
            "Find",
            Ty::Option(Box::new(elem.clone())),
            format!("(ctor Option.none [{}])", elem.render()),
        ),
    };
    let elem_text = elem.render();
    let cons = |rec: Recur<'_>, _: &str| -> Expr {
        let next = rec(&[]);
        Expr::Flat(match name {
            "map" => format!("(cons {core} {next})"),
            "filter" => format!("(if {core} then (cons {head} {next}) else {next})"),
            "any" => format!("(if {core} then true else {next})"),
            "all" => format!("(if {core} then {next} else false)"),
            _ => format!("(if {core} then (ctor Option.some [{elem_text}] {head}) else {next})"),
        })
    };
    Ok(some(walk(
        env,
        ctx,
        acc,
        kind,
        r,
        elem,
        &lam,
        &head,
        &[],
        &result,
        nil,
        &cons,
    )))
}

/// `.fold(init, f)`.
fn comb_fold(
    init: &rs::RustExpr,
    f: &rs::RustExpr,
    r: &str,
    elem: &Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let ((iv, it), acc) = flat_of(init, env, ctx, acc, None)?;
    let (lam, acc) = lambda(f, &[it.clone(), elem.clone()], env, ctx, acc, Some(&it))?;
    if !lam.ty.same(&it) {
        return Err(Residue::Form("a fold step of another type".into()));
    }
    let folded = lam.binders[0].clone();
    let head = lam.binders[1].clone();
    let core = lam.core.clone();
    let cons = |rec: Recur<'_>, _: &str| Expr::Flat(rec(std::slice::from_ref(&core)));
    Ok(some(walk(
        env,
        ctx,
        acc,
        "Fold",
        r,
        elem,
        &lam,
        &head,
        &[(folded.clone(), it.clone(), iv)],
        &it,
        folded,
        &cons,
    )))
}

/// `.nth(n)`.
fn comb_nth(
    n: &rs::RustExpr,
    r: &str,
    elem: &Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let ((n, nt), acc) = flat_of(n, env, ctx, acc, Some(&Ty::Nat(None)))?;
    if !matches!(nt, Ty::Nat(_)) {
        return Err(Residue::Form("nth of another type".into()));
    }
    flat_some(
        format!("(prim index {r} {n} : (Option {}))", elem.render()),
        Ty::Option(Box::new(elem.clone())),
        acc,
    )
}

/// `.chain(other)`.
fn comb_chain(
    other: &rs::RustExpr,
    r: &str,
    list: &Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let ((o, ot), acc) = flat_of(other, env, ctx, acc, Some(list))?;
    if !ot.same(list) {
        return Err(Residue::Form("chain of another type".into()));
    }
    flat_some(
        format!("(prim append {r} {o} : {})", list.render()),
        list.clone(),
        acc,
    )
}

/// `.rev()` and `.enumerate()`.
#[allow(clippy::too_many_arguments)]
fn comb_rev(
    name: &str,
    r: &str,
    elem: &Ty,
    list: &Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let (head, acc) = ctx.fresh(env, acc, "item");
    let lam = Lambda {
        binders: vec![head.clone()],
        lets: Vec::new(),
        core: head.clone(),
        ty: elem.clone(),
        calls: BTreeSet::new(),
        axioms: BTreeSet::new(),
        descends: BTreeSet::new(),
    };
    if name == "rev" {
        let e = elem.render();
        let cons = |rec: Recur<'_>, _: &str| {
            Expr::Flat(format!(
                "(prim append {} (cons {head} (nil {e})) : (List {e}))",
                rec(&[])
            ))
        };
        return Ok(some(walk(
            env,
            ctx,
            acc,
            "Rev",
            r,
            elem,
            &lam,
            &head,
            &[],
            list,
            format!("(nil {e})"),
            &cons,
        )));
    }
    let pair = env.types.tuple(vec![Ty::Nat(None), elem.clone()]);
    let acc = acc.saw(&pair);
    let (index, acc) = ctx.fresh(env, acc, "index");
    let pair_name = pair.render();
    let item = tuple_value(&pair, &[index.clone(), head.clone()]);
    let cons = |rec: Recur<'_>, _: &str| {
        Expr::Flat(format!(
            "(cons {item} {})",
            rec(&[format!("({index} + 1)")])
        ))
    };
    Ok(some(walk(
        env,
        ctx,
        acc,
        "Enumerate",
        r,
        elem,
        &lam,
        &head,
        &[(index.clone(), Ty::Nat(None), "0".into())],
        &Ty::List(Box::new(pair.clone())),
        format!("(nil {pair_name})"),
        &cons,
    )))
}

/// `.skip(n)` and `.take(n)`.
#[allow(clippy::too_many_arguments)]
fn comb_skip(
    n: &rs::RustExpr,
    name: &str,
    r: &str,
    elem: &Ty,
    list: &Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let ((n, nt), acc) = flat_of(n, env, ctx, acc, Some(&Ty::Nat(None)))?;
    if !matches!(nt, Ty::Nat(_)) {
        return Err(Residue::Form(format!(".{name} of another type")));
    }
    let (head, acc) = ctx.fresh(env, acc, "item");
    let (count, acc) = ctx.fresh(env, acc, "count");
    let lam = Lambda {
        binders: vec![head.clone()],
        lets: Vec::new(),
        core: head.clone(),
        ty: elem.clone(),
        calls: BTreeSet::new(),
        axioms: BTreeSet::new(),
        descends: BTreeSet::new(),
    };
    let e = elem.render();
    let take = name == "take";
    let cons = |rec: Recur<'_>, _: &str| {
        let fewer = rec(&[format!("(prim subtract {count} 1 : Nat)")]);
        let same = rec(std::slice::from_ref(&count));
        Expr::Flat(if take {
            format!("(if ({count} =? 0) then (nil {e}) else (cons {head} {fewer}))")
        } else {
            format!("(if ({count} =? 0) then (cons {head} {same}) else {fewer})")
        })
    };
    let kind = if take { "Take" } else { "Skip" };
    Ok(some(walk(
        env,
        ctx,
        acc,
        kind,
        r,
        elem,
        &lam,
        &head,
        &[(count.clone(), Ty::Nat(None), n)],
        list,
        format!("(nil {e})"),
        &cons,
    )))
}

/// `.take_while(p)`, `.skip_while(p)` and `.position(p)`.
#[allow(clippy::too_many_arguments)]
fn comb_while(
    f: &rs::RustExpr,
    name: &str,
    r: &str,
    elem: &Ty,
    list: &Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let (lam, acc) = lambda(
        f,
        std::slice::from_ref(elem),
        env,
        ctx,
        acc,
        Some(&Ty::Bool),
    )?;
    if lam.ty != Ty::Bool {
        return Err(Residue::Form(format!(
            "a .{name} predicate of another type"
        )));
    }
    let head = lam.binders[0].clone();
    let core = lam.core.clone();
    let e = elem.render();
    if name == "position" {
        let (index, acc) = ctx.fresh(env, acc, "index");
        let cons = |rec: Recur<'_>, _: &str| {
            Expr::Flat(format!(
                "(if {core} then (ctor Option.some [Nat] {index}) else {})",
                rec(&[format!("({index} + 1)")])
            ))
        };
        return Ok(some(walk(
            env,
            ctx,
            acc,
            "Position",
            r,
            elem,
            &lam,
            &head,
            &[(index.clone(), Ty::Nat(None), "0".into())],
            &Ty::Option(Box::new(Ty::Nat(None))),
            "(ctor Option.none [Nat])".into(),
            &cons,
        )));
    }
    let take = name == "take_while";
    let cons = |rec: Recur<'_>, rest: &str| {
        let next = rec(&[]);
        Expr::Flat(if take {
            format!("(if {core} then (cons {head} {next}) else (nil {e}))")
        } else {
            format!("(if {core} then {next} else (cons {head} {rest}))")
        })
    };
    let kind = if take { "TakeWhile" } else { "SkipWhile" };
    Ok(some(walk(
        env,
        ctx,
        acc,
        kind,
        r,
        elem,
        &lam,
        &head,
        &[],
        list,
        format!("(nil {e})"),
        &cons,
    )))
}

/// `.filter_map(f)`, `.find_map(f)` and `.flat_map(f)`.
fn comb_filter_map(
    f: &rs::RustExpr,
    name: &str,
    r: &str,
    elem: &Ty,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(Option<(Expr, Ty)>, Acc)> {
    let (lam, acc) = lambda(f, std::slice::from_ref(elem), env, ctx, acc, None)?;
    let inner = match (&lam.ty, name) {
        (Ty::Option(u), "filter_map" | "find_map") | (Ty::List(u), "flat_map") => (**u).clone(),
        _ => return Err(Residue::Form(format!("a .{name} step of another type"))),
    };
    let head = lam.binders[0].clone();
    let core = lam.core.clone();
    let (found, acc) = ctx.fresh(env, acc, "found");
    let u = inner.render();
    let (kind, result, nil) = match name {
        "filter_map" => (
            "FilterMap",
            Ty::List(Box::new(inner.clone())),
            format!("(nil {u})"),
        ),
        "find_map" => (
            "FindMap",
            Ty::Option(Box::new(inner.clone())),
            format!("(ctor Option.none [{u}])"),
        ),
        _ => (
            "FlatMap",
            Ty::List(Box::new(inner.clone())),
            format!("(nil {u})"),
        ),
    };
    let cons = |rec: Recur<'_>, _: &str| {
        let next = rec(&[]);
        match name {
            "flat_map" => Expr::Flat(format!("(prim append {core} {next} : (List {u}))")),
            _ => Expr::Match {
                scrutinee: core.clone(),
                arms: vec![
                    Arm {
                        pattern: "Option.none".into(),
                        body: Expr::Flat(next.clone()),
                    },
                    Arm {
                        pattern: format!("Option.some {found}"),
                        body: Expr::Flat(if name == "find_map" {
                            format!("(ctor Option.some [{u}] {found})")
                        } else {
                            format!("(cons {found} {next})")
                        }),
                    },
                ],
            },
        }
    };
    Ok(some(walk(
        env,
        ctx,
        acc,
        kind,
        r,
        elem,
        &lam,
        &head,
        &[],
        &result,
        nil,
        &cons,
    )))
}

/// A match arm's body: an expression, or the block of an `if let`.
#[derive(Clone, Copy)]
enum Body<'a> {
    Expr(&'a rs::RustExpr),
    Block(&'a rs::Block),
    /// An arm with a guard: the guard, then the arm's expression.
    Guarded(&'a rs::RustExpr, &'a rs::RustExpr),
}

fn body(b: Body<'_>, env: &Env, ctx: &Ctx, acc: Acc, want: Option<&Ty>) -> Lift<((Expr, Ty), Acc)> {
    match b {
        Body::Expr(e) | Body::Guarded(_, e) => expr(e, env, ctx, acc, want),
        Body::Block(b) => block(&b.stmts, env, ctx, acc, want),
    }
}

/// The constructors of a scrutinee type: `(Rust shape key, LexLean
/// constructor, binder types)`.
fn constructors(t: &Ty, types: &Types) -> Option<Vec<(String, String, Vec<Ty>)>> {
    Some(match t {
        Ty::Named(n) if types.datas.contains_key(n) => types.datas[n]
            .iter()
            .map(|(v, fields)| {
                (
                    v.clone(),
                    format!("{n}.{v}"),
                    fields.iter().map(|(_, t)| t.clone()).collect(),
                )
            })
            .collect(),
        Ty::Named(n) => {
            let vs = types.enums.get(n)?;
            vs.iter()
                .map(|v| (v.clone(), format!("{n}.{v}"), Vec::new()))
                .collect()
        }
        Ty::Option(inner) => vec![
            ("None".into(), "Option.none".into(), Vec::new()),
            ("Some".into(), "Option.some".into(), vec![(**inner).clone()]),
        ],
        Ty::Outcome(ok, err) => vec![
            ("Ok".into(), "Result.ok".into(), vec![(**ok).clone()]),
            ("Err".into(), "Result.error".into(), vec![(**err).clone()]),
        ],
        Ty::List(inner) => vec![
            ("[]".into(), "List.nil".into(), Vec::new()),
            (
                "[..]".into(),
                "List.cons".into(),
                vec![(**inner).clone(), t.clone()],
            ),
        ],
        _ => return None,
    })
}

/// A cell of the match compiler: one Rust pattern, a wildcard the compiler
/// fills in, or the elements of a slice pattern after its head.
#[derive(Clone, Copy, PartialEq)]
enum Cell<'a> {
    Pat(&'a rs::Pat),
    Wild,
    Elems(&'a [rs::Pat]),
}

/// What a cell tests of its column, once the names it binds are taken.
enum Head<'a> {
    /// Every value.
    Any,
    /// A constructor, by its Rust key, with a cell for each field.
    Ctor(String, Vec<Cell<'a>>),
    /// A tuple's or a structure's items, in order.
    Items(Vec<Cell<'a>>),
    /// A literal or a range, tested on the value.
    Test(&'a rs::Pat),
    /// Alternatives.
    Or(Vec<Cell<'a>>),
}

/// A clause (a row) of the match compiler: a cell per column, the Rust names its
/// cells bind (with the value and type each names), and its arm.
#[derive(Clone)]
struct Clause<'a> {
    /// The arm the row is of: the alternatives of an or-pattern are rows
    /// of one arm.
    arm: usize,
    cells: Vec<Cell<'a>>,
    binds: Vec<(String, String, Ty)>,
    body: Body<'a>,
}

/// A column of the match compiler: the value it tests, and its type.
#[derive(Clone)]
struct Column {
    value: String,
    ty: Ty,
}

/// The Rust key of a constructor path: `Some`, `Ok`, `Err`, `None`, or the
/// variant's own name.
fn ctor_key(path: &rs::Path) -> String {
    match wrapper(path) {
        Some(w) => w.to_string(),
        None => path
            .segments
            .last()
            .map(|s| s.ident.clone())
            .unwrap_or_default(),
    }
}

/// Whether a slice element is `..` or `name @ ..`.
fn is_rest(p: &rs::Pat) -> bool {
    match p {
        rs::Pat::Rest => true,
        rs::Pat::Ident(i) => matches!(i.subpat.as_deref(), Some(rs::Pat::Rest)),
        _ => false,
    }
}

/// The head of the elements of a slice pattern: `[]`, the rest, or the
/// first element and the elements after it.
fn slice_head(elems: &[rs::Pat]) -> Option<(Vec<String>, Head<'_>)> {
    match elems {
        [] => Some((Vec::new(), Head::Ctor("[]".into(), Vec::new()))),
        [rs::Pat::Rest] => Some((Vec::new(), Head::Any)),
        [rs::Pat::Ident(i)] if is_rest(&elems[0]) => Some((vec![i.ident.clone()], Head::Any)),
        [first, rest @ ..] => {
            if is_rest(first) {
                return None;
            }
            let cells = vec![Cell::Pat(first), Cell::Elems(rest)];
            Some((Vec::new(), Head::Ctor("[..]".into(), cells)))
        }
    }
}

/// The cells of a struct pattern, a field's in the order `names` gives
/// (LexLean names): the pattern given for it, or a wildcard where `..`
/// leaves it out.
fn struct_cells<'a>(ps: &'a rs::PatStruct, names: &[String]) -> Option<Vec<Cell<'a>>> {
    let mut cells = Vec::new();
    for name in names {
        let given = ps
            .fields
            .iter()
            .find(|f| matches!(&f.member, rs::Member::Named(n) if local_name(n) == *name));
        match given {
            Some(f) => cells.push(Cell::Pat(&f.pat)),
            None if ps.rest => cells.push(Cell::Wild),
            None => return None,
        }
    }
    let used = cells.iter().filter(|c| matches!(c, Cell::Pat(_))).count();
    (used == ps.fields.len()).then_some(cells)
}

/// The names a cell binds and what it tests, in a column of type `ty`.
fn head<'a>(cell: Cell<'a>, ty: &Ty, types: &Types) -> Option<(Vec<String>, Head<'a>)> {
    match cell {
        Cell::Wild => Some((Vec::new(), Head::Any)),
        Cell::Elems(elems) => slice_head(elems),
        Cell::Pat(p) => pat_head(p, ty, types),
    }
}

fn pat_head<'a>(p: &'a rs::Pat, ty: &Ty, types: &Types) -> Option<(Vec<String>, Head<'a>)> {
    if binds_cell(p, ty, types) {
        return None;
    }
    match p {
        rs::Pat::Wild | rs::Pat::Rest => Some((Vec::new(), Head::Any)),
        rs::Pat::Reference(inner) => pat_head(inner, ty, types),
        rs::Pat::Type(t) => pat_head(&t.pat, ty, types),
        rs::Pat::Ident(i) => match &i.subpat {
            Some(sub) => {
                let (names, h) = pat_head(sub, ty, types)?;
                let mut all = vec![i.ident.clone()];
                all.extend(names);
                Some((all, h))
            }
            None => {
                let unit = constructors(ty, types).is_some_and(|cs| {
                    cs.iter()
                        .any(|(k, _, fields)| *k == i.ident && fields.is_empty())
                });
                if unit && i.ident.starts_with(char::is_uppercase) {
                    Some((Vec::new(), Head::Ctor(i.ident.clone(), Vec::new())))
                } else {
                    Some((vec![i.ident.clone()], Head::Any))
                }
            }
        },
        rs::Pat::Path(path) => Some((Vec::new(), Head::Ctor(ctor_key(path), Vec::new()))),
        rs::Pat::Lit(rs::Lit::Bool(b)) => Some((Vec::new(), Head::Ctor(b.to_string(), Vec::new()))),
        rs::Pat::Lit(_) | rs::Pat::Range(_) => Some((Vec::new(), Head::Test(p))),
        rs::Pat::Tuple(items) => Some((
            Vec::new(),
            Head::Items(items.iter().map(Cell::Pat).collect()),
        )),
        rs::Pat::Slice(elems) => slice_head(elems),
        rs::Pat::Or(alts) => Some((Vec::new(), Head::Or(alts.iter().map(Cell::Pat).collect()))),
        rs::Pat::TupleStruct(ts) => {
            let key = ctor_key(&ts.path);
            let ctors = constructors(ty, types)?;
            let (_, _, fields) = ctors.iter().find(|(k, _, _)| *k == key)?;
            let arity = fields.len();
            let cells: Vec<Cell<'a>> =
                match ts.elems.iter().position(|e| matches!(e, rs::Pat::Rest)) {
                    None => ts.elems.iter().map(Cell::Pat).collect(),
                    Some(at) => {
                        let before = &ts.elems[..at];
                        let after = &ts.elems[at + 1..];
                        let given = before.len() + after.len();
                        if given > arity {
                            return None;
                        }
                        before
                            .iter()
                            .map(Cell::Pat)
                            .chain(std::iter::repeat_n(Cell::Wild, arity - given))
                            .chain(after.iter().map(Cell::Pat))
                            .collect()
                    }
                };
            (cells.len() == arity).then_some((Vec::new(), Head::Ctor(key, cells)))
        }
        rs::Pat::Struct(ps) => {
            let Ty::Named(n) = ty else { return None };
            if let Some(variants) = types.datas.get(n) {
                let key = ctor_key(&ps.path);
                let (_, fields) = variants.iter().find(|(v, _)| *v == key)?;
                let names: Vec<String> = fields
                    .iter()
                    .map(|(name, _)| name.as_deref().map(local_name).unwrap_or_default())
                    .collect();
                let cells = struct_cells(ps, &names)?;
                return Some((Vec::new(), Head::Ctor(key, cells)));
            }
            let fields = types.structs.get(n)?;
            let names: Vec<String> = fields.iter().map(|(f, _)| f.clone()).collect();
            Some((Vec::new(), Head::Items(struct_cells(ps, &names)?)))
        }
        rs::Pat::Guard(_) | rs::Pat::Other => None,
    }
}

/// The row's heads at column `j`, one row per alternative of an or-pattern,
/// each with the names the cell binds there.
fn heads_at<'a>(
    row: &Clause<'a>,
    j: usize,
    col: &Column,
    types: &Types,
) -> Option<Vec<(Clause<'a>, Head<'a>)>> {
    let (names, h) = head(row.cells[j], &col.ty, types)?;
    let mut row = row.clone();
    row.binds.extend(
        names
            .into_iter()
            .map(|n| (n, col.value.clone(), col.ty.clone())),
    );
    match h {
        Head::Or(alts) => {
            let mut out = Vec::new();
            for alt in alts {
                let mut one = row.clone();
                one.cells[j] = alt;
                out.extend(heads_at(&one, j, col, types)?);
            }
            Some(out)
        }
        h => Some(vec![(row, h)]),
    }
}

/// The test that a value `s` of a literal type `st` matches a literal or
/// range pattern, and the state with the axioms the test elaborates with.
fn literal_test(s: &str, st: &Ty, p: &rs::Pat, acc: Acc) -> Option<(String, Acc)> {
    let constant = |e: &rs::RustExpr| -> Option<String> {
        match e {
            rs::RustExpr::Lit(l) => match literal_of(l, Some(st), false) {
                Ok((Expr::Flat(k), kt)) if kt.same(st) => Some(k),
                _ => None,
            },
            rs::RustExpr::Unary(u) if matches!(u.op, rs::UnOp::Neg) => match &*u.expr {
                rs::RustExpr::Lit(l) => match literal_of(l, Some(st), true) {
                    Ok((Expr::Flat(k), kt)) if kt.same(st) => Some(k),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    };
    let at_most = |a: &str, b: &str| -> Option<String> {
        match st {
            Ty::Nat(_) => Some(format!("({a} <=? {b})")),
            Ty::Int(_) | Ty::Str | Ty::Char => {
                Some(format!("(not (prim less_than {b} {a} : Bool))"))
            }
            _ => None,
        }
    };
    // The axioms an order test elaborates with.
    let ordered = |acc: Acc| match st {
        Ty::Int(_) => acc.axiom("propext"),
        Ty::Str | Ty::Char => acc.all_axioms(),
        _ => acc,
    };
    match p {
        rs::Pat::Lit(l) => {
            let k = constant(&rs::RustExpr::Lit(l.clone()))?;
            let test = match st {
                Ty::Nat(_) => format!("({s} =? {k})"),
                Ty::Int(_) | Ty::Str | Ty::Char => format!("(prim equal {s} {k} : Bool)"),
                _ => return None,
            };
            Some((test, acc))
        }
        rs::Pat::Range(rg) => {
            let mut acc = acc;
            let mut parts = Vec::new();
            if let Some(e) = &rg.start {
                let lo = constant(e)?;
                parts.push(at_most(&lo, s)?);
                acc = ordered(acc);
            }
            if let Some(e) = &rg.end {
                let hi = constant(e)?;
                let c = if rg.closed {
                    at_most(s, &hi)?
                } else {
                    format!("(not {})", at_most(&hi, s)?)
                };
                parts.push(c);
                acc = ordered(acc);
            }
            match parts.as_slice() {
                [one] => Some((one.clone(), acc)),
                [a, b] => Some((format!("({a} && {b})"), acc)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// A match: its scrutinee once, then its arms compiled to LexLean matches
/// on one constructor at a time and `if` tests on literals and guards.
fn matching<'a>(
    scrutinee: &rs::RustExpr,
    arms: &[(&'a rs::Pat, Body<'a>)],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    // `xs.split_first()`: a match on the slice itself, so a recursive call
    // on its rest descends.
    if let Some((slice, pats)) = split_first(scrutinee, arms) {
        let arms: Vec<(&rs::Pat, Body<'_>)> =
            pats.iter().zip(arms).map(|(p, (_, b))| (p, *b)).collect();
        return matching(&slice, &arms, env, ctx, acc, want);
    }
    // A tuple written in place is its items, a column each.
    let parts: Vec<&rs::RustExpr> = match scrutinee {
        rs::RustExpr::Tuple(items) if items.len() >= 2 => items.iter().collect(),
        one => vec![one],
    };
    let mut cols = Vec::new();
    let mut acc = acc;
    for part in &parts {
        let ((value, ty), after) = flat_of(part, env, &ctx.below(), acc, None)?;
        acc = after;
        cols.push(Column { value, ty });
    }
    let spread = |p: &'a rs::Pat| -> Vec<Cell<'a>> {
        match p {
            rs::Pat::Tuple(items) if parts.len() > 1 && items.len() == parts.len() => {
                items.iter().map(Cell::Pat).collect()
            }
            _ if parts.len() > 1 && matches!(p, rs::Pat::Wild) => vec![Cell::Wild; parts.len()],
            _ => vec![Cell::Pat(p)],
        }
    };
    // An arm of alternatives over the columns is an arm per alternative,
    // each with the arm's body, tried in order as Rust tries them.
    let arms: Vec<(&'a rs::Pat, Body<'a>)> = arms
        .iter()
        .flat_map(|(p, b)| match p {
            rs::Pat::Or(alts) if parts.len() > 1 => alts.iter().map(|a| (a, *b)).collect(),
            _ => vec![(*p, *b)],
        })
        .collect();
    let rows: Option<Vec<Clause<'a>>> = arms
        .iter()
        .enumerate()
        .map(|(arm, (p, b))| {
            let cells = spread(p);
            (cells.len() == cols.len()).then_some(Clause {
                arm,
                cells,
                binds: Vec::new(),
                body: *b,
            })
        })
        .collect();
    let Some(rows) = rows else {
        return Err(Residue::Form("match pattern".to_string()));
    };
    compile(&cols, &rows, env, ctx, acc, want)
}

/// A match on `xs.split_first()`, alone or as an item of a tuple, as the
/// match on `xs` itself: `None` is `[]`, and `Some((head, rest))` is
/// `[head, rest @ ..]`.
fn split_first(
    scrutinee: &rs::RustExpr,
    arms: &[(&rs::Pat, Body<'_>)],
) -> Option<(rs::RustExpr, Vec<rs::Pat>)> {
    let is_split = |e: &rs::RustExpr| matches!(e, rs::RustExpr::MethodCall(m) if m.method == "split_first" && m.args.is_empty());
    let receiver = |e: &rs::RustExpr| match e {
        rs::RustExpr::MethodCall(m) => (*m.receiver).clone(),
        other => other.clone(),
    };
    let (slice, at) = match scrutinee {
        rs::RustExpr::Tuple(items) if items.len() >= 2 => {
            let k = items.iter().position(is_split)?;
            let mut items = items.clone();
            items[k] = receiver(&items[k]);
            (rs::RustExpr::Tuple(items), Some(k))
        }
        one if is_split(one) => (receiver(one), None),
        _ => return None,
    };
    let pats = arms
        .iter()
        .map(|(p, _)| match (at, p) {
            (None, p) => sliced(p),
            (Some(_), rs::Pat::Wild) => Some(rs::Pat::Wild),
            (Some(k), rs::Pat::Tuple(items)) => {
                let mut items = items.clone();
                items[k] = sliced(items.get(k)?)?;
                Some(rs::Pat::Tuple(items))
            }
            _ => None,
        })
        .collect::<Option<Vec<rs::Pat>>>()?;
    Some((slice, pats))
}

/// A pattern of `split_first`'s value as a pattern of the slice.
fn sliced(p: &rs::Pat) -> Option<rs::Pat> {
    match p {
        rs::Pat::Wild => Some(rs::Pat::Wild),
        rs::Pat::Path(path) if wrapper(path) == Some("None") => Some(rs::Pat::Slice(Vec::new())),
        rs::Pat::Ident(i) if i.ident == "None" && i.subpat.is_none() => {
            Some(rs::Pat::Slice(Vec::new()))
        }
        rs::Pat::TupleStruct(ts) if wrapper(&ts.path) == Some("Some") => {
            let [rs::Pat::Tuple(pair)] = ts.elems.as_slice() else {
                return None;
            };
            let [head, rest] = pair.as_slice() else {
                return None;
            };
            let mut elems = vec![head.clone()];
            match rest {
                rs::Pat::Wild => elems.push(rs::Pat::Rest),
                rs::Pat::Ident(i) if i.subpat.is_none() => {
                    elems.push(rs::Pat::Ident(rs::PatIdent {
                        subpat: Some(Box::new(rs::Pat::Rest)),
                        ..i.clone()
                    }))
                }
                rs::Pat::Slice(more) => elems.extend(more.iter().cloned()),
                _ => return None,
            }
            Some(rs::Pat::Slice(elems))
        }
        _ => None,
    }
}

/// The row's cells with the cells at column `j` in place of its cell there.
fn replaced<'a>(row: &Clause<'a>, j: usize, cells: Vec<Cell<'a>>) -> Vec<Cell<'a>> {
    let mut out = row.cells[..j].to_vec();
    out.extend(cells);
    out.extend(row.cells[j + 1..].iter().copied());
    out
}

/// The rows of a match over the columns, as one expression: the first row
/// whose cells all match, and whose guard holds, gives the value.
fn compile<'a>(
    cols: &[Column],
    rows: &[Clause<'a>],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let Some((first, rest)) = rows.split_first() else {
        return Err(Residue::Form("non-exhaustive match".to_string()));
    };
    // The first column the first row tests, and the names it binds before.
    let mut binds = first.binds.clone();
    let mut tested = None;
    for (j, (cell, col)) in first.cells.iter().zip(cols).enumerate() {
        let Some((names, h)) = head(*cell, &col.ty, &env.types) else {
            return refused();
        };
        if matches!(h, Head::Any) {
            binds.extend(
                names
                    .into_iter()
                    .map(|n| (n, col.value.clone(), col.ty.clone())),
            );
        } else {
            tested = Some(j);
            break;
        }
    }
    let Some(j) = tested else {
        let matched = Matched {
            cols,
            first,
            rest,
            binds,
        };
        return matched_row(&matched, env, ctx, acc, want);
    };
    let col = &cols[j];
    let mut heads = Vec::new();
    for row in rows {
        let Some(hs) = heads_at(row, j, col, &env.types) else {
            return refused();
        };
        heads.extend(hs);
    }
    let split = Split { cols, j, heads };
    if let Some(items) = item_columns(col, env) {
        return compile_items(&split, items, env, ctx, acc, want);
    }
    if matches!(split.heads.first(), Some((_, Head::Test(_)))) {
        return compile_tests(&split, env, ctx, acc, want);
    }
    if col.ty == Ty::Bool {
        return compile_bool(&split, env, ctx, acc, want);
    }
    compile_ctors(&split, env, ctx, acc, want)
}

/// A match pattern outside the compiler.
fn refused<T>() -> Lift<T> {
    Err(Residue::Form("match pattern".to_string()))
}

/// A first row that matches every value, with the names its cells bind,
/// and the rows after it.
struct Matched<'c, 'a> {
    cols: &'c [Column],
    first: &'c Clause<'a>,
    rest: &'c [Clause<'a>],
    binds: Vec<(String, String, Ty)>,
}

/// The first row matches every value: its arm, when its guard holds. A
/// name bound to a part of the scrutinee is a binder of its own, so what
/// the arm lifts to names it.
fn matched_row(
    m: &Matched<'_, '_>,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let mut scope = ctx.clone();
    let mut lets = Vec::new();
    let mut acc = acc;
    for (rust, value, ty) in m.binds.iter().cloned() {
        let value = ctx.rebuilt.get(&value).cloned().unwrap_or(value);
        if value.chars().all(|c| c.is_alphanumeric() || c == '_') {
            if let Some((old, _)) = scope.locals.insert(rust, (value, ty)) {
                scope.hidden.push(old);
            }
        } else {
            let (next, name, after) = scope.bind(env, acc, &rust, ty.clone());
            scope = next;
            acc = after;
            lets.push((name, ty.render(), value));
        }
    }
    let Body::Guarded(g, e) = m.first.body else {
        let ((e, t), acc) = body(m.first.body, env, &scope, acc, want)?;
        return Ok(((wrap_lets(lets, e), t), acc));
    };
    let ((gv, gt), acc) = flat_of(g, env, &scope.below(), acc, Some(&Ty::Bool))?;
    if gt != Ty::Bool {
        return Err(Residue::Form("non-boolean guard".to_string()));
    }
    let ((then, tt), acc) = expr(e, env, &scope, acc, want)?;
    // The rows after it are written inside this row's binders, which wrap
    // the whole conditional: none of them may name one of these again.
    let mut after = ctx.clone();
    after
        .hidden
        .extend(lets.iter().map(|(name, _, _)| name.clone()));
    let ((other, ot), acc) = compile(m.cols, m.rest, env, &after, acc, Some(want.unwrap_or(&tt)))?;
    if !tt.same(&ot) {
        return Err(Residue::Form("arms of different types".to_string()));
    }
    let e = Expr::If {
        condition: Box::new(Expr::Flat(gv)),
        then: Box::new(then),
        otherwise: Box::new(other),
    };
    Ok(((wrap_lets(lets, e), tt), acc))
}

/// The rows of a match split at the column `j` they first test, each with
/// what its cell there tests.
struct Split<'c, 'a> {
    cols: &'c [Column],
    j: usize,
    heads: Vec<(Clause<'a>, Head<'a>)>,
}

impl<'a> Split<'_, 'a> {
    /// The columns with `middle` in place of the split one.
    fn around(&self, middle: Vec<Column>) -> Vec<Column> {
        let mut out = self.cols[..self.j].to_vec();
        out.extend(middle);
        out.extend(self.cols[self.j + 1..].iter().cloned());
        out
    }

    /// The rows a constructor's values reach, its fields' cells in place
    /// of the column's.
    fn reach(&self, key: &str, arity: usize) -> Option<Vec<Clause<'a>>> {
        let mut out = Vec::new();
        for (row, h) in &self.heads {
            let cells = match h {
                Head::Any => vec![Cell::Wild; arity],
                Head::Ctor(k, cs) if k == key && cs.len() == arity => cs.clone(),
                Head::Ctor(k, _) if k != key => continue,
                _ => return None,
            };
            out.push(Clause {
                cells: replaced(row, self.j, cells),
                ..row.clone()
            });
        }
        Some(out)
    }
}

/// The columns of a tuple's or a structure's items, which a column of
/// that type splits into.
fn item_columns(col: &Column, env: &Env) -> Option<Vec<Column>> {
    match &col.ty {
        Ty::Tuple(ts) | Ty::Pair(ts) => Some(
            ts.iter()
                .enumerate()
                .map(|(k, t)| Column {
                    value: tuple_item(&col.ty, &col.value, k),
                    ty: t.clone(),
                })
                .collect(),
        ),
        Ty::Named(n) if env.types.structs.contains_key(n) => Some(
            env.types.structs[n]
                .iter()
                .map(|(f, t)| Column {
                    value: if env.types.boxed(n) {
                        format!("({} {})", Types::accessor(n, f), col.value)
                    } else {
                        format!("(project {} {f})", col.value)
                    },
                    ty: t.clone(),
                })
                .collect(),
        ),
        _ => None,
    }
}

/// A tuple or a structure: a column for each item.
fn compile_items(
    split: &Split<'_, '_>,
    items: Vec<Column>,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let mut next = Vec::new();
    for (row, h) in &split.heads {
        let cells = match h {
            Head::Any => vec![Cell::Wild; items.len()],
            Head::Items(cs) if cs.len() == items.len() => cs.clone(),
            _ => return refused(),
        };
        next.push(Clause {
            cells: replaced(row, split.j, cells),
            ..row.clone()
        });
    }
    compile(&split.around(items), &next, env, ctx, acc, want)
}

/// A literal or a range: a test, then the rows after it.
fn compile_tests(
    split: &Split<'_, '_>,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let (cols, j, heads) = (split.cols, split.j, &split.heads);
    let col = &cols[j];
    let Some((row, _)) = heads.first() else {
        return refused();
    };
    // The alternatives of the first arm that test this column and agree
    // elsewhere: one test, their disjunction.
    let mut tests = Vec::new();
    let mut acc = acc;
    let mut taken = 0;
    for (other, h) in heads {
        let Head::Test(p) = h else { break };
        let alike = other.arm == row.arm
            && other.binds == row.binds
            && (0..cols.len()).all(|i| i == j || other.cells[i] == row.cells[i]);
        if !alike {
            break;
        }
        let Some((test, after)) = literal_test(&col.value, &col.ty, p, acc) else {
            return refused();
        };
        acc = after;
        tests.push(test);
        taken += 1;
    }
    let Some(test) = tests.into_iter().reduce(|a, b| format!("({a} || {b})")) else {
        return refused();
    };
    let mut matched = row.clone();
    matched.cells[j] = Cell::Wild;
    let after: Vec<Clause<'_>> = heads[taken..].iter().map(|(r, _)| r.clone()).collect();
    let then_rows: Vec<Clause<'_>> = std::iter::once(matched)
        .chain(after.iter().cloned())
        .collect();
    let ((then, tt), acc) = compile(cols, &then_rows, env, ctx, acc, want)?;
    let ((other, ot), acc) = compile(cols, &after, env, ctx, acc, Some(want.unwrap_or(&tt)))?;
    if !tt.same(&ot) {
        return Err(Residue::Form("arms of different types".to_string()));
    }
    let e = Expr::If {
        condition: Box::new(Expr::Flat(test)),
        then: Box::new(then),
        otherwise: Box::new(other),
    };
    Ok(((e, tt), acc))
}

/// A `bool` column: an `if` on it.
fn compile_bool(
    split: &Split<'_, '_>,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let (Some(yes), Some(no)) = (split.reach("true", 0), split.reach("false", 0)) else {
        return refused();
    };
    let ((then, tt), acc) = compile(&split.around(Vec::new()), &yes, env, ctx, acc, want)?;
    let want = Some(want.unwrap_or(&tt));
    let ((other, ot), acc) = compile(&split.around(Vec::new()), &no, env, ctx, acc, want)?;
    if !tt.same(&ot) {
        return Err(Residue::Form("arms of different types".to_string()));
    }
    let e = Expr::If {
        condition: Box::new(Expr::Flat(split.cols[split.j].value.clone())),
        then: Box::new(then),
        otherwise: Box::new(other),
    };
    Ok(((e, tt), acc))
}

/// An arm of a constructor match still to lift: where it goes, its
/// columns, its rows and its scope.
type Pending<'a> = (usize, Vec<Column>, Vec<Clause<'a>>, Ctx);

/// A column of a data type: a match with an arm for each constructor.
fn compile_ctors(
    split: &Split<'_, '_>,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let col = &split.cols[split.j];
    let Some(ctors) = constructors(&col.ty, &env.types) else {
        return Err(Residue::Form(format!("match on {}", col.ty.render())));
    };
    let mut out = Vec::new();
    let mut result: Option<Ty> = want.cloned();
    let mut deferred: Vec<Pending<'_>> = Vec::new();
    let mut acc = acc;
    for (key, lex, field_tys) in &ctors {
        let Some(next) = split.reach(key, field_tys.len()) else {
            return refused();
        };
        let (scope, names, after) = ctor_scope(split, &next, lex, field_tys, env, ctx, acc);
        acc = after;
        let binders: Vec<String> = names.iter().map(|c| c.value.clone()).collect();
        let pattern = std::iter::once(lex.clone())
            .chain(binders)
            .collect::<Vec<_>>()
            .join(" ");
        let cols = split.around(names);
        let ((e, t), after) = match compile(&cols, &next, env, &scope, acc.clone(), result.as_ref())
        {
            // An arm whose type only a sibling says waits for the others.
            Err(r) if result.is_none() && untyped(&r) => {
                deferred.push((out.len(), cols, next, scope));
                out.push(Arm {
                    pattern,
                    body: Expr::Flat(String::new()),
                });
                continue;
            }
            lifted => lifted?,
        };
        acc = after;
        match &result {
            Some(r) if !r.same(&t) => {
                return Err(Residue::Form("arms of different types".to_string()));
            }
            _ => result = Some(t),
        }
        out.push(Arm { pattern, body: e });
    }
    let Some(result) = result else {
        return Err(Residue::Form("empty match".to_string()));
    };
    for (at, cols, next, scope) in deferred {
        let ((e, t), after) = compile(&cols, &next, env, &scope, acc, Some(&result))?;
        acc = after;
        if !result.same(&t) {
            return Err(Residue::Form("arms of different types".to_string()));
        }
        out[at].body = e;
    }
    let e = Expr::Match {
        scrutinee: col.value.clone(),
        arms: out,
    };
    Ok(((e, result), acc))
}

/// The scope of a constructor's arm and its fields' columns: each field's
/// binder is the name the first row reaching it gives it, or a fresh one.
fn ctor_scope(
    split: &Split<'_, '_>,
    next: &[Clause<'_>],
    lex: &str,
    field_tys: &[Ty],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> (Ctx, Vec<Column>, Acc) {
    let (col, j) = (&split.cols[split.j], split.j);
    let mut acc = acc;
    let mut scope = ctx.clone();
    let mut names = Vec::new();
    for (i, ty) in field_tys.iter().enumerate() {
        let given = next.iter().find_map(|row| match row.cells[j + i] {
            Cell::Pat(rs::Pat::Ident(id)) if id.subpat.is_none() => Some(local_name(&id.ident)),
            _ => None,
        });
        let base = given.unwrap_or_else(|| {
            if i == 0 {
                "value".into()
            } else {
                "rest".into()
            }
        });
        let (name, after) = if scope.taken(env, &base) {
            scope.fresh(env, acc, &base)
        } else {
            (base, acc)
        };
        acc = after;
        scope.hidden.push(name.clone());
        names.push(Column {
            value: name,
            ty: ty.clone(),
        });
    }
    if ctx.params.contains(&col.value) {
        // The tail of a list parameter, and a field of a data parameter's
        // own type, are sub-terms: a recursive call on one descends.
        for (i, field) in names.iter().enumerate() {
            let tail = lex == "List.cons" && i == 1;
            let data = types_has_data(&col.ty, &env.types);
            let list = Ty::List(Box::new(col.ty.clone()));
            if tail || (data && (field.ty.same(&col.ty) || field.ty.same(&list))) {
                scope.tails.insert(field.value.clone(), col.value.clone());
            }
        }
        // A structural match names the parameter only through its
        // parts: in the arm, the parameter is its constructor applied
        // to them.
        if types_has_data(&col.ty, &env.types) {
            let parts: String = names.iter().map(|f| format!(" {}", f.value)).collect();
            let whole = format!("(ctor {lex}{parts})");
            let rust: Vec<String> = ctx
                .locals
                .iter()
                .filter(|(_, (n, _))| *n == col.value)
                .map(|(r, _)| r.clone())
                .collect();
            for r in rust {
                scope.locals.insert(r, (whole.clone(), col.ty.clone()));
            }
            scope.rebuilt.insert(col.value.clone(), whole);
        }
    }
    (scope, names, acc)
}

/// A residue that says only that an expression's type was not known where
/// it was lifted: an empty collection or a `None` no sibling had typed yet.
fn untyped(r: &Residue) -> bool {
    match r {
        Residue::Call(n) => n == "new",
        Residue::Form(f) => f == "None of an unknown type",
        _ => false,
    }
}

/// A branch followed by the statements after it, as one block expression.
fn then_rest(body: &rs::RustExpr, rest: &[rs::Stmt]) -> rs::RustExpr {
    let mut stmts = match body {
        rs::RustExpr::Block(b) if !b.label => b.block.stmts.clone(),
        rs::RustExpr::Tuple(t) if t.is_empty() => Vec::new(),
        other => vec![rs::Stmt::Expr(other.clone(), true)],
    };
    stmts.extend(rest.iter().cloned());
    rs::RustExpr::Block(rs::ExprBlock {
        label: false,
        block: rs::Block { stmts },
    })
}

/// The `return v` a block is made of, alone: `{ return v; }`.
fn returned(b: &rs::Block) -> Option<&rs::RustExpr> {
    match b.stmts.as_slice() {
        [rs::Stmt::Expr(rs::RustExpr::Return(r), _)] => r.as_deref(),
        _ => None,
    }
}

/// The local a statement updates in place, and how: `x = e`, `x += e` on a
/// string, `x.push(e)` on a list or string, `x.push_str(e)`.
fn update_target(e: &rs::RustExpr) -> Option<String> {
    let ident = |e: &rs::RustExpr| match e {
        rs::RustExpr::Path(p) => p.get_ident().cloned(),
        _ => None,
    };
    match e {
        // `x.f = v`: the structure `x` with its field `f` replaced.
        rs::RustExpr::Assign(a) => match &*a.left {
            rs::RustExpr::Field(f) if matches!(f.member, rs::Member::Named(_)) => ident(&f.base),
            // `*x = v` through a `&mut` the function threads: its new value.
            rs::RustExpr::Unary(u) if matches!(u.op, rs::UnOp::Deref) => ident(&u.expr),
            left => ident(left),
        },
        // `*m.entry(k).or_default() += v` and `m.entry(k).or_default().push(v)`
        // update the map.
        rs::RustExpr::Binary(b) if b.op == rs::BinOp::AddAssign => match &*b.left {
            rs::RustExpr::Unary(u) if matches!(u.op, rs::UnOp::Deref) => {
                ident(default_entry(&u.expr)?.0)
            }
            // `x.f += v`: the structure `x` with its field `f` added to.
            rs::RustExpr::Field(f) if matches!(f.member, rs::Member::Named(_)) => ident(&f.base),
            left => ident(left),
        },
        rs::RustExpr::MethodCall(m) if default_entry(&m.receiver).is_some() => {
            ident(default_entry(&m.receiver)?.0)
        }
        rs::RustExpr::MethodCall(m)
            if matches!(m.method.as_str(), "push" | "push_str" | "extend") && m.args.len() == 1 =>
        {
            ident(&m.receiver).or_else(|| ident(&named_field(&m.receiver)?.base))
        }
        // The value of `insert` (whether it was new, or the replaced value)
        // is dropped where it is a statement of a loop.
        rs::RustExpr::MethodCall(m) if m.method == "insert" && (1..=2).contains(&m.args.len()) => {
            ident(&m.receiver).or_else(|| ident(&named_field(&m.receiver)?.base))
        }
        _ => None,
    }
}

/// `x.f`: a named field of a value.
fn named_field(e: &rs::RustExpr) -> Option<&rs::ExprField> {
    match e {
        rs::RustExpr::Field(f) if matches!(f.member, rs::Member::Named(_)) => Some(f),
        _ => None,
    }
}

/// A character literal as the one-character string it pushes.
fn char_text(e: &rs::RustExpr) -> Option<String> {
    match e {
        rs::RustExpr::Lit(rs::Lit::Char(c)) => exact_string(&c.to_string()),
        _ => None,
    }
}

/// The new value of the local an update statement changes.
fn updated(
    e: &rs::RustExpr,
    ty: &Ty,
    current: &str,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<(String, Acc)> {
    let inner = ctx.below();
    let joined = |piece: String, acc: Acc| {
        let acc = acc.axiom("propext");
        Ok((
            format!("(prim join (cons {current} (cons {piece} (nil String))) \"\" : String)"),
            acc,
        ))
    };
    let value_of = |v: &rs::RustExpr, want: &Ty, acc: Acc| -> Lift<(String, Acc)> {
        let ((x, xt), acc) = flat_of(v, env, &inner, acc, Some(want))?;
        if xt.same(want) {
            Ok((x, acc))
        } else {
            Err(Residue::Form("an update of another type".to_string()))
        }
    };
    match (e, ty) {
        // `x.f.push(v)`: the structure `x` with its field `f` updated.
        (rs::RustExpr::MethodCall(m), Ty::Named(name)) if named_field(&m.receiver).is_some() => {
            let Some(f) = named_field(&m.receiver) else {
                return Err(Residue::Form("an update outside the lift".to_string()));
            };
            let rs::Member::Named(member) = &f.member else {
                return Err(Residue::Form("an update outside the lift".to_string()));
            };
            let Some(fields) = env.types.structs.get(name) else {
                return Err(Residue::Form(format!("a field update of {name}")));
            };
            let target = local_name(member);
            let Some((_, fty)) = fields.iter().find(|(f, _)| *f == target) else {
                return Err(Residue::Form(format!("a field update of {name}")));
            };
            if let Some(kind) = env.types.cell(name, &target) {
                return Err(cell_unread(kind, name, &target, Some(&m.method)));
            }
            let read = |field: &String| {
                if env.types.boxed(name) {
                    format!("({} {current})", Types::accessor(name, field))
                } else {
                    format!("(project {current} {field})")
                }
            };
            let call = rs::RustExpr::MethodCall(rs::ExprMethodCall {
                receiver: f.base.clone(),
                ..m.clone()
            });
            let (v, acc) = updated(&call, fty, &read(&target), env, ctx, acc)?;
            let parts: Vec<(&String, String)> = fields
                .iter()
                .map(|(field, _)| {
                    let value = if *field == target {
                        v.clone()
                    } else {
                        read(field)
                    };
                    (field, value)
                })
                .collect();
            Ok((struct_value(name, &parts, &env.types), acc))
        }
        (rs::RustExpr::Assign(a), Ty::Named(name))
            if matches!(&*a.left, rs::RustExpr::Field(_)) =>
        {
            let rs::RustExpr::Field(f) = &*a.left else {
                return Err(Residue::Form("an update outside the lift".to_string()));
            };
            let rs::Member::Named(member) = &f.member else {
                return Err(Residue::Form("an update outside the lift".to_string()));
            };
            let Some(fields) = env.types.structs.get(name) else {
                return Err(Residue::Form(format!("a field update of {name}")));
            };
            let target = local_name(member);
            if let Some(kind) = env.types.cell(name, &target) {
                return Err(cell_unread(kind, name, &target, None));
            }
            let mut acc = acc;
            let mut parts = Vec::new();
            for (field, fty) in fields {
                let v = if *field == target {
                    let (v, after) = value_of(&a.right, fty, acc)?;
                    acc = after;
                    v
                } else if env.types.boxed(name) {
                    format!("({} {current})", Types::accessor(name, field))
                } else {
                    format!("(project {current} {field})")
                };
                parts.push((field, v));
            }
            if !fields.iter().any(|(f, _)| *f == target) {
                return Err(Residue::Form(format!("a field update of {name}")));
            }
            Ok((struct_value(name, &parts, &env.types), acc))
        }
        (rs::RustExpr::Assign(a), _) => value_of(&a.right, ty, acc),
        // `xs.extend(o)` of an option: the one value it holds, if any.
        (rs::RustExpr::MethodCall(m), Ty::List(t) | Ty::Set(t, true))
            if m.method == "extend" && extends_by_option(&m.args[0], t, env, &inner, &acc) =>
        {
            let ((v, _), acc) = flat_of(&m.args[0], env, &inner, acc, None)?;
            let (x, acc) = ctx.fresh(env, acc, "item");
            let (added, acc) = match ty {
                Ty::Set(..) => {
                    let (insert, acc) = sorted_insert(ty, env, ctx, acc)?;
                    (format!("({insert} {current} {x})"), acc)
                }
                _ => (
                    format!(
                        "(prim append {current} (cons {x} (nil {})) : {})",
                        t.render(),
                        ty.render()
                    ),
                    acc,
                ),
            };
            Ok((
                format!("(match {v} | Option.none => {current} | Option.some {x} => {added})"),
                acc,
            ))
        }
        // `xs.extend(ys)`: the list followed by the items.
        (rs::RustExpr::MethodCall(m), Ty::List(_)) if m.method == "extend" => {
            let (v, acc) = value_of(&m.args[0], ty, acc)?;
            Ok((
                format!("(prim append {current} {v} : {})", ty.render()),
                acc,
            ))
        }
        // `set.extend(items)`: each item inserted in turn.
        (rs::RustExpr::MethodCall(m), Ty::Set(t, true)) if m.method == "extend" => {
            let items = Ty::List(t.clone());
            let ((v, vt), acc) = flat_of(&m.args[0], env, &inner, acc, Some(ty))?;
            let held = matches!(&vt, Ty::List(e) | Ty::Set(e, true) if e.same(t));
            if !held {
                return Err(Residue::Form("an update of another type".to_string()));
            }
            let (insert, acc) = sorted_insert(ty, env, ctx, acc)?;
            let (helper, acc) = insert_all(ty, &items, &insert, env, ctx, acc);
            Ok((format!("({helper} {v} {current})"), acc))
        }
        // `x.f += v`: the field's new value, as a local's would be.
        (rs::RustExpr::Binary(b), Ty::Named(name)) if named_field(&b.left).is_some() => {
            let Some(rs::Member::Named(member)) = named_field(&b.left).map(|f| &f.member) else {
                return Err(Residue::Form("an update outside the lift".to_string()));
            };
            if let Some(kind) = env.types.cell(name, &local_name(member)) {
                return Err(cell_unread(kind, name, &local_name(member), None));
            }
            let place = Place {
                local: String::new(),
                field: Some(member.clone()),
            };
            let Some((fty, read)) = place.field_of(name, current, env) else {
                return Err(Residue::Form(format!("a field update of {name}")));
            };
            if matches!(fty, Ty::Named(_)) {
                return Err(Residue::Form(format!(
                    "an operator update of a field of {name} of type {}",
                    fty.render()
                )));
            }
            let (v, acc) = updated(e, &fty, &read, env, ctx, acc)?;
            Ok((place.replaced(name, current, &v, env)?, acc))
        }
        (rs::RustExpr::Binary(b), Ty::Str) => {
            let (piece, acc) = value_of(&b.right, &Ty::Str, acc)?;
            joined(piece, acc)
        }
        // `i += k` on `usize`: the sum, where it stays in range.
        (rs::RustExpr::Binary(b), Ty::Nat(None)) if b.op == rs::BinOp::AddAssign => {
            let (k, acc) = value_of(&b.right, ty, acc)?;
            let sum = usize_sum(&b.left, &b.right, current, &k, env, &ctx.capped)?;
            Ok((sum, acc))
        }
        (rs::RustExpr::Binary(b), _) => Err(Residue::Arithmetic(b.op.text().to_string())),
        (rs::RustExpr::MethodCall(m), Ty::Set(t, true))
            if m.method == "insert" && m.args.len() == 1 =>
        {
            let (v, acc) = value_of(&m.args[0], t, acc)?;
            let (helper, acc) = sorted_insert(ty, env, ctx, acc)?;
            Ok((format!("({helper} {current} {v})"), acc))
        }
        (rs::RustExpr::MethodCall(m), Ty::Map(k, v, true))
            if m.method == "insert" && m.args.len() == 2 =>
        {
            let (key, acc) = value_of(&m.args[0], k, acc)?;
            let (value, acc) = value_of(&m.args[1], v, acc)?;
            let (helper, acc) = sorted_insert(ty, env, ctx, acc)?;
            let entry = ty.entry();
            Ok((
                format!(
                    "({helper} {current} {})",
                    tuple_value(&entry, &[key, value])
                ),
                acc,
            ))
        }
        (rs::RustExpr::MethodCall(m), Ty::List(t)) if m.method == "push" => {
            let (v, acc) = value_of(&m.args[0], t, acc)?;
            Ok((
                format!(
                    "(prim append {current} (cons {v} (nil {})) : {})",
                    t.render(),
                    ty.render()
                ),
                acc,
            ))
        }
        (rs::RustExpr::MethodCall(m), Ty::Str) => {
            let (piece, acc) = match char_text(&m.args[0]) {
                Some(text) if m.method == "push" => (text, acc),
                _ if m.method == "push_str" => value_of(&m.args[0], &Ty::Str, acc)?,
                _ => value_of(&m.args[0], &Ty::Char, acc)?,
            };
            joined(piece, acc)
        }
        _ => Err(Residue::Form("an update outside the lift".to_string())),
    }
}

/// Whether `e` is an option of the element type `t`: what `extend` adds
/// at most one of.
fn extends_by_option(e: &rs::RustExpr, t: &Ty, env: &Env, ctx: &Ctx, acc: &Acc) -> bool {
    match flat_of(e, env, ctx, acc.clone(), None) {
        Ok(((_, Ty::Option(inner)), _)) => inner.same(t),
        _ => false,
    }
}

/// The helper inserting into the sorted list an ordered set or map is held
/// as, keeping it sorted by key with each key once: a set ignores a member
/// it holds, a map replaces the value of a key it holds. `BTreeSet` and
/// `BTreeMap` order `String` keys by their bytes, which is the order of
/// their scalars, and naturals as numbers; other keys' order is not modelled.
fn sorted_insert(ty: &Ty, env: &Env, ctx: &Ctx, acc: Acc) -> Lift<(String, Acc)> {
    let (key, entry) = match ty {
        Ty::Set(t, true) => ((**t).clone(), (**t).clone()),
        Ty::Map(k, _, true) => ((**k).clone(), ty.entry()),
        _ => return Err(Residue::Form("insert into an unordered collection".into())),
    };
    let list = Ty::List(Box::new(entry.clone()));
    let (items, acc) = ctx.fresh(env, acc, "items");
    let (item, acc) = ctx.fresh(env, acc, "item");
    let (head, acc) = ctx.fresh(env, acc, "head");
    let (rest, acc) = ctx.fresh(env, acc, "rest");
    let key_of = |v: &str| match ty {
        Ty::Map(..) => tuple_item(&entry, v, 0),
        _ => v.to_string(),
    };
    let (k, h) = (key_of(&item), key_of(&head));
    let mut axioms = BTreeSet::new();
    let (same, less) = match &key {
        Ty::Nat(_) => (format!("({k} =? {h})"), format!("({k} <? {h})")),
        Ty::Str | Ty::Char => {
            axioms.insert("propext".to_string());
            axioms.extend(ALL_AXIOMS.map(String::from));
            (
                format!("(prim equal {k} {h} : Bool)"),
                format!("(prim less_than {k} {h} : Bool)"),
            )
        }
        other if crate::order::derived(other, &env.types) => {
            axioms.extend(crate::order::axioms(other, &env.types));
            (
                crate::order::operator("==", other, &k, &h).unwrap_or_default(),
                crate::order::operator("<", other, &k, &h).unwrap_or_default(),
            )
        }
        other => {
            return Err(Residue::Form(format!(
                "insert by a key of {}, whose order the lift does not model",
                other.render()
            )))
        }
    };
    let acc = if crate::order::derived(&key, &env.types)
        && !matches!(key, Ty::Nat(_) | Ty::Str | Ty::Char)
    {
        acc.ordered(&key, &env.types)
    } else {
        acc
    };
    let name = ctx.helper_name(env, &acc, "Insert");
    let acc = acc.saw(&entry);
    // A structural match names the list only through its parts.
    let whole = format!("(cons {head} {rest})");
    let kept = match ty {
        Ty::Map(..) => format!("(cons {item} {rest})"),
        _ => whole.clone(),
    };
    let body = Expr::Match {
        scrutinee: items.clone(),
        arms: vec![
            Arm {
                pattern: "List.nil".into(),
                body: Expr::Flat(format!("(cons {item} (nil {}))", entry.render())),
            },
            Arm {
                pattern: format!("List.cons {head} {rest}"),
                body: Expr::Flat(format!(
                    "(if {same} then {kept} else (if {less} then (cons {item} {whole}) else (cons {head} ({name} {rest} {item}))))"
                )),
            },
        ],
    };
    let acc = acc.axioms_all(axioms.clone());
    let acc = acc.helper(Helper {
        name: name.clone(),
        params: vec![(items, list.render()), (item, entry.render())],
        result: list.render(),
        body,
        calls: BTreeSet::new(),
        axioms,
        recursive: true,
        entwined: false,
        termination: None,
    });
    let acc = acc.called(format!("#{name}"));
    Ok((name, acc))
}

/// The helper inserting each item of a list into an ordered set, in turn,
/// through the set's `insert` helper.
fn insert_all(ty: &Ty, items: &Ty, insert: &str, env: &Env, ctx: &Ctx, acc: Acc) -> (String, Acc) {
    let (set, acc) = ctx.fresh(env, acc, "set");
    let (list, acc) = ctx.fresh(env, acc, "items");
    let (head, acc) = ctx.fresh(env, acc, "head");
    let (rest, acc) = ctx.fresh(env, acc, "rest");
    let name = ctx.helper_name(env, &acc, "InsertAll");
    let body = Expr::Match {
        scrutinee: list.clone(),
        arms: vec![
            Arm {
                pattern: "List.nil".into(),
                body: Expr::Flat(set.clone()),
            },
            Arm {
                pattern: format!("List.cons {head} {rest}"),
                body: Expr::Flat(format!("({name} {rest} ({insert} {set} {head}))")),
            },
        ],
    };
    let axioms = acc
        .helpers
        .iter()
        .find(|h| h.name == insert)
        .map(|h| h.axioms.clone())
        .unwrap_or_default();
    let acc = acc.helper(Helper {
        name: name.clone(),
        params: vec![(list, items.render()), (set, ty.render())],
        result: ty.render(),
        body,
        calls: BTreeSet::from([format!("#{insert}")]),
        axioms,
        recursive: true,
        entwined: false,
        termination: None,
    });
    let acc = acc.called(format!("#{name}"));
    (name, acc)
}

/// The locals the statements update and the locals they declare, outside
/// closures.
fn scan_stmts(stmts: &[rs::Stmt]) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut updated = BTreeSet::new();
    let mut declared = BTreeSet::new();
    for stmt in stmts {
        let (u, d) = match stmt {
            rs::Stmt::Local(l) => {
                let mut exprs: Vec<&rs::RustExpr> = Vec::new();
                if let Some(init) = &l.init {
                    exprs.push(&init.expr);
                    exprs.extend(init.diverge.as_deref());
                }
                let (u, mut d) = scan_exprs(&exprs);
                if let rs::Pat::Ident(i) = &l.pat {
                    d.insert(i.ident.clone());
                }
                (u, d)
            }
            rs::Stmt::Expr(e, _) => scan_exprs(&[e]),
            rs::Stmt::Macro(_) | rs::Stmt::Other => (BTreeSet::new(), BTreeSet::new()),
        };
        updated.extend(u);
        declared.extend(d);
    }
    (updated, declared)
}

/// [`scan_stmts`] over expressions.
fn scan_exprs(exprs: &[&rs::RustExpr]) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut updated = BTreeSet::new();
    let mut declared = BTreeSet::new();
    for e in exprs {
        if matches!(e, rs::RustExpr::Closure(_)) {
            continue;
        }
        updated.extend(update_target(e));
        let (inner, blocks) = e.parts();
        let (u, d) = scan_exprs(&inner);
        updated.extend(u);
        declared.extend(d);
        for b in blocks {
            let (u, d) = scan_stmts(&b.stmts);
            updated.extend(u);
            declared.extend(d);
        }
    }
    (updated, declared)
}

/// The locals a loop body updates that are bound before the loop.
fn loop_state(body: &[rs::Stmt], ctx: &Ctx) -> Lift<Vec<String>> {
    let (updated, declared) = scan_stmts(body);
    // A local the body declares and updates is the body's own, unless it
    // shadows one bound before the loop.
    if updated
        .iter()
        .any(|u| declared.contains(u) && ctx.locals.contains_key(u))
    {
        return Err(Residue::Form("a loop that shadows its state".to_string()));
    }
    let state: Vec<String> = updated
        .into_iter()
        .filter(|u| ctx.locals.contains_key(u) && !declared.contains(u))
        .collect();
    for local in &state {
        ctx.unsealed(local)?;
    }
    // A loop of the function's own statements that updates nothing may
    // still leave it: a walk whose state is `()`.
    if state.is_empty() && !ctx.top {
        return Err(Residue::Form("a loop that updates no local".to_string()));
    }
    Ok(state)
}

/// The state of a loop as one value: the local, or the tuple of them.
fn state_expr(state: &[String]) -> rs::RustExpr {
    let path = |name: &String| rs::RustExpr::Path(rs::Path::ident(name));
    match state {
        [one] => path(one),
        many => rs::RustExpr::Tuple(many.iter().map(path).collect()),
    }
}

/// Every tail of a lifted expression, rewritten: the step of a loop ends in
/// its recursive call on the new state.
fn map_tails(e: Expr, f: &dyn Fn(String) -> String) -> Expr {
    match e {
        Expr::Flat(s) => Expr::Flat(f(s)),
        Expr::If {
            condition,
            then,
            otherwise,
        } => Expr::If {
            condition,
            then: Box::new(map_tails(*then, f)),
            otherwise: Box::new(map_tails(*otherwise, f)),
        },
        Expr::Match { scrutinee, arms } => Expr::Match {
            scrutinee,
            arms: arms
                .into_iter()
                .map(|a| Arm {
                    pattern: a.pattern,
                    body: map_tails(a.body, f),
                })
                .collect(),
        },
        Expr::Let {
            name,
            ty,
            value,
            body,
        } => Expr::Let {
            name,
            ty,
            value,
            body: Box::new(map_tails(*body, f)),
        },
    }
}

/// Bind a loop's state to the names its locals take next: the value itself,
/// or each item of the tuple.
fn rebind(
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    state: &[String],
    whole: &str,
    lets: Lets,
) -> (Ctx, Acc, Lets) {
    let mut scope = ctx.clone();
    if let [one] = state {
        let ty = ctx.locals[one].1.clone();
        if let Some((old, _)) = scope.locals.insert(one.clone(), (whole.to_string(), ty)) {
            scope.hidden.push(old);
        }
        return (scope, acc, lets);
    }
    let mut acc = acc;
    let mut lets = lets;
    for (i, var) in state.iter().enumerate() {
        let ty = ctx.locals[var].1.clone();
        let (next, name, after) = scope.bind(env, acc, var, ty.clone());
        scope = next;
        acc = after;
        lets.push((name, ty.render(), format!("(project {whole} item{i})")));
    }
    (scope, acc, lets)
}

fn wrap_lets(lets: Vec<(String, String, String)>, body: Expr) -> Expr {
    lets.into_iter()
        .rev()
        .fold(body, |body, (name, ty, value)| Expr::Let {
            name,
            ty,
            value: Box::new(Expr::Flat(value)),
            body: Box::new(body),
        })
}

/// The mark on a loop step's value that ends the walk: a `break`.
const BREAK: &str = "#break ";

/// The mark on a loop step's value that leaves the function: a `return`
/// or a failed `?`.
const LEAVE: &str = "#leave ";

/// The value of a `continue` or `break` in a loop step: the loop's state as
/// the step leaves it.
fn loop_exit(env: &Env, ctx: &Ctx, acc: Acc, label: bool, breaks: bool) -> Lift<((Expr, Ty), Acc)> {
    let Some(state) = &ctx.looping else {
        return Err(Residue::Form(
            "continue or break outside a loop step".to_string(),
        ));
    };
    if label {
        return Err(Residue::Form("labelled continue or break".to_string()));
    }
    let ((v, t), acc) = flat_of(&state_expr(state), env, ctx, acc, None)?;
    let v = if breaks { format!("{BREAK}{v}") } else { v };
    Ok(((Expr::Flat(v), t), acc))
}

/// `for p in xs { body } rest`: a helper that walks `xs` with the locals the
/// body updates as its accumulator, then `rest` with their final values.
fn for_loop(
    f: &rs::ExprForLoop,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    if f.label {
        return Err(Residue::Form("labelled loop".to_string()));
    }
    let ((list, lt), acc) = flat_of(&f.expr, env, &ctx.below(), acc, None)?;
    let elem = match &lt {
        Ty::List(t) => (**t).clone(),
        Ty::Set(t, true) => (**t).clone(),
        Ty::Map(_, _, true) => lt.entry(),
        _ => {
            return Err(Residue::Form(
                "a loop over something other than a list".to_string(),
            ))
        }
    };
    let state = loop_state(&f.body.stmts, ctx)?;
    let tys: Vec<Ty> = state.iter().map(|v| ctx.locals[v].1.clone()).collect();
    let state_ty = match tys.as_slice() {
        [] => Ty::Unit,
        [one] => one.clone(),
        many => Ty::Tuple(many.to_vec()),
    };
    let acc = acc.saw(&state_ty);
    // The step, in a scope of its own.
    let (acc, before) = acc.apart();
    let mut base = Ctx {
        tails: BTreeMap::new(),
        looping: Some(state.clone()),
        exits: ctx.top.then(|| state_ty.clone()),
        ..ctx.clone()
    };
    // The outer names stay taken inside the step: the step's binders may
    // not shadow a capture.
    for (rust, binding) in &ctx.locals {
        base.locals
            .insert(format!("#outer {rust}"), binding.clone());
    }
    let (current, acc) = base.fresh(env, acc, "state");
    let (scope, acc, lets) = rebind(env, &base, acc, &state, &current, Vec::new());
    let (mut scope, head, acc, pat_lets) = bind_pat(&f.pat, &elem, env, &scope, acc, Vec::new())?;
    // Each element of a list that is a sub-term of a parameter is one: a
    // recursive call on it descends, and the loop and the function are one
    // mutual group.
    let element_of = ctx.tails.get(&list).cloned();
    if let Some(param) = &element_of {
        scope.tails.insert(head.clone(), param.clone());
    }
    let mut stmts = f.body.stmts.clone();
    stmts.push(rs::Stmt::Expr(state_expr(&state), false));
    let ((step, st), acc) = block(&stmts, env, &scope, acc, Some(&state_ty))?;
    let (acc, own) = acc.rejoin(before);
    if !st.same(&state_ty) || (!own.descends.is_empty() && element_of.is_none()) {
        return Err(Residue::Form("a loop step of another type".to_string()));
    }
    let entwined = !own.descends.is_empty();
    let acc = own.descends.iter().fold(acc, |acc, i| acc.descended(*i));
    let name = ctx.helper_name(env, &acc, "Loop");
    let (items, acc) = ctx.fresh(env, acc, "items");
    let (rest_name, acc) = ctx.fresh(env, acc, "rest");
    let text = step.flat();
    // A step that can leave the function walks to the final state or to
    // the function's value.
    let leaving = text.contains(LEAVE);
    if state.is_empty() && !leaving {
        return Err(Residue::Form("a loop that updates no local".to_string()));
    }
    let walked = if leaving {
        Ty::Outcome(Box::new(state_ty.clone()), Box::new(ctx.result.clone()))
    } else {
        state_ty.clone()
    };
    let acc = acc.saw(&walked);
    let done = |s: String| {
        if leaving {
            format!(
                "(ctor Result.ok [{}, {}] {s})",
                state_ty.render(),
                ctx.result.render()
            )
        } else {
            s
        }
    };
    let caps = captures(ctx, &[text], std::slice::from_ref(&current));
    let cap_names: Vec<String> = caps.iter().map(|(c, _)| c.clone()).collect();
    let recur = |s: String| {
        if let Some(last) = s.strip_prefix(BREAK) {
            return done(last.to_string());
        }
        if let Some(value) = s.strip_prefix(LEAVE) {
            return format!(
                "(ctor Result.error [{}, {}] {value})",
                state_ty.render(),
                ctx.result.render()
            );
        }
        let mut parts = vec![name.clone(), rest_name.clone(), s];
        parts.extend(cap_names.iter().cloned());
        format!("({})", parts.join(" "))
    };
    let body = wrap_lets(lets, wrap_lets(pat_lets, map_tails(step, &recur)));
    let mut params = vec![
        (items.clone(), lt.render()),
        (current.clone(), state_ty.render()),
    ];
    params.extend(caps.iter().map(|(c, t)| (c.clone(), t.render())));
    let acc = acc.helper(Helper {
        name: name.clone(),
        params,
        result: walked.render(),
        body: Expr::Match {
            scrutinee: items,
            arms: vec![
                Arm {
                    pattern: "List.nil".to_string(),
                    body: Expr::Flat(done(current)),
                },
                Arm {
                    pattern: format!("List.cons {head} {rest_name}"),
                    body,
                },
            ],
        },
        calls: own.calls,
        axioms: own.axioms,
        recursive: true,
        entwined,
        termination: None,
    });
    let acc = acc.called(format!("#{name}"));
    // The loop's value, then the rest with the final state.
    let ((init, _), acc) = flat_of(&state_expr(&state), env, &ctx.below(), acc, Some(&state_ty))?;
    let mut parts = vec![name, list, init];
    parts.extend(cap_names);
    let (whole, acc) = ctx.fresh(env, acc, "state");
    let (after, acc, lets) = rebind(env, ctx, acc, &state, &whole, Vec::new());
    let ((rest, rt), acc) = block(rest, env, &after, acc, want)?;
    let call = format!("({})", parts.join(" "));
    if !leaving {
        return Ok((
            (
                Expr::Let {
                    name: whole,
                    ty: state_ty.render(),
                    value: Box::new(Expr::Flat(call)),
                    body: Box::new(wrap_lets(lets, rest)),
                },
                rt,
            ),
            acc,
        ));
    }
    // The walk's value: the final state goes on to the rest, the
    // function's value leaves.
    let (value, acc) = ctx.fresh(env, acc, "value");
    let (left, left_ty) = ctx.returning((Expr::Flat(value.clone()), ctx.result.clone()));
    if !left_ty.same(&rt) {
        return Err(Residue::Form("a loop step of another type".to_string()));
    }
    Ok((
        (
            Expr::Match {
                scrutinee: call,
                arms: vec![
                    Arm {
                        pattern: format!("Result.ok {whole}"),
                        body: wrap_lets(lets, rest),
                    },
                    Arm {
                        pattern: format!("Result.error {value}"),
                        body: left,
                    },
                ],
            },
            rt,
        ),
        acc,
    ))
}

/// The cursor and bound of a loop condition `i < n`: a local the loop
/// updates below a value it does not.
fn cursor_of<'e>(cond: &'e rs::RustExpr, state: &[String]) -> Option<(String, &'e rs::RustExpr)> {
    let rs::RustExpr::Binary(b) = cond else {
        return None;
    };
    let rs::RustExpr::Path(p) = &*b.left else {
        return None;
    };
    let cursor = p.get_ident()?.clone();
    let (updated_bound, _) = scan_exprs(&[&b.right]);
    (b.op == rs::BinOp::Lt
        && state.contains(&cursor)
        && updated_bound.is_empty()
        && !state.iter().any(|name| uses_local(&b.right, name)))
    .then_some((cursor, &*b.right))
}

/// Whether an expression reads the local `name`.
fn uses_local(e: &rs::RustExpr, name: &str) -> bool {
    match e {
        rs::RustExpr::Path(p) => p.is_ident(name),
        rs::RustExpr::Macro(_) => true,
        other => {
            let (exprs, blocks) = other.parts();
            exprs.iter().any(|x| uses_local(x, name))
                || blocks.iter().any(|b| {
                    b.stmts.iter().any(|s| match s {
                        rs::Stmt::Local(l) => {
                            l.init.as_ref().is_some_and(|i| uses_local(&i.expr, name))
                        }
                        rs::Stmt::Expr(x, _) => uses_local(x, name),
                        rs::Stmt::Macro(_) | rs::Stmt::Other => true,
                    })
                })
        }
    }
}

/// The constant a statement advances the local `cursor` by: `i += k` or
/// `i = i + k`, with `k` at least one.
fn advance(stmt: &rs::Stmt, cursor: &str) -> Option<u128> {
    let rs::Stmt::Expr(e, true) = stmt else {
        return None;
    };
    let is_cursor = |x: &rs::RustExpr| matches!(x, rs::RustExpr::Path(p) if p.is_ident(cursor));
    let k = match e {
        rs::RustExpr::Binary(b) if b.op == rs::BinOp::AddAssign && is_cursor(&b.left) => {
            int_literal(&b.right)
        }
        rs::RustExpr::Assign(a) if is_cursor(&a.left) => match &*a.right {
            rs::RustExpr::Binary(b) if b.op == rs::BinOp::Add && is_cursor(&b.left) => {
                int_literal(&b.right)
            }
            _ => None,
        },
        _ => None,
    }?;
    (k >= 1).then_some(k)
}

/// Whether statements leave a loop step other than at its end: a
/// `continue`, `break`, `return` or `?` outside a closure.
fn leaves_step(stmts: &[rs::Stmt]) -> bool {
    stmts.iter().any(|s| match s {
        rs::Stmt::Local(l) => l
            .init
            .as_ref()
            .is_some_and(|i| i.diverge.is_some() || leaves_in(&i.expr)),
        rs::Stmt::Expr(e, _) => leaves_in(e),
        rs::Stmt::Macro(_) | rs::Stmt::Other => false,
    })
}

/// [`leaves_step`] in an expression.
fn leaves_in(e: &rs::RustExpr) -> bool {
    match e {
        rs::RustExpr::Continue(_)
        | rs::RustExpr::Break(_)
        | rs::RustExpr::Return(_)
        | rs::RustExpr::Try(_) => true,
        rs::RustExpr::Closure(_) => false,
        other => {
            let (exprs, blocks) = other.parts();
            exprs.iter().any(|x| leaves_in(x)) || blocks.iter().any(|b| leaves_step(&b.stmts))
        }
    }
}

/// `while i < n { body } rest`: a walk whose state is the locals the body
/// updates, measured by `n - i`. The body advances `i` by a constant once
/// and leaves only at its end, so each step is a helper of the state, and
/// the walk calls itself on the step while the condition holds; its one
/// recursive call carries the theorem that the step decreases `n - i`.
fn while_loop(
    w: &rs::ExprWhile,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    if w.label {
        return Err(Residue::Form("labelled loop".to_string()));
    }
    if matches!(&*w.cond, rs::RustExpr::Let(_)) {
        return Err(Residue::Form(
            "a while let, whose walk the lift reads no measure for".to_string(),
        ));
    }
    let state = loop_state(&w.body.stmts, ctx)?;
    let Some((cursor, bound)) = cursor_of(&w.cond, &state) else {
        return Err(Residue::Form(
            "a while loop whose condition is not a cursor below a bound".to_string(),
        ));
    };
    let steps: Vec<&rs::Stmt> = w
        .body
        .stmts
        .iter()
        .filter(|s| advance(s, &cursor).is_some())
        .collect();
    let elsewhere = w
        .body
        .stmts
        .iter()
        .filter(|s| advance(s, &cursor).is_none())
        .any(|s| scan_stmts(std::slice::from_ref(s)).0.contains(&cursor));
    if steps.len() != 1 || elsewhere || leaves_step(&w.body.stmts) {
        return Err(Residue::Form(
            "a while loop that does not advance its cursor once per step".to_string(),
        ));
    }
    // `i < n` types `i` as `n`: a cursor below a `usize` is one, though
    // the lift read its unsuffixed literal as a `u32`. The bound's type is
    // checked below.
    let mut retyped = ctx.clone();
    match retyped.locals.get_mut(&cursor) {
        Some((_, t @ Ty::Nat(_))) => *t = Ty::Nat(None),
        _ => {
            return Err(Residue::Form(
                "a while loop over a cursor of another type".to_string(),
            ))
        }
    }
    let ctx = &retyped;
    let tys: Vec<Ty> = state.iter().map(|v| ctx.locals[v].1.clone()).collect();
    let state_ty = match tys.as_slice() {
        [one] => one.clone(),
        many => Ty::Tuple(many.to_vec()),
    };
    let acc = acc.saw(&state_ty);
    let Some(step_by) = steps.first().and_then(|s| advance(s, &cursor)) else {
        return Err(Residue::Form(
            "a while loop that does not advance its cursor once per step".to_string(),
        ));
    };
    // The walk takes the cursor apart from the rest of the state: the
    // decrease reads the cursor alone, and its next value is the cursor
    // plus the constant, which the step adds once.
    let others: Vec<String> = state.iter().filter(|v| **v != cursor).cloned().collect();
    let other_tys: Vec<Ty> = others.iter().map(|v| ctx.locals[v].1.clone()).collect();
    let rest_ty = match other_tys.as_slice() {
        [] => None,
        [one] => Some(one.clone()),
        many => Some(Ty::Tuple(many.to_vec())),
    };
    let acc = rest_ty.as_ref().map_or(acc.clone(), |t| acc.saw(t));
    let (acc, before) = acc.apart();
    let mut base = Ctx {
        tails: BTreeMap::new(),
        looping: None,
        exits: None,
        top: false,
        ..ctx.clone()
    };
    // The outer names stay taken inside the walk.
    for (rust, binding) in &ctx.locals {
        base.locals
            .insert(format!("#outer {rust}"), binding.clone());
    }
    let (at, acc) = base.fresh(env, acc, "at");
    let (held, acc) = base.fresh(env, acc, "state");
    let mut scope = base.clone();
    if let Some((old, _)) = scope
        .locals
        .insert(cursor.clone(), (at.clone(), Ty::Nat(None)))
    {
        scope.hidden.push(old);
    }
    // The condition and the bound read the cursor from the walk's
    // parameter: LexLean states a decrease over parameters only.
    let ((cond, ct), acc) = flat_of(&w.cond, env, &scope.below(), acc, Some(&Ty::Bool))?;
    let ((limit, lt), acc) = flat_of(bound, env, &scope.below(), acc, Some(&Ty::Nat(None)))?;
    if ct != Ty::Bool || lt != Ty::Nat(None) {
        return Err(Residue::Form(
            "a while loop over a cursor of another type".to_string(),
        ));
    }
    // The rest of the state after one step, from the state where the
    // condition holds: the cursor is below a length there.
    let mut step_text = Vec::new();
    let mut step_def = None;
    let mut acc = acc;
    if let Some(rt) = &rest_ty {
        let (inner, after, lets) = rebind(env, &scope, acc, &others, &held, Vec::new());
        let mut inner = inner;
        let is_len =
            matches!(bound, rs::RustExpr::MethodCall(m) if m.method == "len" && m.args.is_empty());
        let half = env
            .widths
            .first()
            .and_then(|(w, _)| 1u128.checked_shl(w - 1));
        let room = if is_len { half.unwrap_or(1) } else { 1 };
        inner.capped.insert(at.clone(), room);
        let mut stmts = w.body.stmts.clone();
        stmts.push(rs::Stmt::Expr(state_expr(&others), false));
        let ((step, st), after) = block(&stmts, env, &inner, after, Some(rt))?;
        if !st.same(rt) {
            return Err(Residue::Form("a loop step of another type".to_string()));
        }
        acc = after;
        let step = wrap_lets(lets, step);
        step_text.push(step.flat());
        step_def = Some(step);
    }
    let (acc, own) = acc.rejoin(before);
    if !own.descends.is_empty() {
        return Err(Residue::Form("recursion inside a loop step".to_string()));
    }
    let mut texts = step_text;
    texts.push(cond.clone());
    texts.push(limit.clone());
    let caps = captures(ctx, &texts, &[at.clone(), held.clone()]);
    let cap_names: Vec<String> = caps.iter().map(|(c, _)| c.clone()).collect();
    let mut params = vec![(at.clone(), Ty::Nat(None).render())];
    if let Some(rt) = &rest_ty {
        params.push((held.clone(), rt.render()));
    }
    params.extend(caps.iter().map(|(c, t)| (c.clone(), t.render())));
    let applied = |head: &str, lead: &[String]| {
        let mut parts = vec![head.to_string()];
        parts.extend(lead.iter().cloned());
        parts.extend(cap_names.iter().cloned());
        format!("({})", parts.join(" "))
    };
    let mut lead = vec![format!("({at} + {step_by})")];
    let mut acc = acc;
    let mut own_calls = BTreeSet::new();
    if let Some(step) = step_def {
        let step_name = ctx.helper_name(env, &acc, "Step");
        acc = acc
            .helper(Helper {
                name: step_name.clone(),
                params: params.clone(),
                result: rest_ty.as_ref().map(Ty::render).unwrap_or_default(),
                body: step,
                calls: own.calls,
                axioms: own.axioms,
                recursive: false,
                entwined: false,
                termination: None,
            })
            .called(format!("#{step_name}"));
        own_calls.insert(format!("#{step_name}"));
        lead.push(applied(&step_name, &[at.clone(), held.clone()]));
    }
    // The final state, in the locals' order.
    let items: Vec<String> = state
        .iter()
        .map(|v| match others.iter().position(|o| o == v) {
            None => at.clone(),
            Some(_) if others.len() == 1 => held.clone(),
            Some(j) => tuple_item(rest_ty.as_ref().unwrap_or(&Ty::Unit), &held, j),
        })
        .collect();
    let done = match items.as_slice() {
        [one] => one.clone(),
        _ => tuple_value(&state_ty, &items),
    };
    let measure = |cursor_text: &str| format!("(prim subtract {limit} {cursor_text} : Nat)");
    let name = ctx.helper_name(env, &acc, "Loop");
    let evidence = format!("decreases_{name}_0");
    let axioms: Vec<String> = ["Quot.sound", "propext"].map(String::from).to_vec();
    let theorem = Decl::Theorem {
        type_parameters: Vec::new(),
        name: evidence,
        parameters: params.clone(),
        statement: format!(
            "(({cond} = true) -> ({} < {}))",
            measure(&lead[0]),
            measure(&at)
        ),
        proof: "linear_arithmetic{}".to_string(),
        axioms: axioms.clone(),
    };
    let acc = acc.helper(Helper {
        name: name.clone(),
        params,
        result: state_ty.render(),
        body: Expr::If {
            condition: Box::new(Expr::Flat(cond)),
            then: Box::new(Expr::Flat(applied(&name, &lead))),
            otherwise: Box::new(Expr::Flat(done)),
        },
        calls: own_calls,
        axioms: axioms.into_iter().collect(),
        recursive: false,
        entwined: false,
        termination: Some((measure(&at), vec![theorem])),
    });
    let acc = acc.called(format!("#{name}"));
    // The walk from the state before the loop, then the rest with its end.
    let ((first, _), acc) = flat_of(
        &rs::RustExpr::Path(rs::Path::ident(&cursor)),
        env,
        &ctx.below(),
        acc,
        Some(&Ty::Nat(None)),
    )?;
    let mut start = vec![first];
    if let Some(rt) = &rest_ty {
        let ((init, _), next) = flat_of(
            &state_expr(&others),
            env,
            &ctx.below(),
            acc.clone(),
            Some(rt),
        )?;
        start.push(init);
        return finish_walk(
            applied(&name, &start),
            &state,
            &state_ty,
            rest,
            env,
            ctx,
            next,
            want,
        );
    }
    finish_walk(
        applied(&name, &start),
        &state,
        &state_ty,
        rest,
        env,
        ctx,
        acc,
        want,
    )
}

/// The rest of a block after a walk: the walk's value bound, and the
/// statements after it with the locals its state names taking their final
/// values.
#[allow(clippy::too_many_arguments)]
fn finish_walk(
    call: String,
    state: &[String],
    state_ty: &Ty,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let (whole, acc) = ctx.fresh(env, acc, "state");
    let (after, acc, lets) = rebind(env, ctx, acc, state, &whole, Vec::new());
    let ((rest, rt), acc) = block(rest, env, &after, acc, want)?;
    Ok((
        (
            Expr::Let {
                name: whole,
                ty: state_ty.render(),
                value: Box::new(Expr::Flat(call)),
                body: Box::new(wrap_lets(lets, rest)),
            },
            rt,
        ),
        acc,
    ))
}

/// The first `?` an expression evaluates unconditionally, and the
/// expression with the local `var` in its place: its operand, and the rest.
/// A `?` under a closure, a branch or the right side of `&&`/`||` may not
/// run, so it stays.
fn take_try(e: &rs::RustExpr, var: &str) -> Option<(rs::RustExpr, rs::RustExpr)> {
    use rs::RustExpr as E;
    match e {
        E::Try(t) => Some(match take_try(t, var) {
            Some((found, inner)) => (found, E::Try(Box::new(inner))),
            None => ((**t).clone(), E::Path(rs::Path::ident(var))),
        }),
        E::MethodCall(m) => try_in_method(m, var),
        E::Call(c) => try_in_call(c, var),
        E::Field(f) => try_in_field(f, var),
        E::Paren(p) => {
            let (found, inner) = take_try(p, var)?;
            Some((found, E::Paren(Box::new(inner))))
        }
        E::Reference(r) => try_in_reference(r, var),
        E::Unary(u) => try_in_unary(u, var),
        E::Cast(c) => try_in_cast(c, var),
        E::Index(i) => try_in_index(i, var),
        E::Binary(b) => try_in_binary(b, var),
        E::Tuple(items) => {
            let (found, items) = take_first(items, var)?;
            Some((found, E::Tuple(items)))
        }
        E::Struct(st) => try_in_struct(st, var),
        E::If(i) => try_in_if(i, var),
        E::Match(m) => try_in_match(m, var),
        _ => None,
    }
}

/// The first `?` of a method call: in its receiver, else its arguments.
fn try_in_method(m: &rs::ExprMethodCall, var: &str) -> Option<(rs::RustExpr, rs::RustExpr)> {
    if let Some((found, receiver)) = take_try(&m.receiver, var) {
        let mut m = m.clone();
        m.receiver = Box::new(receiver);
        return Some((found, rs::RustExpr::MethodCall(m)));
    }
    let (found, args) = take_first(&m.args, var)?;
    let mut m = m.clone();
    m.args = args;
    Some((found, rs::RustExpr::MethodCall(m)))
}

/// The first `?` of a call's arguments.
fn try_in_call(c: &rs::ExprCall, var: &str) -> Option<(rs::RustExpr, rs::RustExpr)> {
    let (found, args) = take_first(&c.args, var)?;
    let mut c = c.clone();
    c.args = args;
    Some((found, rs::RustExpr::Call(c)))
}

/// The first `?` of a field's base.
fn try_in_field(f: &rs::ExprField, var: &str) -> Option<(rs::RustExpr, rs::RustExpr)> {
    let (found, base) = take_try(&f.base, var)?;
    let mut f = f.clone();
    f.base = Box::new(base);
    Some((found, rs::RustExpr::Field(f)))
}

/// The first `?` under a reference.
fn try_in_reference(r: &rs::ExprReference, var: &str) -> Option<(rs::RustExpr, rs::RustExpr)> {
    let (found, inner) = take_try(&r.expr, var)?;
    let mut r = r.clone();
    r.expr = Box::new(inner);
    Some((found, rs::RustExpr::Reference(r)))
}

/// The first `?` under a unary operator.
fn try_in_unary(u: &rs::ExprUnary, var: &str) -> Option<(rs::RustExpr, rs::RustExpr)> {
    let (found, inner) = take_try(&u.expr, var)?;
    let mut u = u.clone();
    u.expr = Box::new(inner);
    Some((found, rs::RustExpr::Unary(u)))
}

/// The first `?` of a cast's operand.
fn try_in_cast(c: &rs::ExprCast, var: &str) -> Option<(rs::RustExpr, rs::RustExpr)> {
    let (found, inner) = take_try(&c.expr, var)?;
    let mut c = c.clone();
    c.expr = Box::new(inner);
    Some((found, rs::RustExpr::Cast(c)))
}

/// The first `?` of an index: the indexed value's, else the index's.
fn try_in_index(i: &rs::ExprIndex, var: &str) -> Option<(rs::RustExpr, rs::RustExpr)> {
    let mut i = i.clone();
    if let Some((found, inner)) = take_try(&i.expr, var) {
        i.expr = Box::new(inner);
        return Some((found, rs::RustExpr::Index(i)));
    }
    let (found, index) = take_try(&i.index, var)?;
    i.index = Box::new(index);
    Some((found, rs::RustExpr::Index(i)))
}

/// The first `?` of a binary operation: the left's, else, where the
/// right is always evaluated, the right's.
fn try_in_binary(b: &rs::ExprBinary, var: &str) -> Option<(rs::RustExpr, rs::RustExpr)> {
    let mut b = b.clone();
    if let Some((found, left)) = take_try(&b.left, var) {
        b.left = Box::new(left);
        return Some((found, rs::RustExpr::Binary(b)));
    }
    if matches!(b.op, rs::BinOp::And | rs::BinOp::Or) {
        return None;
    }
    let (found, right) = take_try(&b.right, var)?;
    b.right = Box::new(right);
    Some((found, rs::RustExpr::Binary(b)))
}

/// The first `?` of a struct literal's field values.
fn try_in_struct(st: &rs::ExprStruct, var: &str) -> Option<(rs::RustExpr, rs::RustExpr)> {
    let values: Vec<rs::RustExpr> = st.fields.iter().map(|f| f.expr.clone()).collect();
    let (found, values) = take_first(&values, var)?;
    let mut st = st.clone();
    for (field, value) in st.fields.iter_mut().zip(values) {
        field.expr = value;
    }
    Some((found, rs::RustExpr::Struct(st)))
}

/// The first `?` of an `if`'s condition.
fn try_in_if(i: &rs::ExprIf, var: &str) -> Option<(rs::RustExpr, rs::RustExpr)> {
    let (found, cond) = take_try(&i.cond, var)?;
    let mut i = i.clone();
    i.cond = Box::new(cond);
    Some((found, rs::RustExpr::If(i)))
}

/// The first `?` of a match's scrutinee.
fn try_in_match(m: &rs::ExprMatch, var: &str) -> Option<(rs::RustExpr, rs::RustExpr)> {
    let (found, scrutinee) = take_try(&m.expr, var)?;
    let mut m = m.clone();
    m.expr = Box::new(scrutinee);
    Some((found, rs::RustExpr::Match(m)))
}

/// [`take_try`] on the first of these expressions that has one.
fn take_first(items: &[rs::RustExpr], var: &str) -> Option<(rs::RustExpr, Vec<rs::RustExpr>)> {
    let (at, (found, replaced)) = items
        .iter()
        .enumerate()
        .find_map(|(i, e)| take_try(e, var).map(|hit| (i, hit)))?;
    let mut items = items.to_vec();
    items[at] = replaced;
    Some((found, items))
}

/// The map and key of `m.entry(k).or_default()`, where `m` is a local.
fn default_entry(e: &rs::RustExpr) -> Option<(&rs::RustExpr, &rs::RustExpr)> {
    let rs::RustExpr::MethodCall(d) = e else {
        return None;
    };
    let rs::RustExpr::MethodCall(en) = &*d.receiver else {
        return None;
    };
    match (
        d.method.as_str(),
        d.args.as_slice(),
        en.method.as_str(),
        en.args.as_slice(),
    ) {
        ("or_default", [], "entry", [k]) if matches!(&*en.receiver, rs::RustExpr::Path(p) if p.get_ident().is_some()) => {
            Some((&en.receiver, k))
        }
        _ => None,
    }
}

/// An update through an entry, `m.entry(k).or_default().push(v);` or
/// `*m.entry(k).or_default() += v;`, as the statements that read the entry
/// into the local `entry{n}`, update it, and insert it back.
fn entry_update(stmt: &rs::Stmt, n: usize) -> Option<Vec<rs::Stmt>> {
    let rs::Stmt::Expr(e, true) = stmt else {
        return None;
    };
    let var = format!("entry{n}");
    let local = rs::RustExpr::Path(rs::Path::ident(&var));
    let ((map, key), update) = match e {
        rs::RustExpr::MethodCall(m) => {
            let found = default_entry(&m.receiver)?;
            let update = rs::RustExpr::MethodCall(rs::ExprMethodCall {
                receiver: Box::new(local.clone()),
                ..m.clone()
            });
            (found, update)
        }
        rs::RustExpr::Binary(b) if b.op == rs::BinOp::AddAssign => {
            let rs::RustExpr::Unary(u) = &*b.left else {
                return None;
            };
            if !matches!(u.op, rs::UnOp::Deref) {
                return None;
            }
            let found = default_entry(&u.expr)?;
            let update = rs::RustExpr::Binary(rs::ExprBinary {
                left: Box::new(local.clone()),
                ..b.clone()
            });
            (found, update)
        }
        _ => return None,
    };
    let call = |receiver: rs::RustExpr, method: &str, args: Vec<rs::RustExpr>| {
        rs::RustExpr::MethodCall(rs::ExprMethodCall {
            receiver: Box::new(receiver),
            method: method.to_string(),
            turbofish: None,
            args,
        })
    };
    let borrowed = rs::RustExpr::Reference(rs::ExprReference {
        mutable: false,
        expr: Box::new(key.clone()),
    });
    let read = call(
        call(
            call(map.clone(), "get", vec![borrowed]),
            "cloned",
            Vec::new(),
        ),
        "unwrap_or_default",
        Vec::new(),
    );
    let bind = rs::Stmt::Local(rs::Local {
        pat: rs::Pat::Ident(rs::PatIdent {
            ident: var,
            by_ref: false,
            subpat: None,
        }),
        init: Some(rs::LocalInit {
            expr: Box::new(read),
            diverge: None,
        }),
    });
    let write = call(map.clone(), "insert", vec![key.clone(), local]);
    Some(vec![
        bind,
        rs::Stmt::Expr(update, true),
        rs::Stmt::Expr(write, true),
    ])
}

/// A statement with a `?` inside it, as `let v = e?;` before the statement
/// with `v` in its place.
fn hoist_try(stmt: &rs::Stmt, n: usize) -> Option<(rs::Stmt, rs::Stmt)> {
    let var = format!("tried{n}");
    let (inner, stmt) = match stmt {
        rs::Stmt::Expr(e, semi) => {
            let (inner, e) = take_try(e, &var)?;
            (inner, rs::Stmt::Expr(e, *semi))
        }
        rs::Stmt::Local(l) => {
            let init = l.init.as_ref()?;
            if matches!(&*init.expr, rs::RustExpr::Try(_)) && matches!(l.pat, rs::Pat::Ident(_)) {
                return None;
            }
            let (inner, e) = take_try(&init.expr, &var)?;
            let mut l = l.clone();
            if let Some(init) = &mut l.init {
                *init.expr = e;
            }
            (inner, rs::Stmt::Local(l))
        }
        rs::Stmt::Macro(_) | rs::Stmt::Other => return None,
    };
    let binding = rs::Stmt::Local(rs::Local {
        pat: rs::Pat::Ident(rs::PatIdent {
            ident: var,
            by_ref: false,
            subpat: None,
        }),
        init: Some(rs::LocalInit {
            expr: Box::new(rs::RustExpr::Try(Box::new(inner))),
            diverge: None,
        }),
    });
    Some((binding, stmt))
}

fn block(
    stmts: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    match stmts.first() {
        Some(rs::Stmt::Expr(rs::RustExpr::Continue(label), _)) => {
            return loop_exit(env, ctx, acc, *label, false);
        }
        Some(rs::Stmt::Expr(rs::RustExpr::Break(b), _)) if b.expr.is_none() => {
            return loop_exit(env, ctx, acc, b.label, true);
        }
        _ => {}
    }
    if let Some((first, rest)) = stmts.split_first() {
        if let Some(rewritten) = rewritten(first, env, ctx, &acc) {
            return block_rewritten(rewritten, rest, env, ctx, acc, want);
        }
    }
    match stmts {
        // A block with no value is `()`.
        [] if want == Some(&Ty::Unit) => Ok(((Expr::Flat("()".into()), Ty::Unit), acc)),
        // `x.m(..);`, `f(&mut x, ..);` and a block's last call: a call of a
        // function that threads `&mut` parameters, each place it lends
        // rebound to the value it gives back.
        [rs::Stmt::Expr(e, semi), rest @ ..]
            if (*semi || rest.is_empty()) && threaded_call(e, env, ctx).is_some() =>
        {
            let Some(call) = threaded_call(e, env, ctx) else {
                return Err(Residue::Statements);
            };
            threaded_stmt(call, None, !*semi, rest, env, ctx, acc, want)
        }
        // `let x = x.m(..);`: the same, with `x` bound to the call's result.
        [rs::Stmt::Local(l), rest @ ..]
            if threaded_let(l).is_some_and(|(_, _, e)| threaded_call(e, env, ctx).is_some()) =>
        {
            let (Some((binder, declared, e)), true) =
                (threaded_let(l), !rest.is_empty() || want == Some(&Ty::Unit))
            else {
                return Err(Residue::Statements);
            };
            let Some(call) = threaded_call(e, env, ctx) else {
                return Err(Residue::Statements);
            };
            let declared = match declared {
                Some(t) => Some(env.types.ty(&t.localized(&ctx.imports))?),
                None => None,
            };
            let bound = binder.map(|b| (b, declared));
            threaded_stmt(call, bound, false, rest, env, ctx, acc, want)
        }
        // The arms of a match and the branches of an if in tail position
        // are in tail position too: a `return` there leaves the function.
        [rs::Stmt::Expr(e @ (rs::RustExpr::Match(_) | rs::RustExpr::If(_)), false)] => {
            expr(e, env, ctx, acc, want)
        }
        [rs::Stmt::Expr(e, false)] if !(ctx.top && matches!(e, rs::RustExpr::Return(_))) => {
            below(e, env, ctx, acc, want)
        }
        // `let P = match s { .., _ => return v }; rest`: the match, with
        // the binding and the rest in each arm that does not leave.
        [rs::Stmt::Local(l), rest @ ..] if ctx.top && leaving_match(l).is_some() => {
            leaving_let(l, rest, env, ctx, acc, want)
        }
        [rs::Stmt::Expr(rs::RustExpr::Return(r), _), ..] if ctx.top => {
            early_return(r.as_deref(), env, ctx, acc)
        }
        // `if c { return v; } rest`: an early exit from the function.
        [rs::Stmt::Expr(rs::RustExpr::If(i), _), rest @ ..]
            if ctx.top
                && !rest.is_empty()
                && i.else_branch.is_none()
                && returned(&i.then_branch).is_some() =>
        {
            early_if(i, rest, env, ctx, acc, want)
        }
        // `let P = e else { return v; }; rest`: a match with an early exit.
        [rs::Stmt::Local(local), rest @ ..]
            if ctx.top
                && !rest.is_empty()
                && local.init.as_ref().is_some_and(|i| i.diverge.is_some()) =>
        {
            let_else(local, rest, env, ctx, acc, want)
        }
        // `let (a, b) = e; rest`: the tuple, then its items.
        [rs::Stmt::Local(local), rest @ ..]
            if !rest.is_empty()
                && matches!(local.pat, rs::Pat::Tuple(_))
                && local.init.as_ref().is_some_and(|i| {
                    i.diverge.is_none() && !matches!(*i.expr, rs::RustExpr::Try(_))
                }) =>
        {
            let_tuple(local, rest, env, ctx, acc, want)
        }
        // `for p in xs { .. }`: a walk with the updated locals as its state.
        [rs::Stmt::Expr(rs::RustExpr::ForLoop(f), _), rest @ ..] if !rest.is_empty() => {
            for_loop(f, rest, env, ctx, acc, want)
        }
        // `while i < n { .. }`: a walk measured by `n - i`.
        [rs::Stmt::Expr(rs::RustExpr::While(w), _), rest @ ..] if !rest.is_empty() => {
            while_loop(w, rest, env, ctx, acc, want)
        }
        // `x = e;`, `x += s;`, `x.push(e);`: the local's next value.
        [rs::Stmt::Expr(e, true), rest @ ..] if !rest.is_empty() && update_target(e).is_some() => {
            update_stmt(e, rest, env, ctx, acc, want)
        }
        // `()` as a statement does nothing, nor does a local's name: what
        // `x?;` leaves once its `?` is bound.
        [rs::Stmt::Expr(rs::RustExpr::Tuple(t), _), rest @ ..]
            if t.is_empty() && !rest.is_empty() =>
        {
            block(rest, env, ctx, acc, want)
        }
        [rs::Stmt::Expr(rs::RustExpr::Path(p), true), rest @ ..]
            if !rest.is_empty() && p.get_ident().is_some_and(|i| ctx.locals.contains_key(i)) =>
        {
            block(rest, env, ctx, acc, want)
        }
        // `match e { p => .. } rest`: the rest after each arm.
        [rs::Stmt::Expr(rs::RustExpr::Match(m), _), rest @ ..] if !rest.is_empty() => {
            match_rest(m, rest, env, ctx, acc, want)
        }
        // `if let p = e { .. } [else { .. }] rest`: the rest after each branch.
        [rs::Stmt::Expr(rs::RustExpr::If(i), _), rest @ ..]
            if !rest.is_empty()
                && matches!(&*i.cond, rs::RustExpr::Let(_))
                && returned(&i.then_branch).is_none() =>
        {
            if_let_rest(i, rest, env, ctx, acc, want)
        }
        // `if c { .. } [else { .. }] rest` with statements in its branches: the
        // rest after each branch.
        [rs::Stmt::Expr(rs::RustExpr::If(i), _), rest @ ..]
            if !rest.is_empty()
                && !matches!(&*i.cond, rs::RustExpr::Let(_))
                && returned(&i.then_branch).is_none() =>
        {
            if_rest(i, rest, env, ctx, acc, want)
        }
        [rs::Stmt::Local(local), rest @ ..] if !rest.is_empty() => {
            let_stmt(local, rest, env, ctx, acc, want)
        }
        // `x.c.set(v);`, `x.c.borrow_mut().push(v);`: a change of a cell
        // field, which keeps its own row.
        [rs::Stmt::Expr(rs::RustExpr::MethodCall(m), true), ..] => {
            let mut call = m;
            loop {
                if let Some(Err(row)) = cell_method(call, env, ctx, &acc).map(|r| r(acc.clone())) {
                    return Err(row);
                }
                match &*call.receiver {
                    rs::RustExpr::MethodCall(inner) => call = inner,
                    _ => return Err(Residue::Statements),
                }
            }
        }
        _ => Err(Residue::Statements),
    }
}

/// A statement the block reads as others: `m.entry(k).or_default().push(v);`
/// as the entry read, updated and written back, and, among the function's
/// own statements, a `?` hoisted to a binding of its own. With them, the
/// fresh binder the rewrite takes.
fn rewritten(first: &rs::Stmt, env: &Env, ctx: &Ctx, acc: &Acc) -> Option<(Vec<rs::Stmt>, String)> {
    let n = acc.given.len() + 1;
    if let Some(written) = entry_update(first, n) {
        return Some((written, format!("entry{n}")));
    }
    if !ctx.top {
        return None;
    }
    // Unpack the callee's state before matching its fallible return value.
    // Otherwise the failure branch would return the caller's old state.
    if let rs::Stmt::Local(local) = first {
        if let Some(init) = &local.init {
            if let rs::RustExpr::Try(inner) = &*init.expr {
                if init.diverge.is_none() && threaded_call(inner, env, ctx).is_some() {
                    let mut name = format!("propagated{n}");
                    while ctx.locals.contains_key(&name)
                        || env.fns.contains_key(&name)
                        || ctx.taken(env, &name)
                        || acc.given.contains(&name)
                    {
                        name.push('_');
                    }
                    let binding = rs::Stmt::Local(rs::Local {
                        pat: rs::Pat::Ident(rs::PatIdent {
                            ident: name.clone(),
                            by_ref: false,
                            subpat: None,
                        }),
                        init: Some(rs::LocalInit {
                            expr: inner.clone(),
                            diverge: None,
                        }),
                    });
                    let mut next = local.clone();
                    if let Some(init) = &mut next.init {
                        *init.expr =
                            rs::RustExpr::Try(Box::new(rs::RustExpr::Path(rs::Path::ident(&name))));
                    }
                    return Some((vec![binding, rs::Stmt::Local(next)], name));
                }
            }
        }
    }
    let (binding, stmt) = hoist_try(first, n)?;
    Some((vec![binding, stmt], format!("tried{n}")))
}

/// The block with its first statement rewritten, the rewrite's binder taken.
fn block_rewritten(
    rewritten: (Vec<rs::Stmt>, String),
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let (mut out, binder) = rewritten;
    let mut given = acc.given;
    given.push(binder);
    let acc = Acc { given, ..acc };
    out.extend(rest.iter().cloned());
    block(&out, env, ctx, acc, want)
}

/// An expression below the function's own statements.
fn below(
    e: &rs::RustExpr,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    expr(e, env, &ctx.below(), acc, want)
}

/// `let P = match s { .., _ => return v }; rest`: the match, with the
/// binding and the rest in each arm that does not leave.
fn leaving_let(
    l: &rs::Local,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let Some(m) = leaving_match(l) else {
        return Err(Residue::Statements);
    };
    let arms = m
        .arms
        .iter()
        .map(|a| {
            let body = if leaves(&a.body) {
                (*a.body).clone()
            } else {
                let mut stmts = vec![rs::Stmt::Local(rs::Local {
                    pat: l.pat.clone(),
                    init: Some(rs::LocalInit {
                        expr: a.body.clone(),
                        diverge: None,
                    }),
                })];
                stmts.extend(rest.iter().cloned());
                rs::RustExpr::Block(rs::ExprBlock {
                    label: false,
                    block: rs::Block { stmts },
                })
            };
            rs::MatchArm {
                pat: a.pat.clone(),
                body: Box::new(body),
            }
        })
        .collect();
    let whole = rs::RustExpr::Match(rs::ExprMatch {
        expr: m.expr.clone(),
        arms,
    });
    expr(&whole, env, ctx, acc, want)
}

/// `return v` among the function's own statements: the function's value.
fn early_return(
    r: Option<&rs::RustExpr>,
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
) -> Lift<((Expr, Ty), Acc)> {
    match r {
        Some(v) => {
            let (value, acc) = expr(v, env, &ctx.below(), acc, Some(&ctx.result))?;
            Ok((ctx.returning(value), acc))
        }
        None if ctx.result == Ty::Unit => {
            Ok((ctx.returning((Expr::Flat("()".into()), Ty::Unit)), acc))
        }
        None => Err(Residue::Type("()".to_string())),
    }
}

/// `if c { return v; } rest`: an early exit from the function.
fn early_if(
    i: &rs::ExprIf,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let Some(v) = returned(&i.then_branch) else {
        return Err(Residue::Statements);
    };
    let inner = ctx.below();
    if matches!(&*i.cond, rs::RustExpr::Let(_)) {
        let wild = rs::Pat::Wild;
        let rs::RustExpr::Let(l) = &*i.cond else {
            return Err(Residue::Statements);
        };
        let tail = rs::Block {
            stmts: rest.to_vec(),
        };
        let early = rs::Block {
            stmts: vec![rs::Stmt::Expr(leaving_value(v, ctx), false)],
        };
        let arms = vec![(&l.pat, Body::Block(&early)), (&wild, Body::Block(&tail))];
        return matching(&l.expr, &arms, env, ctx, acc, want);
    }
    let ((c, ct), acc) = flat_of(&i.cond, env, &inner, acc, Some(&Ty::Bool))?;
    if ct != Ty::Bool {
        return Err(Residue::Form("non-boolean condition".to_string()));
    }
    let (value, acc) = expr(v, env, &inner, acc, Some(&ctx.result))?;
    if !value.1.same(&ctx.result) {
        return Err(Residue::Form("branches of different types".to_string()));
    }
    let (then, tt) = ctx.returning(value);
    let after = ctx.assuming(&i.cond, false, env, &acc);
    let ((other, ot), acc) = block(rest, env, &after, acc, want)?;
    if !ot.same(&tt) {
        return Err(Residue::Form("branches of different types".to_string()));
    }
    Ok((
        (
            Expr::If {
                condition: Box::new(Expr::Flat(c)),
                then: Box::new(then),
                otherwise: Box::new(other),
            },
            tt,
        ),
        acc,
    ))
}

/// `let P = e else { return v; }; rest`: a match with an early exit.
fn let_else(
    local: &rs::Local,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let Some(init) = &local.init else {
        return Err(Residue::Statements);
    };
    let Some(otherwise) = &init.diverge else {
        return Err(Residue::Statements);
    };
    let rs::RustExpr::Block(eb) = &**otherwise else {
        return Err(Residue::Form("let else".to_string()));
    };
    // A `continue` or `break` in a loop step leaves the step.
    let skips = ctx.looping.is_some()
        && matches!(
            eb.block.stmts.as_slice(),
            [rs::Stmt::Expr(
                rs::RustExpr::Continue(_) | rs::RustExpr::Break(_),
                _
            )]
        );
    let early = match returned(&eb.block) {
        Some(v) => rs::Block {
            stmts: vec![rs::Stmt::Expr(leaving_value(v, ctx), false)],
        },
        None if skips => eb.block.clone(),
        None => return Err(Residue::Form("let else".to_string())),
    };
    let wild = rs::Pat::Wild;
    let tail = rs::Block {
        stmts: rest.to_vec(),
    };
    let arms = vec![
        (&local.pat, Body::Block(&tail)),
        (&wild, Body::Block(&early)),
    ];
    matching(&init.expr, &arms, env, ctx, acc, want)
}

/// `let (a, b) = e; rest`: the tuple, then its items.
fn let_tuple(
    local: &rs::Local,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let Some(init) = &local.init else {
        return Err(Residue::Statements);
    };
    let ((value, vt), acc) = flat_of(&init.expr, env, &ctx.below(), acc, None)?;
    let (scope, whole, acc, lets) = bind_pat(&local.pat, &vt, env, ctx, acc, Vec::new())?;
    let ((body, bt), acc) = block(rest, env, &scope, acc, want)?;
    let body = lets
        .into_iter()
        .rev()
        .fold(body, |body, (name, ty, value)| Expr::Let {
            name,
            ty,
            value: Box::new(Expr::Flat(value)),
            body: Box::new(body),
        });
    Ok((
        (
            Expr::Let {
                name: whole,
                ty: vt.render(),
                value: Box::new(Expr::Flat(value)),
                body: Box::new(body),
            },
            bt,
        ),
        acc,
    ))
}

/// `x = e;`, `x += s;`, `x.push(e);`: the local's next value.
fn update_stmt(
    e: &rs::RustExpr,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let Some(target) = update_target(e) else {
        return Err(Residue::Statements);
    };
    let Some((current, ty)) = ctx.locals.get(&target).cloned() else {
        return Err(Residue::Form(format!("an update of {target}")));
    };
    ctx.unsealed(&target)?;
    let (value, acc) = updated(e, &ty, &current, env, ctx, acc)?;
    let (scope, name, acc) = ctx.bind(env, acc, &target, ty.clone());
    let ((rest, rt), acc) = block(rest, env, &scope, acc, want)?;
    Ok((
        (
            Expr::Let {
                name,
                ty: ty.render(),
                value: Box::new(Expr::Flat(value)),
                body: Box::new(rest),
            },
            rt,
        ),
        acc,
    ))
}

/// The names the `ref` patterns of these statements bind, anywhere in them.
fn ref_binders(stmts: &[rs::Stmt]) -> BTreeSet<String> {
    fn pat(p: &rs::Pat, out: &mut BTreeSet<String>) {
        match p {
            rs::Pat::Ident(i) => {
                if i.by_ref {
                    out.insert(i.ident.clone());
                }
                if let Some(sub) = &i.subpat {
                    pat(sub, out);
                }
            }
            rs::Pat::Reference(inner) => pat(inner, out),
            rs::Pat::Type(pt) => pat(&pt.pat, out),
            rs::Pat::Tuple(ps) | rs::Pat::Slice(ps) | rs::Pat::Or(ps) => {
                ps.iter().for_each(|p| pat(p, out));
            }
            rs::Pat::TupleStruct(ts) => ts.elems.iter().for_each(|p| pat(p, out)),
            rs::Pat::Struct(ps) => ps.fields.iter().for_each(|f| pat(&f.pat, out)),
            rs::Pat::Guard(g) => pat(&g.pat, out),
            rs::Pat::Wild | rs::Pat::Rest | rs::Pat::Lit(_) | rs::Pat::Range(_) => {}
            rs::Pat::Path(_) | rs::Pat::Other => {}
        }
    }
    fn expr(e: &rs::RustExpr, out: &mut BTreeSet<String>) {
        match e {
            rs::RustExpr::Match(m) => m.arms.iter().for_each(|a| pat(&a.pat, out)),
            rs::RustExpr::Let(l) => pat(&l.pat, out),
            rs::RustExpr::ForLoop(f) => pat(&f.pat, out),
            rs::RustExpr::Closure(c) => c.inputs.iter().for_each(|p| pat(p, out)),
            rs::RustExpr::Macro(m) => {
                if let Some(mm) = &m.matches {
                    pat(&mm.pat, out);
                }
            }
            _ => {}
        }
        let (exprs, blocks) = e.parts();
        exprs.into_iter().for_each(|e| expr(e, out));
        blocks.into_iter().for_each(|b| walk(&b.stmts, out));
    }
    fn walk(ss: &[rs::Stmt], out: &mut BTreeSet<String>) {
        for s in ss {
            match s {
                rs::Stmt::Local(l) => {
                    pat(&l.pat, out);
                    if let Some(init) = &l.init {
                        expr(&init.expr, out);
                        if let Some(d) = &init.diverge {
                            expr(d, out);
                        }
                    }
                }
                rs::Stmt::Expr(e, _) => expr(e, out),
                rs::Stmt::Macro(_) | rs::Stmt::Other => {}
            }
        }
    }
    let mut out = BTreeSet::new();
    walk(stmts, &mut out);
    out
}

/// Why a cell field is not read where it stands: as itself, or by a
/// method other than the one that reads its content (`method`).
fn cell_unread(kind: CellKind, owner: &str, field: &str, method: Option<&str>) -> Residue {
    let what = match method {
        Some(m) => format!("{}::{m} on {owner}.{field}", kind.name()),
        None => format!("the {} field {owner}.{field} as itself", kind.name()),
    };
    Residue::Form(match kind.read() {
        Some(read) => format!(
            "{what}: the lift reads a {} only by {read}, and changes none, since another \
             borrow of the value would see the change",
            kind.name()
        ),
        None => format!(
            "{what}: a {} is shared across threads, and another may change it between two \
             reads",
            kind.name()
        ),
    })
}

/// The content a cell field is made with: `Kind::new(v)` is `v`.
fn cell_made<'e>(
    e: &'e rs::RustExpr,
    kind: CellKind,
    owner: &str,
    field: &str,
) -> Lift<&'e rs::RustExpr> {
    if let rs::RustExpr::Call(c) = e {
        if let (rs::RustExpr::Path(p), [content]) = (&*c.func, c.args.as_slice()) {
            let names: Vec<&str> = p.segments.iter().map(|s| s.ident.as_str()).collect();
            if let [.., cell, "new"] = names.as_slice() {
                if *cell == kind.name() {
                    return Ok(content);
                }
            }
        }
    }
    Err(cell_unread(kind, owner, field, None))
}

/// `x.c.borrow()` of a `RefCell` field and `x.c.get()` of a `Cell` field:
/// the content (see [`CellKind::read`]); any other method of a cell field
/// is a typed row. `None` where the receiver is no cell field.
#[allow(clippy::type_complexity)]
fn cell_method<'a>(
    m: &'a rs::ExprMethodCall,
    env: &'a Env,
    ctx: &'a Ctx,
    acc: &Acc,
) -> Option<Box<dyn FnOnce(Acc) -> Lift<((Expr, Ty), Acc)> + 'a>> {
    let mut receiver = &*m.receiver;
    while let rs::RustExpr::Paren(inner) = receiver {
        receiver = inner;
    }
    let rs::RustExpr::Field(f) = receiver else {
        return None;
    };
    let field = match &f.member {
        rs::Member::Named(name) => local_name(name),
        rs::Member::Unnamed(position) => field_name(&position.to_string()),
    };
    let ((_, bt), _) = flat_of(&f.base, env, ctx, acc.clone(), None).ok()?;
    let Ty::Named(owner) = bt else { return None };
    let kind = env.types.cell(&owner, &field)?;
    Some(Box::new(move |acc: Acc| {
        if kind.read() != Some(m.method.as_str()) || !m.args.is_empty() {
            return Err(cell_unread(kind, &owner, &field, Some(&m.method)));
        }
        let ((base, _), acc) = flat_of(&f.base, env, ctx, acc, None)?;
        let Some((_, ty)) = env
            .types
            .structs
            .get(&owner)
            .and_then(|fs| fs.iter().find(|(n, _)| *n == field))
        else {
            return Err(Residue::Form("field of an unknown struct".to_string()));
        };
        let read = if env.types.boxed(&owner) {
            format!("({} {base})", Types::accessor(&owner, &field))
        } else {
            format!("(project {base} {field})")
        };
        flat_lift(read, ty.clone(), acc)
    }))
}

/// Whether a pattern of a struct binds a field of it that is a cell: the
/// binder would be the cell, which the lift reads only by its method.
fn binds_cell(p: &rs::Pat, ty: &Ty, types: &Types) -> bool {
    let Ty::Named(n) = ty else { return false };
    let Some(cells) = types.cells.get(n) else {
        return false;
    };
    let bound = |p: &rs::Pat| !matches!(p, rs::Pat::Wild | rs::Pat::Rest);
    match p {
        rs::Pat::Struct(ps) => ps.fields.iter().any(|f| {
            let name = match &f.member {
                rs::Member::Named(m) => local_name(m),
                rs::Member::Unnamed(i) => field_name(&i.to_string()),
            };
            cells.contains_key(&name) && bound(&f.pat)
        }),
        rs::Pat::TupleStruct(ts) => ts.elems.iter().any(bound),
        _ => false,
    }
}

/// Why a call of a function that threads `&mut` parameters is not read
/// where it stands: as an operand, nothing writes back what it gives back.
fn threaded_operand(key: &str) -> Residue {
    Residue::Form(format!(
        "a call of {key}, which changes what it borrows mutably, as an operand"
    ))
}

/// A place a statement lends mutably: a local, or a named field of one.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Place {
    local: String,
    field: Option<String>,
}

impl Place {
    /// The place an expression names, read through `&mut`, `*` and
    /// parentheses: `x`, `&mut x`, `&mut *x`, `x.f`, `&mut x.f`.
    fn of(e: &rs::RustExpr) -> Option<Place> {
        match e {
            rs::RustExpr::Paren(inner) => Place::of(inner),
            rs::RustExpr::Reference(r) if r.mutable => Place::of(&r.expr),
            rs::RustExpr::Unary(u) if matches!(u.op, rs::UnOp::Deref) => match &*u.expr {
                rs::RustExpr::Path(p) => p.get_ident().map(|l| Place {
                    local: l.clone(),
                    field: None,
                }),
                _ => None,
            },
            rs::RustExpr::Path(p) => p.get_ident().map(|l| Place {
                local: l.clone(),
                field: None,
            }),
            rs::RustExpr::Field(f) => match (&*f.base, &f.member) {
                (rs::RustExpr::Path(p), rs::Member::Named(m)) => p.get_ident().map(|l| Place {
                    local: l.clone(),
                    field: Some(m.clone()),
                }),
                _ => None,
            },
            _ => None,
        }
    }

    /// The place as an expression that reads it.
    fn read(&self) -> rs::RustExpr {
        let local = rs::RustExpr::Path(rs::Path::ident(&self.local));
        match &self.field {
            None => local,
            Some(m) => rs::RustExpr::Field(rs::ExprField {
                base: Box::new(local),
                member: rs::Member::Named(m.clone()),
            }),
        }
    }

    /// The new value of the place's local once the place holds `value`:
    /// `value` itself, or the structure with that field replaced.
    fn written(&self, value: &str, env: &Env, ctx: &Ctx) -> Lift<String> {
        let Some((current, ty)) = ctx.locals.get(&self.local) else {
            return Err(Residue::Form(format!("an update of {}", self.local)));
        };
        if self.field.is_none() {
            return Ok(value.to_string());
        }
        let Ty::Named(name) = ty else {
            return Err(Residue::Form(format!(
                "an update of a field of {}",
                self.local
            )));
        };
        self.replaced(name, current, value, env)
    }

    /// The type of the place's field in the structure `name`, and its value
    /// in the structure's value `current`.
    fn field_of(&self, name: &str, current: &str, env: &Env) -> Option<(Ty, String)> {
        let target = local_name(self.field.as_ref()?);
        let fields = env.types.structs.get(name)?;
        let (_, ty) = fields.iter().find(|(f, _)| *f == target)?;
        let read = if env.types.boxed(name) {
            format!("({} {current})", Types::accessor(name, &target))
        } else {
            format!("(project {current} {target})")
        };
        Some((ty.clone(), read))
    }

    /// The structure `name` of value `current` with the place's field
    /// replaced by `value`.
    fn replaced(&self, name: &str, current: &str, value: &str, env: &Env) -> Lift<String> {
        let Some(member) = &self.field else {
            return Ok(value.to_string());
        };
        let Some(fields) = env.types.structs.get(name) else {
            return Err(Residue::Form(format!("a field update of {name}")));
        };
        let target = local_name(member);
        if !fields.iter().any(|(f, _)| *f == target) {
            return Err(Residue::Form(format!("a field update of {name}")));
        }
        let parts: Vec<(&String, String)> = fields
            .iter()
            .map(|(field, _)| {
                let v = if *field == target {
                    value.to_string()
                } else if env.types.boxed(name) {
                    format!("({} {current})", Types::accessor(name, field))
                } else {
                    format!("(project {current} {field})")
                };
                (field, v)
            })
            .collect();
        Ok(struct_value(name, &parts, &env.types))
    }
}

/// A call of a function that threads `&mut` parameters: its key, its
/// signature, its arguments with the receiver first, and the place each
/// threaded one lends.
struct Threaded<'e> {
    key: String,
    sig: Sig,
    args: Vec<&'e rs::RustExpr>,
    places: Vec<Place>,
}

/// The call `e` is, where it calls a function that threads `&mut`
/// parameters and each threaded argument lends a place: the receiver of a
/// method (which Rust borrows mutably on its own), `&mut p`, or a local
/// that is itself such a borrow (a threaded parameter, reborrowed).
fn threaded_call<'e>(e: &'e rs::RustExpr, env: &Env, ctx: &Ctx) -> Option<Threaded<'e>> {
    let (key, args): (String, Vec<&rs::RustExpr>) = match e {
        rs::RustExpr::MethodCall(m) => {
            let place = Place::of(&m.receiver)?;
            let (_, ty) = ctx.locals.get(&place.local)?;
            let ty = match &place.field {
                None => ty.clone(),
                Some(member) => {
                    let Ty::Named(name) = ty else { return None };
                    let target = local_name(member);
                    // A cell's methods are its own (see `cell_method`).
                    if env.types.cell(name, &target).is_some() {
                        return None;
                    }
                    let fields = env.types.structs.get(name)?;
                    fields.iter().find(|(f, _)| *f == target)?.1.clone()
                }
            };
            let Ty::Named(owner) = ty else { return None };
            let mut args = vec![&*m.receiver];
            args.extend(&m.args);
            (format!("{owner}::{}", m.method), args)
        }
        rs::RustExpr::Call(c) => {
            let rs::RustExpr::Path(p) = &*c.func else {
                return None;
            };
            let segments: Vec<String> = p.segments.iter().map(|s| s.ident.clone()).collect();
            (callee_key(&segments, env, ctx), c.args.iter().collect())
        }
        _ => return None,
    };
    let sig = env.fns.get(&key)?;
    if sig.threads.is_empty() || sig.params.len() != args.len() {
        return None;
    }
    let mut places = Vec::new();
    for &i in &sig.threads {
        let arg = args[i];
        // A method's receiver is a place Rust borrows on its own; an
        // argument lends one by `&mut`, or is a borrow already.
        let lends = i == 0 && matches!(e, rs::RustExpr::MethodCall(_))
            || matches!(arg, rs::RustExpr::Reference(r) if r.mutable)
            || matches!(arg, rs::RustExpr::Path(_));
        if !lends {
            return None;
        }
        places.push(Place::of(arg)?);
    }
    Some(Threaded {
        key,
        sig: sig.clone(),
        args,
        places,
    })
}

/// The key a call by path names: `Type::method` where the crate has one
/// (`Self` the type of the function in hand), else the last segment.
fn callee_key(segments: &[String], env: &Env, ctx: &Ctx) -> String {
    if let [.., owner, method] = segments {
        let owner = if owner == "Self" {
            ctx.current
                .split_once("::")
                .map_or(owner.as_str(), |(o, _)| o)
        } else {
            owner.as_str()
        };
        let key = format!("{}::{method}", env.types.resolve(owner));
        if env.fns.contains_key(&key) {
            return key;
        }
    }
    segments.last().cloned().unwrap_or_default()
}

/// A statement that calls a function threading `&mut` parameters: the
/// call, then each place it lent written back with the value the callee
/// gives back for it, in the order it gives them, then `bound` (the
/// `let`'s binder, if any) bound to the call's own result, then `rest`. A
/// call that ends the block (`tail`) is its own result.
#[allow(clippy::too_many_arguments)]
fn threaded_stmt(
    call: Threaded<'_>,
    bound: Option<(&str, Option<Ty>)>,
    tail: bool,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    if call.key == ctx.current {
        return Err(Residue::Form(
            "a function that threads its &mut parameters and calls itself".to_string(),
        ));
    }
    let full = call.sig.result.clone();
    let items = match &full {
        Ty::Tuple(ts) | Ty::Pair(ts) if ts.len() == call.places.len() + 1 => ts.clone(),
        _ => {
            return Err(Residue::Form(format!(
                "a call of {} of another type",
                call.key
            )))
        }
    };
    let mut acc = acc;
    let mut parts = vec![call.sig.name.clone()];
    for (i, (arg, ty)) in call.args.iter().zip(&call.sig.params).enumerate() {
        let read = match call.sig.threads.iter().position(|&t| t == i) {
            Some(j) => call.places[j].read(),
            None => (*arg).clone(),
        };
        let ((v, vt), next) = flat_of(&read, env, ctx, acc, Some(ty))?;
        acc = next;
        if !vt.same(ty) {
            return Err(Residue::Form(format!(
                "argument of another type to {}",
                call.key
            )));
        }
        parts.push(v);
    }
    let acc = acc.called(call.key.clone());
    let (whole, acc) = ctx.fresh(env, acc, "threaded");
    let mut lets: Vec<(String, String, String)> = vec![(
        whole.clone(),
        full.render(),
        format!("({})", parts.join(" ")),
    )];
    let mut scope = ctx.clone();
    let mut acc = acc;
    for (j, place) in call.places.iter().enumerate() {
        scope.unsealed(&place.local)?;
        let value = place.written(&tuple_item(&full, &whole, j + 1), env, &scope)?;
        let Some((_, ty)) = scope.locals.get(&place.local).cloned() else {
            return Err(Residue::Form(format!("an update of {}", place.local)));
        };
        let (next, name, after) = scope.bind(env, acc, &place.local, ty.clone());
        lets.push((name, ty.render(), value));
        scope = next;
        acc = after;
    }
    let result = tuple_item(&full, &whole, 0);
    let ((body, rt), acc) = if tail {
        ((Expr::Flat(result), items[0].clone()), acc)
    } else {
        match bound {
            Some((ident, declared)) => {
                if declared.as_ref().is_some_and(|d| !d.same(&items[0])) {
                    return Err(Residue::Form("let of another type".to_string()));
                }
                let (next, name, acc) = scope.bind(env, acc, ident, items[0].clone());
                lets.push((name, items[0].render(), result));
                block(rest, env, &next, acc, want)?
            }
            None => block(rest, env, &scope, acc, want)?,
        }
    };
    Ok(((wrap_lets(lets, body), rt), acc))
}

/// The statements of a body whose `&mut` parameters `states` are threaded,
/// rewritten so that every way out of it gives the tuple of its value and
/// their values at that point: each tail `v` and `return v` becomes
/// `{ let r = v; (r, s1, ..) }`, and a body of no value ends in
/// `((), s1, ..)`. The rewrite is refused where a state escapes as itself
/// rather than through its fields or a lifted call (`let s = self;`,
/// `match self { .. }`, `Some(self)`): there it is a second borrow of the
/// place, which the lift would read as a copy that changes apart from it.
fn thread_body(
    stmts: &[rs::Stmt],
    states: &[String],
    full: &Ty,
    env: &Env,
    current: &str,
) -> Lift<Vec<rs::Stmt>> {
    let unit = matches!(full, Ty::Tuple(ts) | Ty::Pair(ts) if ts.first() == Some(&Ty::Unit));
    let scan = StateScan {
        states,
        env,
        current,
    };
    for stmt in stmts {
        scan.stmt(stmt)?;
    }
    let mut result = "threaded".to_string();
    while states.contains(&result) {
        result.push('_');
    }
    let path = |n: &str| rs::RustExpr::Path(rs::Path::ident(n));
    let out = |value: Option<rs::RustExpr>| -> rs::RustExpr {
        let mut items = vec![path(&result)];
        items.extend(states.iter().map(|s| path(s)));
        let stmts = match value {
            None => {
                items[0] = rs::RustExpr::Tuple(Vec::new());
                vec![rs::Stmt::Expr(rs::RustExpr::Tuple(items), false)]
            }
            // A value of `()` is a statement, so that what it updates
            // reaches the tuple after it.
            Some(v) if unit => vec![
                rs::Stmt::Expr(v, true),
                rs::Stmt::Expr(
                    rs::RustExpr::Tuple({
                        items[0] = rs::RustExpr::Tuple(Vec::new());
                        items
                    }),
                    false,
                ),
            ],
            Some(v) => vec![
                rs::Stmt::Local(rs::Local {
                    pat: rs::Pat::Ident(rs::PatIdent {
                        ident: result.clone(),
                        by_ref: false,
                        subpat: None,
                    }),
                    init: Some(rs::LocalInit {
                        expr: Box::new(v),
                        diverge: None,
                    }),
                }),
                rs::Stmt::Expr(rs::RustExpr::Tuple(items), false),
            ],
        };
        rs::RustExpr::Block(rs::ExprBlock {
            label: false,
            block: rs::Block { stmts },
        })
    };
    let mut body = stmts.to_vec();
    for stmt in &mut body {
        returns_threaded(stmt, &out);
    }
    Ok(tails_threaded(body, &out))
}

/// The statements with their value threaded: the tail expression's own
/// tails, or `out(None)` after statements of no value.
fn tails_threaded(
    mut stmts: Vec<rs::Stmt>,
    out: &dyn Fn(Option<rs::RustExpr>) -> rs::RustExpr,
) -> Vec<rs::Stmt> {
    match stmts.pop() {
        Some(rs::Stmt::Expr(e, false)) => stmts.push(rs::Stmt::Expr(tail_threaded(e, out), false)),
        Some(other) => {
            stmts.push(other);
            stmts.push(rs::Stmt::Expr(out(None), false));
        }
        None => stmts.push(rs::Stmt::Expr(out(None), false)),
    }
    stmts
}

/// A tail expression with its value threaded at each of its own tails.
fn tail_threaded(
    e: rs::RustExpr,
    out: &dyn Fn(Option<rs::RustExpr>) -> rs::RustExpr,
) -> rs::RustExpr {
    match e {
        rs::RustExpr::Block(b) if !b.label => rs::RustExpr::Block(rs::ExprBlock {
            label: false,
            block: rs::Block {
                stmts: tails_threaded(b.block.stmts, out),
            },
        }),
        rs::RustExpr::Paren(inner) => tail_threaded(*inner, out),
        rs::RustExpr::If(i) => {
            let then_branch = rs::Block {
                stmts: tails_threaded(i.then_branch.stmts, out),
            };
            let else_branch = match i.else_branch {
                Some(e) => tail_threaded(*e, out),
                None => out(None),
            };
            rs::RustExpr::If(rs::ExprIf {
                cond: i.cond,
                then_branch,
                else_branch: Some(Box::new(else_branch)),
            })
        }
        rs::RustExpr::Match(m) => rs::RustExpr::Match(rs::ExprMatch {
            expr: m.expr,
            arms: m
                .arms
                .into_iter()
                .map(|a| rs::MatchArm {
                    pat: a.pat,
                    body: Box::new(tail_threaded(*a.body, out)),
                })
                .collect(),
        }),
        // A `return` already gives the tuple (see `returns_threaded`).
        rs::RustExpr::Return(r) => rs::RustExpr::Return(r),
        other => out(Some(other)),
    }
}

/// Each `return v` of a statement, outside the closures it holds, as
/// `return out(v)`.
fn returns_threaded(stmt: &mut rs::Stmt, out: &dyn Fn(Option<rs::RustExpr>) -> rs::RustExpr) {
    match stmt {
        rs::Stmt::Local(l) => {
            if let Some(init) = &mut l.init {
                return_threaded(&mut init.expr, out);
                if let Some(d) = &mut init.diverge {
                    return_threaded(d, out);
                }
            }
        }
        rs::Stmt::Expr(e, _) => return_threaded(e, out),
        rs::Stmt::Macro(_) | rs::Stmt::Other => {}
    }
}

fn return_threaded(e: &mut rs::RustExpr, out: &dyn Fn(Option<rs::RustExpr>) -> rs::RustExpr) {
    if let rs::RustExpr::Return(r) = e {
        let value = r.take().map(|v| {
            let mut v = *v;
            return_threaded(&mut v, out);
            v
        });
        *r = Some(Box::new(out(value)));
        return;
    }
    let each = |e: &mut rs::RustExpr| return_threaded(e, out);
    let blocks = |b: &mut rs::Block| {
        for s in &mut b.stmts {
            returns_threaded(s, out);
        }
    };
    each_part(e, each, blocks);
}

/// Every expression and block directly inside `e`, as
/// [`rs::RustExpr::parts`] lists them, outside a closure's body: what a
/// `return` there leaves is the closure.
fn each_part(
    e: &mut rs::RustExpr,
    mut each: impl FnMut(&mut rs::RustExpr),
    mut blocks: impl FnMut(&mut rs::Block),
) {
    match e {
        rs::RustExpr::Array(items) | rs::RustExpr::Tuple(items) => items.iter_mut().for_each(each),
        rs::RustExpr::Assign(a) => {
            each(&mut a.left);
            each(&mut a.right);
        }
        rs::RustExpr::Binary(b) => {
            each(&mut b.left);
            each(&mut b.right);
        }
        rs::RustExpr::Block(b) => blocks(&mut b.block),
        rs::RustExpr::Break(b) => {
            if let Some(v) = &mut b.expr {
                each(v);
            }
        }
        rs::RustExpr::Call(c) => {
            each(&mut c.func);
            c.args.iter_mut().for_each(each);
        }
        rs::RustExpr::Cast(c) => each(&mut c.expr),
        rs::RustExpr::Closure(_) => {}
        rs::RustExpr::Field(f) => each(&mut f.base),
        rs::RustExpr::ForLoop(f) => {
            each(&mut f.expr);
            blocks(&mut f.body);
        }
        rs::RustExpr::Loop(l) => blocks(&mut l.body),
        rs::RustExpr::While(w) => {
            each(&mut w.cond);
            blocks(&mut w.body);
        }
        rs::RustExpr::If(i) => {
            each(&mut i.cond);
            blocks(&mut i.then_branch);
            if let Some(e) = &mut i.else_branch {
                each(e);
            }
        }
        rs::RustExpr::Index(i) => {
            each(&mut i.expr);
            each(&mut i.index);
        }
        rs::RustExpr::Let(l) => each(&mut l.expr),
        rs::RustExpr::Match(m) => {
            each(&mut m.expr);
            for arm in &mut m.arms {
                if let rs::Pat::Guard(g) = &mut arm.pat {
                    each(&mut g.guard);
                }
                each(&mut arm.body);
            }
        }
        rs::RustExpr::MethodCall(m) => {
            each(&mut m.receiver);
            m.args.iter_mut().for_each(each);
        }
        rs::RustExpr::Paren(e) | rs::RustExpr::Try(e) => each(e),
        rs::RustExpr::Reference(r) => each(&mut r.expr),
        rs::RustExpr::Return(r) => {
            if let Some(v) = r {
                each(v);
            }
        }
        rs::RustExpr::Struct(s) => {
            for f in &mut s.fields {
                each(&mut f.expr);
            }
            if let Some(r) = &mut s.rest {
                each(r);
            }
        }
        rs::RustExpr::Unary(u) => each(&mut u.expr),
        rs::RustExpr::Other(o) => {
            o.exprs.iter_mut().for_each(&mut each);
            o.blocks.iter_mut().for_each(blocks);
        }
        rs::RustExpr::Continue(_)
        | rs::RustExpr::Lit(_)
        | rs::RustExpr::Macro(_)
        | rs::RustExpr::Path(_) => {}
    }
}

/// The macros whose arguments Rust only reads (each is formatted or
/// compared through a shared borrow).
const READING_MACROS: [&str; 12] = [
    "format",
    "write",
    "writeln",
    "print",
    "println",
    "eprint",
    "eprintln",
    "panic",
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
];

/// Where a threaded body's states are used as themselves.
struct StateScan<'a> {
    states: &'a [String],
    env: &'a Env,
    current: &'a str,
}

impl StateScan<'_> {
    /// The state `e` is, itself or through `&`, `&mut`, `*` or
    /// parentheses.
    fn state<'e>(&self, e: &'e rs::RustExpr) -> Option<&'e String> {
        match e {
            rs::RustExpr::Path(p) => p.get_ident().filter(|i| self.states.contains(i)),
            rs::RustExpr::Paren(inner) => self.state(inner),
            rs::RustExpr::Reference(r) => self.state(&r.expr),
            rs::RustExpr::Unary(u) if matches!(u.op, rs::UnOp::Deref) => self.state(&u.expr),
            _ => None,
        }
    }

    fn escapes(&self, state: &str) -> Residue {
        Residue::Form(format!(
            "{state} used as itself, a second borrow of the place it lends, which the lift \
             would read as a copy"
        ))
    }

    fn stmt(&self, stmt: &rs::Stmt) -> Lift<()> {
        match stmt {
            rs::Stmt::Local(l) => {
                if let Some(init) = &l.init {
                    self.operand(&init.expr)?;
                    if let Some(d) = &init.diverge {
                        self.operand(d)?;
                    }
                }
                Ok(())
            }
            rs::Stmt::Expr(e, _) => self.operand(e),
            rs::Stmt::Macro(m) => self.mac(m),
            rs::Stmt::Other => Ok(()),
        }
    }

    /// An expression in a position that takes its value.
    fn operand(&self, e: &rs::RustExpr) -> Lift<()> {
        if let Some(s) = self.state(e) {
            return Err(self.escapes(s));
        }
        self.within(e)
    }

    /// What is inside `e`: a state read through (a field's base, a method's
    /// receiver, an argument of a lifted function) is no escape.
    fn within(&self, e: &rs::RustExpr) -> Lift<()> {
        match e {
            rs::RustExpr::Field(f) if self.state(&f.base).is_some() => Ok(()),
            // `*s = v` writes the place itself.
            rs::RustExpr::Assign(a)
                if matches!(&*a.left, rs::RustExpr::Unary(u)
                    if matches!(u.op, rs::UnOp::Deref) && self.state(&u.expr).is_some()) =>
            {
                self.operand(&a.right)
            }
            rs::RustExpr::MethodCall(m) => {
                if self.state(&m.receiver).is_none() {
                    self.operand(&m.receiver)?;
                }
                m.args.iter().try_for_each(|a| self.operand(a))
            }
            rs::RustExpr::Call(c) => {
                let lifted = match &*c.func {
                    rs::RustExpr::Path(p) => {
                        let segments: Vec<String> =
                            p.segments.iter().map(|s| s.ident.clone()).collect();
                        self.lifted(&segments)
                    }
                    _ => false,
                };
                self.operand(&c.func)?;
                c.args.iter().try_for_each(|a| {
                    if lifted && self.state(a).is_some() {
                        Ok(())
                    } else {
                        self.operand(a)
                    }
                })
            }
            rs::RustExpr::Macro(m) => self.mac(m),
            _ => {
                let (exprs, blocks) = e.parts();
                for inner in exprs {
                    self.operand(inner)?;
                }
                for b in blocks {
                    for s in &b.stmts {
                        self.stmt(s)?;
                    }
                }
                Ok(())
            }
        }
    }

    /// A macro's arguments: read by a formatting or comparing macro, an
    /// operand of any other.
    fn mac(&self, m: &rs::ExprMacro) -> Lift<()> {
        let reads = READING_MACROS.iter().any(|r| m.path.is_ident(r));
        for item in m.args.iter().flat_map(|a| &a.items) {
            if reads && self.state(item).is_some() {
                continue;
            }
            self.operand(item)?;
        }
        if let Some(mm) = &m.matches {
            self.operand(&mm.expr)?;
        }
        Ok(())
    }

    /// Whether a call by these segments names a candidate function, which
    /// takes a state's value (or threads it) and returns no borrow of it.
    fn lifted(&self, segments: &[String]) -> bool {
        if let [.., owner, method] = segments {
            let owner = if owner == "Self" {
                self.current
                    .split_once("::")
                    .map_or(owner.as_str(), |(o, _)| o)
            } else {
                owner.as_str()
            };
            if self
                .env
                .fns
                .contains_key(&format!("{}::{method}", self.env.types.resolve(owner)))
            {
                return true;
            }
        }
        segments.len() == 1
            && segments
                .last()
                .is_some_and(|s| self.env.fns.contains_key(s))
    }
}

/// `let x = call;`, `let x: T = call;` or `let _ = call;` of a call the
/// block threads: the binder, its declared type and the call.
fn threaded_let(l: &rs::Local) -> Option<(Option<&str>, Option<&rs::RustType>, &rs::RustExpr)> {
    let init = l.init.as_ref().filter(|i| i.diverge.is_none())?;
    fn plain(p: &rs::Pat) -> Option<Option<&str>> {
        match p {
            rs::Pat::Ident(i) if i.subpat.is_none() && !i.by_ref => Some(Some(i.ident.as_str())),
            rs::Pat::Wild => Some(None),
            _ => None,
        }
    }
    let (binder, declared) = match &l.pat {
        rs::Pat::Type(pt) => (plain(&pt.pat)?, Some(&pt.ty)),
        p => (plain(p)?, None),
    };
    Some((binder, declared, &init.expr))
}

/// `match e { p => .. } rest`: the rest after each arm.
fn match_rest(
    m: &rs::ExprMatch,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let bodies: Vec<rs::RustExpr> = m.arms.iter().map(|a| then_rest(&a.body, rest)).collect();
    let arms: Vec<(&rs::Pat, Body<'_>)> = m
        .arms
        .iter()
        .zip(&bodies)
        .map(|(a, b)| match &a.pat {
            rs::Pat::Guard(g) => (&*g.pat, Body::Guarded(&g.guard, b)),
            p => (p, Body::Expr(b)),
        })
        .collect();
    matching(&m.expr, &arms, env, ctx, acc, want)
}

/// `if let p = e { .. } [else { .. }] rest`: the rest after each branch.
fn if_let_rest(
    i: &rs::ExprIf,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let rs::RustExpr::Let(l) = &*i.cond else {
        return Err(Residue::Statements);
    };
    if let Some(state) = branch_state(i, ctx) {
        let tail = [rs::Stmt::Expr(state_expr(&state), false)];
        let then = then_rest(
            &rs::RustExpr::Block(rs::ExprBlock {
                label: false,
                block: i.then_branch.clone(),
            }),
            &tail,
        );
        let unit = rs::RustExpr::Tuple(Vec::new());
        let other = then_rest(i.else_branch.as_deref().unwrap_or(&unit), &tail);
        let wild = rs::Pat::Wild;
        let arms = vec![(&l.pat, Body::Expr(&then)), (&wild, Body::Expr(&other))];
        let state_ty = state_type(&state, ctx);
        let (value, acc) = matching(&l.expr, &arms, env, &ctx.below(), acc, Some(&state_ty))?;
        return after_state(value, &state, rest, env, ctx, acc, want);
    }
    let then = then_rest(
        &rs::RustExpr::Block(rs::ExprBlock {
            label: false,
            block: i.then_branch.clone(),
        }),
        rest,
    );
    let unit = rs::RustExpr::Tuple(Vec::new());
    let other = then_rest(i.else_branch.as_deref().unwrap_or(&unit), rest);
    let wild = rs::Pat::Wild;
    let arms = vec![(&l.pat, Body::Expr(&then)), (&wild, Body::Expr(&other))];
    matching(&l.expr, &arms, env, ctx, acc, want)
}

/// `if c { .. } [else { .. }] rest` with statements in its branches: the
/// rest after each branch.
fn if_rest(
    i: &rs::ExprIf,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let ((c, ct), acc) = flat_of(&i.cond, env, &ctx.below(), acc, Some(&Ty::Bool))?;
    if ct != Ty::Bool {
        return Err(Residue::Form("non-boolean condition".to_string()));
    }
    if let Some(state) = branch_state(i, ctx) {
        let state_ty = state_type(&state, ctx);
        let tail = [rs::Stmt::Expr(state_expr(&state), false)];
        let then = settled(&i.then_branch.stmts, &tail);
        let other = match &i.else_branch {
            None => tail.to_vec(),
            Some(e) => match &**e {
                rs::RustExpr::Block(b) => settled(&b.block.stmts, &tail),
                rs::RustExpr::If(_) => settled(&[rs::Stmt::Expr((**e).clone(), false)], &tail),
                _ => return Err(Residue::Statements),
            },
        };
        let inner = ctx.below();
        let yes = inner.assuming(&i.cond, true, env, &acc);
        let no = inner.assuming(&i.cond, false, env, &acc);
        let ((a, at), acc) = block(&then, env, &yes, acc, Some(&state_ty))?;
        let ((b, bt), acc) = block(&other, env, &no, acc, Some(&state_ty))?;
        if !at.same(&state_ty) || !bt.same(&state_ty) {
            return Err(Residue::Form("branches of different types".to_string()));
        }
        let value = Expr::If {
            condition: Box::new(Expr::Flat(c)),
            then: Box::new(a),
            otherwise: Box::new(b),
        };
        return after_state((value, state_ty), &state, rest, env, ctx, acc, want);
    }
    let mut then: Vec<rs::Stmt> = i.then_branch.stmts.clone();
    then.extend(rest.iter().cloned());
    let mut other: Vec<rs::Stmt> = match &i.else_branch {
        None => Vec::new(),
        Some(e) => match &**e {
            rs::RustExpr::Block(b) => b.block.stmts.clone(),
            rs::RustExpr::If(_) => vec![rs::Stmt::Expr((**e).clone(), false)],
            _ => return Err(Residue::Statements),
        },
    };
    other.extend(rest.iter().cloned());
    let ((a, at), acc) = block(&then, env, ctx, acc, want)?;
    let ((b, bt), acc) = block(&other, env, ctx, acc, Some(want.unwrap_or(&at)))?;
    if !at.same(&bt) {
        return Err(Residue::Form("branches of different types".to_string()));
    }
    Ok((
        (
            Expr::If {
                condition: Box::new(Expr::Flat(c)),
                then: Box::new(a),
                otherwise: Box::new(b),
            },
            at,
        ),
        acc,
    ))
}

/// The locals the branches of an `if` that is a statement update, when
/// the branches only update them: no branch leaves the statement, and
/// none shadows a local it updates. The `if` is then the value of those
/// locals after it, and the statements after it lift once, in its scope.
fn branch_state(i: &rs::ExprIf, ctx: &Ctx) -> Option<Vec<String>> {
    let mut stmts = i.then_branch.stmts.clone();
    if let Some(e) = &i.else_branch {
        stmts.push(rs::Stmt::Expr((**e).clone(), true));
    }
    if leaves_step(&stmts) {
        return None;
    }
    let (updated, declared) = scan_stmts(&stmts);
    if updated.iter().any(|u| declared.contains(u)) {
        return None;
    }
    let state: Vec<String> = updated
        .into_iter()
        .filter(|u| ctx.locals.contains_key(u))
        .collect();
    (!state.is_empty()).then_some(state)
}

/// The type of the locals `state` names, as one value.
fn state_type(state: &[String], ctx: &Ctx) -> Ty {
    let tys: Vec<Ty> = state.iter().map(|v| ctx.locals[v].1.clone()).collect();
    match tys.as_slice() {
        [] => Ty::Unit,
        [one] => one.clone(),
        many => Ty::Tuple(many.to_vec()),
    }
}

/// A branch's statements, each a statement, then `tail`.
fn settled(stmts: &[rs::Stmt], tail: &[rs::Stmt]) -> Vec<rs::Stmt> {
    let mut out: Vec<rs::Stmt> = stmts
        .iter()
        .map(|s| match s {
            rs::Stmt::Expr(e, _) => rs::Stmt::Expr(e.clone(), true),
            other => other.clone(),
        })
        .collect();
    out.extend(tail.iter().cloned());
    out
}

/// The statements after a branching statement, in the scope of the state
/// it leaves: the state bound once, its locals rebound to its items.
fn after_state(
    value: (Expr, Ty),
    state: &[String],
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let (value, state_ty) = value;
    let acc = acc.saw(&state_ty);
    let (whole, acc) = ctx.fresh(env, acc, "state");
    let (after, acc, lets) = rebind(env, ctx, acc, state, &whole, Vec::new());
    let ((rest, rt), acc) = block(rest, env, &after, acc, want)?;
    Ok((
        (
            Expr::Let {
                name: whole,
                ty: state_ty.render(),
                value: Box::new(value),
                body: Box::new(wrap_lets(lets, rest)),
            },
            rt,
        ),
        acc,
    ))
}

/// `let x = e; rest`: the binding, then the rest in its scope.
fn let_stmt(
    local: &rs::Local,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let (ident, declared) = match &local.pat {
        rs::Pat::Ident(i) if i.subpat.is_none() && !i.by_ref => (i, None),
        rs::Pat::Type(pt) => match &*pt.pat {
            rs::Pat::Ident(i) if i.subpat.is_none() && !i.by_ref => {
                (i, Some(env.types.ty(&pt.ty.localized(&ctx.imports))?))
            }
            other => return let_pattern(other, local, rest, env, ctx, acc, want),
        },
        other => return let_pattern(other, local, rest, env, ctx, acc, want),
    };
    let Some(init) = &local.init else {
        return Err(Residue::Statements);
    };
    if init.diverge.is_some() {
        return let_pattern(&local.pat, local, rest, env, ctx, acc, want);
    }
    if let rs::RustExpr::Closure(c) = &*init.expr {
        if c.asyncness || declared.is_some() {
            return Err(Residue::Form("expr::closure".to_string()));
        }
        let mut scope = ctx.clone();
        scope.closures.insert(
            ident.ident.clone(),
            Closure {
                inputs: c.inputs.clone(),
                body: (*c.body).clone(),
                locals: ctx.locals.clone(),
            },
        );
        return block(rest, env, &scope, acc, want);
    }
    if let rs::RustExpr::Try(t) = &*init.expr {
        if !ctx.top {
            return Err(Residue::Form(
                "? below the function's own statements".to_string(),
            ));
        }
        return question(t, &ident.ident, declared, rest, env, ctx, acc, want);
    }
    // An empty vector the block goes on to return takes the type the
    // block returns.
    let hint = match &declared {
        Some(d) => Some(d.clone()),
        None if is_empty_vec(&init.expr) => returned_type(&ident.ident.clone(), rest, ctx, want)
            .or_else(|| used_type(&ident.ident, &init.expr, rest, env, ctx)),
        None => {
            returned_type(&ident.ident, rest, ctx, want).filter(|t| matches!(t, Ty::Outcome(..)))
        }
    };
    let ((value, vt), acc) = flat_of(&init.expr, env, &ctx.below(), acc, hint.as_ref())?;
    if declared.as_ref().is_some_and(|d| !d.same(&vt)) {
        return Err(Residue::Form("let of another type".to_string()));
    }
    let (scope, name, acc) = ctx.bind(env, acc, &ident.ident.clone(), vt.clone());
    let ((rest, rt), acc) = block(rest, env, &scope, acc, want)?;
    Ok((
        (
            Expr::Let {
                name,
                ty: vt.render(),
                value: Box::new(Expr::Flat(value)),
                body: Box::new(rest),
            },
            rt,
        ),
        acc,
    ))
}

/// `let P = e; rest` where `P` takes every value of `e`: a match of one arm.
fn let_pattern(
    pat: &rs::Pat,
    local: &rs::Local,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let Some(init) = &local.init else {
        return Err(Residue::Statements);
    };
    if matches!(&*init.expr, rs::RustExpr::Try(_)) {
        return Err(Residue::Form("let pattern".to_string()));
    }
    let tail = rs::Block {
        stmts: rest.to_vec(),
    };
    let wild = rs::Pat::Wild;
    // `let P = e else { .. };`: the block the other values take, such as
    // a `continue` of the loop the statements are a step of.
    let arms = match init.diverge.as_deref() {
        None => vec![(pat, Body::Block(&tail))],
        Some(rs::RustExpr::Block(b)) if !b.label => {
            vec![(pat, Body::Block(&tail)), (&wild, Body::Block(&b.block))]
        }
        Some(_) => return Err(Residue::Form("let else".to_string())),
    };
    matching(&init.expr, &arms, env, ctx, acc, want)
}

/// The value of an early exit, as the arm that takes it states it: the
/// value itself, or in a loop step a `return` of it, which leaves the walk.
fn leaving_value(v: &rs::RustExpr, ctx: &Ctx) -> rs::RustExpr {
    match ctx.exits {
        Some(_) => rs::RustExpr::Return(Some(Box::new(v.clone()))),
        None => v.clone(),
    }
}

/// Whether an arm's value leaves the function: a `return`, alone or as
/// the last statement of a block.
fn leaves(e: &rs::RustExpr) -> bool {
    match e {
        rs::RustExpr::Return(_) => true,
        rs::RustExpr::Block(b) if !b.label => matches!(
            b.block.stmts.last(),
            Some(rs::Stmt::Expr(rs::RustExpr::Return(_), _))
        ),
        _ => false,
    }
}

/// The match a `let` binds where an arm of it leaves the function.
fn leaving_match(l: &rs::Local) -> Option<&rs::ExprMatch> {
    let init = l.init.as_ref()?;
    if init.diverge.is_some() {
        return None;
    }
    match &*init.expr {
        rs::RustExpr::Match(m) if m.arms.iter().any(|a| leaves(&a.body)) => Some(m),
        _ => None,
    }
}

/// `Vec::new()` or `Vec::with_capacity(n)`: a vector whose element type the
/// expression does not say.
fn is_empty_vec(e: &rs::RustExpr) -> bool {
    let rs::RustExpr::Call(c) = e else {
        return false;
    };
    let rs::RustExpr::Path(p) = &*c.func else {
        return false;
    };
    let names: Vec<String> = p.segments.iter().map(|s| s.ident.to_string()).collect();
    matches!(
        (
            names
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .as_slice(),
            c.args.len()
        ),
        (["Vec" | "BTreeSet" | "BTreeMap", "new"], 0) | (["Vec", "with_capacity"], 1)
    )
}

/// How a later statement uses an empty collection: it adds an element,
/// an entry, or it is passed where a type is declared.
enum Use {
    Element(Ty),
    Entry(Ty, Ty),
    Whole(Ty),
}

/// The type an empty collection bound to `ident` takes from its first use
/// in the statements after it whose type the lifted signatures say: an
/// element `insert`ed, `push`ed or `extend`ed with, an entry `insert`ed,
/// or an argument of a lifted function's declared type.
fn used_type(
    ident: &str,
    init: &rs::RustExpr,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
) -> Option<Ty> {
    let rs::RustExpr::Call(c) = init else {
        return None;
    };
    let rs::RustExpr::Path(p) = &*c.func else {
        return None;
    };
    let kind = p.segments.first()?.ident.clone();
    let locals: BTreeMap<String, Ty> = ctx
        .locals
        .iter()
        .map(|(r, (_, t))| (r.clone(), t.clone()))
        .collect();
    match (kind.as_str(), use_in_stmts(ident, rest, env, &locals)?) {
        (_, Use::Whole(t)) => Some(t),
        ("Vec", Use::Element(t)) => Some(Ty::List(Box::new(t))),
        ("BTreeSet", Use::Element(t)) => Some(Ty::Set(Box::new(t), true)),
        ("BTreeMap", Use::Entry(k, v)) => Some(Ty::Map(Box::new(k), Box::new(v), true)),
        _ => None,
    }
}

/// [`used_type`] over statements, with the locals they bind in turn.
fn use_in_stmts(
    ident: &str,
    stmts: &[rs::Stmt],
    env: &Env,
    locals: &BTreeMap<String, Ty>,
) -> Option<Use> {
    let mut scope = locals.clone();
    for stmt in stmts {
        match stmt {
            rs::Stmt::Local(l) => {
                let init = l.init.as_ref()?;
                if let Some(u) = use_in(ident, &init.expr, env, &scope) {
                    return Some(u);
                }
                let ty = match &l.pat {
                    rs::Pat::Type(pt) => env.types.ty(&pt.ty).ok(),
                    _ => static_type(&init.expr, env, &scope),
                };
                scope = typed_binders(&l.pat, ty.as_ref(), scope);
            }
            rs::Stmt::Expr(e, _) => {
                if let Some(u) = use_in(ident, e, env, &scope) {
                    return Some(u);
                }
            }
            rs::Stmt::Macro(_) | rs::Stmt::Other => {}
        }
    }
    None
}

/// The locals with the binders of `pat` typed from `ty`, the type of the
/// value it matches: an identifier takes it, a tuple's items take its
/// items'. A binder of a type not known is dropped.
fn typed_binders(
    pat: &rs::Pat,
    ty: Option<&Ty>,
    locals: BTreeMap<String, Ty>,
) -> BTreeMap<String, Ty> {
    match (pat, ty) {
        (rs::Pat::Ident(i), Some(t)) if i.subpat.is_none() => {
            let mut locals = locals;
            locals.insert(i.ident.clone(), t.clone());
            locals
        }
        (rs::Pat::Ident(i), _) => locals
            .into_iter()
            .filter(|(name, _)| *name != i.ident)
            .collect(),
        (rs::Pat::Type(pt), _) => typed_binders(&pt.pat, ty, locals),
        (rs::Pat::Reference(inner), _) => typed_binders(inner, ty, locals),
        (rs::Pat::Tuple(items), Some(Ty::Tuple(ts) | Ty::Pair(ts))) if items.len() == ts.len() => {
            items
                .iter()
                .zip(ts)
                .fold(locals, |scope, (p, t)| typed_binders(p, Some(t), scope))
        }
        (rs::Pat::Tuple(items), _) => items
            .iter()
            .fold(locals, |scope, p| typed_binders(p, None, scope)),
        _ => locals,
    }
}

/// [`used_type`] in an expression.
fn use_in(ident: &str, e: &rs::RustExpr, env: &Env, locals: &BTreeMap<String, Ty>) -> Option<Use> {
    let named = |x: &rs::RustExpr| match x {
        rs::RustExpr::Reference(r) => {
            matches!(&*r.expr, rs::RustExpr::Path(p) if p.is_ident(ident))
        }
        rs::RustExpr::Path(p) => p.is_ident(ident),
        _ => false,
    };
    match e {
        rs::RustExpr::MethodCall(m) if named(&m.receiver) => {
            let ty = |x: &rs::RustExpr| static_type(x, env, locals);
            match (m.method.as_str(), m.args.as_slice()) {
                ("insert" | "push", [x]) => ty(x).map(Use::Element),
                ("insert", [k, v]) => Some(Use::Entry(ty(k)?, ty(v)?)),
                ("extend", [x]) => match ty(x)? {
                    Ty::List(t) | Ty::Set(t, _) => Some(Use::Element(*t)),
                    _ => None,
                },
                _ => None,
            }
        }
        rs::RustExpr::Call(c) => match c.args.iter().position(named) {
            Some(i) => {
                let rs::RustExpr::Path(p) = &*c.func else {
                    return None;
                };
                let sig = env.fns.get(&p.segments.last()?.ident)?;
                sig.params.get(i).cloned().map(Use::Whole)
            }
            None => c.args.iter().find_map(|a| use_in(ident, a, env, locals)),
        },
        rs::RustExpr::ForLoop(f) => {
            if let Some(u) = use_in(ident, &f.expr, env, locals) {
                return Some(u);
            }
            let mut inner = locals.clone();
            if let rs::Pat::Ident(i) = &f.pat {
                let elem = match static_type(&f.expr, env, locals) {
                    Some(Ty::List(t) | Ty::Set(t, _)) => Some(*t),
                    Some(m @ Ty::Map(..)) => Some(m.entry()),
                    _ => None,
                };
                match elem {
                    Some(t) => inner.insert(i.ident.clone(), t),
                    None => inner.remove(&i.ident),
                };
            }
            use_in_stmts(ident, &f.body.stmts, env, &inner)
        }
        rs::RustExpr::Closure(_) => None,
        other => {
            let (exprs, blocks) = other.parts();
            exprs
                .iter()
                .find_map(|x| use_in(ident, x, env, locals))
                .or_else(|| {
                    blocks
                        .iter()
                        .find_map(|b| use_in_stmts(ident, &b.stmts, env, locals))
                })
        }
    }
}

/// The type of an expression of locals, borrows, clones and calls of
/// lifted functions and methods, read from their signatures.
fn static_type(e: &rs::RustExpr, env: &Env, locals: &BTreeMap<String, Ty>) -> Option<Ty> {
    match e {
        rs::RustExpr::Path(p) => match p.get_ident() {
            Some(ident) => locals.get(ident).cloned(),
            None => variant_owner(p, env),
        },
        rs::RustExpr::Reference(r) => static_type(&r.expr, env, locals),
        rs::RustExpr::Paren(x) => static_type(x, env, locals),
        // `x?`: the value an option or result holds.
        rs::RustExpr::Try(x) => match static_type(x, env, locals)? {
            Ty::Option(t) | Ty::Outcome(t, _) => Some(*t),
            _ => None,
        },
        rs::RustExpr::MethodCall(m)
            if m.args.is_empty()
                && matches!(
                    m.method.as_str(),
                    "iter" | "into_iter" | "clone" | "to_owned" | "cloned"
                ) =>
        {
            static_type(&m.receiver, env, locals)
        }
        rs::RustExpr::MethodCall(m) if m.args.is_empty() && m.method == "to_string" => {
            Some(Ty::Str)
        }
        rs::RustExpr::MethodCall(m)
            if m.args.is_empty() && matches!(m.method.as_str(), "keys" | "values") =>
        {
            match static_type(&m.receiver, env, locals)? {
                Ty::Map(k, _, _) if m.method == "keys" => Some(Ty::List(k)),
                Ty::Map(_, v, _) => Some(Ty::List(v)),
                _ => None,
            }
        }
        rs::RustExpr::MethodCall(m) => match static_type(&m.receiver, env, locals)? {
            Ty::Named(owner) => env
                .fns
                .get(&format!("{owner}::{}", m.method))
                .map(|s| s.result.clone()),
            _ => None,
        },
        rs::RustExpr::Call(c) => match &*c.func {
            rs::RustExpr::Path(p) => {
                let last = &p.segments.last()?.ident;
                if let Some(sig) = env.fns.get(last) {
                    return Some(sig.result.clone());
                }
                match (last.as_str(), c.args.as_slice()) {
                    ("Some", [x]) => Some(Ty::Option(Box::new(static_type(x, env, locals)?))),
                    _ => variant_owner(p, env),
                }
            }
            _ => None,
        },
        // A struct literal is of the struct it names.
        rs::RustExpr::Struct(st) => {
            let name = env.types.resolve(&st.path.segments.last()?.ident);
            env.types
                .structs
                .contains_key(&name)
                .then_some(Ty::Named(name))
        }
        rs::RustExpr::Tuple(items) if items.len() > 1 => items
            .iter()
            .map(|x| static_type(x, env, locals))
            .collect::<Option<Vec<Ty>>>()
            .map(Ty::Tuple),
        rs::RustExpr::Lit(rs::Lit::Str(_)) => Some(Ty::Str),
        rs::RustExpr::Lit(rs::Lit::Bool(_)) => Some(Ty::Bool),
        rs::RustExpr::Macro(m) if m.path.is_ident("format") => Some(Ty::Str),
        rs::RustExpr::Field(f) => match (static_type(&f.base, env, locals)?, &f.member) {
            (Ty::Named(n), rs::Member::Named(m)) => env
                .types
                .structs
                .get(&n)?
                .iter()
                .find(|(field, _)| *field == local_name(m))
                .map(|(_, t)| t.clone()),
            (Ty::Tuple(ts), rs::Member::Unnamed(i)) => ts.get(usize::try_from(*i).ok()?).cloned(),
            _ => None,
        },
        _ => None,
    }
}

/// The enum a variant's path names: `Kind::Variant`, of the enum `Kind`
/// the lift declares.
fn variant_owner(p: &rs::Path, env: &Env) -> Option<Ty> {
    let owner = p.segments.iter().rev().nth(1)?;
    let name = env.types.resolve(&owner.ident);
    (env.types.datas.contains_key(&name) || env.types.enums.contains_key(&name))
        .then_some(Ty::Named(name))
}

/// The type a local takes from where the block returns it: the block's own
/// type when its last statement is the local, or the function's result when
/// a `return` of the local appears.
fn returned_type(ident: &str, rest: &[rs::Stmt], ctx: &Ctx, want: Option<&Ty>) -> Option<Ty> {
    if let (Some(rs::Stmt::Expr(e, false)), Some(w)) = (rest.last(), want) {
        if let Some(t) = tail_type(e, ident, w) {
            return Some(t);
        }
    }
    returns_local(rest, ident).then(|| ctx.result.clone())
}

/// The type of the local `ident` where a tail of type `want` holds it: the
/// tail itself, a member of a tuple, or the value of `Some`, `Ok` or
/// `then_some`.
fn tail_type(e: &rs::RustExpr, ident: &str, want: &Ty) -> Option<Ty> {
    let is_local = |e: &rs::RustExpr| matches!(e, rs::RustExpr::Path(p) if p.is_ident(ident));
    let inner = match want {
        Ty::Option(t) | Ty::Outcome(t, _) => Some((**t).clone()),
        _ => None,
    };
    match (e, want) {
        (rs::RustExpr::Path(p), _) if p.is_ident(ident) => Some(want.clone()),
        (rs::RustExpr::Tuple(items), Ty::Tuple(ts) | Ty::Pair(ts)) if items.len() == ts.len() => {
            items
                .iter()
                .zip(ts)
                .find(|(item, _)| is_local(item))
                .map(|(_, t)| t.clone())
        }
        (rs::RustExpr::Call(c), _) => match (&*c.func, c.args.as_slice()) {
            (rs::RustExpr::Path(p), [arg])
                if is_local(arg) && (p.is_ident("Some") || p.is_ident("Ok")) =>
            {
                inner
            }
            _ => None,
        },
        (rs::RustExpr::MethodCall(m), _) => match m.args.as_slice() {
            [arg] if m.method == "then_some" && is_local(arg) => inner,
            _ => None,
        },
        _ => None,
    }
}

/// Whether a `return` of the local `ident` appears in the statements,
/// outside closures.
fn returns_local(stmts: &[rs::Stmt], ident: &str) -> bool {
    stmts.iter().any(|stmt| match stmt {
        rs::Stmt::Local(l) => l.init.as_ref().is_some_and(|init| {
            let mut exprs = vec![&*init.expr];
            exprs.extend(init.diverge.as_deref());
            exprs.iter().any(|e| returns_in(e, ident))
        }),
        rs::Stmt::Expr(e, _) => returns_in(e, ident),
        rs::Stmt::Macro(_) | rs::Stmt::Other => false,
    })
}

/// [`returns_local`] in an expression.
fn returns_in(e: &rs::RustExpr, ident: &str) -> bool {
    if matches!(e, rs::RustExpr::Closure(_)) {
        return false;
    }
    if let rs::RustExpr::Return(Some(v)) = e {
        if matches!(&**v, rs::RustExpr::Path(p) if p.is_ident(ident)) {
            return true;
        }
    }
    let (exprs, blocks) = e.parts();
    exprs.iter().any(|x| returns_in(x, ident))
        || blocks.iter().any(|b| returns_local(&b.stmts, ident))
}

/// `let x = e?; rest`: a match that returns the failure and continues with
/// the value.
#[allow(clippy::too_many_arguments)]
fn question(
    e: &rs::RustExpr,
    ident: &str,
    declared: Option<Ty>,
    rest: &[rs::Stmt],
    env: &Env,
    ctx: &Ctx,
    acc: Acc,
    want: Option<&Ty>,
) -> Lift<((Expr, Ty), Acc)> {
    let ((s, st), mut acc) = flat_of(e, env, &ctx.below(), acc, None)?;
    let result = if ctx.return_states.is_empty() {
        &ctx.result
    } else {
        match &ctx.result {
            Ty::Tuple(items) | Ty::Pair(items) if items.len() == ctx.return_states.len() + 1 => {
                &items[0]
            }
            _ => {
                return Err(Residue::Form(
                    "? with an incomplete threaded result".to_string(),
                ))
            }
        }
    };
    let (inner, early) = match (&st, result) {
        (Ty::Option(inner), Ty::Option(r)) => (
            (**inner).clone(),
            (
                "Option.none".to_string(),
                format!("(ctor Option.none [{}])", r.render()),
            ),
        ),
        (Ty::Outcome(inner, err), Ty::Outcome(ok, rerr)) if err.same(rerr) => {
            let (e, after) = ctx.fresh(env, acc, "error");
            acc = after;
            (
                (**inner).clone(),
                (
                    format!("Result.error {e}"),
                    format!(
                        "(ctor Result.error [{}, {}] {e})",
                        ok.render(),
                        rerr.render()
                    ),
                ),
            )
        }
        _ => return Err(Residue::Form("? of another type".to_string())),
    };
    if declared.as_ref().is_some_and(|d| !d.same(&inner)) {
        return Err(Residue::Form("let of another type".to_string()));
    }
    let (scope, name, acc) = ctx.bind(env, acc, ident, inner.clone());
    let ((rest, rt), acc) = block(rest, env, &scope, acc, want)?;
    let ok = if matches!(st, Ty::Option(_)) {
        format!("Option.some {name}")
    } else {
        format!("Result.ok {name}")
    };
    let failure = if ctx.return_states.is_empty() {
        early.1
    } else {
        let mut items = vec![early.1];
        for state in &ctx.return_states {
            let (value, _) = ctx
                .locals
                .get(state)
                .ok_or_else(|| Residue::Form(format!("? without the current value of {state}")))?;
            items.push(value.clone());
        }
        tuple_value(&ctx.result, &items)
    };
    let (failed, _) = ctx.returning((Expr::Flat(failure), ctx.result.clone()));
    let arms = if matches!(st, Ty::Option(_)) {
        vec![
            Arm {
                pattern: early.0,
                body: failed,
            },
            Arm {
                pattern: ok,
                body: rest,
            },
        ]
    } else {
        vec![
            Arm {
                pattern: ok,
                body: rest,
            },
            Arm {
                pattern: early.0,
                body: failed,
            },
        ]
    };
    Ok(((Expr::Match { scrutinee: s, arms }, rt), acc))
}

/// A candidate function: its LexLean name, signature and body.
pub(crate) struct Candidate<'a> {
    pub(crate) rust: String,
    pub(crate) name: String,
    /// Its type parameters and dictionary, where it is generic: the
    /// dictionary's parameters come before `params`.
    pub(crate) generic: Option<Generic>,
    pub(crate) params: Vec<Param>,
    pub(crate) result: Ty,
    pub(crate) block: &'a rs::Block,
    /// The `use` imports of its file: name -> the path it names.
    pub(crate) imports: BTreeMap<String, String>,
    /// How its file reads the names two of its crate's types share: each
    /// such name -> the LexLean name of the one the file means.
    pub(crate) aliases: BTreeMap<String, String>,
    /// How its file reads the keys two of its crate's functions share
    /// (`toml_string` in two modules): each such key, as a call writes it
    /// -> the module-qualified `rust` key of the one the file means, or
    /// none where the file means neither.
    pub(crate) homonyms: BTreeMap<String, Option<String>>,
    /// The key its own body calls it by: `rust`, or for a function whose
    /// key the crate shares, the key a call writes.
    pub(crate) called: String,
    /// The formatter parameter of a `Display` impl's `fmt`, which lifts as
    /// the function from the value to the text it writes.
    pub(crate) formatter: Option<String>,
    /// What its own module scope binds (see [`Seen`]).
    pub(crate) seen: Seen,
    /// The positions of the parameters it threads (see [`threads`]): its
    /// `result` is then the tuple of the Rust result and their values.
    pub(crate) threads: Vec<usize>,
}

impl Candidate<'_> {
    /// The Rust result: `result`, or where the definition threads `&mut`
    /// parameters, the first item of its tuple.
    pub(crate) fn forward(&self) -> Ty {
        match (&self.result, self.threads.is_empty()) {
            (Ty::Tuple(ts) | Ty::Pair(ts), false) if !ts.is_empty() => ts[0].clone(),
            (t, _) => t.clone(),
        }
    }
}

/// What a lift produced.
pub(crate) struct Lifted {
    pub(crate) decls: Vec<Decl>,
    /// Rust name -> why it did not lift.
    pub(crate) failed: BTreeMap<String, Residue>,
    /// The enums, structs and tuples the lifted definitions name.
    pub(crate) types: BTreeSet<Ty>,
    /// Rust name -> the Lean axioms its definition elaborates with.
    pub(crate) axioms: BTreeMap<String, Vec<String>>,
    /// The lifted functions whose value the kernel cannot compute by
    /// unfolding: they reach a primitive Lean implements opaquely (string
    /// encoding, joining, decimal text), so `decide` cannot close a closed
    /// instance of them.
    pub(crate) opaque: BTreeSet<String>,
    /// The types whose derived order the lifted definitions compare by.
    pub(crate) ordered: BTreeSet<Ty>,
    /// Rust name -> where its definition parts from the Rust function, its
    /// own and its callees'.
    pub(crate) divergences: BTreeMap<String, Vec<String>>,
}

/// What a generic definition's lift produced that it does not write, if
/// anything: a helper or theorem the lift generates takes no type
/// parameter or dictionary of its own, and a tuple over a type parameter
/// is a generated structure that would need one.
fn generic_unread(acc: &Acc, well_founded: bool, c: &Candidate<'_>) -> Option<Residue> {
    let over_param = |t: &Ty| matches!(t, Ty::Param(_));
    let tuple = acc
        .seen
        .iter()
        .chain(c.params.iter().map(|(_, _, t)| t))
        .any(|t| {
            t.mentions().iter().any(|m| match m {
                Ty::Tuple(ts) => ts
                    .iter()
                    .any(|x| over_param(x) || x.render().contains('\'')),
                _ => false,
            })
        });
    if !acc.helpers.is_empty() {
        Some(Residue::Form(
            "a generic function whose lift needs a helper definition".to_string(),
        ))
    } else if well_founded {
        Some(Residue::Form(
            "a generic function whose recursion needs termination evidence".to_string(),
        ))
    } else if tuple {
        Some(Residue::Form("a tuple over a type parameter".to_string()))
    } else {
        None
    }
}

/// The primitives whose Lean definitions the kernel does not unfold.
const KERNEL_OPAQUE: [&str; 6] = [
    "prim split_exact",
    "prim parse_decimal",
    "prim format_decimal",
    "prim compare_bytes",
    "prim utf8_encode",
    "prim join",
];

/// Lift every candidate, to the fixpoint where no lifted body calls a function
/// that failed to lift and no two lifted functions call each other.
pub(crate) fn lift_all(
    candidates: &[Candidate<'_>],
    types: &Types,
    widths: &[(u32, Vec<String>)],
    names: &BTreeSet<String>,
) -> Lifted {
    lift_candidates(candidates, types, widths, names, false)
}

/// Import existing source models with totality required by compiler lowering.
pub(crate) fn lift_models(
    candidates: &[Candidate<'_>],
    types: &Types,
    widths: &[(u32, Vec<String>)],
    names: &BTreeSet<String>,
) -> Lifted {
    lift_candidates(candidates, types, widths, names, true)
}

fn lift_candidates(
    candidates: &[Candidate<'_>],
    types: &Types,
    widths: &[(u32, Vec<String>)],
    names: &BTreeSet<String>,
    total: bool,
) -> Lifted {
    let mut failed: BTreeMap<String, Residue> = BTreeMap::new();
    loop {
        let fns: BTreeMap<String, Sig> = candidates
            .iter()
            .filter(|c| !failed.contains_key(&c.rust))
            .map(|c| {
                (
                    c.rust.clone(),
                    Sig {
                        name: c.name.clone(),
                        params: c.params.iter().map(|(_, _, t)| t.clone()).collect(),
                        result: c.result.clone(),
                        generic: c.generic.clone(),
                        threads: c.threads.clone(),
                    },
                )
            })
            .collect();
        let mut env = Env {
            total,
            types: types.clone(),
            fns: fns.clone(),
            widths: widths.to_vec(),
            names: names.clone(),
        };
        let mut decls = Vec::new();
        let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut mentioned: BTreeMap<String, BTreeSet<Ty>> = BTreeMap::new();
        let mut compares: BTreeMap<String, BTreeSet<Ty>> = BTreeMap::new();
        // Each mutual group of a function and its entwined helpers, by key.
        let mut groups: Vec<Vec<String>> = Vec::new();
        let mut own: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut changed = false;
        for c in candidates {
            if failed.contains_key(&c.rust) {
                continue;
            }
            let names: Vec<String> = c.params.iter().map(|(_, n, _)| n.clone()).collect();
            let ctx = Ctx {
                current: c.called.clone(),
                params: names.clone(),
                result: c.result.clone(),
                return_states: c.threads.iter().map(|&i| c.params[i].0.clone()).collect(),
                locals: c
                    .params
                    .iter()
                    .map(|(rust, n, t)| (rust.clone(), (n.clone(), t.clone())))
                    .collect(),
                tails: BTreeMap::new(),
                elements: None,
                rebuilt: BTreeMap::new(),
                bounds: BTreeMap::new(),
                capped: BTreeMap::new(),
                exits: None,
                own: c.name.clone(),
                top: true,
                looping: None,
                hidden: Vec::new(),
                closures: BTreeMap::new(),
                imports: c.imports.clone(),
                seen: c.seen.clone(),
                formatter: c.formatter.clone(),
                dictionary: c
                    .generic
                    .as_ref()
                    .map(|g| g.dictionary.clone())
                    .unwrap_or_default(),
                sealed: BTreeSet::new(),
                borrowed: ref_binders(&c.block.stmts),
            };
            env.types.paths_open = c.seen.paths_open;
            // A type parameter is the candidate's own.
            env.types.generic = c
                .generic
                .as_ref()
                .map(|g| g.reads.clone())
                .unwrap_or_default();
            // Every candidate's file reads each shared name, so one file's
            // reading never outlives its candidates.
            env.types
                .aliases
                .extend(c.aliases.iter().map(|(n, t)| (n.clone(), t.clone())));
            // Each key the crate's functions share reads as the function
            // its file means, or as no function.
            for (key, meant) in &c.homonyms {
                match meant.as_ref().and_then(|m| fns.get(m)) {
                    Some(sig) => env.fns.insert(key.clone(), sig.clone()),
                    None => env.fns.remove(key),
                };
            }
            let threaded;
            let stmts = if c.threads.is_empty() {
                &c.block.stmts
            } else {
                let states: Vec<String> =
                    c.threads.iter().map(|&i| c.params[i].0.clone()).collect();
                match thread_body(&c.block.stmts, &states, &c.result, &env, &c.called) {
                    Ok(stmts) => {
                        threaded = stmts;
                        &threaded
                    }
                    Err(r) => {
                        failed.insert(c.rust.clone(), r);
                        changed = true;
                        continue;
                    }
                }
            };
            let lifted = block(stmts, &env, &ctx, Acc::default(), Some(&c.result));
            let outcome = match lifted {
                Ok(((body, ty), acc)) if ty.same(&c.result) => {
                    let descends = acc.descends.clone();
                    if descends.len() > 1 {
                        Err(Residue::Form(
                            "recursion on more than one parameter".to_string(),
                        ))
                    } else if let Some(i) = descends.into_iter().next() {
                        match names.get(i) {
                            Some(name) => Ok((body, Some(name.clone()), acc)),
                            None => Err(Residue::Form(
                                "recursive descent outside the caller's parameters".to_string(),
                            )),
                        }
                    } else {
                        Ok((body, None, acc))
                    }
                }
                Ok(_) => Err(Residue::Form("a body of another type".to_string())),
                Err(r) => Err(r),
            };
            let outcome = match outcome {
                // Recursion through a helper of the same mutual group is
                // structural on each member's own argument.
                Ok((body, Some(arg), acc)) if acc.helpers.iter().any(|h| h.entwined) => {
                    Ok((body, Some(arg), None, Vec::new(), acc))
                }
                Ok((body, Some(arg), acc)) => {
                    well_founded(c, &body, &arg, types).map(|(recursive, termination, theorems)| {
                        (body, recursive, termination, theorems, acc)
                    })
                }
                Ok((body, None, acc)) => Ok((body, None, None, Vec::new(), acc)),
                Err(r) => Err(r),
            };
            let outcome = match (outcome, &c.generic) {
                (Ok(lifted), Some(_)) => {
                    generic_unread(&lifted.4, lifted.2.is_some(), c).map_or(Ok(lifted), Err)
                }
                (outcome, _) => outcome,
            };
            match outcome {
                Ok((body, recursive, termination, theorems, acc)) => {
                    // A well-founded definition elaborates with its evidence.
                    let mut evidence_axioms = BTreeSet::new();
                    for t in theorems {
                        if let Decl::Theorem { axioms, .. } = &t {
                            evidence_axioms.extend(axioms.iter().cloned());
                        }
                        decls.push((format!("#{}", t.name()), t));
                    }
                    let calls = acc
                        .calls
                        .into_iter()
                        .map(|k| match c.homonyms.get(&k) {
                            Some(Some(meant)) => meant.clone(),
                            _ => k,
                        })
                        .collect();
                    graph.insert(c.rust.clone(), calls);
                    let mut seen = acc.seen;
                    for (_, _, t) in &c.params {
                        seen.extend(t.mentions());
                    }
                    seen.extend(c.result.mentions());
                    mentioned.insert(c.rust.clone(), seen);
                    compares.insert(c.rust.clone(), acc.ordered);
                    let mut axioms = acc.axioms;
                    axioms.extend(evidence_axioms);
                    own.insert(c.rust.clone(), axioms);
                    let group = acc
                        .helpers
                        .iter()
                        .any(|h| h.entwined)
                        .then(|| format!("{}Mutual", c.name));
                    if group.is_some() {
                        let mut members = vec![c.rust.clone()];
                        members.extend(
                            acc.helpers
                                .iter()
                                .filter(|h| h.entwined)
                                .map(|h| format!("#{}", h.name)),
                        );
                        groups.push(members);
                    }
                    for h in acc.helpers {
                        let key = format!("#{}", h.name);
                        graph.insert(key.clone(), h.calls);
                        own.insert(key.clone(), h.axioms);
                        let termination = h.termination.map(|(measure, theorems)| {
                            let names: Vec<String> =
                                theorems.iter().map(|t| t.name().to_string()).collect();
                            for t in theorems {
                                decls.push((format!("#{}", t.name()), t));
                            }
                            (measure, names)
                        });
                        decls.push((
                            key,
                            Decl::Definition {
                                type_parameters: Vec::new(),
                                recursive: if h.recursive {
                                    h.params.first().map(|(n, _)| n.clone())
                                } else {
                                    None
                                },
                                name: h.name,
                                parameters: h.params,
                                result: h.result,
                                body: h.body,
                                axioms: Vec::new(),
                                termination,
                                mutual: if h.entwined { group.clone() } else { None },
                            },
                        ));
                    }
                    decls.push((
                        c.rust.clone(),
                        Decl::Definition {
                            type_parameters: c
                                .generic
                                .as_ref()
                                .map(|g| g.types.clone())
                                .unwrap_or_default(),
                            name: c.name.clone(),
                            parameters: c
                                .generic
                                .iter()
                                .flat_map(|g| &g.dictionary)
                                .map(|e| (e.name.clone(), e.ty.render()))
                                .chain(c.params.iter().map(|(_, n, t)| (n.clone(), t.render())))
                                .collect(),
                            result: c.result.render(),
                            body,
                            axioms: Vec::new(),
                            recursive,
                            termination,
                            mutual: group,
                        },
                    ));
                }
                Err(reason) => {
                    failed.insert(c.rust.clone(), reason);
                    changed = true;
                }
            }
        }
        // A call cycle is a mutual group once its members lift on their
        // own: before that, what stops a member is the member's own cause.
        let mut mutual_of: BTreeMap<String, String> = BTreeMap::new();
        if !changed {
            for component in call_cycles(&graph) {
                // A mutual group's members share no type parameter or
                // dictionary: a generic member stops the group.
                let generic = candidates
                    .iter()
                    .any(|c| c.generic.is_some() && component.contains(&c.rust));
                let grouped = if generic {
                    Err(Residue::Form(
                        "a generic function in a cycle of calls".to_string(),
                    ))
                } else {
                    mutual_group(&component, candidates, &mut decls, types)
                };
                match grouped {
                    Ok(label) => {
                        for m in &component {
                            if let Some(axioms) = own.get_mut(m) {
                                axioms.extend(evidence_axioms());
                            }
                            mutual_of.insert(m.clone(), label.clone());
                        }
                        groups.push(component.into_iter().collect());
                    }
                    Err(reason) => {
                        for m in component.iter().filter(|m| !m.starts_with('#')) {
                            failed.insert(m.clone(), reason.clone());
                            changed = true;
                        }
                    }
                }
            }
        }
        if !changed {
            let types = mentioned.into_values().flatten().collect();
            // A definition elaborates with its own axioms and its callees'.
            let mut axioms: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for (rust, callees) in &graph {
                let mut all = own[rust].clone();
                let mut stack: Vec<&String> = callees.iter().collect();
                let mut seen = BTreeSet::new();
                while let Some(n) = stack.pop() {
                    if seen.insert(n) {
                        all.extend(own.get(n).into_iter().flatten().cloned());
                        stack.extend(graph.get(n).into_iter().flatten());
                    }
                }
                axioms.insert(rust.clone(), all.into_iter().collect());
            }
            // A mutual group elaborates as one: each member with every
            // member's axioms.
            for members in &groups {
                let all: BTreeSet<String> = members
                    .iter()
                    .flat_map(|m| axioms.get(m).cloned().unwrap_or_default())
                    .collect();
                for m in members {
                    axioms.insert(m.clone(), all.iter().cloned().collect());
                }
            }
            // A divergence is the register's, not Lean's: it leaves the
            // axioms, and a definition diverges where a callee does.
            let mut divergences: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for (key, all) in &mut axioms {
                let parted: Vec<String> = all
                    .iter()
                    .filter_map(|a| a.strip_prefix(DIVERGES).map(String::from))
                    .collect();
                all.retain(|a| !a.starts_with(DIVERGES));
                if !parted.is_empty() {
                    divergences.insert(key.clone(), parted);
                }
            }
            for (key, d) in &mut decls {
                if let Decl::Definition { axioms: a, .. } = d {
                    a.clone_from(&axioms[key.as_str()]);
                }
            }
            axioms.retain(|key, _| !key.starts_with('#'));
            divergences.retain(|key, _| !key.starts_with('#'));
            let opaque_prelude = prelude_opaque();
            let mut opaque: BTreeSet<String> = decls
                .iter()
                .filter(|(_, d)| match d {
                    // Well-founded recursion does not reduce in the kernel.
                    Decl::Definition {
                        termination: Some(_),
                        ..
                    } => true,
                    Decl::Definition { body, .. } => {
                        let text = body.flat();
                        prelude_calls(&text)
                            .iter()
                            .any(|name| opaque_prelude.contains(name))
                            || KERNEL_OPAQUE.iter().any(|p| text.contains(p))
                    }
                    _ => false,
                })
                .map(|(key, _)| key.clone())
                .collect();
            loop {
                let more: Vec<String> = graph
                    .iter()
                    .filter(|(k, callees)| {
                        !opaque.contains(*k) && callees.iter().any(|c| opaque.contains(c))
                    })
                    .map(|(k, _)| k.clone())
                    .collect();
                if more.is_empty() {
                    break;
                }
                opaque.extend(more);
            }
            opaque.retain(|key| !key.starts_with('#'));
            // Each definition after the definitions it calls.
            let mut placed: BTreeSet<String> = BTreeSet::new();
            let mut ordered = Vec::new();
            let mut pending: Vec<(String, Decl)> = decls;
            while !pending.is_empty() {
                let before = ordered.len();
                let mut rest = Vec::new();
                for (key, d) in pending {
                    let ready = graph.get(&key).is_none_or(|callees| {
                        callees.iter().all(|c| {
                            placed.contains(c)
                                || c == &key
                                || !graph.contains_key(c)
                                || mutual_of
                                    .get(c)
                                    .is_some_and(|l| mutual_of.get(&key) == Some(l))
                        })
                    });
                    if ready {
                        placed.insert(key);
                        ordered.push(d);
                    } else {
                        rest.push((key, d));
                    }
                }
                if ordered.len() == before {
                    ordered.extend(rest.into_iter().map(|(_, d)| d));
                    break;
                }
                pending = rest;
            }
            return Lifted {
                decls: grouped(ordered),
                failed,
                types,
                axioms,
                opaque,
                ordered: compares.into_values().flatten().collect(),
                divergences,
            };
        }
    }
}

/// The definitions with each mutual group's members together, where the
/// group's last member stands.
fn grouped(decls: Vec<Decl>) -> Vec<Decl> {
    let label = |d: &Decl| match d {
        Decl::Definition { mutual, .. } => mutual.clone(),
        _ => None,
    };
    let last: BTreeMap<String, usize> = decls
        .iter()
        .enumerate()
        .filter_map(|(i, d)| label(d).map(|l| (l, i)))
        .collect();
    let mut held: BTreeMap<String, Vec<Decl>> = BTreeMap::new();
    let mut out = Vec::new();
    for (i, d) in decls.into_iter().enumerate() {
        match label(&d) {
            Some(l) if last[&l] == i => {
                out.extend(held.remove(&l).unwrap_or_default());
                out.push(d);
            }
            Some(l) => held.entry(l).or_default().push(d),
            None => out.push(d),
        }
    }
    out
}

/// The field types of a constructor of a family's member, as a match
/// enclosing a recursive call binds them: `None` for no such constructor.
fn binder_types(types: &Types, ctor: &str) -> Option<Vec<String>> {
    let (owner, variant) = ctor.rsplit_once('.')?;
    if let Some(vs) = types.datas.get(owner) {
        let (_, fs) = vs.iter().find(|(v, _)| v == variant)?;
        return Some(fs.iter().map(|(_, t)| t.render()).collect());
    }
    if types.boxed(owner) && variant == "mk" {
        return Some(
            types.structs[owner]
                .iter()
                .map(|(_, t)| t.render())
                .collect(),
        );
    }
    types
        .enums
        .get(owner)
        .filter(|vs| vs.iter().any(|v| v == variant))
        .map(|_| Vec::new())
}

/// The definitions a decrease theorem over `family` unfolds: the family's
/// size functions and field accessors its statements name.
fn evidence_definitions(
    types: &Types,
    family: &BTreeSet<String>,
    statements: &[&str],
) -> Vec<String> {
    let mut definitions: Vec<String> = types
        .size_domains(family)
        .iter()
        .filter_map(|d| types.size_name(d, family))
        .collect();
    for m in family {
        if let Some(fields) = types.structs.get(m).filter(|_| types.boxed(m)) {
            definitions.extend(fields.iter().map(|(f, _)| Types::accessor(m, f)));
        }
    }
    let used: BTreeSet<&str> = statements
        .iter()
        .flat_map(|s| s.split(|ch: char| !ch.is_alphanumeric() && ch != '_'))
        .collect();
    definitions.retain(|d| used.contains(d.as_str()));
    definitions.sort();
    definitions
}

/// The proof of a decrease theorem: linear arithmetic after unfolding.
fn evidence_proof(definitions: &[String]) -> String {
    format!(
        "linear_arithmetic{{definitions: [{}]}}",
        definitions.join(", ")
    )
}

/// What a decrease theorem's proof elaborates with: omega's axioms.
fn evidence_axioms() -> Vec<String> {
    vec!["Quot.sound".to_string(), "propext".to_string()]
}

/// The recursion of a candidate descending on `arg`: structural, or, over
/// a nested or mutual family, well-founded on the family's size with one
/// evidence theorem per recursive call.
#[allow(clippy::type_complexity)]
fn well_founded(
    c: &Candidate<'_>,
    body: &Expr,
    arg: &str,
    types: &Types,
) -> Lift<(Option<String>, Option<(String, Vec<String>)>, Vec<Decl>)> {
    let structural = || Ok((Some(arg.to_string()), None, Vec::new()));
    let Some(index) = c.params.iter().position(|(_, n, _)| n == arg) else {
        return structural();
    };
    let Ty::Named(n) = &c.params[index].2 else {
        return structural();
    };
    if !types.nested(n) {
        return structural();
    }
    let family = types.family_of(n).cloned().unwrap_or_default();
    let refused = || {
        Err(Residue::Form(format!(
            "recursion over the nested type {n} that LexLean cannot measure"
        )))
    };
    if !types.sizable(&family) {
        return refused();
    }
    let size = format!("size_of_{n}");
    let binder_types = |ctor: &str| binder_types(types, ctor);
    let params: Vec<(String, String)> = c
        .params
        .iter()
        .map(|(_, n, t)| (n.clone(), t.render()))
        .collect();
    let Some(obligations) =
        crate::wf::obligations(&c.name, &params, index, &size, &body.flat(), &binder_types)
    else {
        return refused();
    };
    let statements: Vec<&str> = obligations.iter().map(|(_, s)| s.as_str()).collect();
    let definitions = evidence_definitions(types, &family, &statements);
    let mut theorems = Vec::new();
    let mut evidence = Vec::new();
    for (i, (parameters, statement)) in obligations.into_iter().enumerate() {
        let name = format!("decreases_{}_{i}", c.name);
        evidence.push(name.clone());
        theorems.push(Decl::Theorem {
            type_parameters: Vec::new(),
            name,
            parameters,
            statement,
            proof: evidence_proof(&definitions),
            axioms: evidence_axioms(),
        });
    }
    Ok((None, Some((format!("({size} {arg})"), evidence)), theorems))
}

/// The call cycles of the graph: each set of functions that call one
/// another, round and round, whole.
fn call_cycles(graph: &BTreeMap<String, BTreeSet<String>>) -> Vec<BTreeSet<String>> {
    let reach = |start: &String| -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut stack: Vec<&String> = graph[start].iter().collect();
        while let Some(n) = stack.pop() {
            if seen.insert(n.clone()) {
                stack.extend(graph.get(n).into_iter().flatten());
            }
        }
        seen
    };
    let on = cyclic(graph);
    let reached: BTreeMap<&String, BTreeSet<String>> = on.iter().map(|n| (n, reach(n))).collect();
    let mut out: Vec<BTreeSet<String>> = Vec::new();
    for n in &on {
        if out.iter().any(|c| c.contains(n)) {
            continue;
        }
        out.push(
            on.iter()
                .filter(|m| reached[n].contains(*m) && reached[*m].contains(n))
                .cloned()
                .collect(),
        );
    }
    out
}

/// A measured parameter's position, size function and family.
type Measured = (usize, String, BTreeSet<String>);

/// The parameter a member of a mutual group is measured on: the first of a
/// nested family's member, or a list or an option of one, with its size
/// function and family.
fn measured(c: &Candidate<'_>, types: &Types) -> Option<Measured> {
    c.params.iter().enumerate().find_map(|(i, (_, _, t))| {
        let n = match t {
            Ty::Named(n) => n,
            Ty::List(inner) | Ty::Option(inner) => match &**inner {
                Ty::Named(n) => n,
                _ => return None,
            },
            _ => return None,
        };
        let family = types
            .family_of(n)
            .filter(|f| types.nested(n) && types.sizable(f))?;
        let size = types.size_name(t, family)?;
        Some((i, size, family.clone()))
    })
}

/// Every text of a lifted expression, rewritten.
fn map_text(e: Expr, f: &dyn Fn(String) -> String) -> Expr {
    match e {
        Expr::Flat(s) => Expr::Flat(f(s)),
        Expr::If {
            condition,
            then,
            otherwise,
        } => Expr::If {
            condition: Box::new(map_text(*condition, f)),
            then: Box::new(map_text(*then, f)),
            otherwise: Box::new(map_text(*otherwise, f)),
        },
        Expr::Match { scrutinee, arms } => Expr::Match {
            scrutinee: f(scrutinee),
            arms: arms
                .into_iter()
                .map(|a| Arm {
                    pattern: a.pattern,
                    body: map_text(a.body, f),
                })
                .collect(),
        },
        Expr::Let {
            name,
            ty,
            value,
            body,
        } => Expr::Let {
            name,
            ty,
            value: Box::new(map_text(*value, f)),
            body: Box::new(map_text(*body, f)),
        },
    }
}

/// A body that reads fields of its measured parameter, a struct of a
/// family, through accessors, as a match on the parameter whose binders
/// are the fields: a call on a binder is on a part, whose decrease the
/// match states.
fn fields_bound(body: Expr, param: &str, ty: &Ty, types: &Types, params: &[String]) -> Expr {
    let Ty::Named(name) = ty else {
        return body;
    };
    let Some(fields) = types.structs.get(name).filter(|_| types.boxed(name)) else {
        return body;
    };
    let reads = |e: &Expr| {
        let text = e.flat();
        fields
            .iter()
            .any(|(f, _)| text.contains(&format!("({} {param})", Types::accessor(name, f))))
    };
    if !reads(&body) {
        return body;
    }
    let text = body.flat();
    let words: BTreeSet<&str> = text
        .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
        .chain(params.iter().map(String::as_str))
        .collect();
    let mut binders: Vec<String> = Vec::new();
    for (f, _) in fields {
        let base = local_name(f);
        let taken: Vec<String> = words
            .iter()
            .map(|w| w.to_string())
            .chain(binders.iter().cloned())
            .filter(|w| w.starts_with(&base))
            .collect();
        let free = if taken.contains(&base) {
            first_free(&base, &taken)
        } else {
            base
        };
        binders.push(free);
    }
    let reads: Vec<(String, String)> = fields
        .iter()
        .zip(&binders)
        .map(|((f, _), b)| (format!("({} {param})", Types::accessor(name, f)), b.clone()))
        .collect();
    let replaced = map_text(body, &|s: String| {
        reads.iter().fold(s, |s, (read, b)| s.replace(read, b))
    });
    Expr::Match {
        scrutinee: param.to_string(),
        arms: vec![Arm {
            pattern: format!("{name}.mk {}", binders.join(" ")),
            body: replaced,
        }],
    }
}

/// A call cycle of lifted functions as one LexLean mutual group, each
/// member well-founded on the size of its measured argument, with a theorem
/// that each call of a member decreases the measure. The group's label.
fn mutual_group(
    members: &BTreeSet<String>,
    candidates: &[Candidate<'_>],
    decls: &mut Vec<(String, Decl)>,
    types: &Types,
) -> Lift<String> {
    let form = |why: &str| Err(Residue::Form(format!("mutual recursion {why}")));
    if members.iter().any(|m| m.starts_with('#')) {
        return form("through a helper");
    }
    let mut rows = Vec::new();
    for m in members {
        let Some(c) = candidates.iter().find(|c| &c.rust == m) else {
            return form("of a function outside the lift");
        };
        let Some((at, size, family)) = measured(c, types) else {
            return form("on no argument of a nested family");
        };
        let names: Vec<String> = c.params.iter().map(|(_, n, _)| n.clone()).collect();
        let Some((_, Decl::Definition { body, mutual, .. })) =
            decls.iter_mut().find(|(k, _)| k == m)
        else {
            return form("of a function outside the lift");
        };
        if mutual.is_some() {
            return form("of a function with entwined helpers");
        }
        let (_, param, ty) = &c.params[at];
        *body = fields_bound(body.clone(), param, ty, types, &names);
        let params: Vec<(String, String)> = c
            .params
            .iter()
            .map(|(_, n, t)| (n.clone(), t.render()))
            .collect();
        rows.push((c, params, at, size, family, body.flat()));
    }
    let group: Vec<crate::wf::GroupMember<'_>> = rows
        .iter()
        .map(|(c, params, at, size, _, body)| crate::wf::GroupMember {
            name: &c.name,
            params,
            measured: *at,
            size,
            body,
        })
        .collect();
    let (measures, obligations) =
        crate::wf::group_obligations(&group, &|ctor| binder_types(types, ctor))?;
    let family: BTreeSet<String> = rows.iter().flat_map(|r| r.4.iter().cloned()).collect();
    let label = format!("{}Mutual", rows[0].0.name);
    // The theorems a member's own recursion carried give way to the group's.
    let names: Vec<String> = rows.iter().map(|r| r.0.name.clone()).collect();
    decls.retain(|(key, _)| {
        !names.iter().any(|n| {
            key.strip_prefix(&format!("#decreases_{n}_"))
                .is_some_and(|i| i.chars().all(|ch| ch.is_ascii_digit()))
        })
    });
    // The theorems stand before every member, which cites them.
    let mut at = decls
        .iter()
        .position(|(k, _)| members.contains(k))
        .unwrap_or(decls.len());
    for ((row, measure), owed) in rows.iter().zip(measures).zip(obligations) {
        let statements: Vec<&str> = owed.iter().map(|(_, s)| s.as_str()).collect();
        let definitions = evidence_definitions(types, &family, &statements);
        let mut evidence = Vec::new();
        for (i, (parameters, statement)) in owed.into_iter().enumerate() {
            let name = format!("decreases_{}_{i}", row.0.name);
            evidence.push(name.clone());
            decls.insert(
                at,
                (
                    format!("#{name}"),
                    Decl::Theorem {
                        type_parameters: Vec::new(),
                        name,
                        parameters,
                        statement,
                        proof: evidence_proof(&definitions),
                        axioms: evidence_axioms(),
                    },
                ),
            );
            at += 1;
        }
        if let Some((
            _,
            Decl::Definition {
                recursive,
                termination,
                mutual,
                ..
            },
        )) = decls.iter_mut().find(|(k, _)| k == &row.0.rust)
        {
            *recursive = None;
            *termination = Some((measure, evidence));
            *mutual = Some(label.clone());
        }
    }
    Ok(label)
}

/// The functions on a call cycle through another function.
fn cyclic(graph: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for start in graph.keys() {
        let mut seen = BTreeSet::new();
        let mut stack: Vec<&String> = graph[start].iter().collect();
        while let Some(n) = stack.pop() {
            if n == start {
                out.insert(start.clone());
                break;
            }
            if seen.insert(n.clone()) {
                if let Some(next) = graph.get(n) {
                    stack.extend(next.iter());
                }
            }
        }
    }
    out
}

/// A literal argument of an acceptance assertion, of the parameter's type.
fn literal(e: &rs::RustExpr, want: &Ty, types: &Types) -> Option<String> {
    match e {
        rs::RustExpr::Lit(lit) => match literal_of(lit, Some(want), false) {
            Ok((Expr::Flat(s), t)) if t.same(want) => Some(s),
            _ => None,
        },
        rs::RustExpr::Unary(u) if matches!(u.op, rs::UnOp::Neg) => match &*u.expr {
            rs::RustExpr::Lit(lit) => match literal_of(lit, Some(want), true) {
                Ok((Expr::Flat(s), t)) if t.same(want) => Some(s),
                _ => None,
            },
            _ => None,
        },
        rs::RustExpr::Path(p) if wrapper(p) == Some("None") => match want {
            Ty::Option(inner) => Some(format!("(ctor Option.none [{}])", inner.render())),
            _ => None,
        },
        rs::RustExpr::Path(p) => {
            let (owner, variant) = ctor(p, types)?;
            (Ty::Named(owner.clone()) == *want).then(|| format!("(ctor {owner}.{variant})"))
        }
        rs::RustExpr::Paren(p) => literal(p, want, types),
        rs::RustExpr::Reference(r) => literal(&r.expr, want, types),
        // `"text".to_string()`: the string.
        rs::RustExpr::MethodCall(m)
            if *want == Ty::Str
                && m.args.is_empty()
                && matches!(m.method.as_str(), "to_string" | "to_owned" | "into") =>
        {
            match &*m.receiver {
                rs::RustExpr::Lit(rs::Lit::Str(_)) => literal(&m.receiver, want, types),
                _ => None,
            }
        }
        // `S { a: lit, .. }`: a structure of literals, every field given.
        rs::RustExpr::Struct(st) if st.rest.is_none() && !st.qself => {
            // The crate declares a structure's name once: its last segment.
            let name = types.resolve(&st.path.segments.last()?.ident);
            let fields = types.structs.get(&name)?;
            if *want != Ty::Named(name.clone()) || st.fields.len() != fields.len() {
                return None;
            }
            let mut parts = Vec::new();
            for (field, ty) in fields {
                let given = st.fields.iter().find(
                    |f| matches!(&f.member, rs::Member::Named(n) if local_name(n) == *field),
                )?;
                parts.push((field, literal(&given.expr, ty, types)?));
            }
            Some(struct_value(&name, &parts, types))
        }
        // `Some(lit)`, and `E::V(lit, ..)` of a data-carrying enum.
        rs::RustExpr::Call(c) => {
            let rs::RustExpr::Path(p) = &*c.func else {
                return None;
            };
            match (wrapper(p), want, c.args.as_slice()) {
                (Some("Some"), Ty::Option(inner), [one]) => Some(format!(
                    "(ctor Option.some [{}] {})",
                    inner.render(),
                    literal(one, inner, types)?
                )),
                _ => {
                    let (owner, variant, fields) = data_variant(p, types)?;
                    if Ty::Named(owner.clone()) != *want || fields.len() != c.args.len() {
                        return None;
                    }
                    let mut parts = vec![format!("ctor {owner}.{variant}")];
                    for (arg, (_, ty)) in c.args.iter().zip(fields) {
                        parts.push(literal(arg, ty, types)?);
                    }
                    Some(format!("({})", parts.join(" ")))
                }
            }
        }
        // `[a, b]`: a list of literals.
        rs::RustExpr::Array(items) => {
            let Ty::List(elem) = want else {
                return None;
            };
            let mut list = format!("(nil {})", elem.render());
            for item in items.iter().rev() {
                list = format!("(cons {} {list})", literal(item, elem, types)?);
            }
            Some(list)
        }
        _ => None,
    }
}

/// `f(lit, ..)` against a lifted function: the applied term, the result
/// type and the function's Rust name.
fn applied(
    e: &rs::RustExpr,
    lifted: &BTreeMap<String, Sig>,
    types: &Types,
) -> Option<(String, Ty, String)> {
    let rs::RustExpr::Call(c) = e else {
        return None;
    };
    let rs::RustExpr::Path(p) = &*c.func else {
        return None;
    };
    // Associated functions are keyed by their owner too. A short-name
    // lookup loses their acceptance root or binds an unrelated function.
    let name = p
        .segments
        .iter()
        .map(|s| s.ident.as_str())
        .collect::<Vec<_>>()
        .join("::");
    let sig = lifted.get(&name)?;
    if sig.params.len() != c.args.len() {
        return None;
    }
    let mut parts = vec![sig.name.clone()];
    for (arg, ty) in c.args.iter().zip(&sig.params) {
        parts.push(literal(arg, ty, types)?);
    }
    Some((format!("({})", parts.join(" ")), sig.result.clone(), name))
}

/// What an assertion of a test states about a lifted function, when it is a
/// closed instance: `assert!(f(1, 2))`, `assert!(!f(..))`, `assert_eq!(f(..), 3)`.
pub(crate) struct Acceptance {
    /// The claim as a proposition: `(f 1 2 = true)`.
    pub(crate) statement: String,
    /// The same claim as a Boolean term with no free variable, the body of the
    /// acceptance root a native library declares (SPEC 12.6: acceptance roots
    /// are zero-argument Boolean definitions).
    pub(crate) check: String,
    /// The lifted function's Rust name.
    pub(crate) callee: String,
}

/// The Boolean term that holds exactly when `call`, of type `result`,
/// equals the literal `right`: the decidable equality of each type the
/// statement admits, written with the operators LexLean gives it.
fn holds(call: &str, result: &Ty, right: &rs::RustExpr, types: &Types) -> Option<String> {
    match result {
        Ty::Bool => match literal(right, result, types)?.as_str() {
            "true" => Some(call.to_string()),
            "false" => Some(format!("(not {call})")),
            _ => None,
        },
        Ty::Nat(_) => Some(format!("({call} =? {})", literal(right, result, types)?)),
        Ty::Str => Some(format!(
            "(prim equal {call} {} : Bool)",
            literal(right, result, types)?
        )),
        Ty::Option(inner) => {
            let (none, some) = match right {
                rs::RustExpr::Path(p) if wrapper(p) == Some("None") => {
                    ("true".to_string(), "false".to_string())
                }
                rs::RustExpr::Call(c) => {
                    let rs::RustExpr::Path(p) = &*c.func else {
                        return None;
                    };
                    match (wrapper(p), c.args.as_slice()) {
                        (Some("Some"), [one]) => {
                            ("false".to_string(), holds("held", inner, one, types)?)
                        }
                        _ => return None,
                    }
                }
                rs::RustExpr::Paren(p) => return holds(call, result, p, types),
                _ => return None,
            };
            if matches!(**inner, Ty::Option(_)) {
                return None;
            }
            Some(format!(
                "(match {call} | Option.none => {none} | Option.some held => {some})"
            ))
        }
        _ => None,
    }
}

/// The acceptance an assertion makes about a lifted function, when it is a
/// closed instance over a type with decidable equality.
pub(crate) fn acceptance(
    kind: &str,
    args: &[rs::RustExpr],
    lifted: &BTreeMap<String, Sig>,
    types: &Types,
) -> Option<Acceptance> {
    match (kind, args) {
        ("assert", [single, ..]) => {
            let (inner, value) = match single {
                rs::RustExpr::Unary(rs::ExprUnary {
                    op: rs::UnOp::Not,
                    expr,
                    ..
                }) => (&**expr, false),
                other => (other, true),
            };
            let (call, result, callee) = applied(inner, lifted, types)?;
            (result == Ty::Bool).then(|| Acceptance {
                statement: format!("({call} = {value})"),
                check: if value {
                    call.clone()
                } else {
                    format!("(not {call})")
                },
                callee,
            })
        }
        ("assert_eq", [left, right, ..]) => {
            let (call, result, callee) = applied(left, lifted, types)?;
            let decidable = |t: &Ty| matches!(t, Ty::Bool | Ty::Nat(_) | Ty::Str);
            let comparable = match &result {
                Ty::Option(inner) => decidable(inner),
                t => decidable(t),
            };
            if !comparable {
                return None;
            }
            let value = literal(right, &result, types)?;
            let check = holds(&call, &result, right, types)?;
            Some(Acceptance {
                statement: format!("({call} = {value})"),
                check,
                callee,
            })
        }
        _ => None,
    }
}

/// The function type of a trait's method in a dictionary over the type
/// parameter `param`, whose reading `types.generic` holds under `Self`
/// too: the receiver (`self` or `&self`, both the value) first, then the
/// parameters. A method of type or const parameters of its own, of a
/// receiver it changes or of no parameter at all is no entry.
pub(crate) fn entry_ty(types: &Types, sig: &rs::Signature, param: &Ty) -> Lift<Ty> {
    if let Some(what) = unread_signature(sig) {
        return Err(Residue::Signature(format!("a trait method {what}")));
    }
    let mut inputs = Vec::new();
    for input in &sig.inputs {
        match input {
            rs::FnArg::Receiver { plain: true } => inputs.push(param.clone()),
            rs::FnArg::Receiver { plain: false } => {
                return Err(Residue::Signature(
                    "a trait method that takes &mut self, mut self or a typed self: a dictionary \
                     holds functions of the value, and threads no changed receiver"
                        .to_string(),
                ))
            }
            rs::FnArg::Typed(pat) => inputs.push(types.ty(&pat.ty)?),
        }
    }
    if inputs.is_empty() {
        return Err(Residue::Signature(
            "a trait method of no parameter, which a dictionary holds as no function".to_string(),
        ));
    }
    let result = match &sig.output {
        None => Ty::Unit,
        Some(t) => types.ty(t)?,
    };
    Ok(Ty::Fn(inputs, Box::new(result)))
}

/// What of a signature the lift does not read, each part named, or `None`
/// when it reads all of it. A type or const parameter makes the function
/// one per instantiation, and an `async`, `unsafe` or variadic function
/// is not a total function of its arguments' values. A lifetime parameter
/// says how long a borrow lives, which a value of LexLean does not have:
/// it is erased.
fn unread_signature(sig: &rs::Signature) -> Option<String> {
    let mut parts = Vec::new();
    let named = |kind: &str, names: Vec<&String>| {
        let names: Vec<&str> = names.iter().map(|n| n.as_str()).collect();
        format!("generic over {kind} {}", names.join(", "))
    };
    let types: Vec<&String> = sig
        .generics
        .iter()
        .filter_map(|p| match p {
            rs::GenericParam::Type(n) => Some(n),
            _ => None,
        })
        .collect();
    let consts: Vec<&String> = sig
        .generics
        .iter()
        .filter_map(|p| match p {
            rs::GenericParam::Const(n) => Some(n),
            _ => None,
        })
        .collect();
    if !types.is_empty() {
        parts.push(named("type parameters", types));
    }
    if !consts.is_empty() {
        parts.push(named("const parameters", consts));
    }
    for (holds, what) in [
        (sig.asyncness, "async"),
        (sig.unsafety, "unsafe"),
        (sig.variadic, "variadic"),
    ] {
        if holds {
            parts.push(what.to_string());
        }
    }
    (!parts.is_empty()).then(|| parts.join(", "))
}

#[cfg(test)]
mod tests {

    #[test]
    fn map_carriers_keep_closed_abi_and_transport_generic_instances_ue_71() {
        let k = Ty::Param("K".into());
        let v = Ty::Param("V".into());
        let generic = Ty::Map(Box::new(k), Box::new(v), false);
        let closed = Ty::Map(Box::new(Ty::Nat(Some(8))), Box::new(Ty::Bool), false);
        assert_eq!(closed.render(), "(List TupleNatBool)");
        assert_eq!(closed.tag(), "ListTupleNatBool");
        assert_eq!(generic.render(), "(List ('K * 'V))");
        assert!(!generic.mentions().iter().any(|t| matches!(t, Ty::Tuple(_))));
        let bindings = BTreeMap::from([("K".into(), Ty::Nat(Some(8))), ("V".into(), Ty::Bool)]);
        assert_eq!(
            model_carrier(&generic, &bindings).render(),
            "(List (Nat * Bool))"
        );
        let sig = Sig {
            name: "keep".into(),
            params: vec![generic.clone()],
            result: generic,
            threads: Vec::new(),
            generic: Some(Generic {
                types: vec!["K".into(), "V".into()],
                reads: BTreeMap::new(),
                dictionary: Vec::new(),
            }),
        };
        let (_, result, calls, decls) = bind_model_call(
            &sig,
            &[("m".into(), closed.clone())],
            &Types::default(),
            &BTreeMap::new(),
            &[],
            &BTreeSet::new(),
        )
        .unwrap();
        assert_eq!(result, closed);
        assert_eq!(calls, BTreeSet::from(["keep".into()]));
        assert_eq!(
            decls.len(),
            4,
            "entry and list transports in both directions"
        );
        assert!(decls
            .iter()
            .all(|d| matches!(d,Decl::Definition {axioms,..} if axioms.is_empty())));
    }
    use super::*;

    #[test]
    fn compiler_generic_bindings_keep_type_identity_and_dictionary_evidence_ue_66() {
        let t = Ty::Param("T".into());
        let mut sig = Sig {
            name: "transport".into(),
            params: vec![t.clone(), t.clone()],
            result: t.clone(),
            threads: Vec::new(),
            generic: Some(Generic {
                types: vec!["T".into()],
                reads: BTreeMap::from([("T".into(), t.clone())]),
                dictionary: Vec::new(),
            }),
        };
        let call =
            |sig: &Sig, args: &[(String, Ty)], types: &Types, fns: &BTreeMap<String, Sig>| {
                bind_model_call(sig, args, types, fns, &[], &BTreeSet::new())
            };
        let args = [
            ("a".into(), Ty::Nat(Some(8))),
            ("b".into(), Ty::Nat(Some(8))),
        ];
        let (term, result, calls, declarations) =
            call(&sig, &args, &Types::default(), &BTreeMap::new()).unwrap();
        assert!(declarations.is_empty());
        assert_eq!(term, "(transport [Nat] a b)");
        assert_eq!(result, Ty::Nat(Some(8)));
        assert_eq!(calls, BTreeSet::from(["transport".into()]));
        let mixed = [args[0].clone(), ("b".into(), Ty::Nat(Some(64)))];
        assert!(call(&sig, &mixed, &Types::default(), &BTreeMap::new()).is_err());
        sig.generic.as_mut().unwrap().dictionary.push(Entry {
            param: "T".into(),
            declarer: "Observe".into(),
            method: "observe".into(),
            name: "observe".into(),
            ty: Ty::Fn(vec![t], Box::new(Ty::Bool)),
        });
        let args = [
            ("a".into(), Ty::Named("Flag".into())),
            ("b".into(), Ty::Named("Flag".into())),
        ];
        assert!(call(&sig, &args, &Types::default(), &BTreeMap::new()).is_err());
        let mut types = Types::default();
        types.impls.insert(
            ("Observe".into(), "Flag".into(), "observe".into()),
            "FlagObserve".into(),
        );
        let mut method = Sig {
            name: "flagObserve".into(),
            params: vec![Ty::Named("Flag".into())],
            result: Ty::Bool,
            generic: None,
            threads: Vec::new(),
        };
        let fns = BTreeMap::from([("FlagObserve".into(), method.clone())]);
        let (_, _, calls, _) = call(&sig, &args, &types, &fns).unwrap();
        assert!(calls.contains("flagObserve"));
        method.result = Ty::Nat(Some(8));
        assert!(call(
            &sig,
            &args,
            &types,
            &BTreeMap::from([("FlagObserve".into(), method)])
        )
        .is_err());
    }

    #[test]
    fn qualified_acceptance_retains_its_owner_ue_27() {
        let lifted = BTreeMap::from([
            (
                "One::empty".into(),
                Sig {
                    threads: Vec::new(),
                    generic: None,
                    name: "oneEmpty".into(),
                    params: vec![Ty::Bool],
                    result: Ty::Bool,
                },
            ),
            (
                "Two::empty".into(),
                Sig {
                    threads: Vec::new(),
                    generic: None,
                    name: "twoEmpty".into(),
                    params: vec![Ty::Bool],
                    result: Ty::Bool,
                },
            ),
        ]);
        let check = |source: &str| {
            let expression: syn::Expr = syn::parse_str(source).unwrap();
            acceptance(
                "assert",
                &[crate::convert::expr(&expression)],
                &lifted,
                &Types::default(),
            )
        };
        assert_eq!(check("One::empty(true)").unwrap().check, "(oneEmpty true)");
        assert_eq!(
            check("Two::empty(false)").unwrap().check,
            "(twoEmpty false)"
        );
        assert!(check("Other::empty(true)").is_none());
    }

    #[test]
    fn recursive_descent_must_name_a_callers_parameter_ue_24() {
        let mut env = Env {
            total: false,
            types: Types::default(),
            fns: BTreeMap::from([(
                "again".into(),
                Sig {
                    threads: Vec::new(),
                    generic: None,
                    name: "again".into(),
                    params: vec![Ty::Bool; 3],
                    result: Ty::Bool,
                },
            )]),
            widths: pointer_widths(&[]),
            names: BTreeSet::new(),
        };
        let mut ctx = Ctx {
            sealed: BTreeSet::new(),
            borrowed: BTreeSet::new(),
            dictionary: Vec::new(),
            current: "again".into(),
            own: "again".into(),
            params: vec!["first".into(), "second".into()],
            result: Ty::Bool,
            return_states: Vec::new(),
            locals: BTreeMap::new(),
            tails: BTreeMap::new(),
            top: true,
            looping: None,
            hidden: Vec::new(),
            imports: BTreeMap::new(),
            closures: BTreeMap::new(),
            elements: None,
            rebuilt: BTreeMap::new(),
            bounds: BTreeMap::new(),
            capped: BTreeMap::new(),
            exits: None,
            formatter: None,
            seen: Seen::default(),
        };
        let call = |source: &str, env: &Env, ctx: &Ctx| {
            let parsed: syn::Expr = syn::parse_str(source).unwrap();
            let rs::RustExpr::Call(call) = crate::convert::expr(&parsed) else {
                panic!("call")
            };
            fn_call(&call, &["again".into()], env, ctx, Acc::default())
        };
        // Two absent lookups are not evidence of descent at index two.
        assert!(matches!(
            call("again(false, false, false)", &env, &ctx),
            Err(Residue::Form(_))
        ));
        let list = Ty::List(Box::new(Ty::Bool));
        env.fns.get_mut("again").unwrap().params = vec![Ty::Bool, list.clone()];
        ctx.locals.insert("tail".into(), ("tail".into(), list));
        ctx.tails.insert("tail".into(), "second".into());
        let (_, acc) = call("again(false, tail)", &env, &ctx).unwrap();
        assert_eq!(acc.descends, BTreeSet::from([1]));
        ctx.params.clear();
        assert!(matches!(
            call("again(false, tail)", &env, &ctx),
            Err(Residue::Form(_))
        ));
    }

    fn lift(src: &str, types: &Types) -> Result<String, Residue> {
        let f: syn::ItemFn = syn::parse_str(src).expect("parses");
        let sig = crate::convert::signature(&f.sig);
        let block = crate::convert::block(&f.block);
        let (params, result) = types.signature(&sig, None)?;
        let c = Candidate {
            generic: None,
            rust: sig.ident.clone(),
            name: camel(&[&sig.ident]),
            params,
            result,
            block: &block,
            imports: BTreeMap::from([("Sha256".to_string(), "sha2::Sha256".to_string())]),
            aliases: BTreeMap::new(),
            homonyms: BTreeMap::new(),
            called: sig.ident.clone(),
            formatter: None,
            seen: Seen::default(),
            threads: threads(&sig, None),
        };
        let out = lift_all(&[c], types, &pointer_widths(&[]), &BTreeSet::new());
        match out.failed.into_values().next() {
            Some(r) => Err(r),
            None => Ok(out
                .decls
                .iter()
                .map(crate::lex::render_decl)
                .collect::<Vec<_>>()
                .join("\n")),
        }
    }

    #[test]
    fn usize_widths_come_from_the_declared_targets() {
        let declared = [
            "x86_64-unknown-linux-gnu".to_string(),
            "wasm32-unknown-unknown".to_string(),
        ];
        let widths = pointer_widths(&declared);
        assert_eq!(widths.len(), 2);
        assert_eq!(widths[0].0, 32);
        assert_eq!(widths[0].1, vec!["wasm32-unknown-unknown".to_string()]);
        assert_eq!(widths[1].0, 64);
        let open: Vec<u32> = pointer_widths(&[]).iter().map(|(w, _)| *w).collect();
        assert_eq!(open, vec![16, 32, 64]);
        assert_eq!(pointer_width("avr-unknown-gnu-atmega328"), Some(16));
    }

    fn plain() -> Types {
        Types::default()
    }

    /// A function of `n` statement `if`s in a row, each updating the locals
    /// the next reads, and its statement count.
    fn ifs_in_a_row(n: usize) -> (String, usize) {
        let mut body = String::from("let mut a = x; let mut b = false; ");
        for k in 0..n {
            body.push_str(&format!(
                "if a < {k} {{ a = {k}; b = !b; }} else {{ b = a == {k}; }} "
            ));
        }
        body.push_str("(a, b)");
        (
            format!("fn steps(x: u32) -> (u32, bool) {{ {body} }}"),
            n + 3,
        )
    }

    #[test]
    fn statement_ifs_lift_the_statements_after_them_once() {
        // The whole function with one `if` bounds what each statement of a
        // longer one costs, where copying the statements after each `if`
        // into both its branches doubles the output per `if`.
        let (one, one_stmts) = ifs_in_a_row(1);
        let one_len = lift(&one, &plain()).expect("lifts").len();
        // Up to as many `if`s as one statement costs bytes: past the point
        // where doubling per `if` would outgrow any linear bound.
        let mut n = 1;
        while n <= one_len / one_stmts {
            let (src, stmts) = ifs_in_a_row(n);
            let out = lift(&src, &plain()).expect("lifts");
            assert!(
                out.len() <= one_len * stmts,
                "{n} ifs: {} bytes for {stmts} statements",
                out.len()
            );
            n *= 2;
        }
    }

    #[test]
    fn a_percent_sign_is_residue() {
        let out = lift(r#"fn rem() -> String { "%=".to_string() }"#, &plain());
        assert_eq!(out, Err(Residue::Form(PERCENT.to_string())));
    }

    #[test]
    fn comparisons_and_logic_lift() {
        let out = lift(
            "fn within(c: u32, lo: u32, hi: u32) -> bool { lo <= c && c <= hi }",
            &plain(),
        )
        .expect("lifts");
        assert!(out.contains("((lo <=? c) && (c <=? hi))"), "{out}");
    }

    #[test]
    fn arithmetic_without_a_named_overflow_is_residue() {
        assert_eq!(
            lift("fn add(a: u32, b: u32) -> u32 { a + b }", &plain()),
            Err(Residue::Arithmetic("+".to_string()))
        );
        assert_eq!(
            lift(
                "fn add(a: usize, b: usize) -> Option<usize> { a.checked_add(b) }",
                &plain()
            ),
            Err(Residue::Width(".checked_add on usize".to_string()))
        );
    }

    #[test]
    fn named_overflow_lifts() {
        let out = lift(
            "fn add(a: u8, b: u8) -> Option<u8> { a.checked_add(b) }",
            &plain(),
        )
        .expect("lifts");
        assert!(out.contains("(if ((a + b) <? 256) then (ctor Option.some [Nat] (a + b)) else (ctor Option.none [Nat]))"), "{out}");
        let out =
            lift("fn add(a: u8, b: u8) -> u8 { a.wrapping_add(b) }", &plain()).expect("lifts");
        assert!(
            out.contains("(prim remainder (a + b) 256 0 : Nat)"),
            "{out}"
        );
        let out = lift("fn half(a: u32) -> u32 { a / 2 }", &plain()).expect("lifts");
        assert!(out.contains("(prim quotient a 2 0 : Nat)"), "{out}");
    }

    #[test]
    fn signed_strings_options_and_lets_lift() {
        let out = lift("fn neg(a: i64) -> bool { a < 0 }", &plain()).expect("lifts");
        assert!(
            out.contains("(prim less_than a (0 : Int64) : Bool)"),
            "{out}"
        );
        let out = lift("fn yes(s: &str) -> bool { s == \"yes\" }", &plain()).expect("lifts");
        assert!(out.contains("(prim equal s \"yes\" : Bool)"), "{out}");
        let out = lift(
            "fn f(o: Option<u32>) -> Option<u32> { let v = o?; let w = v / 2; Some(w) }",
            &plain(),
        )
        .expect("lifts");
        assert!(out.contains("| Option.some v => (let w : Nat := (prim quotient v 2 0 : Nat); (ctor Option.some [Nat] w)))"), "{out}");
    }

    #[test]
    fn slice_recursion_is_structural() {
        let out = lift(
            "fn count(xs: &[bool]) -> u32 { match xs { [] => 0, [x, rest @ ..] => if *x { count(rest).saturating_add(1) } else { count(rest) } } }",
            &plain(),
        )
        .expect("lifts");
        assert!(out.contains("recursive_argument: xs"), "{out}");
        assert_eq!(
            lift("fn spin(n: u32) -> u32 { spin(n) }", &plain()),
            Err(Residue::Form(
                "recursion that does not descend on a slice".to_string()
            ))
        );
    }

    #[test]
    fn literal_matches_and_string_formats_lift() {
        let out = lift(
            "fn name(n: u8) -> &'static str { match n { 0 | 1 => \"low\", _ => \"high\" } }",
            &plain(),
        )
        .expect("lifts");
        assert!(
            out.contains("(if ((n =? 0) || (n =? 1)) then \"low\" else \"high\")"),
            "{out}"
        );
        let out = lift(
            "fn q(m: &str, n: &str) -> String { format!(\"{m}::{}\", n) }",
            &plain(),
        )
        .expect("lifts");
        assert!(
            out.contains("(prim join (prim append (cons m (nil String)) (cons \"::\" (cons n (nil String))) : (List String)) \"\" : String)"),
            "{out}"
        );
    }

    #[test]
    fn a_long_literal_is_balanced_halves() {
        // The linker reads a term to a bounded depth, and a literal is as
        // long as its author likes: no list nests one cell per element.
        let items: Vec<String> = (0..300).map(|i| (i % 250).to_string()).collect();
        let array = lift(
            &format!("fn table() -> [u8; 300] {{ [{}] }}", items.join(", ")),
            &plain(),
        )
        .expect("lifts");
        let bytes = lift(
            &format!("fn text() -> &'static [u8] {{ b\"{}\" }}", "ab".repeat(150)),
            &plain(),
        )
        .expect("lifts");
        let pieces = lift(
            &format!(
                "fn row(m: &str) -> String {{ format!(\"{}\") }}",
                "{m}, ".repeat(150)
            ),
            &plain(),
        )
        .expect("lifts");
        for out in [array, bytes, pieces] {
            assert!(out.contains("(prim append "), "{out}");
            let mut depth = 0usize;
            let mut deepest = 0usize;
            for c in out.chars() {
                match c {
                    '(' => {
                        depth += 1;
                        deepest = deepest.max(depth);
                    }
                    ')' => depth -= 1,
                    _ => {}
                }
            }
            assert!(deepest < 40, "nests {deepest} deep");
        }
    }

    #[test]
    fn a_guarded_row_and_the_rows_after_it_bind_distinct_names() {
        // The guarded row's binders wrap the whole conditional, so the row
        // after it, which binds the same Rust name to another part, is
        // written inside them: a binder there of the same name shadows.
        let out = lift(
            "fn k(p: (u8, u8)) -> u8 { match p { (0, d) | (d, 0) if d > 5 => d, (1, d) if d > 7 => d, _ => 2 } }",
            &plain(),
        )
        .expect("lifts");
        let lets: Vec<usize> = out.match_indices("(let ").map(|(at, _)| at).collect();
        assert!(lets.len() >= 2, "{out}");
        for at in lets {
            let name = out[at + 5..].split_whitespace().next().expect("a binder");
            let mut depth = 0usize;
            let end = out[at..]
                .char_indices()
                .find_map(|(i, c)| {
                    match c {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    (depth == 0).then_some(at + i)
                })
                .expect("a closed binder");
            assert!(
                !out[at + 5..end].contains(&format!("(let {name} ")),
                "{name} is bound again inside its own scope: {out}"
            );
        }
    }

    #[test]
    fn a_match_on_a_state_enum_lifts_exhaustively() {
        let mut types = plain();
        types
            .enums
            .insert("Mode".into(), vec!["Idle".into(), "On".into()]);
        let out = lift(
            "fn flip(m: Mode) -> Mode { match m { Mode::Idle => Mode::On, _ => Mode::Idle } }",
            &types,
        )
        .expect("lifts");
        assert!(
            out.contains("(match m | Mode.Idle => (ctor Mode.On) | Mode.On => (ctor Mode.Idle))"),
            "{out}"
        );
    }

    #[test]
    fn structs_and_tuples_lift() {
        let mut raw = BTreeMap::new();
        let ty = crate::convert::ty(&syn::parse_str("u32").expect("type"));
        raw.insert(
            "Band".to_string(),
            vec![("low".to_string(), ty.clone()), ("high".to_string(), ty)],
        );
        let types = Types::new(BTreeMap::new(), &raw, BTreeMap::new());
        let out = lift(
            "fn widen(b: Band) -> (u32, bool) { (b.high, b.low <= b.high) }",
            &types,
        )
        .expect("lifts");
        assert!(
            out.contains("(record TupleNatBool {item0 := (project b high), item1 := ((project b low) <=? (project b high))})"),
            "{out}"
        );
    }

    #[test]
    fn iterator_chains_lift_to_structural_helpers() {
        let out = lift(
            "fn over(xs: &[u32], floor: u32) -> bool { xs.iter().filter(|x| **x < floor).any(|&x| x == 0) }",
            &plain(),
        )
        .expect("lifts");
        assert!(out.contains("name: overFilter1,"), "{out}");
        assert!(out.contains("recursive_argument: items"), "{out}");
        assert!(out.contains("(overAny"), "{out}");
        let folded = lift(
            "fn total(xs: &[u32]) -> Option<u32> { xs.iter().fold(Some(0), |acc, &x| match acc { Some(a) => a.checked_add(x), None => None }) }",
            &plain(),
        )
        .expect("a fold step with a match lifts through a helper");
        assert!(folded.contains("name: totalPart1,"), "{folded}");
        let sum = lift("fn sum(xs: &[u32]) -> u32 { xs.iter().sum() }", &plain());
        assert!(matches!(sum, Err(Residue::Form(_))), "{sum:?}");
    }

    #[test]
    fn early_returns_and_tuple_lets_lift() {
        let out = lift(
            "fn pick(p: (u32, u32), c: bool) -> u32 { let (a, b) = p; if c { return a; } b }",
            &plain(),
        )
        .expect("lifts");
        assert!(out.contains("(let a : Nat := (project pair"), "{out}");
        assert!(out.contains("(if c then a else b)"), "{out}");
        let nested = lift(
            "fn nested(c: bool) -> u32 { let x = { if c { return 1; } 2 }; x }",
            &plain(),
        );
        assert!(
            nested.is_err(),
            "a return below the function's own statements: {nested:?}"
        );
    }

    #[test]
    fn maps_lift_only_where_their_order_is_defined() {
        let ordered = lift(
            "fn keys(m: &BTreeMap<String, u32>) -> Vec<String> { m.keys().cloned().collect() }",
            &plain(),
        )
        .expect("lifts");
        assert!(ordered.contains("(List TupleStringNat)"), "{ordered}");
        let hashed = lift(
            "fn keys(m: &HashMap<String, u32>) -> Vec<String> { m.keys().cloned().collect() }",
            &plain(),
        );
        assert!(matches!(hashed, Err(Residue::Form(_))), "{hashed:?}");
        let lookup = lift(
            "fn has(m: &HashMap<String, u32>, k: &str) -> bool { m.contains_key(k) }",
            &plain(),
        )
        .expect("lifts");
        assert!(lookup.contains("(prim equal (project item"), "{lookup}");
    }

    #[test]
    fn string_methods_lift_to_the_prelude() {
        let out = lift(
            "fn t(text: &str) -> Option<u32> { text.trim().parse::<u32>().ok() }",
            &plain(),
        )
        .expect("lifts");
        assert!(
            out.contains("(Prelude::textParseNat (Prelude::textTrim text) 4294967295)"),
            "{out}"
        );
        let out = lift(
            "fn t(text: &str) -> Option<(String, String)> { let (a, b) = text.split_once('=')?; Some((a.to_string(), b.to_string())) }",
            &plain(),
        )
        .expect("lifts");
        assert!(out.contains("(Prelude::textFirst text \"=\")"), "{out}");
        // `to_lowercase` lifts to the ASCII lowering, and its row records
        // the divergence rather than an axiom.
        let out = lift(
            "fn t(text: &str) -> String { text.to_lowercase() }",
            &plain(),
        )
        .expect("lifts");
        assert!(out.contains("(Prelude::textAsciiLower text)"), "{out}");
        assert!(!out.contains(DIVERGES), "{out}");
    }

    #[test]
    fn character_matches_lift_to_tests() {
        let out = lift(
            "fn k(c: char) -> u8 { match c { '-' => 1, '0'..='9' => 2, c if c.is_whitespace() => 3, _ => 4 } }",
            &plain(),
        )
        .expect("lifts");
        assert!(out.contains("(prim equal c \"-\" : Bool)"), "{out}");
        assert!(out.contains("(Prelude::textIsWhitespace c)"), "{out}");
    }

    #[test]
    fn loops_continue_break_and_enumerate() {
        let out = lift(
            "fn f(xs: &[u32]) -> u32 { let mut n: u32 = 0; for (i, x) in xs.iter().enumerate() { if *x == 0 { continue; } if i > 3 { break; } n = n.saturating_add(1); } n }",
            &plain(),
        )
        .expect("lifts");
        assert!(out.contains("fEnumerate"), "{out}");
        assert!(out.contains("fLoop"), "{out}");
    }

    #[test]
    fn a_question_mark_in_an_operand_hoists() {
        let out = lift(
            "fn f(o: Option<u32>) -> Option<u32> { Some(o?.min(3)) }",
            &plain(),
        )
        .expect("lifts");
        assert!(out.contains("Option.some tried"), "{out}");
    }

    #[test]
    fn an_empty_vector_takes_the_type_the_block_returns() {
        let out = lift(
            "fn f(xs: &[u32]) -> Vec<u32> { let mut out = Vec::new(); for x in xs { out.push(*x); } out }",
            &plain(),
        )
        .expect("lifts");
        assert!(out.contains("(nil Nat)"), "{out}");
        let early = lift(
            "fn g(n: u32) -> Vec<u32> { let out = Vec::with_capacity(4); if n > 2 { return out; } out }",
            &plain(),
        )
        .expect("lifts");
        assert!(early.contains("(nil Nat)"), "{early}");
    }

    #[test]
    fn an_index_is_the_element_or_the_default() {
        let out = lift("fn f(xs: &[u32]) -> u32 { xs[2] }", &plain()).expect("lifts");
        assert!(out.contains("(prim index xs 2 : (Option Nat))"), "{out}");
        assert!(out.contains("Option.none => 0"), "{out}");
    }

    // Types outside the lift.

    /// The types of one library source, registered as the projection
    /// registers them: every enum and struct of the file, by its own name.
    fn types_of(src: &str) -> Types {
        let (file, syntax) = crate::ingest::read_text(src);
        let mut enums = BTreeMap::new();
        let mut datas = BTreeMap::new();
        for (e, payload) in file.enums.iter().zip(&syntax.enums) {
            if e.variants.iter().all(|v| v.unit) {
                let variants = e.variants.iter().map(|v| v.name.clone()).collect();
                enums.insert(e.name.clone(), variants);
            } else {
                datas.insert(e.name.clone(), payload.clone());
            }
        }
        let structs = file
            .structs
            .iter()
            .map(|s| s.name.clone())
            .zip(syntax.structs.iter().cloned())
            .collect();
        Types::with_data(enums, &datas, &structs, BTreeMap::new())
    }

    /// A struct's fields, as the lift names and reads them.
    fn fields(types: &Types, name: &str) -> Vec<(String, Ty)> {
        types.structs.get(name).cloned().unwrap_or_default()
    }

    fn field(name: &str, ty: Ty) -> (String, Ty) {
        (name.to_string(), ty)
    }

    #[test]
    fn a_type_outside_the_lift_names_the_chain_to_its_leaf() {
        let types = types_of(
            "struct Leaf { x: UnsafeCell<u8> } struct Mid { n: u32, leaf: Leaf, other: UnsafeCell<u16> } \
             struct Top { mids: Vec<Option<Mid>> }",
        );
        // Through the containers the lift reads, down to the first field
        // in declaration order that is outside.
        assert_eq!(
            lift("fn through(t: &Top) -> bool { true }", &types),
            Err(Residue::Type(
                "Top.mids: Mid.leaf: Leaf.x: UnsafeCell<u8>".to_string()
            ))
        );
        // The owner of a method is named the same way; a name the
        // workspace does not declare is its own leaf.
        assert_eq!(
            types.unnamed("Mid"),
            Residue::Type("Mid.leaf: Leaf.x: UnsafeCell<u8>".to_string())
        );
        assert_eq!(
            types.unnamed("Elsewhere"),
            Residue::Type("Elsewhere".to_string())
        );
        assert_eq!(
            types.unnamed("Mid").detail(),
            "type outside the lift: Mid.leaf: Leaf.x: UnsafeCell<u8>"
        );
    }

    #[test]
    fn a_chain_never_leads_back_into_the_type_it_explains() {
        // `A` holds `B` first, and `B` is outside only because it holds
        // `A`: the chain of `A` names what kept `A` out on its own.
        let types = types_of("struct A { b: Vec<B>, x: UnsafeCell<u8> } struct B { a: Option<A> }");
        assert_eq!(types.outside["A"], "A.x: UnsafeCell<u8>");
        assert_eq!(types.outside["B"], "B.a: A.x: UnsafeCell<u8>");
        // Each of two types outside on its own account names the other
        // first, as declared, and then what keeps that one out.
        let types = types_of(
            "struct T { u: Vec<U>, x: UnsafeCell<u8> } struct U { t: Box<T>, y: UnsafeCell<u16> }",
        );
        assert_eq!(types.outside["T"], "T.u: U.y: UnsafeCell<u16>");
        assert_eq!(types.outside["U"], "U.t: T.x: UnsafeCell<u8>");
    }

    #[test]
    fn an_enum_names_the_variant_and_the_field_of_its_leaf() {
        let types = types_of(
            "enum Reading { Count(u32), Pair(u8, Cell<u8>), Level { level: Cell<u8> } } \
             enum Tree { Leaf, Node(BTreeMap<String, Tree>) } \
             struct Parent { kids: BTreeSet<Child> } struct Child { parent: Vec<Parent> }",
        );
        assert_eq!(types.outside["Reading"], "Reading::Pair.1: Cell<u8>");
        assert_eq!(
            types.outside["Tree"],
            format!("Tree::Node.0: BTreeMap<String, Tree>: {NOT_NESTED}")
        );
        // A family that names itself where Lean nests no inductive: the
        // member that holds the occurrence states it, the other names it.
        let held = format!("Parent.kids: BTreeSet<Child>: {NOT_NESTED}");
        assert_eq!(types.outside["Parent"], held);
        assert_eq!(
            types.outside["Child"],
            format!("Child: in one recursive family with {held}")
        );
    }

    // Functions refused by their signature: what `self` and `Self` name in
    // a method's signature, by the type that owns the method.

    fn method_signature(
        src: &str,
        owner: Option<&Owner<'_>>,
        types: &Types,
    ) -> Lift<(Vec<Param>, Ty)> {
        let f: syn::ImplItemFn = syn::parse_str(src).expect("parses");
        types.signature(&crate::convert::signature(&f.sig), owner)
    }

    fn with_square() -> Types {
        let mut types = Types::default();
        types.structs.insert("Square".to_string(), Vec::new());
        types
    }

    #[test]
    fn a_method_takes_the_value_of_its_type_first() {
        let types = with_square();
        for (src, receiver) in [
            ("fn area(&self, by: bool) -> bool { by }", "&self"),
            ("fn area(self, by: bool) -> bool { by }", "self"),
        ] {
            let owner = Owner::Type {
                name: "Square",
                receiver: Some(receiver),
            };
            let (params, result) = method_signature(src, Some(&owner), &types).expect("lifts");
            assert_eq!(params.len(), 2, "{src}");
            assert_eq!(
                params[0],
                (
                    "self".to_string(),
                    local_name("self"),
                    Ty::Named("Square".to_string())
                )
            );
            assert_eq!(result, Ty::Bool);
        }
        // `Self` is the type the method is of.
        let owner = Owner::Type {
            name: "Square",
            receiver: None,
        };
        let (params, result) = method_signature(
            "fn same(other: Self) -> Self { other }",
            Some(&owner),
            &types,
        )
        .expect("lifts");
        assert_eq!(params[0].2, Ty::Named("Square".to_string()));
        assert_eq!(result, Ty::Named("Square".to_string()));
    }

    #[test]
    fn a_receiver_the_lift_does_not_thread_is_named_with_its_type() {
        let types = with_square();
        let receiver = "self: Box<Self>";
        let owner = Owner::Type {
            name: "Square",
            receiver: Some(receiver),
        };
        let src = format!("fn grow({receiver}, by: bool) {{}}");
        assert_eq!(
            method_signature(&src, Some(&owner), &types).err(),
            Some(Residue::Signature(format!(
                "takes {receiver} of Square: the lift reads self, mut self, &self and &mut \
                 self, and no other receiver"
            )))
        );
        // A type the scope does not name, and a receiver outside any type.
        let owner = Owner::Type {
            name: "Circle",
            receiver: Some("&self"),
        };
        assert_eq!(
            method_signature("fn area(&self) -> bool { true }", Some(&owner), &types).err(),
            Some(Residue::Signature(
                "takes self of Circle, a type outside the lift".to_string()
            ))
        );
        assert_eq!(
            method_signature("fn area(&self) -> bool { true }", None, &types).err(),
            Some(Residue::Signature("takes self outside a type".to_string()))
        );
    }

    #[test]
    fn shared_ownership_reads_as_its_content() {
        let types = types_of(
            "struct Shared { inner: Arc<u32>, name: Rc<String>, deep: Arc<Vec<Rc<u8>>> } \
             struct Cellular { inner: Rc<RefCell<u32>> } struct Counted { hits: Cell<u32> } \
             struct Locked { state: Arc<Mutex<Vec<u8>>> }",
        );
        assert_eq!(
            fields(&types, "Shared"),
            [
                field("inner", Ty::Nat(Some(32))),
                field("name", Ty::Str),
                field("deep", Ty::List(Box::new(Ty::Nat(Some(8))))),
            ]
        );
        // Shared cells carry the allocation reference, not a snapshot.
        assert_eq!(
            fields(&types, "Cellular"),
            [field("inner", Ty::Named("Prelude::RcCellRef".into()))]
        );
        assert_eq!(
            fields(&types, "Counted"),
            [field("hits", Ty::Nat(Some(32)))]
        );
        assert_eq!(
            types.outside["Locked"],
            "Locked.state: Arc<Mutex<Vec<u8>>>: a Mutex that another owner shares, which sees \
             every change"
        );
        // Another owner, and a new one, are the content.
        let out = lift(
            "fn again(s: &Shared) -> Arc<u32> { Arc::clone(&s.inner) }",
            &types,
        )
        .expect("lifts");
        assert!(out.contains("body: (project s inner)"), "{out}");
        let out = lift("fn build(n: u32) -> Rc<u32> { Rc::new(n) }", &types).expect("lifts");
        assert!(out.contains("result: Nat, body: n"), "{out}");
        // A type the workspace declares under the name is not the
        // library's pointer.
        let own = types_of("struct Rc<T> { value: T } struct Holder { inner: Rc<u32> }");
        assert_eq!(
            own.outside["Holder"],
            format!("Holder.inner: Rc: {PARAMETRIC_STRUCT}")
        );
    }

    #[test]
    fn the_wide_signed_integers_lift_as_the_mathematical_int() {
        let types = types_of("struct Wide { n: i128, offset: isize }");
        assert_eq!(
            fields(&types, "Wide"),
            [
                field("n", Ty::Integer(Some(128))),
                field("offset", Ty::Integer(None)),
            ]
        );
        let out = lift(
            "fn below(a: i128, b: isize) -> bool { a < 0 || b >= 1 }",
            &types,
        )
        .expect("lifts");
        assert!(out.contains("parameters: [a : Int, b : Int]"), "{out}");
        assert!(out.contains("(prim less_than a (0 : Int) : Bool)"), "{out}");
        assert!(
            out.contains("(not (prim less_than b (1 : Int) : Bool))"),
            "{out}"
        );
        // `Int` has no `equal`: neither value is below the other.
        let out = lift("fn same(a: i128) -> bool { a == -1 }", &types).expect("lifts");
        assert!(
            out.contains("(not ((prim less_than a (-1 : Int) : Bool) || (prim less_than (-1 : Int) a : Bool)))"),
            "{out}"
        );
        // A checked operation is the mathematical one where 128 bits hold
        // its result.
        let out = lift(
            "fn sum(a: i128, b: i128) -> Option<i128> { a.checked_add(b) }",
            &types,
        )
        .expect("lifts");
        assert!(
            out.contains("(prim subtract a (prim negate b : Int) : Int)"),
            "{out}"
        );
        assert!(out.contains(&format!("(-{} : Int)", 1u128 << 127)), "{out}");
        assert!(out.contains(&format!("({} : Int)", i128::MAX)), "{out}");
        let out = lift(
            "fn quotient(a: i128, b: i128) -> Option<i128> { a.checked_div(b) }",
            &types,
        )
        .expect("lifts");
        assert!(out.contains("(prim quotient a b (0 : Int) : Int)"), "{out}");
        // The width of `isize` is the target's, as `usize`'s is.
        assert_eq!(
            lift(
                "fn sum(a: isize, b: isize) -> Option<isize> { a.checked_add(b) }",
                &types
            ),
            Err(Residue::Width(".checked_add on isize".to_string()))
        );
        // A signed value widens as itself: to `i128` always, to `isize`
        // where the narrowest declared pointer holds it.
        let out = lift("fn widen(a: i64) -> i128 { a as i128 }", &types).expect("lifts");
        assert!(out.contains("(prim to_int a : Int)"), "{out}");
        let out = lift("fn widen(a: i8) -> isize { a as isize }", &types).expect("lifts");
        assert!(out.contains("(prim to_int a : Int)"), "{out}");
        let narrowing = Err(Residue::Form("narrowing or sign-changing cast".to_string()));
        assert_eq!(
            lift("fn widen(a: i32) -> isize { a as isize }", &types),
            narrowing
        );
        assert_eq!(
            lift("fn narrow(a: i128) -> i64 { a as i64 }", &types),
            narrowing
        );
        assert_eq!(
            default_of(&Ty::Integer(None), &types),
            Some("(0 : Int)".to_string())
        );
    }

    #[test]
    fn every_struct_shape_is_a_type_or_names_its_leaf() {
        let types = types_of(
            "struct Marker; struct Empty {} struct Meters(u32); struct Pair(u32, UnsafeCell<u8>); \
             struct Wrapper<T> { value: T } struct Holder { m: Marker, w: Wrapper<u32> }",
        );
        // A struct of no fields has one value: one constructor.
        assert_eq!(types.enums["Marker"], [SOLE]);
        assert_eq!(types.enums["Empty"], [SOLE]);
        let out = lift("fn make() -> Marker { Marker }", &types).expect("lifts");
        assert!(out.contains("body: (ctor Marker.mk)"), "{out}");
        // A tuple struct's fields are named by position.
        assert_eq!(
            fields(&types, "Meters"),
            [field("item0", Ty::Nat(Some(32)))]
        );
        let out = lift("fn first(m: &Meters) -> u32 { m.0 }", &types).expect("lifts");
        assert!(out.contains("body: (project m item0)"), "{out}");
        let out = lift("fn make(n: u32) -> Meters { Meters(n) }", &types).expect("lifts");
        assert!(out.contains("body: (record Meters {item0 := n})"), "{out}");
        assert_eq!(types.outside["Pair"], "Pair.1: UnsafeCell<u8>");
        // A struct with a type parameter is the leaf.
        assert_eq!(
            types.outside["Wrapper"],
            format!("Wrapper: {PARAMETRIC_STRUCT}")
        );
        assert_eq!(
            types.outside["Holder"],
            format!("Holder.w: Wrapper: {PARAMETRIC_STRUCT}")
        );
    }

    #[test]
    fn a_lifetime_is_erased_and_a_taken_name_is_named() {
        let types = types_of("struct Reader<'a> { text: &'a str } struct Prop { weight: u8 }");
        // The owner of `impl<'a> Reader<'a>` is the struct.
        assert!(types.structs.contains_key("Reader"));
        assert_eq!(types.resolve("Reader<'a>"), "Reader");
        assert_eq!(types.resolve("Reader<'_, 'b>"), "Reader");
        // A type or const argument is not erased.
        assert_eq!(types.resolve("Wrapper<T>"), "Wrapper<T>");
        assert_eq!(types.resolve("Letter<'a'>"), "Letter<'a'>");
        // A name LexLean writes for a type of its own does not lift as
        // itself; the projection lifts it under its crate's name.
        assert_eq!(types.outside["Prop"], format!("Prop: {TAKEN_NAME}"));
        assert!(name_taken("Prop", true));
        assert!(name_taken("TupleNat", true));
        assert!(!name_taken("TupleNat", false));
        assert!(!name_taken("Reader", true));
        let (file, syntax) = crate::ingest::read_text("struct Prop { weight: u8 }");
        let structs = BTreeMap::from([("StageProp".to_string(), syntax.structs[0].clone())]);
        let aliases = BTreeMap::from([(file.structs[0].name.clone(), "StageProp".to_string())]);
        let named = Types::with_data(BTreeMap::new(), &BTreeMap::new(), &structs, aliases);
        let out = lift("fn weight(p: &Prop) -> u8 { p.weight }", &named).expect("lifts");
        assert!(out.contains("parameters: [p : StageProp]"), "{out}");
    }

    #[test]
    fn library_types_read_by_name_or_by_their_full_path() {
        let types = types_of(
            "struct Window { bounds: Range<u32>, queue: VecDeque<u8>, \
             index: std::collections::BTreeMap<String, u32>, text: std::string::String, \
             shared: alloc::sync::Arc<core::option::Option<u8>>, at: std::path::PathBuf } \
             struct Borrowed { cell: std::cell::OnceCell<u8> } struct Misplaced { rc: std::fmt::Rc<u8> }",
        );
        let nat = |bits: u32| Box::new(Ty::Nat(Some(bits)));
        assert_eq!(
            fields(&types, "Window"),
            [
                field("bounds", Ty::Tuple(vec![*nat(32), *nat(32)])),
                field("queue", Ty::List(nat(8))),
                field("index", Ty::Map(Box::new(Ty::Str), nat(32), true)),
                field("text", Ty::Str),
                field("shared", Ty::Option(nat(8))),
                field("atValue", Ty::Path),
            ]
        );
        // A full path the lift does not read is the leaf, as written.
        assert_eq!(
            types.outside["Borrowed"],
            "Borrowed.cell: std::cell::OnceCell<u8>"
        );
        assert_eq!(types.outside["Misplaced"], "Misplaced.rc: std::fmt::Rc<u8>");
    }

    #[test]
    fn a_default_method_takes_no_value_of_an_implementing_type() {
        let types = with_square();
        let (params, result) = method_signature(
            "fn corners(&self, sharp: bool) -> bool { sharp }",
            Some(&Owner::Implementor),
            &types,
        )
        .expect("lifts");
        assert_eq!(
            params,
            [("sharp".to_string(), local_name("sharp"), Ty::Bool)]
        );
        assert_eq!(result, Ty::Bool);
        // `Self` there is each implementing type: no one type.
        assert!(method_signature(
            "fn same(&self) -> Self { loop {} }",
            Some(&Owner::Implementor),
            &types
        )
        .is_err());
        // And a method is generic as a function is.
        assert_eq!(
            method_signature(
                "fn map<T>(&self, t: T) -> T { t }",
                Some(&Owner::Implementor),
                &types
            )
            .err(),
            Some(Residue::Signature(
                "generic over type parameters T".to_string()
            ))
        );
    }

    // residue: the translator's own functions, rewritten inside the fragment
    // the tool lifts from itself; each test pins what the old body did.

    fn residue_path(src: &str) -> rs::Path {
        crate::convert::path(&syn::parse_str::<syn::Path>(src).expect("parses"))
    }

    fn residue_ty(src: &str) -> rs::RustType {
        crate::convert::ty(&syn::parse_str::<syn::Type>(src).expect("parses"))
    }

    #[test]
    fn residue_a_unit_value_is_named_by_its_type_or_its_variant() {
        let types = types_of(
            "struct Mark; enum Two { A, B } enum Only { One } \
             enum Mixed { Bare, Held(u8) } enum Never {}",
        );
        let ctor_of = |src: &str| ctor(&residue_path(src), &types);
        let named = |o: &str, v: &str| Some((o.to_string(), v.to_string()));
        // A struct of no fields: its one value, under the name alone.
        assert_eq!(ctor_of("Mark"), named("Mark", SOLE));
        // An enum's name alone is no value, whatever its variants.
        assert_eq!(ctor_of("Two"), None);
        assert_eq!(ctor_of("Only"), None);
        assert_eq!(ctor_of("Mixed"), None);
        // An enum of no variants has no value to name.
        assert_eq!(ctor_of("Never"), None);
        // A unit variant, by the last two segments.
        assert_eq!(ctor_of("Two::A"), named("Two", "A"));
        assert_eq!(ctor_of("crate::Two::B"), named("Two", "B"));
        assert_eq!(ctor_of("Only::One"), named("Only", "One"));
        assert_eq!(ctor_of("Mixed::Bare"), named("Mixed", "Bare"));
        // A variant that carries data, or none of the enum's, is no unit.
        assert_eq!(ctor_of("Mixed::Held"), None);
        assert_eq!(ctor_of("Two::C"), None);
        assert_eq!(ctor_of("Missing::A"), None);
        assert_eq!(ctor_of("Missing"), None);
    }

    #[test]
    fn residue_a_library_path_is_plain_full_and_of_a_type_the_lift_reads() {
        for full in [
            "std::collections::BTreeMap",
            "core::option::Option",
            "alloc::sync::Arc",
            "std::vec::Vec",
            "std::path::PathBuf",
            // The last segment's arguments are the type's own.
            "std::collections::HashMap<u8, u8>",
        ] {
            assert!(std_path(&residue_path(full)), "{full}");
        }
        for other in [
            "std::fmt::Rc",
            "std::collections::Rc",
            "mine::collections::BTreeMap",
            "collections::BTreeMap",
            "std::collections::btree_map::BTreeMap",
            "BTreeMap",
        ] {
            assert!(!std_path(&residue_path(other)), "{other}");
        }
        // Arguments on the root or the module make it no library path.
        for at in [0, 1] {
            let mut path = residue_path("std::collections::BTreeMap");
            path.segments[at].arguments = rs::PathArguments::Parenthesized;
            assert!(!std_path(&path), "arguments at {at}");
        }
    }

    #[test]
    fn residue_a_field_named_by_position_is_an_item() {
        assert_eq!(field_name("0"), "item0");
        assert_eq!(field_name("12"), "item12");
        // What `str::parse` accepts as a number reads as one.
        assert_eq!(field_name("+3"), "item3");
        assert_eq!(field_name("007"), "item7");
        assert_eq!(field_name("4294967295"), "item4294967295");
        // A name is the local name of its identifier.
        assert_eq!(field_name("name"), local_name("name"));
        assert_eq!(field_name("r#type"), local_name("r#type"));
        assert_eq!(field_name("-1"), local_name("-1"));
        assert_eq!(field_name(""), local_name(""));
    }

    #[test]
    fn residue_fields_and_variants_read_only_where_every_one_lifts() {
        let types = types_of("struct Leaf { n: u8 }");
        let raw = |fields: &[(&str, &str)]| -> Vec<(String, rs::RustType)> {
            fields
                .iter()
                .map(|(f, t)| (f.to_string(), residue_ty(t)))
                .collect()
        };
        assert_eq!(
            types.fields_read(&raw(&[("0", "u8"), ("1", "String")])),
            Some(vec![
                field("item0", Ty::Nat(Some(8))),
                field("item1", Ty::Str)
            ])
        );
        assert_eq!(
            types.fields_read(&raw(&[("leaf", "Leaf"), ("flag", "bool")])),
            Some(vec![
                field(&local_name("leaf"), Ty::Named("Leaf".to_string())),
                field(&local_name("flag"), Ty::Bool)
            ])
        );
        assert_eq!(types.fields_read(&[]), Some(Vec::new()));
        // One field outside, first or last, keeps the whole out.
        assert_eq!(
            types.fields_read(&raw(&[("a", "std::cell::UnsafeCell<u8>"), ("b", "u8")])),
            None
        );
        assert_eq!(
            types.fields_read(&raw(&[("a", "u8"), ("b", "std::cell::UnsafeCell<u8>")])),
            None
        );
        let variant = |v: &str, fields: &[(Option<&str>, &str)]| -> RawVariant {
            let fields = fields
                .iter()
                .map(|(f, t)| (f.map(str::to_string), residue_ty(t)))
                .collect();
            (v.to_string(), fields)
        };
        let mut own = types.clone();
        own.datas.insert("Tree".to_string(), Vec::new());
        let read = own.variants_read(
            "Tree",
            &[
                variant("Leaf", &[]),
                variant("Node", &[(None, "Vec<Tree>"), (Some("tag"), "u8")]),
            ],
        );
        assert_eq!(
            read,
            Some(vec![
                ("Leaf".to_string(), Vec::new()),
                (
                    "Node".to_string(),
                    vec![
                        (None, Ty::List(Box::new(Ty::Named("Tree".to_string())))),
                        (Some("tag".to_string()), Ty::Nat(Some(8)))
                    ]
                )
            ])
        );
        assert_eq!(own.variants_read("Tree", &[]), Some(Vec::new()));
        // A field outside the lift, or the enum inside a map where Lean
        // nests no inductive, keeps the whole out.
        for bad in ["std::cell::UnsafeCell<u8>", "BTreeMap<u8, Tree>"] {
            let raw = [variant("Leaf", &[]), variant("Node", &[(None, bad)])];
            assert_eq!(own.variants_read("Tree", &raw), None, "{bad}");
        }
    }

    // Registry number types.

    /// A scope where the package's lock admits the big integer reading and
    /// the file's `use` binds `BigInt` to it.
    fn registry_number_scope() -> Types {
        let path = "num_bigint::BigInt".to_string();
        Types {
            aliases: BTreeMap::from([("BigInt".to_string(), path.clone()), (path.clone(), path)]),
            ..Types::default()
        }
    }

    fn registry_number_lift(src: &str, imports: &[(&str, &str)]) -> Result<String, Residue> {
        let types = registry_number_scope();
        let f: syn::ItemFn = syn::parse_str(src).expect("parses");
        let sig = crate::convert::signature(&f.sig);
        let block = crate::convert::block(&f.block);
        let (params, result) = types.signature(&sig, None)?;
        let c = Candidate {
            threads: threads(&sig, None),
            generic: None,
            rust: sig.ident.clone(),
            name: camel(&[&sig.ident]),
            params,
            result,
            block: &block,
            imports: imports
                .iter()
                .map(|(n, p)| (n.to_string(), p.to_string()))
                .collect(),
            aliases: BTreeMap::new(),
            homonyms: BTreeMap::new(),
            called: sig.ident.clone(),
            formatter: None,
            seen: Seen {
                uses: imports
                    .iter()
                    .map(|(n, p)| (n.to_string(), p.to_string()))
                    .collect(),
                ..Seen::default()
            },
        };
        let out = lift_all(&[c], &types, &pointer_widths(&[]), &BTreeSet::new());
        match out.failed.into_values().next() {
            Some(r) => Err(r),
            None => Ok(out
                .decls
                .iter()
                .map(crate::lex::render_decl)
                .collect::<Vec<_>>()
                .join("\n")),
        }
    }

    #[test]
    fn residue_a_struct_value_is_a_record_or_its_family_constructor() {
        let types = types_of("struct P { a: u8, b: u8 }");
        let (a, b) = ("a".to_string(), "b".to_string());
        let parts = [(&a, "1".to_string()), (&b, "2".to_string())];
        assert_eq!(
            struct_value("P", &parts, &types),
            "(record P {a := 1, b := 2})"
        );
        let mut family = types.clone();
        family.families.push(BTreeSet::from(["P".to_string()]));
        assert_eq!(struct_value("P", &parts, &family), "(ctor P.mk 1 2)");
        assert_eq!(struct_value("P", &[], &family), "(ctor P.mk)");
    }

    #[test]
    fn a_registered_big_integer_is_an_int_where_the_lock_admits_it() {
        let types = registry_number_scope();
        let named = |text: &str| {
            let f: syn::ItemFn = syn::parse_str(&format!("fn f(x: {text}) {{}}")).expect("parses");
            let sig = crate::convert::signature(&f.sig);
            types
                .signature(&sig, None)
                .map(|(params, _)| params[0].2.clone())
        };
        assert_eq!(named("BigInt"), Ok(Ty::Unbounded));
        assert_eq!(named("num_bigint::BigInt"), Ok(Ty::Unbounded));
        assert_eq!(named("&::num_bigint::BigInt"), Ok(Ty::Unbounded));
        assert_eq!(
            named("Vec<BigInt>").map(|t| t.render()),
            Ok("(List Int)".to_string())
        );
        // Without the lock's admission the name is the leaf, as written.
        let f: syn::ItemFn = syn::parse_str("fn f(x: BigInt) {}").expect("parses");
        let sig = crate::convert::signature(&f.sig);
        assert_eq!(
            Types::default().signature(&sig, None).err(),
            Some(Residue::Type("BigInt".to_string()))
        );
        let f: syn::ItemFn = syn::parse_str("fn f(x: num_bigint::BigUint) {}").expect("parses");
        let sig = crate::convert::signature(&f.sig);
        assert_eq!(
            types.signature(&sig, None).err(),
            Some(Residue::Type("num_bigint::BigUint".to_string()))
        );
    }

    // Mutation as state passing: an update is a new binding of the local
    // for the statements after it, so it is read only where those
    // statements are part of the expression that makes it.

    fn mutation_refused_as_lost(src: &str, local: &str) {
        assert_eq!(
            lift(src, &plain()),
            Err(Residue::Form(format!(
                "an update of {local} inside an operand, which the statements after it do not see"
            ))),
            "{src}"
        );
    }

    #[test]
    fn a_registered_big_integer_computes_exactly() {
        let lifted = registry_number_lift(
            "fn mix(a: BigInt, b: &BigInt) -> BigInt { -(a.clone() * b + 2) / 3 % 5 }",
            &[],
        )
        .expect("lifts");
        assert!(
            lifted.contains(
                "(prim remainder (prim quotient (prim negate (prim subtract \
                 (prim multiply a b : Int) (prim negate (2 : Int) : Int) : Int) : Int) \
                 (3 : Int) (0 : Int) : Int) (5 : Int) (0 : Int) : Int)"
            ),
            "{lifted}"
        );
        let lifted =
            registry_number_lift("fn below(a: &BigInt, b: &BigInt) -> bool { a < b }", &[])
                .expect("lifts");
        assert!(lifted.contains("(prim less_than a b : Bool)"), "{lifted}");
    }

    #[test]
    fn a_registered_big_integer_is_built_from_what_the_crate_defines() {
        let lifted = registry_number_lift("fn of(n: i64) -> BigInt { BigInt::from(n) }", &[])
            .expect("lifts");
        assert!(lifted.contains("(prim to_int n : Int)"), "{lifted}");
        let lifted =
            registry_number_lift("fn none() -> BigInt { BigInt::default() }", &[]).expect("lifts");
        assert!(lifted.contains("(0 : Int)"), "{lifted}");
        // `zero` and `one` are the trait's where the file brings it into
        // scope, and no function of the lift otherwise.
        let zero = "fn none() -> BigInt { BigInt::zero() }";
        let lifted = registry_number_lift(zero, &[("Zero", "num_traits::Zero")]).expect("lifts");
        assert!(lifted.contains("(0 : Int)"), "{lifted}");
        assert!(registry_number_lift(zero, &[]).is_err());
        let one = "fn unit() -> BigInt { BigInt::one() }";
        let lifted =
            registry_number_lift(one, &[("One", "num_traits::identities::One")]).expect("lifts");
        assert!(lifted.contains("(1 : Int)"), "{lifted}");
        // The width-driven binary reading supplies the unsigned conversion.
        assert!(registry_number_lift("fn of(n: u32) -> BigInt { BigInt::from(n) }", &[]).is_ok());
    }

    #[test]
    fn mutation_an_update_inside_an_operand_is_not_lost() {
        // A `let`'s value, an argument and a closure's body are operands:
        // the statements after them would read the local as it was.
        mutation_refused_as_lost(
            "fn f(c: bool) -> u32 { let mut a = 0u32; \
             let r = if c { a = 1; 5 } else { 6 }; a.saturating_add(r) }",
            "a",
        );
        mutation_refused_as_lost(
            "fn f(c: Option<u32>) -> u32 { let mut a = 0u32; \
             let r = match c { Some(v) => { a = v; 1 } None => 2 }; a.saturating_add(r) }",
            "a",
        );
        mutation_refused_as_lost(
            "fn f() -> u32 { let mut a = 0u32; let b = a.saturating_add({ a = 3; 1 }); \
             a.saturating_add(b) }",
            "a",
        );
        mutation_refused_as_lost(
            "fn f(xs: Vec<u32>) -> u32 { let mut n = 0u32; \
             let ys: Vec<u32> = xs.iter().map(|x| { n = 7; *x }).collect(); n }",
            "n",
        );
        mutation_refused_as_lost(
            "fn f(xs: Vec<u32>) -> u32 { let mut n = 0u32; \
             let m = { for x in xs { n = x; } 1u32 }; n.saturating_add(m) }",
            "n",
        );
    }

    #[test]
    fn a_registered_ratio_is_the_pair_it_stores() {
        let mut types = registry_number_scope();
        let ratio = "num_rational::BigRational".to_string();
        types.aliases.insert("BigRational".into(), ratio.clone());
        types.aliases.insert(ratio.clone(), ratio);
        let lift = |src: &str, imports: &[(&str, &str)]| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parses");
            let sig = crate::convert::signature(&f.sig);
            let block = crate::convert::block(&f.block);
            let (params, result) = types.signature(&sig, None)?;
            let c = Candidate {
                threads: threads(&sig, None),
                generic: None,
                rust: sig.ident.clone(),
                name: camel(&[&sig.ident]),
                params,
                result,
                block: &block,
                imports: imports
                    .iter()
                    .map(|(n, p)| (n.to_string(), p.to_string()))
                    .collect(),
                aliases: BTreeMap::new(),
                homonyms: BTreeMap::new(),
                called: sig.ident.clone(),
                formatter: None,
                seen: Seen {
                    uses: imports
                        .iter()
                        .map(|(n, p)| (n.to_string(), p.to_string()))
                        .collect(),
                    ..Seen::default()
                },
            };
            let out = lift_all(&[c], &types, &pointer_widths(&[]), &BTreeSet::new());
            match out.failed.into_values().next() {
                Some(r) => Err(r),
                None => Ok(out
                    .decls
                    .iter()
                    .map(crate::lex::render_decl)
                    .collect::<Vec<_>>()
                    .join("\n")),
            }
        };
        let lifted = lift(
            "fn raw(n: BigInt, d: BigInt) -> BigRational { BigRational::new_raw(n, d) }",
            &[],
        )
        .expect("lifts");
        assert!(lifted.contains("result: (Int * Int)"), "{lifted}");
        assert!(lifted.contains("(n, d)"), "{lifted}");
        let lifted = lift(
            "fn whole(n: BigInt) -> BigRational { BigRational::from(n) }",
            &[],
        )
        .expect("lifts");
        assert!(lifted.contains("(n, (1 : Int))"), "{lifted}");
        let lifted = lift(
            "fn unit() -> BigRational { BigRational::one() }",
            &[("One", "num_traits::One")],
        )
        .expect("lifts");
        assert!(lifted.contains("((1 : Int), (1 : Int))"), "{lifted}");
        let lifted = lift(
            "fn top(r: &BigRational) -> BigInt { r.numer().clone() }",
            &[],
        )
        .expect("lifts");
        assert!(lifted.contains("(first r)"), "{lifted}");
        // The reciprocal panics on a zero numerator: it is not read.
        assert!(lift("fn flip(r: &BigRational) -> BigRational { r.recip() }", &[]).is_err());
        // Two stored pairs may be one value: no comparison is read, and
        // what reduces stays outside.
        assert!(matches!(
            lift(
                "fn same(a: &BigRational, b: &BigRational) -> bool { a == b }",
                &[]
            ),
            Err(Residue::Form(_))
        ));
        assert!(matches!(
            lift(
                "fn reduced(n: BigInt, d: BigInt) -> BigRational { BigRational::new(n, d) }",
                &[]
            ),
            Err(Residue::Form(_))
        ));
    }

    #[test]
    fn a_float_is_ieee_754s_binary_format_core_states() {
        let f64_format = Types::default().float("f64").expect("binary64");
        assert_eq!((f64_format.exp, f64_format.frac), (11, 52));
        let f32_format = Types::default().float("f32").expect("binary32");
        assert_eq!((f32_format.exp, f32_format.frac), (8, 23));
        let lifted = lift(
            "fn mix(a: f64, b: f64) -> f64 { -(a * b + 0.5) / a % b }",
            &plain(),
        )
        .expect("lifts");
        let add = format!(
            "(Prelude::floatAdd 11 52 (Prelude::floatMul 11 52 a b) {})",
            crate::mir::float::literal(f64_format, "0.5").expect("a literal")
        );
        assert!(lifted.contains(&add), "{lifted}");
        assert!(
            lifted.contains("(Prelude::floatRem 11 52 (Prelude::floatDiv"),
            "{lifted}"
        );
        assert!(lifted.contains("(Prelude::floatNeg 11 52 "), "{lifted}");
        let lifted = lift(
            "fn below(a: f32, b: f32) -> bool { a < b && !b.is_nan() }",
            &plain(),
        )
        .expect("lifts");
        assert!(lifted.contains("(Prelude::floatLt 8 23 a b)"), "{lifted}");
        assert!(lifted.contains("(Prelude::floatIsNan 8 23 b)"), "{lifted}");
        let lifted = lift("fn mag(a: f64) -> f64 { a.abs() }", &plain()).expect("lifts");
        assert!(lifted.contains("(Prelude::floatAbs 11 52 a)"), "{lifted}");
        // Two formats do not mix, and a method the model has not is no
        // definition.
        assert!(lift("fn sin(a: f64) -> f64 { a.sin() }", &plain()).is_err());
    }

    // Class: calls the language resolves by type (kai, round 3): a
    // signature the lift does not read says which part.

    #[test]
    fn an_unread_signature_names_each_part_that_stops_it() {
        let types = Types::default();
        let reason = |text: &str| method_signature(text, None, &types).err();
        let row = |s: &str| Some(Residue::Signature(s.to_string()));
        assert_eq!(
            reason("fn pick<T, U, const N: usize>(t: T) -> u8 { 1 }"),
            row("generic over type parameters T, U, generic over const parameters N")
        );
        assert_eq!(reason("async fn wait() -> u8 { 1 }"), row("async"));
        assert_eq!(reason("unsafe fn raw() -> u8 { 1 }"), row("unsafe"));
        assert_eq!(
            reason("async unsafe fn both() -> u8 { 1 }"),
            row("async, unsafe")
        );
        // A lifetime parameter is erased.
        assert!(!matches!(
            reason("fn first<'a>(x: &'a u8) -> u8 { *x }"),
            Some(Residue::Signature(_))
        ));
    }

    // Class: refuted readings (repair of round 4's confirmed counterexamples).

    /// A lift of `src` in `types` where the function's scope is `seen`.
    fn refuted_lift(src: &str, types: &Types, seen: Seen) -> Result<String, Residue> {
        let f: syn::ItemFn = syn::parse_str(src).expect("parses");
        let sig = crate::convert::signature(&f.sig);
        let block = crate::convert::block(&f.block);
        let (params, result) = types.signature(&sig, None)?;
        let c = Candidate {
            threads: threads(&sig, None),
            generic: None,
            rust: sig.ident.clone(),
            name: camel(&[&sig.ident]),
            params,
            result,
            block: &block,
            imports: seen.uses.clone(),
            aliases: BTreeMap::new(),
            homonyms: BTreeMap::new(),
            called: sig.ident.clone(),
            formatter: None,
            seen,
        };
        let out = lift_all(&[c], types, &pointer_widths(&[]), &BTreeSet::new());
        match out.failed.into_values().next() {
            Some(r) => Err(r),
            None => Ok(out
                .decls
                .iter()
                .map(crate::lex::render_decl)
                .collect::<Vec<_>>()
                .join("\n")),
        }
    }

    /// The scope of the registry number tests, with the ratio read too.
    fn refuted_scope() -> Types {
        let ratio = "num_rational::BigRational".to_string();
        let mut types = registry_number_scope();
        types.aliases.insert("BigRational".into(), ratio.clone());
        types.aliases.insert(ratio.clone(), ratio);
        types
    }

    /// A scope where traits of the workspace declaring `methods` may be in
    /// scope.
    fn refuted_traits(methods: &[&str]) -> Seen {
        Seen {
            trait_methods: methods.iter().map(|m| m.to_string()).collect(),
            ..Seen::default()
        }
    }

    #[test]
    fn a_big_integer_divides_only_by_a_divisor_that_is_not_zero() {
        let types = refuted_scope();
        let by = |src: &str| refuted_lift(src, &types, Seen::default());
        assert!(by("fn q(a: BigInt) -> BigInt { a / BigInt::from(2i64) }").is_err());
        assert!(by("fn q(a: BigInt, b: BigInt) -> BigInt { a / b }").is_err());
        assert!(by("fn r(a: BigInt, b: BigInt) -> BigInt { a % b }").is_err());
        assert!(by("fn r(a: BigInt) -> BigInt { a % 0 }").is_err());
        let lifted = by("fn h(a: BigInt) -> BigInt { a / 2 }").expect("lifts");
        assert!(
            lifted.contains("(prim quotient a (2 : Int) (0 : Int) : Int)"),
            "{lifted}"
        );
    }

    #[test]
    fn a_ratio_reciprocal_and_a_trait_named_accessor_are_not_read() {
        let flip = "fn flip(r: BigRational) -> BigRational { r.recip() }";
        assert!(refuted_lift(flip, &refuted_scope(), Seen::default()).is_err());
        let top = "fn top(r: BigRational) -> BigInt { r.numer().clone() }";
        assert!(refuted_lift(top, &refuted_scope(), Seen::default()).is_ok());
        // A trait of the workspace declaring `numer`, or one from outside
        // it the scope may import, may be what the call reaches.
        assert!(refuted_lift(top, &refuted_scope(), refuted_traits(&["numer"])).is_err());
        let outside = Seen {
            outside_traits: true,
            ..Seen::default()
        };
        assert!(refuted_lift(top, &refuted_scope(), outside).is_err());
    }

    #[test]
    fn a_float_method_a_trait_may_take_is_not_read() {
        let abs = "fn mag(x: &f64) -> f64 { x.abs() }";
        assert!(refuted_lift(abs, &plain(), Seen::default()).is_ok());
        assert!(refuted_lift(abs, &plain(), refuted_traits(&["abs"])).is_err());
        let outside = Seen {
            outside_traits: true,
            ..Seen::default()
        };
        assert!(refuted_lift(abs, &plain(), outside).is_err());
    }

    #[test]
    fn an_identity_is_read_only_through_its_own_scopes_import_of_the_crate() {
        let zero = "fn z() -> BigInt { BigInt::zero() }";
        let uses = |local: &[&str]| Seen {
            uses: BTreeMap::from([("Zero".to_string(), "num_traits::Zero".to_string())]),
            local: local.iter().map(|n| n.to_string()).collect(),
            ..Seen::default()
        };
        let lifted = refuted_lift(zero, &refuted_scope(), uses(&[])).expect("lifts");
        assert!(lifted.contains("(0 : Int)"), "{lifted}");
        // A module `num_traits` of the scope is what the path names.
        assert!(refuted_lift(zero, &refuted_scope(), uses(&["num_traits"])).is_err());
        // A trait of the workspace declaring `zero` may be in scope.
        let traits = Seen {
            trait_methods: BTreeSet::from(["zero".to_string()]),
            ..uses(&[])
        };
        assert!(refuted_lift(zero, &refuted_scope(), traits).is_err());
        // Another module's import does not reach this scope.
        assert!(refuted_lift(zero, &refuted_scope(), Seen::default()).is_err());
        // Nor where a macro of the package makes items.
        let made = Seen {
            macro_items: true,
            ..uses(&[])
        };
        assert!(refuted_lift(zero, &refuted_scope(), made).is_err());
    }

    #[test]
    fn a_float_literal_takes_its_format_from_its_use() {
        let lifted = lift("fn s() -> f32 { 0.1 + 0.2 }", &plain()).expect("lifts");
        assert!(lifted.contains("(Prelude::floatAdd 8 23 "), "{lifted}");
        assert!(!lifted.contains("11 52"), "{lifted}");
        let lifted = lift("fn t() -> f32 { let y: f32 = 0.1 + 0.2; y }", &plain()).expect("lifts");
        assert!(lifted.contains("(Prelude::floatAdd 8 23 "), "{lifted}");
        // A literal no use types, and a binary64 value where a binary32 is
        // returned or bound.
        assert!(lift("fn u() -> f32 { let v = 0.1; v }", &plain()).is_err());
        assert!(lift("fn w() -> f32 { 0.5f64 }", &plain()).is_err());
        assert!(lift("fn x(a: f64) -> f32 { let y: f32 = a; y }", &plain()).is_err());
        assert!(lift("fn n(a: f64) -> u64 { a }", &plain()).is_err());
    }

    #[test]
    fn what_reads_a_nans_sign_or_a_zeros_sign_of_min_and_max_is_not_read() {
        for src in [
            "fn b(a: f64, c: f64) -> u64 { (a / c).to_bits() }",
            "fn s(a: f64, c: f64) -> bool { (a / c).is_sign_negative() }",
            "fn p(a: f64) -> bool { a.is_sign_positive() }",
            "fn c(a: f64, c: f64) -> f64 { a.copysign(c) }",
            "fn m(a: f64, c: f64) -> f64 { a.min(c) }",
            "fn x(a: f64, c: f64) -> f64 { a.max(c) }",
            "fn z(a: f64) -> f64 { a.max(0.0) }",
        ] {
            assert!(lift(src, &plain()).is_err(), "{src}");
        }
        // One operand a literal that is not zero: the two are not both zeros.
        let lifted = lift("fn k(a: f64) -> f64 { a.max(1.0) }", &plain()).expect("lifts");
        assert!(
            lifted.contains("(Prelude::floatMaxNum 11 52 a "),
            "{lifted}"
        );
    }

    #[test]
    fn a_primitive_name_a_type_of_the_crate_takes_is_not_read() {
        let types = types_of("pub struct f32(pub u8);");
        assert!(lift("fn w(x: f32) -> bool { true }", &types).is_err());
        assert!(lift("fn v(x: f64) -> bool { true }", &types).is_ok());
    }

    #[test]
    fn mutation_an_update_the_statements_after_it_read_lifts() {
        for src in [
            "fn f(c: bool) -> u32 { let mut a = 0u32; if c { a = 1; } a }",
            "fn f(xs: Vec<u32>) -> u32 { let mut n = 0u32; \
             for x in xs { n = n.saturating_add(x); } n }",
            // A local the operand binds is its own to update.
            "fn f() -> u32 { let r = { let mut b = 2u32; b = 3; b }; r }",
            "fn f() -> u32 { let a = 1u32; let r = { let mut a = 2u32; a = 3; a }; \
             a.saturating_add(r) }",
        ] {
            assert!(
                lift(src, &plain()).is_ok(),
                "{src}: {:?}",
                lift(src, &plain())
            );
        }
    }

    #[test]
    fn mutation_a_receiver_is_threaded_where_rust_makes_it_the_only_way_in() {
        assert_eq!(Receiver::of("mut self"), Receiver::Owned);
        assert_eq!(Receiver::of("&mut self"), Receiver::Unique);
        assert_eq!(Receiver::of("&'a mut self"), Receiver::Unique);
        assert_eq!(Receiver::of("self: Box<Self>"), Receiver::Other);
        let types = with_square();
        let f: syn::ImplItemFn = syn::parse_str(
            "fn grow(&mut self, by: &mut u32, n: u32, s: &mut Vec<u8>) -> bool { true }",
        )
        .expect("parses");
        let sig = crate::convert::signature(&f.sig);
        let unique = Owner::Type {
            name: "Square",
            receiver: Some("&mut self"),
        };
        assert_eq!(threads(&sig, Some(&unique)), vec![0, 1, 3]);
        let (params, result) = types.signature(&sig, Some(&unique)).expect("lifts");
        assert_eq!(params[1].2, Ty::Nat(Some(32)));
        assert_eq!(
            result,
            Ty::Tuple(vec![
                Ty::Bool,
                Ty::Named("Square".to_string()),
                Ty::Nat(Some(32)),
                Ty::List(Box::new(Ty::Nat(Some(8)))),
            ])
        );
        // An owned receiver is a local of the method: nothing is given back.
        let owned = Owner::Type {
            name: "Square",
            receiver: Some("mut self"),
        };
        let f: syn::ImplItemFn =
            syn::parse_str("fn grown(mut self) -> Self { self }").expect("parses");
        let sig = crate::convert::signature(&f.sig);
        assert!(threads(&sig, Some(&owned)).is_empty());
        assert_eq!(
            types.signature(&sig, Some(&owned)).expect("lifts").1,
            Ty::Named("Square".to_string())
        );
        // A borrowed slice is changed in place, item by item: not threaded.
        let f: syn::ItemFn = syn::parse_str("fn zero(xs: &mut [u8]) {}").expect("parses");
        let sig = crate::convert::signature(&f.sig);
        assert!(threads(&sig, None).is_empty());
        assert_eq!(
            types.signature(&sig, None).err(),
            Some(Residue::Type("&mut [u8]".to_string()))
        );
    }

    #[test]
    fn mutation_a_mut_parameter_is_given_back_after_the_result() {
        let out = lift(
            "fn fill(out: &mut Vec<u32>, x: u32) { out.push(x); }",
            &plain(),
        )
        .expect("lifts");
        assert!(out.contains("result: TupleUnitListNat"), "{out}");
        assert!(
            out.contains("(record TupleUnitListNat {item0 := (), item1 := out1})"),
            "{out}"
        );
        let out = lift(
            "fn both(a: &mut u32, b: &mut u32, c: bool) -> bool { if c { *a = 1; return false; } \
             *b = 2; true }",
            &plain(),
        )
        .expect("lifts");
        // Each way out gives the values the places hold there.
        assert!(
            out.contains("item0 := threaded, item1 := a1, item2 := b"),
            "{out}"
        );
        assert!(
            out.contains("item0 := threaded, item1 := a, item2 := b1"),
            "{out}"
        );
    }

    #[test]
    fn mutation_a_borrow_the_lift_would_copy_is_refused() {
        let escapes = |state: &str| {
            Err(Residue::Form(format!(
                "{state} used as itself, a second borrow of the place it lends, which the lift \
                 would read as a copy"
            )))
        };
        for src in [
            "fn f(a: &mut u32) -> u32 { let b = a; *b }",
            "fn f(a: &mut Option<u32>) { match a { Some(v) => *v = 1, None => {} } }",
            "fn f(a: &mut u32) -> Option<u32> { let o = Some(a); None }",
            "fn f(a: &mut Vec<u32>) -> usize { let v = vec![a]; v.len() }",
        ] {
            assert_eq!(lift(src, &plain()), escapes("a"), "{src}");
        }
        // A `ref` binder borrows the place it matched.
        assert_eq!(
            lift(
                "fn f(o: Option<Vec<u32>>) -> Option<Vec<u32>> { let mut o = o; \
                 match o { Some(ref mut v) => v.push(1), None => {} } o }",
                &plain()
            ),
            Err(Residue::Form(
                "an update of v, which a ref pattern binds to the place it borrows".to_string()
            ))
        );
        // Propagation returns the current place on both exits.
        let out = lift(
            "fn f(a: &mut u32, s: Option<u32>) -> Option<u32> { let x = s?; *a = x; Some(x) }",
            &plain(),
        )
        .expect("threaded propagation");
        assert!(
            out.contains("item1 := a") && out.contains("Option.none"),
            "{out}"
        );
    }

    #[test]
    fn mutation_an_operator_update_of_a_field_is_the_fields_new_value() {
        let types = types_of("struct Log { text: String, time: u64 }");
        let out = lift(
            "fn note(log: &mut Log, piece: &str) { log.text += piece; }",
            &types,
        )
        .expect("lifts");
        assert!(
            out.contains(
                "text := (prim join (cons (project log text) (cons piece (nil String))) \"\" : String)"
            ) && out.contains("time := (project log time)"),
            "{out}"
        );
        // A field's arithmetic is the arithmetic, whatever holds it.
        assert_eq!(
            lift("fn tick(log: &mut Log, t: u64) { log.time += t; }", &types),
            Err(Residue::Arithmetic("+=".to_string()))
        );
    }

    fn mutation_cells() -> Types {
        types_of(
            "struct Shelf { books: RefCell<Vec<u32>>, loans: Cell<u32>, \
             latch: std::sync::Mutex<u32>, label: String } \
             struct Shared { inner: Rc<RefCell<u32>> }",
        )
    }

    #[test]
    fn mutation_a_cell_field_holds_its_content() {
        let types = mutation_cells();
        assert_eq!(
            fields(&types, "Shelf"),
            [
                field("books", Ty::List(Box::new(Ty::Nat(Some(32))))),
                field("loans", Ty::Nat(Some(32))),
                field("latch", Ty::Nat(Some(32))),
                field("label", Ty::Str),
            ]
        );
        // Another owner may change the cell: only the reference is held.
        assert_eq!(
            fields(&types, "Shared"),
            [field("inner", Ty::Named("Prelude::RcCellRef".into()))]
        );
        assert!(lift("fn read(m: &Shared) -> u32 { *m.inner.borrow() }", &types).is_err());
        assert!(lift(
            "fn write(m: &Shared) { *m.inner.borrow_mut() = 1; }",
            &types
        )
        .is_err());
        let out = lift(
            "fn size(m: &Shelf) -> usize { m.books.borrow().len() }",
            &types,
        )
        .expect("lifts");
        assert!(
            out.contains("(prim length (project m books) : Nat)"),
            "{out}"
        );
        assert!(lift("fn loans(m: &Shelf) -> u32 { m.loans.get() }", &types).is_ok());
        let out = lift(
            "fn make(label: String) -> Shelf { Shelf { books: RefCell::new(Vec::new()), \
             loans: Cell::new(3), latch: std::sync::Mutex::new(1), label } }",
            &types,
        )
        .expect("lifts");
        assert!(out.contains("loans := 3, latch := 1"), "{out}");
    }

    #[test]
    fn mutation_a_cell_is_changed_or_read_apart_only_with_a_row() {
        let types = mutation_cells();
        let changes = |method: &str, kind: &str, field: &str, read: &str| {
            Err(Residue::Form(format!(
                "{kind}::{method} on Shelf.{field}: the lift reads a {kind} only by {read}, and \
                 changes none, since another borrow of the value would see the change"
            )))
        };
        assert_eq!(
            lift("fn touch(m: &Shelf) { m.loans.set(1); }", &types),
            changes("set", "Cell", "loans", "get")
        );
        assert_eq!(
            lift(
                "fn push(m: &Shelf) { m.books.borrow_mut().push(1); }",
                &types
            ),
            changes("borrow_mut", "RefCell", "books", "borrow")
        );
        // `take` is the cell's own, whatever the content's methods are.
        assert_eq!(
            lift(
                "fn grab(m: &mut Shelf) -> Vec<u32> { m.books.take() }",
                &types
            ),
            changes("take", "RefCell", "books", "borrow")
        );
        assert_eq!(
            lift(
                "fn locked(m: &Shelf) -> u32 { *m.latch.lock().unwrap() }",
                &types
            ),
            Err(Residue::Form(
                "Mutex::lock on Shelf.latch: a Mutex is shared across threads, and another may \
                 change it between two reads"
                    .to_string()
            ))
        );
        assert_eq!(
            lift(
                "fn itself(m: &Shelf) -> usize { let r = &m.books; r.borrow().len() }",
                &types
            ),
            Err(Residue::Form(
                "the RefCell field Shelf.books as itself: the lift reads a RefCell only by \
                 borrow, and changes none, since another borrow of the value would see the change"
                    .to_string()
            ))
        );
        assert!(lift(
            "fn bound(m: &Shelf) -> usize { let Shelf { books, .. } = m; books.borrow().len() }",
            &types
        )
        .is_err());
        assert!(lift(
            "fn other(m: &Shelf) -> Shelf { Shelf { books: m.books.clone(), loans: Cell::new(0), \
             latch: std::sync::Mutex::new(0), label: String::new() } }",
            &types
        )
        .is_err());
    }
}
