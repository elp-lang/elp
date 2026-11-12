# Core language probes

Thirteen programs written to put pressure on the parts of elp the original
corpus never exercised. They are **design probes, not specimens** — they
use constructs the language does not have, written in the most plausible
spelling, so the gaps can be read rather than listed.

| probe | what it is | what it stresses |
|---|---|---|
| `examples/TextSearch` | Boyer-Moore-Horspool substring search | indexing, integer arithmetic, comparison and logical operators, conditional loops, `Map`, `char` |
| `examples/LinkShortener` | an HTTP server with a channel-driven store | parallel match, string manipulation, concurrency off the UI thread, a non-client target |
| `examples/Expenses` | an app that aggregates rather than fetches | grouping, sorting, percentage arithmetic, optional narrowing, reactivity over computed data |
| `examples/QuadTree` | a spatial index over a recursive tree | **recursive value types**, mutable paths into an interior node, matching through a reference, multi-line expressions |
| `examples/Undo` | a notes editor with an undo stack of closures | function-typed fields, escaping captures, one value captured twice, recursive closures, value-producing match arms |
| `examples/Worker` | a job pool with graceful shutdown | `break`, channel closure, abandoned in-flight requests, cancellation during mutation, multi-consumer channels |
| `examples/Buffer` | a split-pane editor: shared text, per-pane cursors | **proved `Copy` meant two incompatible things**; now migrated onto `Confined<T>`/`.alias()` so it demonstrates the fix, with whether reading a handle in markup subscribes left open |
| `examples/Sheet` | a spreadsheet with shared cell styles | field-wise `Copy` over four levels; mixed duplication ("copy the values, share the styles") is a domain operation and **a derive cannot close it** |
| `examples/Pipeline` | a static-site build fanning out to background workers | the **immutable** half: `Shared<T>` as `Copy` for any `T` and `Sendable` when `T` is, `Synchronized<T>`'s `update`/`with` surface, and a `Confined` refused at a thread boundary |
| `examples/SqliteBinding` | a library binding to libsqlite3 | the **FFI boundary**: an opaque handle with no declarable duplication semantics, a thread discipline that is a runtime value, no exactly-once release, borrows invalidated by unrelated calls |

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

These put that right before the decisions get made, not after.

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

## 7. Function values — resolved

Used heavily, spelled by analogy with the corpus's `fn { dispatch(...) }`:

```
Server.listen(PORT, fn (req) { route(req, store) })
Picker(&category, options = Category.all(), label = fn (c) { c.label() })
```

All three open points are settled in `functions.md`:

- **Parameter types are inferred** from the expected type. That is not
  the pathology mandatory type arguments were adopted to avoid: there is
  exactly one expected function type at the position, so it is
  propagation rather than constraint solving.
- **The type spelling is `fn(T1, T2) -> R`**, with no parameter names —
  a function value has no overload set, so a label would have nothing to
  disambiguate.
- **`return` in a closure returns from the closure**, as in Swift and
  Rust, not from the enclosing function as in Ruby. And `return` is now
  required in a closure that produces a value, which migrated nine
  corpus sites.

Two further decisions came out of it: a `*fn` callback prop is
**required** (an optional one is an ordinary prop with an
`Optional<fn(...)>` type, which migrated `login-form.velp`), and a bound
method reference is not a value — `fn (x) { return money.add(x) }` makes
the capture visible where `money.add` would hide it.

Still open, both from `examples/Undo`: a capture that outlives its frame,
and one value captured by two closures at once.

## 7b. The second round of probes

The first three probes were written because the original corpus could not
drive *expression* design. These three were written because it could not
drive **data shape, lifetime or shutdown** design, and they were chosen
over a suggested list that mixed design probes with compiler-robustness
tests. Malformed syntax, recursive macro expansion and deliberately
deep generics all target a compiler that does not exist; they belong in
`crates/parser/test_projects/`, and the corpus parse test actively
requires an `examples/` file to *fail* parsing, so they would assert
nothing here.

### Recursive value types have no spelling

The largest hole any probe has found, and it was not on the suggested
list.

```elp
export enum QuadNode<T> {
	LEAF { entries [Entry<T>] },
	BRANCH { children [QuadNode<T>] },
}
```

A `BRANCH` contains four `QuadNode`s, each of which may contain four
more, so as a value type it has no size and cannot be laid out. Every
value-semantics language needs a marker for "this field is behind an
indirection" — Rust's `Box<Node>`, C++'s pointer — or sidesteps it by
making the recursive case a reference type, which is Swift's `class`.

`memory-model.md` has `Copy`, `Move`, `Protected` and `Binding<T>` and
not one of them says "boxed". This is not a corner case: every tree,
linked list, AST and JSON value is this shape, and `BlogStyle` already
wants a `JSON` type.

### Mutable paths into an interior node

`collections.md` introduced `&var self` for a mutating receiver, always
for the *whole* receiver. `QuadTree.insert` holds `&var self` and then
needs `&var` to a node inside `self.root`:

```elp
return insertInto<T>(&var self.root, &self.bounds, at, value)
```

Nothing says whether `&var self.root` exists, what stops two such paths
being live at once, or what `&var` to a local (`&var found` in `query`)
and a re-borrow of a `&var` parameter mean. Cell-thread confinement does
not help: it rules out *interleaving*, and this is straight-line
aliasing inside one turn.

### Matching through a reference

Every corpus match so far scrutinised an owned value. `insertInto`
matches `node &var QuadNode<T>`, so `entries` in the LEAF arm has to
bind a *borrow* — otherwise the match copies the payload and the
`entries.append(...)` below it is lost. `patterns.md` settled how a
record pattern binds names and said nothing about ownership.

### Captures that outlive their frame

Every closure in the first three probes was called immediately or lived
exactly as long as its component. `Undo`'s do not:

```elp
export fn Edit.insert(at int, line string) -> Self {
	return Self {
		apply  = fn (doc) { doc.lines.insert(at, line) },
		revert = fn (doc) { doc.lines.remove(at) },
	}
}
```

`at` and `line` are parameters of a function that has already returned.
And `Edit.composite` captures one `[Edit]` in **two** closures at once —
a closure is not `Copy`, so neither is `[Edit]`, so capturing it is a
move and the same value cannot move twice. Either non-`Copy` captures
are implicitly shared, which contradicts how explicit the memory model
is everywhere else, or that function is illegal.

### A recursive closure cannot name itself

```elp
const retry = fn {
	if !body() && remaining > 0 {
		remaining -= 1
		retry()
	}
}
```

`retry` is not bound until the literal finishes evaluating, so the body
cannot see it. Rust refuses this for closures outright; JavaScript
allows it because the binding is in scope by the time the body runs.
Whichever way elp goes, the rule has to interact with `const` meaning
immutable.

### There is no `break`

The corpus contains `while` and `for` and **not one** `break` or
`continue`. `Worker`'s shutdown arm has to stop its message loop, and
the only exit available is `return` from the enclosing closure — which
is simultaneously the loop exit and the coroutine's end. It works only
because the loop happens to be the closure's last statement, and it
rules out the drain-then-exit phase that GAP 5 below needs.

