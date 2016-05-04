use ::{ UiState, UiContext, Rect, Id };
use parrot::geom::{Contains, Point};

pub trait SliderRender {
    type Error;

    fn draw_slider(
        &mut self,
        id: Id,
        rect: Rect<f32>,
        covering: f32) -> Result<(), Self::Error>;
}

pub fn slider<B: SliderRender>(ctx: &mut UiContext<B>, id: Id, rect: Rect<f32>, percentage: &mut f32) -> Result<bool, B::Error> {
    let &mut UiState { ref mut component_state, ref event_data }  = ctx.state;
    let mut covering = *percentage;
    let mut moved = false;

    for (source, pos) in event_data.positions() {
        if rect.contains(pos) {
            component_state.make_hot(&id, source);
            if event_data.went_down(&source) {
                component_state.make_active(&id, source);
            }
        } 
    }

    if component_state.is_active(&id) {
        for source in component_state.why_active(&id) {
            if event_data.went_up(&source) {
                component_state.remove_active(&id, source);
                continue;
            }
            if let Some(Point(x, _)) = event_data.position_of(&source) {
                covering = ((x - (rect.0).0) / rect.width()).min(1.0).max(0.0);
                moved = true;
            }
        }
    }

    try!(ctx.backend.draw_slider(id, rect, covering));
    *percentage = covering;
    Ok(moved)
}
