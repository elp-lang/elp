# elp interfaces — pure method contracts, nominal conformance

Grew out of the memory-model discussion (`Copy`/`Move` needed to live
*somewhere*, and interfaces were the natural place) but the result is
bigger than memory management — this is the general interface model going
forward. **Supersedes `grammar-script-notes.md` point #11**, which
described a structural, field-permitting model that's no longer accurate.

## The decision

**Interfaces declare method signatures only — no fields.** The earlier
design (interfaces can require fields, e.g. `LoginProps { email
Optional<string> }`) is dropped. Anything that was a field requirement
becomes a required method instead: `email Optional<string>` → `fn email()
-> Optional<string>`. The implements type provides that method however
it wants internally; the interface never sees how.

Why: field requirements only exist to make *default method* bodies
checkable (a default body needs something concrete to operate on), and for
domain interfaces a named field requirement works fine for that — but
`Copy`/`Move`'s natural default ("copy every field, whatever they are")
needs a *generic, reflective* field requirement, which is either an open,
extensible "any interface can request field access" mechanism (rejected —
see "Why `Copy`/`Move` don't get a default method" below) or a special
case carved out just for those two (rejected — inconsistent, and the
question of *which* interfaces get the exception has no principled
answer). Dropping fields entirely avoids the whole question.

**Conformance is nominal, not structural, and `implements` is the only
declaration form — there is no combined `object Foo implements A, B {
fields }` form anymore.** An object implements an interface only via a
separate `TypeName implements InterfaceName` declaration —
coincidentally having a same-named, same-shaped method does not count,
and there's no shorthand that folds conformance into the object's own
declaration. This replaces the earlier "an object satisfies an interface
if it structurally provides every field and method" rule.

That declaration is a bare marker, not a block requiring bodies inside
it — the actual method implementations are ordinary dotted free
functions (`fn TypeName.method(...) -> ... { ... }`), exactly the style
already used throughout the example corpus (`fn ChatAppState.default ->
Self { ... }`, `fn ApiError.new(...) -> Self { ... }`). Nothing new needed
here: once conformance is a separate declaration (dropping the combined
form) and file location is confirmed to carry no compiler meaning (next
paragraph), the existing free-function convention already covers "where
does the method body live" — there was no gap to fill with new syntax.
Reasoning: the compiler is going to use "does this type implement `Copy`"
to *authorize* things that are otherwise illegal (capturing a value into a
closure, passing it by value) — accidental structural conformance would
mean a type becomes copyable without anyone deciding that on purpose,
which is exactly the "memory ballooning, and nobody knows why" failure
mode this whole discussion started from.

## Default methods, and the isolation rule

An interface method can either be required (signature only, the
implementing type must supply a body) or have a default implementation
(a body, in the interface itself, that the implementing type can accept
as-is or override).

