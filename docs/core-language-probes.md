# Core language probes

Three programs written to put pressure on the parts of elp the original
corpus never exercised. They are **design probes, not specimens** — they
use constructs the language does not have, written in the most plausible
spelling, so the gaps can be read rather than listed.

| probe | what it is | what it stresses |
|---|---|---|
| `examples/TextSearch` | Boyer-Moore-Horspool substring search | indexing, integer arithmetic, comparison and logical operators, conditional loops, `Map`, `char` |
| `examples/LinkShortener` | an HTTP server with a channel-driven store | parallel match, string manipulation, concurrency off the UI thread, a non-client target |
| `examples/Expenses` | an app that aggregates rather than fetches | grouping, sorting, percentage arithmetic, optional narrowing, reactivity over computed data |

None of them parse today, which is expected — the corpus parse test
treats non-parsing as the default and they are not in `expect_success`.

## Why these exist

Every design decision so far has been driven by corpus evidence, and
that method found the real errors: the `Input(value = name)` bug, the
inheritance/defaults conflict, frequency-as-prelude-criterion. The
original corpus cannot drive the expression language, because it barely
computes — across 24 files there are eight `!=` (all of them `!= nil`),
one `+=`, zero `==`, zero `&&`/`||`, zero comparisons, and three
collection calls.

These three put that right before the decisions get made, not after.

---

## 1. ~~Operators~~ — resolved

Designed in `operators.md`. Operators are interface methods (`a + b` is
`a.add(b)`), the set is fixed with no user-defined operators, `&&`/`||`/
`!`/`??` are non-overloadable fixed forms because they short-circuit,
there is no `===`, and narrowing through `&&`/`||` is sound because
cells are thread-confined and a turn is not preemptible.

`Expenses` has been updated to show it: `Money implements Copy + Add +
Equatable`, and `sum = sum + expense.amount` where it previously read
`sum = sum.plus(expense.amount)`.

`compareTo` now returns a three-valued `Ordering` enum rather than an
`int`, and `Money implements Comparable` is the corpus's first example
of it — `expense.amount > biggest.amount` in `stats.largest`, and a
descending comparator in `stats.breakdown`.

Correcting that also found that `Comparable: Equatable + Hashable` in
`interfaces.md` was a syntax demonstration being read as a stdlib
specification: ordering and hashing are orthogonal, and `Hashable` had
no definition anywhere, so `Comparable` was unimplementable as written.
Now `Comparable: Equatable`.

Still open there: `a[i] = v` as an assignable index, and the numeric
tower.

**`Hashable` still has no definition**, and `Map` needs one — see the
collections section below.

## 2. Conditional loops

`grammar-script-notes.md` has this open as gap #2. **Boyer-Moore settles
whether it is needed**: the outer loop advances `offset` by a computed
skip distance, not by one, so no counted `for` expresses it.

```
while offset <= h - n {
	...
	offset += self.skipFor(text[offset + n - 1])
}
```

Also unresolved around loops:

- `for i in 0..n - 1` uses a range as an *iterable expression*. The
  corpus only ever uses `..` as a **match pattern** (`400..499`). Are
  they the same construct?
- `for category, amount in byCategory(expenses)` destructures a two-value
  iteration. Does `for` take multiple bindings, and does iterating a
  `Map` yield pairs?
- `break` and `continue` appear nowhere, in the old corpus or the new.
  Both probes were writable without them, which is weak evidence they
  can wait.

## 3. ~~Collections~~ — resolved

Designed in `collections.md`. `Hashable: Equatable` now has a
definition (it takes a seeded `Hasher` rather than returning an `int`,
so hash flooding is defensible on a server target), `Map` is
insertion-ordered so compile-time macros emit deterministic code,
`[]` traps while `get` returns `Optional`, and the iteration protocol
covers arrays, `Map`, ranges and channels alike.

Two things the probes forced that were not on the list:

- **`&var self`** — `map.set(k, v)` mutates its receiver and elp had no
  spelling for a mutating method. `memory-model.md` only established
  that *component* methods never need one. Interfaces force it to be
  explicit, since `Iterator.next` has no body to infer from.
- **`_` is the only catch-all.** The corpus spelled it `..` twice and
  `_` three times, and `..` actively conflicts with range syntax. Both
  `..` sites migrated.

`Iterable`'s accidental existential is resolved too — `dispatch.md`
adds associated types, so the iterator's type is named in the contract.
Still open: `a[i] = v`, and whether two-binding `for` generalises past
`Entry`.

