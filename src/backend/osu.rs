use std::io;
use rosu_v2::{Osu, error::OsuError, prelude::GameMode};

pub struct SearchConfig {
    game_mode: GameMode,
    mapper: Option<String>
}

impl SearchConfig {
    pub fn new() -> Self {
        Self { game_mode: GameMode::Mania, mapper: None }
    }

    #[inline]
    pub const fn game_mode(mut self, game_mode: GameMode) -> Self {
        self.game_mode = game_mode;
        self 
    }

    #[inline]
    pub fn mapper(mut self, mapper: String) -> Self {
        self.mapper = Some(mapper);
        self 
    }
}

pub async fn login() -> Result<Osu, OsuError> {
    // Login with secret id and password
    println!("Signing in...");
    let client_id: u64 = 47208;
    let client_secret = String::from("D400j2fmT5xWN55uuj51r4EGgTnweSZLItPJhvgu");

    Osu::new(client_id, client_secret).await
}

pub async fn search_maps(osu: &Osu, config: SearchConfig) -> Vec<u32>{
    let game_mode = config.game_mode;
    let mapper = config.mapper.unwrap();

     //Search Maps
    let mut found_maps = osu.beatmapset_search()
        .nsfw(false)
        .status(None)
        .mode(game_mode)
        .query(format!("mapper={}", mapper))
        .await
        .unwrap();
    let mut all_mapset_ids : Vec<u32>= Vec::new();
    loop {
        let mut mapset_lst: Vec<u32> = found_maps.mapsets.iter()
                        .filter(|map| {
                            let found = map.creator_name.to_lowercase();
                            let query = mapper.to_lowercase();
                            found.contains(query.as_str())
                        })  // make sure creator_name is equal to mapper
                        .map(|map| {
                            // println!("Debug: Find map name {}", map.title);
                            map.mapset_id
                        })
                        .collect();
        all_mapset_ids.append(&mut mapset_lst);
        if found_maps.has_more() {
            found_maps = found_maps.get_next(&osu).await.unwrap().unwrap();
        } else {
            println!("Collected {} beatmaps by this mapper.", all_mapset_ids.len());
            break;
        }
    }

    all_mapset_ids
}

pub fn get_game_mode() -> GameMode {
    // Get Game mode
    let game_mode;
    loop {
        println!("Choose mode:\n1. Mania");
        println!("Enter mode number:");
        let mut input_mode = String::new();
        io::stdin()
            .read_line(&mut input_mode)
            .unwrap();
        if let Ok(m) = input_mode.trim().parse::<u8>() {
            match m {
                1 => {
                    game_mode = GameMode::Mania;
                    break;
                },
                _ => {
                    println!("Unsupported game mode! Please try again.");
                }
            }
        } else {
            println!("Please enter a number!");
        }
    }

    game_mode
}

pub fn get_mapper() -> String {
    // Get Mapper
    println!("Enter mapper name:");
    let mut mapper = String::new();
    io::stdin()
        .read_line(&mut mapper)
        .unwrap();
    mapper = mapper.trim().to_string();
    
    mapper
}