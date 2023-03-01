<img src="https://raw.githubusercontent.com/maciejhirsz/kobold/master/kobold.svg?sanitize=true" alt="Kobold logo" width="250" align="right" style="max-width: 40vw;">

# Kobold

_Easy declarative web interfaces._

**Kobold** uses macros to deliver familiar HTML-esque syntax for building declarative web interfaces,
while leveraging Rust's powerful type system for safety and performance.

### Zero-cost static HTML

Like in [React](https://reactjs.org/) or [Yew](https://yew.rs/) updates are done by repeating calls
to a render function whenever the state changes. However, unlike either, **Kobold** does not produce a
full blown [virtual DOM](https://en.wikipedia.org/wiki/Virtual_DOM). Instead the [`html!`](html) macro compiles
all static HTML elements to a single JavaScript function that constructs the exact
[DOM](https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model) for it.

All expressions, which must implement the [`Html`](Html) trait, are injected into the constructed DOM on first
render. Kobold keeps track of the DOM node references for these expressions. Since the exact types the
expressions evaluate to are known to the Rust compiler, update calls can diff them by value and surgically
update the DOM should they change. Changing a string or an integer only updates the exact
[`Text` node](https://developer.mozilla.org/en-US/docs/Web/API/Text) that string or integer was rendered to.

_If the `html!` macro invocation contains HTML elements with no expressions, the constructed `Html`
type will be zero-sized, and its `Html::update` method will be empty, making updates of static
HTML quite literally zero-cost._

### Hello World!

Components in **Kobold** are created by annotating a _render function_ with a `#[component]` attribute.

```rust
use kobold::prelude::*;

#[component]
fn Hello(name: &str) -> impl Html {
    html! {
        <h1>"Hello "{ name }"!"</h1>
    }
}

fn main() {
    kobold::start(html! {
        <Hello name="Kobold" />
    });
}
```

The _render function_ must return a type that implements the `Html` trait. Since the `html!` macro
produces _transient types_, or [_Voldemort types_](https://wiki.dlang.org/Voldemort_types), the best approach
here is to always use the `impl Html` return type.

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

### Stateful components

The [`stateful`](stateful::stateful) function can be used to create components that own and manipulate
their state:

```rust
use kobold::prelude::*;

#[component]
fn Counter() -> impl Html {
    stateful(0_u32, |count| {
        let onclick = count.bind(|count, _event| *count += 1);

        html! {
            <p>
                "You clicked on the "
                // `{onclick}` here is shorthand for `onclick={onclick}`
                <button {onclick}>"Button"</button>
                " "{ count }" times."
            </p>
        }
    })
}

fn main() {
    kobold::start(html! {
        <Counter />
    });
}
```

The `stateful` function takes two parameters:

* State constructor that implements the `IntoState` trait. **Kobold** comes with default
  implementations for most primitive types, so we can use `u32` here.
* The anonymous render function that uses the constructed state, in our case `fn(&Hook<u32>) -> impl Html`.

The `Hook` here is a smart pointer to the state itself that allows non-mutable access to the
state, as well as the `bind` method for creating event callbacks. These take a `&mut`
reference to the state and a `&` reference to a DOM `Event` (ignored above).

For more details visit the [`stateful` module documentation](https://docs.rs/kobold/latest/kobold/stateful/index.html).

### Conditional rendering

Because the `html!` macro produces unique transient types, `if` and `match` expressions that invoke
the macro will naturally fail to compile.

Using the `auto_branch` flag on the `#[component]` attribute
**Kobold** will scan the body of of your component render function, and make all `html!` macro invocations
inside an `if` or `match` expression, and wrap them in an enum making them the same type:


```rust
# use kobold::prelude::*;
#[component(auto_branch)]
fn Conditional(illuminatus: bool) -> impl Html {
    if illuminatus {
        html! { <p>"It was the year when they finally immanentized the Eschaton."</p> }
    } else {
        html! { <blockquote>"It was love at first sight."</blockquote> }
    }
}
```

For more details visit the [`branching` module documentation](https://docs.rs/kobold/latest/kobold/branching/index.html).

### Lists and Iterators

To render an iterator use the `list` method from the
`ListIteratorExt` extension trait:

```rust
// `ListIteratorExt` is included in the prelude
use kobold::prelude::*;

#[component]
fn IterateNumbers(count: u32) -> impl Html {
    html! {
        <ul>
        {
            (1..=count)
                .map(|n| html! { <li>"Item #"{n}</li> })
                .list()
        }
        </ul>
    }
}
```

This wraps the iterator in the transparent `List<_>` type that implements `Html`.
On updates the iterator is consumed once and all items are diffed with the previous version.
No allocations are made by **Kobold** when updating such a list, unless the rendered list needs
to grow past its original capacity.

### Borrowed values

`Html` types are truly transient and only need to live for the duration of the initial render,
or for the duration of the subsequent update. This means that you can easily and cheaply render borrowed
state without unnecessary clones:

```rust
# use kobold::prelude::*;
#[component]
fn Users(names: &[&str]) -> impl Html {
    html! {
        <ul>
        {
            names
                .iter()
                .map(|name| html! { <li>{ name }</li> })
                .list()
        }
        </ul>
    }
}
```

### Components with children

If you wish to capture children from parent `html!` invocation, simply change
`#[component]` to `#[component(children)]`:

```rust
use kobold::prelude::*;

#[component(children)]
fn Header(children: impl Html) -> impl Html {
    html! {
        <header><h1>{ children }</h1></header>
    }
}

fn main() {
    kobold::start(html! {
        <Header>"Hello Kobold"</Header>
    });
}
```

You can change the name of the function argument used, or even set a concrete type:

```rust
use kobold::prelude::*;

// Capture children into the argument `n`
#[component(children: n)]
fn AddTen(n: i32) -> impl Html {
    // integers implement `Html` so they can be passed by value
    n + 10
}

fn main() {
    kobold::start(html! {
        <p>
            "Meaning of life is "
            <AddTen>{ 32 }</AddTen>
        </p>
    });
}
```

## More examples

To run **Kobold** you'll need to install [`trunk`](https://trunkrs.dev/):
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
cd examples/counter

## Run with trunk
trunk serve
```

## Acknowledgements

+ [Pedrors](https://pedrors.pt/) for the **Kobold** logo.

## License

Kobold is free software, and is released under the terms of the GNU Lesser General Public
License version 3. See [LICENSE](LICENSE).
