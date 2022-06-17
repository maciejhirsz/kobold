use std::cell::RefCell;
use std::rc::{Rc, Weak};

use wasm_bindgen::JsValue;
use web_sys::Node;

mod util;

pub trait Html: Sized {
    type Built: Mountable;

    fn build(self) -> Self::Built;

    fn update(self, built: &mut Self::Built);
}

pub trait Mountable: 'static {
    fn js(&self) -> &JsValue;

    fn mount(&self, parent: &Node) {
        util::__kobold_mount(parent, self.js());
    }

    fn unmount(&self, parent: &Node) {
        util::__kobold_unmount(parent, self.js());
    }
}

pub type ShouldRender = bool;

pub trait EventedComponent: Sized {
    type State: Render;

    fn init(self, link: Link<Self::State>) -> Self::State;

    fn update(self, link: Link<Self::State>, state: &mut Self::State) -> ShouldRender {
        *state = self.init(link);

        true
    }
}

pub trait Render: 'static {
    type Out: Html;

    fn render(&self) -> Self::Out;
}

pub struct Link<'a, S: Render> {
    state: &'a Weak<RefCell<S>>,
}

impl<S: Render> Link<'_, S> {
    pub fn link(&self, f: impl Fn(&mut S) -> ShouldRender + 'static) -> Closure {
        let state = self.state.clone();

        Closure(Box::new(move || {
            if let Some(rc) = state.upgrade() {
                let mut state = rc.borrow_mut();

                if f(&mut state) {
                    state.render();
                }
            }
        }))
    }
}

pub struct Closure(Box<dyn FnMut()>);

trait Component: Sized {
    type Built: Mountable;

    // fn create(self) -> Self::Built;

    // fn update(self, _built: &mut Self::Built);
}

trait SimpleComponent: Sized {
    type Out: Html;

    fn render(self) -> Self::Out;

    fn create(self) -> <Self::Out as Html>::Built {
        self.render().build()
    }

    fn update(self, built: &mut <Self::Out as Html>::Built) {
        self.render().update(built);
    }
}

struct EventedBuilt<State: Render> {
    built_html: <State::Out as Html>::Built,
    state: State,
}

impl<State: Render> Mountable for EventedBuilt<State> {
    fn js(&self) -> &JsValue {
        self.built_html.js()
    }

    fn mount(&self, parent: &Node) {
        self.built_html.mount(parent);
    }

    fn unmount(&self, parent: &Node) {
        self.built_html.unmount(parent);
    }
}

impl<E> Component for E
where
    E: EventedComponent,
{
    type Built = EventedBuilt<E::State>;

    // fn create(self) -> Self::Built {

    // }
}

// pub trait Component: Sized {
//     type Properties;

//     fn create(props: Self::Properties) -> Self;

//     fn update(&mut self, new: Self::Properties) -> ShouldRender;
// }

// struct Evented<Out> {
//     Out
// }

// impl<E> Component for E
// where
//     E: EventedComponent,
// {
//     type Out = <E::State as Render>::Out;

//     fn render(self) -> Self::Out {
//         let state = Rc::new_cyclic(|state| RefCell::new(self.init(Link { state })));

//         let state = state.borrow();

//         state.render()
//     }
// }

// struct MyState;


// impl State for MyState {
//     #[kobold]
//     fn render(&self) -> impl Html {
//         panic!()
//     }
// }

// #[kobold]
// fn Modal(name: &str) -> impl Html {
//     <p>{name}</p>
// }

// struct Modal<'a> {
//     name: &'a str,
// }

// kobold! {
//     impl Component for Modal<'_> {
//         fn render()
//     }
// }
