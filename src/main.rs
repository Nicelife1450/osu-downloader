use std::io;
use rosu_v2::{Osu, prelude::GameMode};

use crate::download::download_map;

mod download;

fn main() {
    trpl::block_on(run());
}

async fn run() {
    // Login with secret id and password
    println!("账户登录中……");
    let client_id: u64 = 47208;
    let client_secret = String::from("D400j2fmT5xWN55uuj51r4EGgTnweSZLItPJhvgu");
    let osu = Osu::new(client_id, client_secret).await.expect("请输入正确的账户id和密钥！");

    // Get Game mode
    let game_mode;
    loop {
        println!("选择模式:\n1. Mania");
        println!("输入模式:");
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
                    println!("不支持的游戏类型！请重新输入。");
                }
            }
        } else {
            println!("请输入数字！");
        }
    }

    // Get Mapper
    println!("输入谱师名:");
    let mut mapper = String::new();
    io::stdin()
        .read_line(&mut mapper)
        .unwrap();
    mapper = mapper.trim().to_string();
    
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
                            found == query
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
            println!("已收集该作者的 {} 张谱面.", all_mapset_ids.len());
            break;
        }
    }

    match download_map(all_mapset_ids).await {
        Ok(_) => {
            println!("下载完成.");
        },
        Err(e) => {
            println!("下载失败: {e}");
        }
    }

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        // Because of the async nature of the downloader, we need to keep the main thread alive
    }
}