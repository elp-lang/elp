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

## What a macro attaches to may be a block

**A macro is not an entry in the program.** It is code generation: it
receives the AST of whatever follows it and produces declarations, and
*those* are what the program contains. Nothing about the attachment site
survives into the program unless the macro emits it.

That reading answers the grouping question on its own. `@Foo(args) { ... }`
needs no rule permitting it, because the braces are not a construct in the
language's grammar of program structure — **they are an argument.** A
declaration list is an AST node like any other, so a macro may be handed
one.

```elp
@external(.C, library = "sqlite3") {
	export fn sqlite3_open(path &CString, db &Optional<Sqlite3>) -> int
	export fn sqlite3_close(db Sqlite3) -> int
	export fn sqlite3_errmsg(db &Sqlite3) -> &CString
}
```

The demand came from `examples/SqliteBinding`, which writes
`library = "sqlite3"` fourteen times in a file that links exactly one
library. The tempting fix was a rule of "`library =` is required only when
the package links more than one" — a conditional requirement the language
refuses everywhere else (mandatory type arguments, `const` on every field,
`&` at every writable call site). The probe rejected grouping too, on the
grounds that it would be *a new declaration shape, which is what
`@external` being an attribute was supposed to avoid.*

**Both objections were about the wrong thing.** The repetition was not an
argument for weakening an attribute's rules, and the grouping form is not
a form of `@external`. It is what a macro's input is allowed to be, and it
pays off wherever a run of declarations shares one attribute:

```elp
@CodeGen(Copy) {
	export object Point { const x int, const y int }
	export object Size  { const w int, const h int }
}
```

### It is not a scope, and not by exception

An earlier draft of this section called the block "the one brace in the
language that introduces no scope". That was the wrong description of a
right outcome. The brace does not scope because **the brace is not in the
program.** It is input to a generator. What reaches the module is the
macro's output — ordinary declarations, at module level, scoped exactly as
if hand-written. There is no exception to carve.

The same reasoning disposes of two rules that draft invented:

- **`export` is not distributed**, but not because "`export` is not an
  attribute". *Nothing* is distributed. The macro was handed declarations
  that said `export` and emitted declarations that say `export`. It copied
  its input.
- **Nested attributes do not "merge"**, and whether an `@external` inside
  an `@external` block is an error is `@external`'s business rather than
  the language's. The macro receives the AST; the macro decides.

Both become properties of a particular macro instead of rules of the
language — one less thing to specify, and specified in the right place.

### A block is more than shorthand

That earlier draft made the block a **desugaring**: expanded into N copies
of the attribute before any macro ran, so a macro never saw a group. Safe,
free of shape checks, and too weak — it also inserted a pipeline step to
buy nothing.

Under the reading above the macro receives the group, so it may generate
code that spans the group:

```elp
@Registry {
	export object Png  { const width int, const height int }
	export object Jpeg { const width int, const height int }
	export object Webp { const width int, const height int }
}
```

— which can emit the three types *and* an enum over them *and* one lookup
keyed by magic bytes, none of which three independent expansions can
produce. That is the capability the desugaring wrote off as an accepted
cost, and it is the point of the whole mechanism: a little source standing
in for a lot of generated code.

The price, named: `@Foo { A B }` no longer *necessarily* means `@Foo A` and
`@Foo B`, the same way `f([a, b])` is not `f(a); f(b)`. A macro that
distributes says that it distributes, and `@external` does.

### `@macro` is the marker, and the AST is the last parameter

**A `pure fn` becomes a macro by carrying stdlib's `@macro`, and the AST it
is attached to is its final parameter.**

```elp
@macro
export pure fn CodeGen(interfaces [Interface], node Declaration) -> [Declaration]
```

**The AST goes last** because that is the only ordering in which the
signature reads in the order the source is written: `@Foo(a, b) { ... }` is
`Foo(a, b, node)`. The explicit arguments are in the parentheses and the
AST follows them, so last in the signature is last in the file.