### Channel shutdown is entirely undesigned

`LinkShortener` declares a `SHUTDOWN` variant and never sends one, so
nothing was ever forced. `Worker` sends it, and four questions fall out
in the ten lines after it arrives:

- **What does `send` to a closed channel do** — error, trap, or drop? A
  worker mid-job will report `COMPLETED` after the supervisor has gone.
- **What does `send` to a *full* channel do** — block or fail? `main`
  behaves very differently under each.
- **In-flight requests are abandoned.** A caller blocked in
  `reply.receive()` whose `SUBMIT` sits behind the `SHUTDOWN` is never
  answered. `PoolError.ABANDONED` exists for them and there is no way to
  send it, because the only code that could is the loop that just
  stopped. A reply channel with no writer left is a **deadlock, not an
  error**, and nothing in the signature says so.
- **Is `Channel<T>` multi-consumer?** Four workers iterate one `work`
  channel and each message must reach exactly one of them. That is a
  different guarantee from broadcast and `LinkShortener`'s single
  consumer never had to state it.

### Cancellation during mutation, on a target with no cell thread

`Worker`'s shutdown arm marks every queued job `CANCELLED` while a
worker may be concurrently finishing one of them and about to report
`DONE` for the same id. On the three client targets `reactivity.md`'s
cell-thread confinement rules that out; `Worker` is `kind = .Binary`
with no UI thread, so the guarantee does not exist and this is a real
race on `states`.

This is the standing open question *"the server has no UI thread for
cell confinement"* arriving as a concrete bug rather than a note.

### Two questions the probes raise about existing decisions

**Does a `{ }` match arm produce a value?** `LinkShortener`'s router has
expression arms; `store.elp` has block arms used purely for effect.
`History.undo` needs a block arm to *produce a bool*. That is the same
question as implicit return in a closure body, and the two should be
answered together rather than separately.

**May an expression span a line break?** The lexer emits a `Newline`
token, so this is significant:

```elp
return p.x >= self.minX && p.x < self.maxX
	&& p.y >= self.minY && p.y < self.maxY
```

Python continues inside brackets, Swift and Kotlin continue when the
line cannot yet be a complete statement, Go inserts semicolons and
forbids it. Under Go's rule `QuadTree/types/geometry.elp` does not
compile.

## 8. Findings that are not about expressions

These surfaced while writing the probes and belong to other docs.

### ~~A hyphen in a module path does not lex~~ — resolved

`main.velp` contains `import { EntryForm } from .components.entry-form`,
for the file `components/entry-form.velp`. That tokenises as
`entry - form`.

The old corpus has `login-form.velp` and **never imports it**, so the
collision was latent the whole time.
`examples/VulkanTriangle/internal/vk-types.elp` is the second witness and
the worse one: three files import it, so the path is load-bearing rather
than decorative.

**Resolved** in `name-resolution.md`: a path segment may contain internal
hyphens. It needs no lexer mode, because the thing after `from` is a path
and never an expression, so the path parser accepts `Ident (Minus Ident)*`
as one adjacent segment and `-` keeps meaning subtraction everywhere it
could. And it is not an exception to the no-magic rule — that doc already
said a path segment *is* a file, so a segment was never an identifier.
`examples/BlogStyle`'s `[postId].velp` makes the same point harder, and
the corpus shows the workaround people reach for without the rule:
`components/feeditem.velp`, spelling `FeedItem` run together because no
separator worked.

Rejected: mapping a hyphenated name to `.entryForm` *or* `.entry_form`
automatically, because two spellings for one file means two module paths
and one-file-per-module-path is what keys the no-cycles graph. Picking one
still breaks `grep` both ways and is not injective.

What a filename may contain *otherwise* is still open, and is a routing
question rather than a lexing one.

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

## 9. Aliasing — resolved, and the two probes that closed it

Designed in `aliasing.md`. `examples/Buffer` proved that `Copy` meant
two incompatible things; `examples/Sheet` tested the fix by depth;
`examples/Pipeline` and `examples/SqliteBinding` were written afterwards,
because the decision had **two outcomes with no corpus witness at all**.

### Why two more probes rather than none

`Buffer` and `Sheet` both share *mutably*, so both migrated onto
`Confined<T>` and left `Shared<T>` — the one type in `aliasing.md`
carrying a conditional conformance that has to be sound — with zero
users. A type nothing uses is a type nothing has tested, and the claim
being made about it ("`Sendable` when `T` is") is the claim the whole
concurrency story rests on.

And `unsafe` being FFI-only is what makes everything else in
`aliasing.md` *inferable*: mutable aliasing is obtainable nowhere else,
so field-wise conformance always sees it. That assumption is load-bearing
and had no probe either.

### What `examples/Pipeline` found

A `kind = .Binary` build pipeline on macOS and Linux, so there is no
cell thread and every rule that leans on confinement has to say what it
does.

1. **`Shared<T>` makes its holder *more* duplicable, not less.**
   `Config` is `Move` (it holds a `[string]`), yet `Settings` holding a
   `Shared<Config>` is `Copy`. That is the one case a handle improves
   the holder's tier, and it is the reason `Shared<T>` has to be a
   separate type rather than a mode of `Confined<T>`.
2. **`.get()` returns `&T`, and nobody had said so.** Written `-> Config`
   first, which cannot work — it would move the referent out of the cell
   and leave every other holder pointing at nothing. The borrow's
   *lifetime* is still open, and it has to answer to the same "a call
   ends the turn" rule as narrowing.
3. **`Synchronized<T>`'s surface is two closures, not one.**
   `aliasing.md` proposed `.with(fn (v &var T) { ... })`, which needs a
   `&var T` parameter spelling that `memory-model.md` Finding 4
   deliberately rejected, plus whole-value assignment through a
   reference, which is `QuadTree`'s GAP 4. Splitting it by direction —
   `update(fn (T) -> T)` and `with(fn (&T) -> R) -> R` — needs neither,
   cannot forget to write back, and makes the critical section total.
4. **`background` has no join.** The fan-out loop never collects anything
   to wait on, so `snapshot()` reads before the workers finish.
   `reactivity.md` says `background` blocks its caller, which works for
   one call and not for a loop of them. `Worker` reaches this wall from
   the channel side; this reaches it from the `background` side.

### What `examples/SqliteBinding` found

The FFI boundary, and the first thing in the corpus that `aliasing.md`
cannot answer **even in principle**.

