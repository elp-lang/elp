# elp strings and characters

## The definition

**`string` is an immutable sequence of Unicode scalar values with a
guaranteed UTF-8 representation.**

**`char` is one Unicode scalar value** — a code point excluding
surrogates, so a `char` is always validly encodable and a `[char]` can
always be joined back into a `string`.

**`==` on `string` and `char` is scalar-wise. There is no implicit
normalization.**

Both are primitives: lowercase, atmospheric, never imported.

## Why scalars, and why no normalization

The decisive argument is one the design already made for another
feature. `collections.md` ratified an insertion-ordered `Map` because a
compile-time `pure fn` macro **must emit identical code every build
across all three targets**, and hash order is not stable across
processes once hashing is seeded.

Grapheme segmentation and NFC normalization tables are both
**Unicode-version-dependent**, and iOS, Android and Web ship different
Unicode and ICU versions. So:

- if `char` were a grapheme cluster, `"👨‍👩‍👧‍👦".chars().length()` could
  differ between iOS and Android for byte-identical source;
- if `==` used canonical equivalence, `"é" == "e\u{301}"` could differ
  the same way.

Either one puts a target-dependent value inside a construct the
language promises is deterministic. This is the determinism
requirement arriving a second time, not a new preference.

Three supporting reasons:

- **A grapheme cluster is unbounded in length**, so it is
  heap-allocated. `char` would stop being `Copy` and stop being a
  primitive — but `TextSearch` puts it in a `Map<char, int>` key and
  binds it with `const`, both of which want a fixed-width value.
- **Scalar values are version-stable.** An assigned code point never
  changes meaning. Grapheme boundaries do, between Unicode releases.
- **Swift chose canonical equivalence and pays for it by shipping its
  own copy of ICU.** elp targets Web, where that payload is the
  shipping constraint `dispatch.md` already flagged for binary size.

### The cost, stated plainly

`"é" != "e\u{301}"`. A user typing é on one platform and another
typing it on a second can produce strings that compare unequal, and
nothing in the language will warn them.

That is a real footgun and it is worth naming rather than burying. Two
things make it the right trade anyway: it is the same footgun Rust, Go,
Python 3, Java, C# and JavaScript all have — Swift is the outlier, not
elp — and it is **predictable**, which is the property this language
has chosen at every previous fork. Normalization is available, as an
explicit stdlib operation that says it is happening.

## Literals

String and `char` literals are grammar, so they are settled here.

| | |
|---|---|
| string | `"..."`, with `{ expr }` interpolation |
| `char` | `'...'`, exactly one scalar value |
| escapes | `\\` `\"` `\'` `\n` `\r` `\t` `\0` `\{` `\u{...}` — a fixed, closed set |

`'ab'` and `'👨‍👩‍👧‍👦'` are errors, and the diagnostic states the rule: a
`char` is one Unicode scalar value. The second case is the useful one
to get right, because it is the error a user will actually hit and the
message is where they learn what `char` means.

**The single quote was free.** It is absent from `LexerSymbol` and
there is no `Char` token (`crates/lexer/src/tokens.rs`), so nothing had
claimed it. elp has no lifetimes, so nothing was going to.

`\{` is in the set because interpolation claims the brace: a literal
`{` in a string has to be spelled somehow, and an escape is consistent
with the rest of the table rather than inventing a `{{` doubling rule
that applies to one character.

No octal escapes, no `\x` — it reads as a Unicode escape and is not one
in any language that has it — and no `\a`/`\v`/`\f`. A closed set means
an unknown escape is an error rather than a silently-dropped backslash.

Interpolation is already designed and unimplemented: the lexer emits
one opaque string token today with escapes passed through raw, and
chunked tokens are the outstanding work (`grammar-script-notes.md`
gap #9).

## What the lexer had already decided

Worth recording as decisions rather than leaving as accidents of
implementation, because all three are observable:

| | |
|---|---|
| `lib.rs:14` | `Peekable<Chars<'a>>` — source is scanned as **scalars** |
| `lib.rs:51` | `cursor += c.len_utf8()` — spans are **byte offsets** |
| `lib.rs:52` | `UnicodeWidthChar::width(c)` — columns are **display width** |

The compiler needs all three units simultaneously: bytes to slice
source, scalars to scan it, display width to put a caret under the
right column. So "bytes versus scalars versus graphemes" was never a
pick-one — the only open question was which unit `string`'s public
surface exposes, and the answer above is scalars.

One further shipped decision: **identifiers are Unicode XID plus emoji
presentation** (`lib.rs:64`), which is a more permissive alphabet than
most languages allow and should be deliberate rather than incidental.

## What this pushes to the stdlib

The language defines what a `string` and a `char` *are*; the methods
are stdlib built on top, and are not language design.

The one consequence worth stating, because it looks like a stdlib
choice and is not: **UTF-8 means a scalar offset is not O(1)**, so
`string` does not conform to `Index` and there is no `s[i]`.
`.chars() -> [char]` is the random-access path.

The corpus validates this without having been asked to. `TextSearch`
is a Boyer-Moore-Horspool implementation — the program most likely to
demand string indexing — and it never indexes a string:

```elp
export fn Pattern.find(&self, haystack string) -> Optional<int> {
	const text = haystack.chars()
	const h = text.length()
	...
	while j >= 0 && text[offset + j] == self.needle[j] {
```

`needle` is `[char]` and `skip` is `Map<char, int>`. Every index, every
`length()` and every comparison in the algorithm is on an **array**.
There is no `s[i]` anywhere in the corpus.

The four string `length()` calls are all incidental: two are emptiness
tests (`code.length() == 0`, `note.trim().length() == 0`) and one is a
bound in `path.slice(1, path.length())` that means "strip the leading
slash". **Nothing in the corpus needs to know how many units a string
has**, which is the usual objection to a non-indexable string and does
not apply here.

So `length`, `slice`, `isEmpty`, `trim`, `split`, `padEnd` and
`normalized(.NFC)` are stdlib surface, designed with the rest of it.

## Migration

`Char` → `char`, five sites, all in `TextSearch/search.elp`. `string`,
`int`, `int8` and `bool` were already lowercase; `Char` was the only
uppercase primitive. `UUID` stays uppercase correctly — it is an
imported stdlib type, not a primitive.

## Open

- **Multi-line and raw strings.** Zero corpus evidence either way. The
  lexer currently ends a string scan at a newline.
- **Is `string` `Iterable<char>`?** Leaning yes: forward iteration is
  O(1) amortised in UTF-8 even though indexing is not, so `for c in s`
  costs nothing the way `s[i]` would. A stdlib conformance, but forced
  by the definition above.
- **Byte access for I/O.** `LinkShortener` speaks HTTP, so bytes exist
  somewhere. Whether that is `[uint8]`, a distinct `Bytes` type, or
  `string`'s UTF-8 view is undecided — and the integer-width question
  is undesigned too.
- **Is `char` `Comparable`?** Scalar ordering is well-defined, so this
  is cheaply yes, but it should be a stated conformance rather than
  assumed.
- **Interpolation's chunked tokens**, outstanding in the lexer.
