use crate::ptr::Weak;
use crate::traits::{Component, MessageHandler};
use std::fmt::{self, Debug};

pub struct Link<T: Component + ?Sized> {
    // once GATs are stabilized and trait methods can return
    // `impl Trait`, we can get rid of the `dyn` here.
    handler: Weak<dyn MessageHandler<Message = T::Message>>,
}

impl<T: Component + ?Sized> Debug for Link<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Link")
    }
}

impl<T> Clone for Link<T>
where
    T: Component + ?Sized,
{
    fn clone(&self) -> Self {
        Link {
            handler: self.handler.clone(),
        }
    }
}

impl<T: Component> Link<T> {
    pub(crate) fn new(handler: Weak<impl MessageHandler<Message = T::Message>>) -> Self {
        Link {
            handler: handler.coerce(),
        }
    }

    pub fn bind<E>(&self, mut f: impl FnMut(&E) -> T::Message) -> impl FnMut(&E) {
        let link = self.clone();

        move |event| {
            if let Some(mut handler) = link.handler.borrow() {
                handler.handle(f(event));
            }
        }
    }
}
