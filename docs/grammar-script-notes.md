# Elp script grammar — design notes and intentional gaps

**Language-level decisions** — what Elp does and doesn't do as a
language, independent of how any parser implements it: design decisions,
features that are deliberately absent, and rules that parse fine
syntactically but that semantic analysis must still reject.

The hand-written lexer and parser (`crates/lexer`, `crates/parser`) are
the only implementation and the final authority on what parses. There is
no separate formal grammar to diff against — an ANTLR `.g4` spec grammar
existed briefly and was deleted; it drifted from the real parser and
generated phantom design concerns faster than it caught real ones. The
genuinely language-level questions it did surface were folded into
"Open questions" below before it went.

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
   bodies lives in the view dialect, not the script dialect — script code
   always returns an `Either` explicitly. `throw` is sugar for
   `return Err(...)` and nothing more: it does not bubble anywhere and
   there is no boundary catching it. See `error-handling.md` — an earlier
   version of this note described a "root error boundary"; that mechanism
   was considered and rejected.

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
       SERVER { message string }
       USER { message string }
       UNKNOWN
   }
   ```

   (Payload fields gained names in `patterns.md`; the point about the
   bare variant identifier is unaffected.)

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
   hand-written lexer does not yet do this — it lexes strings as a
   single opaque payload with no escape or interpolation handling.

10. **Struct tags** (`` `json: "id"` ``) are accepted on any `Field`.
    Their semantics — who reads them, at what phase — are part of the
    macro subsystem. The grammar allows them everywhere; whether they
    mean anything depends on which macros are applied to the containing
    object.

## Block results — explicit `return` or a trailing expression

A block's result is its explicit `return`, or its trailing expression
when there is none. A block whose last statement is not an expression
yields unit.

This was previously stated only for component bodies, under *Markup
context* below, but it is a general rule and the corpus already relies
on it in ordinary functions — `ChatState.new` is
`fn ChatState.new(user User) -> Self { Self { ... } }` with no
`return`, while `ChatState.usersInRoom` immediately below it writes one
explicitly. Both are legal; only one is legal if a trailing expression
is not a result.

It is what makes a closure passed to `background` work without
ceremony:

```
const compressed = background(fn { compressImage(path) })
```

and `composer.velp`'s `handleSubmit`, whose last statement is an
assignment, yields unit rather than a value.

## Interfaces

11. **Superseded — see `interfaces.md` for the current model.** This
    point originally said interfaces may declare fields and method
    signatures, checked structurally (an object satisfies an interface by
    shape, no explicit declaration required). That's no longer accurate:
    interfaces are method-only (no field requirements at all) and
    conformance is nominal (an explicit `implements` declaration is
    required; structural coincidence doesn't count). The change came out
    of designing `Copy`/`Move`, but it applies to every interface.

## Things the grammar accepts that semantic analysis must still reject

The parser's grammar is intentionally looser than valid Elp in a few
places, so it can give targeted errors instead of generic ones:

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

## Markup context — `component`, `stylesheet`, and what `Name { ... }` means

**Decided, not yet implemented.** This section resolves the
object-literal-vs-builder-block ambiguity and the `if`-condition
ambiguity together, because they turned out to be the same problem.

### The problem

`Name { ... }` is one syntax with three meanings — an object literal
(`Self { a = 1 }`), an element (`Column { Text("hi") }`), and a style
rule (`Text { padding = 16 }`) — and no rule separated them. The parser
guessed from the shape of the first entry inside the braces. Probing it
against corpus-shaped input showed that guess failing four ways:

| Source | Was parsed as | |
|---|---|---|
| `Column { child }` | object literal | silently wrong |
| `Column { x = 1 ⏎ f() }` | — | parse error |
| `Stylesheet { Text, Date { padding = 16 } }` | — | parse error |
| `Text, Date { padding = 16 }` | — | parse error |

The collision is not resolvable by lookahead: `Self { type }` (shorthand
field init) and `Column { child }` (one-child element) are
*token-identical*. Something other than the token stream has to decide.

### The decision

**Three declaration forms, not one.** A component is its own declaration
keyword, and so is a stylesheet:

```elp
component Name(params) { ... }      // markup-producing
stylesheet { name { prop = v } }    // named styles only
fn name(params) -> T { ... }        // value-producing, unchanged
```

**A component declares no return type.** This is the part that does the
most work. It removes the `Component` / `App` / `Element` inconsistency
in the corpus as a *category* rather than fixing it file by file, and it
means a custom type that conforms to `Component` (for lifecycle
handlers, say) never has to be *deduced by the parser* — conformance
becomes a typechecker fact and nothing else.

**Markup context is established by the `component` keyword and
propagates by position:**

- A component *body* is an ordinary statement block — props (`export
  var`), reactive locals, nested `fn`s. Unchanged from today.
- The body's **result expression** — explicit `return`, or the trailing
  expression — is parsed in **markup context**.
- In markup context: a bare `Name { ... }` is an **element**; `if` /
  `match` / `for` stay in markup context through their bodies and arms;
  an element's `{ ... }` block contains markup nodes.
- **Call arguments and the right-hand side of `=` revert to value
  context.** `Row(class = "error-row")` is a value expression even
  though it sits inside a tree, so `Style { color = red }` there is an
  object literal.
- Everywhere outside markup context, `Name { ... }` is an object
  literal. Full stop, no lookahead.

**`Slot` becomes a keyword** rather than an identifier, which removes
`Column(class="x") { Slot }` from the ambiguity entirely instead of
leaving it to context.

### Why context is required, with the evidence

"Elements alone are enough" was tested and fails. Of the nine components
in the corpus, only four have an unambiguous call-receiver root
(`App(...) { }`, `Column(lazy = true) { }`, `Form(...) { }`, and
`Text(text)` with no block). The other five need context:

- `About`, `BlogPost`, `Home` — root is a bare `Column { ... }`.
- `ChatApp` — root is an `if / else` whose branches yield markup.
- `Message` — root is a `match` whose arms yield markup.

The last two are why context has to propagate through control flow in
*expression* position. Nono (`/Users/dave/www/nono-dist`, the static-HTML
predecessor this design borrows from) never needs that: its `if`/`for`/
`match` are markup *nodes* inside a block, so they are always already in
context. elp puts them at the component root, where nothing has
established context yet.

**Control flow produces values, exactly as in Rust** — `if` and `match`
are expressions, and that is what lets them sit at a component root at
all. One divergence worth naming: in markup context a `for` yields a
*sequence of nodes* (`for post in posts by post.id { Row(...) { ... } }`
emits one row per post), where Rust's `for` yields unit. That's a markup-context
rule, not a change to `for` in value context.

### What this borrows from nono

Verified against `src/nono.pest`, which is nono's authoritative grammar:

- `item = _{ stylesheet | component | function | const_decl }` —
  stylesheet and component are top-level keywords, not expressions.
- `component = { "component" ~ ident ~ params? ~ block }` — no return
  type.
- `node` ordering puts `local_const` and `named_fill` before `element`,
  because all three can start with an identifier. elp needs the same
  ordering discipline.
- nono has **no bare assignment** inside a block — mutation requires
  `const`. That is the principled answer to `Column { x = 1 ⏎ f() }`:
  inside markup, `x = 1` is not an assignment statement, and `name = {
  ... }` is a named slot fill rather than either.

### The dialect bounds the problem — `component` is `.velp`-only

**View, business logic and styling are legally separate, by design**, and
the file extension is what enforces it:

| | `.elp` — script | `.velp` — view | `.selp` — style |
|---|---|---|---|
| `import`, `const`, `object`, `interface`, `enum`, `fn`, `implements` | ✅ | ✅ | imports only |
| `component` | ❌ | ✅ | ❌ |
| `stylesheet { ... }` block | ❌ | ✅ | implicit — the file *is* one |
| Can `Name { ... }` be markup? | **never** | yes | **never** (always a style rule) |

Borne out by the corpus: `.velp` carries every declaration form `.elp`
does — `object` (`[postId].velp`), `interface` and `enum`
(`login-form.velp`), `pure fn` and `const` (`main.velp`, `home.velp`) —
plus `Stylesheet` blocks. `.selp` has imports and style rules and
nothing else.

The consequence for the parser is larger than "dialect labels the file":

- In `.elp`, markup context can never be entered, so `Name { ... }` is
  unconditionally an object literal. No context tracking at all.
- In `.selp`, `Name { ... }` is unconditionally a style rule.
- **Only `.velp` needs the markup-context machinery**, because only
  `.velp` can hold script, styles and components at once. It is the
  hard case and the only hard case.

So the dialect doesn't *solve* the ambiguity — it lives in `.velp`,
which is exactly where the dialect stops narrowing — but it does bound
it to one of three file types, and it turns `component` and
`stylesheet` into dialect-gated keywords that can produce a real
diagnostic ("`component` is only legal in a `.velp` file — view and
business logic are separate by design") instead of a syntax error.

**Implementation note:** the parser is currently dialect-blind.
`ElpScript::from_tokens` takes no `ElpScriptType`, and `ElpParser::parse`
knows the extension only because it globbed for it — it doesn't pass it
down. Threading the dialect through is a prerequisite for this design,
and it's the smallest piece of it.

### What a markup block contains

A markup block admits these node forms and nothing else:

```
node = element | if | match | for | Slot | const
element = IDENT arguments? block?
for = 'for' IDENT 'in' expr 'by' expr block
```

- **`element`** is one rule covering all four shapes the corpus uses:
  `Content()`, `Surface { ... }`, `Column(lazy = true) { ... }`, and a
  bare `Slot`-less receiver. Today the parser splits these between
  `parse_primary` and the postfix loop; unifying them is what removes
  the special-casing.
- **`if` / `match` / `for`** keep markup context through their bodies
  and arms, as above.
- **`const x = expr`** is allowed, for a value computed once on entering
  the block. `var` is not — reactive state belongs in the component
  body, which is a statement block and already handles it.
- **No bare assignment.** `x = 1` is not a markup node. This is nono's
  rule and it's what makes `Column { x = 1 ⏎ f() }` unambiguous rather
  than a parse error: there is no reading of it as markup, so the error
  says so.
- **No bare text.** elp has no text node — the corpus always writes
  `Text("hi")`, a call. This is a real simplification over nono, which
  needs `text_node` because it targets HTML directly.

Note that `IDENT` beginning three different node forms (`element`,
`const`, and any future named-slot fill) means ordering matters: the
forms requiring a following `=` must be tried before `element`, or
`element` shadows them. nono's grammar carries exactly this comment.

### `Slot`

`Slot` becomes a keyword. The corpus uses it bare, twice, both in
`message.velp` (`Column(class="from-me") { Slot }`) — never with
arguments, and there are **no named slot fills anywhere in the corpus**.

**Named slots are in, ported from nono** rather than deferred:

```
slot       = 'Slot' ( '(' slot_arg (',' slot_arg)* ','? ')' )?
slot_arg   = IDENT '=' expr            // named = "sidebar", or = <fallback>
named_fill = IDENT '=' block           // sidebar = { ... }, at fill site
```

- `Slot` bare, or `Slot()`, is the default slot. The corpus writes it
  bare, so bare is sugar for the empty argument list rather than a
  separate form.
- `Slot(named = "sidebar")` declares a named hole; `or = <expr>` gives a
  fallback when it goes unfilled, with `nil` meaning render nothing.
- At the call site, an element's trailing block fills the **default**
  slot; `sidebar = { ... }` inside that block routes into the named one.
- `named_fill` is **pure syntax — it never binds a value.** That's why
  it has to be ordered before `element` in the node list: both start
  with an identifier, and only the `= block` distinguishes them.

Making it a keyword is what takes `{ Slot }` out of the ambiguity
outright rather than leaving it to context — it's precisely the
`Name { x }` shape that misparses today.

Nothing in the corpus uses a named slot yet, so `message.velp`'s two
bare `Slot`s remain valid as written; the named form is grammar built
ahead of its first use, deliberately.

### Intrinsics — the props a component never declares

**An intrinsic is a prop that something other than the component
consumes.** A declared prop is read by the component it is written on,
so it belongs in that component's signature. `style` is not read by the
component at all — it is read by the platform node behind the element.
That is why it has nowhere to live in a signature, and why it is
intrinsic rather than merely conventional.

The corpus gives almost no evidence here: of 34 distinct prop names
across the `.velp` files, only `class` (14 uses) is intrinsic-shaped,
and that is the one becoming `style`. So the criterion matters more
than the list, which will grow.

The list as it stands:

- **`style`** — read by the backing node. Settled above.
- **`accessibility`** — one record, designed below.
- **Test identity** — `testID` / resource-id / `data-testid`. Same
  shape, lower stakes.

#### Two kinds, and why only one kind exists

Intrinsics do not form a single category:

- **Host intrinsics** — `style`, accessibility, test identity. Consumed
  by the platform node. A user-defined component does not have one; it
  is a function producing more tree. These therefore *cannot* apply to
  it. This is a stronger reason for "a user component gets no `style`
  unless it declares one" than the comparison with React: it is not a
  policy choice, there is nothing to attach them to.
- **Positional intrinsics** — identity. Consumed by the reconciler,
  which needs only a position in the tree, which a user component does
  have. `Card(key = ...)` would work.

**The positional category is deliberately left empty** — see the
identity spelling decision below. With identity on the loop rather than
the child, every intrinsic is a host intrinsic, so the rule reads
*intrinsics exist on built-in elements only, and a user component gets
nothing it does not declare*, with no exceptions.

#### `accessibility` — the record, and the rule that does the work

`accessibilityLabel` on iOS, `contentDescription` on Android, `aria-*`
on web: three names for one concept. A language whose premise is one
source for three targets cannot make people reach for a per-platform
escape hatch here, so this is mandatory rather than optional.

**Elp can make accessibility a compile error, and almost nothing else
can.** The web cannot know at build time whether an `<img>` has a
meaningful `alt`. Elp can: the element set is closed, built-in and
statically known, and the language has already decided that a thing
which cannot work is an error rather than a silent skip. This is the
reason to design it now rather than bolt it on — the checking is only
available while the element set is still something the compiler owns.

It is **one intrinsic taking a record**, mirroring `style`:

```
Image(source = post.imageUrl, accessibility = A11y(label = "Photo in post"))
```

A known signature, consumed at the use site rather than redefined on
every component, so it composes and can be named once and reused — the
same argument that made `style` a record of named styles rather than a
spray of properties. It also holds the intrinsic namespace at three.

| field | iOS | Android (Compose) | Web |
|---|---|---|---|
| `label` | `accessibilityLabel` | `contentDescription` | `aria-label` |
| `role` | traits (`.isButton`, `.isHeader`) | `Role` + `heading()` | `role` |
| `value` | `accessibilityValue` | `stateDescription` | `aria-valuetext` / `aria-checked` |
| `hint` | `accessibilityHint` | folds into the description | `aria-describedby` |
| `hidden` | `accessibilityHidden` | `invisibleToUser()` | `aria-hidden` |
| `group` | `.accessibilityElement(children:)` | `mergeDescendants` | label the container, hide the children |
| `live` | imperative `post(.announcement)` | `liveRegion` | `aria-live` |

`role` is a **single enum**, not iOS's set of traits: the traits that
are not roles (`.isSelected`) are state, and state is handled below.
`hint`, `group` and `live` each have one target that does not match
cleanly; all three degrade rather than break.

##### Two things are derived, never written

This matters more than the field list.

**Label is derived from text content.** `Text("Save")` is labelled
"Save"; `Button { Text("Save") }` derives it from what it contains. A
label is written only where there is no text — icons, images,
icon-buttons. This is the web's accessible-name computation, and it is
why `aria-label` is rare in good HTML. The explicit case is the
minority case, so the record being slightly wordy costs little.

**State is derived from the component's own props.** `disabled` is
already a corpus prop; `Button(disabled = true)` emits the disabled
state itself, and there is deliberately no `A11y(disabled = ...)` to
write. Two sources of truth is the biggest practical failure of ARIA in
the wild — `aria-checked` drifting out of sync with the real checked
state — and deriving makes the drift unrepresentable rather than
lintable.

##### Not in the record

- **Focus** is a state object passed in, exactly as under `ref` above.
  Not an intrinsic.
- **Traversal order** — `accessibilitySortPriority` / `traversalIndex`
  / `tabindex`. All three platforms advise against it. Left out until
  something needs it.

##### The rule

**A built-in that renders something perceivable, has no derivable text,
and is not marked `hidden = true` fails to compile without a label.**
Decorative content opts out explicitly, which is the right thing to
have to say out loud.

The corpus has two instances, both unlabelled images:
`SocialFeed/components/feeditem.velp:18` and
`SocialFeed/components/composer.velp:49`.

#### Three intrinsics Elp does not need

`children`, `ref` and `is` are intrinsics elsewhere. None of them
survive the trip, and each fails for a different reason — all three are
artefacts of a host language Elp does not share.

- **`children`** is a prop in React because JSX nesting desugars into
  one. In Elp the block *is* the children, `element = IDENT arguments?
  block?` is grammar rather than a prop name, and `Slot` is how a
  component consumes what it was given. So there is no name to collide
  with a declared prop and no way to pass children positionally — which
  also removes the tricks that depend on treating `children` as a
  value.
- **`ref`** is a mutable box the renderer writes into, which fights
  move semantics directly. The need behind it is real — focus a field,
  scroll a list — but Compose and SwiftUI both turned it inside out:
  you construct the state object (`FocusRequester`, `LazyListState`,
  `@FocusState`) and pass it *in* as an ordinary prop, so nothing is
  handed back out. That is an ordinary prop of an ordinary type and
  needs no intrinsic, and it is the shape that fits a language with no
  out-params and no globals.
- **`is`** exists because in HTML and Vue the element name is a
  *string*, so selecting a component dynamically needs an escape hatch.
  In Elp an element name is an identifier resolved like any other, so a
  binding holding a component is already usable in element position.
  Nothing to add. (Whether components are first-class values is a real
  question, but it is that question and not an intrinsic.)

### Styling — named styles only, no selectors

**There are no type selectors and no scope chain.** A stylesheet is a
module of *named styles*, nothing more. Styling is applied by referring
to a style at the use site:

```elp
import { primary, secondary } from .components.button

