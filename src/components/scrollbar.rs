use super::*;
use parrot::geom::clamp;
use ::{Rect, UiContext, Id, Point, NullRenderer};

pub trait ScrollbarRender {
    type Error;
    fn draw_scrollbar(&mut self, id: &Id, entire_rect: Rect<f32>, drag_rect: Rect<f32>, over: bool, down: bool) -> Result<(), Self::Error>;
}

pub fn scrollbar<E, A>(ctx: &mut UiContext<A>, id: Id, rect: Rect<f32>, container_size: f32, contents_size: f32, scroll_pos: &mut f32) -> Result<bool, E>
where A: ScrollbarRender<Error=E> + DragRegionRender<Error=E>
{
    let (w, h) = (rect.width(), rect.height());
    let percent_covering = (container_size / contents_size).min(1.0).max(0.0);
    let size_covering = percent_covering * h;
    let starting_at = (*scroll_pos / contents_size) * h;
    let height_minus_dead_zone = h - size_covering;

    let Point(x, y) = rect.top_left();
    let mut bar_rect = Rect::xywh(x, y + starting_at, w, size_covering);

    ctx.with_other_backend(&mut NullRenderer, |ctx| {
        drag_region(ctx, id!().with_parent(id.clone()), &mut bar_rect).unwrap();
    });

    bar_rect = bar_rect.set_left(rect.left());
    bar_rect = bar_rect.set_top(
        clamp(rect.top(),
              bar_rect.top(),
              rect.top() + height_minus_dead_zone));

    *scroll_pos = ((bar_rect.top() - rect.top()) * contents_size) / h;

    let &mut UiContext{ref mut backend, ..} = ctx;
    let over = false;
    let down = true;

    try!(backend.draw_scrollbar(&id, rect, bar_rect, over, down));

    Ok(false)
}
