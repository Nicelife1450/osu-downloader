use iced::{Element, Length};
use iced::widget::{button, pick_list, text, text_input};

use super::{GameModeOption, Message};

pub fn mapper_input<'a>(value: &'a str) -> Element<'a, Message> {
    text_input("Enter mapper name", value)
        .on_input(Message::MapperInputChanged)
        .padding(10)
        .width(Length::Fill)
        .into()
}

pub fn custom_query_input<'a>(value: &'a str) -> Element<'a, Message> {
    text_input("Enter custom query (e.g. key=7 status=r)", value)
        .on_input(Message::CustomQueryChanged)
        .padding(10)
        .width(Length::Fill)
        .into()
}

pub fn game_mode_pick(selected: Option<GameModeOption>) -> Element<'static, Message> {
    pick_list(GameModeOption::ALL, selected, Message::GameModeSelected)
        .width(Length::Fill)
        .padding(10)
        .into()
}

pub fn download_button(is_downloading: bool) -> Element<'static, Message> {
    let label = if is_downloading {
        "Downloading..."
    } else {
        "Start download"
    };

    button(
        text(label)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .width(Length::Fill),
    )
    .on_press(Message::StartDownload)
    .padding(10)
    .width(Length::Fill)
    .into()
}

pub fn status_text<'a>(status: &'a str) -> Element<'a, Message> {
    text(status)
        .size(16)
        .width(Length::Fill)
        .into()
}
