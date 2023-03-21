pub trait ShouldRender {
    fn should_render(self) -> bool;
}

impl ShouldRender for () {
    fn should_render(self) -> bool {
        true
    }
}

pub enum Then {
    Stop,
    Render,
}


impl ShouldRender for Then {
    fn should_render(self) -> bool {
        match self {
            Then::Stop => false,
            Then::Render => true,
        }
    }
}
