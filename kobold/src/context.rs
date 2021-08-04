use crate::traits::{Component, HandleMessage};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Context<Component> {
    component: Rc<RefCell<Component>>,
}

impl<Component> Clone for Context<Component> {
    fn clone(&self) -> Self {
        Context {
            component: self.component.clone(),
        }
    }
}

impl<Comp: Component> Context<Comp> {
    pub(crate) fn new(component: Comp) -> Self {
        let component = Rc::new(RefCell::new(component));

        Context { component }
    }

    pub fn bind<Callback, Message, Param>(&self, callback: Callback) -> impl FnMut(Param)
    where
        Callback: Fn(Param) -> Message,
        Comp: HandleMessage<Message>,
    {
        let weak = Rc::downgrade(&self.component);

        move |event| {
            let msg = callback(event);

            if let Some(rc) = Weak::upgrade(&weak) {
                if let Ok(mut component) = rc.try_borrow_mut() {
                    component.handle(msg);
                }
            }
        }
    }
}
