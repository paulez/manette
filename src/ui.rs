pub mod update {

    use cursive::{Cursive, traits::{Nameable, Scrollable}, views::TextView};
    use cursive::views::{LinearLayout, SelectView};

    use crate::command::run::CommandResult;

    pub fn file_list_view(s: &mut Cursive, file_list: Vec<String>) {
        s.call_on_name("command_layout", |layout: &mut LinearLayout| {
            clear_output_layers(layout);
            let mut select = SelectView::new();
            select.add_all_str(file_list);
            layout.add_child(select.scrollable().with_name("filelist_view"))
        });
    }

    pub fn command_output(s: &mut Cursive, result: CommandResult) {
        s.call_on_name("command_layout", |layout: &mut LinearLayout| {
            clear_output_layers(layout);
            layout.add_child(TextView::new(result.output).with_name("command_output"));
            layout.add_child(TextView::new(result.error_output).with_name("command_error"));
        });
    }

    pub fn show_error(s: &mut Cursive, error: String) {
        s.call_on_name("command_layout", |layout: &mut LinearLayout| {
            clear_output_layers(layout);
            layout.add_child(TextView::new(error).with_name("command_error"));
        });
    }

    fn clear_output_layers(layout: &mut LinearLayout) {
        for i in 0..layout.len() + 1 {
            log::debug!("Removing child {}", i);
            layout.remove_child(i);
        }
    }
}
