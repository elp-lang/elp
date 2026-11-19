# What each decision cost, and where several solve one problem

Every other document here argues for a decision. This one looks across the
decided set and asks two questions no single document can: **how often does
one problem have several answers**, and **which decisions made another
program worse**.

Written after an outside review read the corpus rather than the docs and
asked both. The counting is deliberately conservative — a multiplicity with
a stated reason is not redundancy, and a cost with no program paying it is
not a defect.

## One problem, several answers

Five genuine overlaps. Two are already recorded as problems; three were not.

| problem | answers | status |
|---|---|---|
| **leaving a scope** | `defer`, `with … as x { }`, `Scoped` | **recorded.** Three constructs across two documents that each argued as if the others did not exist — `aliasing.md` twice reasoned from *"elp has no `defer`, nothing at scope exit"*. None is mandatory, which is the real shape of the exactly-once question. |
| **a property of a function body** | `pure`, `realtime`, + what may appear in a `Synchronized` transform, + thread affinity | **recorded.** One mechanism, four customers, two of them spelled as keywords and two unspelled. `functions.md` said a third is the moment to reconsider; there are two. |
| **an immutable refcounted indirection** | `Shared<T>`, `indirect`, `string` | **not recorded.** Identical machinery — immutable, refcounted, `Copy` by the same argument three times. What differs is *use-site visibility*: `Shared<T>` names a type and needs `.get()`, `indirect` is invisible, `string` is a primitive. That is a real axis and no document says it is the axis. |
| **changing a value** | `.set(v)`, `update(fn)`, `copyWith(…)`, `.copy()` then mutate, field assignment | **not recorded.** `.set(v)` is exactly `update(fn (&var t) { t = v })`, kept because it is shorter. `copyWith` survives only in a reducer, where there is no cell. Defensible, undocumented as a set. |
| **reading at an index** | `xs[i]`, `xs.at(i)`, `xs.get(i)`, `xs.update(i, fn)` | **partly justified.** `collections.md` argues the trapping and `Optional` forms both exist because both situations are real, which is right. `[]` and `at` being one operation with two names is not argued anywhere. |

And three things that **look** like overlaps and are not, recorded so they
stop being raised: `by` / `Equatable` / `Hashable` are layered rather than
alternative; backtick raw identifiers and `\{` are the same idea in two
domains with no shared mechanism to unify; and `Copy` / `Duplicate` / `Move`
are one decision with three outcomes, which is the point of it.

## One decision, another program's cost

Twelve, of which five have a program paying today and one has no payer at
all. The last column is what makes this list worth keeping: a cost with
nobody paying it is a prediction, not a finding.

| decision | what it bought | what it cost | paying |
|---|---|---|---|
| `string` is immutable, so `Copy` | the tier of nearly every record; `string` leaves the CoW question | building a string is O(n²); no builder | **`Json`** writer and lexer |
| no tuple type | `for k, v` is record destructuring, one rule | a compound key allocates; `enumerated()` must yield a record | **`control-flow`'s `by`** |
| `a..b` includes both ends | `400..499` means what it reads | `0..n - 1` for n iterations | **`Sheet`**, `Worker` |
| `update(fn (&var T) -> R)` | nine gaps at once | a failing transform leaves a partial write | **`Ledger`** FINDING 5 |
| exclusivity between `&var` paths | no two live mutable paths | a pairwise element operation has no spelling | **`Ledger`** FINDING 1 |
| `Map.get` returns `V` | one simple signature | a deep copy to ask a question about a field | **`Ledger`** FINDING 3, `LruCache`, `Interpreter`, `Json` |
| collections hold values, not places | no hidden aliases | `a[i] = v` has no spelling; `[T]` has no `set` | **`Editor`** FINDING 6 |
| `Index<K, V>.at -> V` | `a[i]` reads well | unimplementable for a non-`Copy` payload | **`Json`** |
| a default method may not touch fields | encapsulation holds absolutely | a generic `Codec` is impossible, so codegen is forced | **`Json`**'s macro |
| `by` is mandatory | the forgotten-key bug cannot be written | a key on every markup loop | every `.velp` |
| no `loop` | one spelling for one thing | no divergence information, no `break value` | `Json`'s parser, as a dissent |
| `indirect` is immutable | the container stays `Copy`; GAP 1 and the write surface stay separate | an in-place mutable recursive structure is unexpressible | **nobody** — the claim still holds |

## The middle ground

The list above has a shape, and it is not "these decisions were wrong".
Eleven of the twelve bought something worth more than they cost. The
question is why five of them produced a *defect* rather than a cost.

**A removal is affordable when a substitute exists at another layer.**
Immutable `string` removes in-place building and `[char].joined()` replaces
it — a library answer, and the decision holds. No tuples removes positional
pairs and records replace them — a language answer, and the decision holds.
No `loop` removes a spelling and `while true` replaces it.

**The defects are all places where two removals compose and the substitute
is gone too.**

- `Index.at -> V` needs `V: Copy`, **and** there is no borrowing accessor,
  because `Optional<&V>` is barred by a field not holding a borrow. Two
  removals, no substitute.
- A pairwise element operation is refused by exclusivity, **and** the
  move-out workaround is expensive because `Map.get` returns a value,
  **and** there is no transactional form. Three.
- `a[i] = v` is refused by values-not-places, **and** `[T]` has no `set`,
  **and** `update(i, fn)` is two closures deep at the one site that wants it.

So the middle ground is a check rather than a compromise: **before removing
a capability, name the substitute and confirm it is not itself removed.**
Cheap, mechanical, and it would have caught all three — each is a pair of
decisions made in different documents months apart, neither wrong on its
own.

That is the same failure `core-language-probes.md` §16 found one level down,
where three rules had been checked against the case that motivated them and
not against the set they governed. The corpus-wide version is this: a
decision is checked against the program that asked for it, and the cost
lands on a program written later.

## What this file is not

It is not a list of regrets and it should not become one. A design with no
costs in this table would be a design that had refused nothing, and the
refusals are most of what makes this language a particular language rather
than a collection of features.

The entries worth acting on are the five with a payer, and of those the two
with no substitute at all — the pairwise operation and the borrowing read —
are one decision away from each other: both dissolve if `Optional<&V>` is
answered.