**A default method's body may only call other methods of the same
interface** (an implicit `self` bound to that interface's own method set)
— it can never reach into the implementing object's private fields, full
stop. This is deliberately the same rule Rust already enforces for trait
default methods: `fn greet(&self) -> String { format!("Hello, {}",
self.name()) }` can call `self.name()` (another trait method) but has no
way to see a struct's actual fields, no matter who implements the trait.
No exceptions, including for `Copy`/`Move` — see below.

## Why `Copy`/`Duplicate`/`Move` don't get a default method

**`aliasing.md` amends this section's cast.** There are three
duplication conformances, not two: `Copy` (implicit, independent),
`Duplicate` (explicit via `.copy()`, independent, possibly deep) and
`Move`. Everything below applies unchanged to all three, and
`Duplicate` is the one that most wants the derive — `examples/Sheet`
forces four duplication functions and three of them contain no decision
at all.

**Eligibility, as against implementation, is not hand-written.** `Copy`
is **field-wise**: a type is `Copy` only if every field is. That is a
compiler rule and not a default method, so the isolation rule is
untouched — the compiler reads the field list, which no elp code is
allowed to do. `aliasing.md` has the consequences, the important one
being that `Confined<T>` and `Synchronized<T>` are `Duplicate` rather
than `Copy`, so any type holding one is not `Copy` either, transitively,
with no special rule.

Their natural default — "copy/move every field, whatever they happen to
be" — needs field access, which the isolation rule above categorically
forbids. So there is no default implementation for `Copy`/`Move` at the
interface level. Every type that wants to be `Copy` or `Move` writes its
own `FooBar implements Copy { fn copy(&self) -> Self { ... } }` by hand,
field by field.

The ergonomic case (not having to hand-write that for a trivial struct) is
deferred to **macros**: `@Copy`/`@Move` (names TBD) would generate that
exact `implements` block from the type's field list at compile time —
the same mechanism already implied by `@CodeGen(ToJSON, Default)` and
`@CodeGen(Codec<JSON>)` in the example corpus (`api.elp`). Nothing new is
being invented here conceptually; `Copy`/`Move` just become two more
consumers of a codegen mechanism that already needs to exist for other
reasons.

Load-bearing distinction: the macro *generates ordinary, explicit,
non-magical elp code* (a real `implements Copy { ... }` block a human
could have typed) before the real compiler ever sees it. The isolation
rule is never violated — the field access happens in the macro, which
legitimately operates on the full AST, not in the interface system, which
never does.

## Open questions

1. ~~Does `implements` replace the combined form, or coexist?~~
   **Resolved: replaces it outright.** There is no combined form. See
   "The decision" above.
2. ~~Are multi-file splits filename-driven or pure convention?~~
   **Resolved: pure developer convention.** Filenames carry zero
   compiler significance — ordinary cross-file name resolution (Phase 2
   needs this anyway) is the entire mechanism. A type's declaration, its
   `implements` markers, and its dotted-function method bodies can be
   split across as many or as few files as a developer likes, named
   however they like; nothing about where a declaration lives changes
   what it means.
3. ~~Can a default method's body depend on the type also implementing a
   *different* interface (a "supertrait"-shaped requirement)?~~
   **Resolved: yes, via a dependency declared at the interface's own
   definition site, spelled with the existing `: Bound + Bound` shape.**
   See "Interface dependencies" below.

4. ~~Grammar/parser gap: `interfaceMember` has no body-allowed method
   form.~~ **Dropped as a tracked concern.** This was an artifact of the
   ANTLR spec grammar, which has since been deleted outright
   (`docs/README.md`). The hand-written parser is the only authority and
   simply doesn't have default-method-body parsing built yet, same as it
   doesn't have the new `implements` declaration form built yet either.
   Ordinary follow-up implementation work once the design above is acted
   on, not a design question.

5. **Conditional conformance has an unstated soundness premise, and
   nothing guards it.** `Channel<T: Sendable> implements Sendable`
   (`generics.md`) is sound because a channel is *internally
   synchronized* — its own operations already order concurrent access,
   so the contents really are the only open question. That premise is
   nowhere in the syntax. Nothing stops a library author writing
   `MyCache<T: Sendable> implements Sendable` over an unsynchronized
   cache and getting a silent data race, and `aliasing.md` had to rule
   out `Confined<T: Sendable> implements Sendable` by hand for exactly
   this reason.

   The orphan rule does not help: the author owns `MyCache`, so the
   conformance is legally theirs to declare. **Open**, and the options
   are narrow — restrict conditional conformance on `Sendable`
   specifically to stdlib types, require some marker that says "this
   container orders its own access", or accept it as the one place a
   conformance can lie and lint it. Worth settling before any
   concurrency work lands, because it is the only hole in `Sendable`
   that `aliasing.md` could not close from outside.

## Interface dependencies

An interface may declare that it depends on other interfaces, at its own
definition site, using the same `: Bound + Bound` shape the language
already uses for generic parameter bounds:

```elp
interface Equatable {
    fn equals(&self, other &Self) -> bool
}

interface Comparable: Equatable {
    fn compareTo(&self, other &Self) -> Ordering

    fn isEqualTo(&self, other &Self) -> bool {
        return self.equals(other)   // legal — Equatable is declared above
    }
}
```

An earlier draft wrote `Comparable: Equatable + Hashable`, to show two
dependencies at once. That was a syntax demonstration that read as a
stdlib specification, and it was wrong twice over:

- **Ordering and hashing are orthogonal.** Nothing about being sortable
  implies being hashable, and Rust and Swift both keep them apart. Under
  the explicit-conformance rule below, the dependency would force every
  sortable type to also declare `Hashable` and supply a hash it does not
  need.
- **`Hashable` has no definition anywhere**, so `Comparable` was
  unimplementable as written.

For a genuine two-dependency example, `generics.md` has a real one:
`Codec<T>: From<T> + To<T>`.

This is Rust's supertrait shape and Rust's spelling. There is no
`requires` keyword and never was — a set of interfaces is written as a
`+`-joined list everywhere it appears, so the same idea reads the same
way in a generic bound, in an interface header, and in a conformance
declaration:

```elp
ChatState implements State + Copy
```

(This replaces the comma-separated list of the retired combined form,
`object ChatState implements State, Copy { ... }`.)

### Rules

1. **Conformance stays explicit.** `Foo implements Comparable` is an
   error unless `Foo implements Equatable` and `Foo implements
   Hashable` are also declared. A dependency is never auto-satisfied or
   implied — the reason structural conformance was rejected (§"The
   decision") is that nothing should become conformant without a human
   declaring it, and inferring dependent conformance would reintroduce
   exactly that. The compiler error should name the missing declarations.
2. **The isolation rule extends to the transitive closure.** A default
   method body may call methods of its own interface plus those of every
   interface in its dependency closure, transitively: given `A: B` and
   `B: C`, a default body in `A` may call `C`'s methods. The requirement
   is transitive, so the visibility is too. Everything else about the
   isolation rule is unchanged — still no field access, ever.
3. **Name collisions are rejected at the definition site.** An interface
   may not declare a method whose name is declared by any interface in
   its dependency closure. This is what keeps `self.equals(other)`
   unambiguous, and it means elp needs no disambiguation syntax
   (Rust's `<T as Trait>::method`) at all. Dependency cycles (`A: B`,
   `B: A`) are likewise rejected.
4. **Generic bounds inherit the closure.** `fn max<T: Comparable>(...)`
   may call `.equals()` on a `T`, because bounding on `Comparable`
   implies its dependencies. This is the part that makes dependencies
   worth having beyond "one default body compiles" — without it, every
   call site would have to restate the full bound list by hand.

## Coherence — who may declare a conformance

**Rust's orphan rule.** `T implements I` is legal only in the package
that defines `T` or the package that defines `I`. A third package may
not connect two types it doesn't own.

This matters more here than it does in most languages, because of what
conformance authorizes. Nominal conformance exists so that nothing
becomes `Copy` without a human deciding it (§"The decision"); a
third-party package granting `Copy` to someone else's type is that same
hole through a different door, since the type's author never made the
call and the memory behaviour changes underneath them. Without the rule
there's also no principled tiebreak when two packages declare the same
conformance with different method bodies.

## Interfaces as types — object safety, computed not declared

An interface may be used as a type (`fn Paragraph(text string) ->
Component`), giving dynamic dispatch, **but only when it is object-safe**.
Object safety is *computed from the interface's own declaration*, never
annotated: an interface is object-safe unless it mentions `Self` in
parameter position, and interfaces with no methods at all (`Copy`,
`Move`) are not object-safe either.

So `Component` keeps working as a return type throughout the example
corpus, while `Equatable` (`fn equals(&self, other &Self) -> bool`)
can't be one — behind a dynamic reference there is no way to know that
two values share a `Self`. Using a non-object-safe interface as a type
is an error at the *use site*, naming the method that disqualified it.
Interfaces remain usable as generic bounds either way; object safety
constrains only the as-a-type use.

## Conformance checking — exact signature match

An implements type satisfies a required method only with an exact
match: same name, same arity, same parameter types, **same parameter
names**, same return type. No variance, no arity flexibility. Anything
looser leaves the interface unable to express what a caller may rely
on, which is the whole job of having one.

Parameter names are part of the contract because a call through an
interface-typed value has to know what label to use — so the label must
be fixed by the interface, not by whichever type happens to be behind
it. See `generics.md`, "Overloading". The corpus's
`fn ApiError.from(err HttpError)` is migrated to `value` for this
reason, since `From<T>` declares the parameter as `value`.

This has already caught a real error in the example corpus:
`ChatRoom/state/app.elp` declared `fn new -> Self` in `interface State`
while `fn ChatState.new(user User) -> Self` took a parameter. Fixed by
correcting the interface.

## Generic interfaces — see `generics.md`

Generic interfaces (`From<T>`, `To<T>`, `Codec<T>: From<T> + To<T>`),
conditional conformance (`Channel<T: Sendable> implements Sendable`)
and overload resolution are designed in `generics.md`. Two rules in
this document are amended there:

- **Exact signature match** now includes parameter names, above.
- **The name-collision rule** below rejects two interfaces
  contributing the same method name. That still holds for *unrelated*
  interfaces, but not across instantiations of one generic interface:
  `ApiError` conforms to `From<HttpError>`, `From<string>`,
  `From<int8>` and `From<Error>`, each contributing a `from`, and
  those are resolved by overloading on parameter type.

## Implementation status

**Built.** The conformance declaration is parsed end to end:

- `IMPLEMENTING_DECL` / `INTERFACE_LIST` CST nodes, `IMPLEMENTS_KW`
  retired in favour of `IMPLEMENTING_KW` (`crates/cst/src/syntax_kind.rs`).
- `ImplementingDecl` with `type_name()` / `interfaces()`, and
  `implementing_from_cursor` parsing `TypeName implements A + B`
  (`crates/parser/src/ast/declaration.rs`). *The Rust-side names
  (`IMPLEMENTING_KW`, `ImplementingDecl`, `implementing_from_cursor`)
  still say `implementing` and want renaming when the keyword change is
  implemented.*
- The old combined form is a parse error that names the replacement:
  `object Foo implements A, B` reports "the combined `implements` form
  was removed; declare conformance separately as `Foo implements A + B`".
- All three example files migrated (`ChatRoom/main.velp`,
  `ChatRoom/state/app.elp`, `BlogStyle/internal/api.elp`).

Field requirements are also gone from the parser: an interface body
accepts `fn` members only, and a field requirement is a parse error
naming its rewrite (`email Optional<string>` → "write `fn email() -> T`
instead"). Both corpus interfaces migrated (`State`, `LoginProps`).

Two details decided at implementation time rather than in the design
above:

- `implements` is **contextual, not reserved** — `object implements
  { ... }` still declares an object called `implements`. Top-level
  dispatch checks declaration keywords first, and only then falls back to
  the two-token `IDENT implements` shape.
- An `implements` declaration **can't be exported**. `export Foo
  implements Bar` is a parse error: conformance travels with the type
  it's declared for rather than being separately visible. Ratified.

**Not built.** Everything that needs more than parsing:

- Interface *dependencies* — the `interface Comparable: Equatable +
  Hashable` header form is designed above but not parsed.
- Default method bodies in interfaces (`interfaceMember` has no
  body-allowed form yet).
- Every checking rule in this document. Nothing verifies signature
  match, the orphan rule, object safety, dependency presence, method-name
  collisions, or dependency-graph acyclicity — `implements` currently
  parses and is then believed. All of it needs cross-package name
  resolution first.
- Generic args anywhere (`From<HttpError>`), which is why
  `BlogStyle/internal/api.elp` still doesn't parse. Pre-existing
  whole-grammar gap, not specific to interfaces.

## A method and a field may share a name

**Allowed, as in Rust.** `self.foo` and `self.foo()` are different
expressions, so the call syntax disambiguates completely and the parser
needs no special rule. Go forbids this; elp doesn't.

Migrating `State`'s field requirements to method requirements forced
`ChatState` to grow accessors, and the natural name for an accessor is
the name of the field it returns:

```elp
export object ChatState {
    usersInRoom [User] = []
}

fn ChatState.usersInRoom(&self) -> [User] {
    return self.usersInRoom
}
```

This isn't just tolerated, it's the point: **one member name, two access
paths, and the type controls which one callers get.** A type can keep
storage private and expose the same name as a method that returns a
computed value, a clone, a filtered view, or a projection of several
fields — and later change which of those it does without renaming
anything or touching a call site. Every interface that used to state a
field requirement lands here, since a field requirement is now by
definition a method requirement.

This depended on field visibility, now settled: **fields are private by
default and `export` exposes them**, and a private field is reachable
only through `self`, from the type's own implementations — see
"Visibility" in `memory-model.md`. Storage stays private while the
same-named method is the public path, which is exactly the control this
decision was made for.

That access rule interacts directly with this document: since `other
&Self` is *not* reachable field-wise even inside the type's own methods,
an `Equatable` implementation compares through `other`'s methods, the
same way an interface default body has to. The concrete-type rule and
the isolation rule end up saying the same thing. One sub-question
remains open — whether `export` on a field grants writes or only reads
— and it doesn't change the rule here.

## Open — how does a component conform to a props interface?

`login-form.velp` declares `interface LoginProps { fn email() ->
Optional<string> ... }` and, separately, a `LoginForm` component whose
props are `export var email Optional<string>` and `export *fn onSubmit`.
Nothing connects the two: `LoginForm` has no `implements` declaration,
and `LoginProps` isn't referenced anywhere.

It can't be connected as written, either. Under exact signature match, a
prop named `email` does **not** satisfy a required `fn email() ->
Optional<string>` — one is storage, the other is a method. So one of:

- Props interfaces are a real thing, and a component declares
  `LoginForm implements LoginProps` plus accessors — which collides
  head-on with the point of props (they're set from outside, not
  computed).
- A prop *does* satisfy a same-named, same-typed method requirement — a
  deliberate exception to exact match, justified by the field/method
  name-sharing rule above (the accessor is exactly what `export` on a
  field already means).
- `LoginProps` is aspirational and should be deleted.

Undecided. It's the last place where the interface model and the
component model don't line up, and it wants an answer before conformance
checking is written, since that's the pass that would reject the corpus
as it stands.

## The macro engine this depends on, and isn't designed yet

The metaprogramming discussion that produced this doc revealed a much
bigger feature than "parse `@Foo(...)` attribute syntax," which is as far
as macros were previously scoped. The actual shape: a macro is an elp
function that receives the full AST of whatever it's attached to and
returns generated code, run at a pre-compile stage before normal
compilation starts. That needs:

- elp's own AST exposed as data a macro can pattern-match and construct
  (a reflection API over the compiler's own types). `macros.md` adds the
  bound those types carry: they are **compile-time only** — no runtime
  representation, checked field-wise like `Copy` — so this is a reflection
  API that cannot become a runtime one.
- Some execution model for actually *running* macro code before the rest
  of the program compiles — macros are themselves ordinary elp, subject
  to all the same rules as any other elp code (including needing `Copy`/
  `Move` implementations for their own values, if it comes to that).
- One explicit constraint already decided: **a macro cannot consume
  (invoke/expand) another macro.** No macro-of-macros recursion — macro
  expansion is a single, bounded, non-recursive pass over the source, not
  an open-ended expansion system.

This is its own multi-session design effort — noted here only so
`Copy`/`Move`'s "ergonomic default via macros" doesn't get mistaken for
something that already has a plan behind it. It doesn't.
