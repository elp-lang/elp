# elp functions, methods and the call model

## A method belongs to the package that owns its receiver

**`fn T.method(...)` may be declared only in the package that defines
`T`.** There are no extension methods on foreign types.

This is what Go and Rust both do: Go permits methods only on types
declared in the same package, and Rust permits an inherent `impl` only
on a local type. The principle in the user's words — *you cannot modify
the signature of a package you do not own, or a module out of band.*

It is stronger than the orphan rule alone implies, so it is worth
stating separately. The orphan rule (`interfaces.md`) governs
*conformances*; a plain method is not a conformance, so nothing in that
rule prevented `fn [Expense].total()` until now.

The reason it has to be forbidden is the one that made conformances
global in the first place. `name-resolution.md` settled that a
conformance needs an import to be *named* but not to be *dispatched*,
which rests on there being exactly one conformance of `I` for `T`
anywhere in a program. An extension method has no such guarantee: if
two packages each declare `fn [Expense].total()`, the winner depends on
what the calling file imported, and method dispatch becomes
import-sensitive. That is the property the design spent the
global-conformance decision to avoid.

Swift accepts that cost and pays it with import-scoped extension
visibility. Rust declines it. elp declines it.

### The sanctioned route is an interface you own

The restriction is narrower than it first looks, because the orphan
rule is deliberately two-sided: `T implements I` is legal in the
package defining `T` **or** the package defining `I`. So a package may
add behaviour to a foreign type through an interface it declares
itself:

```elp
interface Summable {
	fn total(&self) -> Money
}

[Expense] implements Summable

fn [Expense].total(&self) -> Money { ... }
```

This is Rust's extension-trait pattern, and it is the right shape for
the same reasons: the addition is **named** (someone can see what was
extended and by whom), **declared** (it is a conformance, not an
ambient method appearing from an import), and **unique** (the orphan
rule guarantees one).

And it is better than Rust's, because of a decision already made.
`name-resolution.md` settled that a conformance needs an import to be
*named* but not to be *used*: if you hold an `ApiError`, `.toString()`
is callable whether or not `Error` is in scope. So `expenses.total()`
works anywhere in the program without importing `Summable`.

That is worth saying plainly: **elp gets the ergonomics of extension
methods — a method that is simply available — without the hazard,
because the orphan rule makes it unique and the conformance declaration
makes it attributable.** Rust has the safety but requires `use
Summable` at every call site; Swift has the ergonomics but pays with
import-sensitive dispatch. The combination here is a consequence of two
earlier decisions meeting, not a new feature.

So `Expenses/internal/stats.elp`'s four free functions over
`[Expense]` — `total`, `byCategory`, `breakdown`, `largest` — are free
functions by the author's choice, not by prohibition. They could be a
local interface. The corpus is left as it is because the free-function
form reads fine and nothing in it chains.

## No pipe operator

**`|>` is rejected.** The corpus never asks for it, and the survey is
exhaustive rather than suggestive:

| | |
|---|---|
| free functions | 13 |
| methods | 28 |
| nested free-function calls `f(g(x))` | **0** |
| method chains at depth 3+ | 1 — `UUID.v4().toString().replace("-", "").slice(0, 6)` |
| method chains at depth 2 | 9 |

Three arguments, in order of weight:

**1. It is a second spelling for method chaining.** Piping earns its
keep in F#, Elixir, Elm and OCaml because functions there are not
methods, so `f(g(h(x)))` reads inside-out and the pipe repairs the
reading order. elp declares `fn Money.add(&self, ...)`, so
`x.h().g().f()` already reads in the order it executes. Two ways to
write one thing is the objection that removed `..` as a catch-all
(`collections.md`) and merged `ToFrom`/`FromTo` into `Codec`
(`generics.md`).

**2. It has to invent an argument position.** `x |> f(a, b)` must
decide where `x` lands. F# says last; Elixir says first. elp has named
arguments and `generics.md` made parameter names part of the signature,
so a positional convention hidden inside an operator is the kind of
implicitness the language rejects everywhere else.

**3. The corpus's free functions are parallel, not composed.**
`breakdown` calls both `total(expenses)` and `byCategory(expenses)`;
each takes `expenses`, neither consumes the other's output. There is
not even a latent pipeline to shorten.

For genuine composition the answer is a named local, which reads better
than a pipe because it names the intermediate value rather than leaving
it anonymous.

