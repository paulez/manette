/*

Copyright or © or Copr. Paul Ezvan (2022)

paul@ezvan.fr

This software is a computer program whose purpose is to provide a terminal file explorer.

This software is governed by the CeCILL license under French law and
abiding by the rules of distribution of free software.  You can  use,
modify and/ or redistribute the software under the terms of the CeCILL
license as circulated by CEA, CNRS and INRIA at the following URL
"http://www.cecill.info".

As a counterpart to the access to the source code and  rights to copy,
modify and redistribute granted by the license, users are provided only
with a limited warranty  and the software's author,  the holder of the
economic rights,  and the successive licensors  have only  limited
liability.

In this respect, the user's attention is drawn to the risks associated
with loading,  using,  modifying and/or developing or reproducing the
software by the user in light of its specific status of free software,
that may mean  that it is complicated to manipulate,  and  that  also
therefore means  that it is reserved for developers  and  experienced
professionals having in-depth computer knowledge. Users are therefore
encouraged to load and test the software's suitability as regards their
requirements in conditions enabling the security of their systems and/or
data to be ensured and,  more generally, to use and operate it in the
same conditions as regards security.

The fact that you are presently reading this means that you have had
knowledge of the CeCILL license and that you accept its terms.

*/

use crate::autocomplete::{autocomplete, CompletionChoice};
use crate::view::CliView;
use cursive::align::Align;
use cursive::event::{Callback, Event, EventResult, Key, MouseButton, MouseEvent};
use cursive::impl_scroller;
use cursive::view::scroll;
use cursive::view::View;
use cursive::Printer;
use cursive::Rect;
use cursive::Vec2;
use cursive::With;
use std::rc::Rc;

pub struct AutocompletePopup {
    input: Rc<String>,
    choices: Rc<Vec<CompletionChoice>>,
    focus: usize,
    scroll_core: scroll::Core,
    align: Align,
    on_dismiss: Option<Callback>,
    on_action: Option<Callback>,
}

impl_scroller!(AutocompletePopup::scroll_core);

impl AutocompletePopup {
    pub fn new(input: Rc<String>, choices: Rc<Vec<CompletionChoice>>) -> Self {
        AutocompletePopup {
            input,
            choices,
            focus: 0,
            scroll_core: scroll::Core::new(),
            align: Align::top_left(),
            on_dismiss: None,
            on_action: None,
        }
    }

    fn push(&mut self, ch: char) -> EventResult {
        Rc::make_mut(&mut self.input).push(ch);
        let choices = autocomplete::autocomplete(&self.input);
        match choices {
            Ok(choices) => {
                if choices.len() > 0 {
                    let focused_item = &self.choices[self.focus];
                    let new_focus = match choices.iter().position(|r| r == focused_item) {
                        Some(focus) => focus,
                        None => 0,
                    };
                    self.focus = new_focus;
                    self.choices = Rc::new(choices);
                    return EventResult::with_cb(move |s| {
                        s.call_on_name("cli_input", |view: &mut CliView| {
                            log::debug!("Popup callback");
                            view.insert(ch);
                        });
                    });
                } else {
                    self.dismiss()
                }
            }
            Err(error) => {
                log::error!("Cannot autocomplete: {:?}", error);
                self.dismiss()
            }
        }
    }

    fn scroll_up(&mut self, mut n: usize, cycle: bool) {
        while n > 0 {
            if self.focus > 0 {
                self.focus -= 1;
            } else if cycle {
                self.focus = self.choices.len() - 1;
            } else {
                break;
            }
            n -= 1;
        }
    }

    fn scroll_down(&mut self, mut n: usize, cycle: bool) {
        while n > 0 {
            if self.focus + 1 < self.choices.len() {
                self.focus += 1;
            } else if cycle {
                self.focus = 0;
            } else {
                // Stop if we're at the bottom.
                break;
            }

            n -= 1;
        }
    }

    fn submit(&mut self) -> EventResult {
        let action_cb = self.on_action.clone();
        let completion = self.choices[self.focus].completion.clone();
        EventResult::with_cb(move |s| {
            // Remove ourselves from the face of the earth
            s.pop_layer();
            // If we had prior orders, do it now.
            if let Some(ref action_cb) = action_cb {
                action_cb.clone()(s);
            }
            let content = completion.clone();
            s.call_on_name("cli_input", |view: &mut CliView| {
                view.set_content(content);
            });
        })
    }

    fn dismiss(&mut self) -> EventResult {
        let dismiss_cb = self.on_dismiss.clone();
        EventResult::with_cb(move |s| {
            if let Some(ref cb) = dismiss_cb {
                cb.clone()(s);
            }
            s.pop_layer();
        })
    }

