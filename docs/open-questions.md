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

## What `examples/Json` opened

The last core-language probe, and the biggest thing it found is in the one
mechanism nothing had exercised.

| | |
|---|---|
| **An AST has no literal syntax** | `macros.md` claims the AST is first-class data. True of what a macro *reads*, false of what it *writes*. A `quote` form is now **designed** there — parsed as elp, holes declared as parameters so they highlight as identifiers, a hole's type deciding how it splices, and no repetition syntax at all — and tested on `examples/Json/codec.elp`. Not ratified. |
| **Hygiene for free identifiers in a quote** | `macros.md` — the half the proposal does *not* settle. A free name must resolve in the macro's definition scope, which means a quote carries its defining module: a representation decision on `Declaration`. **The piece to design next.** |
| A macro has no diagnostic channel | `macros.md` — a `pure fn` cannot say "I do not know how to do that, here" |
| What names a serialised field | `macros.md` — the dropped Go-style tags were the macro's *input*, not a duplicate of it |
| Conformance as compile-time data | `macros.md` — decode needs "does this type conform to `Codec<JSON>`?" |
| Building a `string` has no spelling | `strings.md` / `collections.md` — stdlib surface: `[char].joined()` and a `Buffer`-backed writer |
| `Index<K, V>` silently requires `V: Copy` | `operators.md` |
| A type iterable two ways cannot conform to `Iterable<T>` | `collections.md` — the rule is one obvious traversal |
| Range patterns over `char` | `patterns.md` — and the pattern form may legitimately differ from the expression form |
| An overflowing numeric literal arriving from **input** | `operators.md` — the numeric tower's first runtime face |
| Cross-numeric `==` | `operators.md` — `1` and `1.0`, with implicit conversion forbidden |

And one correction it forced: **`aliasing.md` twice argued from "elp has no
`defer`, nothing at scope exit"**, and the language has three scope-exit
constructs. Fixed there; the exactly-once-release item survives with a
different shape — not "elp needs a mechanism" but "elp has three and none is
mandatory".

## What a second outside review found, reading the corpus

The first review read five documents. This one read the programs, which is
the right way round — and it found two things the probes and the
ratifications had both missed, because no corpus program does either.

| | |
|---|---|
| ~~**A failing transform leaves a partial write**~~ | **Resolved** by `commit` (`aliasing.md`, `error-handling.md`): the strong exception-safety guarantee by copy-and-swap, for `T: Duplicate`. `update` gives the basic guarantee and says so; a fallible transform must say which it wants. |
| **Two related elements cannot be mutated in one operation** | `collections.md` — `xs.update(i, …)` holds `&var xs[i]`, and reading `xs[j]` inside it needs `&xs`, which overlaps and is refused. So a swap, a sort step, or any pairwise operation has no spelling. Nothing in the corpus does one. |

Three of its other stress tests turn out to be **already answered**, which is
worth recording so they are not re-opened: mutating an element while
iterating the collection falls out of the exclusivity rule and
`for i in xs.indices()` is the legal form (`control-flow.md`); nested updates
on one cell trap; and returning a *derived value* from an update is what
`-> R` is for, while returning a *reference* is refused because the `&var T`
may not escape.

### And one synthesis, half of which was wrong

**Execution context is not a body property**, and the half that holds is
still worth having.

The review's sharpest point is that `Sendable` says whether a *value* may
cross a thread and nothing says whether a *function* must run on a
particular one — GLFW's event loop being the example, since it must be
called from the thread that created the window. That is real, it is tracked
in two places and connected in neither (`core-language-probes.md` §10
finding 7, and `functions.md`'s open item on a callback's affinity), and
nothing answers it.

It was filed here as a fourth body property, on the grounds that it is a
declared bit propagated call-wise. **It is not**, and `functions.md`'s
admission test is what caught it: *"this function must run on the main
thread"* is not a restriction on a body, it is an **obligation on callers**,
and it propagates the other way — every caller must itself be on that
thread. §10 finding 7 had it right the first time, in the words that were
quoted and then ignored: *"main-thread affinity is a property of a call, not
of a value."* A property of a call is not a property of a body.

So affinity needs its own mechanism and remains open. What the episode
settled is the thing it was raised to settle: the body-property set is
three, and it stops there because a candidate has to pass four questions —
not because nobody has suggested a fourth.

## What the composability probes found

`examples/Dashboard` and `examples/Rejected` were written to the rubric in
`core-language-probes.md` §18, and between them they moved three things out
of "nobody has checked" and into this file.

