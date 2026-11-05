# elp dispatch

## What this document decides, and what it defers

Two questions travel together under the word "dispatch" and only one of
them is a language question.

**The language question:** when an interface name appears in a type
position, what does the program mean, and what may a reader rely on? That
is decided below.

**The backend question:** whether such a value is boxed, monomorphised,
devirtualised, or some mixture; whether the compiler sees the whole
program; whether a dependency can ship precompiled. That is **deferred**
until the language is designed, and the last section records the fork so
it is not lost.

The split matters because the first answer must not change when the
second one is made. If a program's meaning depends on how clever the
compiler was, the signature has stopped being a contract.

## The rule

**An interface name in a type position means "some type that conforms",
chosen by the callee. A generic bound means "one type that conforms",
chosen by the caller.**

```elp
fn max<T: Comparable>(a T, b T) -> T      // universal: the caller picks T
var err Error                              // existential: Error is a type here
fn load() -> Either<Content, Error>        // existential: callee picks
sum = sum + expense.amount                 // neither: Money is concrete
```

The distinction is **syntactic**. A reader can tell which is which by
looking — no inference, no whole-program analysis — because the two are
spelled differently.

## What follows from it

| | universal (`<T: I>`) | existential (`I` as a type) |
|---|---|---|
| who chooses the type | the caller, at the call site | the callee, inside its body |
| is the type written in the source | yes, always (`generics.md`) | no |
| can you name it | yes, it is `T` | no |
| can two values have different types | no — one `T` per instantiation | yes |
| to get at the concrete type | you already have it | type switch |

The last row is the one with consequences. `Either<Content, Error>` is
in the corpus precisely so a function can return either of two error
types without naming a closed set, and `BlogStyle/internal/api.elp` says
so: *"Because there are two potential Error implementations in the return
tree we just say this can return an Error, and at the call site we can
match on its type."* That makes the **type switch** load-bearing
language surface rather than a convenience, and it is still undesigned
(`error-handling.md`).

The second row is the other one. An existential's type is *not* written
anywhere in the program, so nothing a caller does can depend on it.
That is what makes the signature stable: a function may add a third
reachable error type without any caller's source changing meaning.

## Mandatory type arguments make the universal side explicit

`generics.md` ratified that a generic is never instantiated by
inference — `Http.get<[Post]>(url)` and `value.to<JSON>()` are written
out. That was decided to avoid return-type overloading, and it pays off
again here: on the universal side, **every type that a generic is used
at is written somewhere in the source**. The set is enumerable by
reading the program.

That is a language property, not a codegen one. It is what lets a reader
answer "what types does this function get used at" by grepping, and it
is what will make any later codegen choice a cheap one rather than a
search problem.

## Where existentials occur, and all three are deliberate

**1. An interface as a value type.** `var err Error`,
`Either<Content, Error>`. Four corpus functions return `Either<T, Error>`
for the reason quoted above.

**2. `*fn` callback props.** `export *fn onPosted(post Post)`. A
callback's target is unknown by construction — it is supplied from
outside the component — and `memory-model.md` already settled that a
callback is a stored, shareable value rather than a type parameter.

**3. `Component` in a markup tree.** A node's children are
heterogeneous: `Column { Text(...) Row { ... } }` holds values of
different types in one container. No universal quantifier expresses
that; it is the textbook case for an existential.

Each is a case where the program genuinely does not know the type, not a
case where it knows and declines to say.

## `Iterable` was an accident, and associated types fix it

The one existential that was *not* deliberate:

```elp
interface Iterable<T> {
	fn iterator(&self) -> Iterator<T>      // existential, and nothing asked for one
}
```

Nothing here wants the iterator's type hidden. An `Array<T>` knows
exactly what its iterator is, and a `for` loop has no use for the
freedom to not know. The interface was throwing the type away by
accident of spelling.

**Associated types** let a conformance put it back in the contract:

```elp
interface Iterable<T> {
	type Iter: Iterator<T>

	fn iterator(&self) -> Self.Iter
}
```

A conformance binds it in its own declaration, exactly as its methods
are separate declarations rather than a block:

```elp
Array<T> implements Iterable<T>

type Array<T>.Iter = ArrayIterator<T>

fn Array<T>.iterator(&self) -> ArrayIterator<T> { ... }
```

`Self.Iter` reuses `Self` and ordinary member access rather than
introducing new syntax.

The reason this is the right fix rather than an optimisation: the
concrete iterator type is now **part of what the conformance promised**.
A caller may name it, and a later backend may rely on it, without
anything being inferred behind a signature's back. Compare the
alternative — having the compiler notice that `iterator()` only ever
returns one type and specialising accordingly — which would make a
signature mean something narrower than it says. That is the line this
document exists to hold.

It is still worth flagging that the *motivation* for looking at
`Iterable` at all was performance, which makes associated types the
first feature in the design arrived at that way.

## What is not decided here

Deliberately deferred to the backend design, once the language is
settled:

- **Representation.** Monomorphisation, uniform boxing, or a mixture.
  Uniform boxing is better for code size and compile coupling;
  monomorphisation is better for run time. Both implement the rule
  above, which is why this can wait.
- **Devirtualisation.** Permitted in principle at any site, because it
  has no semantic consequence — an existential's type is unnameable
  either way, so proving it changes no program's meaning. How much is
  provable depends on the next item.
- **Whole-program versus separate compilation.** Whether the compiler
  always sees every package's source, or a dependency can ship
  precompiled. This is a bigger fork than it looks: it also governs
  what `Package.lock` pins, and how much a backend can know.
- **Static versus shared linking**, per target.

## Open

- **Associated type bounds.** `type Iter: Iterator<T>` names one
  interface; whether the `+`-joined form is permitted there too.
- **The type switch.** Matching an existential by concrete type. The
  corpus promises it and the first table row above depends on it.
  Belongs with `error-handling.md`.
- **Whether an existential site should be lintable.** A
  `boxed_interface_value` lint would make the three categories above
  visible on demand, in the spirit of `diagnostics.md` — though the
  name presumes a representation, so it wants renaming to something
  like `dynamic_type`.
- **Can an existential appear in a generic bound's position by
  mistake?** `fn f<T: Error>` and `fn f(e Error)` are different
  functions and read similarly. Whether that needs a diagnostic.
