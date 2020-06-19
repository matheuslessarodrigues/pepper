use std::io::stdout;

mod buffer;
mod buffer_position;
mod buffer_view;
mod config;
mod editor;
mod event;
mod history;
mod mode;
mod theme;
mod tui;

fn main() {
    ctrlc::set_handler(|| {}).unwrap();

    let stdout = stdout();
    let stdout = stdout.lock();

    let mut editor = editor::Editor::default();
    let handle = editor.buffers.add(buffer::Buffer::with_contents(
        buffer::BufferContents::from_str(include_str!("main.rs")),
    ));
    editor
        .buffer_views
        .push(buffer_view::BufferView::with_handle(handle));

    tui::show(stdout, editor).unwrap();
}
