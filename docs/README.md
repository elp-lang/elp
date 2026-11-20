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
| `grammar-script-notes.md` | Language-design notes: intentional gaps, use-site rules, semantic constraints, raw identifiers (`` `type` ``, Kotlin's and Swift's spelling, wanted by the corpus before the macro engine was), open questions | Read before arguing with a parse result |
| `memory-model.md` | How elp's ownership constructs (`var`/`const`/`&type`/`&self`, closures/`Copy`/`Move`) elaborate to Rust, validated by hand-porting a real file; `&var T` **ratified** as a parameter spelling written at the declaration and the call site (and why Finding 4's call-site half is what gives way), the nine gaps that one absence was reported as and the two decisions that close all of them, plus scoped release (`with ... as x { }` and `Scoped`), `Buffer<T>` for contiguous storage with the out-of-line principle for where C may hold an address, what `&` means where there is no callee to infer from, and `indirect`, **ratified**, as a field marker for a recursive value type (an immutable box, so the containing type stays `Copy`; replaceable but never written through) | Read before designing the typechecker's ownership tracking or the Rust-transpiler codegen |
| `aliasing.md` | Aliasing and duplication: `Shared<T>`/`Confined<T>`/`Synchronized<T>`, the three conformances (`Copy`/`Duplicate`/`Move`), `Copy` is field-wise, the transform closure `update(fn (&var T) -> R) -> R` **ratified** across both cells and both collections, `.copy()` vs `.alias()`, `Atomic<T>` (over a word-sized `Copy` object) and `Ring<T>` for threads with a deadline (and `Move` enforcing SPSC), where a deadline body's private state lives (a `Confined<T>`, never a plain field) and why cells get no endpoint split, `Opaque<Confined<T>>` as the one sanctioned non-`Sendable` crossing, “all speed on the writer, all policy on the reader”, why three of the four cells cannot be an edge in a reference cycle and why an interpreter's `let rec` defeats that result and makes `Weak<T>` mandatory (**ratified**, on the cell rather than the payload — and why it still does not fix `let rec`, which leaks, as in Rust and Swift) while a data structure's back-edge dissolves, `string` as `Copy` because it is immutable and what that leaves of the copy-on-write question (a stdlib property of `[T]` and `Map` only), why `Copy` does not mean trivially copyable in any of its three refcounting cases, what Rust/Go/Swift do instead and which two of the three mechanisms are closed off here, paid for with a layout-gated lint, `Protected<T>` designed at last as secrecy by absent conformance — plain elp, `Duplicate` never `Copy`, with `.expose()` as the one greppable way out — `unsafe` as FFI-only, and a C callback checked as a `background` body | Read before implementing conformance checking, closure captures, or anything that crosses a thread |
| `stdlib-boundary.md` | What is language, what is a privileged library and what is plain elp; why “bare” means fewer *privileged* features rather than fewer features; what Rust, Swift and Go actually do (and why Go is the opposite); the unmade decision between a stdlib-privilege marker and compiler-known types; the current open items sorted into the three buckets, and the order the ergonomics round has to run in | Read before deciding whether a gap is a language change, and before the corpus scan |
| `tradeoffs.md` | A cross-cutting look at the *decided* set rather than any one decision: the five problems that have several answers, the twelve decisions whose cost landed on another program (with who is paying each), and the one check that would have caught every defect — name the substitute before removing a capability, and confirm it is not itself removed | Read before removing a capability, and when a decision feels like it is fighting another |
| `open-questions.md` | The single register of live open questions: one line each, naming the doc that owns it, plus the three load-bearing things with no design at all (`pure` and its undesigned effect system, `Protected<T>`, loop statements), the migration backlog the write-surface round created, and a record of the stale entries a `docs/` scan corrected | Read before opening a new question, and before the corpus scan |
| `interfaces.md` | Interface model: method-only, nominal conformance, interface dependencies, why `Copy`/`Move` don't get default implementations | Read before implementing interface conformance checking, or before designing the macro engine |
| `macros.md` | Macro model: a macro is a `pure fn` run at compile time via direct AST evaluation, elp data (incl. interfaces/types) is uniformly first-class, stdlib `@macro` marks one, the attached AST is its last parameter and may be a block of declarations, reflection is compile-time only | Read before implementing `Copy`/`Move` codegen or any compile-time evaluation |
| `reactivity.md` | Reactivity model: the component body runs once, reads subscribe, writes patch; how a reactive value crosses a prop boundary | Read before implementing component lowering, markup control flow, or prop passing |
| `generics.md` | Generics and overloading: mandatory type arguments, overload resolution on parameter types and names, conditional conformance, variance, and the two `<>` parsing problems | Read before implementing type arguments, interface conformance with generics, or method resolution |
| `name-resolution.md` | Modules and name resolution: packages, the two import path forms, one-file-per-module-path (a segment is a filename, so hyphens are allowed in one), global conformances, the atmospheric-type criterion for the prelude (and why `Buffer` fails it), the no-cycles rule, the dependency manifest, `links` as an exclusive allow-list of native libraries, and `scripts` as build-stage jobs with declared inputs | Read before implementing the module loader, import resolution, or the package manifest |
| `core-language-probes.md` | Sixteen probe programs (Boyer-Moore, an HTTP server, an aggregating app, a libsqlite3 binding, a Vulkan/GLFW triangle, a real-time audio playhead ported from a closed-source app and held locally, a spreadsheet recalculation graph that answers where reference cycles leak, a tree-walking interpreter that reverses the answer and makes `Weak<T>` mandatory, an LRU cache that shows the canonical cycle site needs none of it, a text editor that prices the cell write surface, and a JSON package whose macro — the first written in the corpus — finds that an AST has no literal syntax, a ledger whose transfer cannot be written atomically, a dashboard that puts five concurrency primitives in one place, and sixteen programs written to be refused), the three-question rubric (safety, expressiveness, composability) those last two were chosen by, a full-corpus scan that found three defects in decided things — including an FFI call the exclusivity rule made unwritable and a `Duplicate` idiom two docs cite that never type-checked — and the gap log of constructs they exposed — operators, loops, `Map`, `char`, tuples and aliasing now resolved; optional ergonomics and interpolation still open, recursive value types now answered by `indirect`, plus rounds on captures, `break`, channel shutdown, the FFI boundary and system resources | Read to see which core-language gaps remain, which probe demanded each, and the three-question rubric (safety, expressiveness, composability) for choosing the next one |
| `functions.md` | Functions, methods and the call model: a method belongs to its receiver's package, no extension methods on foreign types, the extension-interface route, `fn(...)` as a C function pointer inside an `@external` block plus `@abi(.C)` and `@abi(.C, realtime)`, **`pure` specified at last** as a declared bit over a closed set of operations propagated call-wise (no effect system, by `Copy`-is-field-wise's pattern one level up) with `realtime` as the second such property and *not* a superset of it, and why there is no `|>` | Read before implementing method resolution, or before adding any call-syntax sugar |
| `strings.md` | Strings and characters: `string` as UTF-8 scalar sequence and `Copy` because it is immutable, `char` as one scalar value, scalar-wise `==` with no implicit normalization, literal and escape syntax, bytes as `Buffer<uint8>`, and why `s[i]` does not exist | Read before implementing string literals, `char`, interpolation, or any string stdlib |
| `patterns.md` | Patterns and destructuring: why there is no tuple type, `(a, b)` as a `match`-scrutinee form only, named destructuring with `as` (exhaustive for an owned aggregate), Optional narrowing by binding, and the collected pattern grammar | Read before implementing `match`, `for` bindings, or pattern parsing |
| `dispatch.md` | What an interface name in a type position *means*: existential vs universal quantification, what each promises a caller, the three deliberate existential sites, associated types for `Iterable`, and the representation questions deliberately deferred to the backend | Read before designing the type checker's handling of interface-typed values, or before choosing a dispatch representation |
| `control-flow.md` | Loop statements: `while`, `break` and `continue`, why there is no `loop` and no labels (with the cost of each absence), `a..b` including both ends, where the rest of `for` was already decided, why a markup `for`'s `by` is an identity rather than a binding, what happens when a `by` key repeats (it traps, as in Compose, rather than going undefined, as in SwiftUI), and the four failures around a reconciliation key with the three the design reaches — mandatory `by`, a lint whose message names the *model*, and a comma-separated compound key that costs no tuple and no allocation | Read before implementing loops, or before asking where `for` is specified |
| `collections.md` | Collections: `Hashable` and the hasher, insertion-ordered `Map`, trapping `[]` vs `Optional`-returning `get` vs trapping `update(i, fn)`, `&var self` for mutating methods, the iteration protocol, why `Buffer<T>` is not a collection, a note on growth mode as the axis an indexed mutable place would arrive on (Goose's grow-only arrays), and `_` as the only catch-all | Read before implementing `Map`/`Array`, `for` lowering, or mutating method receivers |
| `operators.md` | Operators as interface methods: the interface table, what cannot be overloaded and why, `==` with no `===`, the precedence table, and narrowing through `&&`/`||` | Read before implementing expression parsing, operator lowering, or flow-sensitive typing |
| `diagnostics.md` | Diagnostic model: noisy-by-default and filtered by configuration, the soundness floor that keeps errors non-configurable, named lints, where levels are set, and compile-time vs runtime diagnostics | Read before adding any compiler warning, or designing the lint configuration surface |
| `error-handling.md` | Error model: `Either<T, E>` values only, no throw/catch/boundary mechanism, `throw` is sugar for `return Err(...)`. Plus the 2026-11-20 round: one concrete error type per layer, `?` with three propagation rules, `@message`, the three exception-safety guarantees and `commit`, `errdefer` (proposed), typed aggregation | Read before implementing `Either` consumption-checking, `?`, any `.velp` `throw` lowering, or a cell's write surface |

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
