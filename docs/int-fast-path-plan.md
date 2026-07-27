# Plan: fixed-size `Int` fast path for `Value`

Status: planned, not implemented. Breaking change — target a 0.7.0 release.

## Motivation

Every integer in a document is currently a `BigInt`, which costs a heap
allocation per value. Measured costs (Apple Silicon, release build):

| operation              | cost      |
|------------------------|-----------|
| `BigInt::from(i64)`    | ~14 ns (1 heap alloc) |
| `BigInt::clone()`      | ~12 ns (1 heap alloc) |
| `BigInt + BigInt`      | ~14 ns    |
| `i64` copy / eq / add  | 0.3–0.6 ns |

Impact by area:

- **Parsing**: only ~6% of int-heavy parse time (228 ns per int field total),
  so this is *not* primarily a parse-speed optimization.
- **Read path**: `value.get_int()` clones the `BigInt` — an allocation on
  every access. This is the biggest practical win for applications that read
  config values repeatedly.
- **Memory**: every int is a 32-byte heap allocation plus a pointer chase.
  The fast path stores small ints inline with zero heap.
- **In-document arithmetic** (`x: 2 + 3 * 4`): each operation allocates a
  result `BigInt`; `i64` checked ops are ~50x cheaper.

## Design

A normalized small/big integer type replaces `BigInt` in the `Value` enum:

```rust
/// An integer value: inline i64 when it fits, heap BigInt otherwise.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Int {
    Small(i64),
    Big(BigInt), // invariant: only when the value exceeds i64 range
}

pub enum Value {
    // ...
    Int(Int), // was Int(BigInt)
    // ...
}
```

Key points:

- **Invariant**: `Big` is only used when the value does not fit in `i64`.
  Enforced by all constructors (`From<BigInt>` normalizes via `to_i64()`).
  This makes `PartialEq` trivially correct — cross-variant values are never
  equal, so derived equality works.
- **Arithmetic**: implement `Add`/`Sub`/`Mul`/`Rem`/`Neg` for `Int` using
  `i64` checked ops (`checked_add` etc.), promoting to `Big` on overflow.
  The parser's `binary_op_on_values` and `unary_op_on_value` switch to these.
- **Parsing**: `digits_to_bigint` in `src/parse/parser.rs` becomes
  `digits_to_int` returning `Int` directly — small ints never touch `BigInt`.
- **Compatibility**:
  - Keep `get_int() -> OverResult<BigInt>` (converts, allocates) so most
    callers keep working.
  - Add `get_i64() -> OverResult<i64>` as the new fast read path.
  - Re-point the `impl_eq_int!` macros in `src/value.rs` at `Int` accessors
    mirroring `ToPrimitive` (`to_i64`, `to_usize`, ...).
- **`Value` size**: unchanged — `Frac(BigRational)` (64 bytes) still
  dominates the enum layout.

## Files to touch

- `src/value.rs` — `Int` type, enum change, `From`/`PartialEq` impls.
- `src/parse/parser.rs` — `digits_to_int`, int arms of binary/unary ops.
- `src/parse/format.rs` — `Display`/format for `Int`.
- `src/macros.rs` — `int!` macro.
- `src/tests.rs`, `tests/` — update pattern matches on `Value::Int`.

Estimated ~200 lines changed.

## Rejected alternatives

- **Two `Value` variants** (`Int(i64)` + `BigInt(BigInt)`): representation
  leaks into the type system; every match site must handle both variants and
  equality across variants becomes error-prone.
- **Swap num-bigint for a crate with inline small-int storage**
  (ibig / dashu / malachite): still breaks the public API (`get_int` type),
  and the alternatives are heavier dependencies or worse-licensed
  (malachite is LGPL).

## Extension

The same trick applies to `Frac`: an `i64/i64` rational with
promote-on-overflow. Benchmarks suggest a bigger relative win there, since
`BigRational` operations involve gcd reductions on heap values.