## The demand the corpus actually voiced

`Expenses/internal/stats.elp` says it outright:

> *"Grouping is the single most common thing an app does to a list and
> elp currently has no vocabulary for it. Written out longhand with a
> Map, which also raises the question of what makes `Category` a valid
> key."*

That is a **collections** gap, not a piping gap. `map`, `filter`,
`reduce`, `sorted` and `groupBy` belong to `Array<T>`, which stdlib
owns — so they are ordinary methods, they chain left to right, and
neither `|>` nor an extension mechanism is needed to reach them.

This is also why rejecting both is safe. The construct that would
produce long chains is stdlib's own property, so the restriction above
never stands in its way.

## Function values

### The type spelling

```
fn(T1, T2) -> R        // `-> R` omitted for a function that returns nothing
```

Already in the corpus: `fn Store.ask<T>(&self, build fn(Channel<T>) -> StoreMsg) -> T`.

**A function type has no parameter names.** `generics.md` made names part
of a signature in order to resolve overloads, and a function *value* has
no overload set — there is exactly one thing to call, so a label has
nothing to disambiguate. `onSubmit(email, password)` is necessarily
positional.

The names on `*fn onSubmit(username string, password Protected<string>)`
survive because that is a **declaration form**, not a type. It declares
a prop, documents the names for a reader, and its type is
`fn(string, Protected<string>)`.

### `*fn` declares a required callback prop

**A callback prop follows its declared type like every other prop.** It
is required, and the compiler checks at every use site that one was
supplied.

This makes `ChatRoom/components/login-form.velp` a corpus bug.
`memory-model.md`'s Finding 4 observed that `*fn` was *"still nilable at
runtime despite being marked as a required callback"*, because the
component calls `onSubmit.isNil()`. That is a special implicit rule for
one prop kind, in a file whose line directly above spells its optionality
out: `export var email Optional<string>`.

**An optional callback is an ordinary prop with an optional function
type:**

```elp
export var onSubmit Optional<fn(string, Protected<string>)>
```

So `*fn` is sugar for the common case and the general mechanism covers
the rare one. A trailing `?` on the declaration form was considered and
rejected: `*fn f(a A) -> B?` cannot say whether the `?` applies to `B`
or to the function.

### Why the `*` earns its place

Without it, a bodiless `fn` inside a component body is indistinguishable
from a method whose body the author forgot — and it would silently
become a prop. With it, a bodiless `fn` *without* `*` is an error.

### Closure literals

```elp
fn (a, b) { return b.total.compareTo(a.total) }
fn { dispatch(.LOG_IN { name }) }               // no params: no parens
```

Parameter types are omitted and taken from the expected type, and may be
written when there is no expected type or when clarity wants them.

This is not the problem `generics.md` built mandatory type arguments to
avoid. There is **exactly one** expected function type at the position,
so this is propagation, not constraint solving — the Swift pathology
comes from an overload set and there is none here. Mandatory type
*arguments* remain mandatory; a closure parameter type is not one.

### `return` is always explicit

**A function body is a statement block and needs `return` to produce a
value. This includes closures.**

```elp
rows.sorted(fn (a, b) { return b.total.compareTo(a.total) })
self.ask<[Link]>(fn (reply) { return .LIST(reply) })
```

Every corpus closure originally omitted it while every named function
used it, so the corpus was already running two rules. One rule, and it
is the explicit one — consistent with mandatory type arguments, `&` at
every writable call site, and conformances written out.

**`return` in a closure returns from the closure**, not from the
enclosing function. There is therefore no way to return from an outer
function out of a closure body, which is Swift's and Rust's behaviour
and not Ruby's.

This reads oddly against `patterns.md`'s rule that a `match` arm's block
*does* produce its final expression, and the two are reconciled there:
`return` exits a **function**, and an arm does not exit a function. A
statement block needs a keyword to hand a value back; an expression
position does not. The asymmetry is forced rather than chosen.

One consequence worth naming: in `Worker`'s message loop, `return` from
the `background` closure is simultaneously the loop exit and the
coroutine's end, because the loop is the closure's last statement. That
works, and it is why the absence of `break` is a gap
(`core-language-probes.md`).

### Captures

Captures follow ordinary value rules — a `Copy` type is copied, anything
else moves unless borrowed. A closure is `Sendable` **iff every capture
is**, which is conditional conformance and needs nothing new
(`generics.md`).

