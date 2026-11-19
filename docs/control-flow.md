# elp control flow — loops, and the two ways out of one

`match` and `if` are expressions and belong to `patterns.md`. The iteration
protocol, `Range`, and what `for` desugars to belong to `collections.md`.
This file owns what was left with no owner at all: **loop statements**, and
`break`/`continue`.

It is the last of the three things `open-questions.md` listed as
load-bearing with no design. `while` appears in eleven corpus files and
nothing specified it.

## `while`, and nothing else

```elp
while offset <= h - n {
	...
}
```

- **The condition is an expression of type `bool`.** There is no
  truthiness; `operators.md` has no implicit conversion and this is not the
  place to introduce one.
- **Narrowing in the condition reaches the body**, by the rule that already
  governs `if` — and is invalidated the same way, by a write to the narrowed
  binding or by any call, since `operators.md`'s amendment says a call ends
  the turn. A loop body does both constantly, so in practice a narrowed
  binding is re-narrowed each iteration. No new rule.
- **It is a statement.** `while` produces no value. See "Loops are not
  expressions", below.

Two probes force it and neither is a counted loop in disguise.
`examples/TextSearch` advances by a computed skip distance — *"the outer
loop advances `offset` by a computed skip distance, not by one, so no
counted `for` expresses it"* — and `examples/VulkanTriangle` writes
`while !window.shouldClose()`, where the condition is the platform's.

### There is no `loop`

`while true` is the infinite loop, and a second spelling for one thing is
the argument this design has already made twice — for `const` on fields,
and for `nil` against `None`.

**The cost, stated.** Rust's `loop` exists for two things `while true` does
not give: it is known to diverge, so a function returning `T` may end in
one without a `return`; and `break value` makes it an expression. elp gets
neither. No corpus function ends in an infinite loop, and
`examples/Worker`'s worker loops are `for job in inbox` — a channel's
iterator blocks and returns `nil` on close (`collections.md`), which is the
common "loop forever receiving" already covered. If a probe ever needs the
diverging tail, the answer is `loop` or a `never` type, and both are
additive.

## `break` and `continue`

`break` leaves the innermost enclosing loop; `continue` begins its next
iteration. Both are statements, both take no value, and both are legal in
`while` and in `for`.

`examples/LruCache` is the evidence and it is specific: that cache's
eviction scans for the oldest key and has no way to stop once it has found
it, so FINDING 3 records the missing `break` turning an O(1) eviction into
an O(n) one. `examples/Worker` lists `break` among the constructs it was
written to stress.

**`break` out of a `for` over a channel is ordinary.** It stops iterating;
it does not close the channel. That is worth saying because it is how a
worker leaves its loop early, and because channel shutdown is separately
undesigned (`core-language-probes.md` §7b) — these two must not be confused
for each other.

### No labels

`break` and `continue` name no loop, so there is no way to leave two at
once except `return`.

`examples/Sheet` has the corpus's only nested loop pair and wants nothing of
the sort; `examples/QuadTree` escapes a loop by returning from the function,
which is the shape the corpus actually has. A labelled break is additive
whenever a program needs one, and adding it now would be designing against
no evidence. The honest statement of the gap: a nested search that must
leave both loops *and keep going in the same function* has to use a flag.

## `for` was already decided — in three other files

Nothing here changes it; it is collected because it was hard to find.

| part | where |
|---|---|
| `for x in xs` calls `xs.iterator()` then `next()` until `nil` | `collections.md` |
| `Iterable<T>` / `Iterator<T>`, with an associated type so it stays static | `collections.md` |
| a channel's iterator blocks and ends on close | `collections.md` |
| `0..n` is an expression producing a `Range`, which is `Iterable<int>` | `collections.md` |
| the binding is a **pattern**, not an identifier — `for { key as k, value as v } in map` | `patterns.md` |
| a markup `for` yields a sequence of nodes and takes a mandatory `by` | `grammar-script-notes.md` |

Two corrections fall out of collecting it. `grammar-script-notes.md`'s
markup grammar line reads `for = 'for' IDENT 'in' expr 'by' expr block`,
and the binding is a pattern rather than an `IDENT` since `patterns.md`
resolved two-binding iteration as record destructuring. And
`core-language-probes.md` §2 asks whether a range as an iterable expression
and a range as a match pattern are the same construct; `collections.md`
answers it — *"same token, two constructs, distinguished by context exactly
as Rust does"* — and that entry was never updated.

### `a..b` includes both ends

Undecided until now, and the match side forces it: `400..499` has to
include 499 or every HTTP status table in the corpus is wrong. One range
form with one meaning beats an inclusive form and an exclusive form, so the
iterable side takes the same reading.

**A range whose end is below its start is empty**, so `0..n - 1` with
`n == 0` iterates nothing rather than trapping or running backwards.

**The cost, and it is real.** "Do this `n` times" is `for i in 0..n - 1`,
and an off-by-one there is a silent bug of the kind an exclusive range
exists to prevent. Two things take the pressure off: a loop over a
collection iterates the collection rather than its indices, which is most
cases; and where it genuinely wants indices, `xs.indices()` is stdlib surface
rather than arithmetic. If the corpus still reads badly after migration,
Kotlin's `until` is the additive answer — a second *spelling*, which this
file argues against, but for a second *meaning*, which is different.

### Counting down has no range form

`examples/Undo` walks a document backwards with `while i >= 0`. A
descending range is not a thing — `n..0` is empty by the rule above — and
the answer is `reversed()` on the iterable, which is stdlib surface. That
keeps one direction in the range type and puts reversal where every other
sequence transformation already lives.

### A collection may not be mutated while it is iterated

