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

**Still true after scoped release.** `memory-model.md` adds
`with ... as x { }`, which binds a resource to a block and releases it at
every exit — RAII-shaped, and not RAII, because release is tied to a brace
rather than to a value's lifetime. It does not rehabilitate the guard: a
`with` block's boundary is marked, but a yield *inside* it is still
invisible, so a lock held across one deadlocks exactly as before. The line
is that `with` is for **resources**, which need release once and in order,
not for **locks**, which need exclusivity between statements.

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

Four precisions, because each one catches somebody:

**A field may not hold a borrow.** `&T` is unconditionally `Copy`
(`memory-model.md` Finding 6), which is harmless inside a call and not
harmless in a field: the field outlives the call and is freely copyable,
so a `&T` field is a duplication route field-wise approves of without
looking at `T` at all.

A borrow is bounded by the call that receives it; a field is not, and
there is nothing to bound it against. `examples/SqliteBinding` had the only
two `&` fields in the corpus and both were the bug, so the rule costs
nothing today and keeps Finding 6 unconditional rather than caveated.

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
is obtainable only from `Confined<T>`, `Synchronized<T>`, a closure that
captured one, or **a borrow of a type whose mutators elp cannot see** —
and the first three are what field-wise conformance already sees. A user
type cannot invent interior mutability, so the compiler never has to ask.

The fourth is FFI's, and it is worth stating exactly, because an earlier
draft of this document claimed the compiler simply *could not see* an
external handle's semantics. It sees them; it gets them wrong. `&T` is
`Copy` because `T`'s only mutators are elp code honouring the borrow, so
two holders of a `&T` have nothing to observe about each other. An
`@external` type's mutators are in C: `sqlite3_close(db &Sqlite3)`
destroys the pointee **through a read-only borrow**. The rule that makes
`&T` free is the rule FFI breaks.

### Three parts, and no new vocabulary

**1. A field may not hold a borrow** — the precision above, which is the
general form of the problem rather than an FFI special case.

**2. A foreign handle is a value, not a borrow** — whatever the library's
own ownership story is. There is no owned `Sqlite3` anywhere in the
program for a borrow to borrow *from*: SQLite owns the pointee and elp
owns the pointer. So the field is `handle Sqlite3`, and `&Sqlite3` has no
referent in elp's model. Borrows stay legal at a *parameter*, where the
call bounds them.

This holds for the handles elp never releases too, which is the larger
class — see "Three kinds of foreign pointer" below. What varies between
them is the **tier**, not whether they are values.

**3. An `@external type` must declare its tier, and silence is an
error.** An external type has no fields, so field-wise is *vacuously*
satisfied and the derivation's answer is `Copy` — the dangerous one for a
`sqlite3*`. But no default in the other direction works either: `Move`
would mis-describe a `time_t`, a C enum, a POD struct, and most of
Vulkan. The tiers genuinely differ per handle, which is the whole argument
for making the declaration mandatory rather than picking a side.

```elp
@external(.C, library = "sqlite3") {
	export type Sqlite3
	export type Sqlite3Stmt
}

Sqlite3     implements Move
Sqlite3Stmt implements Move
```

**And then nothing above it needs extending.** `Connection` holds a
`Move` field, so field-wise forces `Connection implements Move` — a
conclusion, exactly as `Style implements Move` is a conclusion from
holding a `Confined<StyleValues>`. One type at the boundary whose
semantics the compiler is *told*, everything above it derived, which is
the same shape stdlib already uses.

That is also what makes it legitimate rather than a hole. stdlib declares
`Confined<T> implements Move` and the compiler trusts it; `@external` is
the one place a *user* gets the same privilege. Not an exception to the
rules — the same trust, scoped to the marked surface.

### The three questions, answered by conformances that exist

| | answer |
|---|---|
| is duplicating the handle independent, aliasing, or illegal? | the **conformance**: `Copy`, `Duplicate`, `Move` already mean exactly those three |
| may it cross a thread? | **`Sendable`**, declared rather than derived |
| who releases it, and is exactly-once guaranteed? | **open** — elp has no destructors, no `defer`, nothing at scope exit |

So two of the three need no new syntax, which retires a strawman
`examples/SqliteBinding` was carrying:

```elp
@external(.C, library = "sqlite3", duplication = .aliasing, threads = .confined)
```

That invented `.aliasing`/`.independent`/`.illegal` for precisely what
`Move`/`Copy`/`Duplicate` say. A parallel vocabulary for a distinction the
language already draws is worth deleting rather than designing.

The thread question is answered in *form* and still blocked in *fact*:
SQLite's discipline is a runtime value (`sqlite3_threadsafe()`) against a
static conformance, which no part of this language covers.

### Three kinds of foreign pointer, and only one is a handle you release

