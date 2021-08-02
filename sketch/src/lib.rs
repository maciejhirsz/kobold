// use std::any::Any;
use beef::Cow;
use web_sys::Document;

pub use web_sys::Node;
pub use sketch_macro::html;

pub mod reexport {
    pub use web_sys;
    pub use wasm_bindgen;
}

pub trait Html: Sized {
    type Rendered: Update<Self> + Mountable;

    fn render(self) -> Self::Rendered;
}

pub trait Update<H: Html> {
    fn update(&mut self, new: H);
}

pub trait Mountable {
    fn node(&self) -> &Node;

    fn mount(&self, parent: &Node) {
        parent.append_child(&self.node()).unwrap();
    }

    fn unmount(&self, parent: &Node) {
        parent.remove_child(&self.node()).unwrap();
    }
}

pub struct RenderedText {
    text: Cow<'static, str>,
    node: Node,
}

impl Mountable for RenderedText {
    fn node(&self) -> &Node {
        &self.node
    }
}

pub fn document() -> Document {
    let window = web_sys::window().expect("no window exists");
    window.document().expect("window should have a document")
}

impl Html for &'static str {
    type Rendered = RenderedText;

    fn render(self) -> Self::Rendered {
        let node = document().create_text_node(&self).into();

        RenderedText {
            text: Cow::borrowed(self),
            node,
        }
    }
}

impl Html for String {
    type Rendered = RenderedText;

    fn render(self) -> Self::Rendered {
        let node = document().create_text_node(&self).into();

        RenderedText {
            text: Cow::owned(self),
            node,
        }
    }
}

impl Html for Cow<'static, str> {
    type Rendered = RenderedText;

    fn render(self) -> Self::Rendered {
        let node = document().create_text_node(&self).into();

        RenderedText {
            text: self,
            node,
        }
    }
}

impl Html for std::borrow::Cow<'static, str> {
    type Rendered = RenderedText;

    fn render(self) -> Self::Rendered {
        let node = document().create_text_node(&self).into();

        RenderedText {
            text: self.into(),
            node,
        }
    }
}

impl Html for bool {
    type Rendered = RenderedText;

    fn render(self) -> Self::Rendered {
        let text = match self {
            true => "true",
            false => "false",
        };

        text.render()
    }
}

impl Update<&'static str> for RenderedText {
    fn update(&mut self, new: &'static str) {
        self.update(Cow::borrowed(new));
    }
}

impl Update<String> for RenderedText {
    fn update(&mut self, new: String) {
        self.update(Cow::owned(new));
    }
}

impl Update<Cow<'static, str>> for RenderedText {
    fn update(&mut self, new: Cow<'static, str>) {
        if self.text != new {
            self.node.set_text_content(Some(&new));
            self.text = new;
        }
    }
}

impl Update<std::borrow::Cow<'static, str>> for RenderedText {
    fn update(&mut self, new: std::borrow::Cow<'static, str>) {
        let new: Cow<'static, str> = new.into();

        self.update(new);
    }
}

impl Update<bool> for RenderedText {
    fn update(&mut self, new: bool) {
        let text = match new {
            true => "true",
            false => "false",
        };

        self.update(text);
    }
}

macro_rules! impl_render_display {
    ($($t:ty),*) => {
        $(
            impl Html for $t {
                type Rendered = RenderedText;

                fn render(self) -> Self::Rendered {
                    self.to_string().render()
                }
            }

            impl Update<$t> for RenderedText {
                fn update(&mut self, new: $t) {
                    self.update(new.to_string())
                }
            }
        )*
    };
}

impl_render_display!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
