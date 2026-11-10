# Elp grammar & spec docs

This directory holds the language specification artifacts, and right now
they are the only authority on what the language is. There is no compiler
in this repository: the hand-written lexer, parser and CST crates were
removed because the design is still moving and a front end built against a
moving design is rework. Where a doc cites `crates/lexer` or
`crates/parser` as evidence, it is citing that removed front end, and the
observation stands as a record of what had been built rather than as
something to go and read.

## Files

| File | What it is | When to use it |
|---|---|---|
| `grammar-script-notes.md` | Language-design notes: intentional gaps, use-site rules, semantic constraints, open questions | Read before arguing with a parse result |
| `memory-model.md` | How elp's ownership constructs (`var`/`const`/`&type`/`&self`, closures/`Copy`/`Move`) elaborate to Rust, validated by hand-porting a real file; plus scoped release (`with ... as x { }` and `Scoped`), `Buffer<T>` for contiguous storage with the out-of-line principle for where C may hold an address, and what `&` means where there is no callee to infer from | Read before designing the typechecker's ownership tracking or the Rust-transpiler codegen |
| `aliasing.md` | Aliasing and duplication: `Shared<T>`/`Confined<T>`/`Synchronized<T>`, the three conformances (`Copy`/`Duplicate`/`Move`), `Copy` is field-wise, `.copy()` vs `.alias()`, `Atomic<T>` and `Ring<T>` for threads with a deadline (and `Move` enforcing SPSC), `unsafe` as FFI-only, and a C callback checked as a `background` body | Read before implementing conformance checking, closure captures, or anything that crosses a thread |
| `interfaces.md` | Interface model: method-only, nominal conformance, interface dependencies, why `Copy`/`Move` don't get default implementations | Read before implementing interface conformance checking, or before designing the macro engine |
| `macros.md` | Macro model: a macro is a `pure fn` run at compile time via direct AST evaluation, elp data (incl. interfaces/types) is uniformly first-class, stdlib `@macro` marks one, the attached AST is its last parameter and may be a block of declarations, reflection is compile-time only | Read before implementing `Copy`/`Move` codegen or any compile-time evaluation |
| `reactivity.md` | Reactivity model: the component body runs once, reads subscribe, writes patch; how a reactive value crosses a prop boundary | Read before implementing component lowering, markup control flow, or prop passing |
| `generics.md` | Generics and overloading: mandatory type arguments, overload resolution on parameter types and names, conditional conformance, variance, and the two `<>` parsing problems | Read before implementing type arguments, interface conformance with generics, or method resolution |
| `name-resolution.md` | Modules and name resolution: packages, the two import path forms, one-file-per-module-path (a segment is a filename, so hyphens are allowed in one), global conformances, the atmospheric-type criterion for the prelude (and why `Buffer` fails it), the no-cycles rule, the dependency manifest, `links` as an exclusive allow-list of native libraries, and `scripts` as build-stage jobs with declared inputs | Read before implementing the module loader, import resolution, or the package manifest |
| `core-language-probes.md` | Eleven probe programs (Boyer-Moore, an HTTP server, an aggregating app, a libsqlite3 binding, a Vulkan/GLFW triangle) and the gap log of constructs they exposed — operators, loops, `Map`, `char`, tuples and aliasing now resolved; optional ergonomics and interpolation still open, plus rounds on recursive value types, captures, `break`, channel shutdown, the FFI boundary and system resources | Read to see which core-language gaps remain, and which probe demanded each |
| `functions.md` | Functions, methods and the call model: a method belongs to its receiver's package, no extension methods on foreign types, the extension-interface route, `fn(...)` as a C function pointer inside an `@external` block plus `@abi(.C)` and `@abi(.C, realtime)`, and why there is no `|>` | Read before implementing method resolution, or before adding any call-syntax sugar |
| `strings.md` | Strings and characters: `string` as UTF-8 scalar sequence, `char` as one scalar value, scalar-wise `==` with no implicit normalization, literal and escape syntax, bytes as `Buffer<uint8>`, and why `s[i]` does not exist | Read before implementing string literals, `char`, interpolation, or any string stdlib |
| `patterns.md` | Patterns and destructuring: why there is no tuple type, `(a, b)` as a `match`-scrutinee form only, named destructuring with `as` (exhaustive for an owned aggregate), Optional narrowing by binding, and the collected pattern grammar | Read before implementing `match`, `for` bindings, or pattern parsing |
| `dispatch.md` | What an interface name in a type position *means*: existential vs universal quantification, what each promises a caller, the three deliberate existential sites, associated types for `Iterable`, and the representation questions deliberately deferred to the backend | Read before designing the type checker's handling of interface-typed values, or before choosing a dispatch representation |
| `collections.md` | Collections: `Hashable` and the hasher, insertion-ordered `Map`, trapping `[]` vs `Optional`-returning `get`, `&var self` for mutating methods, the iteration protocol, why `Buffer<T>` is not a collection, and `_` as the only catch-all | Read before implementing `Map`/`Array`, `for` lowering, or mutating method receivers |
| `operators.md` | Operators as interface methods: the interface table, what cannot be overloaded and why, `==` with no `===`, the precedence table, and narrowing through `&&`/`||` | Read before implementing expression parsing, operator lowering, or flow-sensitive typing |
| `diagnostics.md` | Diagnostic model: noisy-by-default and filtered by configuration, the soundness floor that keeps errors non-configurable, named lints, where levels are set, and compile-time vs runtime diagnostics | Read before adding any compiler warning, or designing the lint configuration surface |
| `error-handling.md` | Error model: `Either<T, E>` values only, no throw/catch/boundary mechanism, `throw` is sugar for `return Err(...)` | Read before implementing `Either` consumption-checking or any `.velp` `throw` lowering |

## There is no formal grammar file

An ANTLR4 spec grammar (`Elp.g4`, plus `ElpStyle.g4` / `ElpView.g4` for
the `.selp` and `.velp` dialects) lived here briefly, along with a
`Makefile` test rig that drove `grun` against the example corpus. All of
it has been deleted, deliberately.

The reasoning: a spec grammar that nothing consumes has to be kept in
sync with the real parser by hand, and it never is. In practice it
diverged in eight documented ways within a single session, and the
divergences generated design questions that were really just artifacts
of the spec grammar's own simplifications — time spent resolving
problems the language doesn't have. The parser is hand-written and
always was; a second, lossy description of it was a liability, not a
safety net.

The genuinely language-level questions that surfaced while writing it
were preserved in `grammar-script-notes.md` under "Open questions"
before deletion.

If a machine-checkable grammar is ever wanted again, the thing to build
is one the compiler actually consumes (parser-generator-driven, or a
grammar extracted *from* the hand-written parser), not a parallel
document that chases it.

## Workflow when changing the language

1. Probe the question in `examples/` — a program that uses the construct,
   with its unanswered parts written down as comments in the file.
2. Record the *decision*, and the reasoning that forced it, in the
   relevant doc above. These files explain why the language is shaped the
   way it is.
3. Migrate the corpus onto the decision, so the examples and the docs
   never disagree.

Step 3 used to be checked by a corpus parse test in `crates/parser`. With
the front end removed, nothing enforces it, so it is a manual discipline
until there is a compiler again.
