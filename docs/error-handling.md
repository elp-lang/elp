# elp error handling — values for errors, a separate boundary for panics

## The decision

**Errors are values (`Either<T, E>`), always returned, always matched
explicitly. There is no throw/catch/boundary control-flow mechanism for
error values anywhere in the language.** This applies uniformly to `.elp`
script code and `.velp` component code — one model, not two. (Panics —
bugs, not `Err` values — get a separate, narrower containment mechanism;
see "Panic containment" below. The two are unrelated and shouldn't be
conflated.)

This was a real fork, not the only option on the table. The alternative
considered was "two mechanisms": ordinary `Either` matching for expected
failures, plus a separate UI-level boundary that would catch an error left
"unconsumed" anywhere in a component subtree and surface it at the nearest
ancestor boundary, so a forgotten `match` somewhere deep in a tree couldn't
silently misbehave or take down the app. Rejected. Reasoning, in the
decision-maker's own words: mechanism 1 alone "solves more problems than
it creates," and the invisibility of an error going unhandled deep in a
tree is a userspace problem, not a compiler one — intentionally so. The
team building an app is responsible for their own error-propagation
standards; the compiler's job stops at guaranteeing a value is never
silently dropped, not at guaranteeing it was handled *well* or *close to*
where it occurred.

## Why one mechanism is enough, not risky

The load-bearing rule making this safe rather than reckless already
exists and predates this doc: `grammar-script-notes.md` gap #3 — the
compiler is expected to warn (error) on an `Either<_, E>` value that
escapes a scope unconsumed, Go-style. That rule is what stops "no
boundary" from meaning "errors can vanish." It doesn't mean every error
lands somewhere *sensible* — it only means every error is *touched*
(matched, or explicitly returned onward as a value) at every step. Whether
the resulting propagation chain is well-designed is exactly the part left
to the team, deliberately.

## `throw` in `.velp`

`throw` stays as syntax, but strictly as sugar for `return Err(...)` —
nothing more. No bubbling past the function it's written in, no boundary
catching it, no default recovery behavior. The caller of a
`throw`-containing function receives an ordinary `Either` and matches it
exactly like the result of any other function call. This supersedes the
framing in the original `login-form.velp` comment and in
`grammar-script-notes.md` gap #4, both of which described `throw` as
bubbling to "the default root error boundary" — that boundary doesn't
exist; both are being corrected alongside this doc.

Why `.velp` gets this sugar and `.elp` doesn't: unchanged from gap #3/#4's
original scope split — script code stays fully explicit, `throw` is a
component-body ergonomic. Nothing here revisits that split; it wasn't
part of the boundary question this doc resolves.

## One error type per layer, and the existential only at the top

**Decided**, after a survey of the corpus on 2026-11-20 that counted the
thing nobody had counted: of 59 fallible signatures, **51 return the bare
existential `Either<_, Error>`** and 8 return a concrete type — and every
`Err(...)` handler in 11,300 lines of corpus does one of exactly two
things, `e.toString()` or `Err(Wrapper.from(e))`. **Not one program
branches on which error it got.**

That is not a style problem, it is information destroyed at the boundary.
`LinkShortener/main.elp` answers a disk failure with `CONFLICT` because
`store.put` returns `Either<Link, Error>` and `CONFLICT` is the only thing
the call site can say. `SocialFeed` cannot tell a 404 from a dropped
connection. In both cases the error *arrived* knowing which it was.

C++, Go and Rust reached the same arrangement from three different
starting points, and it is the one elp takes:

| | |
|---|---|
| **A concrete error type per layer** | Each layer's type names only what that layer can fail with. Rust: `thiserror` in a library. Go: a typed error wrapped with `%w`. C++23: `std::expected<T, E>` with a real `E`. elp: an `@error` enum, which the corpus already had and had only ever used for leaf errors it then discarded. |
| **Converted at the boundary** | A layer converts the layer below's error into its own, and the conversion is where the policy lives. |
| **The existential at the top, where nobody branches** | Rust's `Box<dyn Error>` and `anyhow::Error` exist for a real reason: at the top of a program there is nobody left to branch on behalf of. The error is logged, reported, and the process exits. |

