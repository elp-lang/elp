# elp memory model — how it elaborates to Rust

Companion to the front-end roadmap's Phase 1. The decision already made
(see the roadmap): elp's typechecker tracks ownership/move/borrow *shapes*
only to decide what Rust to emit — it does not reimplement lifetime
inference or NLL. A bug in elp's tracking surfaces as a **rustc compile
error on generated code**, not a shipped memory-safety hole. This document
is the elaboration mapping that decision produces, validated by hand-porting
a real file rather than reasoning about it in the abstract.

Companion to `interfaces.md`: whether a value can be captured by value at
all (Finding 3, below) turned out to hinge on the general interface model
— `Copy`/`Move` are ordinary, nominally-checked interfaces, not
memory-model-specific magic. Read that doc for the interface story; this
one stays focused on what elaborates to what.

## The exercise

Source: `examples/ChatRoom/components/login-form.velp`, in full — chosen
because it's the one real file in the corpus that actually exercises `&`
(`&email`, `&password`) rather than just declaring `&type` in a signature
nobody calls.

```elp
export fn LoginForm -> Component {
	export var email Optional<string>
	export *fn onSubmit(username string, password Protected<string>)

	var password Protected<string>
	var attempts = 0

	fn handleSubmit() {
		attempts += 1
		if onSubmit.isNil() {
			throw LoginError.NO_HANDLER
		}
		onSubmit(email, password)
	}

	return Form(onSubmit=handleSubmit) {
		EmailInput(&email, placeholder="Enter your email")
		SecureTextInput(&password, placeholder="Enter your password")
		Button(type=.FORM_SUBMIT) { Text("Log in") }
	}
}
```

Hand-elaborated:

```rust
pub struct LoginForm {
    // Exported prop. `var` inside a component is reactive (the source
    // comment says so directly: "declaring vars in a component... cause
    // reactivity") — see "Finding 2" below for why that means `Reactive<T>`,
    // not a plain field.
    pub email: Reactive<Option<String>>,
    // `*fn` prop — see "Finding 4": still nilable at runtime despite being
    // marked as a required callback, so `Option` either way. `Arc`, not
    // `Box` — see Finding 3: a closure isn't `Copy`/`Move` on its own, so
    // it can't be held directly; `Arc` is what makes holding one legal.
    pub on_submit: Option<Arc<dyn Fn(String, Protected<String>)>>,

    // Private reactive state — same treatment as `email`.
    password: Reactive<Protected<String>>,
    attempts: Reactive<i32>,
}

impl LoginForm {
    fn handle_submit(&self) -> Result<(), LoginError> {
        self.attempts.update(|a| a + 1);

        if self.on_submit.is_none() {
            return Err(LoginError::NoHandler);
        }

        (self.on_submit.as_ref().unwrap())(self.email.get(), self.password.get());
        Ok(())
    }

    fn render(&self) -> Component {
        Form::new(|| self.handle_submit())
            .child(EmailInput::new(&self.email).placeholder("Enter your email"))
            .child(SecureTextInput::new(&self.password).placeholder("Enter your password"))
            .child(Button::new(ButtonType::FormSubmit).child(Text::new("Log in")))
    }
}
```

(`throw` → `Result`/`Err` is now a real decision, not just an illustration
— see `error-handling.md`: `throw` is sugar for `return Err(...)`, no
bubbling, no boundary. The builder-block calls → method-chained
`.child(...)` calls are still just illustrating *some* plausible lowering
so the ownership-relevant parts have real code around them; `Form`'s
actual runtime shape is a separate, later design question.)

## What confirmed cleanly

- `fn handleSubmit()` / `fn render` (implicit) → both become plain `&self`
  methods. No `&mut self` needed anywhere, which is not obvious up front —
  see Finding 2 for why.
- `export *fn onSubmit(...)` → `Option<Arc<dyn Fn(...)>>` on the struct.
  `.isNil()` → `.is_none()`, direct. The `Arc` isn't a component-specific
  detail — see Finding 3: it's the general mechanism for holding *any*
  function value at all.
- Calling `handleSubmit` as a value (`Form(onSubmit=handleSubmit)`) →
  a `&self`-capturing closure, once handleSubmit only needs `&self` (which
  it does, per Finding 2). This was the thing most likely to *not* work —
  a method needing `&mut self` cannot be closed over from a `&self`
  context — and it resolves itself once the state layer is right.

