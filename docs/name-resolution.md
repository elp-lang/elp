# elp name resolution and modules

## What a package is

A directory containing a `Package.elp`. The manifest is ordinary elp —
a macro invocation, not a separate config dialect:

```
import { Package, Version } from elp.package

@Package(
	name = "social_feed",
	version = .new(1, 0, 0, 0),
	entry = "main.velp",
	screens = "./screens",
	targets = [.iOS, .Android, .Web],
)
```

A package is the unit of distribution, the unit of version pinning, and
the root that a leading-dot import path resolves against.

## Two path forms

| form | meaning | corpus |
|---|---|---|
| `elp.layout`, `elp.net` | another package, rooted at its name | 20 distinct |
| `.types.post`, `.components.paragraph` | *this* package, rooted at the package directory | 22 |

**A leading dot means "this package".** No leading dot means "another
package". There is no file-relative form at all.

This is settled by the corpus rather than by preference.
`examples/BlogStyle/screens/blog/index.velp` sits two directories deep
and imports:

```
import { getBlogPost } from .internal.api
import { Paragraph }   from .components.paragraph
```

Both targets live at the *package root*
(`BlogStyle/internal/api.elp`, `BlogStyle/components/paragraph.velp`),
not beside the importing file. Were the dot file-relative, every one of
these would be wrong.

The consequence worth naming: there is no `..` and no way to say
"the file next to me". Moving a file changes no import in it, only the
imports *of* it. That is the tradeoff taken deliberately — package-root
paths are stable under refactoring in the direction that matters, and
a reader never has to know where the importing file sits to know what
a path points at.

## A module path is a file, and only ever a file

`a.b.c` resolves to `a/b/c.{elp,velp,selp}`, and **exactly one** of
those three may exist. Two files differing only in extension is a
compile error. The path never encodes which dialect it found.

### A path segment is a filename, not an identifier

**A module path segment may contain internal hyphens**, so
`from .components.entry-form` names `components/entry-form.velp` and reads
exactly as the file is named.

The problem it fixes is that `.components.entry-form` tokenises as
`entry - form`, which `core-language-probes.md` found in
`examples/Expenses` and which was latent for a long time because the
corpus's other hyphenated file was never imported. `examples/VulkanTriangle`
made it load-bearing: three files import `internal/vk-types.elp`.

**It needs no lexer mode, because there is nothing to disambiguate
from.** The thing after `from` is a path and only ever a path — never an
expression — and the leading-dot shorthand that makes `.components` look
ambiguous exists only in expression position. So the lexer keeps emitting
`Ident(entry) Minus Ident(form)` and the *path* parser accepts
`Ident (Minus Ident)*` as a single segment, requiring adjacency, which
token spans already carry. `a - b` is still subtraction everywhere it
could be.

**And it is not an exception to anything**, which is the part worth
stating. The rule below already says a path segment *is* a file. So a
segment was never an identifier: it denotes a filename, and filenames
legitimately contain what identifiers cannot. `examples/BlogStyle` has
`[postId].velp`, a dynamic route segment, which is the same point made
more strongly.

Two alternatives were rejected, and the second is the interesting one:

- **Banning hyphens in file names.** Simplest, no new rule, and it costs
  a filename convention that people reach for — the corpus already shows
  someone working around its absence, with `components/feeditem.velp`
  spelling `FeedItem` run together in lowercase because no separator
  worked.
- **Mapping a hyphenated name to an identifier automatically**, so
  `entry-form.velp` imports as `.entryForm` or `.entry_form`. Rejected on
  *"either"*: two spellings for one file means two module paths, and
  one-file-per-module-path is what keys the no-cycles graph and the
  direct-dependency check. Picking one spelling fixes that and leaves
  three costs — the mapping is not injective (`vk-types` and `vk_types`
  both reach `vkTypes`, so co-existing files need a collision rule),
  `grep` stops finding the file from the import and the import from the
  file, and the precedent is genuinely split: types and functions are
  camelCase while 7 of 14 package names are snake_case
  (`hello_world_blog`, `link_shortener`, `social_feed`, `quad_tree`).

**Still open: what else a filename may contain.** Hyphens are decided.
`[postId].velp` is a routing question — a route file is discovered by the
router rather than imported — and routing resolution is undesigned, so
whether such a file is ever a path segment is not settled here.

### No nested namespace imports

