use std::{sync::Arc, time::Duration};

use iced::{
    widget::{column, container, text},
    window, Alignment, Application, Command, Element, Length, Settings, Size, Theme,
};
use rosu_v2::{prelude::GameMode, Osu};
use tokio::time::sleep;

use crate::backend::download::download_maps;
use crate::backend::osu::{login, search_maps, SearchConfig};

mod components;

#[derive(Clone)]
pub enum Message {
    MapperInputChanged(String),
    CustomQueryChanged(String),
    GameModeSelected(GameModeOption),
    StartDownload,
    DownloadComplete(Result<String, String>),
    LoginComplete(Result<Arc<Osu>, String>),
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MapperInputChanged(arg0) => {
                f.debug_tuple("MapperInputChanged").field(arg0).finish()
            }
            Self::CustomQueryChanged(arg0) => {
                f.debug_tuple("CustomQueryChanged").field(arg0).finish()
            }
            Self::GameModeSelected(arg0) => f.debug_tuple("GameModeSelected").field(arg0).finish(),
            Self::StartDownload => write!(f, "StartDownload"),
            Self::DownloadComplete(arg0) => f.debug_tuple("DownloadComplete").field(arg0).finish(),
            Self::LoginComplete(Ok(_)) => f.debug_tuple("LoginComplete").field(&"Ok(Osu)").finish(),
            Self::LoginComplete(Err(e)) => f.debug_tuple("LoginComplete").field(&e).finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameModeOption {
    Osu,
    Taiko,
    Catch,
    Mania,
}

impl GameModeOption {
    const ALL: &'static [GameModeOption] = &[
        GameModeOption::Osu,
        GameModeOption::Taiko,
        GameModeOption::Catch,
        GameModeOption::Mania,
    ];

    fn to_game_mode(self) -> GameMode {
        match self {
            GameModeOption::Mania => GameMode::Mania,
            GameModeOption::Osu => GameMode::Osu,
            GameModeOption::Taiko => GameMode::Taiko,
            GameModeOption::Catch => GameMode::Catch,
        }
    }
}

impl std::fmt::Display for GameModeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameModeOption::Osu => write!(f, "Osu"),
            GameModeOption::Taiko => write!(f, "Taiko"),
            GameModeOption::Catch => write!(f, "Catch"),
            GameModeOption::Mania => write!(f, "Mania"),
        }
    }
}

pub struct App {
    mapper_input: String,
    custom_query: String,
    selected_game_mode: Option<GameModeOption>,
    status_message: String,
    is_downloading: bool,
    osu: Option<Arc<Osu>>,
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
                custom_query: String::new(),
                selected_game_mode: Some(GameModeOption::Mania),
                status_message: String::from("Signing in"),
                is_downloading: false,
                osu: None,
            },
            Command::perform(
                async { login().await.map(Arc::new).map_err(|e| e.to_string()) },
                Message::LoginComplete,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("Osu! Beatmap Downloader")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::MapperInputChanged(input) => {
                self.mapper_input = input;
            }
            Message::CustomQueryChanged(query) => {
                self.custom_query = query;
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

                if self.osu.is_none() {
                    self.status_message = String::from("Error: Not signed in yet. Please wait.");
                    return Command::none();
                }

                if self.selected_game_mode.is_none() {
                    self.status_message = String::from("Error: Please select a game mode");
                    return Command::none();
                }

                self.is_downloading = true;
                self.status_message = String::from("Downloading...");

                let mapper = self.mapper_input.clone();
                let custom_query = self.custom_query.clone();
                let game_mode = self.selected_game_mode.unwrap().to_game_mode();
                let osu_clone = Arc::clone(&self.osu.as_ref().unwrap());

                return Command::perform(
                    async move { download_task(osu_clone, mapper, custom_query, game_mode).await },
                    |result| Message::DownloadComplete(result),
                );
            }
            Message::DownloadComplete(result) => {
                self.is_downloading = false;
                match result {
                    Ok(download_msg) => {
                        self.status_message =
                            String::from(format!("Download finished! {}", download_msg));
                    }
                    Err(e) => {
                        self.status_message = format!("Download failed: {}", e);
                    }
                }
            }
            Message::LoginComplete(result) => match result {
                Ok(osu) => {
                    self.osu = Some(osu);
                    self.status_message = String::from("Signed in successfully");
                }
                Err(e) => {
                    self.status_message = format!("Login failed: {}, retry in 3 seconds...", e);
                    return Command::perform(
                        async {
                            sleep(Duration::from_secs(3)).await;
                            login().await.map(Arc::new).map_err(|e| e.to_string())
                        },
                        Message::LoginComplete,
                    );
                }
            },
        }

        Command::none()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn view(&self) -> Element<'_, Message> {
        let mapper_input = components::mapper_input(&self.mapper_input);
        let custom_query_input = components::custom_query_input(&self.custom_query);
        let game_mode_pick = components::game_mode_pick(self.selected_game_mode);
        let download_button = components::download_button(self.is_downloading);
        let status_text = components::status_text(&self.status_message);
        let content = column![
            text("Osu! Beatmap Downloader").size(24),
            text("Mapper name:").size(14),
            mapper_input,
            text("Custom query (optional):").size(14),
            custom_query_input,
            text("Game mode:").size(14),
            game_mode_pick,
            download_button,
            status_text,
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

async fn download_task(
    osu: Arc<Osu>,
    mapper: String,
    custom_query: String,
    game_mode: GameMode,
) -> Result<String, String> {
    let search_config = SearchConfig::new()
        .game_mode(game_mode)
        .mapper(mapper.trim().to_string())
        .custom_query(custom_query.trim().to_string());

    let mapset_ids = search_maps(&osu, search_config).await;

    if mapset_ids.is_empty() {
        return Err(format!("No beatmaps found."));
    }

    let download_msg = download_maps(mapset_ids)
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    Ok(download_msg)
}

pub fn run() -> iced::Result {
    App::run(Settings {
        window: window::Settings {
            size: Size::new(480.0, 480.0),
            min_size: Some(Size::new(480.0, 480.0)),
            ..Default::default()
        },
        ..Default::default()
    })
}
