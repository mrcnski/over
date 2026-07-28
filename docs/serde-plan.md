# Plan: serde support

Status: planned, not implemented. Additive — feature-gated, no API break,
can ship in a 0.6.x release. Does not depend on the 0.7 plans
([int-fast-path-plan.md](int-fast-path-plan.md),
[parse-optimization-plan.md](parse-optimization-plan.md)), though the Int
fast path simplifies the integer mapping if it lands first.

## Motivation

Most Rust users consume config by deserializing directly into their own
types (`#[derive(Deserialize)] struct Config`), not by walking a dynamic
value tree. Lacking serde support is the crate's biggest ecosystem-fit gap —
bigger than any remaining performance item.

## Approach

Tree-backed, the same architecture as `serde_json`'s `Value` path:

- **Deserialize**: `over::from_str::<T>` / `over::from_file::<T>` parse to
  `Obj` with the existing parser, then a `Deserializer` implementation walks
  the `Value` tree feeding serde visitors. Includes, variables, and parents
  are already resolved by the parser, so the deserializer sees a plain tree.
  Because the tree is materialized, `deserialize_any` is cheap, which makes
  untagged enums and `#[serde(flatten)]` work.
- **Serialize**: `over::to_string(&T)` builds a `Value` tree via a
  `Serializer`, then emits text through the existing writer.
- Gate everything behind a `serde` cargo feature. Also implement
  `Serialize`/`Deserialize` for `Value`/`Obj` themselves (near-free once the
  mapping exists) so parsed OVER data can be embedded in other formats,
  e.g. two-line OVER-to-JSON conversion.
- A streaming (no-tree) deserializer is possible but not worth it for
  config-sized documents; variables would force memoization of referenced
  fields anyway. Rejected.

## Type mapping

| OVER          | serde                                            |
|---------------|--------------------------------------------------|
| `Null`        | unit / `None` (fields absent also map to `None`) |
| `Bool`        | `bool`                                           |
| `Int`         | `i64`/`u64`/`i128` as requested; error if the `BigInt` exceeds the target (see decision 2) |
| `Frac`        | `f64` (see decision 1)                           |
| `Str`         | `String` / `char`                                |
| `Arr`, `Tup`  | sequences / tuples                               |
| `Obj`         | structs / maps                                   |
| enums         | config-format convention: bare string for unit variants, single-key obj otherwise |

`bytes` has no OVER representation; serialize as `Arr` of `Int`s or error —
follow what other config formats do (TOML errors).

## Design decisions

1. **`Frac` <-> float policy.** Deserializing `Frac` into `f64` uses
   `BigRational::to_f64` (nearest). Serializing `f64` must NOT use
   `BigRational::from_f64` (exact binary expansion turns `0.1` into
   `3602879701896397/36028797018963968` and the writer prints its full
   decimal expansion). Instead: shortest-round-trip decimal formatting
   (ryu, or `format!("{}")` which uses the same algorithm since Rust 1.55),
   parsed back as a decimal `Frac`. Exactness-sensitive users can
   deserialize into `BigRational` via a wrapper type later; not in scope.
2. **Oversized ints.** `BigInt` values beyond the requested integer type
   fail with a clear error. No string fallback in v1.
3. **Parent fields.** Deserialization presents the *resolved* view — fields
   inherited through `^` are visible, matching `Obj::get` semantics. The
   parent chain itself is not exposed to serde.
4. **Map keys.** OVER field names must be valid identifiers (no leading
   digit, no spaces, no quoting syntax exists). Serializing a map with a
   non-identifier key is an error, stated plainly in the error message.
   Structs are unaffected (Rust identifiers are valid OVER fields; note
   `r#type` etc. serialize as their unprefixed name and `type` is not
   reserved in OVER).

## Known lossiness (accepted)

Serde's data model is a tree; OVER's is a DAG. Serialization from user types
can never produce aliasing (`ship_to: bill_to`), includes, or variables —
these are authoring conveniences of the text format and do not round-trip
through user structs in any format (same as YAML anchors under serde_yaml).
This is independent of the writer's own aliasing lossiness (see "Other 0.7
considerations" in the Int plan).

## Effort

Roughly 1-2k lines including tests; the deserializer and serializer are
mechanical once the four decisions above are fixed. Test with round-trip
properties (T -> OVER -> T for derived types), the existing test corpus
deserialized into matching structs, and error-path cases for decisions 2
and 4.