The corpus has exactly one import whose last segment names a *type*
rather than a file:

```
import { FontWeight } from elp.app.ui.elements.Text
```

**Rejected.** A path segment is always a file. If you want something
associated with a type, import the type and use member access — which
the corpus already does elsewhere (`UUID.v4()`).

The reason is resolution, not taste. Permitting it makes every import
ambiguous between "a deeper module" and "a member of a type", resolved
by trying one and falling back to the other. A fallback in the name
resolver is the kind of rule that is fine on day one and unexplainable
on day four hundred, and the convenience bought is a member access.

*This closes the "nested namespace imports" open question in
`grammar-script-notes.md`.*

## Conformances are global; naming needs an import, dispatch does not

Rust requires a trait to be in scope to call its methods. Swift does
not. **elp follows Swift, and the orphan rule is what makes that safe:**
there is at most one conformance of `I` for `T` anywhere in a program,
so importing the interface adds friction and no information — it cannot
change which conformance is selected, because there is only ever one.

The split:

- You must import an interface to **name** it — a generic bound, a type
  position, a conformance declaration.
- You need not import it to **use** a conformance. If you hold an
  `ApiError`, `.toString()` is callable whether or not `Error` is in
  scope.

`functions.md` leans on this harder than it first appears: because a
method may only be declared by its receiver's owning package, a local
interface plus a conformance is the *only* way to add behaviour to a
foreign type — and this rule is what makes that addition ambiently
callable rather than needing an import at every call site.

This retroactively supplies the missing reason for a rule already
decided in `interfaces.md`: `export Foo implements Bar` is a parse
error. The justification recorded at the time was "conformance is its
own declaration". The real reason is that a conformance is never
module-scoped in the first place, so `export` has nothing to do.

## The prelude: what is atmospheric

Some types are **atmospheric** — available in every file with no import.
The question is which, and the answer must come from a criterion rather
than a list, because a list grows every time somebody finds their
favourite type inconvenient to import.

### Frequency is the wrong criterion, and the corpus proves it

The obvious rule is "if it is in most files, prelude it". Counting files
that mention each candidate, out of 24:

| | files | | files |
|---|---|---|---|
| `Text` | **13** | `Error` | 6 |
| `nil` | 11 | `Ok(` / `Err(` | 5 |
| `Component` | 9 | `Optional` | 4 |
| `Column` | 9 | `Either` | 3 |

The most common identifier in the corpus is a *widget*. Frequency would
put `Text` and `Column` in the prelude and leave `Either` out — which
would make the component library unswappable and solve nothing. The
rule has to be about kind, not count.

### The criterion

> **A type is atmospheric if the compiler can name it in a diagnostic
> without your help.**

This is self-limiting in the way a prelude needs to be. The compiler's
vocabulary is fixed by the language and cannot grow when the standard
library does, so neither can the prelude. It is also the honest test:
if the compiler is going to say `expected Optional<string>` whether or
not you imported `Optional`, then the import was never what made the
name meaningful to you.

### What passes

| | why the compiler already knows it |
|---|---|
| primitives, `[T]` | grammar |
| `Optional` | `T?` and `nil` are sugar for it |
| `Either` | `throw` is `return Err(...)`; the compiler checks the enclosing fn returns one |
| `Copy`, `Move`, `Sendable` | the compiler enforces conformance and reports failure by name |
| `Equatable`, `Comparable`, `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`, `Index` | what `==`, `<`, `+`, `-`, `*`, `/`, `%`, unary `-` and `a[i]` desugar to (`operators.md`) |
| `Ordering` | what `compareTo` returns, so `<` lowers through it |
| `Protected` | compiler-enforced secrecy, not a library wrapper |
| `Component` | "a component returns `Component` or nil" is a compiler rule |
| `StyleSheet` | what a `stylesheet { }` block evaluates to |
| `PureFn`, `ImpureFn` | the types `fn` and `pure fn` introduce |

### What fails, and stays imported

`Error`, `Http`, `UUID`, and every widget.

`Error` is the instructive one. It feels intrinsic, it is in 6 of 24
files, and it is still an ordinary library interface: `Either<T, E>`
takes any `E`, and `throw` does not require `Error`. The compiler has
no rule that mentions it, so it does not qualify. The corpus already
imports it in all six files.

### One prelude, not three

