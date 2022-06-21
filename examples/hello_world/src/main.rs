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
        <Hello name="Kobold" />
    });
}
