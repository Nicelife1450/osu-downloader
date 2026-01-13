use rosu_v2::{error::OsuError, prelude::GameMode, Osu};

pub struct SearchConfig {
    game_mode: GameMode,
    mapper: Option<String>,
    custom_query: Option<String>,
}

impl SearchConfig {
    pub fn new() -> Self {
        Self {
            game_mode: GameMode::Mania,
            mapper: None,
            custom_query: None,
        }
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

    #[inline]
    pub fn custom_query(mut self, custom_query: String) -> Self {
        self.custom_query = Some(custom_query);
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

pub async fn search_maps(osu: &Osu, config: SearchConfig) -> Vec<u32> {
    let game_mode = config.game_mode;
    let mut query = String::new();
    if let Some(mapper) = &config.mapper {
        query.push_str(format!("{} ", mapper).as_str());
    }
    if let Some(custom_query) = config.custom_query {
        query.push_str(custom_query.as_str());
    }

    println!("Searching beatmaps use query: {}", &query);
    //Search Maps
    let mut found_maps = osu
        .beatmapset_search()
        .nsfw(false)
        .status(None)
        .mode(game_mode)
        .query(query)
        .await
        .unwrap();
    let mut all_mapset_ids: Vec<u32> = Vec::new();
    let mut page = 1;
    loop {
        let mut mapset_lst: Vec<u32> = found_maps
            .mapsets
            .iter()
            .filter(|map| {
                if let Some(mapper) = &config.mapper {
                    let found = map.creator_name.to_lowercase();
                    let query = mapper.to_lowercase();
                    found.contains(query.as_str())
                } else {
                    true
                }
            }) // make sure creator_name is equal to mapper
            .map(|map| {
                // println!("Debug: Find map name {}", map.title);
                map.mapset_id
            })
            .collect();
        all_mapset_ids.append(&mut mapset_lst);
        if found_maps.has_more() {
            println!("Looking up page {}", page.to_string());
            found_maps = found_maps.get_next(&osu).await.unwrap().unwrap();
            page += 1;
        } else {
            println!(
                "Collected {} beatmaps by this mapper.",
                all_mapset_ids.len()
            );
            break;
        }
    }

    all_mapset_ids
}
