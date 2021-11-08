use cursive::direction::Direction;
use cursive::event::{Callback, Event, EventResult, Key};
use cursive::{Cursive, Printer, View, With};
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;

pub type OnSubmit = dyn Fn(&mut Cursive, &str);

pub struct CliView {
    // Current content
    content: Rc<String>,
    // Cursor position in the content, in bytes.
    cursor: usize,
    // Callback when <Enter> is pressed
    on_submit: Option<Rc<OnSubmit>>,
}

impl CliView {
    pub fn new() -> Self {
        CliView {
            content: Rc::new(String::new()),
            cursor: 0,
            on_submit: None,
        }
    }

    fn insert(&mut self, ch: char) -> Callback {
        Rc::make_mut(&mut self.content).insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
        Callback::dummy()
    }

    fn remove(&mut self, len: usize) -> Callback {
        let start = self.cursor;
        let end = self.cursor + len;
        log::debug!("Removing from {} to {}", start, end);
        for _ in Rc::make_mut(&mut self.content).drain(start..end) {}
        Callback::dummy()
    }

    fn backspace(&mut self) -> EventResult {
        let len = self.content[..self.cursor]
            .graphemes(true)
            .last()
            .unwrap()
            .len();
        self.cursor -= len;
        EventResult::Consumed(Some(self.remove(len)))
    }

    pub fn set_on_submit<F>(&mut self, callback: F)
    where
        F: Fn(&mut Cursive, &str) + 'static,
    {
        self.on_submit = Some(Rc::new(callback));
    }

    pub fn on_submit<F>(self, callback: F) -> Self
    where
        F: Fn(&mut Cursive, &str) + 'static,
    {
        self.with(|v| v.set_on_submit(callback))
    }
}

impl View for CliView {
    fn draw(&self, printer: &Printer) {
        printer.print((0, 0), &self.content);
    }

    fn take_focus(&mut self, source: Direction) -> bool {
        log::debug!("Should focus?");
        true
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char(ch) => {
                return EventResult::Consumed(Some(self.insert(ch)));
            }
            Event::Key(Key::Backspace) if self.cursor > 0 => {
                return self.backspace();
            }
            Event::CtrlChar('h') if self.cursor > 0 => {
                return self.backspace();
            }
            _ => {
                log::debug!("Got unknown event {:?}", event);
                return EventResult::Ignored;
            }
        }
    }
}
