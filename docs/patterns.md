# elp patterns and destructuring

## There is no tuple type

`(a, b)` is a **`match` scrutinee form**, not a value. It cannot be
returned, stored, named, or written in a type position.

The corpus uses the shape twice, and both times it is destructured on
the spot and discarded:

```elp
// LinkShortener/main.elp — the route table
return match (req.method, req.path) {
	(.GET,  "/")       -> Response.text(.OK, "link shortener")
	(.POST, "/links")  -> createLink(req, store)
	(.GET,  path)      -> followLink(path, store)
	_                  -> Response.text(.METHOD_NOT_ALLOWED, "method not allowed")
}
```

```elp
// Expenses/components/entry-form.velp — parsing "12.34"
2 -> match (parts[0].toInt(), parts[1].padEnd(2, "0").toInt()) {
	(nil, _)           -> nil
	(_, nil)           -> nil
	(whole, fraction)  -> Money { cents = whole * 100 + fraction }
}
```

There is no `.0`/`.1` anywhere, no tuple in a type position, no tuple
return, and nothing stored in a field. `collections.md` had already
chosen a concrete `Entry<K, V>` over a tuple for Map iteration.

Three reasons this is the right refusal rather than a lucky one:

- **Ordinal members contradict the principle overloading rests on.**
  `generics.md`: *"Parameter names are part of a signature because a
  signature is an explicit contract."* A type whose members are
  addressed by position is the inverse of that, and it would be the one
  place in the language where a name is unavailable.
- **It would be the second anonymous aggregate.** `object{name string}`
  already exists, and `generics.md` flags structural typing as its own
  undesigned area. Two overlapping features for "a few values with no
  name" is one too many.
- **The orphan rule has no home for it.** Nobody owns `(A, B)`, so
  `(A, B) implements Equatable` cannot be declared anywhere legal — and
  it would need one declaration per arity, meaning variadic generics or
  a compiler special case. Both are large; neither buys anything the
  corpus wants.

**What to use instead:** `object{...}` for an ad-hoc aggregate, or a
named type. There are no multiple return values; `Either<T, E>` covers
success-or-failure and a named type covers the rest.

## Parallel match

| | |
|---|---|
| **arity** | every pattern in a parenthesised match has the scrutinee's arity; a mismatch is a type error |
| **evaluation** | all scrutinees evaluated left to right, exactly once, before any pattern is tried — `parts[0].toInt()` is a call, not re-run per arm |
| **order** | arms top to bottom, first match wins |
| **exhaustiveness** | over the product of the scrutinees |
| **bare `_`** | permitted as a whole-list catch-all. `LinkShortener` writes `_`, not `(_, _)`, and `collections.md` made `_` the only catch-all |
| **nesting** | the parenthesised list exists at one level only, at the top of an arm. There is no nested `((a, b), c)`, because there is no tuple value to nest |

Evaluating every scrutinee up front rather than lazily per arm is the
only defensible rule once a scrutinee can be a call. It also means
arity is fixed before matching begins, which is what makes the arity
check a plain type error.

`diagnostics.md`'s `unreachable_pattern` catches an arm shadowed by an
earlier one, which is the cost of first-match-wins and is worth paying
for the readability of the route table above.

## Destructuring is named, never positional

A **record pattern** binds fields by name:

```elp
{ key, value }                      // bind fields key and value to key and value
{ key as category, value as amount } // rename while binding
```

`as` is the rename token because `as` already means exactly that in
`import { Text as IText }` (`name-resolution.md`). One keyword, one
meaning, two places. Using `=` would have overloaded the
record-literal's `field = value`, where the right-hand side is an
expression rather than a binding name.

The shorthand mirrors construction. `SocialFeed/internal/api.elp`
already writes `object{body, image = imagePath}`, where bare `body`
means `body = body`; a record pattern's bare `key` is the same
abbreviation read in the other direction.

**A record pattern binds what it names and matches anything else.** It
is not an exhaustive field list, so `{ cents }` matches any `Money`.
There is no `..` rest-marker to write, because naming the wanted fields
is already the opt-in.

### Where it is allowed

| position | |
|---|---|
| a `for` binding | **yes** — `for { key, value } in totals` |
| a `match` arm pattern | **yes** — it is one of the pattern productions below |
| a `const`/`var` binding | **yes** — `const { producer, consumer } = Ring<Event>.of(256)`. See "Destructuring an owned aggregate" |
| a function parameter | **no.** A signature is an explicit contract and its parameter names are part of it (`generics.md`). Destructuring there would hide the contract behind the implementation's convenience |

### This resolves `for k, v in map` without a special case

`collections.md` recognised `for category, amount in byCategory(...)` as
"destructuring sugar over `Entry`" and left open whether two-binding
`for` generalises beyond `Entry`.

