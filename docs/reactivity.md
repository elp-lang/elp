# elp reactivity — the body runs once, reads subscribe

## The decision

**Model B: run-once, fine-grained.** A component body executes exactly
once, at mount. Declarations in it create cells. Reads in markup
establish subscriptions at the point of the read. A write notifies
exactly the readers of that value and patches the affected nodes
directly.

The alternative — **Model A: coarse re-render and reconcile**, as React
and SwiftUI do it — was rejected.

## Why, and it isn't preference

The corpus is already written in Model B, whether or not anyone chose
it. Component state is declared as a plain local and mutated by plain
assignment:

```
var posting = false
var attempts = 0
...
posting = true
attempts += 1
```

and captured by closures — `composer.velp`'s `handleSubmit` mutates
`posting`, `body` and `err`, and is called later as a callback.

**That is incompatible with re-running the body.** If the body re-runs
on change, `var posting = false` re-initialises to `false` every time.
This is exactly why React needs `useState` and Compose needs a
compiler-inserted slot table keyed by call position. elp's syntax has
no "this one persists" marker because it was written assuming the body
runs once.

Three things agree with that reading:

- It is the better fit for a Rust target — no tree allocation per
  update, no diff pass.
- `memory-model.md` Finding 2 already lowers a reactive `var` to a cell
  with interior mutability, which is the Model B representation.
- Model A would require retrofitting hook-shaped, position-keyed state
  onto every `var` in a component.

### The Svelte 4 lineage, and where it stops

The ergonomics being targeted are Svelte 4's: variables are reactive,
the compiler does the work, there is no ceremony at the use site. Runes
were a step in the wrong direction for this language's purposes.

But the history is instructive. Runes exist largely *because* Svelte 4's
approach only worked at the top level of a component — the compiler
could see the whole component, so `let` could be reactive, and that
broke the moment state needed to live outside one. Svelte 4's answer to
crossing component boundaries is the module-level store, which elp bans
outright (`memory-model.md`, no package-level variables).

So the rule for elp is: **keep Svelte 4 ergonomics inside a component,
and find a different answer for crossing them.** The mistake to avoid is
making the inside case uglier in order to serve the crossing case, which
is what runes did.

## What follows from Model B

### Markup control flow is a set of reactive constructs

`if`, `match` and `for` in markup position are not merely parsed
differently from their value-context counterparts — they behave
differently at runtime. `if state.loggedIn != nil { ... } else { ... }`
in `ChatApp` has to swap subtrees when the condition changes; that is a
reactive node, Solid's compiled `<Show>`.

This retroactively strengthens the markup-context decision in
`grammar-script-notes.md`. The two contexts were separated because
`Name { ... }` is ambiguous between an object literal and a builder
block. They turn out to differ at runtime as well, which is a second,
independent reason for the split.

### Mandatory loop identity survives

A worry raised when `by` was chosen — that fine-grained reactivity might
make identity unnecessary — does not hold. Fine-grained systems still
key lists; Solid's `<For>` is keyed. Identity is needed under either
model, so `for post in posts by post.id` stands.

## Passing a reactive value

A reactive `var` lowers to a cell, so passing it has three possible
meanings. The corpus needs two of them and has a spelling for one.

| mode | lives | direction | spelling |
|---|---|---|---|
| snapshot | dead | — | — |
| read-only share | live | one-way | `Input(value = name)` |
| writable share | live | two-way | `TextArea(&body)` |

The **snapshot** is what a move would actually give you, and it is
almost never wanted: moving a reactive value severs the subscription
that makes the prop reactive in the first place.

This amends `memory-model.md`'s "props cross the boundary by move".
That is right for plain values and wrong for reactive ones. **A
reactive prop crosses as a handle.**

### Props are shares, not borrows

The borrow-checker mental model is the wrong one here, and the reason
matters for the lowering.

A borrow does not transfer ownership and does not extend the referent's
lifetime — the constraint runs the other way, and the reference must
die before the owner does. But a child component is not a scoped callee
that returns. It is a retained object in a tree that outlives the
expression which constructed it, so "child holds `&mut parent_local`"
is not expressible.

