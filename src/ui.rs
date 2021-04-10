pub mod update {

    use cursive::{
        theme::{BaseColor, Color},
        utils::markup::StyledString,
        views::{LinearLayout, SelectView},
    };
    use cursive::{
        traits::{Nameable, Scrollable},
        views::{ResizedView, ScrollView, TextView},
        Cursive,
    };

    use crate::command::run::CommandResult;
    use crate::command::run::{self, FileEntry, FileType};

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
            layout.add_child(ResizedView::with_full_screen(
                select.scrollable().with_name("filelist_view"),
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
