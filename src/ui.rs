pub mod update {

    use cursive::Cursive;
    use cursive::views::{LinearLayout, SelectView};
    use std::path::PathBuf;
    pub fn add_file_list_view(s: &mut Cursive, file_list: Vec<String>) {
        s.call_on_name("command_layout", |layout: &mut LinearLayout| {
            let mut select = SelectView::new();
            select.add_all_str(file_list);
            layout.add_child(select)
        });
    }
}