It is also the one parameter with no call-site spelling. `generics.md`
makes parameter names part of a signature because they resolve overloads,
and the attached AST is never written as an argument — so a trailing
positional parameter is the only shape that does not lie about how it
arrives.

**`@macro` is the marker**, which closes open question 2 below, and it is
itself a macro — stdlib's, applied to the `pure fn` that defines one. That
matters twice over. It needs no keyword, which `reactivity.md` rules out as
policy and which is why `@external` is not `extern`. And a macro system
able to state its own entry condition is evidence the mechanism is strong
enough to be worth having: had `@macro` needed a keyword, that would have
been a finding about the macro system rather than a convenience.

The name is UpperCamelCase against every other `fn` in the language,
because it is never written at a call site — only after an `@`. So it is
less a violation of function naming than a different namespace, and the
section below is what makes that readable rather than arbitrary.

**The correction: `@macro` checks the last parameter, it does not append
it.** Appending is the obvious reading — the author writes the arguments
and stdlib adds the AST — and it fails on the rule everything else here
follows. A parameter the author did not write is a name the body refers to
with no declaration in sight, which is exactly the implicitness behind
mandatory type arguments, `&` at every writable call site, `return` in
every closure and `const` on every field.

It also could not know *what* to append. `@CodeGen` wants a declaration,
`@default` an enum case, `@allow` a statement, `@external` a group — so
appending needs the type in `@macro`'s own arguments, which is the same
information in a worse place. Checking costs nothing and gives the
guarantee asked for: `@macro` rejects a `fn` that is not `pure`, one whose
last parameter is not an AST type, and one whose return type is not
generated code, all at the declaration site rather than at a use.

The use site is then checked the same way any call is. A macro whose last
parameter is `Declaration`, attached to a block, is an ordinary type
mismatch at the attachment site — which is a better diagnostic than any
shape check the macro's author could hand-write, and needs no machinery
beyond "macro arguments are checked like any other function call".

What it needs that does not exist yet is a name for each AST shape —
`Declaration`, `[Declaration]`, `EnumCase`, `Statement`. That is open
question 1 below, the reflection API's shape, which now has a second
dependent and is no longer only about inspecting interfaces.

### Three things are spelled `@`, and the corpus already distinguishes them

Deciding what a macro *is* forces the question of what everything else
wearing an `@` is, and the answer was already on the page:

| | emits | spelling | examples |
|---|---|---|---|
| **macro** | declarations | Capitalised | `@CodeGen`, `@Duplicate`, `@Screen`, `@Reducer` |
| **directive** | nothing — it tells the compiler something | lowercase | `@external`, `@abi`, `@error`, `@default`, `@allow` |
| **a value** | nothing — it *is* one | Capitalised, being a type | `@Package` |

Every elp attribute in the corpus obeys this, which makes it evidence
rather than a convention invented here.

**Directives have to be intrinsic, and `@external` is the proof.** What it
produces is a link to a foreign symbol, and no `pure fn` returning elp
declarations can express that — so it cannot be a macro even in principle,
and lowercase is the honest spelling. `@allow` suppresses a diagnostic and
`@default` marks an enum case; both are facts the compiler consumes, not
code it splices.

**`@Package` is a third thing.** Every manifest opens
`import { Package, Version } from elp.package`, so `Package` is an imported
**type** and `@Package(name = ..., version = ...)` is a constructor call in
attribute position, declaring the module's manifest value. It attaches to
nothing — which no macro can do, and no directive needs to.

**The capitalisation is enforced, and not as naming police.** A user cannot
declare a directive, because a directive is the compiler — so the set of
lowercase attributes is closed and known to it. `@macro` therefore rejects
a lowercase name: not because lowercase reads badly, but because the name
would claim to be an intrinsic and the compiler is in a position to know it
is not.

That is what separates this from `name-resolution.md`'s package-name
convention, which has nothing behind it deliberately. There the
capitalisation carries no information the compiler holds; here it carries
exactly one, and it is the distinction a reader most needs: whether what
follows the `@` was written in elp or is built in.

## Reflection is compile-time only

