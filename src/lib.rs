extern crate parrot;

pub mod components;

pub use parrot::geom::{Rect, Point};

#[macro_export]
macro_rules! id {
    () => (::tabula::Id::new(file!(), line!(), column!(), 0,  0, 0));
    ($a: expr) => (::tabula::Id::new(file!(), line!(), column!(), a, 0, 0));
    ($a: expr, $b: expr) => (::tabula::Id::new(file!(), line!(), column!(), a, b, 0));
    ($a: expr, $b: expr, $c: expr) => (::tabula::Id::new(file!(), line!(), column!(), a, b, c));
}

pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect<f32> {
    use ::parrot::geom::{Point, Vector};
    let start = Point(x, y);
    let size = Vector(w, h);
    Rect(start, start + size)
}

pub trait BasicRenderer: components::ButtonRender {
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Ord, PartialOrd)]
pub struct Id {
    line: u32,
    column: u32,
    aux_1: u32,
    aux_2: u32,
    aux_3: u32,
    file: &'static str,
}

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
pub enum Event {
    PointerMove(EventSource, f32, f32),
    PointerDown(EventSource),
    PointerUp(EventSource),
}

#[derive(Eq, Debug, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct EventSource(pub u32);

pub struct UnloadedUiContext {
    pub component_state: ComponentState,
    pub event_data: EventData,
}

pub struct ComponentState {
    pub hot: Vec<(Id, EventSource)>,
    pub active: Vec<(Id, EventSource)>,
}

pub struct EventData {
    pub positions: Vec<(EventSource, Point<f32>)>,
    pub down: Vec<EventSource>,
    pub up: Vec<EventSource>,
}

pub struct UiContext<'a, B> {
    pub backend: B,
    pub state: &'a mut UnloadedUiContext,
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
        }
    }
}

impl EventData {
    pub fn new() -> EventData {
        EventData {
            positions: vec![],
            down: vec![],
            up: vec![],
        }
    }

    pub fn went_down(&self, source: EventSource) -> bool {
        self.down.iter().any(|&a| source == a)
    }

    pub fn went_up(&self, source: EventSource) -> bool {
        self.up.iter().any(|&a| source == a)
    }

    pub fn position_of(&self, source: EventSource) -> Option<Point<f32>> {
        self.positions.iter().filter_map(|&(s, pos)| 
            if s == source { Some(pos) } else { None }
        ).next()
    }
}

impl ComponentState {
    pub fn new() -> ComponentState {
        ComponentState {
            hot: vec![],
            active: vec![],
        }
    }

    fn mark(coll: &mut Vec<(Id, EventSource)>, id: Id, because: EventSource) {
        // Update an existing active reason if one exists
        for &mut (e_id, ref mut reason) in coll.iter_mut() {
            if e_id == id {
                *reason = because;
                return;
            }
        }
        coll.push((id, because));
    }

    pub fn make_active(&mut self, id: Id, because: EventSource) {
        Self::mark(&mut self.active, id, because);
    }

    pub fn make_hot(&mut self, id: Id, because: EventSource) {
        Self::mark(&mut self.hot, id, because);
    }

    pub fn remove_all_active(&mut self, id: Id) {
        self.active.retain(|&(a, _)| a != id);
    }

    pub fn remove_active(&mut self, id: Id, because: EventSource) {
        self.active.retain(|&(a, b)| a != id || b != because);
    }

    pub fn remove_hot(&mut self, id: Id, because: EventSource) {
        self.hot.retain(|&(a, b)| a != id || b != because);
    }

    pub fn remove_all_hot(&mut self, id: Id) {
        self.hot.retain(|&(a, _)| a != id);
    }

    pub fn is_active(&self, id: Id) -> bool {
        self.active.iter().any(|&(a, _)| a == id)
    }

    pub fn is_hot(&self, id: Id) -> bool {
        self.hot.iter().any(|&(a, _)| a == id)
    }

    pub fn why_hot(&self, id: Id) -> Vec<EventSource> {
        self.hot.iter().filter_map(|&(i, e)| if i == id { Some(e) } else { None }).collect()
    }

    pub fn why_active(&self, id: Id) -> Vec<EventSource> {
        self.active.iter().filter_map(|&(i, e)| if i == id { Some(e) } else { None }).collect()
    }
}

impl UnloadedUiContext {
    pub fn new() -> UnloadedUiContext {
        use std::f32::NAN;
        UnloadedUiContext {
            component_state: ComponentState::new(),
            event_data: EventData::new(),
        }
    }

    pub fn load<'a, B>(&'a mut self, b: B) -> UiContext<B> {
        UiContext {
            backend: b,
            state: self
        }
    }

    fn switch_frames(&mut self) {
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