The same set in `.elp`, `.velp` and `.selp`. `Component` is most often
named in a `.velp` file and `StyleSheet` in a `.selp` one, but naming
either from an `.elp` file is legitimate, and the set is small enough
that splitting it would buy a second rule and nothing else.

### `T?` and `Optional<string>` are the same

An earlier draft had `Optional` imported, which made `password string?`
free and `password Optional<string>` cost an import line. Two spellings
of one type behaving differently was a symptom that the rule was wrong,
not a seam worth keeping.

### The keyword and the type are different things

`stylesheet` is a keyword; `StyleSheet` is a type. The block form
declares one, the type lets you pass one between components. Neither is
imported, and they are not in competition.

This is a general pattern in elp rather than a special case for styles:

| declaration form | the type it introduces |
|---|---|
| `component Name { }` | `Component` |
| `stylesheet { }` | `StyleSheet` |
| `fn` / `pure fn` | `ImpureFn` / `PureFn` |

The corpus states the third one already, in a comment in
`ChatRoom/main.velp`: *"the type of a function is ImpureFn and a pure
function is PureFn. These types are type compatible."* The rule was
there before it was named.

It also sits on a position the language has already taken. `macros.md`
makes a macro a `pure fn` evaluated at compile time over the AST, with
types and interfaces as ordinary first-class values — code as data, in
the Lisp sense. A declaration form that both introduces a construct and
yields a value describing it is the same commitment at the level of
syntax.

*Current corpus spelling is `Stylesheet` imported from
`elp.app.ui.styling`, used as a block in two `.velp` files. Both the
import and the capitalisation are migration items.*

## No import cycles, at any granularity

**Any cycle in the file import graph is a compile error**, within a
package as much as between packages. The fix is always the same: extract
the shared declarations into a new file and import it from both sides.

This is stricter than Go (which permits cycles within a package) and
much stricter than Rust (where `mod` siblings see each other freely).

**The cost, stated plainly:** two types that refer to each other must
live in the same file. There is no arrangement in which `user.elp`
names `Message` and `message.elp` names `User`.

The corpus already complies — the intra-package graph across all three
examples is acyclic, and `ChatRoom/types/chat.elp` already co-locates
`User` and `Message` for exactly this reason. So the rule costs the
existing code nothing, and buys a file-level dependency order that is
a topological sort with no cycle-breaking pass anywhere in the
compiler.

## Module visibility

`export` is the visibility modifier, and it is module-level: an
unexported top-level declaration is private to its file. The corpus
matches — `pure fn AppReducer` in `ChatRoom/main.velp` is unexported
and consumed only by the `@App` macro in that same file.

There is no package-internal tier. A declaration is file-private or it
is public, and the `internal/` directory convention in `BlogStyle` and
`SocialFeed` is a naming convention with no enforcement behind it.

## Imports are explicit, always

- **No glob imports.** No `import * from x`.
- **No default exports.** Every import names what it binds.
- **No implicit re-export.** Importing a name does not re-export it.
- **Collisions are an error**, fixed with `as`:

  ```
  import { Message as MessageView } from .components.message
  ```

  which the corpus already uses, in `ChatRoom/main.velp`, to
  disambiguate the `Message` *type* from the `Message` *component*.

## One namespace for every importable name

Types, interfaces, enums, components, functions and **macros** all live
in a single namespace. There is no separate macro namespace, no separate
type namespace, and no disambiguation by position.

So two things called `Text` — a component and a type — cannot both be
imported bare. One of them is renamed at the import site:

```
import { Text } from elp.components
import { Text as IText } from elp.types
```

This is the same `as` that already resolves
`Message as MessageView` in `ChatRoom/main.velp`; the rule simply does
not care what *kind* of thing is colliding.

### A macro is an ordinary name

`@CodeGen(Codec<JSON>)` resolves `CodeGen` in exactly the namespace
above. The `@` marks the invocation, not a different lookup, which is
the position `macros.md` already takes — a macro is a `pure fn`
evaluated at compile time, with types and interfaces as first-class
values.

This was the test case that prompted the rule, and applying it showed
the corpus had the relationship inverted: `@CodeGen` is invoked nine
times and imported nowhere, while `Codec` and `ToString` — the
*interfaces* passed to it as arguments — were being imported from
`elp.macros`. Now fixed in four files: `CodeGen` comes from
`elp.macros`, and the interfaces it derives come from `elp.types`.

