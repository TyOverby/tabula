extern crate parrot;

pub use parrot::geom::{Rect, Point, Vector};
use parrot::geom::{Contains, Translate};

#[macro_export]
macro_rules! id {
    () => ($crate::Id::new(file!(), line!(), column!(), 0,  0, 0));
    ($a: expr) => ($crate::Id::new(file!(), line!(), column!(), a, 0, 0));
    ($a: expr, $b: expr) => ($crate::Id::new(file!(), line!(), column!(), a, b, 0));
    ($a: expr, $b: expr, $c: expr) => ($crate::Id::new(file!(), line!(), column!(), a, b, c));
}

pub mod components;

pub struct NullRenderer;

pub struct PositionIterator<'a> {
    slice: &'a[(EventSource, Point<f32>)],
    masks: &'a[Rect<f32>],
    offset: (f32, f32),
}

#[derive(Debug, PartialEq, Eq, Clone, Ord, PartialOrd)]
pub struct Id {
    line: u32,
    column: u32,
    aux_1: u32,
    aux_2: u32,
    aux_3: u32,
    file: &'static str,
    parent: Option<Box<Id>>,
}

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
pub enum Event {
    PointerMove(EventSource, f32, f32),
    PointerDown(EventSource),
    PointerUp(EventSource),
}

#[derive(Eq, Debug, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct EventSource(pub u32, pub u32);

pub struct UiState {
    pub component_state: ComponentState,
    pub event_data: EventData,
}

pub struct ComponentState {
    pub hot: Vec<(Id, EventSource)>,
    pub active: Vec<(Id, EventSource)>,
}

pub struct EventData {
    prev_positions: Vec<(EventSource, Point<f32>)>,
    positions: Vec<(EventSource, Point<f32>)>,
    down: Vec<EventSource>,
    up: Vec<EventSource>,
    offset: (f32, f32),
    masks: Vec<Rect<f32>>,
}

pub struct UiContext<'a, B: 'a> {
    pub backend: &'a mut B,
    pub state: &'a mut UiState,
}

impl Id {
    pub fn new(file: &'static str, line: u32, column: u32, aux_1: u32, aux_2: u32, aux_3: u32) -> Id {
        Id {
            line: line,
            column: column,
            aux_1: aux_1,
            aux_2: aux_2,
            aux_3: aux_3,
            file: file,
            parent: None,
        }
    }

    pub fn with_parent(mut self, other: Id) -> Id {
        self.parent = Some(Box::new(other));
        self
    }
}

impl EventData {
    pub fn new() -> EventData {
        EventData {
            prev_positions: vec![],
            positions: vec![],
            down: vec![],
            up: vec![],
            offset: (0.0, 0.0),
            masks: vec![],
        }
    }

    pub fn positions(&self) -> PositionIterator {
        PositionIterator {
            slice: &self.positions,
            masks: &self.masks,
            offset: self.offset,
        }
    }

    pub fn offset(&mut self, x: f32, y: f32) {
        self.offset.0 += x;
        self.offset.1 += y;
    }

    fn is_in_mask(&self, p: Point<f32>) -> bool {
        self.masks.iter().all(|mask| mask.contains(p))
    }

    pub fn push_mask(&mut self, mask: Rect<f32>) {
        let mask = mask.translate(self.offset.0, self.offset.1);
        self.masks.push(mask);
    }

    pub fn pop_mask(&mut self) {
        self.masks.pop();
    }

    pub fn pointer_movement(&self, source: &EventSource) -> Option<(f32, f32)> {
        // Don't worry about the offset for these calls to fetch_position, because
        // we are just calculating a delta anyway.
        match (EventData::fetch_position(self.positions.iter().cloned(), source),
               EventData::fetch_position(self.prev_positions.iter().cloned(), source)) {
            (Some(pos), Some(prev)) if self.is_in_mask(prev) => {
                let Vector(x, y) = pos - prev;
                Some((x, y))
            }
            _ => None
        }
    }

    pub fn went_down(&self, source: &EventSource) -> bool {
        self.down.iter().any(|&a| *source == a)
    }

    pub fn went_up(&self, source: &EventSource) -> bool {
        self.up.iter().any(|&a| *source == a)
    }

    fn fetch_position<I>(list: I, source: &EventSource) -> Option<Point<f32>>
    where I: Iterator<Item=(EventSource, Point<f32>)>{
        list.filter_map(|(s, pos)|
            if s == *source { Some(pos) } else { None }
        ).next()
    }

    pub fn position_of(&self, source: &EventSource) -> Option<Point<f32>> {
        match EventData::fetch_position(self.positions(), source) {
            Some(p) if self.is_in_mask(p) => Some(p),
            _ => None
        }
    }
}