**A closure's own tier is field-wise too**, which `aliasing.md` makes the
general rule: a closure is `Copy` iff every capture is, exactly as it is
`Sendable` iff every capture is. One reading covers both, and it is why
capturing a mutable alias twice needs two `.alias()` calls rather than a
capture list — `examples/Buffer`'s `View.hooks`.

`background(fn { ... })` in `LinkShortener` capturing `inbox` is exactly
that, and `Store implements Sendable` one line above is what makes it
legal. `reactivity.md` is where the cell-side consequence lives: a
reactive cell, a `Binding<T>` and a `Confined<T>` never conform, so a
`background` closure cannot touch component state or any thread-confined
shared state.

**A function type carries no capture information**, and should not —
`fn(int)` says nothing about whether the value behind it may cross a
thread. So thread-eligibility is a property of a *value*, not something
a signature can promise, and `examples/Buffer`'s `Hooks` is the shape
where that matters: an ordinary-looking pair of callbacks that cannot
cross `background`, with the error landing wherever someone eventually
tries.

Two capture questions the probes raised are still open, and both are in
`examples/Undo`: a capture that outlives its frame, and one value
captured by two closures at once.

### A name is a value; a bound method is not

`Composer(onPosted = handlePosted)` works — a plain function name is a
value, and it takes no `&`, which is reserved for writable bindings.

**A bound method reference is refused.** `money.add` as a value would
silently capture `money`; write `fn (x) { return money.add(x) }` and the
capture is visible. No corpus site wants the shorter form.

## A C function pointer, in both directions

`examples/VulkanTriangle` reaches this twice: `vkGetInstanceProcAddr`
*returns* a C function value, and any callback-based library
(`new_fluid_audio_driver2`, `sqlite3_exec`) *takes* one. A function type
here says nothing about calling convention, so neither was spellable.

Five things are needed, and only one of them is new.

### Inside an `@external` block, `fn(A, B) -> R` is a C function pointer

No new type constructor. **An `@external` block is already a context that
re-reads the types inside it** — `memory-model.md`'s FFI amendment made
`&T` mean read-and-write there, because there is no callee body to infer
from. Reading `fn(...)` as a C function pointer in the same context is the
same move, not a new kind of rule.

```elp
@external(.C, library = "fluidsynth") {
	export type FluidSettings
	export type FluidAudioDriver

	export fn new_fluid_audio_driver2(
		settings &FluidSettings,
		callback fn(Opaque<Synchronized<Playhead>>, int, &Buffer<float>) -> int,
		data     Opaque<Synchronized<Playhead>>,
	) -> Optional<FluidAudioDriver>

	export fn delete_fluid_audio_driver(driver FluidAudioDriver)
}
```

That also types `vkGetInstanceProcAddr`'s return value, which was the other
half of the gap.

### `@abi(.C)` marks a definition callable from C

A calling convention is a property of a *definition*, not of a use site, so
it needs a marker there. It is lowercase, emits nothing, and is a compiler
intrinsic — a **directive** by `macros.md`'s table, with `.C` as the
dialect exactly as `@external(.C, ...)` has it.

```elp
@abi(.C)
fn onAudio(state Opaque<Synchronized<Playhead>>, frames int, output &Buffer<float>) -> int {
	render(output, frames)
	state.update(fn (&var p) { p.advance(frames) })
	return 0
}
```

Everything else follows from machinery that exists:

- **Only a top-level `fn` can be one.** A C function pointer carries no
  environment, so a closure can never be converted — nothing is refused by
  a rule, there is simply nowhere for captures to live.
- **The conversion is by naming the function.** `new_fluid_audio_driver2(settings, onAudio, state.alias())`
  takes `onAudio` as a value, with no cast and no `&`, consistent with
  `Composer(onPosted = handlePosted)` above.
- **The body is checked as a `background` body** — see `aliasing.md`. The
  ABI does not say which thread calls it, so this is unconditional, which
  costs a synchronous comparator callback access to `Confined` state.
- **Its parameter and return types must be C-representable**, which is what
  `Opaque<T>` and `Buffer<T>` are for (`memory-model.md`).

### `@abi(.C, realtime)` — a deadline is a property of the body

