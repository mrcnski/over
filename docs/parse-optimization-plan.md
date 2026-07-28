# Plan: remaining parser optimizations

Status: planned, not implemented. Companion to
[int-fast-path-plan.md](int-fast-path-plan.md).

## Where the time goes now

After the batch byte-scanning work, scanning is no longer the bottleneck.
Benchmarks (criterion, Apple Silicon): int-heavy ~179 ns per field,
string-heavy ~196 ns per field, nested ~1.8 µs per object.

Remaining cost per `field: 12345` pair is dominated by ~5 heap allocations:

1. field-name `String` (built in `parse_field`)
2. its clone into the duplicate-detection `HashSet`
3. the digit-buffer `String` in `parse_numeric`
4. the `BigInt`
5. amortized `Vec` growth (`obj_pairs`)

Nested documents add one `Arc` per Obj/Arr/Tup plus `BigRational`
construction (two BigInts, `pow(10, n)`, heap gcd) per decimal.

## Why not an arena

A bump arena fits parsers that return a *borrowed* AST and free it all at
once. This crate's public API returns owned, independently-lived,
`Arc`-shared values; backing them with an arena either leaks (arena outlives
everything) or infects the public API with `Obj<'arena>` lifetimes. Rejected.
The arena's *goal* — stop paying malloc per token — is captured instead by
the avenues below (scratch reuse, slice-based tokens).

## Avenues, in recommended order

### 1. Cursor restructure (internal, no API change) — do first

`CharStream` is `Rc<RefCell<Inner>>` only so parser helpers can share the
cursor through cloned handles. Replace it with a plain cursor struct
(`&str` source + pos/line/col) passed as `&mut` through the parser.

- Removes the residual `RefCell` borrow on every remaining `peek`/`next`.
- Unlocks slice-returning scanners: a field name becomes one `String`
  allocation directly from its source slice (no intermediate buffer);
  numbers and keywords need no allocation at all.
- `parse` is a private module, so this is invisible to users.

Avenues 2 and 3 land most cleanly on top of this.

### 2. Direct numeric accumulation (internal)

`parse_numeric` builds a digit `String`, then `i64::from_str` re-scans it.
Instead accumulate `value = value * 10 + digit` during the digit scan with
checked overflow, falling back to the BigInt path on overflow or >18 digits.
Underscores fall out naturally (they separate digit runs). Deletes one
allocation and one re-scan per number. Estimated 15–25% on int-heavy input.

### 3. Clone-free duplicate detection (internal)

The dedup `HashSet<String>` clones every field name. Replace with a map from
field-name *hash* to index into `obj_pairs`; on hash hit, confirm against the
stored `String` at that index. Exact semantics, zero clones, graceful on
collision. Kills one allocation per field.

### 4. memchr for string bodies (adds a dependency — decide)

`take_str_span` hand-rolls a ~1 ns/byte loop. The `memchr` crate's SIMD
`memchr2(b'"', b'\\')` is roughly an order of magnitude faster on long
strings. Only matters for string-heavy documents. This would be the crate's
first perf-motivated dependency; skip if minimal deps matter more.

### 5. Int fast path (breaking, 0.7)

See [int-fast-path-plan.md](int-fast-path-plan.md). Removes the last
allocation per small int and fixes the allocating `get_int` read path.

### 6. Frac small path (breaking, 0.7)

Extension of the Int plan: an `i64/i64` rational with promote-on-overflow.
Today every decimal costs two BigInt allocations, a `pow(10, n)`, and a heap
gcd; the small path makes typical decimals allocation-free with a
register-width gcd. Likely the largest remaining win for real config files
with decimals, and most of the nested benchmark's floor.

## Explored and rejected

- **Arenas** — see above.
- **Field-name interning** — would help schema-repetitive documents (many
  objects sharing field names), but `Pair` is `pub struct Pair(pub String,
  pub Value)`, so interning is an API break that only pays off for one
  document shape. The cursor restructure already reduces field names to one
  tight allocation.
- **Parallelism / mmap** — config documents are too small to amortize either.

## Irreducible floor

One `Arc` per container (identity semantics are part of the data model), one
allocation per stored `String`, one per `Vec`. Going below that means
redesigning the value model, which is out of scope.

## Expected outcome and discipline

Avenues 1–3 (no API breaks, no new deps) are estimated to bring int-heavy
parsing from ~179 ns/field to ~90–110 ns/field. Avenues 5–6 push small-int
and decimal parsing to near-allocation-free, batched as a deliberate 0.7
release.

All numbers above are estimates. Benchmark each avenue in isolation with the
existing `benches/parse.rs` suite before and after, and validate behavior
with the differential-test approach (old binary vs new over the error corpus
and adversarial inputs) used for the batch-scanning change.

## Other 0.7 considerations

`ReferenceType::id()` has no consumers: `ptr_eq` is `Arc::ptr_eq`,
`num_references` is `Arc::strong_count`, `PartialEq` is structural, and the
writer does no sharing/cycle detection. The only call ever written is a
commented-out round-trip assertion in `tests/integration.rs` (aliasing is
not preserved across write → re-parse, so it could never pass). Its only
effect today is a global `AtomicUsize` increment per container (`gen_id` in
`src/lib.rs`) plus a stored `usize`, producing ids that differ between two
parses of identical input.

Since 0.7 already breaks the API, either:

- **drop `id()`** from `ReferenceType` and delete `gen_id`, or
- **give it a real job**: make the writer detect shared substructure (visited
  set keyed by id — ids are ABA-safe where `Arc::as_ptr` keys are not) and
  emit variables to preserve aliasing across write → re-parse, which would
  also let that commented-out assertion finally pass.

The first is a simplification; the second fixes a documented lossiness in
`write_to_file`. Decide when scoping 0.7.