1. ~~**An opaque handle has no declarable duplication semantics.**~~
   **Resolved**, and the finding was mis-diagnosed. It said field-wise
   `Copy` had "no fields to be wise about" — but `db.elp` held the handle
   as `&Sqlite3`, and `memory-model.md` Finding 6 makes `&T`
   unconditionally `Copy`. The rule looked, applied, and said *yes*:
   `Connection` was `Copy`-eligible, and `Connection implements Move` was
   the author declining a `Copy` the compiler would have granted. A
   soundness bug in a decided rule, not a gap in an undesigned corner.

   `aliasing.md` closes it in three parts, with no new vocabulary: a field
   may not hold a borrow (the corpus had exactly two, both these handles);
   a foreign handle is a value rather than a borrow, since nothing in the
   program owns the pointee; and an `@external type` must declare its tier,
   because zero fields makes field-wise *vacuously* satisfied and the
   derivation's answer is `Copy`. Then `Connection implements Move` is a
   conclusion, exactly as `Style implements Move` is.

   Two of the three questions the probe asked turned out to need no syntax:
   duplication **is** the conformance, and thread-crossing **is**
   `Sendable`. That retired a strawman putting both in `@external`'s
   arguments as `duplication = .aliasing, threads = .confined` — a parallel
   vocabulary for what `Move`/`Copy`/`Duplicate` already say.

   And one thing it forced into the open: **`sqlite3*` is the easy case.**
   GLFW and Vulkan hand back handles they own and never take back —
   `VkQueue`, `VkPhysicalDevice`, `GLFWmonitor*` — for which `Copy` is the
   right tier, not `Move`. Same rule, opposite answer, which is the
   strongest argument for a mandatory declaration over a default. What
   those handles do *not* get from a tier is **validity**: a `VkQueue` is
   alive only while its `VkDevice` is, and Vulkan's handles form a strict
   ownership tree where every edge is a constraint elp cannot express. That
   is finding 4 below, far larger than one `Statement`/`Connection` pair
   made it look.
2. **The thread discipline is a runtime value, and
   `Confined`/`Synchronized` is a static distinction.**
   `sqlite3_threadsafe()` reports which of three modes libsqlite3 was
   built with, and the answer is not knowable when elp compiles. Nothing
   in `aliasing.md`, `reactivity.md` or `dispatch.md` covers a
   capability that is real but not statically known. **This is new**, and
   it is the sharpest finding in either probe.
3. **`Move` gives at-most-once release, never at-least-once.** A dropped
   `Connection` leaks the C allocation and nothing notices. elp has no
   destructors, no `defer`, nothing at scope exit. The smallest candidate
   is a scoped form (`with Connection.open(path) as db { ... }`) — and it
   is worth noting that `Synchronized<T>`'s `update`/`with` are the same
   shape, a resource borrowed for one block, arrived at independently.
4. **A borrow invalidated by an unrelated call cannot be typed.**
   `sqlite3_column_text` returns a pointer killed by the next `step` on
   the same statement. Finding 4 infers borrows from usage *inside one
   call* and cannot express a constraint spanning two. Rust needs a
   hand-written lifetime; elp has no lifetime parameters, which also
   means `Statement` cannot be tied to its `Connection`.
5. **A C callback cannot be an elp closure**, because elp closures
   capture and a capturing closure has no C representation — which is why
   the C API carries a `void*` for state in the first place.

   **Followed up, and it was a bigger hole than it looked.** Asking what a
   playback callback writing a caret position would take — FluidSynth
   shaped, a C library calling *into* elp on its own audio thread — found
   that elp could not receive a C callback **at all**: no outbound calling
   convention, and no way to hand C a pointer to elp state. Both closed in
   `functions.md` ("A C function pointer, in both directions") for less
   than expected: inside an `@external` block `fn(A, B) -> R` *is* a C
   function pointer, because that block is already a context that re-reads
   the types in it, and only the definition side needed a marker,
   `@abi(.C)`. The closure restriction above then stops being a rule and
   becomes an observation — a C function pointer carries no environment,
   so there is nowhere for captures to live.

   The thread question needed nothing new either: **a foreign callback
   body is checked exactly as a `background` body is** (`aliasing.md`), so
   `Confined<T>` is refused and anything elp also reads must be
   `Synchronized<T>`. Unconditionally, because the ABI does not say which
   thread calls — which costs a synchronous comparator callback access to
   confined state, and connects to finding 7 of §10.

   What it exposed, and what closed next: `Synchronized<T>` is a mutex, and
   a real-time audio callback must not take a lock — so elp could write a
   *safe* audio callback and not yet a *correct* one. Closed by
   `aliasing.md`'s `Atomic<T>`, a third cell kind restricted to word-sized
   `T`, sequentially consistent with no orderings exposed, plus
   `@abi(.C, realtime)` in `functions.md` to make the guarantee mean
   something. **The boundary table did not change**, which is the test of
   whether `Opaque<T>` was drawn at the right level: it names how C sees a
   thing, the cell kind names the discipline, and the two stay orthogonal.

   Two hazards worth recording because they are not the obvious ones. A
   refcount decrement reaching zero **deallocates**, so a deadline thread
   must never hold the last handle to anything — which the no-retain rule
   on `Opaque<T>` already guaranteed, for a reason it was not chosen for.
   And a CAS retry loop is unbounded in the worst case, which is why
   `Atomic<T>` is load/store rather than a transform even though its
   siblings are transforms.

   Still open, and the one that cost the most time in practice: a
   single-producer/single-consumer ring buffer. Not the atomics — the
   **invalidations**, and the difficulty of seeing from the source which
   thread produces what.
6. ~~**A native dependency is not a package dependency.**~~ **Resolved**
   in `name-resolution.md`, "Native libraries": three questions, three
   places. `links` on `@Package` is an **allow-list** — the complete
   native surface a package may reach, in one reviewable file;
   `@external`'s arguments say which allowed library a symbol comes from;
   and `.native(...)` in `dependencies` stays for the other kind, a
   library the build actually acquires, which has the full version
   algebra a system library has none of.

   It also partly answers an older open item: a `.Library` normally takes
   no `targets`, but one with a `links` entry does, because `.Web` has no
   shared libraries — and those constraints propagate to consumers.

   One thing it opened and did not close. `minimumVersion` on a system
   dynamic library is **unenforceable at build time**, so it is a probe
   where one exists or a promise the code must verify — the same "declared
   statically, knowable only at runtime" shape as finding 2, which makes
   it a pattern.
7. **`CString` and `OpaqueHandle` are invented in the probe and belong in
   the stdlib.** `strings.md` makes `string` a sequence of Unicode scalar
   values with guaranteed UTF-8; a `const char *` is bytes with a
   terminator and no encoding guarantee, so conversion is fallible one way
   and allocating the other, and `strings.md`'s open byte-access question
   is on the critical path for any FFI at all. `OpaqueHandle` stands in
   for `void*` / `T**`, which is the thing the type system exists to
   prevent, needed in exactly one place. Finding 8 below turns this from
   tidy-up into a prerequisite.