`aliasing.md` adds `Atomic<T>` for a thread that must not take a lock, and
on its own that type is decoration: nothing stops an `@abi(.C)` body
calling `Synchronized.update`, interpolating a string, or appending to a
`[T]`. The guarantee has to be checked where the code is, so the marker
carries it:

```elp
@abi(.C, realtime)
fn onAudio(state Opaque<Atomic<int>>, frames int, output &Buffer<float>) -> int
```

An argument rather than a second directive, because it is a stronger form
of a check `@abi(.C)` already performs rather than a different kind of
thing.

**It is `pure fn`'s shape, not a new effect system.** *(The first half of
this sentence survived; the second half did not — see "`pure` and
`realtime` are not nested", below. A real-time body calls C and reads a
cell, so it is maximally impure, and reading the two as nested would make
`renderAudio` unwritable.)* `pure` already
propagates through calls and already bans external calls — `background`,
channel receives, I/O. The distance between pure and real-time-safe is one
further property: **allocation**. So the rule is purity's propagation plus
a ban on naming any operation that allocates, rather than a new analysis
over bodies.

The cost, stated plainly: stdlib operations have to carry an
allocates/does-not-allocate property, propagated through calls. That is
the first thing the language would track about a function body other than
purity, and it is real weight. What it buys is that the **diagnostic
becomes the teaching** — *"`\"{frame}\"` allocates; a real-time body may
not allocate"*, pointing at the line nine calls deep that would otherwise
be found as an intermittent glitch under load.

### Spelling it as an `@external fn` with a body was rejected

It is tempting, since the block already establishes the C world. But
`library =` is meaningless on a function elp defines, and the direction
silently inverts — a reader seeing a brace-bodied `fn` inside
`@external(.C, library = "fluidsynth")` cannot tell which side owns it.
Two directions want two markers.

## `pure`, and the two body properties

`pure` is depended on by eleven documents. `macros.md` rests the entire
compile-time evaluator on it; `reactivity.md` makes it what earns
memoization; `strings.md` and `collections.md` both argue from it;
`name-resolution.md` says it introduces a type. Its complete specification
until now was one line in `grammar-script-notes.md`:

> **`pure` functions containing side-effectful calls.** Enforced by the
> effect system, not the grammar.

There is no effect system. Four documents refer to one; none designs one.

### There is no effect system, and there does not need to be

An effect system is an open algebra: effects are values, they compose, a
signature carries a row of them, and polymorphism over effects is the
interesting part. Nothing in this language has asked for any of that. What
the eleven dependents actually want is **one bit, in the signature, that
the compiler can check**.

That has a precedent here, one level down. `aliasing.md` decides `Copy`
field-wise — *"that is the whole mechanism. There is no propagation rule,
no contagion rule, and nothing to derive."* The same shape works for a
function body:

**A body property is a declared bit, checked against a closed set of
forbidden operations, propagated call-wise.** `Copy` is field-wise over a
type; a body property is call-wise over a function. One pattern, two
positions.

The language has **three** such properties: `pure`, `realtime` and
`nonsuspending` — which is three keywords, and this section originally said
two was not an algebra and that a third was the moment to reconsider. It is,
and the reconsideration is "The admission test" below rather than a count:
a candidate passes four questions or it is not one. Two of the four
candidates that have arrived since fail it, which is the test doing the work
the count was doing badly.

**And there is one candidate third already queued, so "not now" has a
date on it.** `aliasing.md` requires that a `Synchronized<T>` transform
must not `background`, receive from a channel, or lock a second
`Synchronized` — *"a restriction on a closure body by position"*, which is
precisely a body property. It was going to be spelled `pure`, and the
section below is why that cannot work. If it is spelled as a third
property, this stops being two special cases and becomes a small algebra,
and the honest move at that point is to design one rather than add a third
keyword. That decision is not made here; it is recorded so the next person
to want a body property knows they are the one who tips it.

### `nonsuspending`, ratified — and it was already owed

The queued candidate turned out to be the remediation for the most serious
finding in the corpus, which is why it is decided here rather than deferred.

`examples/Rejected` case 13 and `examples/Dashboard` found that
**cancellation can unwind out of a `Synchronized.update` transform**, leaving
the payload half-written with no failure path on the page. `reactivity.md`
makes cancellation cooperative: it lands at the next yield point and unwinds.
A transform may call anything, so it may yield.

