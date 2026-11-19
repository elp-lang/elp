# Open questions — the single register

Every design doc keeps its own `Open` section, and that is still where the
*reasoning* lives. This file is the **index**: one line per live question,
naming the doc that owns it, so a question can be found without reading
seventeen files and so the same question stops being recorded in four.

It holds pointers, not content. If a line here and the owning doc disagree,
the owning doc wins and this line is the bug.

Written after a scan of `docs/` on 2026-11-19, which found eleven stale
entries, five questions recorded in more than one place with no owner, and
three load-bearing things with no design at all.

## The three with no design at all

These are not "open" in the ordinary sense. They are depended on by decided
work and have nothing written down.

| | depended on by | state |
|---|---|---|
| **`pure`** | `macros.md` (a macro *is* a `pure fn`), `memory-model.md` Finding 5, `aliasing.md`, `functions.md`, `collections.md`, `strings.md` — 11 docs | One line, in `grammar-script-notes.md`: *"`pure` functions containing side-effectful calls — enforced by the effect system, not the grammar."* **There is no effect system.** Four docs refer to it; none designs it. |
| **`Protected<T>`** | `memory-model.md`'s worked example, `functions.md`'s signatures, `generics.md`'s evidence table, the original corpus | `aliasing.md` says only what it is *not* ("not a peer of these three, it is a compiler-enforced access restriction"). What it restricts, and how, is nowhere. |
| **Loops** | every probe — `while` appears in `TextSearch`, `QuadTree`, `Worker` and more | No doc owns loop statements. `core-language-probes.md` §2 establishes that `while` is *needed*; `grammar-script-notes.md` gap #2 lists it as open; nothing specifies `while`, `loop`, `break` or `continue`. |

`pure` is the one to take first. It is the only one whose absence can make a
decided thing wrong rather than unwritten — `macros.md` rests the entire
compile-time evaluator on it.

## Owned, and live

**Expressions and syntax** — `operators.md` unless noted