It generalises completely, and for free: `for { key, value } in xs` is
record destructuring applied to whatever the iterator yields. `Entry`
was never special — it just happens to have those two fields. The
corpus site migrates:

```elp
for { key as category, value as amount } in byCategory(expenses) { ... }
```

Longer than the positional form, and that is the trade the ruling makes:
the reader no longer has to know `Entry`'s field order to know which
binding is which.

## Destructuring an owned aggregate

`const { producer, consumer } = Ring<Event>.of(256)` is the forcing case
(`aliasing.md`): one constructor hands back two endpoints, and splitting
them is what makes a lock-free queue's single-producer precondition a
property of the types rather than a line in its documentation.

The worry it raises is not the binding form, it is **what happens when the
aggregate gains a field upstream.** A library adding a third field to
`Ring<T>` must not cause consumer code to start silently discarding it.
Three rules, and together they make the hazard absent rather than
diagnosed:

**1. Destructuring an owned aggregate is exhaustive.** Every field is
named or explicitly `_`. So the upstream change is a compile error at
every destructuring site, which is the signal that was wanted. This is the
same posture as `collections.md` rejecting `..` as a catch-all and leaving
`_` as the only one.

**2. Destructuring never destroys; it transfers.** Each bound field is
owned by its binding afterwards. Nothing is implicitly dropped, so there
is no "silently discards" case left to warn about.

**3. `_` on a `Scoped` field is an error.** Binding it would owe a
release, and `_` cannot discharge one:

```
`cleanup` must be released; `_` cannot discharge a `Scoped` field
```

### It is not reflection, and the direction is why

Field-name destructuring reads like code enumerating a type's shape, which
is what `macros.md` confines to compile time and keeps out of userland.
The distinction is the direction information flows: **reflection asks a
type what it has; a pattern states what you expect and is checked.**
`@CodeGen` enumerates fields it has never seen and produces declarations
from them. A pattern produces nothing — it names fields the author
believes are there, and fails to match if they are not. Rule 1 is what
makes that failure loud.

## Optional narrowing by binding

`entry-form.velp` does not typecheck without this, and the arity-1
match directly above the parallel one needs the same rule:

```elp
match parts[0].toInt() {           // Optional<int>
	nil    -> nil
	whole  -> Money { cents = whole * 100 }   // whole is int, not Optional<int>
}
```

**A binding pattern on an `Optional<T>` whose `nil` arm is already
covered binds `T`.** It generalises to the product for free — that is
what makes `(whole, fraction)` in the parallel match above bind two
`int`s after `(nil, _)` and `(_, nil)` have been handled.

This is the same flow-sensitive machinery `operators.md` settled for
narrowing through `&&` and `||`, and inside one turn it is sound for
the same reason: `reactivity.md`'s cell thread confinement means
nothing interleaves with straight-line code.

**Binding is also the only form that survives a yield point**, which is
why it is the primary mechanism and not a convenience. A bound pattern
produces a **local**, and no other turn can write a local — whereas an
in-place narrowing of a cell is invalidated by the first call in the
narrowed scope, since elp has no function colouring and so no marker for
where a turn ends. `operators.md` carries the amendment; the consequence
here is that `match cell { nil -> ...; x -> ... }` is always correct,
while `if cell != nil { ... }` is correct only up to the first call.

## A variant payload is a record

Enum payloads were the one place ordinals survived the ruling above, and
they no longer do. **A variant's payload fields are declared with
names:**

```elp
export enum StoreMsg {
	PUT { link Link, reply Channel<Either<Link, Error>> },
	RESOLVE { code string, reply Channel<Either<Link, Error>> },
	LIST { reply Channel<[Link]> },
	SHUTDOWN,
}
```

The field syntax is `name Type`, the same as `object{name string}` and an
object declaration's fields. A variant with no payload keeps no braces.

Construction and destructuring then both follow the **record** rules,
not the function-call rules, because a payload is an aggregate literal
rather than an argument list — `Money { cents = 100 }` already requires
names and `Money { 100 }` is not legal:

```elp
self.ask<Either<Link, Error>>(fn (reply) { .PUT { link, reply } })   // construct
.PUT { link, reply } -> { ... }                                      // destructure
```

Those two lines are **the same text**, which is the payoff. The bare
field `link` means `link = link` on the way in and binds `link` on the
way out, exactly as `object{body}` and a record pattern already do in
each direction. `as` renames when destructuring.

### Arity 1 keeps the parenthesised form

```elp
Ok(post) -> { ... }
.NOT_FOUND(code) -> "no link with code {code}"
dispatch(.LOADING(true))
```

