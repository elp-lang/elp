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

## Open

- **Collection combinators on `Array<T>`** — `map`, `filter`,
  `reduce`, `groupBy`. The corpus's stated gap, and stdlib design
  rather than language design.
- **Function values.** `*fn` callback props exist (`memory-model.md`,
  `dispatch.md`), and closures appear as
  `rows.sorted(fn (a, b) { ... })`, but the type spelling, capture
  rules and whether a method reference is a value are undesigned.
  Probe gap #7.