8. ~~**Symbol collisions have no orphan rule.**~~ **Resolved**, by moving
   the uniqueness a level up. Two packages could each declare
   `sqlite3_open` with a different signature, each locally correct, and
   the linker would give both the same address. The orphan rule cannot
   reach it because **there is no contract on the far side of a shared
   library**: it works by naming an owner, and nobody owns a C symbol.

   So `name-resolution.md` makes a `links` entry **exclusive across a
   program** — the symbol has no owner, but a library has exactly one
   claimant — which is the orphan rule's shape moved from the type to the
   package, and is the rule Cargo reached under the same key name. A
   whole-program agreement check covers what exclusivity leaves, and elp
   can afford one because resolution is already global where Rust's is
   per-crate.

   It opened one of its own and sharpened another: the agreement check
   cannot compare *elp* types, because each package declares its own
   `CString` and `OpaqueHandle`, so either it compares ABI or those types
   must be stdlib's — which turns finding 7 from tidy-up into a
   prerequisite.
9. ~~**`library = "sqlite3"` fourteen times.**~~ **Resolved** by
   `macros.md`, and the resolution needed no new rule. A macro is not an
   entry in the program; it receives the AST of what follows it and
   generates the declarations the program contains. A declaration list is
   an AST node, so a macro may be handed one, and the attribute is written
   once.

   The probe had rejected grouping on the grounds that it would be a new
   declaration shape, which is what `@external` being an attribute was
   supposed to avoid. That was about the wrong thing: the grouping form is
   not a form of `@external`, it is what a macro's input may be — so no
   FFI content, and `@CodeGen(Copy) { ... }` gets it for the same reason.

   Two things the framing fixed that an intermediate draft got wrong. It
   had called the block "the language's one non-scoping brace" and
   invented rules against distributing `export` and against nested
   attributes merging. None of those exist: the braces are not in the
   program, so there is no scope to carve an exception for; the macro
   copies the `export` it was handed; and nesting is the macro's business,
   not the language's. It had also made the block a **desugaring** into N
   copies of the attribute, which gave up group-spanning codegen — three
   types plus an enum over them plus one dispatch table — for safety that
   a macro's own signature provides anyway.

   The general lesson: the repetition read as an argument for weakening
   the attribute's own rules (a conditional `library =`, which this
   language refuses everywhere else) and was really a question about what
   a macro may be handed.

Plus one smaller thing worth naming: a by-value receiver
(`fn Connection.close(self)`) is exactly what a consuming method needs,
and `functions.md` has only `&self` and `&var self`.

**And one correction this probe produced about itself.** Its first draft
invented an `extern` keyword. That was wrong twice: `@external(.OBJC)`
already exists (`BlogStyle/internal/api.elp`) with parser support for
body-less declarations, and `reactivity.md` rejects new lowercase
intrinsic keywords as policy — it is why `&mut` is not a spelling either.
Migrating to `@external` also dissolved a gap the draft had listed:
there is no question of where `unsafe` goes, because `@external` is the
marker and `links` bounds what it can reach.

---

## 10. System resources — `examples/VulkanTriangle`

A Vulkan triangle in a GLFW window, `kind = .Binary`. Written because
`examples/SqliteBinding` turned out to be the easy case: one library, one
handle you own, scalars across the boundary. This is two libraries that
reference each other's types, a five-deep handle tree with release
*ordering*, C structs built in elp and read by a driver, out-parameters
that must be called twice, and functions that do not exist until the
driver is asked for them by name.

### What it confirmed

**The owned/borrowed split in `aliasing.md` is real, and not a SQLite
quirk.** `window.elp` holds the two rows side by side: `Window` wrapping a
`GLFWwindow` is `Move`, `Monitor` wrapping a `GLFWmonitor` is `Copy`, and
neither file argues for its tier — both are conclusions from a boundary
declaration plus field-wise `Copy`. Vulkan then splits the same way inside
one API: `VkInstance`/`VkDevice`/`VkSwapchainKHR` are `Move`, and
`VkPhysicalDevice`/`VkQueue`/`VkCommandBuffer` are `Copy`. One rule,
opposite answers, in one file — which is the strongest argument yet that
the mandatory declaration beats a default in either direction.

**Exclusivity buys more than it was argued for.** `links` was made an
exclusive claim to stop two packages declaring one symbol differently.
`glfwCreateWindowSurface` takes a `VkInstance`, so the GLFW binding must
import the Vulkan binding — and exclusivity is what guarantees there is
exactly one `VkInstance` *type* in the program to import. Multi-library
FFI works because of a rule adopted for a different reason.

### What it resolved

1. **A `links` entry's first argument is a logical name.** Vulkan is
   `libvulkan.so.1` on Linux and `libMoltenVK.dylib` on macOS — same
   symbols, different file — so `condition` covered the manifest and broke
   the declarations, which refer to the library by name. `soname` is now
   separate and defaults to the logical name. The allow-list is keyed on
   the logical name, so two conditional entries are one claim.
2. **`scripts` on `@Package`** runs an external process at a build stage,
   with **mandatory** `inputs` and `outputs`. That is the deliberate
   divergence from Cargo's `build.rs`: declaring them makes a script a
   function of its inputs, so it is skippable, cacheable and parallel,
   rather than re-running on any change unless it opts out from inside
   itself. It is also where impurity is allowed to live, which is what lets
   `macros.md` keep compile-time elp `pure`.

### What it found that is new

3. **An `@external` type with *fields* needs a layout guarantee.** Every
   previous external type was opaque, which is what let `aliasing.md` say
   field-wise `Copy` is vacuous for them. A `VkInstanceCreateInfo` is
   filled in elp and read by the driver at a C ABI, so `object`'s freedom
   to reorder and pack fields — correct and valuable for elp-only types —
   is a crash here.
4. **`sType`/`pNext` is a heterogeneous intrusive list with no spelling.**
   Every Vulkan struct starts with a tag and a nullable pointer to a struct
   of a *different* type, each with its own tag and pointer. It is a tagged
   union threaded through pointers and assembled by the caller. elp's
   `enum` is the right shape and the wrong representation. **The one place
   this probe needs a new language feature rather than a new rule.**
5. **`Copy` and `Sendable` come apart, in the direction nothing has
   needed.** A `VkQueue` is `Copy` — correctly, it is a pointer you never
   release — and Vulkan queues are explicitly *not* thread-safe, so it
   crosses into `background` with no syntax and two threads submitting to
   one queue is undefined behaviour. Everywhere else in the corpus a `Copy`
   type is trivially `Sendable` because it is plain data. This needs a
   `Copy` type that declines `Sendable`, which is legal and unprecedented
   here.
6. **A field can outlive a sibling field.** `Renderer` is `Move`, so the
   object cannot be duplicated — and `renderer.graphics` is a `Copy`
   `VkQueue` that can be read out and copied anywhere, pointing at
   something owned by `renderer.device`. Field-wise `Copy` protects the
   aggregate and not the members. For `Confined<T>` that asymmetry is fine,
   because reading a handle out keeps the cell alive; a foreign handle
   keeps nothing alive. **The sharpest form of the validity gap in the
   corpus.**
7. **Main-thread affinity is a property of a call, not of a value.**
   `glfwPollEvents` must run on the macOS main thread or it aborts.
   `Sendable` answers *may this value cross a thread?*; the constraint here
   is *this function must run on one specific thread*. `reactivity.md` has
   thread affinity only for data, via the cell thread, which a `.Binary`
   does not have — so a `.Binary` on macOS has an affinity requirement the
   language cannot state. Swift's answer is `@MainActor`, already noted
   there as a comparison and declined.
