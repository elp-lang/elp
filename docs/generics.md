# elp generics and overloading

## What the corpus demanded

Five distinct features were hiding behind `<>`. Separating them is most
of the design.

| | corpus |
|---|---|
| generic types | `Optional<string>` ×7, `Either<Content, Error>` ×5, `Protected<string>`, `HttpBody<Content>`, `StateDispatch<AppActions>` |
| generic interfaces in conformance position | `From<HttpError>`, `From<string>`, `From<int8>`, `From<Error>`, `Codec<JSON>` |
| generic functions, explicit type arguments at the call site | `Http.get<[Post]>(url)`, `Http.post<object{success bool}, Error>({body})`, `to<JSON>` |
| a generic function declaration with bounds | `fn OBJC_msgSend<T: OBJC_Obj + Sendable>(T, OBJC_Selector) -> T` |
| **not generics**, same syntactic slot | `object{name string}` as a type argument; `@Screen<BlogPostParams>` and `@App<ChatApp, @Reducer<...>>`, where a *macro* takes type arguments |

Bounds reuse the `: A + B` shape already settled in `interfaces.md`, so
they need nothing new.

## Type arguments are always mandatory

A generic is never instantiated by inference. `Http.get<[Post]>(url)`
is written out, and so is `value.to<JSON>()`.

This is what makes the rest of the design cheap, and the clearest case
is `To<T>` below: two conformances give a type two methods that differ
*only* in return type, which is the construct that makes Swift's type
checker exponential and produces "expression too complex to be solved
in reasonable time". With the type argument written, it is not an
overload resolution problem at all — it is an explicit instantiation.

Mandatory rather than "optional when unambiguous" is deliberate.
Omitting it when exactly one conformance exists reads better and makes
a call site break the day somebody adds a second conformance in another
file. Rust's `.into()` has exactly that fragility.

**The cost, stated plainly:** `let x: Foo = y.into()`, where the target
type is inferred from context, is not expressible in elp. The corpus
never does it.

## `From`, `To`, and `Codec`

The corpus had `ToFrom<JSON>` and `FromTo<JSON>` — one concept, two
spellings, both in `BlogStyle`. They are really two interfaces:

```
interface From<T> { fn from(value T) -> Self }
interface To<T>   { fn to(&self) -> T }

interface Codec<T>: From<T> + To<T> { }
```

`Codec` rather than `Into`: in Rust `Into` is the *reciprocal* of
`From` — the same conversion in the opposite direction — not the
conjunction of two, so anyone arriving from Rust would read it exactly
backwards.

This is the fifth use of the `: A + B` shape, and it adds one
capability worth naming: the dependency is **parameterised**, with
`Codec<T>` passing its `T` through to both parents.

The split also divides the overloading problem in two, and the halves
need different machinery — `From<T>` overloads on parameter type, while
`To<T>` is disambiguated by its mandatory type argument.

## Overloading

**Overloading is on parameter types and parameter names. A difference
in return type alone is never resolved by inference.**

Parameter names are part of a signature because a signature is an
explicit contract. That has three consequences:

- `move(to Point)` and `move(by Vector)` are different methods, which
  is Swift's argument-label disambiguation.
- An implementation of an interface method must use **the interface's
  parameter names**, not its own. This is not pedantry: a call through
  an interface-typed value has to know what label to use, so the label
  must be fixed by the interface rather than by whichever type happens
  to be behind it.
- Names remain **optional at the call site**. The corpus calls both
  ways, often in one expression —
  `TextArea(&body, placeholder="What's on your mind?")` — and
  `Text("Log in")` should not have to grow a label. A positional call
  that cannot be resolved is an ambiguity error, and naming is the fix.

### Resolution: exact, then unique, then error

1. A candidate whose parameter types match **exactly** wins outright.
2. Otherwise **exactly one** candidate must be reachable by
   conformance. If two are, it is an ambiguity error that the author
   resolves by annotating.

There is deliberately no ranking beyond this. The partial-ordering
rules that decide "which overload is more specific" are what make C++
overload resolution a specialist subject, and the corpus never needs
them: `ApiError.from(err)` with `err HttpError` hits rule 1 and never
reaches rule 2, even though `HttpError` also conforms to `Error`.

Preferring an error over a silently-chosen candidate is the same call
this language has made everywhere else.

### This amends the name-collision rule

`interfaces.md` rejects name collisions at the definition site. That
cannot survive generic interfaces: `ApiError` conforms to
`From<HttpError>`, `From<string>`, `From<int8>` and `From<Error>`, and
each contributes a method named `from`. Collisions **across
instantiations of the same generic interface** are legal and resolved
by the rules above; collisions between unrelated interfaces are still
rejected.

## Conditional conformance

Required, not optional — `reactivity.md` already states that
`Channel<T>` is `Sendable` when `T` is, and that is not otherwise
expressible. The same applies to `Optional<T>`, `[T]`, and `Copy`/
`Duplicate`/`Move` on any container.