The `Codec`-imported-twice collision noted earlier therefore dissolves.
It was never two things sharing a name; it was one name imported from
the wrong module.

## Screens are not importable

`BlogStyle/screens/blog/[postId].velp` cannot be written as a module
path — `[postId]` is not an identifier. That is not an accident to work
around.

**Screens are reachable by routing only, and are strict consumers of
the rest of the package.** Nothing imports a screen; a screen imports
whatever it needs. The `screens = "./screens"` manifest key names the
directory the router owns.

This makes the dependency direction one-way by construction, which is
also what keeps the no-cycles rule cheap: the routing layer can never
participate in a cycle because nothing points into it.

Routing therefore needs its own resolution story — path patterns to
files, and parameter extraction — which is not name resolution and is
not designed yet.

---

## Dependency resolution

### The import root is the dependency's first argument

```
@Package(
	name = "social_feed",
	version = .new(1, 0, 0, 0),
	entry = "main.velp",
	targets = [.iOS, .Android, .Web],
	dependencies = [
		.stdlib("1.0.0"),
		.path("design", "../design-system"),
		.git("http2", "https://github.com/x/http2", tag = "v2.1.0"),
		.registry("vendorUi", "acme/ui", "3.0.0"),
	],
)
```

`design = .path(...)` makes `import { Button } from design.button` work
regardless of what that package calls itself.

**A list of a `Dependency` enum, not a map.** The corpus wrote
`dependencies = { elp = "1.0.0" }`, and that shape does not survive the
decision that a manifest is ordinary elp code — it has to typecheck.
Two problems with it:

1. Arbitrary keys need a map type. There is no `Map` in the language,
   and this is the only brace-literal with non-field keys in the whole
   corpus.
2. Once values vary — a version string beside a path record — they are
   heterogeneous, and `generics.md` rejected the inference-driven
   coercion (`"1.0.0"` → `Dependency`) that would paper over it.

A list of enum-shorthand values needs no new language feature. It is
the same construct as the `targets = [.iOS, .Android, .Web]` line
directly above it, it is homogeneous, and putting the import root in
first position makes the key-is-the-import-root rule visible instead of
hiding it in a map key.

This also means `name` in a manifest is **publishing identity only** —
what a registry calls the package, never what an importer calls it.
Nothing in a package's own source depends on its `name`, because its
own modules are reached with a leading dot.

### `kind` says what the package produces

`targets` conflated two axes: where a package runs, and what it
produces. A CLI or a server still runs on a platform — it just isn't an
app bundle.

```
kind    = .Binary,
targets = [.macOS, .Linux],
```

`kind` is one of `.App` (the default, so existing manifests stay valid),
`.Binary`, or `.Library`. A package is one kind of thing; one that is
both an app and a CLI is two packages.

`.Library` probably takes no `targets` at all — a library's platforms are
whatever its consumer's are, which `examples/TextSearch` makes obvious.

This replaces the invented `targets = [.Server]` in
`examples/LinkShortener/Package.elp`. A server is not a platform in the
way iOS, Android and Web are; it is a standalone binary that happens to
listen on a socket.

*Open: whether `.Binary` + `.Web` means wasm or is simply an invalid
combination.*

### Every variant takes a `condition`

```
.registry("material", "google/material", "latest", condition = target == .iOS)
```

A dependency may be conditional on the build target. The condition is
an ordinary boolean expression evaluated at build time, not a special
manifest sub-language.

**The consequence: the dependency graph is per-target.** `targets`
lists three platforms, so the manifest is evaluated three times and
resolution runs three times. "Single version per package" below is
therefore *per package per target*, and a lockfile records one graph
per target rather than one graph.

*The exact spelling of the condition is pending the operator design.
The corpus contains no `==` at all, so whether elp distinguishes `==`
from `===` — equality from identity — is undecided, and it is not a
question to settle in passing. It matters more than it looks: cells are
`Rc` handles and `by` keys are identity-ish, so there is a real
candidate for the distinction.*

Note also that a condition is just the boolean. `condition = if target
== .iOS` adds nothing over `condition = target == .iOS`; `if` as an
expression already exists in the language (the corpus has
`Text(if posting { "Posting..." } else { "Post" })`) and earns its place
when there are two values to choose between, which a condition does not
have.

### One version per package, forced by global conformances

