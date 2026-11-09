# Aliasing and duplication

## What this document decides

How a value is duplicated, whether its duplicates can observe each
other, and which of those duplicates may cross a thread.

It supersedes "Amendment to Finding 3" in `memory-model.md`, which
reached the right problem and the wrong answer twice. The corrections
are recorded at the end rather than hidden, because two of them were
found by writing corpus against the proposal.

## The problem, in one pair of declarations

`examples/Buffer` is a split-pane editor: two panes share one text, each
keeps its own cursor. Under the original design both are spelled the
same way:

```elp
Cursor implements Copy      // duplicates are INDEPENDENT
Buffer implements Copy      // duplicates ALIAS the same cell
```

One word, two incompatible guarantees, and nothing at either declaration
site says which you get. `memory-model.md` Finding 3 called this *"one
mechanism (type-level `Copy`), not two"* and counted it as an economy.
It is not an economy; it is a conflation.

Rust is careful at exactly this point: `Arc<T>` is `Clone`, **not**
`Copy`, and `#[derive(Copy)]` is refused for any struct holding one. The
restriction is what lets `Copy` keep meaning "trivially duplicable and
independent".

## Three types carry the aliasing

The semantics lives in the **type**, where a reader meets it, not in a
conformance they have to go and look up.

| type | second handle | can two handles observe each other? | crosses a thread? |
|---|---|---|---|
| `Shared<T>` | implicit — it is `Copy` | **no** — nothing can write | **yes**, when `T` is `Sendable` |
| `Confined<T>` | explicit — `.alias()` | **yes** — writes are mutual | no, ever |
| `Synchronized<T>` | explicit — `.alias()` | **yes** — writes are mutual | **yes**, when `T` is `Sendable` |

All three are refcounted handles. Duplicating any of them duplicates a
pointer, never the referent.

### `Shared<T>` — immutable sharing

`.new(v)` and `.get()`. There is no `.set`.

An immutable alias is **observationally identical to a copy**: two
holders have no operation through which either can detect the other. So
it is `Copy`, for *any* `T` — `Shared<[string]>` is `Copy` even though
`[string]` is not.

That is not a new rule. `memory-model.md` Finding 6 already decided it
for references:

> A reference (`&T`, for *any* `T`) is unconditionally cheap to copy…
> so `&_` belongs in the same "free, no declaration" tier as primitives.

`Shared<T>` **is** that rule given an owning form — `&T` with a refcount
instead of a lifetime. Same tier, same reasoning, no new precedent.

### `Confined<T>` — mutable sharing on one thread

`.new(v)`, `.get()`, `.set(v)`, `.alias()`. The thing a library reaches
for when "edit this once, and every holder sees it" is the actual
requirement — `examples/Sheet`'s styles, `examples/Buffer`'s text.

Never `Sendable`, under any `T`. It is `Rc<RefCell<T>>`, and it is sound
only because of cell-thread confinement (`reactivity.md`), which is
where the name comes from.

### `Synchronized<T>` — mutable sharing across threads

The same capability where there is no single thread to confine to:
`kind = .Binary` server code, `background` work, `examples/Worker`'s
pool. `Arc<Mutex<T>>`.

**Its surface is two closures, split by direction:**

```elp
.update(fn (T) -> T)        // read-modify-write, atomically
.with(fn (&T) -> R) -> R    // read under the lock, return something derived
```

```elp
hits.update(fn (n) { return n + 1 })
```

A `.get()`/`.set()` pair over a lock is memory-safe and still
*logically* racy — two writers interleave between the read and the write
and one update is lost. `Confined<T>` can expose that pair precisely
because the cell thread makes the sequence atomic; off the cell thread
nothing does.

