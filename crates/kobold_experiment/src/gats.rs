#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

pub trait Html {
    type Product;

    fn build(self) -> Self::Product;

    fn update(self, built: &mut Self::Product);
}

impl Html for &str {
    type Product = String;

    fn build(self) -> Self::Product {
        self.to_string()
    }

    fn update(self, built: &mut Self::Product) {
        built.clear();
        built.push_str(self);
    }
}

struct Counter {
    name: String,
}

trait Component {
    type Out<'a> where Self: 'a;

    fn render<'a>(&'a self) -> Self::Out<'a>;
}

impl Component for Counter {
    type Out<'a> = impl Html + 'a;

    fn render<'a>(&'a self) -> Self::Out<'a> {
        self.name.as_str()
    }
}

struct Wrapper<'a, C: Component>(&'a C);

impl<'a, C: Component> Html for Wrapper<'a, C>
where
    C::Out<'a>: Html,
{
    type Product = <<C as Component>::Out<'a> as Html>::Product;

    fn build(self) -> Self::Product {
        self.0.render().build()
    }

    fn update(self, p: &mut Self::Product) {
        self.0.render().update(p);
    }
}

fn render(counter: &Counter) -> impl Html + '_ {
    Wrapper(counter)
}