So the rule is not "never use the existential". It is **the existential
belongs where nobody branches, and that is about one function per
program.** New lint, `existential_error`, warn by default: an `export fn`
returning `Either<_, Error>` where `Error` is the interface. The message
names the concrete type the body actually produces, because in every corpus
instance the body knows.

`examples/Recovery` is the probe. Its three layers are `TransportError`
(five failures, four different responses), `OutboxError` (three, all local)
and `SyncError` (five, each with a different follow-up including one that
pages somebody), and the shape of the program is that the five arms at the
top get five different log levels — which is the entire reason the type has
five variants instead of a string.

### What `@error` generates, and `@message`

`macros.md` lists `@error` as a directive and five corpus files use it.
**None of them says what `toString()` returns**, and for
`RenderError.NO_SWAPCHAIN` the only thing derivable from the declaration is
the variant's name. "NO_SWAPCHAIN" is not a sentence anybody can act on.

thiserror's answer is a per-variant format string and it is the feature
that makes the macro worth having rather than a conformance stub. Spelled
`@message`, over the variant's own fields, in elp's existing interpolation
syntax:

```elp
@error
export enum TransportError {
	@message("the request timed out after {elapsedMs}ms")
	TIMEOUT { elapsedMs int },

	@message("gave up after {attempts} attempts: {cause}")
	EXHAUSTED { attempts int, indirect cause TransportError },
}
```

No new expression language, no positional arguments, and the fields are
already named. Two things fall out of it:

**Error wrapping inside one layer makes the error type recursive**, and
`indirect` is exactly the ratified mechanism — sixth sighting, and the
first that is not a tree. "I gave up after six attempts" is useless without
"and here is what the sixth said", so the retry variant carries a
`TransportError`. The box being immutable is what keeps the enum `Copy`
(`memory-model.md`, `indirect` rule 1), and an error type being `Copy` is
not a detail: it is what lets a failure be stored in a report, logged, and
matched again with nobody writing `.copy()`.

**`@message` cannot be specified until string interpolation is.**
`{cause}` is not a primitive. `strings.md`'s open item records that the
corpus interpolates primitives directly and calls `.toString()` for
everything else, consistently, and that no document says so. Either
interpolation accepts any conformer and `{cause}` is right, or it accepts
primitives only and this must read `{cause.toString()}`.

The case that decides it is the one that **must not compile**:

```elp
@message("the token {token} was refused")
REFUSED { token Protected<string> },
```

`Protected<T>` is secret by *absent conformances*, so it has no
`toString` — an interpolation that requires one refuses this on its own,
with no rule about errors and secrets anywhere in the language. That is the
design working. An auth token in a log line is the most ordinary security
defect there is, and elp is one unwritten sentence away from refusing it.

## `?` — decided, and the objection does not reach elp

**Decided.** This section previously read *"still not decided"* and
recorded the objection precisely: not propagation-shorthand as a category,
but *"its implicit `Into`/`From` conversion across error types — the
auto-coercion machinery, not propagation-shorthand as a category, and how
that machinery complicates `Copy`/derive and lifetime management"*.

**The objection was to Rust's blanket conversions, and elp cannot have
them.** `SocialFeed/internal/api.elp` already declares

```elp
ApiError implements Error + From<HttpError>

fn ApiError.from(value HttpError) -> Self { ... }
```

`From<Source>` is an ordinary nominal conformance in elp: one declaration
per pair, greppable, with the policy visible in a named function. Rust's is
implicit because a blanket `impl` can exist for a pair nobody wrote; elp
has no blanket conformances, so every conversion `?` can perform is a
conformance somebody declared on purpose. Four corpus files already write
that conversion **by hand, at every call site**.

`expr?` in a function returning `Either<_, E>` evaluates `expr`, yields the
`Ok` payload, and on `Err(e)` returns early with `Err` of **one** of:

1. **`e` itself**, when the error types match exactly. (The narrow operator
   this doc previously offered as the fallback position.)
2. **`E.from(e)`**, when `E` declares `From<SourceError>`. The conversion
   is a declaration, not inference.
3. **`e` widened**, when `E` is an interface the source type conforms to —
   ordinary existential boxing (`dispatch.md`), and the rule that makes
   `?` work in `main`. Rust has this rule too and nobody notices, because
   the blanket `impl<E: Error> From<E> for Box<dyn Error>` hides it inside
   rule 2.

All three are checkable from a signature alone, which is the same bar
`functions.md` sets for a body property. None of them is a coercion in the
numeric sense, and `operators.md`'s ban on implicit numeric conversion is
untouched: these convert *error types*, by a named conformance, in a return
position.

**The cost, stated rather than discovered later.** There is one `from` per
pair, so one global mapping policy per pair. A call site that needs
something else — `examples/Recovery` has one, where the same
`TransportError` becomes a retryable `SyncError` or a permanent one
depending on the variant — writes the `match` out and does not use `?`.
That is correct, and it is the honest version of Rust's
`.map_err(|e| ...)`. A `from` is a **lossy decision**, and that is the
feature: `ApiError.from(HttpError)` maps status ranges onto three variants
and discards the rest, once, in one place a reviewer can find.

### The cleanup problem `?` creates, and why it is not a new keyword

`?` works by inserting an early return the reader cannot see. The moment a
function acquires something before the first `?` — a claimed work item, a
created row, an opened handle — every invisible return leaks it.
`examples/Recovery/sync.elp` claims an item out of `pending` into
`inFlight` and then has six error paths; an item left in `inFlight` is
never retried and never acknowledged.

That problem is real, derived twice from the corpus, and **older than this
round**: `memory-model.md`'s scoped-release section lists it as problem (2),
in these words — *"a leak per early exit. `examples/VulkanTriangle`'s
construction path holds five handles by the end, so every `throw` on it
must tear down everything built above — C's `goto fail` ladder, written
out."*

**`errdefer` was proposed here and is WITHDRAWN.** It was Zig's keyword,
and precedent is not a derivation — which is exactly what
`functions.md`'s admission test exists to refuse. Two things killed it:

**The rule was wrong.** I wrote *"it runs when the enclosing scope exits
with an `Err`, and not otherwise."* Zig has no cancellation, so "exits with
an error" is a complete classification there. elp's `defer` is specified to
run **on normal exit, on cancellation unwind, and on panic unwind**
(`reactivity.md`), and a cancellation inside `Renderer.new` leaks all five
Vulkan handles under the rule as stated. The borrowed word brought a
borrowed taxonomy that does not fit a language with three kinds of exit.

**And the problem already had an owner** — `with ... as x { }`, which was
ratified *as the answer to this problem*, with multiple bindings releasing
in reverse. **That construct was deleted on 2026-11-20** (`memory-model.md`,
"Release"): `release` takes `&self`, release is `defer x.release()`, and
there is no keyword. The decision-maker's reasoning was to keep magic words
low and let the engineer write explicit code.

Which clarifies this section rather than undermining it, because the
deletion **narrows the candidates to one.** What is left needs to cover a
resource being *assembled for a caller*:

| | |
|---|---|
| **`defer` with an exit binding** — the only live candidate | `defer (exit) { … }`, where the block can see which of the three exits happened. One grammar rule on a keyword that exists, no new word, and it answers the per-failure question below for free. A plain `defer` cannot do it, because it runs on success too and the handles must leave with the returned value. |
| ~~A consuming `with` binding~~ | Gone with the keyword. The whole of it was in service of by-value release, and `release` no longer takes `self`. |
| ~~RAII~~ | Already decided against, with a cost table in `memory-model.md`. |
| ~~`errdefer`~~ | Withdrawn above. |

Until something is ratified, the error path is the `goto fail` ladder
written out, which is what the corpus has — and which is what "explicit"
means here rather than a gap being tolerated.

