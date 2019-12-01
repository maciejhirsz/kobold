use stdweb::{js, Value, Reference};
use sketch_macro::sketch;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub struct Node(Reference);

impl Node {
    pub fn js(&self) -> &Reference {
        &self.0
    }
}

fn mount<T: Rendered>(rendered: &T) {
    js! {
        document.body.appendChild(@{ rendered.root().js() })
    }
}

fn to_node(value: Value) -> Node {
    match value {
        Value::Reference(r) => Node(r),
        _ => panic!("Expected reference"),
    }
}

trait Rendered {
    fn root(&self) -> &Node;
}

trait Renderable {
    fn render(&self) -> Node;

    fn update(&self, with: &Node);
}

impl Renderable for str {
    fn render(&self) -> Node {
        to_node(js! {
            return document.createTextNode(@{ self });
        })
    }

    fn update(&self, with: &Node) {
        js! { @(no_return)
            @{ with.js() }.textContent = @{ self };
        }
    }
}

sketch! {
    Test(name: &str) {
        <h1 style="text-decoration: underline; color: red;">"Hello "{ name }"!"</h1>

        <ul>
            <li>"First"</li>
            <li>"Second"</li>
            <li>"Third"</li>
        </ul>
    }
}

fn main() {
    stdweb::initialize();

    let test = Test::render("Bob");

    mount(&test);

    stdweb::web::set_timeout(move || test.update("Alice"), 5000);
}