    fn inner_on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Key(Key::Up) => self.scroll_up(1, true),
            Event::Key(Key::PageUp) => self.scroll_up(5, false),
            Event::Key(Key::Down) => self.scroll_down(1, true),
            Event::Key(Key::PageDown) => self.scroll_down(5, false),

            Event::Key(Key::Home) => self.focus = 0,
            Event::Key(Key::End) => self.focus = self.choices.len().saturating_sub(1),

            Event::Key(Key::Tab) => {
                return self.submit();
            }
            Event::Mouse {
                event: MouseEvent::Press(_),
                position,
                offset,
            } => {
                // eprintln!("Position: {:?} / {:?}", position, offset);
                if let Some(position) = position.checked_sub(offset) {
                    // Now `position` is relative to the top-left of the content.
                    let focus = position.y;
                    if focus < self.choices.len() {
                        self.focus = focus;
                    }
                }
            }
            Event::Mouse {
                event: MouseEvent::Release(MouseButton::Left),
                position,
                offset,
            } if position
                .checked_sub(offset)
                .map(|position| position.y == self.focus)
                .unwrap_or(false) =>
            {
                return self.submit();
            }
            Event::Key(Key::Esc) => {
                return self.dismiss();
            }
            Event::Char(ch) => {
                return self.push(ch);
            }
            _ => return EventResult::Ignored,
        }

        EventResult::Consumed(None)
    }

    fn inner_required_size(&mut self, _req: Vec2) -> Vec2 {
        let w = 2 + self
            .choices
            .iter()
            .map(|x| x.label.len())
            .max()
            .unwrap_or(1);

        let h = self.choices.len();

        Vec2::new(w, h)
    }

    fn inner_important_area(&self, size: Vec2) -> Rect {
        if self.choices.is_empty() {
            return Rect::from((0, 0));
        }

        Rect::from_size((0, self.focus), (size.x, 1))
    }
}

impl View for AutocompletePopup {
    fn draw(&self, printer: &Printer) {
        if !printer.size.fits((2, 2)) {
            return;
        }

        let h = self.choices.len();
        // If we're too high, add a vertical offset
        let offset = self.align.v.get_offset(h, printer.size.y);
        let printer = &printer.offset((0, offset));

        // Start with a box
        scroll::draw_box_frame(self, &printer, |_s, _y| false, |_s, _x| false);

        // We're giving it a reduced size because of borders.
        let printer = printer.shrinked_centered((2, 2));

        scroll::draw_lines(self, &printer, |s, printer, i| {
            printer.with_selection(i == s.focus, |printer| {
                let item = &s.choices[i];
                printer.print_hline((0, 0), printer.size.x, " ");
                printer.print((1, 0), &item.label);
            });
        });
    }
    fn required_size(&mut self, req: Vec2) -> Vec2 {
        // We can't really shrink our items here, so it's not flexible.

        // 2 is the padding

        scroll::required_size(
            self,
            req.saturating_sub((2, 2)),
            true,
            Self::inner_required_size,
        ) + (2, 2)
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match scroll::on_event(
            self,
            event.relativized((1, 1)),
            Self::inner_on_event,
            Self::inner_important_area,
        ) {
            EventResult::Ignored => {
                // Check back the non-relativized event now
                if let Event::Mouse {
                    event: MouseEvent::Press(_),
                    position,
                    offset,
                } = event
                {
                    // Mouse press will be ignored if they are outside of the content.
                    // They can be on the border, or entirely outside of the popup.

                    // Mouse clicks outside of the popup should dismiss it.
                    if !position.fits_in_rect(offset, self.scroll_core.last_outer_size() + (2, 2)) {
                        let dismiss_cb = self.on_dismiss.clone();
                        return EventResult::with_cb(move |s| {
                            if let Some(ref cb) = dismiss_cb {
                                cb.clone()(s);
                            }
                            s.pop_layer();
                        });
                    }
                }

                EventResult::Ignored
            }
            other => other,
        }
    }

    fn layout(&mut self, size: Vec2) {
        scroll::layout(
            self,
            size.saturating_sub((2, 2)),
            true,
            |_s, _size| (),
            Self::inner_required_size,
        );
    }

    fn important_area(&self, size: Vec2) -> Rect {
        scroll::important_area(
            self,
            size.saturating_sub((2, 2)),
            Self::inner_important_area,
        )
        .with(|area| area.offset((1, 1)))
    }
}