## Finding 1 — plain (non-reactive) `var`/`const` mutability, resolved

Every `var`/`&` in the login-form exercise is either a reactive component
field or a reference *to* one, so this was originally flagged as
unexercised. It's resolved now, not by a new example, but by taking
`var`/`const` at face value: `var` means mutable, `const` means it never
changes, full stop — nothing about *how* it's referenced needs to add
anything on top of that. `var x` → `let mut x`, `const x` → `let x`. No
further annotation is needed at the binding site because the binding
already says everything there is to say about mutability.

`const` buys stack allocation of the binding and rules out any
reallocation-through-mutation of it, but it isn't a blanket "always
stack, never heap" guarantee — a type that's heap-backed by its own
construction (a growable collection, say) is still heap-backed when
`const`-bound. What changes is that nothing can grow/reallocate/mutate it
afterward, which is most of the stack-allocation argument, not literally
all of it.

This finding used to end by flagging `&x` → `&x`/`&mut x` as an
assumed-but-unconfirmed mapping, to be revisited with a synthetic example
before Phase 2 relied on it. That's no longer needed — see Finding 4
below, which derives the `&`/`&mut` choice directly from `var`/`const`
plus usage, closing the gap without a synthetic example being required.

## Finding 2 — reactive `var` needs interior mutability, not a plain field

This is the load-bearing discovery. `var` fields *inside a component body*
are documented as reactive ("observed for changes... cause reactivity").
The naive elaboration — a plain struct field, mutated via `&mut self` —
breaks immediately: `render` needs to hand `EmailInput` something it can
write through as the user types, and `handleSubmit` needs to increment
`attempts`, but `render` also needs to hand out `handleSubmit` *itself* as
a callback (`Form(onSubmit=handleSubmit)`). A `&mut self`-requiring method
cannot be closed over from inside a `&self` method — there's no `&mut self`
available to capture. Something has to give, and it can't be "make render
take `&mut self`" — that doesn't fix `handleSubmit` also needing exclusive
access at the same time `render` is calling it as a value.

