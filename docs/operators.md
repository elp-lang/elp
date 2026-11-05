# elp operators

## Operators are interface methods

`a + b` is `a.add(b)`. Every operator except the short-circuiting ones
desugars to a method on a named interface, so a user type gets `+` by
declaring a conformance.

This resolves the open item in `generics.md`: the `base + emphasis`
style composition that was already decided is **not** a styling special
case — a `StyleSheet` implements `Add` like anything else.

The alternative, built-in operators over known types only, would keep
`Money.plus` as a method forever. The Expenses probe exists partly to
show what that costs: `sum = sum.plus(expense.amount)` in a loop, in a
language where the obvious spelling is unavailable for a type whose
entire purpose is arithmetic.

### Three separable powers, two of them illegal

"Can users define operators?" is three questions. Separating them is
what makes the answer defensible.

| power | legal? |
|---|---|
| 1. Invent an operator **symbol** — `<=>`, `\|*\|` | **no** |
| 2. Define a new **interface the compiler binds to an operator** — add a row to the table below | **no** |
| 3. **Implement** one of the table's interfaces for your own type | **yes** |

**(1)** is Rust's position rather than Swift's. Custom operators move
the *grammar* into user code: precedence, associativity and tokenisation
all become package-dependent.

**(2)** means the operator table is **closed and owned by the stdlib**.
The operator-to-interface mapping is compiler-known machinery and a user
package cannot extend it — there is no way to declare
`interface Shift { fn shl(...) }` and have `<<` start calling it. The
*mapping* is the internal API; the interfaces it names are ordinary
public interfaces, which is what makes (3) possible.

**(3)** is the whole point, and it is what makes `Money + Money` work.
The orphan rule is what keeps it safe — see below.

### Why this is not C++

The hazard in C++ is real but it comes from five specific mechanisms,
four of which are structurally absent here, each by a decision taken for
other reasons.

| C++ hazard | why it cannot happen |
|---|---|
| `operator<<` for streams — an operator meaning anything at all | each operator maps to one interface with a **fixed signature**. `+` is always `(&Self, &Self) -> Self`; it cannot be reshaped, and bitwise operators do not exist to abuse |
| anyone defining `operator+(Money, Money)` in any namespace, found by ADL | the **orphan rule** (`interfaces.md`). `Add` for `Money` is declarable only in `Money`'s package, so there is exactly one `+` for a type in the entire program — no overload set, nothing to resolve |
| implicit conversions silently selecting a surprising overload | **no implicit numeric conversion**, and no inference-driven `into()`. Nothing converts at an operator site |
| overloading `&&`, `\|\|`, `,` and losing short-circuiting | those are **fixed forms, not interfaces**, precisely so short-circuiting cannot be taken away. There is no comma operator, and assignment is a statement |
| user-declared precedence groups | the table is fixed |

What remains, stated plainly: a type can implement `Add` that subtracts,
and `+` can hide expensive work. No type system prevents either — it is
the same exposure as a `toString` that lies. Convention and a lint,
not a guarantee.

## The interfaces

`interfaces.md` already defines `Equatable` and
`Comparable: Equatable`. The rest follow their shape — method only,
`&self`, `&Self` for the operand.

| operator | interface | method |
|---|---|---|
| `==`, `!=` | `Equatable` | `fn equals(&self, other &Self) -> bool` |
| `<`, `<=`, `>`, `>=` | `Comparable` | `fn compareTo(&self, other &Self) -> Ordering` |
| `+` | `Add` | `fn add(&self, other &Self) -> Self` |
| `-` (binary) | `Sub` | `fn sub(&self, other &Self) -> Self` |
| `*` | `Mul` | `fn mul(&self, other &Self) -> Self` |
| `/` | `Div` | `fn div(&self, other &Self) -> Self` |
| `%` | `Rem` | `fn rem(&self, other &Self) -> Self` |
| `-` (unary) | `Neg` | `fn negate(&self) -> Self` |
| `a[i]` | `Index<K, V>` | `fn at(&self, key &K) -> V` |

**Naming follows a rule the codebase already keeps**: an interface
describing *a property you can test* is an adjective — `Equatable`,
`Comparable`, `Hashable`, `Sendable`. One describing *an operation you
perform* is the verb — `From`, `To`, `Add`, `Sub`. Both forms are
already in use and each reads correctly in its own position.

### Operands are `Self` — there is no `Mul<Rhs>`

Every arithmetic interface takes `other &Self`. So `Money + Money`
works and **`Money * 2` is not expressible at all**.

This is stricter than Rust, which parameterises the right-hand side.
Parameterising would buy mixed-type arithmetic, but it reopens the part
of C++ worth avoiding: one type acquires several `*`s and a reader has
to work out which one applies at each site.

Scaling is a named method instead:

```
price.scaledBy(2)
```

which is the better spelling anyway — multiplying money by money is
meaningless, so `*` was never the right operator for the operation
people actually want.

### `Ordering` is a three-valued enum

```elp
enum Ordering {
	LESS,
	EQUAL,
	GREATER,
}
```

`compareTo` returns one of three values rather than an `int`, so a
comparator cannot return a fourth thing and `match a.compareTo(b)` is
exhaustive like every other enum match in the language. The C
convention of "negative, zero, positive" admits four billion values
where three are meaningful, and costs a reader the question of whether
the sign convention is the one they remember.