The reflection types — `Interface`, `Declaration`, `EnumCase`, `Statement`
— **have no runtime representation.** This is not a visibility rule and
there is nothing to make private: they describe source structure, and at
runtime there is no source.

That is stronger than discouraging runtime reflection, and strictly better
for the reason reflection needed discouraging. Its expense is real and it
is famously reached for in hot paths; here the hot-path cost cannot be paid
because there is nothing to call. The cost moves to build time, which is
the one place where paying it once per build instead of once per iteration
is the entire point.

**The rule: a type is compile-time-only iff it is a reflection primitive or
has a field of one**, and such a type may not appear in a runtime-reachable
position — not a field, not a parameter, not the return type of anything
the program can call.

Field-wise, which is `aliasing.md`'s rule for `Copy`, reused rather than
invented. That is now the second thing this one rule decides.

### The one leak that existed

The section above lets compile-time evaluation produce *a plain value*
instead of generated code — build variants, environment-derived config —
which is deliberate, and was the single path by which an `Interface` could
have reached the program:

```elp
const requirements = Summable.requirements()      // illegal: `[Declaration]`
const names        = Summable.requirementNames()  // fine: `[string]`
```

So the usable statement is: **reflection values may be consumed at compile
time and never produced into the program.** A macro may read an interface's
requirements and emit a `[string]` of their names; it may not hand the
program the requirements.

**Checked after expansion, needing no new phase.** Generated code is
program code, so the rule applies to it equally — and the pipeline already
ends when a pass finds no macro attributes remaining, with typecheck after.
The check belongs to that typecheck, over the final tree.

### A type tag is not reflection

Worth drawing, because "no runtime reflection" sounds like it forbids
things it does not:

| at runtime | exists? | why |
|---|---|---|
| an interface value's vtable | **yes** | that is dispatch, and `dispatch.md` owns it |
| a value's type tag — the type switch | **yes**, once designed | a tag comparison: bounded, constant-cost |
| a type's *structure* — fields, names, requirements | **no** | that is reflection, and it is compile-time only |

The line is that dispatch and tags answer *which one is this?* while
reflection answers *what is this made of?*. The first is a pointer or an
integer; the second is a graph the runtime would have to carry for every
type in the program.

### What it costs, named

What uses runtime reflection in other languages becomes codegen here: JSON
decoding, a DI container, debug-printing an arbitrary value, a test runner
discovering its tests. Each is a macro instead — more build time, no
runtime cost, and a generated implementation a reader can go and look at.

**The corpus already works this way**, which is what makes the rule cheap
to adopt rather than a restriction to absorb. `@CodeGen(Codec<JSON>)` on
`Post`, `@CodeGen(ToJSON, Default)` on `ApiError` and `ApiErrorType`:
compile-time serialisation, generated per type, in four packages. Nothing
in the corpus asks for runtime reflection, so nothing is given up.

The residual is build time — a macro reflecting over many types pays on
every build, and `elp data is uniformly first-class` invites exactly that.
It is the cost that replaces the runtime one, and worth watching rather
than designing around yet.

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
   default-method bodies too?) is not designed. It now has a second
   dependent: `@macro` validates a macro's last parameter against an AST
   type, so `Declaration`, `[Declaration]`, `EnumCase` and `Statement`
   need to be nameable types before that check exists. What is *settled*
   about them is that they are compile-time only (above), which bounds the
   question: the API has to be rich enough to read a type's structure and
   has no obligation to be cheap, because nothing calls it at runtime.
2. ~~**How a `pure fn` gets marked as usable as a macro.**~~ **Decided**
   above: stdlib's `@macro`, which is a macro itself, and the attached AST
   is the final parameter. What remains is a sub-question of 1 — the AST
   shapes `@macro` checks against need names before it can check
   anything.
3. **Fixed-point loop termination.** See above — no safety valve designed
   yet for runaway macro expansion.
4. **Diagnostics when macro evaluation fails** — a panic inside the
   evaluator, or a macro that produces code that doesn't parse/typecheck.
   Not discussed at all yet.