The resolution: reactive fields are wrapped in an interior-mutability cell
(`Reactive<T>` above — a stand-in name; the real wrapper is a runtime-design
question, not this document's). Once mutation lives *inside* the cell
(`self.attempts.update(...)`) rather than at the struct-field level, both
`render` and `handleSubmit` only ever need `&self`, and the closure-capture
problem disappears because there's no `&mut self` requirement left to
conflict with anything.

**Consequence for Phase 2 (typecheck):** ownership-shape tracking needs two
modes, not one:
- Plain `.elp` script-level `var`/`const` → ordinary Rust `let
  mut`/`let`, real borrow-checking, exactly as the roadmap assumed.
- Reactive `var` fields inside a `.velp` component body → interior
  mutability, `&self`-only methods, no real Rust borrow-checking of the
  *reactive value itself* (the cell absorbs it) — a different lowering
  rule keyed off "is this binding inside a component and therefore
  reactive," not a smaller version of the same rule.

This isn't a reversal of the "mirror Rust ownership" decision — plain
script code still elaborates directly and rustc still backstops it. It's
a scope correction: that decision covers *script*-level bindings, and
reactive *component* state needs its own elaboration rule sitting next to
it, not underneath it.

## Finding 3 — functions are reference-only values, and `Copy`/`Move` are what govern capturing *anything* by value

Started from a narrower question — "how does `LoginForm` hold a reference
to a closure owned by its parent" — and generalized further than expected.

The starting rule: you can't pass a function by value. A closure is never
moved or copied directly; holding one always means holding a reference to
somebody else's. `Option<Box<dyn Fn(...)>>` (the original draft above)
is wrong under that rule — `Box` *owns* the closure, so constructing a
`LoginForm` would have to move a closure into it, which is exactly the
"pass by value" the rule rules out.

The fix isn't a bespoke "functions are special, always borrowed" rule,
though — it falls out of a more general one, and this is the part worth
stating precisely: **elp has no capture-list syntax, and no per-variable
move/reference/copy annotation at the closure site at all.** Instead,
whether a value can be captured into a closure (or passed by value,
generally) is governed entirely by whether its *type* implements `Copy`
or `Move` — two ordinary interfaces (see `interfaces.md`), checked
nominally, no interface-level magic. A closure/`dyn Fn` type implements
neither by default (its captured internals are opaque), so it can't be
captured or passed directly — the same rule that applies to any other
type that hasn't opted in to `Copy`/`Move`, not a special case for
functions.

`Arc<dyn Fn(...)>` is the way out, and it's not a function-specific
mechanism either: `Arc<T>` implements `Copy` (cloning it is a cheap
refcount bump, not a deep copy of `T`), so wrapping a closure in `Arc`
gives you something that *is* legally capturable/storable. `Reactive<T>`
(Finding 2) is expected to work the same way — its own `Copy`
implementation clones the underlying `Arc`/`Rc` handle rather than
deep-copying the value, which is what lets a reactive field be captured
into a closure without a lifetime parameter appearing anywhere. One
mechanism (type-level `Copy`), not two.

`Arc` specifically, not `Rc` — elp's concurrency model is Go-style spawned
coroutines (see `grammar-script-notes.md` gap #1), so reactive state and
captured callbacks need to be sound across concurrent access; `Rc`/
`RefCell` aren't thread-safe and would be the wrong choice even though
they're cheaper.

**Consequence for Phase 2:** ownership-shape tracking needs a "does this
type implement `Copy` or `Move`" check wired in wherever a value is
captured into a closure or passed by value — this isn't a separate
closure-specific rule, it's the same interface-conformance check Phase 2
needs for `Copy`/`Move` generally, just applied at every capture/pass-by-
value site rather than only at explicit-copy call sites.

**Superseded by this finding:** an earlier version of this discussion
proposed a `move`-keyword capture-list syntax (C++-style, per-variable
move/reference/copy annotations at the closure site) as the mechanism.
That's dropped — it's not needed once capturability is a property of the
*type* rather than the *capture site*, and it would have been redundant
with the `Copy`/`Move` interface check besides.

## Amendment to Finding 3 — superseded by `aliasing.md`

Finding 3 concludes that `Arc<T>` and `Reactive<T>` implement `Copy` and
calls this *"one mechanism (type-level `Copy`), not two."*

**That economy is false, and `examples/Buffer` was written to prove it.**
A split-pane editor needs both semantics in one program, and under
Finding 3's rule both are spelled the same way:

```elp
Cursor implements Copy      // duplicates are INDEPENDENT
Buffer implements Copy      // duplicates ALIAS the same cell
```

The full design, the corpus evidence, and the three drafts rejected on
the way to it are in **`aliasing.md`**, which is authoritative. Only the
corrections to *this* document are recorded here.

### What it changes in Finding 3

- `Reactive<T>` is a lowering name, not an elp type. The user-facing
  aliasing types are `Shared<T>` (immutable, `Copy`), `Confined<T>`
  (mutable, one thread) and `Synchronized<T>` (mutable, cross-thread).
- The last two are **`Move`, not `Copy`**, and a second handle comes
  from `.alias()`. So a type holding one is not `Copy`, transitively,
  by the ordinary field-wise rule and not by a special one.
- `Duplicate` is a third conformance alongside `Copy` and `Move`,
  naming the `.copy()` the corpus has used since the beginning. It
  promises **independence**, which is why the aliasing handles are not
  in it: `.copy()` never aliases and `.alias()` always does.
- Finding 3's claimed benefit survives in a better form: a capture or
  pass-by-value site still asks **one** question — "is this type
  duplicable" — and the *type* now carries whether duplicates alias
  instead of hiding it in a conformance.

### What it confirms in Finding 6

Finding 6 put references in a free tier: *"a reference (`&T`, for any
`T`) is unconditionally cheap to copy… the same `Arc<T>`/`Reactive<T>`
pattern — a cheap-to-copy handle whose own copyability is entirely
independent of whether the thing it points to is copyable."*

`Shared<T>` is exactly that rule given an owning form — `&T` with a
refcount instead of a lifetime — so it is `Copy` for any `T`, including
`Shared<[string]>`. Finding 6 generalised correctly; it just had no type
to name.

Where Finding 6 was too broad is the `Arc`/`Reactive` half of that
sentence: a *mutable* handle is cheap to duplicate and the duplicate is
**not** independent, so it belongs in `Duplicate` rather than the free
tier. The reference half is unaffected, because a `&T` cannot be written
through without `&var`.

## Finding 4 — `&x` doesn't need a mutable/immutable spelling, resolved

elp's grammar has one ref-type spelling: `&type`, `&self`, `&ident`. Rust
has two: `&T` and `&mut T`. For the reactive fields in the login-form
exercise, that was never a gap: `EmailInput(&email, ...)` elaborates to
`&self.email` — a plain shared reference to the `Reactive<T>` cell —
because writes go through the cell's own API, not through a Rust-level
`&mut`. The open question was whether that generalizes to plain,
non-reactive values, where there's no cell absorbing the mutation.

It generalizes, and the single spelling turns out not to be hiding any
real ambiguity — `var`/`const` (Finding 1) already supplies the half of
the answer that isn't pure usage:

- If the referent is `const`, the reference can only ever elaborate to
  `&T`. There's no case where `&mut T` is legal, because nothing can
  mutate through it — the binding forbids that outright, so there's
  nothing to disambiguate.
- If the referent is `var`, the reference elaborates to `&T` or `&mut T`
  chosen per call site, by whether the callee actually mutates through
  that specific reference — the same way Rust's own borrow checker
  already decides `&`/`&mut` from usage rather than from a keyword
  written at the reference site. elp never needed a `&mut` spelling
  because that choice was always fully determined by what the code
  *does*, not something a developer needed to declare.

The enforcement side of this is what makes it more than a convenient
default: passing a `const` binding into a function whose body mutates
through that parameter is a compile error at the call site, not silently
allowed or silently promoted. The diagnostic should name the fix
directly, in the same spirit as the `Copy`/`Move` errors in
`interfaces.md` — something like "`x` is `const` and cannot be passed
where mutation is required — declare it `var` if it needs to change."

**Amended by `reactivity.md`.** Two things above are too strong once
the component tree is retained rather than a single expression:

- `EmailInput(&email, ...)` elaborating to `&self.email`, "a plain
  shared reference to the `Reactive<T>` cell", holds only while the
  parent strictly outlives the child. A child component is a retained
  object, not a scoped callee, so a reactive prop is **shared
  ownership** — a refcounted handle — rather than a Rust borrow. A
  borrow does not extend the referent's lifetime; the constraint runs
  the other way.
- The single spelling with inferred mutability stays for ordinary
  function calls, where the call is scoped and the callee's signature
  is one hover away. In **prop position** the capability is written
  explicitly, because the handle outlives the call and data flow is
  what a reader scans markup for. There, `&` is a capability grant
  rather than a borrow.

**Consequence for Phase 2:** deciding `&` vs `&mut` for a plain-value
reference requires looking at how the reference is *used* inside the
callee — mirroring how closure capture mode already has to be inferred
from usage rather than annotated. `const`-ness of the referent is then a
hard constraint checked against that inferred usage, not a separate,
independent rule.

## Finding 5 — `pure fn` + borrowing, resolved

No function in the login-form exercise is marked `pure`, so the roadmap's
"how `pure fn` interacts with borrowing" question wasn't exercised by it.
Resolved anyway, by tracing it against findings already made rather than
by example:

- **Parameters are immutable, unconditionally, no carve-out.** `pure
  fn`'s existing "immutable arguments" rule means a parameter can never
  be reassigned, full stop — not "unless the mutation doesn't escape."
  This isn't restrictive in practice because of the next point.
- **`Copy` is the entire escape hatch.** If a computation needs a
  mutable value derived from an immutable argument, copy it into a fresh
  local binding and mutate that — a distinct value, not the argument.
  Primitives get this for free; object types need `Copy` explicitly, the
  same opt-in already required everywhere else `Copy` matters. See
  Finding 6 for the syntax this needs when the argument is itself a
  reference rather than an owned value.
- **References into parameters can only ever be `&T`.** Falls straight
  out of Finding 4: a parameter is `const`-for-the-body, so any `&ident`
  into it elaborates to `&T`, never `&mut T`. A `pure fn` body that would
  otherwise need `&mut` access to a parameter is a compile error, for
  the same reason passing `const` into a mutating function already is.
- **"Long-lived borrows" (the roadmap's original phrasing) were already
  impossible, purity or not.** Finding 3 already says a closure can't
  capture anything that isn't `Copy`/`Move`, and a bare `&T` isn't either
  by default — so a `pure fn` can't smuggle a borrowed parameter into a
  returned/stored closure any more than a non-pure fn can. Nothing
  purity-specific needed here.
- **Returning a reference derived from a parameter** (`pure fn
  firstWord(s &string) -> &string`) is unaffected by purity — ordinary
  borrow-return shape, same as any function.
- **Calling other functions from a `pure fn` body** must itself stay
  pure — already covered by the existing "no external calls" rule in
  `grammar-script-notes.md`, not a new restriction added here.

## Finding 6 — getting a value out from behind a reference needs its own syntax, and its own ownership story

Came out of a counter-argument to Finding 5: "copy it into a fresh local"
is fine as a rule, but what does that look like when the argument you're
holding is *already a reference*? `myPureFn(*dereffed, &reffed)` — who
owns what, when, and why.

**Three distinct call-site forms, three distinct ownership stories:**

- **Bare `x`** (no sigil) — pass by value. Governed entirely by Finding
  3's `Copy`/`Move` check on `x`'s own type. Ownership either transfers
  (`Move`) or a fresh independent copy is made (`Copy`) at the call.
- **`&reffed`** — pass a borrowed reference. No ownership change at all.
  Callee gets temporary access, `&T` or `&mut T` per Finding 4 (always
  `&T` into a `pure fn`). Ownership stays exactly where it already was.
- **`*dereffed`** — `dereffed` is itself a reference (`&SomeType`). `*`
  dereferences it and copies the referent, producing a new, independently
  owned value that belongs to the callee's parameter from the instant of
  the call. Whatever `dereffed` pointed to is untouched afterward — still
  valid, still owned by whoever owned it before. **Only legal if
  `SomeType` implements `Copy`.** It can never mean "move the value out
  from behind `dereffed`" — moving out of a shared reference is unsound
  for the same reason Rust itself forbids it (something else may still
  hold a reference to that slot); rejected with a `Copy`/`Move`-style
  diagnostic ("cannot copy `SomeType` through a reference — it doesn't
  implement `Copy`"), not silently deferred to whatever error rustc would
  eventually produce on the generated code.

**The principle this establishes, generally, not just for `pure fn`:**
implicit reference-to-value coercion is banned. If you're holding a `&T`
and a call site wants a `T`, the compiler never silently inserts a copy —
`*` makes that copy visible in source. Same ethos as the `?`-operator
objection in `error-handling.md`: anything that duplicates a value should
be a token someone typed, not something inferred on their behalf.

**Does this mean every type needs its own `&FooBar implements Copy {}`
block?** No — and this is worth being precise about, because it's a
different question than whether `FooBar` itself is `Copy`. A reference
(`&T`, for *any* `T`) is unconditionally cheap to copy — duplicating it
just duplicates a pointer-sized handle, never the referent — so `&_`
belongs in the same "free, no declaration" tier as primitives, not in
the ordinary nominal-`Copy`-conformance tier objects live in. It isn't a
new rule either: `&_` is just the simplest case of the pattern `Arc<T>`
and `Reactive<T>` already established in Finding 3 — a cheap-to-copy
*handle* whose own copyability is entirely independent of whether the
thing it points to is copyable. You can always copy a `&FooBar` to get a
second reference to the same `FooBar`, even when `FooBar` itself isn't
`Copy` at all. What actually gates on `FooBar: Copy` is specifically the
*deref* — `*dereffed` copying the referent out — not the reference
handle itself. One is compiler-intrinsic and universal; the other is the
ordinary, explicitly-declared conformance this whole document already
depends on.

**Implementation status:** prefix-`*` is now real. `parse_unary`
(`crates/parser/src/ast/expression.rs`) recognizes `*` as a fourth unary
operator alongside `!`, `-`, `&`, sharing the same `STAR` token kind
binary `Multiply` already used — grammatical position (start of an
operand vs. between two operands) disambiguates the two, the same way
Rust/C resolve it, and `1 * *x` parses as multiply-of-a-deref to confirm
it. This is parsing only: `*` produces a `UNARY_EXPR` node in the CST,
the same as `!`/`-`/`&` do; the `Copy`-gated semantics described above
(only legal when the referent type is `Copy`, never a move) are a
typecheck-time concern for Phase 2, not enforced by the parser. Also
worth flagging so it doesn't read as a conflict: `*fn` already exists as
a *declaration*-position prefix (the required-callback marker seen in
`login-form.velp`) — a different grammatical slot from expression-
position `*`, not in conflict, but the same character doing two
unrelated jobs is worth naming rather than leaving implicit.

## Visibility — private by default, `export` to expose

**Fields and component members are private by default; `export` exposes
them.** Same keyword as top-level declarations, same single meaning
everywhere: visibility, and nothing else.

This is what makes the interface model's same-name field/method rule
(`interfaces.md`) pay off — storage stays private while a same-named
method is the public path, so a type controls what callers get at that
name and can change it later without touching a call site.

### `export` does *not* confer reactivity

Reactivity is keyed off **`var` inside a component body**, exactly as
Finding 2 established — not off `export`. The two axes are orthogonal:

|                        | private              | `export`ed            |
|------------------------|----------------------|-----------------------|
| `var` in a component   | reactive local state | reactive **prop**     |
| field on an object     | plain private field  | plain readable field  |
| `*fn` callback         | *(illegal)*          | callback **prop**     |

`*fn` is the one exception to `export`-is-optional: a callback slot can
only be filled from outside, so a private one could never hold a value.
Rejected at parse time rather than parsed into something provably nil.

A prop is just the intersection — a reactive var that happens to be
public. The corpus already depends on this being orthogonal:
`login-form.velp`'s private `password` and `attempts` are reactive
without being exported, and Finding 2's whole lowering rule keys off
"is this binding inside a component", not visibility. If `export` also
meant "reactive", those two mechanisms would collide on the same job and
the private-but-reactive case would contradict it outright.

### Arguments are constant; props are reactive

A component can receive data two ways, and they mean different things:

- **Arguments** (`fn Message(from User, body string) -> Component`) are
  constant inputs. Plain owned values, no cell, no observation.
- **Props** (`export var email Optional<string>` in the body) are
  reactive. `Reactive<T>`, observed, re-render on change.

The wrinkle worth stating because it *will* be misread: passing a
reference to a reactive cell as an argument (`EmailInput(&email, ...)`,
which the corpus does) keeps the argument binding constant while writes
through the reference still drive reactivity. Constant binding, reactive
pointee — the two aren't in conflict, and `&` is what carries the
difference.

### What "private" scopes to — the type, reached through `self`

**A private field is reachable only through `self`, and only from the
type's own implementations.** Not package-private, not file-private
(file location carries no compiler meaning anyway — `interfaces.md`
open question #2). `fn ChatState.usersInRoom(&self)` may read
`self.usersInRoom`; a free function holding a `ChatState` may not read
its fields at all, whatever file it lives in.

The model is Lua's: a method is a free function whose first argument is
the receiver, and `self` is the whole of its access to the instance.
That's why the boundary is the *type* rather than a lexical block — a
dotted method body isn't lexically nested inside the object it belongs
to, so what scopes the access is "is this an implementation of this
type, reaching through `self`".

**Consequence worth knowing up front:** this is stricter than Rust,
where any code in the defining module can touch any instance's private
fields. Here, a second instance of the same type is not reachable —
given `fn Foo.equals(&self, other &Self) -> bool`, `self.x` is legal and
`other.x` is not; `other` is reachable only through its public methods.
That lands closest to the interface isolation rule (`interfaces.md`),
which already says a default method body may only call methods, never
touch fields — so the concrete-type rule and the interface rule agree
rather than pulling apart. It does mean `Equatable`-shaped comparisons
are written against methods on `other`, which is a real ergonomic cost
and the thing most likely to be argued with later.

**Parsed.** `export var`/`export const`/`export *fn` now dispatch in
statement position (`parse_statement`), so the prop forms this document
elaborates finally parse where they're actually written — inside a
component body. They previously had no dispatch arm at all and fell
through to the expression parser. The parser accepts them in *any*
block, deliberately: it can't distinguish a component from a plain `fn`
returning `Component`, so "props only exist in components" is a semantic
check for a later pass. A bare `*fn` (no `export`) is a parse error —
see the table above.

### Props cross the boundary by move, or copy if that's all there is

An `export`ed component prop is reactive *within* the component and fed
from outside. The value transfers by **move**, falling back to copy only
when the type implements `Copy` and not `Move` — move is the preferred
direction, not the fallback. So `Copy`/`Move` conformance decides the
prop-passing mechanism; it isn't a component-specific rule.

This splits prop dataflow cleanly in two, which is worth stating because
the corpus uses both:

- **Moved in** (`Message(from=user, body=text)`) — one-way. The parent
  gives the value up; there is no path back.
- **Passed by reference** (`EmailInput(&email, ...)`) — two-way. The
  callee writes through the reference into the caller's reactive cell.

**Amended by `reactivity.md` for reactive values.** Move is right for
plain values and wrong for reactive ones: moving a reactive value
severs the subscription that makes the prop reactive at all. A reactive
prop crosses as a **handle**, and there are three modes rather than
two — a dead snapshot (what a move gives, almost never wanted), a
read-only live share (no sigil), and a writable live share (`&`). The
middle one is the mode this section was missing.

### No package-level variables or globals

Beyond objects, components and interfaces, elp has no package-level
variables and no global values. There is no file-scope mutable state for
anything to share, which is part of why "private" can be scoped to the
type without leaving a hole — there is no ambient scope for a value to
leak into.

The rule targets package-level *state*, so `const` is unaffected — a
compile-time constant has no identity to share. `BlogStyle/constants.elp`
(`export const FONT_COZETTE`) and `screens/home.velp` (`const NAME`)
stay exactly as written.

**Enforced in the parser.** Top-level `var` is a parse error naming the
alternative: *"elp has no package-level variables; use `const` for a
file-scope constant, or hold mutable state in an object or component"*.
`var` is still dispatched as a declaration keyword rather than dropped
from the keyword list, so the message comes out here instead of the
expression parser failing on a stray `var`. Block-scoped `var` is
untouched.

### Fields carry `var`, and that is what grants writes — resolved

The question was whether `export` on a field grants writes as Rust's
`pub` does, or reads only. Rust's answer hands out unmediated mutation
and the access-flow control above evaporates precisely when someone
reaches for it; read-only exposure preserves the invariant story but
makes two-way binding into a model field impossible, which is most of
what a form does.

Both answers are wrong because the question is wrong. It asks for one
global answer to a per-field decision, and the reason it looked global
is a gap: **object fields currently carry no mutability marker at
all.** Locals distinguish `var` from `const`; fields do not.

```
export object ChatState {
	usersInRoom [User] = []     // private, immutable
	messages [Message] = []
}
```

Give fields the marker and the question answers itself:

| declaration | outside the type |
|---|---|
| `const field T` | not visible |
| `var field T` | not visible |
| `export const field T` | read |
| `export var field T` | read and write — `&obj.field` is legal |

**Amended by `aliasing.md`.** This table originally had a bare `field T`
form and said `const` is *"**not** accepted on a field: it is the
default, and two spellings for the default is one more than the language
needs."* The corpus writes `const` on 72 fields and bare on none, so the
rule was both unimplemented and unloved — and it gave two spellings for
immutable against one for mutable. **A field now spells `const` or
`var`, exactly like a local, and bare is an error.** One spelling each,
zero migration, and a reader never has to decide whether a missing word
meant immutable or meant the author forgot.

The marker governs **rebinding only**. Whether anything can be written
*through* the field is a property of the field's type — a
`Confined<Style>` is writable through a `const` field — and that is what
decides `Copy` eligibility. Two independent axes; see `aliasing.md`,
"`Copy` is field-wise".

**The migration cost is zero.** No object field in the corpus is ever
mutated in place — `Post`, `Content`, `ApiError`, `ChatAppState`,
`ChatState`, `Author`, `User`, `BlogPostParams` are all constructed
whole and replaced via `copyWith` or `Self { ... }`. Every existing
field is correct as written, and `var` has no corpus instance yet.

That is worth stating plainly: this is a forward-looking rule, not one
the corpus demanded. It earns its place by closing the `export`
question with a mechanism instead of a blanket answer, and by giving
`&obj` a precise meaning — a binding to the object through which each
field is writable only if that field says so, rather than a blanket
grant that would launder past visibility. See `reactivity.md`, "Bind
the field, not the object".

## Summary for Phase 2

| elp construct | Rust elaboration | Confidence |
|---|---|---|
| Script-level `var x` | `let mut x` | **Decided** (Finding 1) |
| Script-level `const x` | `let x` | **Decided** (Finding 1) |
| Component `var x` (reactive) | `Reactive<T>` field, interior mutability | **Confirmed by this exercise** |
| `&type` / `&ident` on a reactive field | refcounted handle to the `Reactive<T>` cell, not a plain shared ref | **Amended** by `reactivity.md` (props are shares, not borrows) |
| `&` in prop position | constructs a binding from a cell, forwards one it is given; written at every writable call site | **Decided** (`reactivity.md`) |
| `export var` on an object field | the only form granting outside writes; bare fields are immutable and private | **Decided** (this document, above) |
| `&type` / `&ident` on a plain `const` value | `&T`, always | **Decided** (Finding 4) |
| `&type` / `&ident` on a plain `var` value | `&T` or `&mut T`, inferred from callee usage | **Decided** (Finding 4), mechanism not yet in the typechecker |
| Component methods (`render`, event handlers) | `&self`, never `&mut self` | **Confirmed by this exercise**, contingent on Finding 2 |
| `*fn` callback props | `Option<Arc<dyn Fn(...)>>` | **Confirmed by this exercise**, contingent on Finding 3 |
| Capturing *any* value into a closure, or passing it by value | Legal iff the type implements `Copy`/`Move` (`interfaces.md`); no capture-list syntax | **Decided**, mechanism not yet in the typechecker |
| `pure fn` parameters | `const`-for-the-body; refs into them always `&T` | **Decided** (Finding 5) |
| `x` (bare) at a call site | `Copy`/`Move` per Finding 3, unchanged | **Decided** |
| `&x` at a call site | Borrow, no ownership change, `&T`/`&mut T` per Finding 4 | **Decided** |
| `*x` at a call site (`x` itself a reference) | Deref-and-copy the referent; legal iff referent type is `Copy` | **Decided** (Finding 6); `*` prefix parses today, `Copy`-gating is Phase 2 |
| A value read out of a collection | A **copy**; mutating it does not touch the collection, and a write-back (`map.set(k, v)`) is required and explicit | **Decided** (`core-language-probes.md`, `store.elp`) |
| `&T` itself (the reference, not its referent) | Unconditionally `Copy` for any `T`, compiler-intrinsic — same "free" tier as primitives, not ordinary nominal conformance | **Decided** (Finding 6) |
| `Shared<T>` (immutable alias) | Refcounted handle, `Copy` for any `T` — the owning form of the row above | **Decided** (`aliasing.md`) |
| `Confined<T>` / `Synchronized<T>` (mutable alias) | Refcounted handle, `Move`; a second handle via `.alias()`, never `Copy` — so any type holding one is not `Copy`, field-wise | **Decided** (`aliasing.md`) |

Every row in this table is now decided in rule; nothing here is left
genuinely open. Two rows were reopened after this sentence was first
written and are now closed again by `aliasing.md` — the capture/
pass-by-value rows said `Copy`/`Move` when the tier they needed was
`Duplicate`, and the two aliasing rows above did not exist because the
types did not. What remains is implementation, not design: `Copy`/`Move`
depend on `interfaces.md`'s open questions (nominal-conformance syntax,
whether default methods can depend on other interfaces) being settled
first; the `var`/`&mut` usage-inference rule needs the same callee-body
analysis machinery closures already require for capture mode; and the
`Copy`-gating on `*` (legal only when the referent type implements
`Copy`) is a typecheck-time check, not yet written — the parser already
accepts prefix-`*` (`crates/parser/src/ast/expression.rs`), it just
doesn't know yet whether a given use of it is sound. None of that blocks
Phase 2 from starting — it's real work Phase 2 does, not a question
Phase 2 is waiting on an answer to.

## Collections hold values, not places

A value obtained from a collection is a copy. Mutating it has no effect on
the collection, and persisting the change requires an explicit write-back:

```
.RESOLVE(code, reply) -> {
	match links.get(code) {
		nil  -> reply.send(Err(StoreError.NOT_FOUND(code)))
		link -> {
			link.hits += 1
			links.set(code, link)      // required, not optional
			reply.send(Ok(link))
		}
	}
}
```

This is the consistent reading of the old corpus's
`state.messages.copy().append(...)`, and it is the conservative choice:
an API that forgets the write-back is a visible no-op rather than an
invisible aliasing bug.

The cost is verbosity, and it is accepted deliberately — the cheaper
spelling (`map.get(k)` yielding a mutable place, or an `update(k, fn)`
helper) can be built in elp later without changing the semantics. A
language rule that hands out hidden aliases cannot be walked back.