impl ComponentState {
    pub fn new() -> ComponentState {
        ComponentState {
            hot: vec![],
            active: vec![],
        }
    }

    fn mark(coll: &mut Vec<(Id, EventSource)>, id: &Id, because: EventSource) {
        // Update an existing active reason if one exists
        for &mut (ref e_id, ref mut reason) in coll.iter_mut() {
            if *e_id == *id {
                *reason = because;
                return;
            }
        }
        coll.push((id.clone(), because));
    }

    pub fn make_active(&mut self, id: &Id, because: EventSource) {
        Self::mark(&mut self.active, id, because);
    }

    pub fn make_hot(&mut self, id: &Id, because: EventSource) {
        Self::mark(&mut self.hot, id, because);
    }

    pub fn remove_all_active(&mut self, id: &Id) {
        self.active.retain(|&(ref a, _)| *a != *id);
    }

    pub fn remove_active(&mut self, id: &Id, because: EventSource) {
        self.active.retain(|&(ref a, b)| *a != *id || b != because);
    }

    pub fn remove_hot(&mut self, id: &Id, because: EventSource) {
        self.hot.retain(|&(ref a, b)| *a != *id || b != because);
    }

    pub fn remove_all_hot(&mut self, id: &Id) {
        self.hot.retain(|&(ref a, _)| *a != *id);
    }

    pub fn is_active(&self, id: &Id) -> bool {
        self.active.iter().any(|&(ref a, _)| *a == *id)
    }

    pub fn is_hot(&self, id: &Id) -> bool {
        self.hot.iter().any(|&(ref a, _)| *a == *id)
    }

    pub fn why_hot(&self, id: &Id) -> Vec<EventSource> {
        self.hot.iter().filter_map(|&(ref i, e)| if *i == *id { Some(e) } else { None }).collect()
    }

    pub fn why_active(&self, id: &Id) -> Vec<EventSource> {
        self.active.iter().filter_map(|&(ref i, e)| if *i == *id { Some(e) } else { None }).collect()
    }
}

impl UiState {
    pub fn new() -> UiState {
        UiState {
            component_state: ComponentState::new(),
            event_data: EventData::new(),
        }
    }

    pub fn load<'a, B>(&'a mut self, b: &'a mut B) -> UiContext<B> {
        UiContext {
            backend: b,
            state: self
        }
    }

    fn switch_frames(&mut self) {
        self.event_data.prev_positions = self.event_data.positions.clone();
        self.event_data.down.clear();
        self.event_data.up.clear();
    }

    fn feed_event(&mut self, event: Event) {
        match event {
            Event::PointerMove(i, x, y) => {
                for positions in &mut self.event_data.positions {
                    if positions.0 == i {
                        *positions = (i, Point(x, y));
                        return;
                    }
                }
                self.event_data.positions.push((i, Point(x, y)));
            }
            Event::PointerDown(i) => {
                self.event_data.down.push(i);
            }
            Event::PointerUp(i) => {
                self.event_data.up.push(i);
            }
        }
    }

    pub fn feed_events_for_frame<I: Iterator<Item=Event>>(&mut self, events: I) {
        self.switch_frames();
        for event in events { self.feed_event(event); }
    }
}

impl <'a, T> UiContext<'a, T> {
    pub fn with_other_backend<O, F, R>(&mut self, other: &mut O, f: F) -> R
    where F: for<'c> FnOnce(&'c mut UiContext<'c, O>) -> R {
        let &mut UiContext { ref mut state, .. } = self;
        {
            let mut temp = UiContext {state: state, backend: other};
            f(&mut temp)
        }
    }
}

impl components::ButtonRender for NullRenderer {
    type Error = ();

    fn draw_button(&mut self, _: &Id, _: Rect<f32>, _: &str, _: bool, _: bool) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl components::SliderRender for NullRenderer {
    type Error = ();

    fn draw_slider(&mut self, _: Id, _: Rect<f32>, _: f32) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl components::DragRegionRender for NullRenderer {
    type Error = ();

    fn draw_drag_region(&mut self, _: &Id, _: Rect<f32>, _: components::DragAction) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl <'a> Iterator for PositionIterator<'a> {
    type Item = (EventSource, Point<f32>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.len() == 0 {
            return None;
        }

        let (f_es, f_pt) = self.slice[0];
        self.slice = &self.slice[1 ..];
        let new_point = Point(f_pt.0 - self.offset.0, f_pt.1 - self.offset.1);
        if self.masks.iter().all(|mask| mask.contains(f_pt)) {
            Some((f_es, new_point))
        } else {
            self.next()
        }
    }
}