And **yield points are invisible** (`operators.md`, "The amendment: a
call ends the turn"). A lock *guard* object could therefore be held
across a call that yields, and deadlock with nothing in the source
marking the boundary. elp also has no RAII and no `&mut` spelling, so
there was never a good way to express a guard anyway.

`update` takes a **pure transform** rather than a mutating body, which
is what `examples/Pipeline` corrected. A single
`with(fn (v &var T) { ... })` needs two things the language does not
have: a `&var T` *parameter* spelling, which `memory-model.md` Finding 4
deliberately rejected in favour of inferring mutability from usage, and
whole-value assignment through a reference, which is
`examples/QuadTree`'s GAP 4. A transform needs neither, cannot forget to
write back, composes with the `copyWith` the corpus already uses, and
makes the critical section obviously total.

**What may appear inside the closure** is the open part. The body must
not call `background`, receive from a channel, or reach into another
`Synchronized`. That reads like a restriction on a closure body *by
position*, which the language has no other instance of — but the
parameter type being `pure fn(T) -> T` would supply it from a mechanism
that already exists, since `grammar-script-notes.md`'s "no external
calls" rule rules out both hazards outright. Whether `pure fn(T) -> T`
is a writable *function type* as opposed to a declaration form is a
question for `functions.md`.

## Three conformances, and only one rule

`Copy`, `Duplicate`, `Move`. There is **no `Shared` interface** — see
"What this corrects".

| | duplication | independence | examples |
|---|---|---|---|
| `Copy` | implicit | independent | primitives, `Cursor`, `Money`, `Point`, `JobId`, `Value`, **`Shared<T>` for any `T`** |
| `Duplicate` | explicit, via `.copy()` | independent, possibly deep | `[T]`, `Row`, `Sheet` |
| `Move` | exclusive transfer — the source is dead | | everything else that opts in, **`Confined<T>`**, **`Synchronized<T>`** |

`Duplicate` is the row the first draft missed, and the corpus had been
using it all along:

```elp
posts.copy().prepend(post)                      // SocialFeed/screens/home.velp
state.messages.copy().append(...)               // ChatRoom/main.velp
```

`.copy()` is explicit duplication of something that is *not* implicitly
`Copy` — otherwise nobody would have to call it. It is Rust's `Clone`.

Separating it from the aliasing types is where elp ends up **better
placed than Rust** rather than catching up: Rust puts `Arc<T>` and
`Vec<T>` in the same trait, so "this type is `Clone`" tells you nothing
about whether duplicating it aliased. Not retrofittable there, free
here.

### `.copy()` and `.alias()` are two words on purpose

A mutable handle is **`Move`, not `Duplicate`**, and a second handle
comes from `.alias()`.

The first draft of this document put `Confined<T>` and
`Synchronized<T>` in `Duplicate` and was caught while migrating
`examples/Buffer`: `Duplicate` promises independence, and
`Confined.copy()` would hand back something that aliases. That is the
original `Copy` bug one tier down — one word, two guarantees, and no
site at which to see which you got.

So the words do not overlap anywhere in the language:

| | means | duplicates alias? |
|---|---|---|
| `.copy()` | `Duplicate` — a second, independent value | **never** |
| `.alias()` | a second handle to the same referent | **always** |

`.copy()` never aliases and `.alias()` always does, so neither call site
needs its receiver's type checked to be read correctly. That is worth
one extra method name.

It also means a type wrapping a handle hand-writes its own `.alias()`,
which is exactly the honest duplication the design wants — the sharing
is stated at the one site that has to think about it:

```elp
export fn Buffer.alias(&self) -> Self {
	return Self { lines = self.lines.alias(), path = self.path }
}
```

`Shared<T>` needs none of this. It is `Copy`, so a second handle is a
plain assignment — which reflects the reality that immutable sharing is
free and mutable sharing costs you a word.

### The rule: `Copy` is field-wise

**A type is `Copy` only if every field is `Copy`.** That is the whole
mechanism. There is no propagation rule, no contagion rule, and nothing
to derive.

Because `Confined<T>` and `Synchronized<T>` are `Move` and not `Copy`,
any type holding one is not `Copy` either, transitively, all the way up.
`examples/Sheet` has four levels of it — `Style` → `Cell` → `Row` →
`Sheet` — and no rule says so four times. One rule applies four times.

Three precisions, because each one catches somebody:

**It blocks only the implicit tier.** A type holding a `Confined` field
is not `Copy`; it can still be `Duplicate`. Explicit duplication is fine
*because* it is explicit — `.copy()` on a spreadsheet `Cell` sharing the
style and copying the value is a decision somebody wrote down. What is
refused is silent duplication, not duplication.

**A field's `const`/`var` marker does not change it.** Eligibility comes
from the field's **type**:

```elp
export object Cell {
	var value   Value,
	const style Style,     // const field, still not Copy
}
```

`const` says *you cannot rebind this field*. The `Confined` inside
`Style` says *you can write through it*. Two independent axes — which is
what `Sheet/types/style.elp` already notes: *"`&self`, not `&var self`:
the object is unchanged, the cell it points at is not."*

**Closures are field-wise too**, so they need no separate treatment.
`functions.md` already says a closure is `Sendable` iff every capture is;
it is `Copy` iff every capture is, by the same reading. Two closures over
one `Confined` handle therefore need two aliases:

```elp
const a = self.lines.alias()
const b = self.lines.alias()     // two handles, one cell
```

Which is `examples/Undo`'s GAP 3 with an answer rather than a gap, and
the `.alias()` calls are where the sharing gets written down.

## `Sendable`, and the premise under conditional conformance

```elp
Shared<T: Sendable>       implements Sendable
Synchronized<T: Sendable> implements Sendable
```

`Confined<T>` declares no such line, for any `T`, and that is the whole
enforcement — exactly how `reactivity.md` already handles reactive cells
and `Binding<T>`: *"a reactive cell fails to conform by simply never
declaring it."* No bound to propagate, no trait chain to read backwards.

Field-wise conformance then does the propagation for free. `Style` holds
a `Confined` → `Cell` holds a `Style` → `Row` → `Sheet`: none are
`Sendable`, and the error lands on the capture the developer wrote:

> `sheet` cannot cross into `background` — `Sheet` holds a
> `Confined<Style>`, which is confined to one thread. Use
> `Synchronized<Style>`, or send the values it needs and write the result
> back.

### Why `Confined<T>` is not `Sendable` when `T` is

The obvious rule — `Confined<T: Sendable> implements Sendable` — is
**unsound**, and the counter-example is one line:

```elp
const hits = Confined.new(0)

background(fn {
	hits.set(hits.get() + 1)        // thread B
})

hits.set(hits.get() + 1)            // cell thread
```

`int` is as `Sendable` as a type gets. Both sides hold the same cell,
both read-modify-write, and the update is lost. **`T: Sendable`
constrains what is in the box; the race is the box.** The bound checks
the wrong noun.

### The premise, stated once so it stops being invisible

`Channel<T: Sendable> implements Sendable` (`generics.md`) is sound
because a channel is **internally synchronized** — its own operations
handle concurrent access, so the contents really are the only open
question. That is a hidden premise in the form, and it is why `Channel`
looked like a precedent for `Confined` when it is not one.

Every legal use of the form satisfies the premise for a stated reason:

| | why it may conditionally conform |
|---|---|
| `Shared<T>` | immutable — there are no writers to order |
| `Synchronized<T>` | internally synchronized — the lock is in the type |
| `Channel<T>` | internally synchronized |
| `Confined<T>` | **nothing** — so it does not conform |

**This premise is currently unguarded in the language.** Nothing stops a
library author writing `MyCache<T: Sendable> implements Sendable` over an
unsynchronized cache. That is `interfaces.md`'s problem rather than this
document's, and it is recorded as open there.

## Fields spell `const` or `var`

`memory-model.md` originally said `const` is **not** accepted on a field
— *"two spellings for the default is one more than the language needs."*
The corpus writes `const` on 72 fields and bare on none, so the rule was
both unimplemented and unloved.

**Resolved the other way, and more strictly: a field spells `const` or
`var`, exactly like a local, and bare is an error.** The old rule gave
two spellings for immutable and one for mutable; matching locals gives
one each, costs zero migration, and means a reader never has to decide
whether a missing word meant immutable or meant the author forgot.

| declaration | outside the type |
|---|---|
| `const field T` | not visible |
| `var field T` | not visible |
| `export const field T` | read |
| `export var field T` | read and write — `&obj.field` is legal |

This replaces the table in `memory-model.md`, "Fields carry `var`".

## FFI is the one place duplication semantics are declared by hand

`unsafe` exists in elp **only** at a boundary to a native shared library
— C, C++, Objective-C — and nowhere else. It is not a general escape
hatch and there is no safe-Rust-style `unsafe` block for ordinary code.

That decision is what makes everything above inferable. Mutable aliasing
is obtainable only from `Confined<T>`, `Synchronized<T>`, or a closure
that captured one, all of which field-wise conformance already sees. A
user type cannot invent interior mutability, so the compiler never has to
ask.

FFI is the exception, and it is the *only* exception. An opaque handle
behind an `@external` declaration — a `sqlite3*`, a `CURL*` — has
duplication semantics the compiler cannot see, so the declaration must
state them:

- is duplicating the handle independent, aliasing, or illegal?
- may it cross a thread, and under what discipline?

**The mechanism already exists and is not a keyword.** `@external(.OBJC)`
is in the corpus (`BlogStyle/internal/api.elp`) and the parser already
accepts body-less declarations for it; `name-resolution.md` adds `links`
on `@Package` as the allow-list bounding what it may reach. There is no
`unsafe` marker and no `extern` keyword, because `@external` is the
marker and `reactivity.md` rejects new lowercase intrinsic keywords as a
matter of policy. `examples/SqliteBinding` is the probe, and the
arguments `@external` needs to carry are open — see below.

**Where they are written is settled, though.** A macro receives the AST of
what follows it, which may be a block of declarations (`macros.md`), so a
library's handles are declared under one `@external(...)` header and the
semantics are stated once for all of them. That is the right granularity rather than a
convenience: "duplicating this aliases, and it may not cross a thread" is
a property of *this library's handles*, not of each symbol separately.

## What this corrects

Four drafts were rejected on the way here, and each was killed by corpus
rather than by argument. Recorded because the reasoning is the
load-bearing part.

**1. "`Copy` is two things, and the second one is `Shared`"** — a fourth
*conformance* named `Shared`. Rejected: once the aliasing lives in
`Shared<T>`/`Confined<T>`/`Synchronized<T>`, field-wise `Copy` does
everything the conformance did, and a user cannot construct aliasing
without one of those types (see FFI, above, for the exception). The
conformance was only ever needed because there was no type to put the
semantics in.

**2. "`Shared` is not contagious"** — a dedicated rule saying a type with
a `Shared` field is not implicitly duplicable, plus a `@Duplicate` derive
to make it bearable. Rejected: that is field-wise `Copy` restated as a
special case. "Non-contagion" and "poisoning" both named a consequence as
if it were a mechanism, which is why the next question was how to
mitigate it.

The *observation* behind the derive survives and is still wanted.
`examples/Sheet` forces four duplication functions and three of them
contain no decision at all — they map a collection and forward. That is
boilerplate, and boilerplate is how a safety rule gets worked around. A
`@Duplicate` derive generating field-wise duplication is the fix, and it
is honest precisely because each field's *type* declares which it is.

**3. "A field's type may not hide a mutable alias"** — proposed so a
reader of `const style Style` could tell that writes through it are
visible to others. Rejected while migrating `examples/Buffer`: it cannot
be stated without forbidding abstraction. `Buffer` holds a
`Confined<[string]>`, so `View { const buffer Buffer }` would be illegal,
and so would every type that wraps a shared resource — which is most of
what a library is for.

What answers the objection instead is field-wise `Copy` itself. Because
`Buffer` is not `Copy`, `View` is not `Copy`, so duplicating a `View` must
be written by hand — and *that* is where the sharing gets stated, at the
one site that has to think about it. The declaration site stays silent;
the duplication site cannot. All five of `examples/Buffer`'s findings
close on this alone, with `Style.setBold` surviving as a method, which
proposal 3 would have killed.

**4. `Confined<T>`/`Synchronized<T>` as `Duplicate`** — so that one
`.copy()` covered both independent duplication and a second handle.
Rejected while migrating `examples/Buffer`, because `Duplicate` promises
independence and a `Confined.copy()` would hand back something that
aliases. That is the original `Copy` conflation one tier down, and it
would have been worse there: `Copy` at least had no call site to inspect,
while `.copy()` would have had one that looked identical in both cases.
`.alias()` is the fix — see "two words on purpose", above.

## Open

- **What `.get()` returns, and how long it lives.** It returns `&T` —
  anything else would move the referent out of the cell and leave every
  other holder pointing at nothing, and `examples/Pipeline` hit that on
  the first type it wrote. The shape is already legal
  (`memory-model.md` Finding 5 allows borrow-returns) and already
  assumed (`examples/Buffer` writes
  `self.lines.get().copy().insert(...)`, which reads correctly under
  `&T`). **Undecided is the lifetime:** a `&T` out of a `Confined<T>` is
  invalidated by any `.set()` on the same cell, and `operators.md`'s
  amendment says any call may yield, so the "a call ends the turn"
  reasoning that governs narrowing has to govern this too. It currently
  does not.
- **What arguments `@external` carries for an imported handle.** What
  states a foreign handle's duplication and thread semantics, and what the
  default is when the declaration is silent — on a block header, per the
  section above, so the question is what the arguments *are* rather than
  where they go. Worse, SQLite's thread
  discipline is a *runtime* value (`sqlite3_threadsafe()`) while
  `Confined`/`Synchronized` is a static distinction — a capability that is
  real but not statically known, which nothing in this language covers.
  `examples/SqliteBinding` puts the cases on the page; none are decided.
- **Does `Confined<T>` unify with a reactive cell?** A component `var`
  lowers to a cell, `Binding<T>` is a handle to one, and `Confined<T>` is
  a handle to one — three names for what may be one mechanism. If a
  `Confined<T>` read inside a component body subscribed like a cell, the
  reactive system and library sharing would be the same thing. Attractive
  and unproven.
- **How deep is `.copy()`?** Conditional conformance gives the rule
  (`[T]` is `Duplicate` when `T` is `Copy` or `Duplicate`), but whether a
  nested structure duplicates at every level or one needs stating.
- **Can a type be `Copy` and `Duplicate` both?** It should not need to
  be, and overlapping tiers should be a diagnostic rather than a silent
  precedence rule.
- **`Protected<T>` is not a peer of these three.** It is a
  compiler-enforced access restriction, not duplication semantics, and
  the tables above should not be read as including it.