## 4. Strings and `char` — resolved

**`Char` was a type the language did not have**, and `TextSearch` could
not be written without deciding what it is: a byte, a Unicode scalar, or
a grapheme cluster.

**Resolved** by `strings.md`: `char` is one Unicode scalar value,
`string` is an immutable sequence of them with a guaranteed UTF-8
representation, and `==` is scalar-wise with no implicit normalization.
The deciding argument was determinism across the three targets, which is
the same one that made `Map` insertion-ordered — grapheme segmentation
and normalization tables are Unicode-version-dependent and the targets
ship different Unicode versions.

One finding worth keeping: **this probe never indexes a string.** It
calls `haystack.chars()` and indexes the resulting `[char]`, so the
hardest question in the area — whether `s[i]` exists — turned out to
have no corpus demand at all.

Methods the probes assume: `chars`, `length`, `slice`, `split`, `trim`,
`startsWith`, `contains`, `replace`, `padStart`, `padEnd`, `toInt`.
These are stdlib surface, not language design.

Interpolation already exists in the old corpus, but only for a bare
identifier (`"{id}"`). The probes interpolate **expressions**:

```
"{row.total.toString()}  {row.share}%"
```

so the grammar inside `{}` needs settling, along with how a literal `{`
or a trailing `%` escapes.

## 5. Optional ergonomics

The highest-value area, since 8 of 8 comparisons in the old corpus are
nil checks.

### Matching an optional

The probes write:

```
match links.get(code) {
	nil  -> reply.send(Err(StoreError.NOT_FOUND(code)))
	link -> { ... }
}
```

That assumes a bare identifier arm binds the non-nil case. It reads well
and it is not obviously sound — it collides with matching on a plain
value, and there is no precedent for it in the corpus, which only ever
matches `Ok`/`Err`.

### Flow narrowing is load-bearing

`stats.largest` is the clearest case:

```
if biggest == nil || expense.amount.cents > biggest.amount.cents {
```

For the right-hand side to typecheck, the compiler must know `biggest`
is non-nil *because* the left-hand side was false. Same again in
`entry-form.velp`, where `if err != nil { Text(err) }` passes an
`Optional<string>` where a `string` is wanted.

**Narrowing through `||` and `&&` is a real commitment**, and it is the
thing that decides whether `?.` and `??` are conveniences or the only
way to touch an optional. Worth deciding deliberately rather than
discovering during implementation.

## 6. Tuples — resolved

`LinkShortener/main.elp` routes by matching method and path together:

```
match (req.method, req.path) {
	(.GET,  "/")      -> Response.text(.OK, "link shortener")
	(.POST, "/links") -> createLink(req, store)
	(.GET,  path)     -> followLink(path, store)
	_                 -> ...
}
```

and `entry-form.velp` matches a pair of parse results the same way.
Without the shape the route table becomes a nested match on method with
a path match inside each arm, which triples the line count and buries
the thing a reader came to find.

**Resolved** by `patterns.md`, and the answer is narrower than the gap
suggested: the language gains **no tuple type**. `(a, b)` is a `match`
scrutinee form only — not a value, not returnable, not nameable. Both
corpus sites are exactly that, so nothing enters the type system. The
same document rules that destructuring is named rather than positional,
which migrates `stats.elp`'s `for category, amount in ...`.

## 7. Function values

Used heavily, spelled by analogy with the corpus's `fn { dispatch(...) }`:

```
Server.listen(PORT, fn (req) { route(req, store) })
Picker(&category, options = Category.all(), label = fn (c) { c.label() })
```

Two open points:

- **Parameter types are inferred** in every lambda above. `generics.md`
  ratified that *type arguments* are always mandatory; whether that
  extends to lambda parameters is a separate question, and writing them
  out would be heavy at these call sites.
- **Function types as parameters**: `build fn(Channel<T>) -> StoreMsg`
  in `store.elp`. The corpus has `PureFn`/`ImpureFn` as the types of
  functions but no spelling for a *specific* signature.
- `return` inside a lambda — `.SHUTDOWN -> return nil` sits inside
  `background(fn { ... })`. Does it return from the lambda or the
  enclosing function?

## 8. Findings that are not about expressions

These surfaced while writing the probes and belong to other docs.

### A hyphen in a module path does not lex