A reactive prop is therefore **shared ownership** — a refcounted handle,
`Rc`-shaped — not a Rust borrow.

This puts a question mark over Finding 4's phrasing, which has
`EmailInput(&email, ...)` elaborating to `&self.email`, "a plain shared
reference to the `Reactive<T>` cell". That holds only while the parent
strictly outlives the child and the tree is a single expression. Once
the tree is retained and a reconciler moves nodes between collections,
every Rust UI toolkit lands on a refcounted handle instead. Finding 4 is
annotated accordingly.

### Notation: inferred for ordinary borrows, explicit in prop position

Finding 4 established one ref spelling with mutability inferred from
callee usage, and a hard error when a `const` reaches a mutating
position. The safety property is not in question — a `const` can never
be silently promoted. What inference costs is **local readability**.

That trade is acceptable for an ordinary function call: it is scoped, it
returns, and the callee's signature is one hover away. It is not
acceptable in prop position, for three reasons that stack:

- the handle is **retained** rather than scoped,
- data flow is the specific thing a reader scans markup to understand,
- the child's signature is not visible while scanning a tree.

So the rule is **explicit wherever the handle outlives the call**. In
prop position `&` is not a borrow; it is a capability grant, and the
absence of `&` is a read-only share rather than a move.

Note this is a context-sensitivity in what `&` means, and is recorded as
one rather than presented as a uniform rule.

### `&` is idempotent, and always written

`&` on a cell constructs a binding. `&` on a binding forwards the one
it already has. It is written at **every** call site that can be
written through, including when a component is merely passing a binding
it received straight down.

Forwarding without the sigil was considered first and rejected: it is
technically more honest — nothing is being constructed — but it means
the one thing the sigil exists to show disappears exactly where the
data flow gets longer. The component's author knows why
`TextArea(value)` can write back; a reader arriving cold has to find
the declaration to learn that `value` is a `Binding` and not a
`string`.

This is SwiftUI's rule and for the same reason: inside a view holding
`@Binding var text`, you still write `TextField("", text: $text)`.
`$` constructs from `@State` and projects from `@Binding`, so the sigil
is present on every writable call site with no exceptions to learn.

So `&` reads as **a write may happen here**, uniformly, with no
knowledge of where the value came from. The cost is that `&` on a
binding is a no-op; a consistent no-op that preserves a reading
invariant is cheaper to explain than a sigil that is sometimes absent.

### Bind the field, not the object

Prefer `TextArea(value = &state.description)` over handing over
`&state`. Three arguments, which stack:

- **Least privilege.** `&state` grants write access to everything
  reachable through it; a text field needs one field.
- **Subscription granularity.** Under Model B this is not only
  hygiene. A binding to a field path subscribes to that field; a
  binding to the object subscribes to the object. Passing the whole
  thing reintroduces coarse invalidation — the same defect as open
  question 1, where replacing a whole `ChatAppState` notifies every
  reader of `state` on every action.
- **`&obj` is not a blanket grant.** Writing through an object binding
  still respects each field's own declaration (see
  `memory-model.md`, field-level `var`). Otherwise `&` would be a way
  to launder past visibility.

### The parameter type — `Binding<T>`

The call-site sigil is `&x`; the parameter type that demands one is
**`Binding<Something>`**. The component's signature declares the
expectation, so the call site produces the error and the declaration
explains it. The alternatives and why they lost:

- `&mut Something` — Rust's answer, rejected for introducing a
  lowercase intrinsic keyword this language is avoiding.
- `Reactive<Something>` — wrong axis. A read-only prop is reactive too,
  so this names the cell rather than the capability, and it collides
  with the internal lowering name in `memory-model.md`.
- `Mutable<Something>` — names the capability, but reads like a local
  mutable, which `var` already covers.
- `Binding<Something>` — **chosen.** Names the role: a retained,
  shared, read-write connection to state owned elsewhere. It is
  SwiftUI's term for exactly this construct, where `@Binding` plus
  call-site `$x` is the same design as `Binding<T>` plus call-site `&x`.
  It fits the corpus's existing convention of capitalised generic
  wrappers over lowercase primitives (`Optional<string>`,
  `Protected<string>`).