And this needs no rule of its own, which is worth noticing. `xs.iterator()`
takes `&self` and `xs.append(y)` takes `&var self`, so a body that mutates
the collection it is walking has two overlapping paths to one storage —
refused by the exclusivity rule in `memory-model.md` Finding 4's third
amendment. The diagnostic every language eventually bolts on falls out of a
decision made for an unrelated reason.

**`for i in xs.indices()` is not that, and the difference is the whole
reason `indices()` is worth having.** It returns a `Range`, which is a
*value* computed once — it holds no borrow of `xs` — so mutating elements
inside that loop is legal where `for x in xs` would not be.
`examples/Editor`'s `reflow` is the corpus case: it rewrites every line of
a document in place, and the index loop is what makes that expressible.

## Loops are not expressions

`match` and `if` produce values (`patterns.md`); `for` and `while` do not,
in value context. There is no `break value`.

The one exception is already written down and is a context rule rather than
a different construct: in **markup** context a `for` yields a *sequence of
nodes* (`grammar-script-notes.md`). `break` and `continue` are legal there
too, and mean what they mean everywhere else — a markup `for` is `for` in
markup context, not a second loop form, and adding a restriction would make
it one.

## `by` is not vestigial, and it is not a binding

Raised while designing this, and worth answering in full because the
instinct behind it is right about something else.

```elp
for post in posts by post.id { ... }
```

`in posts` is the **binding**: what each iteration gets. `by post.id` is the
**identity**: what makes the third row of this render the same row as the
third row of the last one. `for post of posts` would supply the first and
say nothing about the second, so the two clauses are not two spellings of
one thing.

**Identity cannot come from the value**, and that is a decision this
language has already made. `operators.md` removes `===` on the grounds that
`by` is *better* than reference identity *"because it survives a value
being copied, which — given that collections hold values and props cross as
copies or shares — is the normal case rather than the exception."* Deriving
identity from the bound value reopens that.

**Could it come from the type?** This is the strongest version of the
question: an `Identifiable` conformance, and `for post in posts` suffices.
Rejected, for four reasons that stack:

1. **`[string]` and `[int]`.** Identity would have to be the value, so two
   equal elements silently collapse into one row. Written out, `by name` is
   a claim the author is making and can be wrong about visibly; derived, it
   is a bug with no site.
2. **A projected row has no type-level identity.** `examples/Expenses`
   builds `CategoryTotal` fresh every render and keys it `by row.category` —
   a field chosen *for this list*, not a property of the type.
3. **Enforceability**, which `grammar-script-notes.md` already argues:
   identity cannot be mandatory in any useful sense if the only check is
   semantic and fires on a child the loop never looked at. The loop form is
   what makes the grammar able to require it.
4. **Invisible-and-automatic is the worse half of React's mistake.**
   React's `key` is optional and therefore forgotten; a derived key would be
   automatic and therefore wrong without a line to point at.

So `by` stays, mandatory, in markup context only. What the question does
land on correctly is that `by` is *noise in the common case* — and the
cheapest honest answer to that is the one already available: `by` takes any
expression, so a list keyed by its own elements is `by name`, five
characters, and visibly a claim.

**And `in` stays `in`.** `of` is JavaScript's, where it exists to
distinguish iterating values from iterating keys — a distinction elp does
not have, since `for { key as k, value as v } in map` destructures instead.
Rust, Swift, Python and Kotlin all say `in`, nothing in elp uses `in` as a
membership operator for it to collide with, and `patterns.md` already
writes `in` for the destructuring form.

## Corpus migration

Most `while` loops in the corpus are counted loops written that way because
`for i in 0..n` had never been confirmed to work. It does.

| file | now | after |
|---|---|---|
| `examples/Worker/main.elp` | `var i = 0; while i < 32` | `for i in 0..31` |
| `examples/Sheet/sheet.elp` | two nested counters | `for r in 0..rows - 1`, `for c in 0..columns - 1` |
| `examples/Worker/internal/pool.elp` | `while spawned < workers` | `for _ in 0..workers - 1` |
| `examples/QuadTree` | `while i < 4` ×6, one with `i = 4` as a hand-rolled `break` | `for i in 0..3`, and a real `break` |
| `examples/Undo/main.velp` | `while i >= 0` | `for i in document.lines.indices().reversed()` |
| `examples/TextSearch` | two `while`s | unchanged — this is the probe that proves `while` is needed |
| `examples/VulkanTriangle/main.elp` | `while !window.shouldClose()` | unchanged |

`examples/LruCache` gains the `break` its FINDING 3 is about, and
`examples/Worker`'s pool GAP 2 resolves: a SHUTDOWN arm used `return` out of
the `background` closure, which conflated the loop exit with the coroutine's
end and only worked because the loop was the closure's last statement.

The best evidence for `break` turned out to be in `examples/QuadTree`, which
never filed a gap about it. `split` writes `i = 4` and then `i += 1` — a
hand-rolled `break`, which reads as a bug on first encounter. That is the
cost of a missing word, paid by a file that flagged eight other gaps and not
this one.

## Open

- **`indices()` and `reversed()`** are named here and are stdlib surface,
  designed with the rest of it (`collections.md`). `examples/Undo` is the
  evidence for wanting both at once — it walks a document backwards and
  records the index of each line it deletes, so `reversed()` alone would
  hand back lines and lose the index.
- **An exclusive range form.** Only if the corpus still reads badly after
  the migration above. Kotlin's `until` is the shape.
- **A labelled `break`.** Additive, and no corpus program wants one.
- **A diverging `while true`.** Needs `loop` or a `never` type; no corpus
  function ends in one.
