# elp

[![https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg](https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg)](https://good-labs.github.io/greater-good-affirmation/participate/)
[![codecov](https://codecov.io/gh/elp-lang/elp/graph/badge.svg?token=YKcXc1uHNW)](https://codecov.io/gh/elp-lang/elp)

GNU Licensed compiler for the Ellipsis ('elp) language, very much a work in progress programming language inspired by the likes of modern; platform-specific, UI languages and tools such as Swift UI and Jetpack Compose. Compiles to native binaries for Android, iOS, Mac, Windows and Linux and html/css/javascript for the web.


## Elp?

Originally I was calling it ellipsis, for no other reason than I thought it sounded cool but it evolved into 'elp which is a slang/colloquialism for "help" here in England.


### Obligatory "hello world"

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

More complete examples can be found in the [examples](https://github.com/elp-lang/elp/tree/main/examples) folder.


### Notes and other resources

[Goals](https://github.com/elp-lang/elp/blob/main/GOALS.md)
[Transparency](https://vsoch.github.io/2019/transparency/)
