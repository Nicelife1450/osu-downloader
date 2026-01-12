use iced::{
    widget::{button, column, container, pick_list, text, text_input, Column, Scrollable},
    Alignment, Application, Command, Element, Length, Settings, Theme,
};
use rosu_v2::prelude::GameMode;

use crate::backend::download::download_maps;
use crate::backend::osu::{login, search_maps, SearchConfig};

#[derive(Debug, Clone)]
pub enum Message {
    MapperInputChanged(String),
    GameModeSelected(GameModeOption),
    StartDownload,
    DownloadProgress(String),
    DownloadComplete(Result<(), String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameModeOption {
    Mania,
}

impl GameModeOption {
    const ALL: &'static [GameModeOption] = &[GameModeOption::Mania];

    fn to_game_mode(self) -> GameMode {
        match self {
            GameModeOption::Mania => GameMode::Mania,
        }
    }
}

impl std::fmt::Display for GameModeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameModeOption::Mania => write!(f, "Mania"),
        }
    }
}

pub struct App {
    mapper_input: String,
    selected_game_mode: Option<GameModeOption>,
    status_message: String,
    is_downloading: bool,
    log_messages: Vec<String>,
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                mapper_input: String::new(),
                selected_game_mode: Some(GameModeOption::Mania),
                status_message: String::from("Ready"),
                is_downloading: false,
                log_messages: Vec::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("osu! Beatmap Downloader")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::MapperInputChanged(input) => {
                self.mapper_input = input;
            }
            Message::GameModeSelected(mode) => {
                self.selected_game_mode = Some(mode);
            }
            Message::StartDownload => {
                if self.is_downloading {
                    return Command::none();
                }

                if self.mapper_input.trim().is_empty() {
                    self.status_message = String::from("Error: Please enter a mapper name");
                    return Command::none();
                }

                if self.selected_game_mode.is_none() {
                    self.status_message = String::from("Error: Please select a game mode");
                    return Command::none();
                }

                self.is_downloading = true;
                self.status_message = String::from("Signing in...");
                self.log_messages.clear();
                self.add_log("Starting download workflow");

                let mapper = self.mapper_input.clone();
                let game_mode = self.selected_game_mode.unwrap().to_game_mode();

                return Command::perform(
                    async move { download_task(mapper, game_mode).await },
                    |result| Message::DownloadComplete(result),
                );
            }
            Message::DownloadProgress(msg) => {
                self.add_log(&msg);
            }
            Message::DownloadComplete(result) => {
                self.is_downloading = false;
                match result {
                    Ok(()) => {
                        self.status_message = String::from("Download finished!");
                        self.add_log("All download tasks completed");
                    }
                    Err(e) => {
                        self.status_message = format!("Download failed: {}", e);
                        self.add_log(&format!("Error: {}", e));
                    }
                }
            }
        }

        Command::none()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn view(&self) -> Element<'_, Message> {
        let mapper_input = text_input("Enter mapper name", &self.mapper_input)
            .on_input(Message::MapperInputChanged)
            .padding(10)
            .width(Length::Fill);

        let game_mode_pick = pick_list(
            GameModeOption::ALL,
            self.selected_game_mode,
            Message::GameModeSelected,
        )
        .width(Length::Fill)
        .padding(10);

        let download_button = button(if self.is_downloading {
            "Downloading..."
        } else {
            "Start download"
        })
        .on_press(Message::StartDownload)
        .padding(10)
        .width(Length::Fill);

        let status_text = text(&self.status_message).size(16).width(Length::Fill);

        let log_column: Column<Message> = Column::with_children(
            self.log_messages
                .iter()
                .map(|msg| text(msg).size(12).into())
                .collect::<Vec<_>>(),
        )
        .spacing(5)
        .width(Length::Fill);

        let scrollable_logs = Scrollable::new(log_column)
            .width(Length::Fill)
            .height(Length::Fill);

        let content = column![
            text("osu! Beatmap Downloader").size(24),
            text("Mapper name:").size(14),
            mapper_input,
            text("Game mode:").size(14),
            game_mode_pick,
            download_button,
            status_text,
            text("Logs:").size(14),
            scrollable_logs,
        ]
        .spacing(15)
        .padding(20)
        .align_items(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }
}

async fn download_task(mapper: String, game_mode: GameMode) -> Result<(), String> {
    let osu = login()
        .await
        .map_err(|e| format!("Sign-in failed: {}", e))?;

    let search_config = SearchConfig::new()
        .game_mode(game_mode)
        .mapper(mapper.clone());

    let mapset_ids = search_maps(&osu, search_config).await;

    if mapset_ids.is_empty() {
        return Err(format!("No beatmaps found for mapper '{}'", mapper));
    }

    let mapset_ids: Vec<u32> = mapset_ids.iter().take(10).copied().collect();

    download_maps(mapset_ids)
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    Ok(())
}

impl App {
    fn add_log(&mut self, message: &str) {
        self.log_messages.push(message.to_string());
        // 限制日志数量，避免内存占用过大
        if self.log_messages.len() > 100 {
            self.log_messages.remove(0);
        }
    }
}

pub fn run() -> iced::Result {
    App::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(480.0, 640.0),
            ..Default::default()
        },
        ..Default::default()
    })
}