| | |
|---|---|
| ~~**Cancellation can unwind out of a transform**~~ | **Resolved** by the same mechanism, which is why it was worth taking: under `commit` the unwind leaves the cell holding the original, so the case with no failure path on the page needs no rule of its own. For a non-`Duplicate` payload the basic guarantee is still all there is. |
| A `Published<T>` cannot notify a component | `reactivity.md` — no mechanism joins them, and the only thing that would is `frame`, which that file does not mention. Both halves read as complete alone. |
| A lock taken on the cell thread | `reactivity.md` — legal, correct by every rule, and a UI thread blocking on a worker's lock. Wants a diagnostic, not a rule. |

And five rules that **name a mechanism nobody has built**, which is a
different category from an open question and is collected here so it stops
reading as one: the duration of an iterator's borrow (`control-flow.md`
depends on it), the allocates-or-not property (`functions.md` costed it,
nothing has it, so `realtime` cannot currently fire), the by-position
restriction on a `Synchronized` transform body, what rejects `a[i] = v`, and
the diagnostic for a conformance the fields deny.

Of sixteen ways to write something wrong, **nine are not currently caught**.

## What the error-handling round decided, and what it opened

Run on 2026-11-20 after a survey counted the corpus's error handling rather
than reading it: **51 of 59 fallible signatures returned the bare
existential `Either<_, Error>`**, and every `Err(...)` handler in 11,300
lines did one of exactly two things — `e.toString()` or
`Err(Wrapper.from(e))`. Nothing branched on which error it got, which is
why two things had sat open with no customer (`?`, the type switch) and one
defect had sat unresolved (the partial write).

Decided, all in `error-handling.md`: one concrete error type per layer with
the existential only where nobody branches; `?` with three rules;
`@message` on an `@error` variant; the three exception-safety guarantees
and `commit`; typed aggregation instead of a joined error; handle-once as a
convention; and a library may not trap on input. `examples/Recovery` is the
probe.

What it opened:

| | |
|---|---|
| **May `release` take `&self`?** | `memory-model.md` — **the decision that decides whether `with` survives.** Eleven rows separate `with` from a `defer` and exactly one is a capability: `with` can consume the resource without liveness analysis. If `release(&self)` is acceptable, `defer x.release()` covers the common case with no new surface and the keyword is deletable. Take this before the three additions below. |
| **`with` was never ratified** | `memory-model.md` — introduced on 9 November in the same commit that deleted a keyword I had invented, justified by three probes asking for the *problem*, and treated as decided ever since because it was written up as a designed section. Flagged by the user asking where it came from. Five `Scoped` conformances, two `with` blocks. |
| **A resource owned by a platform runtime** | `memory-model.md` — the third case `with` cannot express, and the one with no candidate: `examples/Playhead`'s `AudioDriver` lives for the app's lifetime, and an app has no `fn main` to put a brace in. A consuming binding does not help, because there is still no block. |
| **A consuming `with` binding** | `memory-model.md` — the cleanup problem `?` creates, with the keyword withdrawn. `with` was ratified as the answer to *"a leak per early exit"* and covers it only where the resource dies in the block; a constructor assembles handles **for a caller**, and scoped release's own safety rule (the binding is a borrow) is what stops them leaving on success. Two customers: `VulkanTriangle` GAP 4 and `Recovery`'s outbox claim. |
| **Cleanup for a resource acquired in a loop** | `memory-model.md` — n handles, failure at item k, n−1 releases. No candidate handles it: a `with` cannot bind a variable number of resources and a per-acquisition cleanup is scoped to the loop body. Nothing in the corpus does it yet, and it is the case that would decide against all of them. |
| **A compensation that must not fire on every failure** | `error-handling.md` — in `Recovery/sync.elp` three failure paths want the claimed item restored and three want it dropped, because a rejected item put back is retried forever. Survived the keyword's withdrawal; only the `defer (exit)` candidate answers it for free. |
| **The bottom type, with four files depending on it** | `control-flow.md` — recorded there as *"a diverging `while true`"* and now arriving from a third direction. `return` in a `match` arm whose siblings produce values, `?? return Err(…)` in `Json` and `Untrusted`, and every `break` in expression position are all diverging expressions, and elp has no type for one. The 36 `throw` sites were a fourth spelling of it, and are migrated rather than admitted. |
| **The nothrow tier has no spelling** | `error-handling.md` — and it is the one the other two rest on. C++ marks it `noexcept` because the guarantee is only useful if the compiler can be told. `Outbox.restore` must never fail and only a comment says so. |
| **No narrower error type than the enum** | `error-handling.md` — the set of failures *one request* can produce is a subset of what the *policy* can produce, and a `match` must still write the unreachable arm. Zig infers an error set from the body; Swift 6 makes the author name it; elp has no spelling for "these three of those six", and the workaround is two enums and twice the conversions. |
| **Cancellation cannot be reported** | `error-handling.md` / `core-language-probes.md` — `SyncError.CANCELLED` is declared and unconstructable, because cancellation is an unwind rather than a value. A caller cannot tell a cancelled batch from a crashed one. Same gap as *"`background` has no join"*, seen from the error side. |
| **What `@message` may interpolate** | `strings.md` — now has a second customer and a sharper case: a `Protected<string>` field must not be interpolable, and whether it is refused depends on whether interpolation is specified in terms of a conformance. |
| **`existential_error`'s level** | `diagnostics.md` — warn by default, with `@allow` expected on `main`. Arrives with the standing open item about default levels. |