**And one thing no candidate reaches**: `Recovery/sync.elp`'s compensation
is not a release at all. Restoring a claimed item to the outbox is a
**compensating action** on a different object, so there is nothing whose
`release` could do it. Only the exit binding reaches it, because the
compensation is ordinary code in a block that knows how the scope ended.

And the sub-question `errdefer` raised survives it: in `sendOne` three
failure paths want the compensation and three (the permanent ones) want
the item dropped, because a rejected item put back in `pending` is retried
forever against a server that will always say no. A guard that releases on
every failure is wrong there. Whichever mechanism wins has to answer it,
and the `defer (exit)` candidate is the only one that answers it for free.

## What the 36 `throw` sites were really asking for

The migration found the largest single violation of a decided rule the
sweeps have turned up: **`throw` is used in 7 `.elp` files at 36 sites**,
and `grammar-script-notes.md` gap #4 has forbidden it in the script dialect
since it was written — `throw` is a `.velp` ergonomic and script code
returns an `Either` explicitly.

Two thirds of those sites are ordinary (`throw E` where `return Err(E)`
would do), and the rule stands for them: two spellings of one thing is the
mistake `realtime` already made once in this record. They are migrated.

The other third is `x ?? throw E`, and it is **not** sugar for
`return Err(E)` — it is `throw` in *expression* position, which makes it a
**diverging expression**, and that is a thing elp has never specified.

### And the corpus already depends on divergence, which is the real finding

The first draft of this section said the 36 sites rested on an undesigned
feature and should therefore go. Half right. The sites should go, and the
feature is not merely undesigned — **it is undesigned and already
load-bearing**:

- `Json/internal/parser.elp` writes `self.take()? ?? return Err(...)`.
- `Untrusted/internal/ingest.elp` writes `?? return Err(...)` twice.
- `Interpreter/interpreter.elp` has `return Err(...)` as a `match` arm
  whose sibling arms produce values.
- Every `break` and `continue` in an expression position is the same thing.

A `return` in an arm of a `match` that is itself an expression has no value
and must be admitted anyway, which is exactly what a bottom type is for.
So elp has **three diverging forms already** — `return`, `break`/`continue`
— and no specified type for them. `control-flow.md`'s open item *"a
diverging `while true`"* is the same question arriving from a third
direction, and it now has customers in four files rather than being a
hypothetical.

That is the thing to settle. `throw` in `.elp` would be a fourth spelling
of it and is refused; the bottom type under the other three is owed.

### `.orError`, which is what the sites wanted

`Optional<T>` to `Either<T, E>`, then `?`:

```elp
const instance = instanceSlot.orError(RenderError.NO_INSTANCE { code = 0 })?
```

Rust's `ok_or(e)?` with elp's naming. A stdlib method on a language type
rather than language surface, which is `stdlib-boundary.md` bucket 2, and
it replaces all twelve `?? throw` sites with two decided mechanisms.

`VulkanTriangle/renderer.elp` is where it earns its keep: eleven fallible
FFI calls in sequence, each with a status code *and* an out-parameter slot,
which is the canonical shape `?` exists for.

### And it is `VulkanTriangle` that shows the shape

GAP 4 in that file predates this round by weeks and states the problem
exactly:

> *"every `throw` above this line still leaks everything created before it
> … By the last call there are five live handles. A failure must destroy
> the swapchain, the device, the surface and the instance, in that order,
> before returning — and a failure is a value that returns immediately, so
> there is no scope exit to hook."*

Three things in that sentence, and the cleanup mechanism has to give all
three: it must run on the error path, it must run **in reverse acquisition
order**, and the handles must still be able to **leave on success**. The
withdrawn keyword gave the first, not the second, and got the third by
accident rather than by design. `with`'s reverse release gives the second
already, which is the strongest argument for keeping this inside scoped
release rather than beside it.

Two limits worth recording before anything is ratified:

- **A resource acquired in a loop.** A `for` creating n command buffers
  that fails at item k wants n−1 releases. A per-acquisition cleanup
  registered in the body runs at the body's exit, not the function's, and a
  `with` cannot bind a variable number of resources. Nothing in the corpus
  does this yet, and it is the case that would decide against every
  candidate above.
