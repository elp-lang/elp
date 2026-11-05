# elp macros — compile-time codegen via a small embedded evaluator

Grew out of `interfaces.md`'s "why `Copy`/`Move` don't get a default
method" — their ergonomic case (not hand-writing field-by-field logic for
every type) was deferred here. This doc is where that gets resolved,
plus everything else macros turned out to need.

## The decision

**A macro is an ordinary `pure fn`, tagged to run at compile time against
the AST of whatever it's attached to.** Safety is `pure fn`'s *existing*
definition — no external calls, no async work, immutable arguments (see
`grammar-script-notes.md`) — not a new restriction invented for macros
specifically. If it's `pure`, it's already safe to run at compile time;
that's the whole check.

**elp data is uniformly first-class, including the language's own
structure.** Interfaces, types, fields, declarations — all of it is just
elp data, inspectable the same way any other elp value is. There's no
separate "macro reflection API" bolted on next to normal elp; a macro is
code that happens to look at that data instead of executing it.
Concretely: `@CodeGen(Copy, Move)`'s arguments (`Copy`, `Move`) are real,
inspectable interface values — not opaque type tags — which is what lets
`CodeGen` (or anyone's user-space equivalent) generate an implementation
for *any* interface by reading its required methods, rather than
hardcoding a fixed list of interfaces it happens to know about. Generics
were considered and rejected as the mechanism for this (they push the
problem into needing a runtime "zero value" + class-comparison dance
inside the macro — more machinery, not less).

**Macro arguments are checked like any other function call** — a macro
declares its own parameter signature, and `@Foo(args)` is checked against
it the normal way. Arguments can be interface/type references (as above)
or constants — including constants computed by *another* `pure fn` at
compile time, which is how build variants and environment configuration
get into the system. That's not a separate feature from macros; it's the
same "evaluate `pure` elp at compile time" mechanism, just used to produce
a plain value instead of generated code.

**A macro's body cannot call another macro while it's running.** No
macro-of-macros recursion — this is a restriction on the *execution call
graph*, distinct from the next section's pipeline, which *does* let
macro-generated code contain further macro attributes.

## Pipeline shape: fixed-point expansion

1. Parse source → AST (existing frontend, unchanged).
2. Scan the AST for macro-attributed items (`@Foo(...)`).
3. For each one found, evaluate the macro against the AST it's attached
   to (see "Execution model" below), producing generated code.
4. Splice the generated code into the tree in place of the macro
   attribute.
5. Repeat from step 2 against the *new* tree. The phase ends when a pass
   finds zero macro attributes remaining.

Generated code from one pass can legally contain more `@Macro(...)`
attributes, picked up by the next pass — this is what makes "a macro
can't call another macro" (previous section) *not* the same rule as "a
macro can't produce code that uses other macros" (this section allows
that freely). One restricts the call graph during execution; the other
governs generated source content, expanded on a later, separate pass.

**Not yet decided: a termination safety valve.** Nothing here stops a
pathological macro from generating code that re-triggers itself forever.
Needs a max-passes limit or equivalent before this is real; not designed.

## Execution model: direct evaluation, not native compile-and-load

Two shapes were on the table:

- **(a) Compile-and-load, Rust-proc-macro style:** transpile the macro
  through the normal elp→Rust→rustc pipeline, produce a native artifact,
  dlopen it, call it via an FFI boundary.
- **(b) Direct evaluation:** run the already-parsed-and-typechecked AST
  through a small embedded evaluator, in-process, with no native
  compilation step at all.

**Chose (b).** The deciding factor: under (a), even a trivial
constant-computing `pure fn` (a build variant, an environment-derived
config value — an explicitly wanted use case) costs a full native
compile-and-link cycle just to learn what it returns. That's not viable
if this is meant to be usable in many places across a build, the way
constant-evaluation generally is. (b) answers the same question in
microseconds, because there's no build step involved at all.

The reason (b) doesn't balloon into "now you're building a general-purpose
language VM": `pure fn`'s existing restrictions have *already* drawn the
boundary. No IO, no external calls, no async, immutable arguments — that
excludes exactly the things that make interpreters hard to build (no
concurrency model to simulate, no outside world to model). The evaluator
only needs to handle literals, arithmetic, control flow (`if`/`match`/
`for`), calls, and object/enum construction — the subset `pure fn` was
already scoped down to, for unrelated reasons, before this document
existed.

Secondary reasons, real but not the deciding one: (b) only depends on
Phase 2 (typecheck) existing, not the full Phase 4 (Rust codegen)
pipeline — macros (and therefore `Copy`/`Move`'s ergonomic case, and
build-time config) become usable earlier in the roadmap than "wait for
everything else to be built" would imply. And (b) has no FFI/ABI boundary
to design and version — that boundary is the single most-complained-about
part of Rust's own proc-macro system (compiler/proc-macro-crate version
mismatches produce famously bad diagnostics).

**Named cost, accepted deliberately:** this is a second execution
mechanism, existing alongside "transpile to Rust, lean on rustc" for
everything else in the compiler. Every other piece of real elp code
becomes Rust and gets compiled; this path doesn't. That's a genuine
deviation from the pattern used everywhere else in this project — worth
it here specifically because `pure fn` already bounds the scope tightly,
and because a small, low-stakes, `pure`-scoped evaluator is a reasonable
place to learn what embedded execution in this codebase looks like,
without it being load-bearing for anything else yet.

## Open questions

1. **The reflection API's exact shape.** "Interfaces are inspectable
   data" is decided; what fields/methods an `Interface` value concretely
   exposes to macro code (its required methods? their full signatures?
   default-method bodies too?) is not designed.
2. **How a `pure fn` gets marked as usable as a macro.** Is any `pure fn`
   with a suitable signature automatically eligible, or does it need its
   own declaration (a keyword, an attribute on the `fn` itself)? Not
   decided.
3. **Fixed-point loop termination.** See above — no safety valve designed
   yet for runaway macro expansion.
4. **Diagnostics when macro evaluation fails** — a panic inside the
   evaluator, or a macro that produces code that doesn't parse/typecheck.
   Not discussed at all yet.