This is not a terseness exemption. **The hazard the ruling exists to
prevent requires two fields to exist at all** — with one field there is
no order to misread, and no ordinal to get wrong. `ChatRoom`'s
`NEW_MESSAGE { userName string, body string }` is the case that makes
the point: two `string`s, so the old `.NEW_MESSAGE(userName, body)`
could be written backwards and produce no type error and no diagnostic,
only a message attributed to the wrong person.

`Ok`/`Err` are the reason this matters more than it looks. They are
arity-1 variants of `Either`, destructured about twenty times in the
corpus, and requiring `Ok { value as post }` would have been a bad trade
for a hazard that cannot occur there.

### What it also buys

`BlogStyle/internal/api.elp` notes that `ApiErrorType` has a
`to<JSON>` generated for it and serialises to `".SERVER("some
error")"`. An unnamed payload gives a JSON encoder no key to use, so
the generated shape has to be invented. `SERVER { message string }`
makes it fall out.

## `match` is an expression, and an arm produces a value

`match` and `if` both produce values. (`if` was settled earlier, when
a dependency's `condition = if target == .iOS` showed it reading better
than a nest of statements.)

An arm is either an expression or a block, and **a block arm's value is
its final expression**:

```elp
return match self.done.pop() {
	nil -> false
	edit -> {
		edit.revert(&var doc)
		self.undone.append(edit)
		true
	}
}
```

### Why `return` is not what produces it

This looks like it contradicts the rule that a function body always
needs an explicit `return` (`functions.md`). It does not, and the
reason is worth stating because the asymmetry is **forced rather than
chosen**:

**`return` exits a function.** An arm does not exit a function — it
produces a value in place, and execution carries on with whatever
consumes the `match`. So `return` could never mean "this is the arm's
value" without becoming ambiguous with the `return` that really does
leave the function:

```elp
const label = match state {
	.DONE { output } -> output
	_ -> {
		Console.log("unknown state")
		return "unreported"      // leaves the enclosing fn; `label` is never bound
	}
}
```

A function body is a **statement** block, so it needs a keyword to hand
a value back. An arm is already in **expression** position, so it needs
none. Two different constructs, one rule each.

### The rules

| | |
|---|---|
| arm forms | `-> expr` or `-> { statements; expr }` |
| a block arm's value | its final expression; nothing if the block ends in a statement |
| `return` / `throw` in an arm | exits the **enclosing function**, skipping the rest of the `match` |
| a diverging arm | `return`, `throw` or a non-returning call imposes no type constraint |
| arm agreement | every non-diverging arm produces the same type |
| a `match` used as a statement | legal; its value is discarded |
| loops | statements, never expressions — there is no `break`, so no loop value |

The diverging-arm rule is what makes this work:

```elp
return match onSubmit {
	nil      -> throw LoginError.NO_HANDLER
	callback -> callback(email, password)
}
```

The `nil` arm never produces anything, so it constrains nothing and the
`match`'s type comes from the second arm alone.

A value-producing `match` in statement position is allowed because
`store.elp`'s message loop is exactly that — arms run for effect and
produce nothing. A `match` whose arms *do* produce a value that nobody
consumes is a candidate for an `unused_value` lint
(`diagnostics.md`), not an error.

## The pattern grammar

```
arm            := scrutinee-list "->" body
scrutinee-list := "(" pattern ("," pattern)+ ")"
                | pattern

pattern        := literal                              // 1, "ok", true
                | "nil"
                | "_"
                | range                                // 400..499
                | IDENT                                // a binding
                | "." IDENT                            // .GET
                | "." IDENT "(" pattern ")"            // .NOT_FOUND(code) — arity 1 only
                | "." IDENT record                     // .PUT { link, reply }
                | record

record         := "{" field ("," field)* "}"
field          := IDENT
                | IDENT "as" IDENT
```

Collected here because it was previously spread across
`error-handling.md` (interface matching, catch-all rules),
`collections.md` (`_` over `..`, ranges as patterns) and this document.
Those remain the place each *rule* is argued; this is only the shape.

Two rules from elsewhere that the grammar does not show:

- A `match` on an **interface**-typed value always needs a catch-all,
  because the conformance set is open (`error-handling.md`).
- A `match` on an **enum** may omit one, and `Category.label` is the
  case that makes writing every arm worth it (`error-handling.md`).

## Open

- ~~**Destructuring in `const`/`var`.**~~ **Resolved** — yes, with the
  three rules in "Destructuring an owned aggregate". `aliasing.md`'s
  `Ring<T>` endpoints supplied the corpus evidence that was missing.
- **Nested record patterns.** `{ amount as { cents } }` and the like.
  Deliberately not in the grammar above; a field binds to a name, not
  to a sub-pattern. Whether that needs to change has no corpus
  evidence either way.
- **The type switch.** Matching an interface-typed value by concrete
  type, which the corpus promises and `dispatch.md`'s existential rule
  depends on. Still undesigned (`error-handling.md`).
