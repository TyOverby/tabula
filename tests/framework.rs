#![feature(set_stdio)]
extern crate tabula;

use tabula::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::io::Write;
use std::io::Result as IoResult;

struct AsciiBackend;

impl Backend for AsciiBackend {
    type Error = ();
    type Image = ();

    fn draw_button(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        label: &str,
        hovered: bool,
        pressed: bool) -> Result<(), Self::Error> {
        println!("Button({:?}, {:?}, {:?}, {:?}) {} | hovered: {:?}, pressed: {:?}",
                           x,    y,    w,    h, label,hovered,       pressed);
        Ok(())
    }

    fn draw_toggle_button(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        label: &str,
        toggled: bool,
        hovered: bool,
        pressed: bool) -> Result<(), Self::Error> {
        println!("ToggleButton({:?}, {:?}, {:?}, {:?}) {} | hovered: {:?}, pressed: {:?}, toggled: {:?}",
                                 x,    y,    w,    h, label,hovered,       pressed,       toggled);
        Ok(())
    }
}

fn shared_writer() -> (Box<Write + Send>, Receiver<u8>) {
    struct WriteSender(Sender<u8>);
    impl Write for WriteSender {
        fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
            for &byte in buf {
                self.0.send(byte).unwrap();
            }
            Ok(buf.len())
        }
        fn flush(&mut self) -> IoResult<()> { Ok(()) }
    }

    let (sx, rx) = channel();

    (Box::new(WriteSender(sx)), rx)
}

fn run_test<F: Fn(&mut UiContext<AsciiBackend>)>(actions: Vec<Vec<Event>>, f: F, expected: &str) {
    let (writer, output) = shared_writer();
    let prev = ::std::io::set_print(writer).unwrap();
    let mut ctx = UnloadedUiContext::new();

    for eventset in actions {
        ctx.switch_frames();
        ctx.feed_events(eventset.into_iter());
        let backend = &mut AsciiBackend;
        let mut ctx = ctx.load(backend);
        f(&mut ctx);
    }
    ::std::io::set_print(prev);

    let content: Vec<u8> = output.iter().collect();
    let content = String::from_utf8(content).unwrap();
    assert_eq!(content, expected);
}

#[test]
fn framework_works() {
    run_test(vec![vec![]], |_| {println!("hi")}, "hi\n");
}
