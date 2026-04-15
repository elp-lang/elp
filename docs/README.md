# Elp grammar & spec docs

This directory holds the language specification artifacts. None of these
files are consumed by the compiler — the production lexer and parser are
hand-written in `crates/lexer` and `crates/parser`. These docs exist to
drive diagrams, test rigs, and (eventually) a human-readable reference.

## Files

| File | What it is | When to use it |
|---|---|---|
| `grammar-script.g4` | ANTLR4 grammar, script dialect (`.elp`) | Railroad diagrams, `grun` test rig, ambiguity reports |
| `grammar-script-notes.md` | Language-design notes: intentional gaps, use-site rules, semantic constraints | Read before arguing with a parse result |

## Working with the ANTLR grammar

### One-time install

```bash
pip install antlr4-tools
```

That gets you `antlr4` (the generator) and `grun` (the test rig). Both
pull a pinned ANTLR jar on first use, so no separate Java install step.

### Generate a throwaway parser

```bash
antlr4 -Dlanguage=Java docs/grammar-script.g4 -o /tmp/elp-grammar
```

This produces a Java parser under `/tmp/elp-grammar/`. You never use the
generated code directly — it exists only so `grun` has something to run
against. Regenerate whenever `grammar-script.g4` changes.

### Test a real `.elp` file

```bash
# Parse tree as an s-expression
grun Elp script -tree < examples/BlogStyle/internal/api.elp

# Parse tree in a Swing window (needs a display)
grun Elp script -gui < examples/BlogStyle/internal/api.elp

# Ambiguity + unreachable-alternative reports while parsing
grun Elp script -diagnostics < examples/BlogStyle/internal/api.elp

# Token stream only, no parsing
grun Elp script -tokens < examples/BlogStyle/internal/api.elp
```

`-diagnostics` is the most useful flag day-to-day. It runs ANTLR's
adaptive-prediction with extra instrumentation and reports any point in
the input where the grammar had to resolve an ambiguity. Those reports
translate directly to "your hand-written parser will also have to make
a choice here — make sure it's the right one".

You can also combine flags: `-tree -diagnostics` prints the tree and any
ambiguity warnings to stderr.

`grun` reads from stdin by default. Pass a filename as a trailing argument
if you prefer.

### Railroad diagrams

**IntelliJ / Android Studio ANTLR plugin.** Open `grammar-script.g4`,
right-click any rule in the editor gutter, choose "Test Rule" → "Show
Railroad Diagram". Best experience by a wide margin. Also shows live
parse trees as you type input in the plugin's tool window.

If you want a throwaway diagram without installing the plugin, the
`antlr4-tools` package includes a `pygrun` command with a `-gui` flag,
and `antlr4-parse` can emit diagram SVGs — both run off the same `.g4`.

### Typical workflow when changing the grammar

1. Edit `crates/parser/src/...` (the production change).
2. Edit `docs/grammar-script.g4` to match.
3. Run `cargo test -p parser` to catch parser regressions.
4. Run `grun Elp script -diagnostics < examples/BlogStyle/...` on the
   example corpus to confirm the spec grammar still accepts it.
5. If the two diverge, pick one as the intended shape and fix the other.
   The hand-written parser is the final authority; the `.g4` is a spec
   that chases it.

The divergence check is not yet automated. A later chore ticket: wire up
a test that invokes `grun` on every file in `examples/` and asserts a
clean parse.

## Caveats

Before spending time on an unexpected parse failure, skim the "Caveats"
section at the bottom of `grammar-script.g4`. The spec grammar is a
deliberate simplification of the real language — it hides `NEWLINE`,
treats strings as opaque (no interpolation), reserves `from` and `as`
fully, and uses ASCII-only identifiers. Eight known divergences are
documented there. The prose companion is `grammar-script-notes.md`.