Both directions are caught at the call site:
passing `&x` to a plain `string` parameter is "this parameter is
read-only"; passing `x` to a `Binding<string>` is "expected a binding,
write `&x`". Passing `&someConst` to a `Binding<T>` is an error for the
Finding 4 reason — a `const` has no cell to bind.

The sigil is kept even though the type already states the requirement,
for the same reason SwiftUI keeps `$`: the point is visibility at the
use site.

## Effects — a coroutine scoped to the component

### `effect` is the spawn primitive

There is one construct, not two. A coroutine's scope is lexical, so a
spawn inside a component body is scoped to that component's lifetime,
which is exactly what an effect is. `effect` is that construct in both
roles — Kotlin's `viewModelScope` and Swift's `.task`, rather than a
separate effect concept layered over a separate concurrency one. This
closes the backlog's "concurrency spawn syntax" item at the same time.

```
effect(roomId) {
    const room = joinRoom(roomId)
    defer room.leave()

    for message in room.messages() {
        messages.append(message)
    }
}
```

### Cleanup is cancellation

The loop blocks. The coroutine is owned by the component. When the
component goes away the coroutine is cancelled, and unsubscribing *is*
the coroutine ending.

There is no cleanup function to return and no teardown protocol, which
removes the most error-prone part of React's model — and it is removed
by structured concurrency rather than by anything effect-specific.

Where cleanup genuinely is not cancellation — closing a file, telling a
server you are leaving, unregistering from a callback-based platform
API — the answer is **`defer`** in the effect body, with Go's
semantics. That is a general language feature rather than an effect
feature, which is the point: there is no bespoke protocol to get wrong.

`defer` forces one sub-decision: **cancellation is cooperative and
unwinds.** It takes effect at the next yield point and unwinds the
stack, running `defer`s on the way out. Preemptive cancellation would
skip them, so `defer` would mean nothing.

Its own rules:

- **Scope-scoped, not function-scoped.** It runs at the end of the
  enclosing block. Go's is function-scoped, which is the familiar
  footgun — a `defer` inside a loop does not run until the function
  returns. Swift's is scope-scoped, and that is what a reader expects.
- **Both forms**, LIFO within a scope: `defer room.leave()` and
  `defer { sending = false }`.
- **Runs on normal exit, on cancellation unwind, and on panic unwind.**
  The last is what makes it a resource-release tool rather than a
  convenience, and it is the point where it meets `RecoveryBoundary`.
- **It cannot modify the result.** Go permits this through named
  returns and it is reliably confusing.

### Keyed restart, not auto-tracking

`effect(roomId) { ... }` restarts when `roomId` changes. Dependencies
are **explicit keys**, not inferred from reads.

This is the one place the language does not auto-track, and the reason
is technical rather than stylistic. If the effect body blocks, the
tracking scope is ambiguous: reads before the first suspension would be
tracked and reads ten minutes later, after it, would not. Solid has
exactly this footgun. And "restart" for a long-running coroutine means
*cancel and relaunch*, which is destructive enough to be worth stating
deliberately.

The defensible line, and the one to keep: **tracking where execution is
instantaneous, keys where it is long-lived.** Markup reads and derived
values track; effects take keys. This is Compose's `LaunchedEffect(key)`
and SwiftUI's `.task(id:)`, so it is also the Android and iOS idiom.

**On restart the replacement waits for the old coroutine to finish
unwinding.** Overlapping a cancelled instance with its successor makes
`defer` ordering nondeterministic, and determinism is worth more than
the microseconds — in the example above it is the difference between
never being joined to two rooms and occasionally being joined to both.

### It returns a handle, and the handle carries control

```
effect { ... }                  // fire; component-scoped
const job Task = effect { ... } // keep it to cancel manually
```

Ignoring it is the common case. The handle type is **`Task`** — Swift's
name, chosen because the vocabulary is already Swift's through
`Binding` and `Sendable`, where Kotlin's `Job` would be borrowing from
a model the rest of this document does not follow.

