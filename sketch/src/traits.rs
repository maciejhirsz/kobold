use web_sys::Node;

pub type ShouldRender = bool;

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
        parent.append_child(self.node()).unwrap();
    }

    fn unmount(&self, parent: &Node) {
        parent.remove_child(self.node()).unwrap();
    }
}

pub trait Component: Sized {
    type Properties;

    fn create(props: Self::Properties) -> Self;

    fn update(&mut self, new: Self::Properties) -> ShouldRender {
        *self = Self::create(new);
        true
    }
}

// pub trait StatelessComponent {}

// impl<T: StatelessComponent> Component for T {
//     type Properties = Self;

//     #[inline]
//     fn create(props: Self::Properties) -> Self {
//         props
//     }

//     #[inline]
//     fn update(&mut self, new: Self::Properties) -> ShouldRender {
//         *self = Self::create(new);
//         true
//     }
// }