Button(style = primary)
Button(style = secondary)
```

**`style` is a language-level intrinsic prop** — every component has it
without declaring it, so `Button(style = primary)` works when `Button`
never mentions `style`.

#### Props are not styles

A component's own props configure **behaviour**, not appearance:
`Row(columns = 2)`, `Column(lazy = true)`, `Row(animationIn = .Fade)`.
They are ordinary typed props — checked against that component's
signature, so `columns` cannot be passed to something with no notion of
columns.

**Appearance arrives from outside by exactly one route: the `style`
prop**, which takes named styles imported from a stylesheet. It is a
language-level intrinsic on **built-in elements** — `Text`, `Column`,
`Row` and the rest have it without declaring it. A user-defined
component does not, unless it declares one; see the resolved question
below.

Two things supply appearance without being that route, and neither
breaks the rule. A **variant** (`Text(variant = .Body)`) selects among
appearances the component itself defines — the prop names a choice, not
a value. A **default** (`App(textStyle = body)`) supplies appearance to
descendants that ask for it. Both are covered below.

Keeping props and styles apart is the point: a component's signature
describes what it *does*, its style describes what it *looks like*, and
neither leaks into the other.

#### The style property set — global, closed, per-component subsets

**Style properties are a single global, closed set.** Each component
accepts a subset of it. `Text` respects `fontFamily`; `Column` does not.
Both respect `backgroundColor` and `backgroundImage`. There is no
specificity and no cascade: a style either applies to a component or it
doesn't.

**A named style's type is the set of properties it sets**, computed, not
annotated:

```elp
stylesheet {
    panel { backgroundColor = .Grey, backgroundImage = nil }     // {bg*}
    body  { fontFamily = .Otf(), backgroundColor = .White }      // {fontFamily, bg}
}

