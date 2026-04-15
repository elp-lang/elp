# Elp script grammar — design notes and intentional gaps

Companion to `grammar-script.g4`. The `.g4` is the formal grammar; this
file holds the prose that would otherwise clutter it — design decisions,
features that are deliberately absent, and rules the grammar accepts
syntactically but that semantic analysis must still reject.

The `.g4` has its own shorter "Caveats" section at the bottom covering
**grammar-spec-vs-hand-written-parser divergences** (hidden newlines,
opaque strings, reserved contextual keywords, etc.). This file covers
**language-level decisions** — what Elp does and doesn't do as a language,
independent of how any parser implements it.

## Intentional gaps — things Elp deliberately doesn't have

1. **No `async` / `.await`.** Concurrency is Go-style spawned coroutines
   owned by the runtime. Syntax for spawning is TBD and deliberately not
   in the grammar yet. The decision is "no coloured functions" —
   callers and callees don't have to know whether work happens on a
   background coroutine.

2. **No `while` / `loop` / `break` / `continue`.** Only `for x in expr` is
   defined today. Conditional loops are a known gap and will be added.
   Until then, recursion or `for x in someIterator` covers it.

3. **No `?` error-propagation operator.** Errors are always pattern-matched
   explicitly. The compiler is expected to warn on unhandled `Either<_, E>`
   values that escape a scope, Go-style. If `?` is ever added it would be
   sugar over an explicit match, not a new control-flow primitive.

4. **No `throw` in `.elp`.** The `throw` form seen in `.velp` component
   bodies lives in the view dialect, not the script dialect. Script code
   returns errors; view code can throw because there's a root error
   boundary catching them.

5. **Field-level `const` is not permitted.** Use immutability at the
   binding level (`const` vs `var`) instead of at the field level. A
   field read from an immutable binding is itself immutable.

6. **No nested function declarations.** A `fn` inside a `fn` body is
   rejected in the semantic pass. The grammar accepts it so the parser
   can produce a specific error message rather than a generic one.
   Use composition or redesign the API.

## Syntactic rules that only exist at use-site

7. **Contextual `.Variant` is an expression, not a declaration prefix.**
   Enum variants are declared as bare identifiers:

   ```elp
   enum ApiErrorType {
       SERVER(string)
       USER(string)
       UNKNOWN
   }
   ```

   The leading dot is inference-only: at a use site where the expected
   type is known, `.SERVER("oops")` is shorthand for
   `ApiErrorType.SERVER("oops")`. Declaring variants with a leading dot
   is a syntax error.

8. **Object-literal keys are bare identifiers — no leading dot.**

   ```elp
   // right
   Self { nodes = .new(), edges = .new() }

   // wrong — the leading dot is for contextual type inference, not field
   // access in a literal
   Self { .nodes = .new(), .edges = .new() }
   ```

   Rule of thumb: a leading dot in expression position means "infer the
   type from context and pick this variant/constant". Nothing else.

9. **String interpolation is implicit Swift/Kotlin style**: `{ expr }`
   inside a string literal interpolates. This requires the lexer to
   emit `STRING` tokens as a chunked sequence (string-part, expr-part,
   string-part, …) rather than a single opaque payload. The current
   hand-written lexer does not yet do this and the `.g4` models strings
   as opaque.

10. **Struct tags** (`` `json: "id"` ``) are accepted on any `Field`.
    Their semantics — who reads them, at what phase — are part of the
    macro subsystem. The grammar allows them everywhere; whether they
    mean anything depends on which macros are applied to the containing
    object.

## Interfaces

11. **Interfaces may declare fields and method signatures.** An object
    satisfies an interface if it structurally provides every field and
    method. This is closer to structural subtyping than to Rust traits
    or Go interfaces — the name `implements` is a compile-time assertion
    that structural conformance holds, not a nominal declaration.

## Things the grammar accepts that semantic analysis must still reject

The `.g4` grammar is intentionally looser than valid Elp in a few places,
so the parser can give targeted errors instead of generic ones:

- **Nested `fn` declarations.** See gap #6.
- **`Type { ... }` object literals where `Type` is not an `object` or
  `enum`.** The parser can't know this until name resolution runs.
- **`NamedType` in expression position that resolves to a function.**
  Ambiguous at parse time between a call and a constructor application —
  disambiguated by the type checker.
- **`self` / `&self` appearing outside a method body.** Rejected in the
  semantic pass.
- **`pure` functions containing side-effectful calls.** Enforced by the
  effect system, not the grammar.

## Open questions

The items below are real design calls that aren't made yet. They appear
in one or more example files but have no specced behaviour:

- **`*fn` prefix** in component-body prop declarations (ChatRoom
  `login-form.velp`). If it means "required callback", that's a
  view-dialect concern, not script grammar. Currently not modelled.
- **Nested namespace imports** like
  `import { Component, components: { Text }, font: { FontWeight } } from elp.ui`
  (BlogStyle `paragraph.velp`). Not in the grammar; decision pending on
  whether the import syntax should grow this shape or whether
  consumers should just write several `import` lines.
- **Conditional loops** (`while`, `loop`). See gap #2.
- **Doc comments** (`///`). The lexer recognises them as a hidden
  channel; no extraction pipeline yet. Not blocking the parser.
