use super::*;
use parrot::geom::clamp;
use ::{Rect, UiContext, Id, Point, NullRenderer};

pub trait ScrollbarRender {
    type Error;
    fn draw_scrollbar(&mut self, id: &Id, entire_rect: Rect<f32>, drag_rect: Rect<f32>, over: bool, down: bool) -> Result<(), Self::Error>;
}

pub fn scrollbar_v<E, A>(ctx: &mut UiContext<A>, id: Id, rect: Rect<f32>, container_size: f32, contents_size: f32, scroll_pos: &mut f32) -> Result<bool, E>
where A: ScrollbarRender<Error=E> + DragRegionRender<Error=E>
{
    scrollbar(ctx, id, rect, container_size, contents_size, scroll_pos,
        Rect::height,
        Rect::top,
        Rect::set_top,
        |dragged, basis| dragged.set_left(basis.left()),
        |outer, starting_at, size_covering| Rect::xywh(outer.left(), outer.top() + starting_at, outer.width(), size_covering))
}
pub fn scrollbar_h<E, A>(ctx: &mut UiContext<A>, id: Id, rect: Rect<f32>, container_size: f32, contents_size: f32, scroll_pos: &mut f32) -> Result<bool, E>
where A: ScrollbarRender<Error=E> + DragRegionRender<Error=E>
{
    scrollbar(ctx, id, rect, container_size, contents_size, scroll_pos,
        Rect::width,
        Rect::left,
        Rect::set_left,
        |dragged, basis| dragged.set_top(basis.top()),
        |outer, starting_at, size_covering| Rect::xywh(outer.left() + starting_at, outer.top(), size_covering, outer.height()))
}

pub fn scrollbar<E, A, DG, DS, CR, CB, SZ>(ctx: &mut UiContext<A>, id: Id, rect: Rect<f32>, container_size: f32, contents_size: f32, scroll_pos: &mut f32, size_getter: SZ, direction_getter: DG, direction_setter: DS, correct: CR, construct_bar: CB) -> Result<bool, E>
where A: ScrollbarRender<Error=E> + DragRegionRender<Error=E>,
      DG: Fn(Rect<f32>) -> f32,
      DS: Fn(Rect<f32>, f32) -> Rect<f32>,
      CR: Fn(Rect<f32>, Rect<f32>) -> Rect<f32>,
      CB: Fn(Rect<f32>, f32, f32) -> Rect<f32>,
      SZ: Fn(Rect<f32>) -> f32,
{
    let sz = size_getter(rect);
    let percent_covering = (container_size / contents_size).min(1.0).max(0.0);
    let size_covering = percent_covering * sz;
    let starting_at = (*scroll_pos / contents_size) * sz;
    let height_minus_dead_zone = sz - size_covering;

    let mut bar_rect = construct_bar(rect, starting_at, size_covering);

    ctx.with_other_backend(&mut NullRenderer, |ctx| {
        drag_region(ctx, id!().with_parent(id.clone()), &mut bar_rect).unwrap();
    });

    bar_rect = correct(bar_rect, rect);
    bar_rect =
        direction_setter(bar_rect,
            clamp(direction_getter(rect),
                  direction_getter(bar_rect),
                  direction_getter(rect) + height_minus_dead_zone));

    *scroll_pos = ((direction_getter(bar_rect) - direction_getter(rect)) * contents_size) / sz;

    let &mut UiContext{ref mut backend, ..} = ctx;
    let over = false;
    let down = true;

    try!(backend.draw_scrollbar(&id, rect, bar_rect, over, down));

    Ok(false)
}
