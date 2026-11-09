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

## Open

- **Collection combinators on `Array<T>`** — `map`, `filter`,
  `reduce`, `groupBy`. The corpus's stated gap, and stdlib design
  rather than language design.
- **Function values.** `*fn` callback props exist (`memory-model.md`,
  `dispatch.md`), and closures appear as
  `rows.sorted(fn (a, b) { ... })`, but the type spelling, capture
  rules and whether a method reference is a value are undesigned.
  Probe gap #7.