### Four comparisons, one method

`<`, `<=`, `>` and `>=` all route through `compareTo`. They are not
separately implementable, which makes it impossible to write a type
where `a < b` and `b >= a` disagree.

`!=` is likewise always `!(a == b)` and never separately defined.

## What is not overloadable

`&&`, `||`, `!` and `??` are **fixed forms, not interface methods**.

The reason is short-circuiting. A method call evaluates its argument
before the call happens, so `a && b` as `a.and(b)` would evaluate `b`
unconditionally — which is not what `&&` means, and would break every
guard in the corpus. `??` is the same: `x ?? expensive()` must not call
`expensive()` when `x` is non-nil.

So:

- `&&`, `||` take `bool` on both sides and produce `bool`.
- `!` takes `bool`.
- `??` takes `Optional<T>` on the left and `T` on the right.

A type cannot opt into any of them. Bitwise `&`, `|`, `^`, `<<`, `>>`
are **deferred entirely**, and two of them have known syntactic
conflicts: `&` is the binding sigil at call sites, and `generics.md`
specifies that `>>` is never lexed as one token.

## `==` only — there is no `===`

Reference identity gets no operator.

`==` is `Equatable.equals`, nominal and explicit, so there is no
coercion to guard against — the JavaScript problem that `===` exists to
solve does not arise. And the one place in elp where identity genuinely
matters, list reconciliation, already has a dedicated mechanism:

```
for message in messages by message.id {
```

`by` is **better** than reference identity for that job, because it
survives a value being copied, which — given that collections hold
values and props cross as copies or shares — is the normal case rather
than the exception.

If pointer identity is ever needed for machinery, it is a method on the
handle (`Rc.pointsAt(a, b)`), not an operator. That keeps it rare,
visible and searchable.

## Precedence

Tightest first. Standard C-family ordering, which parses every
expression in the probes correctly.

| | operators | associativity |
|---|---|---|
| 1 | `.` `()` `[]` | left |
| 2 | `!` `-` (unary) | right |
| 3 | `*` `/` `%` | left |
| 4 | `+` `-` | left |
| 5 | `..` | none |
| 6 | `??` | right |
| 7 | `<` `<=` `>` `>=` | left |
| 8 | `==` `!=` | left |
| 9 | `&&` | left |
| 10 | `\|\|` | left |

`??` binds tighter than comparison so that `count ?? 0 > 5` means
`(count ?? 0) > 5`, which is the only useful reading. It is
right-associative so `a ?? b ?? c` chains.

`..` is non-associative: `a..b..c` is a syntax error rather than
something to memorise.

**Assignment is not in the table.** `=`, `+=` and friends are
statements, not expressions, so `if x = 5` cannot be written at all.

### Mixed `&&` and `||`

`a && b || c` parses, by the table. Whether it should be *written* is a
lint — `mixed_logical_without_parens`, warn by default — rather than a
parse error. Precedence is defined; relying on readers to remember it is
the part worth flagging.

## Compound assignment desugars

`a += b` is `a = a + b`, requiring `Add`. There is no separate
`AddAssign`.

One interface rather than two is the simpler rule, and it is consistent
with collections holding values: there is no in-place mutation to
optimise for, because a value read out of a collection is already a copy.

## No implicit numeric conversion

`int8 + int` is an error. Mixed-width arithmetic is written out.

This is the same position as mandatory type arguments and the rejection
of inference-driven `into()` — elp does not silently change a value's
type to make a call site work. The numeric tower itself (which widths
exist, overflow behaviour, literal defaulting) is a separate design.

## Narrowing through `&&` and `||`

Short-circuiting is what makes optional narrowing work, and the probes
depend on it. `Expenses/internal/stats.elp`:

```
if biggest == nil || expense.amount.cents > biggest.amount.cents {
```

For the right operand to typecheck, the compiler must know `biggest` is
non-nil **because the left operand was false**.

The rules:

- `x == nil` and `x != nil` narrow `x` in both branches of an `if`.
- In `a && b`, `b` is checked under `a`'s true-narrowing.
- In `a || b`, `b` is checked under `a`'s false-narrowing.

### Why this is sound for reactive `var`s too

Narrowing a local is obviously safe. Narrowing a component `var` looks
unsafe, because a `var` is a cell and something else could write it
between the test and the use.

Nothing else can. **Cells are confined to one logical thread and a turn
is not preemptible** (`reactivity.md`), so no other code interleaves
with straight-line code in the same turn. The only thing that can
invalidate a narrowing is this code writing the cell, which the compiler
sees.

This is the third thing thread confinement has paid for, after the
user-written re-entrancy guard and check-then-act atomicity. It was
adopted because all three platforms force it anyway.

## Open

- **`Index` and assignment.** `a[i]` reads via `Index`; `a[i] = v` has
  no interface here, and under "collections hold values" it may not want
  one.
- **Operator conformance and the orphan rule.** `Add` for a foreign type
  is already forbidden by the orphan rule, which is the right answer and
  only needs restating where people will look for it.
- **Numeric tower**: widths, literal defaulting, overflow.