The queued restriction — *a `Synchronized` transform must not `background`,
receive from a channel, or lock a second `Synchronized`* — is **exactly the
list of yield points**. It was written to prevent deadlock and it prevents
this for free, which is the argument for it being a property rather than
three prohibitions.

**`nonsuspending` is the third body property**, and the transform's parameter
type carries it:

```elp
Confined<T>.update(nonsuspending fn (&var T) -> R) -> R
Synchronized<T>.update(nonsuspending fn (&var T) -> R) -> R
```

Expressible only because a function **type** had to admit `&var T` for the
transform to exist at all — the third amendment's decisive argument, paying
for itself a second time.

**Its closed set, and it is not a subset of either other property.** No
channel send or receive, no `background`, no `effect`, no lock acquisition.
An `@external` call **is allowed**: a blocking C call blocks the OS thread
rather than yielding the coroutine, so cancellation cannot land there. So
`realtime` forbids what `nonsuspending` permits and permits what
`nonsuspending` forbids, and neither contains `pure`. Three closed sets, no
nesting — which is the clearest evidence yet that these are instances of one
mechanism rather than three special cases.

**The cost, and it is the real one: the marking is viral.** Every helper
reachable from any transform must declare it, and a helper that does I/O
cannot be called from a transform at all. `examples/Dashboard`'s
`render(path)` is exactly that case and the refusal is correct — the fix is
to compute before the transform and write inside it, which is
`examples/Ledger` FINDING 2's validate-then-apply arriving for the third
time from a third direction.

**A panic is not covered and does not need to be.** A panic inside a
transform also leaves a partial write, and `error-handling.md`'s
`RecoveryBoundary` discards and reinitialises the component — so the
half-written state is thrown away rather than observed. Worth stating so the
gap is known to be closed rather than overlooked.

**Three alternatives, and why not.** *Defer cancellation until the transform
completes* needs no marking and makes cancellation unbounded, so a slow
transform makes a component un-cancellable and breaks `effect`'s teardown;
it is also unnecessary once the body cannot suspend. *Transactional
rollback* needs `T` to be `Duplicate` and costs the copy the transform
exists to avoid. *Declare partial updates possible* gives up the guarantee
rather than discharging it.

### The admission test, and why the set stops at three

Three body properties means three keywords, which is the cost this section
warned about and then paid without arguing. Correcting that is worth more
than the keywords are: what was wrong was not the spelling, it was
**declaring the question settled by adding a member**. So here is the test a
candidate has to pass, written after the fact and applied to the four that
were on the table.

A body property must:

1. **be a closed set of forbidden operations**, nameable without analysing
   a body — "may not acquire a lock", not "may not take too long";
2. **propagate call-wise from signatures**, so the check reads a callee's
   declaration and never its body;
3. **be checkable from a signature alone**, which is what makes it
   compatible with separate compilation and with a useful diagnostic;
4. **have a corpus program that needs it**, because three keywords is
   already two more than this language wanted.

`pure`, `realtime` and `nonsuspending` pass all four. Applied to the other
two candidates, the test does the work the member count was doing badly.

**`bounded` fails (1).** A function with no recursion can loop a number of
times its input chooses, and one allocating a fixed amount per iteration has
no useful upper bound either. Neither is a *forbidden operation*, so no
closed set sees them, and the property would promise what no checker could
deliver. Withdrawn — `core-language-probes.md` §21 has the full argument.

**Thread affinity fails (2), and the reason is more interesting than the
failure.** *"This function must run on the main thread"* is not a
restriction on a body at all; it is an obligation on **callers**. It
propagates the other way — every caller must itself be on that thread — so
it is a different mechanism wearing the same clothes, and filing it here was
a conflation. `core-language-probes.md` §10 finding 7 had it right the first
time: *"main-thread affinity is a property of a call, not of a value."* A
property of a call is not a property of a body.

So the set is three, and it stops there for a stated reason rather than for
want of candidates. The next one to arrive gets the four questions, and if
it passes, three keywords becoming four is a smaller problem than a
mechanism nobody can say no to.

### One inconsistency this leaves, and it is mine

`realtime` is now spelled two ways: as a modifier, `realtime fn
fireEventsFor(…)`, and as a directive argument, `@abi(.C, realtime)`. The
second predates the first and the first was added without reconciling them.

They are not two things. `@abi(.C, realtime)` means *this function is callable
from C **and** its body is real-time*, which is one ABI decision and one body
property stapled together. The honest spelling is both, separately:

```elp
@abi(.C)
realtime fn renderAudio(state Opaque<Confined<PlaybackState>>, …) -> int
```

Two markers, each saying one thing, and the body property reads the same on
a C entry point as on the helper it calls — which is the whole point of
propagating call-wise. Not migrated here: it touches `examples/Playhead`,
`aliasing.md` and `functions.md` in a dozen places and belongs with the
reactivity round rather than squeezed in front of it.

### The rule

1. **`pure` is declared, checked, and never inferred.** `pure fn total(…)`.
   The body is checked against the declaration, not scanned to produce one.
   This is the same argument Finding 4's third amendment makes for `&var`:
   whether a function is pure is observable at every call site in every
   other module, so it belongs in the signature rather than being a
   consequence of private code.
2. **A `pure fn` may call only `pure fn`s.** That is the whole of
   propagation. The check is local — it reads the callee's *signature*,
   never its body — which is what keeps it compatible with separate
   compilation and what makes the diagnostic point at the call rather than
   nine frames away.
3. **`PureFn` is a subtype of `ImpureFn`.** A `pure fn` may be passed
   wherever an `fn` is expected; the reverse is an error. The corpus
   already states the pair — *"the type of a function is ImpureFn and a
   pure function is PureFn. These types are type compatible"* — and this is
   the only direction in which that can be sound.
4. **A closure literal's purity is checked against the expected type, not
   declared.** `rows.sorted(fn (a, b) { … })` writes no marker. The
   position supplies the expectation, exactly as it already supplies the
   parameter types, and the literal's body is checked against it. That is
   propagation, not inference of a contract: there is no signature being
   invented, because the literal has no name to be called by elsewhere.
5. **A `pure fn`'s parameters are immutable and it may not declare `&var`.**
   Already decided in `memory-model.md` Finding 5, and now a rule rather
   than a consequence of inference.
6. **An `@external fn` is impure, and there is no way to declare
   otherwise.** `strlen` is pure in every meaningful sense and this
   forbids saying so. The alternative is an assertion nothing can check,
   in the register `aliasing.md` reserves for `Sendable` on a foreign
   type — affordable there because it is load-bearing, and here it is
   not: no corpus program wants a `pure` C call. Revisit when one does.

### The closed set — what makes a body impure

| operation | why |
|---|---|
| calling an `@external` function | the outside world, by definition |
| `background { }`, `effect { }` | spawning |
| a channel send or receive | communication, and it blocks |
| reading or writing a **mutable** cell — `Confined`, `Synchronized`, `Atomic`, `Ring`, `Published` | two calls can return two answers |
| reading or writing a component `var` | the same thing: a component `var` lowers to a cell |
| calling an impure function, or an impure function value | rule 2 |

Everything else is pure, and the list of what that includes is longer than
people expect: arithmetic, allocation, building and mutating a local `[T]`
or `Map`, string interpolation, recursion, `while`, `.copy()`, `.alias()`,
and **`Shared<T>.get()`**.

That last pair is worth stopping on, because it is where the definition
earns its keep. `Shared<T>.get()` is pure and `Confined<T>.get()` is not,
and the dividing line is immutability — the same argument that makes
`Shared<T>` `Copy` for any `T`, that makes `string` `Copy`, and that makes
an `indirect` box leave its container `Copy`. Four decisions, one argument,
and this one was reached from a different direction.

### What `pure` does not promise

Stated explicitly, because two separate rounds of design reached for `pure`
to supply something it does not have.

- **Not "no mutation".** A `pure fn` mutates its own locals freely —
  `examples/Expenses` writes `var sum = Money.zero()` and assigns to it in
  a loop. Finding 5 already says `Copy` is the escape hatch for deriving a
  mutable local from an immutable argument.
- **Not "no allocation".** Deliberately. See the next section.
- **Not "no aliasing", and this is the one that cost two rounds.**
  `.alias()` is a pure call: it is deterministic and has no observable
  effect, so a `pure` transform can capture a cell handle and store it,
  which is all a reference cycle needs. `aliasing.md` and
  `core-language-probes.md` §12 both proposed `update(pure fn …)` partly to
  prevent exactly that, and `examples/Interpreter` FINDING 8 falsified it.
  **The definition above reproduces that result rather than being patched
  to agree with it**, which is the best evidence available that it is the
  right definition.
