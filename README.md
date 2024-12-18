# elpc

[![https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg](https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg)](https://good-labs.github.io/greater-good-affirmation)
[![codecov](https://codecov.io/gh/elp-lang/elpc/graph/badge.svg?token=9HRZ43JYME)](https://codecov.io/gh/elp-lang/elpc)

GNU Licensed compiler for the Ellipsis ('elp) language, very much a work in progress programming language inspired by the likes of modern; platform-specific, UI languages and tools such as Swift UI and Jetpack Compose. Compiles to native binaries for Android, iOS, Mac, Windows and Linux and html/css/javascript for the web.

## Why am I doing this?

> “For a successful technology, reality must take precedence over public relations, for nature cannot be fooled.”
> ― Richard P. Feynman

Firstly, apparently I'm a glutton for punishment but mostly because I've been a software developer for a long time and not once have I seen a language designed for the one thing I spend most of my time doing. Creating software for people to use that is safe. Why do we settle for shoe horned languages and frameworks or the argument of _"just do it natively for each platform?"_

I want a single language that compiles to native binaries that target each platform, not a web view, not JavaScript, not owned or deeply funded/owned by sponsorship by a large corporation but a single, source of truth language that allows anyone to build **great** software with no matter what needs to get done.

I will never "sell" this language or it's rights and I don't ever plan to "profit" on this language, nor do I plan to create any kind of bureaucratic body to "manage" it or "progress" it. This is FOSS through and through. We; as the community have a right and an obligation to ensure our software chains and so, I encourage you to fork it, learn it, improve it and submit PRs and together we create a language that we own, control and distribute across platforms. There is nothing to protect, nothing to copyright, trademark or otherwise.

>_Actively; and consciously, design away mechanisms designed to screw anyone else over._

## Elp?

Originally I was calling it ellipsis, for no other reason than I thought it sounded cool but it evolved into 'elp which is a slang/colloquialism for "help" here in England.

## Obligatory "hello world"

Targeting a device with a CLI, we can simply `println("hello world")` in our main function.

```
import { println } from "elp/stdio"

fn main {
    println("hello world")
}
```

### An "app" hello world.

```kotlin
import { App, Window } from "elp/app"
import { Column, Row, Text } from "elp/app/components"

@App
export fn HelloWorld -> App {
	App {
		Window {
			Column {
				Row {
					Text("Hello World")
				}
			}
		}
	}
}
```

More complete example apps can be found in the [examples](https://github.com/elp-lang/elpc/tree/main/examples) folder.

## What am I actually trying to do here?

having spent a large amount of my career across the entire spectrum of software development I've been privvy to a whole range of problems with coding. Specifically the cross platform-ness of any given technology and it's always been a gripe of mine that there wasn't actually a native way to do anything across platforms that didn't involve knowing how to write safe C++ or using enormous DSLs like Qt with it's own compilation and licensing problems.

Thus I have started to try and remedy that by creating a new language, that is under the hood actually a compiler on top of a very large [Directed Graph](https://en.wikipedia.org/wiki/Directed_graph) (I have to document this further but it's very cool.)

### my feature wishlist

Design a language:

* that's easy to understand, test and deploy cross-platform
* that's safe to use
  * Memory safe
  * Type safe
  * Intellectual Property safe.
* that enforces testing
* that enforces the best accessibility standards possible
* has built in tooling for
  * Package management
  * Profiling and debugging with llvm
  * Testing with coverage
  * Supply chain management
  * Opsec management
  * Ci/cd integration