**The handle carries control, not results.** There is no `await`,
because Go-style concurrency means no function colouring, so an effect
that produces a value writes it to a cell:

```
effect { post = getBlogPost(params.postId) }
```

which is also the fix for `[postId].velp:13`, where a blocking
`getBlogPost` sits at the top of the body and stops the component
producing a tree until the network returns.

### Capture

Three kinds, all of which fall out of rules that already exist rather
than needing effect-specific ones:

| captured | how |
|---|---|
| a cell (reactive `var`) | refcounted handle, shared — as with props |
| a `Copy` value | copied in |
| anything else | moved in |

Moving out of the component body is sound *because* the body runs once
and has already finished: the effect is the only remaining user. Two
effects moving the same non-`Copy` local is the one error, and it is
the ordinary one-owner rule rather than a new concept.

Cancellation is not instantaneous — a coroutine blocked in a syscall
runs until its next yield point — so a cell can briefly outlive the
component that declared it. That is sound with refcounted handles and
it is *why* they are refcounted. It would be unsound with a borrow.

**Stale closures are structurally impossible.** React's capture bugs
exist because the body re-runs and each run closes over a different
snapshot. Here the body runs once and a cell is the same cell forever,
so there is nothing to go stale. That is a bug class deleted by the
model rather than by discipline.

## Derived values are functions — there is no third construct

A derivation is an ordinary `fn`, and the reactivity comes from *where
it is called* rather than from how it is declared:

```
fn total() -> Money {
	items.sum(fn { it.price }) - discount
}

return Column {
	Text("Subtotal: " + subtotal())
	Text("Due: " + total())
}
```

Each call site reads the cells and subscribes, so a change to
`discount` updates both `Text` nodes. There is no `$derived`, no
`createMemo` and no new keyword — which matters, because the ceremony
those introduce is the specific thing Svelte 5 added and this language
is deliberately not copying.

It need not live in the component either. A file-scope
`fn total(items, discount)` called from markup subscribes at argument
evaluation, which happens in the markup, where tracking is live.

**A read subscribes only in a tracked context.** Calling a derivation
from markup subscribes; calling it from an event handler or an effect
body is a plain read of the current value. That follows from effects
taking keys rather than tracking, and it is the same rule that already
governs an inline expression.

The cost is recomputation per call site rather than one memoized value.
That is a performance property, not a semantic one, and there are two
existing answers: hoist to a cell written by an effect, or memoize a
`pure fn`, which is sound precisely *because* it is pure. Purity
earning memoization is a second job for a keyword that currently only
gates macro eligibility.

## The thread model — cells live on one thread

**Resolves what was open question 2.**

Cells are confined to a single logical thread. A handler or effect runs
its non-blocking segments there, and only blocking work goes elsewhere,
resuming back on it. **Any run of statements containing no yield point
is therefore atomic by construction.**

This is not a choice elp is making. iOS requires UI work on the main
thread, Android requires it on the main thread, and the web has one JS
thread. All three targets force it already, so the cell thread *is* the
platform's UI thread on every target and there is no hop between a cell
write and the patch it causes. It is the JavaScript event loop and
Swift's `@MainActor`.

Three things follow:

1. **Finding 2's interior mutability stays valid** — `RefCell`-shaped,
   not `Mutex`-shaped.
2. **User-written re-entrancy guards work.** In `handleSend` below,
   `if sending { return }` and `sending = true` are adjacent with no
   yield between them, so nothing can interleave. Under free-threaded
   cells with per-cell locking, two coroutines would both read `false`
   and both proceed — the guard would be decorative. Re-entrancy is
   therefore the author's problem, which is only a fair thing to say
   because the model makes the obvious guard correct.
3. **No tearing**, since writes and the patches they cause happen in
   the same place.

```
fn handleSend() {
    if sending { return }
    sending = true
    defer { sending = false }
    ...
}
```

The cost is real: a CPU-heavy handler blocks the UI, and that cannot be
caught statically. A dev-mode watchdog warning when a turn exceeds a
frame budget is the pragmatic answer, and it is a runtime feature
rather than a language one.

