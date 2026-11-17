# The stdlib boundary — what is language, what is library, what is privileged

The goal is a language as bare as possible with a standard library written in
elp, so that the stdlib dogfoods the language it is written in. This file
records what that actually requires, because the obvious two-way split is the
wrong one and the prior art is less uniform than it looks.

## Three buckets, not two

| | what it is | test |
|---|---|---|
| **1. Language** | syntax, and what the compiler desugars to | a user could not write it as a library at any cost |
| **2. Privileged library** | library code that needs a capability users do not have | it is written in elp, but not in *the* elp a user gets |
| **3. Plain library** | ordinary elp | a user could have written it, and could write a competitor |

Bucket 2 is the one that decides whether dogfooding is real. A stdlib that is
"written in elp" but needs a privilege on every other line is dogfooding the
compiler's internals, not the language. **The discipline is to minimise bucket
2, and the way to do that is usually to add something small to bucket 1.**

Which gives the rule this file exists for:

> **Bare does not mean fewer features. It means fewer privileged features.**
> A language feature that moves five things out of bucket 2 makes the language
> barer, not richer.

## What the prior art actually does

All three of Rust, Swift and Go write their standard library in their own
language. Only two of them have a bare language, and the difference is
instructive.

| | core data structures | stdlib's privilege | notes |
|---|---|---|---|
| **Rust** | library (`Vec`, `String`, `HashMap`); `[T]`, `str`, arrays are language | `unsafe`, `core::intrinsics`, `#[lang]` items | `Box` had to become a lang item to support moving out of it, and fifteen years on a user still cannot write an equivalent — no `DerefMove`. The cautionary tale for bucket 2. |
| **Swift** | library, aggressively — `Int`, `Array`, `String`, even `Optional`, with `+` as a protocol method and `?` as sugar | `Builtin`, `@_frozen`, `@inlinable`, `isKnownUniquelyReferenced` | The closest model to the goal here, and library CoW is built on that last privileged call. Costs: compile times, and a stdlib using attributes users cannot. |
| **Go** | **language** — `map`, slice, `chan`, `string` are built-in types; `append`, `len`, `make`, `delete`, `copy` are built-in functions | the runtime, in Go plus assembly | Deliberately the opposite. Go had no generics until 1.18, so collections *could not* live in a library and the language absorbed them. |

So "stdlib written in the language" and "bare language" are different claims,
and Go only makes the first. elp has generics, so the Swift/Rust route is open.

## What is different about elp, and the decision it forces

`aliasing.md` records that **`unsafe` exists in elp only at a boundary to a
native shared library** — there is no safe-Rust-style `unsafe` block for
ordinary code. And privilege is granted in exactly one place today: a declared
tier on an `@external type` is *"the one place a user gets stdlib's
privilege"* (`memory-model.md`, `aliasing.md`).

Rust's stdlib is written in Rust *because* it can say `unsafe`. Swift's is
written in Swift because it can say `Builtin`. elp can say neither, so bucket 2
has no inhabitant mechanism yet. Interior mutability, contiguous storage and
weak references are irreducibly bucket 2 — `Confined<T>`, `Buffer<T>` and
`Weak<T>` cannot be written in safe elp in *any* language.

**The decision, and it is not made:** does elp get a stdlib-privilege marker,
or does every privileged primitive become a compiler-known type?

- A **marker** is Rust's and Swift's answer. The privileged set stays visible
  in stdlib source and is auditable by reading it, and new primitives do not
  need compiler work.
- **Compiler-known types** are Go's answer. The privileged set is smaller and
  harder to abuse, and the cost is that every primitive is a language change —
  which is how Go ended up with `map` in the language.

The leaning is the marker, because elp has generics and therefore does not need
the language to absorb collections, and because an auditable list in stdlib
source matches how every other elp privilege is recorded. What it needs is a
name and a rule for who may write it, and neither exists.

## The current open items, sorted

Written while the write-surface round was closing, so this is the live list
rather than an illustration.

| item | bucket | note |
|---|---|---|
| `&var T` as a non-receiver parameter | **1** | and it is what moves the next row out of bucket 2 |
| `update(fn (&var T) -> R) -> R` on collections | **3** | ordinary elp, conditional on the row above |
| a recursive value type's indirection (`indirect`) | **1** | see `memory-model.md`; chosen *because* it avoids bucket 2 |
| `break` | **1** | not expressible as a library |
| an optional borrow (`Optional<&T>`) | **1** | a lifetime rule |
| `string`'s tier | **1** — it is primitive | but a CoW `string` puts its *implementation* in bucket 2 |
| `update` on the four cells | **2** | interior mutability is irreducible |
| `Weak<T>` | **2** | needs a runtime side table |
| `Map.get`'s two spellings, `Map.set`'s re-insert order | **3** | two methods and a decision |

The shape of that table is the argument for the round order: three bucket-1
items are blocking bucket-3 work, so until they land, half of what looks like
a stdlib ergonomics question is "there is no way to write this at all", which
is the wrong question to be answering about ergonomics.

## The worked example — `indirect` instead of `Boxed<T>`

The recursive-value-type gap is the clearest case of the rule at the top of
this file, because the two candidate answers sit in different buckets.

- **`Boxed<T>` as a stdlib type** is bucket 2, and Rust says what it costs: the
  box needs powers no other type has (moving out of it, placement), so it
  becomes privileged and permanently unmatchable by user code.
- **`indirect` as a field marker** is bucket 1, and adds **no type at all**. It
  is a layout instruction rather than a wrapper, so there is nothing to
  privilege, nothing to name, and nothing for a user to be unable to write a
  competitor to.

The second is smaller *and* barer, and the fact that it is a language feature
is the reason. That is the rule working.

Designed in `memory-model.md`, "A recursive value type needs an indirection,
not a box".

## The ergonomics round

Once the bucket-1 decisions are ratified, the corpus scan collects every
remaining finding and gap, and each one gets asked, in this order:

1. **Is it bucket 1?** Only if a library could not express it at any cost.
   "A library version would be ugly" is not an answer; ugly is bucket 3.
2. **Is it bucket 2?** Only if it needs a capability users do not have. If the
   capability is `&var T`, or an indirection, or an optional borrow, the answer
   is to ratify that instead — one bucket-1 item is cheaper than a privileged
   type.
3. **Otherwise it is bucket 3**, and the question is only whether it belongs in
   the stdlib or in a package, which `name-resolution.md`'s atmospheric
   criterion already answers for the prelude.

The failure mode to watch for is a bucket-1 request that is really a bucket-3
convenience — the corpus has several, and `functions.md`'s rejection of `|>` is
the model for how to turn one down.

<!-- vim: ft=markdown -->