- **Order is still unenforced.** Whatever the mechanism, nothing makes the
  cleanups appear in acquisition order — they do in `renderer.elp` because
  they are written beside their acquisitions. `VulkanTriangle` GAP 5's
  point stands: the order is the specification and the compiler cannot read
  it.

## The three guarantees, and the fallible transform

C++ has had the vocabulary since Abrahams named it, and it is the right
vocabulary because it describes **what a caller may assume after a
failure** rather than how the failure happened:

| | |
|---|---|
| **nothrow** | The operation cannot fail. |
| **strong** | If it fails, nothing changed. All-or-nothing. |
| **basic** | If it fails, invariants hold and nothing leaked, but the value may have been modified. |

`aliasing.md`'s `update(nonsuspending fn (&var T) -> R) -> R` gives the
**basic** guarantee, and the register has carried that as *"a failing
transform leaves a partial write"* — a defect the `&var T` widening
introduced, with three candidate answers and no decision, and no corpus
program that did one. `examples/Recovery/internal/outbox.elp` does one, and
it has no correct handler: two items are marked delivered, the third's id
is unknown, the count is advanced by two and the `Err` says only
`UNKNOWN_ITEM`. The caller cannot retry (double-counting the first two) and
cannot give up (they really were delivered). **No correct handler is the
signature of a missing guarantee, not of a missing arm.**

**Decided: `commit`, the strong guarantee by copy-and-swap.**

```elp
commit(nonsuspending fn (&var T) -> Either<R, E>) -> Either<R, E>
```

C++'s answer to "all-or-nothing, and my operation can fail" is
copy-and-swap, and elp has both halves already: `Duplicate` is the copy,
and a move into the cell is the nothrow swap. Take the lock, copy the
payload, run the closure against `&var` the copy, move it back on `Ok` and
drop it on `Err`, release the lock. Available when `T: Duplicate`.

Three consequences, and they are why this is worth ratifying rather than
leaving to each author:

- **It closes the cancellation hole.** The register lists *"cancellation
  can unwind out of a transform"* as **unsound** rather than open, because
  the payload is left half-written and all three candidate answers were
  about failures the author wrote. Under `commit` a cancellation unwinds
  out of the closure while the cell still holds the original — the
  unsoundness is gone for every `Duplicate` payload, with no new rule
  about cancellation anywhere.
- **It costs a copy inside the critical section**, so it is forbidden in a
  `realtime` body for the same reason everything else that allocates is.
  The right cost to pay: a `realtime` body has no failure path to be
  atomic about.
- **It puts the guarantee at the call site.** `.update(` means the payload
  may be half-written; `.commit(` means it is not. All three of
  `aliasing.md`'s candidates left the call site looking identical.

And it answers the question the candidates could not. *"Require the
transform to be total, so `R` may not be an `Either`"* was rejected as too
strong — it bans a shape to fix a subset. The conditional form is not:

> **A transform whose `R` is an `Either` must be `commit`, not `update`,
> unless the author opts into the basic guarantee explicitly.**

The diagnostic, which is the test `diagnostics.md` sets:

```
error: this transform can fail, and a failure leaves the payload partly written
  --> outbox.elp:84
   |  return self.state.update(fn (&var s) -> Either<int, OutboxError> {
   |                    ^^^^^^ returns `Either`, so `update` gives only the
   |                           basic guarantee
   = note: `OutboxState` is `Duplicate`, so `commit` can give the strong one
   = help: if a partial write is intended, say so with `update(basic: ...)`
```

**The nothrow tier has no spelling, and it is the one the other two rest
on.** C++ marks it `noexcept` precisely because the guarantee is only
useful if the compiler can be told. `Outbox.restore` must never fail — a
restore that could fail would leave an item in neither list, with nothing
left to compensate with — and the only record of that is its return type
and a comment. Open, owned here.

## Aggregation: a typed report, not a joined error