### What the migration found, which is the part that could not be predicted

Nineteen files changed. Four of the findings are about the design and not
about the files:

- **The existential hid a conversion that was already happening.**
  `Json/internal/parser.elp` returns `Ok(.SOME(r?))` where `r` is the
  *lexer's* `Either<Token, LexError>`, so a `LexError` was propagating out
  of a function whose other arms return `ParseError` — and both signatures
  said `Either<_, Error>`, so nothing noticed. Making the types concrete
  forced `ParseError implements From<LexError>` to exist. The conversion
  was always there; it had nowhere to be written down.
- **The existential was justified by a second error type that does not
  exist.** `BlogStyle/internal/api.elp` carried the clearest statement of
  how the corpus got to 51-to-8: *"more than one error type can reach here;
  the caller matches on the interface and narrows if it cares."* No second
  error type reaches it. All four functions have one `Err` arm, and the
  narrowing it promised was the type switch.
- **A fallible signature was standing in for a function nobody had
  written.** `Pipeline`'s `Config.load` and `Dashboard`'s `render` both
  declare `Either<_, Error>` over a body that cannot fail. Naming the real
  failures is the fix; the alternative is the nothrow tier, which has no
  spelling.
- **One error variant was reported for four operations.**
  `SqliteBinding`'s `StatementError.CANNOT_PREPARE` was returned by
  `prepare`, `bind`, `step` and `finalize`, because it was the only variant
  carrying a sqlite code. A caller was told "cannot prepare" when a bind
  failed — a *wrong* answer, which is worse than the existential's no
  answer.

And one piece of stdlib surface the migration needs: **`.orError(e)`** on
`Optional<T>`, which is what the twelve `?? throw E` sites wanted.
`stdlib-boundary.md` bucket 2. Owner: `collections.md`.

And one it sharpened rather than opened: **a field default's timing**.
`OutboxItem.idempotencyKey` is the first corpus field where "once per type"
and "once per construction" differ in *behaviour* rather than in style — a
default that reran would turn at-least-once delivery into duplicate
delivery.

## Where the decided set is in tension with itself

Not open questions — `tradeoffs.md` holds a cross-cutting audit of the
decisions that have already been made: five problems with several answers,
twelve decisions whose cost landed on a different program, and the check
that would have caught every defect. Two entries there are one decision
away from each other, since the pairwise operation and the borrowing read
both dissolve if `Optional<&V>` is answered.

## One question that is not about a feature

Every other entry in this file asks whether one thing is decided. This one
asks whether the decided things **compose**, and nothing in the design record
answers it.

**Do the concurrency primitives form one model?** There are now eight —
`Confined<T>`, `Synchronized<T>`, `Atomic<T>`, `Ring<T>`, `Published<T>`, a
component `var` (which is a cell), `background`, and `Channel<T>` — plus
`effect`, the cell thread, `defer`, and `@abi(.C, realtime)` as a body
property over the lot. Each arrived from a probe that needed it and each is
defensible on its own. **No document reasons about more than two at a time.**

Raised by an outside review of the repository, and it is the one thing in
that review the record did not already track. It is also the same
methodological failure as the three defects `core-language-probes.md` §16
found, one level up: those were rules checked against the case that
motivated them and not against the set they governed, and this is a set of
features each checked against its own probe and never against each other.

Concrete pairs nobody has written down:

- A `Published<T>` read inside an `effect` — does it subscribe, or is it a
  sample? `reactivity.md` says a read subscribes and `aliasing.md` designs
  `Published<T>` for a reader with a deadline, which are different answers.
