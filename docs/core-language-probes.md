# Core language probes

Eleven programs written to put pressure on the parts of elp the original
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
8. **The count-then-fill out-parameter needs a place.**
   `vkEnumeratePhysicalDevices(instance, &count, null)` then again with a
   buffer: a nullable argument whose nullness changes the function's
   meaning, an in-out count, and a destination that wants an elp array's
   backing storage. `collections.md` says collections hold values, not
   places. `SqliteBinding` found single out-parameters awkward; an array
   out-parameter is a different problem.
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

### The construct three unrelated probes have now asked for

A scoped form — `with Connection.open(path) as db { ... }` — has been
reached for independently by `SqliteBinding` (exactly-once release),
`VulkanTriangle` (the five-deep teardown ladder *and* GLFW's
`init`/`terminate` global, which has no handle for a tier to describe),
and noted in `aliasing.md` as the same shape as `Synchronized<T>`'s
`update`/`with`. Three probes, two of them FFI and one not, converging on
one statement form is the strongest signal in the corpus about what to
design next.

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
