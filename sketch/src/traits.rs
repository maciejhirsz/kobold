use web_sys::Node;

pub trait Html: Sized {
    type Rendered: Update<Self> + Mountable;

    fn render(self) -> Self::Rendered;
}

pub trait Update<H> {
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