- `defer` inside a `Synchronized.update` transform. `defer` runs on
  cancellation unwind; the transform is a critical section. Nothing says
  whether a cancellation can land inside one.
- A `Ring<T>` endpoint captured by a `background` body, when `Move` is what
  enforces SPSC and a closure capture is governed by `Copy`/`Move`.
- `Weak<T>` upgraded on the cell thread, which is forbidden in a realtime
  body and unexamined everywhere else.
- A component `var` holding a `Synchronized<T>`: two cells, one on the cell
  thread and one not, and the reactive system has no idea.

The reactivity round is where this lands, because reactivity is the thing
that touches all of them. Worth doing as a **composition audit** rather than
as a list of fixes: the output is a table of which primitives may appear
inside which, not a set of new features.

## The three with no design at all — all three now designed

These were not "open" in the ordinary sense. They were depended on by
decided work and had nothing written down. All three are closed; the rows
are kept because the shape of the gap is worth recognising next time.

Two of the three were assumed to need more machinery than they did. `pure`
was assumed to need an effect system and needs a checked bit;
`Protected<T>` was assumed to be compiler-enforced and is a short
conformance list. Loops were the opposite — mostly already decided, in
three other files, and never gathered.

| | depended on by | state |
|---|---|---|
| ~~**`pure`**~~ | — | **Designed**, in `functions.md`, "`pure`, and the two body properties". A declared bit, checked against a closed set of operations, propagated call-wise — no effect system, by the same pattern that makes `Copy` field-wise. What it left behind is smaller and listed below. |
| ~~**`Protected<T>`**~~ | — | **Designed**, in `aliasing.md`, and it needs no privilege: every leak is closed by an absent conformance, so it is plain elp. `Duplicate`, never `Copy`; `.expose()` is the one way out and the one thing to grep for. |
| ~~**Loops**~~ | — | **Designed**, and they have an owner at last: `control-flow.md`. `while`, `break` and `continue` exist; `loop` and labels do not. `a..b` includes both ends, which was never decided either. |

`Protected<T>` is the one left. `pure` was taken first, for the reason given
here — it was the only one whose absence could make a *decided* thing wrong
rather than unwritten. It did:
`functions.md`'s claim that real-time safety is purity plus a ban on
allocation turned out to be backwards, since a real-time body calls C and
reads cells and is therefore maximally impure. Loops followed, and turned
out to be mostly *collection* rather than design: ranges, the iteration
protocol and the binding pattern were already decided in three other files
and had never been gathered, so what `control-flow.md` actually adds is
`while`, `break`, `continue`, four refusals with their costs, and one
genuinely undecided thing — that `a..b` includes both ends.

## Owned, and live

**Expressions and syntax** — `operators.md` unless noted

| question | owner |
|---|---|
| Numeric tower: widths, literal defaulting, overflow | `operators.md` (also wanted by `strings.md`, `memory-model.md`) |
| Optional chaining | `operators.md` — the last residue of the "core expression language" gap |
| `a[i] = v` | `operators.md`, narrowed by `update(i, fn)` |
| May an expression span a line break? | `grammar-script-notes.md` — `QuadTree/types/geometry.elp` depends on the answer |
| An exclusive range form, if `0..n - 1` reads badly after migration | `control-flow.md` |
| A labelled `break`; a diverging `while true` | `control-flow.md` — both additive, neither wanted by a probe |
| `indices()`, `reversed()` and `enumerated()` | `collections.md` — stdlib surface; `enumerated()` yields a record, not a pair |
| ~~A compound reconciliation key allocates~~ | **solved** — `by` takes a comma-separated list, hashed component-wise; no tuple, no allocation |
| ~~Are `from` and `as` reserved or contextual?~~ | **Decided**: fully reserved, because raw identifiers (`` `name` ``) make the escape one delimiter rather than a lookahead rule per keyword |
| Float vs range lexing — `.5` is currently illegal by accident | `grammar-script-notes.md` |
| Destructured params do not parse | `grammar-script-notes.md` |
| Is `style` a reserved prop name? | `grammar-script-notes.md` |
| Nested record patterns | `patterns.md` — **promoted**: with wrapping done by enum payload rather than a dynamic chain, `.DEFERRED { cause = .TIMEOUT { … } }` is how a nested cause is recovered at all, so this is an error-handling dependency rather than a convenience |
| ~~`?`~~ | **Decided** in `error-handling.md`: three propagation rules (types match, a declared `From`, or widening to a conformed interface), all checkable from a signature. The recorded objection was to Rust's *blanket* conversions, which elp cannot have. |
| The type switch | `error-handling.md` — **no longer urgent**: its only customer was branching on `Either<_, Error>`, and one concrete error type per layer makes that a closed-enum `match`. Still open, no longer load-bearing. |

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

