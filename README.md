<img src="https://raw.githubusercontent.com/maciejhirsz/kobold/master/kobold.svg?sanitize=true" alt="Kobold logo" width="250" align="right">

# Kobold

_Easy web interfaces._

**Kobold** uses macros to deliver familiar HTML-esque syntax for building web interfaces in rust,
while leveraging Rust's powerful type system for safety and performance.

There is no need for a full [virtual DOM](https://en.wikipedia.org/wiki/Virtual_DOM), all static
elements are compiled into plain JavaScript functions that construct the [DOM](https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model)
elements. All expressions wrapped in `{ ... }` braces are then injected into that DOM, and updated
if they change.

Like in [React](https://reactjs.org/) or [Yew](https://yew.rs/) updates are done by calling a render
function/method, but unlike either the `html!` macro in Kobold produces transient static types that
implement the `Html` trait. If you have a component that renders a whole bunch of HTML and one `i32`,
only that one `i32` is diffed between previous and current render and updated in DOM if necessary.

### Hello World

Any struct that implements a `render` method can be used as a component:

```rust
use kobold::prelude::*;

struct Hello {
    name: &'static str,
}

impl Hello {
    fn render(self) -> impl Html {
        html! {
            <h1>"Hello "{ self.name }"!"</h1>
        }
    }
}

fn main() {
    kobold::start(html! {
        <Hello name={"Kobold"} />
    });
}
```

## Examples

To run **Kobold** you'll need to install [`trunk`](https://trunkrs.dev/):
```sh
cargo install --locked trunk
```

You might also need the CLI for wasm-bindgen and ability to compile Rust to Wasm:
```sh
cargo install wasm-bindgen-cli

rustup target add wasm32-unknown-unknown
```

Then just run an example:
```sh
# Go to an example
cd examples/counter

# Run with trunk
trunk serve
```

## Acknowledgements

+ [Pedrors](https://pedrors.pt/) for the **Kobold** logo.

## License

Kobold is free software, and is released under the terms of the GNU Lesser General Public
License version 3. See [LICENSE](LICENSE).
