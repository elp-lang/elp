# Elp grammar & spec docs

This directory holds the language specification artifacts. None of these
files are consumed by the compiler — the production lexer and parser are
hand-written in `crates/lexer` and `crates/parser`, and they are the
final authority on what the language is. These docs are the design
record behind them.

## Files

| File | What it is | When to use it |
|---|---|---|
| `grammar-script-notes.md` | Language-design notes: intentional gaps, use-site rules, semantic constraints, open questions | Read before arguing with a parse result |
| `memory-model.md` | How elp's ownership constructs (`var`/`const`/`&type`/`&self`, closures/`Copy`/`Move`) elaborate to Rust, validated by hand-porting a real file | Read before designing the typechecker's ownership tracking or the Rust-transpiler codegen |
| `interfaces.md` | Interface model: method-only, nominal conformance, interface dependencies, why `Copy`/`Move` don't get default implementations | Read before implementing interface conformance checking, or before designing the macro engine |
| `macros.md` | Macro model: a macro is a `pure fn` run at compile time via direct AST evaluation, elp data (incl. interfaces/types) is uniformly first-class | Read before implementing `Copy`/`Move` codegen or any compile-time evaluation |
| `reactivity.md` | Reactivity model: the component body runs once, reads subscribe, writes patch; how a reactive value crosses a prop boundary | Read before implementing component lowering, markup control flow, or prop passing |
| `generics.md` | Generics and overloading: mandatory type arguments, overload resolution on parameter types and names, conditional conformance, variance, and the two `<>` parsing problems | Read before implementing type arguments, interface conformance with generics, or method resolution |
| `name-resolution.md` | Modules and name resolution: packages, the two import path forms, one-file-per-module-path, global conformances, the atmospheric-type criterion for the prelude, the no-cycles rule, and the dependency manifest | Read before implementing the module loader, import resolution, or the package manifest |
| `core-language-probes.md` | Three probe programs (Boyer-Moore, an HTTP server, an aggregating app) and the gap log of constructs they exposed — operators, loops, `Map`, `char`, tuples now resolved; optional ergonomics, function values and interpolation still open | Read to see which core-language gaps remain, and which probe demanded each |
| `functions.md` | Functions, methods and the call model: a method belongs to its receiver's package, no extension methods on foreign types, the extension-interface route, and why there is no `|>` | Read before implementing method resolution, or before adding any call-syntax sugar |
| `strings.md` | Strings and characters: `string` as UTF-8 scalar sequence, `char` as one scalar value, scalar-wise `==` with no implicit normalization, literal and escape syntax, and why `s[i]` does not exist | Read before implementing string literals, `char`, interpolation, or any string stdlib |
| `patterns.md` | Patterns and destructuring: why there is no tuple type, `(a, b)` as a `match`-scrutinee form only, named destructuring with `as`, Optional narrowing by binding, and the collected pattern grammar | Read before implementing `match`, `for` bindings, or pattern parsing |
| `dispatch.md` | What an interface name in a type position *means*: existential vs universal quantification, what each promises a caller, the three deliberate existential sites, associated types for `Iterable`, and the representation questions deliberately deferred to the backend | Read before designing the type checker's handling of interface-typed values, or before choosing a dispatch representation |
| `collections.md` | Collections: `Hashable` and the hasher, insertion-ordered `Map`, trapping `[]` vs `Optional`-returning `get`, `&var self` for mutating methods, the iteration protocol, and `_` as the only catch-all | Read before implementing `Map`/`Array`, `for` lowering, or mutating method receivers |
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

1. Edit `crates/lexer` / `crates/parser` (the production change).
2. Run `cargo test -p parser` to catch regressions against the example
   corpus in `crates/parser/test_projects/`.
3. Record the *decision* — not the syntax — in the relevant doc above.
   These files explain why the language is shaped the way it is; the
   code explains what it accepts.
