# elp collections

Collections hold **values, not places** — a value read out of one is a
copy and persisting a change needs an explicit write-back. That was
settled in `memory-model.md`; everything here builds on it.

## `Hashable`

`interfaces.md` names `Hashable` as a dependency and defines it nowhere,
so `Map<Category, Money>` is currently unwritable. It takes a hasher
rather than returning a number:

```elp
interface Hashable: Equatable {
	fn hash(&self, into &var Hasher)
}
```

### Why a hasher and not `-> int`

A type that returns its own fixed hash forces the Map to use it as-is,
which makes **hash flooding** possible: an attacker who controls keys
can collide them all into one bucket. That is an ordinary consideration
now that `kind = .Binary` means elp compiles servers.

Feeding a hasher the Map supplies lets the Map seed it per process, so
the same key hashes differently in different runs and precomputed
collisions do not transfer. It also composes — a type hashes by handing
its fields to the hasher rather than inventing a mixing function.

The verbosity is real, and the answer is that almost nobody writes one:

```elp
@CodeGen(Hashable)
export object Content { ... }
```

### `Hashable: Equatable` is a genuine dependency

Two equal values must hash equally, or a Map loses keys. The dependency
is a correctness requirement, not a convenience.

Worth contrasting with the one removed from `Comparable` last turn:
`Comparable: Hashable` was wrong because ordering and hashing are
unrelated, while `Hashable: Equatable` is wrong to omit. A dependency
earns its place when violating it breaks something, not when the two
interfaces merely tend to appear together.

## `Map` is insertion-ordered

Iterating a `Map` yields entries in the order they were first inserted.

This is not the cheapest option and it is chosen deliberately, for two
reasons that are specific to elp:

**Macros run at compile time.** `macros.md` makes a macro a `pure fn`
evaluated over the AST, with types and interfaces as first-class values.
A macro that iterates a Map and emits code must emit *the same code
every build* — three targets, reproducible artifacts. Hash order is not
stable across processes once hashing is seeded, so unordered iteration
would make codegen nondeterministic.

**Seeded hashing and test stability conflict otherwise.** The DoS
defence above makes hash order vary per run, which is a well-known
source of tests that pass four times and fail the fifth.

Insertion order costs a little memory and one indirection. Python and
JavaScript both made this trade and neither regrets it.

`Expenses/internal/stats.elp` sorts its breakdown explicitly, and under
this rule that stays the right thing to do — insertion order is
deterministic, but it is not *meaningful* order.

## Indexing, and what happens at the edge

One rule across every collection:

| | returns | out of range / missing |
|---|---|---|
| `xs[i]`, `xs.at(i)` | `T` | **traps** |
| `xs.get(i)` | `Optional<T>` | `nil` |
| `map[k]`, `map.at(k)` | `V` | **traps** |
| `map.get(k)` | `Optional<V>` | `nil` |

`[]` is `Index.at` from `operators.md`, so the bracket form and the
method are the same thing.

### Why the bracket form traps

`TextSearch/search.elp` is the argument. Its inner loop indexes a
character array with indices the algorithm has already proven in range:

```
while j >= 0 && text[offset + j] == self.needle[j] {
```

Had `[]` returned `Optional<char>`, that line would carry two unwraps
for conditions the programmer established two lines earlier, and the
algorithm would be unreadable. Meanwhile `Expenses/internal/stats.elp`
reaches for `map.get(k) ?? Money.zero()` precisely because a missing key
is expected there.

Both spellings exist because both situations are real. Making one of
them the only option penalises the other.

## Mutating methods take `&var self`

`map.set(k, v)` mutates the receiver, and elp had no spelling for that —
`memory-model.md` only establishes that *component* methods never need
it, because cells have interior mutability.

```elp
fn Map.set(&var self, key K, value V)
fn Map.remove(&var self, key &K) -> Optional<V>
fn Array.append(&var self, value T)
```

`&self` borrows; `&var self` borrows mutably. This reuses the `var` /
`const` vocabulary the language already has rather than importing
Rust's `&mut`, which the design deliberately avoids.

**Interfaces force the explicitness.** An interface method has no body,
so a mutating requirement cannot be inferred — `Iterator.next` below
must declare what it needs. Inference would also be out of character in
a language with mandatory type arguments, explicit conformance, and `&`
written at every writable call site.

