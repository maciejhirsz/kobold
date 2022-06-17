use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};
use std::any::TypeId;
use std::cell::RefCell;
use std::marker::PhantomData;

trait EventedComponent: Sized {
    type State: State<Props = Self>;

    fn init(self) -> Self::State;
}

trait State: 'static {
    type Message;

    type Props: EventedComponent<State = Self>;

    type Out;

    fn render(&self) -> Self::Out;
}

// trait Handle<Message> {
//     fn update(&mut self, message: Message) -> bool;
// }

fn link<S: State>(f: impl Fn(&mut S)) -> Link<impl FnMut()> {
    let type_id = TypeId::of::<S>();

    EVENTED_STACK.with(move |stack| {
        let &(tid, ptr) = stack.borrow_mut().last().unwrap();

        assert_eq!(tid, type_id);

        let rc = unsafe { &*(ptr as *const Rc<RefCell<S>> )}.clone();

        Link(move || f(&mut rc.borrow_mut()))
    })
}

pub struct Link<F: FnMut()>(F);

// pub struct Link<S, F: Fn(&mut S)> {
//     closure: F,
//     _marker: PhantomData<S>,
// }

trait Component {
    type Out;

    fn render(self) -> Self::Out;
}

thread_local! {
    static EVENTED_STACK: RefCell<Vec<(TypeId, *const ())>> = RefCell::new(Vec::new());
}

impl<E> Component for E
where
    E: EventedComponent,
{
    type Out = <E::State as State>::Out;

    fn render(self) -> Self::Out {
        let state = Rc::new(RefCell::new(self.init()));

        let stack_ref = (TypeId::of::<E::State>(), &state as *const _ as *const ());

        EVENTED_STACK.with(move |stack| {
            stack.borrow_mut().push(stack_ref);
        });

        let out = E::State::render(&state.borrow());

        EVENTED_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });

        out
    }
}

// #[kobold]
// fn Modal(name: &str) -> impl Html {
//     <p>{name}</p>
// }