| question | owner |
|---|---|
| Numeric tower: widths, literal defaulting, overflow | `operators.md` (also wanted by `strings.md`, `memory-model.md`) |
| Optional chaining | `operators.md` — the last residue of the "core expression language" gap |
| `a[i] = v` | `operators.md`, narrowed by `update(i, fn)` |
| May an expression span a line break? | `grammar-script-notes.md` — `QuadTree/types/geometry.elp` depends on the answer |
| Are `from` and `as` reserved or contextual? | `grammar-script-notes.md` |
| Float vs range lexing — `.5` is currently illegal by accident | `grammar-script-notes.md` |
| Destructured params do not parse | `grammar-script-notes.md` |
| Is `style` a reserved prop name? | `grammar-script-notes.md` |
| Nested record patterns | `patterns.md` |
| `?` | `error-handling.md` (wanted by `memory-model.md`'s scoped release) |
| The type switch | `error-handling.md` — recorded in `dispatch.md` and `patterns.md` too, which both defer here |

**Types and conformance**

| question | owner |
|---|---|
| Conditional conformance can lie about `Sendable` | `interfaces.md` #5 — **overdue**; it was flagged as "settle before any concurrency work lands" and that work has landed |
| `Optional<&T>` — an optional borrow | `memory-model.md` — four sightings (`QuadTree` GAP 8, `Interpreter`, `VulkanTriangle`, a borrowing `Map.get`) |
| Can a type be `Copy` and `Duplicate` both? | `aliasing.md` |
| How deep is `.copy()`? | `aliasing.md` — the recursive case is answered, the rest is not |
| Structural `object{ }` types | `generics.md` |
| Generic components | `generics.md` |
| Macros taking type arguments | `generics.md` / `macros.md` |
| Associated type bounds | `dispatch.md` |
| How a component conforms to a props interface | `interfaces.md` |

**Cells, concurrency, FFI**

| question | owner |
|---|---|
| What `.get()` returns, and how long it lives | `aliasing.md` |
| A consuming `.take()` on a cell | `aliasing.md` — `Playhead` GAP 6 |
| A borrowing `Map.get` | `collections.md` — blocked on `Optional<&T>` |
| A once-only closure (`Move` capture) | `functions.md` — `tree-places.elp` GAP 7 |
| What is forbidden inside a `Synchronized` transform, and who checks it | `aliasing.md` — a restriction by position, and `pure` is no longer the candidate |
| Exactly-once release, and a thread discipline known only at runtime | `aliasing.md` |
| Does `Confined<T>` unify with a reactive cell? | `aliasing.md` |
| `symbol =` on `@external` | `memory-model.md` |
| Whether `release` may fail; whether a `with` block may yield; early `return` out of one | `memory-model.md` |
| `Buffer<T>`: reinterpretation, growth, alignment, mapped storage, byte access | `memory-model.md` |
| Channel shutdown | `core-language-probes.md` §7b |
| `background` has no join | `core-language-probes.md` |
| Cancellation during mutation, with no cell thread | `core-language-probes.md` |
| A platform API as a third category beside `links` and `dependencies` | `name-resolution.md` — `Playhead` GAP 5 |

**Reactivity** — `reactivity.md`

| question |
|---|
| `frame` is used in the design and appears nowhere in this doc |
| A cell write is a notification point, and nothing says so (`Editor` FINDING 4) |
| A derived value recomputing equal should notify nobody |
| The cell thread has a deadline too |
| Store writes are coarse — a constraint on `copyWith` that is not stated as one |
| Prop drilling, and whether `context` generalises the two ad-hoc injections |

**Tooling**

| question | owner |
|---|---|
| Lint default levels, lint groups, `deny` in a dependency, `@allow` spelling | `diagnostics.md` |
| The macro reflection API's shape; fixed-point termination; macro diagnostics | `macros.md` |
| `RecoveryBoundary`: default fallback, retry shape, diagnostics | `error-handling.md` |
| Routing resolution | `name-resolution.md` |
| Stdlib module layout — four roots for one set of components | `name-resolution.md` |
| Whole-program versus separate compilation | `dispatch.md`, deliberately deferred to the backend |

## Migration backlog created by the write-surface round

Not design questions. Work the ratifications of 2026-11-19 created and that
the corpus scan will otherwise rediscover as findings.

1. **Every `@external` out-parameter wants `&var`.** Finding 4's third
   amendment replaced the conservative *"`&T` is read-write at an `@external`
   declaration"* blanket with C's own `const`. `SqliteBinding` and
   `VulkanTriangle` have roughly twenty parameters between them that C writes
   through and that still read `&`.
2. **And that exposes a question the rule did not answer.** For
   `&Buffer<T>`, C writes the *elements*, not the buffer value. So does
   `&var Buffer<T>` mean "C may replace the buffer" or "C may fill it"? The
   two are different permissions and `vkEnumeratePhysicalDevices` wants the
   second. This is new, and it belongs with `memory-model.md`'s boundary
   forms.
3. **`nil` versus `null` in the corpus** — 117 uses across 23 files against
   41 across 5, the `null` files being the FFI-heavy ones. Possibly a
   deliberate distinction between an empty `Optional` and C's null pointer,
   possibly drift. Nothing records an intent. `core-language-probes.md` §12.
4. `name-resolution.md`'s own corpus migration backlog, four items, none
   applied.

## What the scan corrected

Recorded so the same drift is recognisable next time. All eleven were one of
two shapes: **a doc stating as open something another doc had answered**, or
**a doc resting an argument on a premise that a later decision removed**.

- `functions.md` listed function values as undesigned; its own body and
  `core-language-probes.md` §7 both resolve them.
- `core-language-probes.md` §7b asked whether a block match arm produces a
  value; `patterns.md` answers yes, with an example.
- `name-resolution.md` pointed at `docs/core-language.md`, which was never
  written, and called value-level `+` unratified, which `operators.md` and
  `generics.md` both settle.
- `name-resolution.md` asked whether a dialect constrains a module's
  contents; `grammar-script-notes.md` answers half of it for `.selp`.
- `aliasing.md` justified rejecting lock guards partly on there being "no
  `&mut` spelling", which is no longer true — and the rejection does not
  depend on it.
- `aliasing.md`'s "the loosening is already indicated" and
  `core-language-probes.md` §9 and §12 argued against a narrower `update`
  signature than the one ratified.
- `functions.md` had a live code example using the superseded
  `update(pure fn (T) -> T)`.
- `core-language-probes.md` claimed Finding 4 forbids the `&var T` parameter
  spelling, and that `@external` resolves `&T` to read-write.
- `memory-model.md` Finding 5 derived a `pure fn` rule from Finding 4's
  inference, which is now a rule in its own right.
- `reactivity.md`'s "inferred for ordinary borrows, explicit in prop
  position" contrast no longer holds for mutation — that section made the
  argument that later won everywhere.
- `core-language-probes.md`'s header table listed ten of sixteen probes, and
  described a corpus parse test that was removed with the front end.