`main.velp` contains `import { EntryForm } from .components.entry-form`,
for the file `components/entry-form.velp`. That tokenises as
`entry - form`.

The old corpus has `login-form.velp` and **never imports it**, so the
collision has been latent the whole time. Either hyphens are banned in
file names, or a module path segment is lexed differently from an
identifier. `name-resolution.md` says a path segment is always a file;
it does not say what a file may be called.

### The server has no UI thread

`reactivity.md` confines cells to one logical thread, justified by that
thread being the platform UI thread on all three targets. A server has
no UI thread, so the confinement has nothing to attach to.
`store.elp` sidesteps this with a channel-driven actor rather than
answering it — one task owns the `Map` and everyone else sends messages.

### ~~`.Server` is not a target~~ — resolved

`targets` was conflating where a package runs with what it produces. A
`kind` key (`.App` by default, `.Binary`, `.Library`) splits them;
recorded in `name-resolution.md`. `LinkShortener` is now
`kind = .Binary, targets = [.macOS, .Linux]`, and `TextSearch` is
`kind = .Library` with no `targets` at all.

### ~~Mutating a value taken out of a collection~~ — resolved

A value read out of a collection is a **copy**, and the write-back is
required and explicit. Recorded in `memory-model.md`.

The verbosity is accepted on purpose: a cheaper spelling can be built in
elp later, whereas a rule that hands out hidden aliases cannot be walked
back.

### Two bugs the probes found in the *old* corpus

Both now fixed, and both followed from rules already ratified rather
than from anything new.

**A blocking fetch in a component body.**
`SocialFeed/screens/home.velp` called `loadFeed()` as a bare statement
in the component body. Under Model B the body runs once at mount, so it
was not a render loop — it was worse. The call blocks the cell thread for
the whole fetch, and `loading` goes back to `false` before the first
render, which makes the `Loading(spin = true)` branch below it
**unreachable**. Wrapped in `effect`, which is what a component-scoped
coroutine is for.

**Enum shorthand missing in value position.**
`contentType = Multipart` (SocialFeed) and `contentType = JSON`
(BlogStyle) named bare identifiers that are neither imported nor
package-rooted. They are enum variants, so they take the leading dot the
corpus uses everywhere else: `.Multipart`, `.JSON`.

### `JSON` in type position is still unimported

`@CodeGen(Codec<JSON>)` appears in four files and `JSON` is imported in
none of them. Unlike the value position above, this one cannot be fixed
yet: `JSON` is a *type* here, so under the atmospheric criterion it needs
an import, and which stdlib module exports it is part of the undecided
stdlib layout.

### ~~`Codec` is imported twice, from two modules~~ — resolved

One namespace holds every importable name, and collisions are fixed with
`as` regardless of what kind of thing collides. Recorded in
`name-resolution.md`.

Applying it exposed the actual bug: `@CodeGen` was invoked nine times
and never imported, while the interfaces passed to it were imported from
`elp.macros`. Fixed in four files.

### The corpus promises a type switch that does not exist

`BlogStyle/internal/api.elp` comments that a function returning
`Either<Content, Error>` can be matched "at the call site … to work out
whether it was an ApiError or a SerializationError". Matching an
interface-typed value by its **concrete type** is a distinct construct
from matching an enum's variants, and the language has neither the
syntax nor a decision about it. It is also exactly where a mandatory
catch-all applies, since implementors of an interface cannot be
enumerated.

### Stdlib module layout, again

The probes invent `elp.http`, `elp.storage`, `elp.concurrent` and
`elp.ui.font`. This is the same gap `name-resolution.md` already records
(four roots for `Text` in the old corpus), now with four more guesses on
top of it.

---

## Suggested order

1. **Operators**, including `==`/`===` and whether they are
   interface-driven. Everything below references it and `generics.md`
   already owes an answer.
2. **Optional narrowing**, because it decides how much the rest of the
   language has to work around nil.
3. **Collections** — `Map`, indexing and bounds, iteration protocol
   (already open in `reactivity.md`), value semantics vs `.copy()`.
4. ~~**Tuples**~~ — **done** (`patterns.md`), and the answer was that
   there is no tuple type.
5. ~~**`Char` and strings**~~ — **done** (`strings.md`).
6. ~~**Piping**~~ — **done** (`functions.md`), and the answer was no
   `|>`. In a method-oriented language it is a second spelling for
   method chaining, and the corpus's free functions are parallel rather
   than composed. Extension methods on foreign types are rejected with
   it.