`sqlite3*` is the easy case and generalising from it alone would be a
mistake. GLFW and Vulkan hand back pointers to resources *they* own and
never expect back, and they are the common shape rather than the exotic
one:

| | you release it? | tier | examples |
|---|---|---|---|
| **owned handle** | yes, exactly once | **`Move`** | `sqlite3*`, `sqlite3_stmt*`, `GLFWwindow*`, `VkInstance`, `VkDevice` |
| **borrowed handle** | no — it dies with its parent | **`Copy`** | `VkQueue`, `VkPhysicalDevice`, `GLFWmonitor*` |
| **transient view** | no — invalidated by the next call | *none fits* | `const char*` from `sqlite3_column_text`, `const GLFWvidmode*` |

The first two are both handled by rule 3 with no further machinery, and
they land on *different* tiers — which is the clearest evidence that a
mandatory declaration beats a default. `Move` is right for a `VkDevice`
because you call `vkDestroyDevice` once; `Copy` is right for a `VkQueue`
because you never destroy it, duplicating it is free, and two copies of it
have nothing to observe about each other that matters.

**What the second row does *not* get is validity.** A `VkQueue` is valid
only while its `VkDevice` lives, and nothing in the tier says so.
`Copy`/`Duplicate`/`Move` answer *may I duplicate this, and how?*; they do
not answer *is this still alive?*. Those are orthogonal, and elp has a
mechanism for the first and none for the second.

That gap was already recorded from `Statement`/`Connection`, and Vulkan
makes it much worse than one pair suggested: its handles form a strict
ownership **tree** — instance → physical device → device → queue, buffer,
image → view — and every edge is a constraint elp cannot express. Rust
cannot either without a hand-written lifetime, and `ash` declines to,
making its handle types `Copy` and leaving use-after-free to the
programmer. So the honest statement is that elp's binding would be exactly
as unsound as Rust's here, and the tier is not the thing that fixes it.

**Two things the table does not capture, both from
`examples/VulkanTriangle`.**

A borrowed handle may be `Copy` and **not** `Sendable`. A `VkQueue` is a
pointer you never release, so `Copy` is right; Vulkan queues are not
thread-safe, so it must not cross into `background`. Everywhere else in
the corpus a `Copy` type is trivially `Sendable` because it is plain data,
and this is the first type that needs to decline — legal, since the two
are unrelated conformances, and worth stating because the pairing has
always been free until now.

And a field can outlive a **sibling field**. `Renderer` holds a `Move`
`VkDevice` and a `Copy` `VkQueue` that points into it, so the object
cannot be duplicated while that field can be read out and copied anywhere.
Field-wise `Copy` protects the aggregate, not the members — which is
correct for `Confined<T>`, because reading a handle out keeps the cell
alive, and wrong for a foreign handle, which keeps nothing alive. That
difference is the whole reason the validity gap is FFI's and not the
language's.

**The third row is the one nothing covers.** A transient view is not a
handle: its validity is measured in *calls*, not in the lifetime of a
parent. No tier can express "you may read this and may not keep it",
because `Move` still permits storing. What every real binding does instead
is copy at the boundary and never expose the pointer — which is what
`Statement.textAt` returning `Optional<string>` does — and that is a
discipline at the wrapper rather than a guarantee from the language. It is
also why the `@external` declarations belong in `internal/`.

**The mechanism already exists and is not a keyword.** `@external(.OBJC)`
is in the corpus (`BlogStyle/internal/api.elp`) and the parser already
accepts body-less declarations for it; `name-resolution.md` adds `links`
on `@Package` as the allow-list bounding what it may reach. There is no
`unsafe` marker and no `extern` keyword, because `@external` is the
marker and `reactivity.md` rejects new lowercase intrinsic keywords as a
matter of policy. `examples/SqliteBinding` is the probe.

**And `@external` carries no duplication arguments at all**, which is
where the three parts above land. A macro receives the AST of what follows
it, and that may be a block of declarations (`macros.md`), so one
`@external(...)` header covers a library's handles — but what each handle
*means* is an ordinary conformance written under the block, not an
argument inside it. The attribute says where the symbol comes from; the
conformance says how the value behaves. Those were conflated in the first
draft and they are different questions.

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
- **`string`'s tier is not stated anywhere.** It is not in the table
  above and `strings.md` does not say. It decides whether an ordinary
  record of strings is `Copy` or `Duplicate`, which is most of the corpus,
  so it is a bigger hole than its size suggests. Found while working out
  whether `Connection { handle, path string }` was `Copy`-eligible.
- **Exactly-once release for a foreign handle.** `Move` gives at most
  once and never at least once, so a dropped handle leaks with nothing to
  notice. The candidates are a `Drop`-shaped interface, a scoped `with ...
  as x { }` form, or lint-only. Narrowed from the old "what arguments
  `@external` carries", the rest of which is now answered by ordinary
  conformances. Worse, SQLite's thread
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