8. ~~**The count-then-fill out-parameter needs a place.**~~ **Resolved**
   in `memory-model.md`, "`Buffer<T>` — contiguous storage": a type whose
   purpose is an addressable extent, distinct from `[T]` so that
   "collections hold values, not places" keeps its unqualified form. The
   answer turned out to be **the scalar rule at a second arity** rather
   than a new mechanism — `&Optional<T>` for one slot,
   `&Buffer<Optional<T>>` for n — leaning on a layout promise
   `vkCreateInstance` had already needed. Because length can only grow by
   supplying every element, there is no uninitialised memory and no
   operation that asserts there isn't, which matters here specifically
   because `unsafe` is an FFI-only *concept* with no block form and an
   assert-initialised operation would have nowhere to live.

   Two things it opened. The **nullable** first call is still unspellable,
   and that is the honest signal that one C symbol is two operations — so
   the fix is two declarations over one symbol (`symbol =` on
   `@external`), not a nullable borrow. And deciding where an address may
   exist at all produced a principle worth more than the feature:
   **C may hold the address of elp memory only where that memory is
   out-of-line and kept alive by something other than a stack frame.** Two
   things qualify — a `Buffer<T>`'s elements and a cell's contents — so
   "address of a local" is not missing, it is unsound here, and now
   something says why.
9. **Extension functions are loaded by name and called as values.**
   `vkGetInstanceProcAddr` returns a generic function pointer to cast.
   There is no symbol for the allow-list to check, and the result is a
   function value with a C ABI, which `functions.md` says nothing about —
   `fn(T) -> R` carries no calling convention. It is also not exotic: the
   debug messenger and `vkDestroySurfaceKHR` are both only reachable this
   way, so `renderer.elp` leaks its surface because two unspellable things
   compose.
10. **Teardown ordering is the specification and the types permit every
    permutation.** `vkDeviceWaitIdle` before everything, swapchain before
    device, device before instance, renderer before window across two
    files and two libraries. `Move` gives at-most-once and imposes no
    order. And every `throw` on the construction path leaks everything
    built before it — five handles deep, one leak site per step, which is
    C's `goto fail` ladder and what Rust's `Drop` handles on the error
    path.

### ~~The construct three unrelated probes have now asked for~~ — designed

A scoped form was reached for independently by `SqliteBinding`
(exactly-once release), `VulkanTriangle` (the teardown ladder *and* GLFW's
handle-less `init`/`terminate`), and noted in `aliasing.md` as the same
shape as `Synchronized<T>`'s `update`/`with`.

**Designed** in `memory-model.md`, "Scoped release": `with expr as x { }`,
with multiple bindings released in reverse, and a `Scoped` interface whose
`release(self)` the compiler calls at every exit. It is scoped release
rather than RAII — release is tied to a brace in the source, not to a
value's lifetime, so there are no drop flags, no moved-out analysis and no
drop glue, and a value not bound by a `with` is simply never released.

Three things it settled that were not the headline:

- **Reverse order answers the ordering gap across resources.**
  `VulkanTriangle`'s renderer is built *from* its window, so it is bound
  after it and released before it. That was two hand-written calls in one
  specific order, duplicated on an error path.
- **The handle-less global gets a handle.** `Glfw.initialised()` is a
  token type carrying no data, which exists so that "GLFW is initialised"
  is a value with a lifetime. Worth writing only because `Scoped` exists.
- **`Scoped` is the second and harder witness for a by-value receiver.**
  `release(self)` cannot be declared at all under `functions.md`'s
  `&self`/`&var self`, so the construct cannot be built without it.

And three it did not:

- **A partially-constructed aggregate on an error path.**
  `Renderer.new` assembles five handles *for a caller*, so there is no
  block to bound them and `with` does not apply. The remaining question is
  narrower than "elp has no destructors" — possibly assembly belongs in a
  `with` whose release is cancelled on success, and nothing proposes a
  way to cancel one.
- **Ordering *within* an aggregate.** Reverse order sequences `with`
  bindings relative to each other; the five lines inside
  `Renderer.release` are one binding and the types permit every
  permutation.
- **Borrowed handles derived from a released resource.**
  `Glfw.release` invalidates every `Monitor`, which are `Copy` and were
  never bound by a `with`. `aliasing.md`'s validity gap, seen from the
  release side.

### The `out` keyword was never a design — it was my placeholder

`db.elp` and `renderer.elp` wrote `&out` against a binding nothing
declared, then read `out.asSqlite3()`. Shorthand for "the thing C wrote
into" that should not have been committed, and it spread from one probe to
the other. (`QuadTree/tree.elp`'s `out` is unrelated and legitimate — an
ordinary declared parameter that happens to be named `out`.)

**Resolved** in `memory-model.md` Finding 4, and the resolution deletes
the concept rather than designing it. The difficulty was never mutability
direction, so `&mut` would have described the wrong axis: you cannot take
a mutable reference to a `Sqlite3` that does not exist yet. It is
**emptiness**, and `Optional<T>` is already a slot that may be empty —
niche-optimised to a nullable pointer, so `&db` *is* `sqlite3**`.

What made it legal needed one sentence: **Finding 4 infers mutability from
the callee's body, and at an `@external` declaration the callee is C.** So
at that boundary `&T` is read-write, conservatively, because there is
nothing to infer from. An `Out<T>` slot type was considered and dropped as
`&Optional<T>` with extra vocabulary — and because its closest relative,
`Binding<T>`, is a refcounted reactive cell tied to the cell thread, which
is machinery that has no business near `sqlite3_open`.

## 11. A real-time thread and a UI — `examples/Playhead`, held locally

The first probe **ported from a real application** rather than invented: the
playback and rendering loop of a released audio app driving a C synthesiser
from a platform audio callback.

**The entry is not in this repository.** It derives from closed-source
work, and this is a public repo, so `examples/Playhead/` is gitignored and
lives only on the author's machine. What is recorded here are the
*language* findings, which is the part the design needs; the provenance is
noted so a reader knows why the one probe that corrected three decisions
cannot be opened. Everything below is a general real-time engineering
pattern or an elp conclusion, not an implementation detail of that app.

Its value is that every finding is something that broke in production and
was fixed, rather than a hazard imagined at a whiteboard. It was written to
test the `Buffer<T>`, `Atomic<T>`, `Ring<T>`, `@abi(.C, realtime)` and
`Published<T>` decisions against a real implementation. Three of them it
corrects.

**What it corrected.**

