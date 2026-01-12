mod backend;
mod frontend;

fn main() -> iced::Result {
    frontend::gui::run()
}