## What `pure` being designed left open

- **A candidate third body property.** `aliasing.md`'s restriction on what
  may appear inside a `Synchronized` transform is one. Two body properties
  are two special cases; three is a small algebra, and the honest move at
  that point is to design one. Owner: `functions.md`.
- **`realtime` on a helper is new syntax**, and the grammar has to admit a
  second function modifier. Owner: `grammar-script-notes.md`.
- **A `pure` `@external fn` is not sayable**, deliberately. `strlen` is pure
  and cannot be declared so. Revisit when a corpus program wants it.
- **The evaluator's step budget** — a number and a diagnostic, now that
  `pure` is known not to imply termination. Owner: `macros.md`.

## What `Protected<T>` being designed left open

- **A diagnostic attribute on a declaration**, so a library can say what a
  missing conformance *means* — Rust's `#[diagnostic::on_unimplemented]`.
  `Protected<T>` is the first customer and `stdlib-boundary.md` notes it is
  the bucket-1 feature that keeps two types out of bucket 2. Unowned.
- **What string interpolation accepts.** The corpus interpolates primitives
  directly and calls `.toString()` for everything else, consistently, and
  `strings.md` does not say so. `Protected<T>`'s first leak row depends on
  it. Owner: `strings.md`.
- **`expose_protected`** is a named lint and wants a default level, which
  is `diagnostics.md`'s standing open item arriving with a customer.

## Migration backlog — after the corpus scan

The scan ran (`core-language-probes.md` §16) and did the mechanical half.
What it did **not** do, and why:

1. ~~**Eleven components still use the pre-`component` form.**~~ **Done** —
   the three original apps are swept (`core-language-probes.md` §16, "The
   sweep"), and the component form was the smallest thing wrong with them.
2. **`nil` versus `null`.** The scan sharpened it rather than settling it:
   `null` survives in exactly the seven FFI-heavy files and `nil` everywhere
   else, which is too consistent to be drift. It may be a deliberate line
   between an empty `Optional` and C's null pointer, and nothing records an
   intent. Third sighting.
3. `name-resolution.md`'s own corpus migration backlog, four items, none
   applied.

Done by the scan: twelve `@external` out-parameters and five call sites took
`&var`; six cell round-trips became transforms; two payload types went
`const` to `var`; three method names were aligned to `collections.md`'s
surface list.

## What the sweep of the three original apps opened

- **May a field carry a default, and when does it run?** `id = UUID.v4()`
  was used by the old corpus and no document says. Once per type or once
  per construction is the whole question, and object literals being checked
  exhaustively for an owned aggregate (`patterns.md`) is what a default
  would weaken. Owner: `aliasing.md`, which owns field declarations.
- **Go-style field tags.** A second untyped metadata channel on every
  field, readable by macros, used by the old corpus and absent from
  `macros.md`. Its stated job is what `@CodeGen(Codec<JSON>)` does. Owner:
  `macros.md`.
- ~~**Keying a markup `for` by the element's own value.**~~ **Decided** in
  `control-flow.md`, after looking at what SwiftUI and Compose do: a
  duplicate key **traps**, which is Compose's answer and the only one
  compatible with refusing undefined results, plus a new lint
  `identity_by_value` for the statically visible shape. The two corpus
  sites are kept as evidence; their real fix is a rope's node identity
  (`examples/Editor` FINDING 5, third program).
- **Objective-C interop.** `@external(.OBJC)` was used and only `.C` is
  designed. Owner: `functions.md`.

## What the corpus scan opened

- **`&` has three meanings** — a borrow, a mutable borrow, and, in prop
  position, *constructing a `Binding<T>`*, which is not a borrow at all.
  Decided deliberately and reads identically to the first. Owner:
  `reactivity.md`.
- **Do mutating collection methods return unit or `Self`?**
  `collections.md` says unit; two documents quote `xs.copy().append(y)` as
  an expression. The migration removed the sites, so this is now a question
  about the surface rather than a contradiction. Owner: `collections.md`.
- **The `-ing` method pair.** `inserting`/`removing` read as "returns a new
  collection" and the corpus used them interchangeably with the mutating
  names. Whether non-mutating variants should exist at all is open. Owner:
  `collections.md`.

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