1. **An age is a measure of staleness, not of elapsed time**, and
   `Published<T>`/`Aged<T>` as recorded invites the bug. Extrapolating a
   playhead from its age is right while audio flows and wrong the instant it
   stops: an interruption, an output route change or a stalled engine freezes
   the published position, and unbounded extrapolation then runs the line on
   through music nobody is hearing before snapping it back when sound
   resumes. The working fix caps the run-ahead at about two render buffers.
   Past a threshold an age stops meaning "this much has happened" and starts
   meaning "the producer stopped" — opposite instructions — so handing out a
   bare duration is a loaded gun.

   **Resolved** in `aliasing.md` under "All speed on the writer, all policy on
   the reader" — and the resolution revises this finding's own proposal. The
   threshold is not the producer's to carry either, because a threshold is
   policy and the producer is the thread that must not branch. The producer
   publishes **its rate**, which it alone knows; the reader derives the cap
   from that rate and decides what to do past it.

2. **A display-link-backed `frame` clock is wrong on iOS**, which kills one
   sentence of that clock's justification. The platform throttles its display
   link during the system-gesture gate, so a cold play immediately after a
   scroll — the most ordinary way a user starts playback — stalls the first
   frame and reads as a hang. A plain main-queue timer keeps firing and the
   layer still composites. So `frame`'s contract is **"ticks at roughly
   display rate"**, not "ticks on vsync", and on `.iOS` the implementation must
   be a timer. Demand-driven survives untouched, because the battery argument
   never depended on vsync. The genuinely open part is the **first** tick: it
   has to be dispatched rather than fired synchronously, and `reactivity.md`
   says nothing about a write that lands during the mount pass.

3. **`Ring<T>` is not the default shape for a deadline thread.** This probe was
   expected to be its first corpus use and is not. The strongest design
   available deletes the concurrency problem instead of solving it: generate
   the whole event list up front, sort it once, and keep it **immutable** for
   the duration of playback — then the deadline thread needs no lock, because
   there is no producer to wait for. In elp that is a `Buffer<MidiEvent>`
   moved in **by value**, and the move is what makes the immutability a
   type-level fact rather than a convention, since the control thread no
   longer has the buffer. `Ring<T>` keeps a narrower justification: live
   input, where there is nothing to generate up front.

   Twice in this entry the correct real-time answer is immutability rather
   than synchronisation — the event buffer, and a `Copy` timing table read by
   both threads. Worth naming as a pattern: on a deadline thread the
   language's job is mostly to make *"this cannot change"* expressible, and
   `Copy` plus `Move` already do.

**What it confirmed, and one of them strongly.**

4. **Avoiding a refcount on the audio thread required an unsafe unowned
   capture, with its safety proof written in a comment.** That is exactly
   `aliasing.md`'s non-obvious hazard — a refcount decrement reaching zero
   calls the allocator, which takes a lock — met in production. An ordinary
   strong capture retains and releases per callback, and the release is the
   one that can free. The host language had no way to say "borrow this for the
   callback's lifetime", so the author asserted it unsafely, and the proof was
   *"no callback can run after the engine is stopped and the node detached"*.
   elp's `Opaque<T>` no-retain rule was argued from leaks and landed in the
   same place, with that proof's ordering clause being `Scoped` release order.
   The strongest single piece of evidence in the entry: the design rule and
   the shipped workaround agree, and only one of them is checked.

5. **Keeping reinterpretation out of `Buffer<T>`'s identity was right.**
   Interleaved stereo — the most common real audio layout there is — is
   expressed by a C call taking *offset and stride as arguments* over one
   allocation. A `Buffer<T>` that carried a layout would have had to carry
   this one twice over the same memory.

6. **The host's output buffer is a transient view**, confirming that row of
   the foreign-pointer table has to be a borrow and not a value.

**What it opened.**

7. ~~**Single-writer discipline wants the endpoint split for *cells*, not just
   queues.**~~ **Resolved, and the opposite way.** `aliasing.md` found that
   `Move` on a ring's two ends makes SPSC unconstructible rather than
   documented. The production bug here is the same
   discipline on a single value: when the control thread moved the play cursor
   itself, a seek landing mid-callback was clobbered by that callback's
   write-back, and a note-on fired after the control thread's all-notes-off
   stuck on. The fix was that the deadline thread became the *only* writer and
   the control thread merely **requests** a start, which the next callback
   applies. `Atomic<T>` has one undivided handle, so nothing in the language
   expresses that.

   `aliasing.md` **rejects** the split for cells. Multi-writer atomics are the
   normal case rather than an abuse — a CAS on a list head, a refcount, and
   the transport word in this very probe, which both threads
   compare-exchange — so constraining the general primitive would break them
   all. And the production fix was not a constrained writer: the cursor
   stopped being shared at all, which makes this a **visibility** problem that
   private fields already solve, with nothing on the hot path paying for
   synchronisation it does not need. Recorded as "Where a deadline body's
   private state lives" and "And this is why `Atomic<T>` does **not** get the
   endpoint split".

8. ~~**Two values must sometimes be read as one, and real code packs them.**~~
   **Resolved** — `Atomic<T>` now admits a `Copy` object whose layout fits a
   machine word, with a use-site size diagnostic, and the layout computation
   is the one §10 finding 3 already demanded for `@external` struct fields. A
   transport flag and a generation counter in one word, read with a single
   load, because split atomics allow a combination that never existed to be
   observed. That *defends* `Atomic<T>`'s word-size restriction — packing is
   what an implementation does anyway — and argues that `Atomic<T>` should
   accept a **record** that fits a word rather than only a primitive, since
   `Transport { playing bool, generation int32 }` says what the bits mean and
   an `int` plus two shifting helpers does not. The residual cost is that word
   size is a target property, so an object that fits on 64-bit and not on
   32-bit is a per-target error.

9. ~~**`publish` does not always want to succeed.**~~ **Resolved, and the
   mechanism was right.** The position store is guarded by re-reading the
   transport word: if playback restarted mid-buffer, this callback's stale
   position must not replace the new start's.

   The guarded publish — `publish(value, ifEpoch = word)` — is wrong twice:
   the epoch belongs to the transport rather than to the position, so the
   primitive would be checking a condition it cannot see, and it puts a branch
   and a re-read on the one thread that must have neither. The epoch goes in
   the **payload** instead, and the reader discards a reading whose generation
   does not match the transport it also sampled. So `publish` never fails
   after all, and the finding was about usage rather than mechanism.

10. **Production reaches for relaxed ordering, and would lose nothing to
    sequential consistency.** Every atomic in the real sequencer is relaxed
    except the transport word, which is acquire/release because it orders the
    event buffer behind it. The rates are the defence and they are measurable
    rather than argued: the position is stored about 43 times a second (one
    render buffer at 1024 frames and 44.1 kHz is 23 ms) and loaded about 60,
    so a full barrier on ARM is not detectable by any instrument a user owns.
    The author chose relaxed out of discipline, not measurement — the right
    instinct in a language that offers the choice, and an irrelevance in one
    that does not. The single exception worth noting is a liveness counter
    nobody reads for its value, which is the one shape where relaxed is
    genuinely free and seq-cst is genuinely waste.

