/*

Copyright or © or Copr. Paul Ezvan (2021)

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

pub mod update {

    use cursive::event::EventResult;
    use cursive::event::Event;
    use cursive::{
        theme::{BaseColor, Color},
        utils::markup::StyledString,
        views::{EditView, LinearLayout, OnEventView, SelectView},
    };
    use cursive::{
        traits::{Nameable, Scrollable},
        views::{ResizedView, ScrollView, TextView},
        Cursive,
    };

    use crate::command::run::CommandResult;
    use crate::command::run::{self, FileEntry};
    use crate::file::filetype::FileType;

    pub fn file_list_view(s: &mut Cursive, file_list: Vec<FileEntry>) {
        s.call_on_name("command_layout", |layout: &mut LinearLayout| {
            clear_output_layers(layout);
            let mut select = SelectView::new();
            select.add_all(
                file_list
                    .into_iter()
                    .map(|file_entry| match file_entry.filetype {
                        FileType::Directory => (
                            StyledString::styled(
                                file_entry.filename.clone(),
                                Color::Light(BaseColor::Blue),
                            ),
                            file_entry.filename,
                        ),
                        FileType::Executable => (
                            StyledString::styled(
                                file_entry.filename.clone(),
                                Color::Light(BaseColor::Green),
                            ),
                            file_entry.filename,
                        ),
                        FileType::Symlink => (
                            StyledString::styled(
                                file_entry.filename.clone(),
                                Color::Dark(BaseColor::Blue),
                            ),
                            file_entry.filename,
                        ),
                        _ => (
                            StyledString::plain(file_entry.filename.clone()),
                            file_entry.filename,
                        ),
                    }),
            );
            select.set_on_submit(|s, selection: &String| {
                log::debug!("File list: {:?} selected", selection);
                run::submit_file(s, selection);
            });

            let on_event = OnEventView::new(select)
                .on_event_inner('e', |sel: &mut SelectView, e: &Event| {
                    log::debug!("Pressed e");
                    let selection = sel.selection();
                    match selection {
                        Some(selection) => {
                            Some(EventResult::with_cb(move |s| {
                                run::edit_file(s, &selection);
                            }))
                        }
                        None => None,
                    }
                });

            layout.add_child(ResizedView::with_full_screen(
                on_event.scrollable().with_name("filelist_view"),
            ));
        });
    }

    pub fn command_output(s: &mut Cursive, result: CommandResult) {
        s.call_on_name("command_layout", |layout: &mut LinearLayout| {
            clear_output_layers(layout);
            layout.add_child(ResizedView::with_full_screen(ScrollView::new(
                TextView::new(result.output).with_name("command_output"),
            )));
            layout.add_child(TextView::new(result.error_output).with_name("command_error"));
        });
    }

    pub fn clear_command(s: &mut Cursive) {
        s.call_on_name("command_input", |view: &mut EditView| {
            view.set_content("");
        });
    }

    pub fn show_error(s: &mut Cursive, error: String) {
        s.call_on_name("command_layout", |layout: &mut LinearLayout| {
            clear_output_layers(layout);
            layout.add_child(ResizedView::with_full_screen(
                TextView::new(error).with_name("command_error"),
            ));
        });
    }

    fn clear_output_layers(layout: &mut LinearLayout) {
        let children_names = ["command_output", "command_error", "filelist_view"];
        for child_name in &children_names {
            match layout.find_child_from_name(child_name) {
                Some(child_index) => {
                    layout.remove_child(child_index);
                }
                None => {
                    log::error!("Cannot find {} child", child_name);
                }
            }
        }
    }
}