### A coroutine mutates component state directly

**An effect writes component cells itself. There is no channel, no
posted message and no scheduling hop.**

Worth stating outright because it is the first question anyone asks of
a reactive model, and the answer is only implied above. An effect's
non-blocking segments run *on* the cell thread, so there is no boundary
to cross; `SocialFeed/screens/home.velp` assigns `posts`, `loading` and
`err` from inside `effect { loadFeed() }` and that is correct.

A channel is for crossing a *thread*, which only `background` does —
and that boundary is already enforced by `Sendable`, since a reactive
cell and a `Binding<T>` never conform. `background(fn { posts = x })`
fails as an ordinary conformance error rather than needing a rule of
its own.

### A yield point ends the turn, and yield points are invisible

*"Any run of statements containing no yield point is atomic by
construction"* is the load-bearing sentence above, and its contrapositive
needs saying: **a blocking call ends the turn, and other turns run
before it resumes.**

This is sharper here than in Swift because there is no function
colouring (§"It returns a handle"), so **any call may be a yield
point** and nothing marks it. A reader cannot see where atomicity
stops.

Three consequences:

1. **Flow narrowing of a cell does not survive a call.** See the
   amendment in `operators.md`; the fix is to narrow by binding
   (`patterns.md`), which produces a local that no other turn can
   write.
2. **Check-then-act is not atomic across a call.** The re-entrancy
   guard works precisely because `if sending { return }` and
   `sending = true` are adjacent; move a call between them and it
   stops working.
3. **The component may be gone on resume.** Already handled —
   effects are component-scoped and cleanup is cancellation.

`loadFeed` in `SocialFeed/screens/home.velp` is the worked example of
getting this wrong:

```elp
fn loadFeed() {
	loading = true
	return match getFeed() {          // yields
		Ok(fetched) -> { loading = false; posts = fetched }
		...
	}
}
```

Two refreshes in flight, and the first to return sets `loading = false`
while the second is still fetching, so the spinner disappears early.
The guard above is the fix and the corpus has been migrated to use it.

### `background` and `Sendable`

The distinction that matters is not slow versus fast — **IO yields and
CPU does not.** A socket read parks the coroutine and leaves the cell
thread free; a compression loop owns whatever thread it is on until it
finishes. A coroutine that can touch cells is pinned to the cell
thread, so it cannot be migrated mid-flight. CPU work must therefore
run somewhere that cannot touch cells.

**`background` is an ordinary stdlib function, not a keyword:**

```
const compressed = background(fn { compressImage(path) })
```

elp already has closures as values (`onSubmit={fn { ... }}` in
`main.velp`), so this needs no grammar change, and the whole rule lives
in the function's signature where a reader can see it. The asymmetry
with `effect` is justified rather than awkward: `effect` needs things
only the compiler can supply — a lifetime bound to the enclosing
component, and keyed restart — while `background` needs a thread hop
and a type bound.

The bound is **`Sendable`**, an interface that a reactive cell and a
`Binding<T>` do not implement. Everything captured by the closure, and
the value it returns, must conform.

**Conformance is explicit, like every other interface.** `interfaces.md`
already decided conformance is never inferred, so a type that may cross
threads says `MyType implements Sendable`, with primitives built in
and a derive macro in the shape `@Copy`/`@Move` are already planned to
take. This is the existing mechanism rather than a new one, and a
reactive cell fails to conform by simply never declaring it.

> `messages` is reactive state and cannot cross to a background thread
> — pass in the values it needs and write the result back after
> `background` returns.

Two things recommend the name: it is Swift's, whose actor model is
precisely what this section describes, and it is the Rust lowering
verbatim, since an `Rc`-shaped cell handle is not `Send`. The check and
the codegen agree without being separately specified.

### Three aliasing types, and only one of them is confined

`aliasing.md` is authoritative; what matters here is which side of
`background` each one may sit on.