11. ~~**A view must not get a vote on the thing it is a view of.**~~
    **Resolved as a design rule, with no feature.** The playhead deciding
    playback was over killed the transport during the release tail, before
    the sequencer could re-arm, so looping never survived its first pass
    while the score was on screen. A reader of a `Published<T>` is
    syntactically free to act on what it read, and a read-only endpoint would
    not have helped, because the stop call was on a different object. "Which
    side may act" is a real axis the language says nothing about — and will
    not, because enforcing it needs a capability system that costs more than
    every bug of this shape put together. `aliasing.md` records it as the
    boundary on "all policy on the reader": policy about the *value*, never
    control over the *producer*.

12. **A derived value that recomputes equal should probably notify nobody** —
    which would make a reactive graph's fan-out depend on `Equatable`. This is
    the real content of the proposed structural-position lint: "which bar is
    under the playhead" changes a few times a second but is *derived from* a
    value that changes sixty times a second, so a naive markup `if`
    re-evaluates at the clock's rate rather than the answer's. Real renderers
    memoise it by hand.

13. **The cell thread has a deadline too**, and only the audio thread's is
    marked. 16 ms instead of 23, a dropped frame instead of a click, the same
    discipline — and `@abi(.C, realtime)` says nothing about a `pure fn`
    called once per frame from a markup read.

14. **A platform API is a third category** alongside `links` and
    `dependencies`. The system audio graph is neither a shared library the
    package claims nor a dependency the build acquires, and
    `name-resolution.md` has no row for it.

### What migrating it then found

Findings 7–9 and 11 were recorded as resolved and the probe was migrated onto
those resolutions. The migration produced five more findings and one new gap,
which is the strongest argument available for doing step 3 of the workflow
rather than treating a decision as finished when it is written down.

15. **The `&var self` question dissolved, and the real defect was a boundary
    form.** The recorded answer to finding 7 said private deadline state is
    "a plain `var` field, private", which collides with `&var self` for a
    mutating method. It was wrong, and the probe itself contained the defect:
    it declared the callback parameter `Opaque<PlaybackState>` over a bare
    object, and `memory-model.md`'s table has only `Opaque<Cell<T>>`. The
    out-of-line principle says why — a bare elp value is in-line and moves,
    so there is nothing for C to point at.

    Correcting it to `Opaque<Confined<PlaybackState>>` removes the question
    entirely: mutation goes through the cell, `.set()` takes `&self`, and
    `examples/Sheet/types/style.elp` had written the sentence years of design
    time earlier — *"`&self`, not `&var self`: the object is unchanged, the
    cell it points at is not."* No new spelling, and the cell kind is forced
    rather than chosen, since `Shared` is immutable, `Synchronized` is a lock
    and `Atomic` would put a private value back into shared memory.

16. **`Confined<T>` off the cell thread is sound for a different reason than
    on it.** `examples/Sheet` justifies read/`copyWith`/write by cell-thread
    serialisation. A deadline thread has none, and the idiom is still atomic
    because nothing else holds the cell — which `Move` gives and `.alias()`
    takes away. So `Opaque<Confined<T>>` at an `@abi(.C, realtime)` boundary
    is the one sanctioned crossing of a non-`Sendable` cell, and it is
    **weaker** than the `Buffer<T>`-by-value parallel it invites: there the
    value is gone and the hazard is absent, here the registration must keep
    holding the cell and only *access* is relinquished. Privacy is what
    relinquishes it, so privacy is load-bearing for soundness at this one
    boundary.

17. **A cell holding a `Move` aggregate cannot have its fields updated at
    all.** `Confined<T>`'s only write is `.set(whole)`, which needs `T` to be
    `Copy`, and a `Move` field cannot be taken out of the `&T` that `.get()`
    returns. Every cell in the corpus until now held a `Copy` payload, so this
    is the first place it bites — and it forces a nested cell per mutable
    field of a `Move` aggregate.

18. **GAP 6 — and the teardown of such a cell has no spelling.** Wrapping the
    registration in the cell the boundary form requires leaves
    `release(self)` unable to reach the `Move` handle it must destroy.
    Candidates: a consuming `.take() -> T` on a uniquely-held cell, `Scoped`
    reaching through a cell it is the last holder of, or keeping the `Move`
    resources *beside* the cell rather than inside it. The third is probably
    right and is an architectural claim rather than a workaround — what a
    deadline thread must **read** and what a program must **release** are
    different sets, and the probe had them in one object only because the
    original could hold both in a class.

19. **A private field forces a factory, and that generalises.** The
    registration used to be constructed field-by-field from `main.elp`. It
    cannot be once the render cursor is private, so construction moved into
    the sequencer's own module. The rule worth keeping: **a type with a field
    only one thread may touch cannot have a public initialiser.**

20. **A boundary payload belongs with the reader, not the writer.** The
    transport word had to be declared alongside `Playhead` rather than in the
    sequencer, because the reader's `estimate` needs it for the generation
    check and the sequencer already imports the reader's module — so
    declaring it on the writing side would have made the two files import each
    other, which `name-resolution.md` forbids. The no-cycles rule picked the
    filing, and the principle it picked is the right one: both threads need
    these types and the reader is the side that interprets them.

Two of the four resolutions also came out *cheaper* in the migrated probe than
on paper. The payload already carried the producer's rate before finding 1 was
resolved, so "the producer publishes its rate" cost nothing. And both reader
policies — the extrapolation cap and the stale-generation discard — ended up
in one `pure fn`, which is the clearest available statement of why finding 11
needs no feature: legitimate reader policy is a total function of the value,
and the illegitimate move was never a policy at all but a call to `stop()`.

## 12. Reference cycles — `examples/Recalc`

Written to answer one question: **elp reclaims memory by reference counting
and has no `Weak`, so where does the corpus leak a cycle?** A spreadsheet
recalculation graph is the sharpest honest place to ask, because the
back-edge is not a design preference — incremental recalc needs forward
edges to evaluate a cell and reverse edges to know who to dirty when it
changes, and every real spreadsheet has both.

The answer is narrower than "refcounting leaks cycles" and wider than "it
cannot happen". `aliasing.md` carries the constructibility argument; what
follows is what the probe found on the way to it.

1. **The natural cyclic graph is unconstructible, and not by a rule about
   cycles.** A graph of `Confined<Node>` handles, with each node holding its
   precedents and dependents, is the textbook `weak var` site. It cannot be
   written: `Node` holds `[Confined<Node>]` so `Node` is `Move` field-wise,
   and §11 finding 17 established that a cell holding a `Move` aggregate
   cannot have its fields updated at all. `node.dependents.append(other)`
   has no spelling.

   Two rules argued for on unrelated grounds — "collections hold values, not
   places" and `Confined<T>`'s `Copy` requirement — close off the shape
   between them. This is the second probe to hit that wall from a direction
   its author did not expect, which is the strongest evidence available that
   it is a real rule rather than an artefact.

2. **What is left is an arena keyed by address, and its ownership is a
   tree.** The `Graph` owns every `Node` and no `Node` owns anything, so a
   node's reference count is one, always. The probe needed no cell at all:
   a spreadsheet model turns out not to need aliasing.