```
Channel<T: Sendable> implements Sendable
```

"A `Channel` of a `Sendable` `T` implements `Sendable`." Sixth use of
`: A + B`, and no `where` keyword.

### The form carries a premise it does not state

`Channel`'s line is sound for a reason the syntax does not express: a
channel is **internally synchronized**, so the contents really are the
only open question. Where that premise fails the same spelling produces
a silent data race, which `aliasing.md` found the hard way —
`Confined<T: Sendable> implements Sendable` is the obvious rule and is
unsound, because `T: Sendable` constrains what is in the box while the
race is the box.

So on `Sendable` specifically, every legal use of this form owes a
reason: immutable (`Shared<T>`), or internally synchronized
(`Synchronized<T>`, `Channel<T>`). Nothing currently *checks* that, and
a library author can write `MyCache<T: Sendable> implements Sendable`
over an unsynchronized cache with the orphan rule's blessing. Tracked as
open question 5 in `interfaces.md`, since it is a property of
conformance rather than of generics.

## No template specialisation

Two things get called specialisation and only one of them is wanted.

**Choosing the most specific overload** is Swift's behaviour and is
covered by the resolution rules above.

**Specialising a generic implementation for a particular type
argument**, in the C++ sense, is refused. It is the feature Rust has
kept unstable for most of a decade because it is unsound in the
presence of a coherence rule, and elp has already adopted the orphan
rule (`interfaces.md`). The use case is served by writing a distinct
overload.

## Variance: invariant

`Optional<Cat>` is not an `Optional<Animal>`. Rust does the same for
user types. Variance is a large amount of rule for a small amount of
convenience, and nothing in the corpus asks for it.

## Two parsing problems

**`<` is ambiguous at a call site.** `get<Content>(x)` against
`a < b > (c)`. Rust pays for this with turbofish; Go 1.18 avoided it by
choosing `[T]`. The corpus wants bare `<>`.

The parser is hand-written recursive descent, so the C# approach works:
on seeing `<` after an identifier in expression position, scan ahead to
a matching `>` and accept it as type arguments **only if the next token
is `(`**. Bounded lookahead, no backtracking of parse state.

In type position there is no ambiguity, because a type is never a
comparison.

**`>>` must not lex as a shift.** `Optional<Protected<string>>` closes
with two `>`. The clean fix is to never lex `>>` at all, and have the
parser combine two adjacent `>` into a shift in expression context —
rather than splitting a `>>` token when in type context, which is the
version that gets messy.

## Open

- **Structural object types.** `object{name string}` appears as a type
  argument three times, and `main.velp` describes projection-and-
  truncation semantics ("the type on the left projects the values on
  the right", extra fields a warning and truncated). That is structural
  subtyping and it is its own design, not part of this one — but the
  type grammar has to admit it.
- **Generic components.** `component List<T>(items [T])` has no corpus
  instance and no reason yet not to work.
- **Macros taking type arguments.** `@Screen<BlogPostParams>`, and
  `@App<ChatApp, @Reducer<...>>` where a type argument is itself a
  macro. Belongs with `macros.md`.
- ~~**Is the value-level `+` interface-driven?**~~ **Resolved** by
  `operators.md`: yes. `a + b` is `a.add(b)` via an `Add` interface, so
  `base + emphasis` is not a styling special case and user types get `+`
  by declaring a conformance. The `+` in `implements A + B` remains a
  separate, settled thing: a conformance-list separator, not an
  operator.

## Associated types

An interface method returning an interface type is an existential: the
concrete type is unnameable. `Iterable` did that by accident, and an
associated type puts the type back into the contract. Recorded in
`dispatch.md`:

```elp
interface Iterable<T> {
	type Iter: Iterator<T>

	fn iterator(&self) -> Self.Iter
}
```

A conformance binds it in a separate declaration, the same way its
methods are separate declarations rather than a block:

```elp
type Array<T>.Iter = ArrayIterator<T>
```

`Self.Iter` is **part of what the conformance promised**, so a caller
may name it and nothing is inferred behind a signature's back. That is
what makes this the principled fix rather than an optimisation.

Worth flagging: the reason for looking at `Iterable` at all was
performance, which makes this the first feature in the design arrived at
that way.

## Migration

- `ToFrom<JSON>` and `FromTo<JSON>` become `Codec<JSON>` — four sites
  across `SocialFeed/types/post.elp`, `SocialFeed/internal/api.elp` and
  `BlogStyle/internal/api.elp`, plus two imports and one comment.
- `fn ApiError.from(err HttpError)` becomes
  `fn ApiError.from(value HttpError)` in both `api.elp` files, because
  parameter names are now part of the contract and `From<T>` declares
  `value`.
- `OBJC_msgSend<T: OBJC_Obj + Send>` became `Sendable` when that
  interface was named.
