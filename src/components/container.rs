use ::{ UiContext, Rect, Id, Point };

pub trait ContainerRender {
    type Error;

    fn draw_container<R, F: FnOnce(&mut Self) -> Result<R, Self::Error>> (
        &mut self,
        id: &Id,
        rect: Rect<f32>,
        translate_contents: Point<f32>,
        f: F) -> Result<R, Self::Error>;
}

pub fn with_container<B: ContainerRender, F, R>(ctx: &mut UiContext<B>, id: Id, rect: Rect<f32>, reset_coords: bool, f: F) -> Result<R, B::Error>
where F: FnOnce(&mut UiContext<B>) -> Result<R, B::Error> {
    let &mut UiContext{ ref mut state, ref mut backend } = ctx;
    let trans = if reset_coords { rect.0 } else { Point(0.0, 0.0) };

    state.event_data.push_mask(rect);
    state.event_data.offset(trans.0, trans.1);

    let r = backend.draw_container(&id, rect, trans, |backend| {
        f(&mut state.load(backend))
    });

    state.event_data.offset(-trans.0, -trans.1);
    state.event_data.pop_mask();
    r
}