- **Not termination.** A `pure fn` may loop forever. That is `macros.md`'s
  open "fixed-point loop termination" seen from the other side, and it
  means the compile-time evaluator needs a step budget rather than a proof.

### `pure` and `realtime` are not nested — a correction

The `@abi(.C, realtime)` section above says:

> It is `pure fn`'s shape, not a new effect system. `pure` already
> propagates through calls and already bans external calls … The distance
> between pure and real-time-safe is one further property: **allocation**.

The first sentence is right and the second is wrong. A real-time body is
not a pure body plus a rule — `examples/Playhead`'s `renderAudio` calls
`fluid_synth_write_s16`, which is an `@external` function, and reads a
`Confined<PlaybackState>` through an `Opaque`. It is maximally **impure**,
and it must be: calling C is the entire job.

The two properties are different closed sets over the same mechanism, and
neither contains the other:

| | `pure` | `realtime` |
|---|---|---|
| an `@external` call | **forbidden** | **allowed** — it is the point |
| allocation | allowed | **forbidden** |
| reading a mutable cell | **forbidden** | allowed, through `Opaque` |
| a lock, a blocking call, an unbounded CAS retry | forbidden (no channels, no locks reachable) | **forbidden** |
| a refcount decrement that can reach zero | allowed | **forbidden** |

What the original sentence was reaching for is the shape: a declared bit,
a closed leaf set, call-wise propagation. That is shared. The contents are
not, and reading them as nested would have made `renderAudio` unwritable.

**So `realtime` is a function modifier too.** `@abi(.C, realtime)` is sugar
for "this `@abi(.C)` function is also `realtime`", and a helper reachable
from one is spelled `realtime fn`. Without that there is nothing to
propagate to and the guarantee stops at the first call — which is exactly
where the corpus is today: `renderAudio` is marked, and
`PlaybackState.fireEventsFor`, which it calls on the audio thread, carries
no marker at all and is therefore unchecked. One corpus migration, and it
is the one that makes the check mean anything.

### Tested against the corpus, and it needed one change

Every `pure fn` in the corpus is legal under this definition unchanged —
`Expenses`' four aggregations (local `var`s, a `Map`, no cells),
`Interpreter`'s `apply`, `Playhead`'s three clock methods, `parseAmount`.
The only migration the whole design required is the one marker on
`PlaybackState.fireEventsFor`.

Two of them are better evidence than that count suggests, because they
were written before the rule existed and had to obey it anyway.
`Playhead`'s `estimate` is `pure` and takes `reading` and `transport` as
**values sampled by the caller** — FINDING 8's resolution pushed the cell
reads out of the function to make two readings simultaneous, which is
exactly the line this rule draws, reached for an unrelated reason. And
`Expenses`' `byCategory` is `pure` over a `Map`, which is only
deterministic because `collections.md` ordered `Map` for a different
argument again.

### Determinism is a joint property, not a consequence

`strings.md` requires that a compile-time macro *"emit identical code every
build across all three targets"*, and `collections.md` ordered `Map` to
keep that true. Purity does not supply it on its own: it gives
same-input-same-output, and hash order is not an input anybody wrote down.

So determinism is **purity plus the decisions that make the inputs total** —
an insertion-ordered `Map` is one, and the target being an explicit value
rather than ambient is another (a macro reading `target` is deterministic
*given* the target, which is what conditional compilation needs). Worth
naming because it is easy to cite `pure` for a guarantee that two other
decisions are actually carrying.

## Open

- **Collection combinators on `Array<T>`** — `map`, `filter`,
  `reduce`, `groupBy`. The corpus's stated gap, and stdlib design
  rather than language design.
- ~~**Function values.**~~ **Resolved**, and this bullet outlived its
  subject — "Function values" above answers all three parts (the type
  spelling is `fn(T1, T2) -> R`, captures are governed by `Copy`/`Move`,
  and a bound method is not a value), and `core-language-probes.md` §7
  marks the probe gap resolved. What is still open from that round is
  narrower and lives there: a capture that outlives its frame, and one
  value captured by two closures at once.
- **Whether a callback's thread affinity may be declared.** `@abi(.C)`
  bodies are checked for an unknown thread unconditionally, because the C
  ABI does not say. The alternative is for the `@external` declaration
  *taking* the callback to state it, which is the already-open
  "main-thread affinity as a property of a call" question arriving from a
  second direction.
