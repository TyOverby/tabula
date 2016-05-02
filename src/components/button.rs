use ::{ UnloadedUiContext, UiContext, Rect, Id };
use parrot::geom::{Contains};

pub trait ButtonRender {
    type Error;

    fn draw_button(
        &mut self,
        id: &Id,
        rect: Rect<f32>,
        label: &str,
        hovered: bool,
        pressed: bool) -> Result<(), Self::Error>;
}

pub fn button<B: ButtonRender>(ctx: &mut UiContext<B>, id: Id, rect: Rect<f32>, label: &str) -> Result<bool, B::Error> {
    let &mut UnloadedUiContext { ref mut component_state, ref event_data }  = ctx.state;
    let mut activated = false;

    let mut any_over = false;
    for &(source, pos) in &event_data.positions {
        if rect.contains(pos) {
            any_over = true;

            component_state.make_hot(&id, source);
            if event_data.went_down(&source) {
                component_state.make_active(&id, source);
            }

            if component_state.is_active(&id) && event_data.went_up(&source) {
                activated = true;
                component_state.remove_active(&id, source);
            }
        } else if event_data.went_up(&source) {
            component_state.remove_active(&id, source);
        }
    }

    if !any_over {
        component_state.remove_all_hot(&id);
    }

    let is_hot = component_state.is_hot(&id);
    let is_active = component_state.is_active(&id);
    try!(ctx.backend.draw_button(&id, rect, label, is_hot, is_active));
    Ok(activated)
}