**Two majors of a package may not coexist.** A version conflict is a
build error a human resolves, never a silently duplicated dependency.

This is not a preference; it is already implied by a decision above.
Global conformances rest on the orphan rule — at most one conformance
of `I` for `T` anywhere in a program. Two copies of a package means two
`Error` interfaces and two conformances of it, and that guarantee is
gone, taking with it the reason dispatch needs no import. Binary size
on mobile points the same way, but the conformance argument is decisive
on its own.

### Path dependencies

A path dependency has no version to solve — the path *is* the pin.
Identity is the resolved directory, so two entries pointing at the same
place are one package, and a path dependency that also appears in the
registry graph is a conflict to report rather than to unify.

This is what makes a monorepo the easy case rather than a special one:
a local package is a dependency whose value happens to be a path.

### Native libraries — `links` is an allow-list, not a dependency

`@external(.OBJC) fn OBJC_msgSend<T: OBJC_Obj + Sendable>(...)` already
exists (`BlogStyle/internal/api.elp`) and already parses as a body-less
declaration. What it never said is **which shared library the symbol
comes from**, or how that library is found. `examples/SqliteBinding`
needs both, because libsqlite3 is in the SDK on macOS and iOS, a package
install on Linux, and a platform library on Android.

Three separate questions, and conflating any two of them is what makes
this awkward in other toolchains:

| question | where it is answered |
|---|---|
| may this package bind native code at all, and what? | **`links` on `@Package`** |
| which allowed library does *this symbol* come from? | **`@external`'s arguments** |
| does the build *acquire* the library, or expect to find it? | **`dependencies`**, via `.native(...)` |

```
@Package(
	name = "sqlite",
	version = .new(0, 1, 0, 0),
	entry = "db.elp",
	kind = .Library,
	dependencies = [
		.stdlib("1.0.0"),
	],
	links = [
		.system("sqlite3", binding = .dynamic, minimumVersion = "3.35.0"),
	],
)
```

```
@external(.C, library = "sqlite3") {
	fn sqlite3_open(path &CString, out &OpaqueHandle) -> int
	fn sqlite3_close(db &Sqlite3) -> int
}
```

The block form is `macros.md`'s. A macro receives the AST of what follows
it, and a declaration list is an AST node, so a package that links one
library names it once. It is not an FFI feature, and the braces are not a
scope because they are not in the program at all — they are the generator's
input, and what reaches the module is ordinary module-level declarations.

#### Why `links` is its own property rather than a `dependencies` variant

`dependencies` carries a **version algebra**: a resolver, a transitive
graph, semver, and "one version per package" above. A system library has
none of it. There is no manifest to read, no transitive graph, and no
version to *choose* — you cannot ask the OS for a different libsqlite3.
Putting it in `dependencies` imports an algebra that does not apply, and
version resolution is precisely the part that breaks.

It is also the property most worth reviewing. `links` is the complete
list of native surface a package can reach, in one place in one file, so
adding native code is a manifest change rather than a line buried in a
`.elp` file — which is where review attention already goes. That is why
it reads as permission ("what this package *may* link against") rather
than as an instruction.

**`.native(...)` in `dependencies` is still wanted**, for the other kind:
a native library the build genuinely acquires — fetched, vendored, or
built from source. That one *does* have the full algebra (a source, a
checksum, a version to pick, a build recipe), so it belongs with the
other dependencies, and it **satisfies** a `links` entry rather than
replacing it. A `links` entry with no matching `.native` dependency means
"expect this from the system or SDK".

#### `binding` does not reopen the question `dispatch.md` defers

`dispatch.md` leaves "static vs shared linking" undecided for **elp's own
code**. `binding` says how a **foreign** library attaches, which is a
different decision and not ours to defer: the library dictates it. A
system framework cannot be statically linked whatever elp prefers.

`.dynamic` is link-time dynamic, `.static` links an archive,
and `.runtime` is `dlopen` — see the open items.

#### Per-target variation, and why an entry needs a logical name

`condition` already exists, and the dependency graph is already
per-target, so the manifest side needs nothing new:

```
links = [
	.system("sqlite3", binding = .dynamic),
	.system("sqlite3", binding = .dynamic, pkgConfig = "sqlite3",
		condition = target == .Linux),
]
```

