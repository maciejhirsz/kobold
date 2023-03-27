<img src="https://raw.githubusercontent.com/maciejhirsz/kobold/master/kobold.svg?sanitize=true" alt="Kobold logo" width="250" align="right" style="max-width: 40vw;">

# Kobold

_Easy declarative web interfaces._

**Kobold** uses macros to deliver familiar JSX-esque syntax for building declarative web interfaces,
while leveraging Rust's powerful type system for safety and performance.

### Zero-Cost Static HTML

The `view!` macro produces opaque `impl View` types that by default do no allocations.
All static [DOM](https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model) elements compile to
inline JavaScript code that constructs them. Expressions are injected into the constructed DOM on first render.
Kobold keeps track of the DOM node references for these expressions.

Since the exact types the expressions evaluate to are known to the Rust compiler, update calls can diff them by
value (or pointer) and surgically update the DOM should they change. Changing a
string or an integer only updates the exact [`Text` node](https://developer.mozilla.org/en-US/docs/Web/API/Text)
that string or integer was rendered to.

_If the `view!` macro invocation contains DOM elements with no expressions, the constructed `View`
type will be zero-sized, and its `View::update` method will be empty, making updates of static
DOM literally zero-cost._

### Hello World!

Components in **Kobold** are created by annotating a _render function_ with a `#[component]` attribute.

```rust
use kobold::prelude::*;

#[component]
fn Hello(name: &str) -> impl View + '_ {
    view! {
        <h1>"Hello "{ name }"!"</h1>
    }
}

fn main() {
    kobold::start(view! {
        <Hello name="Kobold" />
    });
}
```

The component function must return a type that implements the `View` trait. Since the `view!` macro
produces transient locally defined types the best approach here is to always use the opaque `impl View` return type.

Everything here is statically typed and the macro doesn't delete any information when manipulating the
token stream, so the Rust compiler can tell you when you've made a mistake:

```text
error[E0560]: struct `Hello` has no field named `nam`
  --> examples/hello_world/src/main.rs:12:16
   |
12 |         <Hello nam="Kobold" />
   |                ^^^ help: a field with a similar name exists: `name`
```

You can even use [rust-analyzer](https://rust-analyzer.github.io/) to refactor component or field names,
and it will change the invocations inside the macros for you.

### Stateful

The `stateful` function can be used to create views that own and manipulate
their state:

```rust
use kobold::prelude::*;

#[component]
fn Counter(init: u32) -> impl View {
    stateful(init, |count| {
        bind! { count:
            // Create an event handler with access to `&mut u32`
            let onclick = move |_event| *count += 1;
        }

        view! {
            <p>
                "You clicked the "
                // `{onclick}` here is shorthand for `onclick={onclick}`
                <button {onclick}>"Button"</button>
                " "{ count }" times."
            </p>
        }
    })
}

fn main() {
    kobold::start(view! {
        <Counter init={0} />
    });
}
```

The `stateful` function takes two parameters:

* State constructor that implements the `IntoState` trait. **Kobold** comes with default
  implementations for most primitive types, so we can use `u32` here.
* The anonymous render function that uses the constructed state, in our case its argument is `&Hook<u32>`.

The `Hook` here is a smart pointer to the state itself that allows non-mutable access to the
state. The `bind!` macro can be invoked for any `Hook` to create closures with `&mut` references to the
underlying state.

For more details visit the [`stateful` module documentation](https://docs.rs/kobold/latest/kobold/stateful/index.html).

### Conditional Rendering

Because the `view!` macro produces unique transient types, `if` and `match` expressions that invoke
the macro will naturally fail to compile.

Using the `auto_branch` flag on the `#[component]` attribute
**Kobold** will scan the body of of your component render function, and make all `view!` macro invocations
inside an `if` or `match` expression, and wrap them in an enum making them the same type:

```rust
#[component(auto_branch)]
fn Conditional(illuminatus: bool) -> impl View {
    if illuminatus {
        view! { <p>"It was the year when they finally immanentized the Eschaton."</p> }
    } else {
        view! { <blockquote>"It was love at first sight."</blockquote> }
    }
}
```

For more details visit the [`branching` module documentation](https://docs.rs/kobold/latest/kobold/branching/index.html).

### Lists and Iterators

To render an iterator use the `for` keyword:

```rust
// `ListIteratorExt` is included in the prelude
use kobold::prelude::*;

#[component]
fn IterateNumbers(count: u32) -> impl View {
    view! {
        <ul>
        {
            for (1..=count).map(|n| view! { <li>"Item #"{n}</li> })
        }
        </ul>
    }
}
```

On updates the iterator is consumed once and all items are diffed with the previous version.
No allocations are made by **Kobold** when updating such a list, unless the rendered list needs
to grow past its original capacity.

For more information about keywords visit the [`keywords` module documentation](https://docs.rs/kobold/latest/kobold/keywords/index.html).

### Borrowed Values

`View` types are truly transient and only need to live for the duration of the initial render,
or for the duration of the subsequent update. This means that you can easily and cheaply render borrowed
state without unnecessary clones:

```rust
#[component]
fn Users<'a>(names: &'a [&'a str]) -> impl View + 'a {
    view! {
        <ul>
        {
            for names.iter().map(|name| view! { <li>{ name }</li> })
        }
        </ul>
    }
}
```

### Components with Children

If you wish to capture children from parent `view!` invocation, simply change
`#[component]` to `#[component(children)]`:

```rust
use kobold::prelude::*;

#[component(children)]
fn Header(children: impl View) -> impl View {
    view! {
        <header><h1>{ children }</h1></header>
    }
}

fn main() {
    kobold::start(view! {
        <Header>"Hello Kobold"</Header>
    });
}
```

You can change the name of the parameter used and even set it to a concrete:

```rust
use kobold::prelude::*;

// Capture children into the argument `n`
#[component(children: n)]
fn AddTen(n: i32) -> i32 {
    // integers implement `View` so they can be passed by value
    n + 10
}

fn main() {
    kobold::start(view! {
        <p>
            "Meaning of life is "
            <AddTen>{ 32 }</AddTen>
        </p>
    });
}
```

## More Examples

To run **Kobold** you'll need to install [`trunk`](https://trunkrs.dev/) (check the [full instructions](https://trunkrs.dev/#install) if you have problems):
```sh
cargo install --locked trunk
```

You might also need to add the Wasm target to Rust:
```sh
rustup target add wasm32-unknown-unknown
```

Then just run an example:
```sh
## Go to an example
cd examples/todomvc

## Run with trunk
trunk serve
```

## Acknowledgements

+ [Pedrors](https://pedrors.pt/) for the **Kobold** logo.

## License

Kobold is free software, and is released under the terms of the [Mozilla Public License](https://www.mozilla.org/en-US/MPL/) version 2.0. See [LICENSE](LICENSE).
