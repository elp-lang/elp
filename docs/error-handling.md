# elp error handling — values for errors, a separate boundary for panics

## The decision

**Errors are values (`Either<T, E>`), always returned, always matched
explicitly. There is no throw/catch/boundary control-flow mechanism for
error values anywhere in the language.** This applies uniformly to `.elp`
script code and `.velp` component code — one model, not two. (Panics —
bugs, not `Err` values — get a separate, narrower containment mechanism;
see "Panic containment" below. The two are unrelated and shouldn't be
conflated.)

This was a real fork, not the only option on the table. The alternative
considered was "two mechanisms": ordinary `Either` matching for expected
failures, plus a separate UI-level boundary that would catch an error left
"unconsumed" anywhere in a component subtree and surface it at the nearest
ancestor boundary, so a forgotten `match` somewhere deep in a tree couldn't
silently misbehave or take down the app. Rejected. Reasoning, in the
decision-maker's own words: mechanism 1 alone "solves more problems than
it creates," and the invisibility of an error going unhandled deep in a
tree is a userspace problem, not a compiler one — intentionally so. The
team building an app is responsible for their own error-propagation
standards; the compiler's job stops at guaranteeing a value is never
silently dropped, not at guaranteeing it was handled *well* or *close to*
where it occurred.

## Why one mechanism is enough, not risky

The load-bearing rule making this safe rather than reckless already
exists and predates this doc: `grammar-script-notes.md` gap #3 — the
compiler is expected to warn (error) on an `Either<_, E>` value that
escapes a scope unconsumed, Go-style. That rule is what stops "no
boundary" from meaning "errors can vanish." It doesn't mean every error
lands somewhere *sensible* — it only means every error is *touched*
(matched, or explicitly returned onward as a value) at every step. Whether
the resulting propagation chain is well-designed is exactly the part left
to the team, deliberately.

## `throw` in `.velp`

`throw` stays as syntax, but strictly as sugar for `return Err(...)` —
nothing more. No bubbling past the function it's written in, no boundary
catching it, no default recovery behavior. The caller of a
`throw`-containing function receives an ordinary `Either` and matches it
exactly like the result of any other function call. This supersedes the
framing in the original `login-form.velp` comment and in
`grammar-script-notes.md` gap #4, both of which described `throw` as
bubbling to "the default root error boundary" — that boundary doesn't
exist; both are being corrected alongside this doc.

Why `.velp` gets this sugar and `.elp` doesn't: unchanged from gap #3/#4's
original scope split — script code stays fully explicit, `throw` is a
component-body ergonomic. Nothing here revisits that split; it wasn't
part of the boundary question this doc resolves.

## `?` — still not decided

Unrelated to the boundary question, and still open. The objection to
Rust's `?` specifically is its implicit `Into`/`From` conversion across
error types — the auto-coercion machinery, not propagation-shorthand as a
category — and how that machinery complicates `Copy`/derive and lifetime
management. A narrower operator (only legal when the error type already
matches exactly, no coercion, otherwise a plain type error at that line)
would dodge that specific complaint if boilerplate ever becomes a real
problem in practice. Not designed, not scheduled — `grammar-script-notes.md`
gap #3 stands as written until real code makes the tradeoff concrete.

## Matching an interface-typed value always needs a fall-through

A `match` on a value whose static type is an **interface** can never be
exhaustive, because the set of conforming types is open — any package may
add a conformance. So a catch-all arm is **mandatory** there, and it is
forced by the type system rather than imposed as a style rule.

This matters most for `Error`, which is the interface the corpus matches
on. `BlogStyle/internal/api.elp` comments that a function returning
`Either<Content, Error>` can be matched "at the call site … to work out
whether it was an ApiError or a SerializationError".

Which surfaces a construct the language does not have: **matching an
interface-typed value by concrete type** is a *type switch*, distinct
from matching an enum's variants, and neither its syntax nor its
semantics is designed. Whatever it ends up looking like, the mandatory
catch-all applies to it.

### Enums are different, and a catch-all is a choice there

An enum's variant set is closed and known, so a match over one *can* be
exhaustive — and a catch-all is always available when most arms are
irrelevant. The corpus already relies on both halves:

- `ChatRoom/main.velp`'s `AppReducer` is exhaustive with no catch-all,
  and its comment says so deliberately.
- `AppMiddleware`, in the same file, handles one variant and ends `_ ->
  nil`, with a comment noting the match "ISN'T exhaustive and we DO have
  a catch-all".

So nothing forces every arm to be written. **A match with no catch-all
must be exhaustive; writing `_` opts out.** That is what keeps
`Category.label` and `PatternError.toString` honest — five arms for five
variants, and a compile error the day a sixth is added, rather than a
silently wrong string from a catch-all that should never have run.

*Open: whether a library author can mark an enum open
(`#[non_exhaustive]`-style), forcing downstream catch-alls so that adding
a variant is not a breaking change. elp pins one version per package and
makes conflicts a build error, so a new variant can only ever arrive
through a deliberate upgrade — which makes the break visible and
informative rather than surprising. That argues for closed-by-default
with an opt-in marker later, not for the reverse.*

## Panic containment: `RecoveryBoundary`

Unrelated to everything above — this section is about bugs, not `Err`
values. An app shouldn't crash out because one component somewhere in
its tree panicked. That needs containment, and containment needs a
recovery story, but recovery has a hard limit: some panics happen because
data was corrupted out of band, and pretending those are recoverable is
worse than crashing. The design here rides on a split Rust already makes
for its own reasons, rather than inventing a new classification scheme.

**Rust's `catch_unwind` only ever catches a clean, cooperative unwind** —
drop handlers ran, nothing left in a state Rust itself can't account for.
A double panic during unwind, a stack overflow, or a panic trying to
cross an `extern "C"` boundary all abort the process outright and never
reach `catch_unwind` at all. That last case is concretely relevant here,
not hypothetical: the UIKit pipeline calls into ObjC via raw
`objc_msgSend`, real unsafe FFI surface. So "can't and shouldn't recover
from data messed with out of band" isn't something elp needs to detect —
it's exactly the set of failures Rust already refuses to hand a recovery
opportunity for. `RecoveryBoundary` wraps `catch_unwind` around a
subtree; the corruption/abort case is handled by never reaching it.

**Scope of what a boundary wraps:** a subtree's render *and* its
event/callback execution, not just the initial render pass. A panic
inside `handleSubmit` mid-interaction is exactly as much a containment
concern as a panic during `render`.

**Recovery means discard-and-reinit, never resume-in-place.** A panic
mid-mutation of reactive state can leave it logically torn even though
Rust's memory safety still holds — the same reason `std::sync::Mutex`
poisons itself when its lock-holder panics. So a caught panic discards
the crashed subtree's existing state, renders the developer's fallback,
and — if the developer offers a retry — re-initializes that subtree fresh
from its props. Never "patch and continue" with the state as it was at
the moment of the panic.

**Nesting:** nearest-ancestor-boundary wins, same as React's error
boundaries. An outer `RecoveryBoundary` only sees a panic if no closer
one exists. Ordinary, well-understood behavior; no new semantics needed.

**An implicit root boundary always exists**, runtime-provided, even if a
developer writes zero `RecoveryBoundary`s anywhere. This is what actually
delivers "an app shouldn't crash out" unconditionally — user-authored
boundaries only narrow the blast radius further; they're never the only
thing standing between a panic and a full crash.

Deliberately not named `ErrorBoundary` — that name already carries the
"bubble an unconsumed `Either` to a boundary" idea from earlier in this
doc's history, which was considered and rejected. `RecoveryBoundary` is
unrelated to `Either` entirely; conflating the two is exactly the
confusion the naming is meant to avoid.

### Open questions

1. **Default fallback for the implicit root boundary.** Some minimal
   runtime-provided UI is needed for the case where a developer wraps
   nothing themselves. Not designed.
2. **Retry API shape.** "Re-initialize fresh from props" implies the
   runtime hands the developer's fallback some way to trigger that.
   Not designed.
3. **Diagnostics on a caught panic** — what surfaces to the developer
   (logs, a dev-mode overlay, a hook to report to a crash service).
   Not discussed.