**But `condition` alone breaks the thing it feeds**, which
`examples/VulkanTriangle` found and a single-library package could not.
Vulkan on Linux is `libvulkan.so.1`. On macOS there is no Vulkan driver:
it is MoltenVK, a translation layer to Metal, shipped as
`libMoltenVK.dylib`. The symbols are identical and the file is not — so a
declaration written `@external(.C, library = "vulkan")` matches one entry
and not the other, and the name a declaration refers to cannot be the
file.

**So an entry's first argument is a logical name, and its realisation is
separate:**

```
links = [
	.system("vulkan", soname = "libvulkan.so.1", condition = target == .Linux),
	.system("vulkan", soname = "libMoltenVK.dylib", condition = target == .macOS),
]
```

`library = "vulkan"` then resolves on both, and the per-target difference
stays where per-target differences already live. Three things follow:

- **The logical name is what the allow-list is keyed on**, so the
  exclusive claim above is a claim on the *logical* name. Two entries
  naming `vulkan` under disjoint conditions are one claim, not two.
- **`soname` is optional and defaults to the logical name**, which is why
  `sqlite3` above never needed it. The common case stays one argument.
- **It is also the honest spelling for a renamed library** — `libz` vs
  `zlib1` on Windows — rather than a Vulkan special case.

#### A library's `links` constrains its consumers' targets

This partially answers the open item above — *"`.Library` probably takes
no `targets` at all"*. A library with no native surface takes none,
because its platforms are whatever its consumer's are. **A library with a
`links` entry is different:** `.Web` has no shared libraries and no
`dlopen`, so a package that links one cannot be built for Web, and
neither can anything that imports it.

So `links` is the first thing that gives a `.Library` real target
constraints, and they propagate to consumers. `examples/SqliteBinding`
spells its `targets` out for that reason.

#### A `links` entry is an exclusive claim

**At most one package in a program may link a given native library.** Two
packages both naming `.system("sqlite3", ...)` is a resolution error, in
the same place and of the same kind as two majors of one package above.

The problem it answers is that two packages may each declare
`@external(.C, library = "sqlite3") fn sqlite3_open(...)` with *different
signatures*, each locally correct, and the linker resolves by name — so
both get the same address and one of them is wrong.

The orphan rule cannot be stretched over it, and the reason is exactly
that **there is no contract on the far side of a shared library.** The
orphan rule works because elp owns both operands: some package defines
`T`, some package defines `I`, and "only those two may declare the
conformance" names a real party in both cases. Nobody defines
`sqlite3_open`. It is a name in a symbol table with no types attached and
no author to appeal to, so a rule of the form "only the owner may declare
it" has nothing to bind to.

The uniqueness therefore has to be imposed one level up, on the only
elp-side entity there is: **the symbol has no owner, but the library can
have exactly one claimant.** That is the orphan rule's shape — one
declaration site, enforced at resolution, so nothing downstream needs
disambiguating — moved from the type to the package.

Cargo reached the same rule under the same name: its `links` key must be
unique across a dependency graph. What makes it liveable there is the
`-sys` crate convention, one binding package per library that everyone
else depends on. elp makes that convention the rule, which is the right
way round, because the convention is what makes the rule tolerable and
leaving it optional is the bad half.

**It obliges stdlib to claim the universal ones.** libc, libSystem,
Foundation and the Android platform libraries are reachable from too many
places for a first-come claim to be sane, so they are stdlib's and no user
package links them.

#### What exclusivity leaves, and the whole-program check

Two cases survive it, and one rule catches both: **within a program, all
declarations of the same symbol in the same library must have identical
signatures.**

| case | caught by |
|---|---|
| two packages declaring one library's symbols | exclusivity, at resolution |
| one package disagreeing with itself across files | the agreement check |
| two *different* libraries exporting the same name | the agreement check |

elp can run that check where Rust cannot, and for something it has already
paid for: the build is whole-program and resolution is global, so every
`@external` declaration in the program is in hand at once. Rust's
`clashing_extern_declarations` lint is per-crate on purpose — separate
compilation never sees the other crate's declaration.

The escape hatch for the third case is a **`symbol =` argument**, so the
elp name and the linker name may differ:

```
@external(.C, library = "mylib", symbol = "init") export fn mylibInit() -> int
```

which is wanted anyway for a C name that is not a valid elp identifier,
and is what makes the agreement check's error fixable rather than fatal.

