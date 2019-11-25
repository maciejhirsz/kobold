use stdweb::{js, Value, Reference};
use system_macro::component;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub struct Node(Reference);

impl Node {
    pub fn js(&self) -> &Reference {
        &self.0
    }
}

fn mount<R: Rendered>(rendered: &R) {
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

component! {
    Test(name: &str) {
        <h1 style="text-decoration: underline; color: red;">"Hello "{ name }"!"</h1>

        <p>"This is some schoeny paragraph!"</p>
    }
}

fn main() {
    stdweb::initialize();

    let test = Test::render("Bob");

    mount(&test);

    stdweb::web::set_timeout(move || test.update("Alice"), 5000);
}