| | may cross `background` | why |
|---|---|---|
| `Shared<T>` | **yes**, when `T` is `Sendable` | immutable — there are no writers to order |
| `Synchronized<T>` | **yes**, when `T` is `Sendable` | internally synchronized — the lock is in the type |
| `Confined<T>` | **never**, under any `T` | nothing orders two writers |
| a reactive cell, `Binding<T>` | **never** | as above, plus they drive the view |

`Confined<T: Sendable> implements Sendable` looks like the obvious rule
and is unsound, which is worth stating here because this is where
somebody will reach for it:

```elp
const hits = Confined.new(0)

background(fn { hits.set(hits.get() + 1) })     // thread B
hits.set(hits.get() + 1)                        // cell thread
```

`int` is as `Sendable` as a type gets, and the update is still lost.
**`T: Sendable` constrains what is in the box; the race is the box.**

So `Confined<T>` simply never declares the conformance, which needs no
new machinery — it is this section's existing rule, *"a reactive cell
fails to conform by simply never declaring it"*, applied to one more
type. Conformance being field-wise then propagates it for free: a type
holding a `Confined` field is not `Sendable`, all the way up, and the
error lands on the capture the developer wrote rather than inside a
generic instantiation.

This is also where `Synchronized<T>` earns its place: a target with no
cell thread has nothing to confine to, so server-side shared state had
no mechanism at all before it existed. That was `examples/Worker`'s
GAP 4.

`background` **blocks its caller and returns the value.** The caller is
a coroutine, so parking it is free and the cell thread stays open;
reading the result is a plain assignment with nothing to await.
Parallelism comes from having several coroutines rather than a second
form of `background`:

```
effect { thumb = background(fn { thumbnail(path) }) }
effect { full  = background(fn { compress(path) }) }
```

Cancellation stays cooperative. If the caller is cancelled while
parked, the background work gets a flag it *may* poll; a CPU loop that
ignores it runs to completion and its result is discarded. No system
does better — a CPU loop cannot be safely interrupted mid-stride — and
pretending otherwise would be the only dishonest option available.

## Channels

**Resolves what was open question 3's remaining half** — the producer
side of a subscription.

**`for` over a channel is the same `for` as over an array.** One
interface, `next() -> Optional<T>`; the only difference is that a
channel's `next` can block, and an empty result means closed and
drained. `for message in room.messages()` is therefore not a new
construct, and a channel is iterable for the same reason an array is.

```
Channel<Message>(capacity = 16)
ch.send(message)
ch.receive() -> Optional<Message>
ch.close()
```

Methods rather than Go's `<-`. elp has no operator soup elsewhere, and
`Optional<T>` already carries the closed signal that Go needs a second
return value for.

**Capacity is required, not defaulted.** Unbounded is a memory leak,
and unbuffered surprises people exactly once in every Go codebase. A
socket feeding a slow UI *should* block its producer: that is
backpressure reaching the server, which is correct behaviour rather
than an edge case.

**Sending to a closed channel panics.** You closed it and then sent,
which is a bug and not a condition — the line `error-handling.md`
already draws between values and panics.

**`Channel<T>` is `Sendable` when `T` is.** This makes it the
sanctioned bridge: a value crosses the thread boundary either through a
channel or through `background`'s return, and `Sendable` is the same
gate on both.

**Channels are not reactive, and markup cannot iterate one.** A channel
is consume-once; a markup `for` needs a materialised collection it can
diff and a `by` identity. So the boundary is clean — a channel is
consumed in an effect, the effect writes a cell, and markup reads
cells. Nothing in a tree ever touches a channel.

**No `select`.** Structured concurrency covers the UI cases without it:
one effect per source, and a timeout is a scope with a deadline rather
than a case in a switch. Fan-in by spawning is simpler than fan-in by
selecting, and `select` can arrive the day something needs "whichever
finishes first", which no corpus case does.

**Most code never constructs one.** `room.messages()` is a library
returning a channel it owns and feeds; application code iterates it.
Closing on scope exit is what stops a cancelled consumer leaving its
producer blocked forever.

## Worked example — everything at once

`ChatRoom` is the corpus's chat component with the hard parts put back.