**What none of this fixes.** Every declaration agreeing with every other
declaration does not mean any of them agrees with the library. No header
is being read and a symbol table carries no types, so a unanimous wrong
signature is still wrong and still corrupts memory. That residual is
irreducible without parsing C, and it is the honest reason `@external` is
the language's one unsafe surface rather than merely its most awkward one.

#### Open within native linking

- **A version constraint on a system dynamic library is unenforceable at
  build time.** The linker does not check it and the installed version is
  whatever is present at run time. So `minimumVersion` is either a
  build-time probe where one exists (pkg-config, an SDK manifest) or a
  promise the code must verify itself (`sqlite3_libversion()`). This is
  the same shape as `examples/SqliteBinding`'s threading-mode finding — a
  property declared statically that is only knowable at runtime — and two
  instances make it a pattern rather than an oddity.
- **`.runtime` binding changes the shape of every declaration in the
  block.** `error-handling.md` makes a failure a value, so a `dlopen`
  that fails is an `Either` — which means the symbols are not callable
  until something resolved the library, and that implies a handle on the
  library itself, with the same exactly-once release problem as any other
  foreign handle. `.runtime` may be out of scope for a first version
  rather than a third enum case.
- **Whether a claim may be delegated.** Exclusivity above says one
  package per library, which is right for a binding package and awkward
  for an application that wants two symbols of its own out of a library
  a dependency already claims. Re-exporting the dependency's declarations
  is the answer in principle; whether that needs anything beyond an
  ordinary import is not designed.

### `scripts` — build-stage jobs, as a function of declared inputs

**A `scripts` entry on `@Package` runs an external process at a named
stage of the build.** It exists because a real program has build inputs
that are not elp: `examples/VulkanTriangle` needs two SPIR-V modules,
produced by running `glslc` over GLSL source, and a real Vulkan binding
needs ~800 struct declarations generated from `vk.xml`.

```
scripts = [
	.run(
		stage   = .beforeCompile,
		command = ["glslc", "shaders/triangle.vert", "-o", "{out}/triangle.vert.spv"],
		inputs  = ["shaders/triangle.vert"],
		outputs = ["{out}/triangle.vert.spv"],
	),
]
```

#### A script is a process, and that is where impurity is allowed

`macros.md` runs `pure fn` at compile time and `pure` means no IO, no
external calls, no async — which is correct and is what keeps the
compile-time evaluator small. So compile-time *elp* cannot shell out, by
design, and nothing here weakens that.

The manifest is a different place with different rules. `scripts` is the
one point in the system that may run arbitrary processes, it is declared
rather than discovered, and it sits in the file where review attention
already goes. That is the same argument `links` is made on, and the two
properties compose: `links` bounds what native code a package may *reach*,
`scripts` bounds what processes its build may *run*.

#### Declared inputs and outputs are mandatory, not optional

This is the one place where this deliberately diverges from Cargo's
`build.rs`, and it is the whole value of the feature.

A `build.rs` is an opaque program. Cargo cannot tell what it reads, so the
default is to re-run it whenever anything in the package changes, and a
package opts out by *printing* `cargo:rerun-if-changed` from inside the
script it is trying to avoid running. The result is the most common
complaint about Cargo builds: scripts that re-run constantly, and
incremental builds that are not.

Requiring `inputs` and `outputs` makes a script **a function of its
declared inputs**, which buys three things at once: the build skips it when
the inputs are unchanged, can cache its outputs across machines, and can
run independent scripts in parallel. A script that reads a file it did not
declare is a bug the build can detect rather than a performance
characteristic nobody can see.

Omitting them is an error. "Always re-run" is available by declaring no
inputs, which is explicit and visible rather than the silent default.

#### Two kinds of output, and only one of them is finished

| output | what happens to it | status |
|---|---|---|
| **generated elp source** | compiled as part of the package, as if it had been written by hand | works — it is a file the compiler already knows how to read |
| **a generated asset** (SPIR-V, an image, a table) | has to reach the program's *bytes* | **open** |

The second is the honest limit. `scripts` closes "how do I run `glslc`";
it does not close "how do those bytes get into the binary", which needs a
way to embed a file as data and therefore needs `strings.md`'s open
byte-access question answered first. `examples/VulkanTriangle` hardcodes
an empty array where the shader bytes belong, for exactly this reason.

