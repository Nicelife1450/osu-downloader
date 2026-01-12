use rosu_v2::{Osu, error::OsuError, prelude::GameMode};

pub struct SearchConfig {
    game_mode: GameMode,
    mapper: Option<String>,
    keys: Option<u8>
}

impl SearchConfig {
    pub fn new() -> Self {
        Self { game_mode: GameMode::Mania, mapper: None, keys: None}
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
    pub const fn keys(mut self, keys: u8) -> Self {
        self.keys = Some(keys);
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
    let mut query = String::from(format!("mapper={} ", mapper));
    if let Some(keys) = config.keys {
        query.push_str(format!("key={} ", keys).as_str());
    }

     //Search Maps
    let mut found_maps = osu.beatmapset_search()
        .nsfw(false)
        .status(None)
        .mode(game_mode)
        .query(query)
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