Call sites are unchanged: `&` at a call site already infers `&T` versus
`&mut T` from the callee (`memory-model.md`, Finding 4). Declarations
state what they need; call sites write `&`.

## The iteration protocol

`reactivity.md` left this open as "blocked on generics: the interface is
generic and generic interfaces are not designed". Generic interfaces are
designed now (`generics.md`), so it unblocks.

```elp
interface Iterator<T> {
	fn next(&var self) -> Optional<T>
}

interface Iterable<T> {
	type Iter: Iterator<T>

	fn iterator(&self) -> Self.Iter
}
```

The associated type is what keeps iteration statically dispatched — see
`dispatch.md`. Returning a bare `Iterator<T>` would box, costing a
virtual call per element in the language's hottest construct.

`for x in xs` calls `xs.iterator()` once and then `next()` until it
returns `nil`. One protocol covers arrays, `Map`, ranges and channels.

### Channels fall out of it

A `Channel<T>`'s iterator blocks in `next()` until a value arrives and
returns `nil` when the channel closes. So this, from the
`LinkShortener` store, is the ordinary `for` loop and not a special
form:

```
for msg in inbox {
	match msg { ... }
}
```

That was the point of "`for` over a channel is the same `for`" when
channels were ratified. It now has a mechanism.

### `for k, v in map` and the tuple problem

A Map's iterator yields a concrete `Entry<K, V>` with `key` and `value`
fields, **not** a tuple. `patterns.md` since established that elp has no
tuple type at all, so this was the right call for a stronger reason than
the one given here.

Two-binding `for` is **record destructuring**, and `patterns.md` settles
that it is named rather than positional:

```
for { key as category, value as amount } in byCategory(expenses) { ... }
```

`Entry` turns out not to be special — it merely has those two fields —
so the question of whether two-binding `for` generalises beyond it
answers itself.

## Ranges

`0..n` is an expression producing a `Range`, which is `Iterable<int>`.
`400..499` in a match is a **pattern**. Same token, two constructs,
distinguished by context exactly as Rust does.

### `..` as a catch-all collides, and `_` wins

The corpus spells the catch-all arm two ways:

```
.. -> .new(.UNKNOWN)          // BlogStyle, SocialFeed
_  -> nil                     // ChatRoom, Expenses
```

**`_` is the catch-all; `..` is not.** Two spellings for one thing is
bad on its own, and here they actively conflict: `..` is range syntax,
so `.. -> x` reads as an unbounded range pattern rather than a wildcard.
`_` is already the unnamed-binding token elsewhere.

Two corpus sites to migrate (`SocialFeed/internal/api.elp:34`,
`BlogStyle/internal/api.elp:67`).

## `Set<T: Hashable>`

Same hashing requirement and the same insertion-ordered iteration as
`Map`. Nothing in the corpus uses one yet, so there is no evidence to
design against beyond consistency.

## Surface, briefly

Not the interesting part, and listed only so the shape is on record.
`Array`: `length`, `append`, `prepend`, `copy`, `contains`, `sorted`,
`map`, `filter`, `reduce`. `Map`: `length`, `get`, `at`, `set`, `has`,
`remove`, `keys`, `values`, `entries`. These are stdlib API rather than
language design, and they can grow without reopening anything above.

`sorted` takes a comparator returning `Ordering`:

```
rows.sorted(fn (a, b) { b.total.compareTo(a.total) })
```

## Open

- ~~**Is `Iterable`'s iterator an existential?**~~ **Resolved** by
  `dispatch.md`: no. `Iterable` gains an associated type, so the
  iterator's concrete type is part of what the conformance promised.
- **`a[i] = v`.** `Index` only reads. Under "collections hold values"
  an assignable index may not be wanted at all, and `set` may be the
  whole story.
- ~~**Two-binding `for` beyond `Entry`.**~~ **Resolved** by
  `patterns.md`: it is record destructuring, named with `as`, and
  applies to anything an iterator yields.
- **Capacity and growth**: whether `Array` and `Map` expose reservation,
  and whether that is language-visible at all.
- ~~**`Char`**~~ **Resolved** by `strings.md`: `char`, lowercase, one
  Unicode scalar value.
