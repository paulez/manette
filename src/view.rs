use cursive::direction::Direction;
use cursive::event::{Callback, Event, EventResult, EventTrigger, Key};
use cursive::theme::{ColorStyle, Effect};
use cursive::{Cursive, Printer, View, With, XY};
use cursive::menu::MenuTree;
use cursive::view::Position;
use cursive::views::{MenuPopup, OnEventView};
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use crate::autocomplete::autocomplete;

pub struct CliView {
    // Current content
    content: Rc<String>,
    // Cursor position in the content, in bytes.
    cursor: usize,
    // Callback when <Enter> is pressed
    on_submit: Option<Rc<dyn Fn(&mut Cursive, &str)>>,
    // Character to fill empty space.
    filler: String,
}

impl CliView {
    pub fn new() -> Self {
        CliView {
            content: Rc::new(String::new()),
            cursor: 0,
            on_submit: None,
            filler: " ".to_string(),
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

    // Allows setting the on_submit callback on an existing
    // view
    pub fn set_on_submit<F>(&mut self, callback: F)
    where
        F: Fn(&mut Cursive, &str) + 'static,
    {
        self.on_submit = Some(Rc::new(callback));
    }


    // Allows setting the on_submit callback when creating
    // the view
    pub fn on_submit<F>(self, callback: F) -> Self
    where
        F: Fn(&mut Cursive, &str) + 'static,
    {
        self.with(|v| v.set_on_submit(callback))
    }

    pub fn set_content<S: Into<String>>(&mut self, content: S) {
        let content = content.into();
        let len = content.len();

        self.content = Rc::new(content);
        self.set_cursor(len);
    }

    /// Sets the cursor position.
    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    fn autocomplete_popup(&mut self, choices: Vec<String>) -> EventResult {
        let offset = XY::new(3, 4);
        let mut tree = MenuTree::new();
        for item in choices.iter() {
            let to_add = item.clone();
            tree.add_leaf(item.clone(), move |s| {
                let mut content = to_add.clone();
                content.push(' ');
                s.call_on_name("cli_input", |view: &mut CliView| {
                    view.set_content(content);
                });
            });
        }

        let tree = Rc::new(tree);

        EventResult::with_cb(move |s| {
            let tree = Rc::clone(&tree);
            s.screen_mut().add_layer_at(
                Position::absolute(offset),
                OnEventView::new(
                    MenuPopup::new(tree)
                )
                    .on_event_inner(EventTrigger::any(), |s, event| {
                        log::debug!("Event: {:?}", event);
                        match event {
                            Event::Char(ch) => {
                                let ch_toadd = ch.clone();
                                Some(EventResult::with_cb(move |s| {
                                    s.call_on_name("cli_input", |view: &mut CliView| {
                                        view.insert(ch_toadd);
                                    });
                                }))
                            },
                            _ => None
                        }
                    })
            );
        })
    }
}


impl View for CliView {

    fn draw(&self, printer: &Printer) {
        let width = self.content.width();
        printer.with_color(ColorStyle::primary(), |printer| {
            printer.with_effect(Effect::Reverse, |printer| {
                printer.print((0, 0), &self.content);
                let filler_len = (printer.size.x - width) / self.filler.width();
                printer.print_hline((width, 0), filler_len, self.filler.as_str());
            });
        });
        // Now print cursor
        printer.with_color(ColorStyle::highlight(), |printer| {
            if printer.focused {
                let c: &str = if self.cursor == self.content.len() {
                    &self.filler
                } else {
                    self.content[self.cursor..]
                        .graphemes(true)
                        .next()
                        .unwrap_or_else(|| {
                            panic!(
                                "Found no char at cursor {} in {}",
                                self.cursor, &self.content
                            )
                        })
                };
                let offset = self.content[0..self.cursor].width();
                printer.print((offset, 0), c);
            }
        });
    }

    fn take_focus(&mut self, _source: Direction) -> bool {
        log::debug!("Should focus?");
        true
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char(ch) => EventResult::Consumed(Some(self.insert(ch))),
            Event::Key(Key::Backspace) if self.cursor > 0 => self.backspace(),
            Event::CtrlChar('h') if self.cursor > 0 => self.backspace(),
            Event::Key(Key::Enter) => {
                let cb = self.on_submit.clone().unwrap();
                let content = Rc::clone(&self.content);
                EventResult::with_cb(move |s| {
                    cb(s, &content);
                })
            }
            Event::Key(Key::Tab) => {
                log::debug!("Trigger autocompletion");
                let completion = autocomplete::autocomplete(&self.content);
                if completion.len() > 0 {
                    self.autocomplete_popup(completion)
                } else {
                    EventResult::Consumed(None)
                }
            }
            _ => {
                log::debug!("Got unknown event {:?}", event);
                EventResult::Ignored
            }
        }
    }
}