```
component ChatRoom(roomId string, draft Binding<string>) {
	var messages [Message] = []
	var sending = false
	var err Error

	effect(roomId) {
		const room = joinRoom(roomId)
		defer room.leave()

		for message in room.messages() {
			messages.append(message)
		}
	}

	fn handleSend() {
		if sending { return }
		sending = true
		defer { sending = false }

		return match sendMessage(roomId, draft) {
			Ok(message) -> {
				messages.append(message)
				draft = ""
			}
			Err(e) -> err = e
		}
	}

	return Column {
		for message in messages by message.id {
			Row(style = bubble) {
				Text(message.body)
			}
		}

		Form(onSubmit = handleSend, style = composer) {
			TextInput(&draft, placeholder = "Message")
			Button(type = .FORM_SUBMIT, disabled = sending) {
				Text(if sending { "Sending..." } else { "Send" })
			}
		}
	}
}
```

`draft` is a `Binding<string>` owned by the parent — realistic, since a
draft should survive switching rooms. `TextInput(&draft)` forwards it,
`&` idempotent; `draft = ""` writes through it. That is why the sigil
has to be visible at the `TextInput` call site: it is the other end of
the same wire.

The lifecycle, in order:

- **Mount.** The body runs once. Three cells exist. The effect's
  coroutine starts. The markup is built and each read subscribes — the
  `for` to `messages`, `disabled` and the button label to `sending`.
- **A message arrives.** The effect is blocked in `room.messages()`,
  wakes, and `messages.append(...)` resumes on the cell thread. The
  `for` node inserts one row. The body does not re-run, `handleSend`
  does not re-run, the other subtree is untouched.
- **Double submit.** The first call reads `sending` as false and sets
  it true with no yield between, so the second click reads true and
  returns. `defer` clears it on the way out of whichever branch the
  `match` took.
- **`roomId` changes.** The effect is blocked in `room.messages()`.
  Cancellation lands at that yield point and unwinds, `defer
  room.leave()` runs, and only then does the replacement start.
- **Unmount.** The same path. The cells stay alive on their refcounts
  until the coroutine finishes unwinding.

## Open questions

1. **Store writes are coarse.** The reducer corpus returns a whole new
   `ChatAppState` via `copyWith`, so replacing the object notifies every
   reader of `state` on every action, including `.LOADING(true)`.
   Fine-grained behaviour requires the store write to **diff field-wise**
   and notify only what changed, which is cheap only if `copyWith`
   preserves identity for untouched fields. That is a constraint on
   `copyWith` and needs stating as one. (The reducer/dispatch shape
   itself is syntax exploration rather than a committed design.)


2. **Crossing component boundaries — prop drilling.** References do not
   solve it: the parameter is still written at every hop. The answer is
   scoped injection, and two ad-hoc instances already exist — the
   `defaults` category (`App(textStyle = body)`) and the macro that
   injects `{state, dispatch}`. Generalising them into one `context`
   declaration would be a consolidation rather than an addition, but the
   edges are sharp: injected values must be *shareable* (a cell or a
   `Copy` value — never a move-only owned value), keying needs a
   declaration form to avoid type collisions, lookup should require a
   default so it is total and cannot fail at runtime, and a writable
   context is a back door to exactly the shared mutable state the
   no-globals rule exists to prevent. The mitigation is read-only state
   plus an explicit write capability.

3. ~~**A blocking iterator protocol.**~~ **Resolved** by
   `collections.md`, once generic interfaces were designed in
   `generics.md`. `Iterable<T>.iterator()` returns an `Iterator<T>` with
   `fn next(&var self) -> Optional<T>`, and one protocol covers arrays,
   `Map`, ranges and channels. A channel's iterator blocks in `next()`
   and returns `nil` on close, which is what made "`for` over a channel
   is the same `for`" true rather than aspirational.

## Corpus bug

`ChatRoom/main.velp` passes `Input(name = "display-name", type = "text",
value = name)` by value, while `composer.velp` and `login-form.velp`
both use `&` for the same job. Under the table above the `Input` gets a
read-only share and can never write back, so `dispatch(.LOG_IN(name))`
would always send an empty string. It wants `&name`.