So the generated-source case is the one that is usable today, and it is
also the bigger one: it is what lets a Vulkan or Win32 binding be
generated from a machine-readable spec instead of hand-written, which is
the case `macros.md`'s attribute-block form was designed for and the case
no corpus package could reach.

#### The stage set is deliberately small

`.beforeCompile` and `.afterLink`: generate inputs, then act on the
finished artifact (codesign, bundle, strip). Nothing else has a witness,
and a stage nobody uses is a stage that will be wrong when somebody
finally does.

#### Open within scripts

- **Whether a dependency's scripts run at all.** Cargo runs every
  transitive `build.rs`, which means installing a package runs its
  author's code on your machine. Declared inputs and outputs make it
  auditable rather than safe. The options are to run them (and lean on the
  declaration being reviewable), to require opt-in per dependency, or to
  forbid them outside the root package — which would stop a binding
  package generating its own bindings, so it is probably too strict.
- **Whether a script's outputs are recorded in the lockfile.** They are
  build products, not dependencies, but a checksum over them is what would
  make a build reproducible across machines.
- **How `{out}` is spelled and where it points**, and whether a script may
  write anywhere else. It should not be able to.
- **Toolchain availability.** `glslc` is assumed present. A missing tool
  should be a clear build error and may want declaring, which is close to
  what `links` does for libraries and may be the same mechanism again.

### Lockfile

A separate generated `Package.lock`, in elp syntax so the toolchain has
one parser. The manifest is hand-written and the lockfile is
machine-owned and rewritten constantly; merging them means a tool
editing your source. The cost is a slower parse than a bespoke format
on a large graph.

### Registry names are scoped

`.registry("vendorUi", "acme/ui", "3.0.0")`. A flat global namespace
invites squatting and cannot be retrofitted; a scope costs nothing now.

### Only direct dependencies are importable

A transitive dependency is not in scope. If you want it, declare it.
This is Cargo's and Go's rule, and it stops an upgrade from silently
removing a name you were using.

### `elp` is a reserved root

No dependency may be keyed `elp`. The standard library is always
available at the toolchain's version; `.stdlib("1.0.0")` pins that
version and can never redirect it.

`ChatRoom/Package.elp` pins and the other two do not, while all three
import `elp.*` freely. Under this rule all three are valid.

### Open within dependency resolution

- Test-only dependencies: a separate tier, or a `condition`?
- Whether `version = "latest"` is permitted at all outside a lockfile.
- Registry protocol and package identity beyond `scope/name`.

## Open questions

- **Routing resolution.** Path patterns to screen files, parameter
  extraction, and what `[postId]` binds to. Named above, not designed.
- **Core expression language.** The `condition` argument above is the
  first thing in the design to need an operator that the corpus does
  not contain. See `docs/core-language.md` once it exists; until then,
  the gap is: operators and precedence, `Map`, array/collection
  surface, indexing, optional chaining, and whether value-level `+` is
  interface-driven (recorded as unratified in `generics.md`).
- **Stdlib module layout.** The corpus imports `Text` from
  `elp.components`, `elp.ui`, `elp.ui.elements` *and*
  `elp.app.ui.elements` — four roots for what is apparently one set of
  components. This is a stdlib organisation question, not a resolution
  one, but it must be settled before the corpus can be made consistent.
- **Does a dialect constrain what a module may contain?** Nothing yet
  says a `.selp` file may only hold styles, or that a `.velp` file may
  hold a non-component. Resolution treats all three identically, which
  may or may not be the final answer.

## Corpus migration backlog

Consequences of the decisions above, none applied yet:

1. `import { Stylesheet } from elp.app.ui.styling` in
   `BlogStyle/screens/home.velp` and `BlogStyle/components/paragraph.velp`
   — drop the import, lowercase the block to `stylesheet { }`, and
   reserve `StyleSheet` for the type.
2. `import { FontWeight } from elp.app.ui.elements.Text` in
   `BlogStyle/components/paragraph.velp` — rewrite as a member access
   on an imported `Text`.
3. `Uuid` and `UUID` are both imported from `elp.types`, and `UUID.v4()`
   is what is called. Pick one.
4. `export @Package` in `ChatRoom` and `SocialFeed`, bare `@Package` in
   `BlogStyle`. A manifest is not imported by anything, so `export` is
   meaningless on it — drop it from both.

The corpus's unimported `Optional` and `Protected` usages, previously
listed here, are **correct** under the atmospheric criterion and need
no change.