Column(style = panel)   // ok — Column accepts both properties
Column(style = body)    // error — Column has no fontFamily
Text(style = body)      // ok
```

`style = S` on component `C` is legal exactly when
`properties(S) ⊆ accepted(C)`. This is the same shape as the
object-safety rule in `interfaces.md`: computed from the declaration,
never annotated, and reported at the use site naming the specific
property that doesn't apply.

**Composition unions the requirement.**
`properties(a + b) = properties(a) ∪ properties(b)`, so composing
narrows what a style can be applied to — `panel + body` is a `Text`-only
style even though `panel` alone works on a `Column`. Worth knowing
because it makes a composed style *less* portable, which is the opposite
of the intuition CSS trains.

**A mismatched property is a compile error, not a silent skip.** The
alternative was considered and rejected: it looks like it buys
shareability, and it doesn't.

Take `card { backgroundColor = .Grey, fontFamily = .Serif }` applied to
a `Column`. Erroring forces a split into `cardBox` + `cardText`, with
`cardText` applied to the `Text` inside — which renders in serif.
Skipping compiles, gives the Column its grey, drops `fontFamily`, and —
because nothing cascades — leaves the inner `Text` *without* serif
anyway. Same number of applications needed either way, minus the
diagnostic. Once inheritance is gone, a text property on a container
style is dead weight in both designs; erroring just says so.

The split it forces is a real boundary rather than arbitrary friction:
box properties versus text properties is the same line CSS draws with
inherited-vs-non-inherited, and SwiftUI with `.font()` on `Text` against
`.background()` on any `View`. And it is narrow in practice — background,
border, padding, margin, opacity and sizing are universal, so only the
type-specific handful (font family/weight/line-height, layout-specific
things like `columns`) ever narrows a style.

Two things this buys that dropping selectors would otherwise have cost:
a mistyped property name is a compile error rather than a rule that
matches nothing, and a style applied to a component that can't honour it
is caught at the use site rather than silently doing nothing.

#### Composing styles — `+`, folded at compile time

Styles compose with `+`, in both positions, with the same meaning: the
right operand wins on conflict, so `base + emphasis` and
`emphasis + base` differ.

**At the declaration site**, using the same `: A + B` shape the language
already uses for interface dependencies, conformance declarations and
generic bounds:

```elp
stylesheet {
    base     { padding = 8, radius = 4 }
    emphasis { fontWeight = .BOLD }

    primary: base + emphasis { background = .Blue, color = .White }
    danger:  base + emphasis { background = .Red,  color = .White }
}
```

A style's own declarations win over everything it is built from.

**At the use site**, the same operator:

```elp
Button(style = base + emphasis)
```

**Composition folds at compile time wherever the operands are static**,
which is nearly always. The compiler collapses `+` into a single flat
style and each target receives that. This keeps the style graph
statically analysable, which is what makes dead-style detection a
name-resolution question rather than a whole-program one.

Folding is an optimisation, not a feasibility requirement. Merging two
flat styles is a fixed-shape, field-wise overwrite — take the right
operand's value where present, else the left — because styles are typed
records over a closed property set. That is one small generated function
per target, not a per-platform algorithm anyone has to design. CSS
merging is expensive because CSS is open-ended and stringly-typed; none
of that carries over.

**Conditional operands are legal**, because `if` and `match` are
expressions that produce values:

```elp
Button(style = base + if selected { selected } else { nil })
```

`nil` is the identity for `+` on styles (`base + nil == base`), so the
empty arm needs no special case.

**Distributing `+` over the branches is deliberately *not* a language
rule.** An earlier draft specified that rewrite — `base + if c { s } else
{ nil }` becoming `if c { base + s } else { base }` — together with a
bound on it, since distribution is exponential in the number of
conditional operands. Both are dropped.

Rust is the precedent, and it has the identical setup: value-producing
branches, `+` overloaded through `std::ops::Add`. It performs no such
rewrite. It evaluates the branch and calls the merge, leaving any
branch-local specialisation to the optimiser — and accepts exponential
code growth elsewhere (monomorphisation) rather than adding bounding
passes. Go doesn't have the problem at all, lacking both operator
overloading and expression-valued `if`; composition there is an explicit
runtime fold, either `merge(a, b)` or the functional-options pattern.
Rust's declaration-site analogue is struct update syntax
(`Style { color: red, ..base }`), which is what `primary: base +
emphasis { ... }` corresponds to.

What remains is ordinary constant folding: a `+` whose operands are both
statically known collapses at compile time, exactly as any other
constant expression would. Everything else is a merge call, cheap for
the reason above. Specifying a distribution pass would mean defining,
documenting and defending a bound in the language spec, for a case that
is rare and inexpensive to fall back from.

One consequence worth stating rather than discovering later: a genuine
runtime merge survives only when a style operand is itself computed at
runtime — from data, not from a branch over static styles. If that turns
out never to happen in practice, `+` can be tightened to a
compile-time-only operation later without breaking anything written
against this rule.

Since `+` yields one style value, the prop is `style` singular. `styles`
plural would only be right if it always took a list.

#### What was rejected, and why

An earlier draft had type-default selectors: `Text { padding = 16 }` in
a stylesheet, applying to every `Text` in some scope, resolved through a
four-tier chain (`style` prop → component sheet → module sheet → global
sheet). That is gone.

The objection is that it is *action at a distance*. A rule written in
one place silently changes the appearance of markup written somewhere
else, and nothing at the use site says so. Reading `Text("hi")` tells
you nothing about how it will look; you have to know which sheets are in
scope. That's the property that makes CSS hard to reason about, and it
survives even after the selector engine, specificity arithmetic and
descendant matching have been stripped out — those make it *worse*, but
the ambient rule is the actual problem.

Removing it closes six open questions at once, none of which have to be
answered any more:

- whether inheritance exists, and over which closed set of properties
  — this one came back, and is answered under *Defaults* below. It is
  the only one that did.
- how a `.selp` names its type defaults so `globalStyles` can be passed one
- whether sheets are module-scoped or component-scoped
- whether scoping is lexical or contextual
- whether capitalisation distinguishes type rules from named styles
- selector lists (`Text, Date { padding = 16 }`)

That is the sign it was the wrong shape: a design whose sub-questions
all evaporate together was one question wearing six hats.

#### How styling is reused without ambient rules

Three mechanisms, all explicit:

- **Component composition.** Want every body paragraph to look the same?
  Declare a `component` for it and use that, rather than a rule that
  restyles `Text` from a distance. Components are already the unit of
  reuse in the language; styling reuse being component reuse is one
  fewer concept, and it is how SwiftUI and Compose work — which matters
  given `.iOS` and `.Android` are two of three targets.
- **Named styles as values.** They are ordinary exported symbols, so
  they compose, can be built from shared tokens, and can be passed
  around like anything else.
- **Variants — a component's own appearances, selected by name.**
  `Text(variant = .Body)`, `Text(variant = .Heading1)`. This is the
  normal shape for a component library and it is the preferred one here.
  A variant is *not* a style arriving from outside: every appearance it
  can select is defined by the component, and the caller picks between
  them. Appearance still has exactly one owner, so this does not weaken
  "props are not styles" — the prop names a choice, not a value.

The cost is verbosity: an element with no `style` and no `variant` is
unstyled, and beyond the single default below there is nothing ambient
to lean on. That is the trade — explicit at every use site, in exchange
for markup that means what it says.

#### Defaults — the one ambient thing, and why it isn't inheritance

Dropping selectors left a real hole, and the corpus names it precisely.
Both `.selp` files in `examples/`, written independently, contain
exactly one text rule each:

```
Text { fontFamily = .GoogleFont("Patua One") }    # BlogStyle
Text { fontFamily = .GoogleFont("Inter") }        # SocialFeed
```

That is the whole of the text styling in the corpus, and it is the same
wish twice: one font for the app. There are 18 `Text(` nodes across 10
files in three small projects. With named styles alone the answer is 18
applications and 10 imports, and for a `Text` inside a component that
declares no `style` prop there is no answer at all.

**Inheritance is still the wrong fix**, for a reason that is local to
this document rather than a matter of taste. The mismatch error is
justified above by *"once inheritance is gone, a text property on a
container style is dead weight in both designs."* Make `fontFamily` on a
`Column` inherit and it stops being dead weight, so the error has to go
— for exactly the properties where it carries the most information. The
cascade and the diagnostic are mutually exclusive. Picking the cascade
would also reopen the composition and property-set rules that hang off
it, where a default reopens nothing: it is purely additive, and can be
widened later without invalidating anything decided here.

So: **defaults are a third category, beside props and styles.**

| | sets | scope |
|---|---|---|
| prop | behaviour | the node it's written on |
| `style` | appearance | the node it's written on |
| default | appearance | descendants that ask for it |

A default is supplied by the built-in that declares one, and read only
by built-ins that consume one. It is the only ambient thing in the
language. The corpus case needs no new mechanism, because `App` is
already a built-in taking props — it previously took
`globalStyles = Styles()`:

```
App(textStyle = body) { ... }
```

Resolution order at a `Text`, last wins: **enclosing default → variant →
`style` prop.** The default supplies the family, the variant supplies
the role's size and weight, an explicit `style` overrides either.

This passes the test selectors failed. What affected your `Text` sits
*on the path from the root to it, in markup you wrote*, so "why is this
serif" is a bounded walk up a visible tree rather than a search of every
sheet in scope. It is opt-in per subtree. And because `Text` is a
built-in, the implicit read lives in the language rather than in
anyone's component.

It is still non-local, and that is the honest cost — strictly worse than
nothing, strictly better than selectors, confined to one closed property
set with one place to look.

Subtree-scoped defaults (Flutter's `DefaultTextStyle`, SwiftUI's
environment) are the obvious generalisation and are deliberately *not*
added yet. `App` covers every observed case; the wider form can arrive
the day the corpus needs it.

#### Semantic variants, and why they matter more here than in a web framework

`Text(variant = .Body)` needs no new syntax — leading-dot enum shorthand
already exists (`.All`, `.SEMIBOLD`, `.Fade`), and the variant type is
inferred from the parameter.

For a user component library this is ordinary good practice. For the
*built-in* `Text` it is close to load-bearing, because Elp targets
`.iOS`, `.Android` and `.Web`, and all three scale text by a user
accessibility setting through a semantic scale: Dynamic Type, Material's
type scale, and the browser's root font size. A literal
`fontSize = 16` cannot participate in any of them. `.Body` can, because
it names a role each target already has a mapping for.

That makes the semantic scale the portable unit and raw sizes the
escape hatch, which is the right way round for a language whose whole
premise is one source for three platforms.

### The `stylesheet` grammar

```
stylesheet   = 'stylesheet' '{' named_style* '}'
named_style  = IDENT (':' style_bound)? '{' style_decl* '}'
style_bound  = IDENT ('+' IDENT)*
style_decl   = IDENT '=' expr
```

- **Every entry is a named style.** No selectors, so no `selector` rule
  and no comma-separated selector lists.
- **Style values are ordinary elp expressions**, not raw text. The
  corpus uses contextual variants (`.All`, `.SEMIBOLD`), calls
  (`.GoogleFont("Patua One")`), member access (`FontWeight.BOLD`) and
  literals (`16`). A deliberate divergence from nono, whose `css_value`
  captures raw to end-of-line and trusts CSS.

A consequence worth noting: `style_decl` is `IDENT '=' expr`, the *same
shape as a field init*. A named style's body and an object literal's
body are grammatically identical; what distinguishes them is the
enclosing `stylesheet`. Same conclusion the markup rules reach, arrived
at from the other direction.

Since capitalisation no longer distinguishes anything here, **the parser
leans on case nowhere at all.** Object-literal-vs-element is settled by
context; everything else is convention the corpus happens to follow.

### `.selp` files: imports and named styles only

No `const`, no `fn`, no declarations beyond styles. Shared values live in
a `.elp` and are imported — which is what the corpus already does:
`FONT_COZETTE` is declared in `BlogStyle/constants.elp` and imported
where needed. Keeping `.selp` to imports plus named styles preserves the
same separation the `component`-is-`.velp`-only rule exists to enforce.

Named styles are exported and imported by destructuring like any other
symbol. There is no default import and no whole-sheet value, so a
`.selp` needs no name of its own — which is what dissolved the
`import { Styles } from .styles` gap rather than solving it.

### Nested `fn`s do not get markup context

A `fn` inside a component body is an ordinary function; its result
expression is value context, whatever it returns. Something that
produces markup is a `component` and should be declared as one. This
keeps the rule "markup context is entered by `component`, once" rather
than "by `component`, and also by any nested function that looks like it
wants it".

### Params and props are different things, already decided

Components have both, and they don't mean the same thing (see
`memory-model.md`, "Arguments are constant; props are reactive"):

- **Params** — `component Message(from User, body string)` — constant
  inputs, plain owned values, no cell, no observation.
- **Props** — `export var email Optional<string>` in the body — reactive,
  observed, re-render on change. Plus `export *fn` callback props.

`component` doesn't change either; it just means the params sit on the
`component` keyword rather than on an `fn`.

### Still to decide

- **Destructured params don't parse.** `BlogPost({ params })` and
  `ChatApp({state, dispatch})` both destructure, and `parse_param`
  accepts only `IDENT type` or `type` — a leading `{` fails. This is an
  unmodelled form that predates this design and isn't listed anywhere
  else; it needs a param-pattern grammar of its own. The open half is
  typing: both corpus uses are untyped and get their type from a macro
  (`@Screen<BlogPostParams>`, `@Reducer<...>`), so either the type is
  written at the param or macros become load-bearing for basic
  signature checking.
- **Is `style` a reserved prop name on a user component?** Nothing
  currently stops `component Card(style String)`. Reserving the *type*
  without requiring the *prop* would keep `style` meaning one thing
  everywhere, which matters for tooling; leaving it ordinary is one
  fewer special case. Small, and the only piece of the styling model
  still open.


Resolved since the styling rework, kept here so the reasoning isn't
lost:

- ~~**Class selectors.**~~ Replaced by named styles through the
  intrinsic `style` prop. `class = "error-row"` becomes
  `style = errorRow`.
- ~~**`nil` vs `None`.**~~ **Resolved: `nil` only, and the unit type
  loses its spelling.** The corpus used three concepts across two
  names — `nil` as a literal, `None` as the empty `Optional` variant,
  and `None` again as the unit return type.

  One framing considered was that `nil` is a value and `None` a type,
  which is self-consistent for the unit value and unit type but leaves
  out the empty-`Optional` meaning — which was the most used of the
  three, at six value-position sites against two in type position.

  So: **`nil` is the only absent value**, covering both the empty
  `Optional` and "nothing produced". **The unit type has no spelling**
  and is written by omitting the `->` clause. `None` is gone from the
  language. That leaves one value, one optional type (`Optional<T>`,
  with `T?` as sugar) and no unit-type name available to collide with
  anything.

  This is Swift's model — `nil` plus `Optional`, with `Void`
  effectively never written — which keeps the vocabulary consistent
  with `Binding` and `Sendable`, both borrowed from there too. The
  deciding detail was that the only two places a unit type was ever
  written were the two `*fn` callbacks, and both read better without
  it.

  **Corpus migrated.** Six value sites in `composer.velp`,
  `feeditem.velp`, `main.velp` and `message.velp`; two `-> None`
  clauses dropped from the callbacks in `composer.velp` and
  `login-form.velp`; two prose comments. No `None` remains.
- ~~**Does inheritance exist?**~~ **No, decided** — but not because
  nothing is ambient, which was the earlier reasoning and was too
  strong. One ambient default exists (`App(textStyle = ...)`). A
  cascade is refused because it would force the removal of the
  mismatch error, and because a default is additive where inheritance
  is not. See *Defaults* above.
- ~~**Do user-defined components accept `style`?**~~ **Only if they
  declare it, decided.** `style` is intrinsic on built-in elements
  only. On a user component it is an ordinary prop with no special
  behaviour, so `Card(style = x)` where `Card` declares none is an
  unknown-prop error like any other — no new rule, just the prop
  checking already needed. This is React's model made static: React
  does nothing with `style` on a composite component and drops it
  silently, and TypeScript is what turns that into an error. Vue's
  alternative — `class` and `style` fall through and *merge* onto the
  single root element — is the magic option, and it is notable that
  declaring `style` as a prop in Vue also takes it over, which is the
  behaviour chosen here unconditionally.

  Forwarding is therefore written out: `Column(style = base + style)`
  says in one visible `+` what Vue infers. The knock-on is that a
  forwarded `style` param's accepted property set is derived from where
  it is used — `accepted(Column)` for the above, the intersection if it
  is forwarded to several elements. That is ordinary inference rather
  than a styling-specific pass.

  A component that declares no `style` cannot be styled from outside at
  all. With no inheritance and no selectors that is the honest outcome,
  and it is the same line the language already draws for fields: the
  public API is the way in. The idiomatic answer to wanting one is to
  give the component variants — an unstyled component is one that
  hasn't been abstracted far enough yet.
- ~~**How does a `.selp` expose its type defaults?**~~ Moot. There are
  no type defaults, so a `.selp` is just a module of named styles and
  needs no name of its own.
- ~~**Do markup `for` loops need identity?**~~ **Yes, decided.** Both
  corpus loops build dynamic lists with nothing to reconcile between
  renders, and `index.velp` animates its rows, which is where a missing
  identity stops being theoretical. Identity is **mandatory** on a markup
  `for` — unlike React's `key`, which is optional and therefore
  forgotten.
- ~~**How is identity spelled?**~~ **On the loop, decided:**
  `for post in posts by post.id { ... }`, not `Row(key = post.id)` on
  the child. Two independent arguments, and they agree:

  1. The loop form is enforceable by the grammar — identity cannot be
     mandatory in any useful sense if the only check is semantic and
     fires on a child the loop never looked at.
  2. `key` as a prop would be the sole *positional* intrinsic, the one
     thing that works on a user-defined component when no other
     intrinsic does. Putting it on the loop empties that category, so
     "intrinsics are built-in-only" holds without a carve-out.

  It also puts identity next to the collection it identifies, which is
  where the reader looks when asking what distinguishes two rows.

### Migration this implies

**Components.** Nine components across eight `.velp` files change
`fn Name -> T` to `component Name`. `Element` in `home.velp` was a
misnomer for `Component` and disappears along with the return type.
`App` stays as it is — a built-in object describing the application
scaffold, not a return-type variant.

**Styles.** Every stylesheet in the corpus is written as type selectors
and none of it survives as-is:

- `styles.selp` — bare `App { ... }` and `Text { ... }` rules at file
  scope, plus `main.velp`'s `App(globalStyles = Styles())` wiring. The
  sheet and the wiring both go. `App { ignoreSafeAreas = .All }` was
  never styling — it becomes the prop `App(ignoreSafeAreas = .All)` —
  and the `Text { fontFamily = ... }` rule, the one piece with no
  replacement when selectors were dropped, becomes
  `App(textStyle = <named style>)`.
- `home.velp` — `Stylesheet { Text, Date { padding = 16 } ... }`, a
  selector list, which has no replacement form. Becomes named styles
  referenced from the markup that wants them.
- `paragraph.velp` — `Stylesheet { Text { .fontFamily = .Otf() ... } }`.
  Note this file also writes properties with a leading dot, which was
  wrong independently (gap #8: leading dots are contextual-type
  inference, not field access) and is the corpus's only instance.
- `class = "error-row"` / `class = "title"` — four sites across
  `about.velp`, `index.velp`, `[postId].velp`, `home.velp` — become
  `style = <importedName>`.

**Loops.** Two markup `for` loops (`index.velp`, `ChatRoom/main.velp`)
gain a mandatory `by` clause — `for post in posts by post.id { ... }`.

**Accessibility.** Two unlabelled `Image(...)` calls
(`feeditem.velp:18`, `composer.velp:49`) need either a label or an
explicit `hidden = true`, once the rule above is enforced.

## Open questions

The items below are real design calls that aren't made yet. They appear
in one or more example files but have no specced behaviour:

- ~~**`*fn` prefix** in component-body prop declarations.~~ **Modelled.**
  `EXPORT '*' FN IDENT '(' paramList? ')' ('->' type)?` parses as a
  `CALLBACK_DECL` statement — a callback slot, signature only, no body.
  Semantics per `memory-model.md` Finding 4: marked required, still
  nilable at runtime.

  **`export` is mandatory on this one form**, the only place in the
  language where it isn't purely optional visibility. A callback slot can
  only be filled from outside the component, so a private one has nothing
  that could ever populate it — it would be provably nil for its whole
  life and optimisable away entirely. Every callback is a prop, so
  `CallbackDecl` has no `is_exported()` accessor; it would be a constant
  `true`.

  What's *not* settled is where it's legal. The parser accepts it in any
  block because it can't tell a component from a plain `fn` returning
  `Component`, so "props only exist in components" has to be a later
  semantic check. "Callbacks must be exported" is expressible in the
  grammar, so that one is enforced at parse time.
- ~~**Nested namespace imports**~~ **Resolved** by `name-resolution.md`:
  consumers write several `import` lines. The shape in question was
  `import { Component, components: { Text }, font: { FontWeight } } from elp.ui`
  (BlogStyle `paragraph.velp`), which is sugar for two sibling module
  paths. It is rejected along with the related
  `import { FontWeight } from elp.app.ui.elements.Text`, under one rule:
  **a module path segment is always a file**, never a sub-namespace and
  never a type. Both forms make an import ambiguous between "a deeper
  module" and "a member of something already resolved", and buy only
  keystrokes.
- **Conditional loops** (`while`, `loop`). See gap #2.
- **Doc comments** (`///`). The lexer recognises them as a hidden
  channel; no extraction pipeline yet. Not blocking the parser.
- **Are `from` and `as` fully reserved, or contextual?** Fully reserved
  means neither can ever be an identifier — no `fn convert(from Foo)`.
  Contextual means they're keywords only in import position and ordinary
  identifiers everywhere else, which costs the parser a lookahead or a
  keyword-union `ident` rule. Not decided.
- ~~**`if` condition vs builder block.**~~ **Resolved** by "Markup
  context" above. An `if` condition is an expression, so it is always
  value context, where `Name { ... }` is an object literal — and object
  literals stay suppressed in condition position (the existing
  `allow_struct_literal = false` rule, which the parser already
  implements). That rule was borrowed from Rust on instinct; it now has
  a reason.
- ~~**Object literal vs builder block.**~~ **Resolved** by "Markup
  context" above: the two live in different contexts, established by the
  `component` keyword and propagated by position, so there is nothing
  left to disambiguate by lookahead.
  `looks_like_object_literal_body` in
  `crates/parser/src/ast/expression.rs` is the heuristic this replaces
  and should be deleted with the change.
- **Float vs range lexing.** `400..499` must lex as `INTEGER RANGE
  INTEGER`, which works only because `FLOAT` requires digits on both
  sides of the dot. The consequence is that `.5` is not a valid float
  literal — leading-dot floats and ranges can't both be supported
  without lexer lookahead. Currently resolved in favour of ranges by
  accident rather than decision.
