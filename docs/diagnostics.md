# elp diagnostics

## The principle

> *"I'd rather filter too much information than there not be enough."*

The compiler is **noisy by default and filtered by configuration**, not
quiet by default and opted into. A developer who never touches the
settings sees everything the compiler noticed; a developer who has tuned
them sees what they asked for.

This is the C++ `-Wall` / `-Wextra` lineage rather than the "errors
only" one, and it inverts the usual default: the information exists
either way, and the question is only who decides what to look at.

## The soundness floor

**A diagnostic that reports unsoundness is always an error and can never
be configured down.** Type errors, ownership violations, conformance
failures, non-exhaustive matches with no fall-through, import cycles —
none of these are tunable at any level.

This is the guard that makes the rest of the design safe. Without it,
"configure how much the compiler tells you" quietly becomes "configure
how much type safety you get", which is how `-Wno-error` ends up in a
build script and stays there for four years.

So every diagnostic sits in one of two tiers:

| tier | what it covers | configurable |
|---|---|---|
| **error** | soundness: the program has no valid meaning | no, ever |
| **lint** | style, risk, redundancy, future breakage | yes, `allow` → `warn` → `deny` |

A lint may be raised to `deny` (fail the build). An error may not be
lowered. There is no `-Wno-type-error`.

## Every diagnostic has a stable name

Filtering requires something to name, so each lint carries a stable
identifier — `unreachable_pattern`, `foreign_enum_exhaustive`,
`truncated_projection`. Names are API: renaming one breaks every
manifest that mentions it.

Lints are also grouped, so a developer can move a whole family at once
rather than enumerating members.

## Where the level is set

Three places, narrowest winning:

1. **The manifest**, since it is already elp code:

   ```
   @Package(
   	name = "expenses",
   	lints = [
   		.deny(.unreachable_pattern),
   		.allow(.foreign_enum_exhaustive),
   	],
   )
   ```

2. **In source**, on the declaration it concerns:

   ```
   @allow(unreachable_pattern)
   fn Category.label(&self) -> string { ... }
   ```

3. **The command line**, overriding both, for a one-off build.

**Lint configuration does not cross a package boundary.** A dependency
cannot silence a lint in your code, and you do not inherit its
settings — you see diagnostics about *your* source under *your* rules.
This follows from the manifest being per-package and is worth stating
because the alternative (inherited lint config) is a common and
miserable failure mode.

## Compile time and run time are different things

`reactivity.md` commits to a dev-mode watchdog warning when a turn
exceeds a frame budget, and notes it is "a runtime feature rather than a
language one". That is a third category and it deserves its own
vocabulary — a **runtime diagnostic**, present in dev builds and absent
from release ones.

| | when | configured by |
|---|---|---|
| error | compile | not at all |
| lint | compile | manifest / attribute / CLI |
| runtime diagnostic | dev-build execution | build profile |

## The exhaustiveness lints

`error-handling.md` settles the rule: a match with no catch-all must be
exhaustive, and writing `_` opts out. Two lints sit around that rule
rather than inside it.

### `unreachable_pattern` — warn by default

A catch-all on a match that is already exhaustive. The arm can never
run, so it is either a mistake or a defence the author wanted; a lint
says so without deciding which.

### `foreign_enum_exhaustive` — warn by default

An exhaustive match, with no catch-all, over an enum **declared in
another package**.

This is the forward-compatibility risk, and a lint is the right register
for it: the match is correct today and will stop compiling if the
upstream enum gains a variant. Saying so is informative. Making it an
error would be a lie, because the code is fine.

It is also narrow by construction — it cannot fire on a match over your
own enums, which is the overwhelming majority of them, so it stays
quiet in the case where exhaustiveness is exactly what you want.

**This is where the package boundary belongs.** An earlier draft made a
catch-all *mandatory* across package boundaries; that was too strong,
because elp pins one version per package and makes conflicts a build
error, so a new variant can only ever arrive through a deliberate
upgrade. The break is visible and informative at the moment you choose
it. A lint flags the exposure; it does not forbid it.

### Not a lint

An exhaustive match with no catch-all over your own enum is correct
code and says nothing. `Category.label` and `PatternError.toString` are
silent, and the day a sixth variant is added they are a hard error —
which is the whole point.

## Commitments already made elsewhere

Scattered through the other docs, collected here so they have a home:

| diagnostic | source | tier |
|---|---|---|
| extra fields in a structural projection are truncated | `generics.md`, and a comment in `ChatRoom/main.velp` | lint — `truncated_projection` |
| a turn exceeds the frame budget | `reactivity.md` | runtime diagnostic |

`grammar-script-notes.md` also contains a deliberate *rejection* of a
lint: accessibility state is derived from props rather than passed
separately, "making the drift unrepresentable rather than lintable".
That is the preferred move whenever it is available — a lint is what you
reach for when the design cannot make the mistake impossible.

## Open

- **Default level of each lint**, which can only really be set once
  there are lints and a corpus noisy enough to judge them against.
- **Lint groups**: what the families are, and whether there is an
  `all` / `extra` pair in the C++ style or Rust's category names.
- **Whether `deny` in a manifest is honoured when the package is built
  as a dependency**, or only when it is the root. Cargo's answer is
  "only the root", and it exists because the alternative makes other
  people's strictness your build failure.
- **Attribute spelling.** `@allow(name)` reuses the macro-attribute
  syntax, which raises the same macro-vs-type namespace question already
  open in `core-language-probes.md`.
