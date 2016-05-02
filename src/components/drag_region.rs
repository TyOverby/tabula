use ::{ UnloadedUiContext, UiContext, Rect, Id };
use parrot::geom::{Contains};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
pub enum DragAction {
    None,
    PickedUp,
    Holding,
    Dropped,
}

pub trait DragRegionRender {
    type Error;

    fn draw_drag_region(
        &mut self,
        id: &Id,
        rect: Rect<f32>,
        action: DragAction) -> Result<(), Self::Error>;
}

/// Creates a dragable region that can be clicked and dragged.
pub fn drag_region<B: DragRegionRender>(ctx: &mut UiContext<B>, id: Id, rect: &mut Rect<f32>) -> Result<DragAction, B::Error> {
    let &mut UnloadedUiContext { ref mut component_state, ref event_data }  = ctx.state;

    let mut action = DragAction::None;

    if component_state.is_active(&id) {
        let why = component_state.why_active(&id);
        assert!(why.len() == 1);
        let why = why[0];
        if event_data.went_up(&why) {
            component_state.remove_all_active(&id);
            action = DragAction::Dropped;
        } else {
            if let Some((dx, dy)) = event_data.pointer_movement(&why) {
                (rect.0).0 += dx;
                (rect.0).1 += dy;
                (rect.1).0 += dx;
                (rect.1).1 += dy;
            }
            action = DragAction::Holding;
        }
    } else {
        for &(source, pos) in &event_data.positions {
            if rect.contains(pos) && event_data.went_down(&source) {
                component_state.make_active(&id, source);
                action = DragAction::PickedUp;
                break;
            }
        }
    }

    try!(ctx.backend.draw_drag_region(&id, *rect, action));
    return Ok(action);
}