3. **The cycle does not disappear, it changes category.** `A1 = B1` with
   `B1 = A1` is still a cycle — now in the *formula* graph rather than the
   *object* graph, which makes it a domain error a spreadsheet must detect
   and report rather than a silent leak of two allocations. Keys cost a hash
   lookup per edge and turn an invisible memory bug into a visible
   user-facing one.

4. **No reference equality is what pushes graphs towards named nodes.**
   The cycle check compares two `Reference`s, which is comparing two ints.
   Had the edges been handles it would have needed identity comparison, and
   `operators.md` has `==` with no `===` — so there is no spelling for "the
   same object" in the language at all. Two decisions supporting each other,
   neither of which cited the other.

5. **A cycle *is* constructible, through one narrow shape.**
   `Shared<Confined<T>>`: `Shared<T>` is `Copy` for any `T`, so a `Copy`
   payload may transitively own a mutable cell, which satisfies
   `Confined<T>`'s `Copy` requirement with exactly the thing it was meant to
   exclude. Three lines, leaks forever, no diagnostic. The argument for
   `Shared<T>` being `Copy` — that an immutable alias is observationally
   identical to a copy — is true and is about observation rather than
   reachability.

6. **GAP — `Weak<T>` is undesigned**, so a program that genuinely needs a
   back-edge between cells has no spelling for one. The recommendation is
   *not* to add it now: the shapes that would want it are mostly
   unconstructible, and the valuable artefact is the record of which
   decisions close them off, since a future loosening would reopen this
   silently.

7. **GAP — a leak has no observable symptom.** No destructors means "was
   this released?" is not a question a program can ask or a test can assert.
   `Move` giving at-most-once and never at-least-once release is recorded for
   FFI handles; the same holds for a cycle of elp cells and had not been
   written down. Leak detection is undesigned.

8. **GAP — `Map.get` on a `Move` value has no stated meaning.** A value read
   out of a collection is a copy with an explicit write-back (§8, resolved),
   but every such read in the corpus until now was of a `Copy` value.
   `Node` is `Move`, so "a copy" has to mean a `Duplicate`, a move-out, or an
   error, and nothing says which.

9. **GAP — recursion over a user-authored graph has no bound.** Dirty
   propagation terminates because `cached` is already `nil`, which is correct
   and implicit. elp has no stated depth limit, no tail-call guarantee, and
   no way to say "bounded by the graph being acyclic" when the graph is
   user-authored and may not be. The same shape as `examples/QuadTree`'s
   recursion gap.

### What re-running it against one proposed addition found

`examples/Recalc/internal/handles.elp` writes the graph finding 3 said was
unconstructible, assuming `Confined<T>.update(fn (T) -> T)`. Findings 1–9
above are all correct and finding 3's conclusion is **conditional**, which was
not stated.

10. **The write surface, not the cycle rule, is the defect.** `Synchronized<T>`
    has `.update(fn (T) -> T)`; `Confined<T>` has `.get()`/`.set()` and
    nothing else, and no `Confined` `.update` appears anywhere in the corpus.
    `.update` does a second thing unrelated to locking — it moves the payload
    out and back, never duplicating it — so a read-modify-write today costs one
    record copy for a `Copy` payload, **a deep copy of the whole payload** for
    a `Duplicate` one, and is impossible for a `Move` one. The middle row is
    `examples/Buffer`'s `self.lines.get().copy().insert(...)`: a shared mutable
    text buffer duplicating every line per keystroke. One defect — shared
    mutable state must be transitively `Copy`, or pay a deep copy per write,
    or be unexpressible — reported four times (§11 finding 17, Playhead's
    GAP 6, this probe's GAP 1, `examples/Buffer`).

11. **`aliasing.md`'s justification for the asymmetry covers a different
    claim.** It argues the `.get()`/`.set()` pair is *permissible* on a
    confined cell because the cell thread makes the sequence atomic. That is
    not an argument that `.update` is *unnecessary*, and the doc ran the two
    together.

12. **Under `.update`, every edge is a cycle.** Not a circular formula — the
    graph is deliberately bidirectional, so one acyclic `A1 = B1` leaves A1
    holding a handle to B1 and B1 holding one to A1. The leak stops being the
    `Shared<Confined<T>>` needle and becomes every parent link, observer
    registration and doubly-linked list. So **the cell write surface and
    `Weak<T>` are one decision**, recorded in two places as if independent.

13. **The fork is whether `.alias()` is a side-effectful call.** `aliasing.md`
    proposes `update(pure fn (T) -> T)`. Closing an edge needs `b.alias()` in
    the transform body; `functions.md`'s example establishes that such a
    transform may *capture*, so captures are not the question. If `pure` is
    observational, `.alias()` is allowed and every edge leaks; if it is about
    reachability, `.alias()` is rejected, the finding-3 result survives, and
    the thing `.update` was wanted for still works — a buffer insert captures
    no handle. The effect system is undesigned and this is the first thing
    asking it a real question.

14. **And it is the same conflation twice.** `Shared<T>` is `Copy` for any `T`
    because an immutable alias is *observationally* identical to a copy —
    a claim about observation settling a question about reachability, and the
    reason the `Knot` needle exists. Deciding observation-versus-reachability
    once, for both sites, is worth more than either answer.

15. **GAP — `.update` empties the cell, which is a worse failure mode than the
    one it replaces.** For the transform's duration the cell holds nothing, so
    a transform reaching its own cell observes a hole rather than a stale
    value. `Synchronized<T>` gets away with the same structure because
    re-entering a held lock deadlocks loudly; `Confined<T>` has no lock, and
    atomicity is what makes the hole observable from inside. Not contrived —
    an ordinary graph `invalidate` holds a borrow across a recursion that
    comes back round, with nothing syntactic marking it.

16. **The handle graph loses on its own terms anyway.** A formula names
    addresses, so it needs the address-to-node map *and* two handle arrays per
    node, where the arena needs only the map. It buys a pointer chase instead
    of a hash lookup. Even with `.update` granted this probe would choose the
    arena — which is why findings 10–15 are about what the language *permits*,
    a different question from what a program should do.

17. **And it is the ninth report of one absence.** `&var self` on a method
    receiver is the only mutable access in the language, so no library type
    can hand out a mutable place. "Mutable paths into an interior node" in
    §4 is the same hole from a spatial tree; `Map.get` on a `Move` value,
    `examples/QuadTree`'s GAP 4, "collections hold values, not places", the
    `&var T` parameter, and `examples/Playhead`'s GAP 6 are the rest.
    Tabulated in `memory-model.md`, "One hole, reported nine times", with
    the argument that a cell's write surface was never decided — it is an
    unfinished edit from the `examples/Pipeline` migration, which corrected
    `Synchronized<T>` and left `Confined<T>` alone.

### An inconsistency this probe tripped over

`nil` and `null` are both in the corpus — 117 uses across 23 files against
41 across 5. The `null` files are the FFI-heavy ones (`SqliteBinding`,
`VulkanTriangle`, `Playhead`), which may be a deliberate distinction between
elp's empty `Optional` and C's null pointer, or may be drift. Nothing records
an intent either way. `nil` is used here.

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