Go added `errors.Join` in 1.20 and it is the right primitive for Go: a
deferred `Close` failing alongside the operation it was closing, two
independent failures of equal weight, neither dropped. It is the **wrong**
primitive for elp, and the reason is about elp rather than about Go.

A joined error is opaque — you recover the parts by walking a dynamic chain
with `errors.Is`/`errors.As`. **elp has no dynamic chain.** Wrapping here is
an enum payload (`.DEFERRED { cause }`): static, matchable, `Copy`. So an
aggregate must be *typed*, and once it is typed there is no reason to
pretend it is an error:

```elp
export object SyncReport {
	const delivered [Receipt],
	const failed    [SyncFailure],
	const remaining int,
}
```

A partially successful batch is not a failure, it is a result with two
lists in it — which is also why it is not `Either<SyncReport, SyncError>`.
There is nothing a caller could do with an `Err` that it cannot do better
with `failed`.

The cost: no generic "does this failure involve a timeout, anywhere down
the chain". Recovering a nested cause is a nested pattern
(`.DEFERRED { cause = .TIMEOUT { elapsedMs } }`), which makes
`patterns.md`'s **nested record patterns** an error-handling dependency
rather than a convenience.

## Handle an error once: logged, or returned, never both

The most portable piece of error-handling advice Go has produced, and it
costs no language surface. A layer that logs *and* returns turns one
failure into four log lines at four layers, none of which is the one with
the context. `Recovery/sync.elp` records and returns; `Recovery/main.elp`
logs, once, because it is the layer that will not return it any further.

Convention, not a rule — it is not checkable without knowing what counts as
logging, and the lint that tried would fire on every sensible `Log.debug`.

## A library may not trap on input

elp traps in several designed places: a duplicate `by` key
(`control-flow.md`), a nested `update` on one cell, numeric overflow. A
trap is a panic, and a panic means *bug*, which makes the rule a
consequence rather than a new policy:

> **A trap reachable from untrusted input is a defect, and the fallible
> form has to exist.**

`examples/Untrusted` already found one — `by` over server-supplied ids is a
remote crash — and this is the sentence that says why the `Keyed<K, T>`
direction is not optional. It is also Go's rule (a library returns an
error, it does not `panic`) and Rust's (`panic!` is for broken invariants,
not for bad input), arrived at from elp's own trap list.

## Matching an interface-typed value always needs a fall-through

A `match` on a value whose static type is an **interface** can never be
exhaustive, because the set of conforming types is open — any package may
add a conformance. So a catch-all arm is **mandatory** there, and it is
forced by the type system rather than imposed as a style rule.

This matters most for `Error`, which is the interface the corpus matches
on. `BlogStyle/internal/api.elp` comments that a function returning
`Either<Content, Error>` can be matched "at the call site … to work out
whether it was an ApiError or a SerializationError".

Which surfaces a construct the language does not have: **matching an
interface-typed value by concrete type** is a *type switch*, distinct
from matching an enum's variants, and neither its syntax nor its
semantics is designed. Whatever it ends up looking like, the mandatory
catch-all applies to it.

**Amended 2026-11-20: the type switch has lost its only customer.** The
quoted comment is `BlogStyle`'s, and it describes the style the decision
above replaces — a function returning `Either<Content, Error>` whose caller
must ask *"was that an ApiError or a SerializationError"*. With a concrete
error type per layer the question does not arise: `Recovery/main.elp`
matches a `SyncError`, a closed enum, so five arms cover it exhaustively
and a sixth variant added later is a compile error at that line rather than
a silently wrong log level.

So the type switch is not needed for error handling, which is the only
place the corpus wanted it. It may still be wanted elsewhere, and the
mandatory catch-all above still applies wherever an interface is matched —
but "nobody needs it yet" is not "it is wrong", so this stays recorded
rather than closed, and it is no longer urgent.

### Enums are different, and a catch-all is a choice there

An enum's variant set is closed and known, so a match over one *can* be
exhaustive — and a catch-all is always available when most arms are
irrelevant. The corpus already relies on both halves:

- `ChatRoom/main.velp`'s `AppReducer` is exhaustive with no catch-all,
  and its comment says so deliberately.
- `AppMiddleware`, in the same file, handles one variant and ends `_ ->
  nil`, with a comment noting the match "ISN'T exhaustive and we DO have
  a catch-all".

So nothing forces every arm to be written. **A match with no catch-all
must be exhaustive; writing `_` opts out.** That is what keeps
`Category.label` and `PatternError.toString` honest — five arms for five
variants, and a compile error the day a sixth is added, rather than a
silently wrong string from a catch-all that should never have run.

*Open: whether a library author can mark an enum open
(`#[non_exhaustive]`-style), forcing downstream catch-alls so that adding
a variant is not a breaking change. elp pins one version per package and
makes conflicts a build error, so a new variant can only ever arrive
through a deliberate upgrade — which makes the break visible and
informative rather than surprising. That argues for closed-by-default
with an opt-in marker later, not for the reverse.*

## Panic containment: `RecoveryBoundary`

Unrelated to everything above — this section is about bugs, not `Err`
values. An app shouldn't crash out because one component somewhere in
its tree panicked. That needs containment, and containment needs a
recovery story, but recovery has a hard limit: some panics happen because
data was corrupted out of band, and pretending those are recoverable is
worse than crashing. The design here rides on a split Rust already makes
for its own reasons, rather than inventing a new classification scheme.

**Rust's `catch_unwind` only ever catches a clean, cooperative unwind** —
drop handlers ran, nothing left in a state Rust itself can't account for.
A double panic during unwind, a stack overflow, or a panic trying to
cross an `extern "C"` boundary all abort the process outright and never
reach `catch_unwind` at all. That last case is concretely relevant here,
not hypothetical: the UIKit pipeline calls into ObjC via raw
`objc_msgSend`, real unsafe FFI surface. So "can't and shouldn't recover
from data messed with out of band" isn't something elp needs to detect —
it's exactly the set of failures Rust already refuses to hand a recovery
opportunity for. `RecoveryBoundary` wraps `catch_unwind` around a
subtree; the corruption/abort case is handled by never reaching it.

**Scope of what a boundary wraps:** a subtree's render *and* its
event/callback execution, not just the initial render pass. A panic
inside `handleSubmit` mid-interaction is exactly as much a containment
concern as a panic during `render`.

**Recovery means discard-and-reinit, never resume-in-place.** A panic
mid-mutation of reactive state can leave it logically torn even though
Rust's memory safety still holds — the same reason `std::sync::Mutex`
poisons itself when its lock-holder panics. So a caught panic discards
the crashed subtree's existing state, renders the developer's fallback,
and — if the developer offers a retry — re-initializes that subtree fresh
from its props. Never "patch and continue" with the state as it was at
the moment of the panic.

**Nesting:** nearest-ancestor-boundary wins, same as React's error
boundaries. An outer `RecoveryBoundary` only sees a panic if no closer
one exists. Ordinary, well-understood behavior; no new semantics needed.

**An implicit root boundary always exists**, runtime-provided, even if a
developer writes zero `RecoveryBoundary`s anywhere. This is what actually
delivers "an app shouldn't crash out" unconditionally — user-authored
boundaries only narrow the blast radius further; they're never the only
thing standing between a panic and a full crash.

Deliberately not named `ErrorBoundary` — that name already carries the
"bubble an unconsumed `Either` to a boundary" idea from earlier in this
doc's history, which was considered and rejected. `RecoveryBoundary` is
unrelated to `Either` entirely; conflating the two is exactly the
confusion the naming is meant to avoid.

### Open questions

1. **Default fallback for the implicit root boundary.** Some minimal
   runtime-provided UI is needed for the case where a developer wraps
   nothing themselves. Not designed.
2. **Retry API shape.** "Re-initialize fresh from props" implies the
   runtime hands the developer's fallback some way to trigger that.
   Not designed.
3. **Diagnostics on a caught panic** — what surfaces to the developer
   (logs, a dev-mode overlay, a hook to report to a crash service).
   Not discussed.
