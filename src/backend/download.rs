use std::{cmp::min, error::Error, path::PathBuf, sync::Arc};
use futures_util::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use tokio::{fs, io::AsyncWriteExt};
use utils::extract_filename;
use tokio::sync::Semaphore;

use crate::backend::download::utils::find_game_dir;

mod utils;

// const SOURCE: &str = "https://catboy.best/d/{id}";
const SOURCE: &str = "https://txy1.sayobot.cn/beatmaps/download/mini/{id}?server=auto";
const DOWNLOAD_DIR: &str = "./songs";

/// 并发下载多个地图文件
/// 
/// 使用 Tokio 的并发机制，通过 `tokio::spawn` 创建并发任务
/// 使用 `futures::future::join_all` 等待所有任务完成
/// 
/// # 参数
/// - `map_id_lst`: 要下载的地图 ID 列表
/// - `concurrent_limit`: 最大并发数（默认 5）
pub async fn download_maps(map_id_lst: Vec<u32>) -> Result<(), Box<dyn Error + Send>> {
    fs::create_dir_all(DOWNLOAD_DIR).await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

    let map_id_lst = remove_duplicates(map_id_lst);
    let concurrent_limit = 3;
    let multi = Arc::new(MultiProgress::new());
    let semaphore = Arc::new(Semaphore::new(concurrent_limit));
    
    let tasks: Vec<_> = map_id_lst
        .into_iter()
        .map(|map_id| {
            let multi_clone = Arc::clone(&multi);
            let semaphore_clone = Arc::clone(&semaphore);
            
            tokio::spawn(async move {
                // 获取 semaphore 许可，控制并发数
                let _permit = semaphore_clone.acquire().await.unwrap();
                
                // 执行下载，错误会被记录但不会中断其他任务
                match download_one(map_id, multi_clone).await {
                    Ok(()) => Ok(map_id),
                    Err(e) => {
                        let error_msg = e.to_string();
                        eprintln!("Failed to download map {}: {}", map_id, error_msg);
                        Err(map_id)
                    }
                }
            })
        })
        .collect();
    
    let results = join_all(tasks).await;
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    for result in results {
        match result {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(map_id)) => {
                eprintln!("Task failed for map {}", map_id);
                fail_count += 1;
            }
            Err(e) => {
                eprintln!("Task panicked: {}", e);
                fail_count += 1;
            }
        }
    }
    
    if fail_count > 0 {
        eprintln!("Download completed: {} succeeded, {} failed", success_count, fail_count);
    }
    
    Ok(())
}

fn remove_duplicates(map_id_lst: Vec<u32>) -> Vec<u32>{
    if let Some(mut song_dir) = find_game_dir() {
        println!("Found songs directory of Osu. Removing duplicate maps...");
        song_dir.push("Songs");
        assert!(song_dir.is_dir());
        let exist_map_ids: Vec<u32> = std::fs::read_dir(song_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.unwrap();
                let exist_map_id: Result<u32, _> = entry.file_name()
                    .into_string()
                    .unwrap()
                    .split(' ')
                    .next()
                    .unwrap()
                    .parse();
                match exist_map_id {
                    Ok(id) => Some(id),
                    Err(_) => None
                }
            })
            .collect();
        let final_id_lst: Vec<u32> = map_id_lst.iter()
            .filter(|&id| !exist_map_ids.contains(id))
            .map(|id| *id)
            .collect();

        let remove_count = map_id_lst.len() - final_id_lst.len();
        println!("Removed {} duplicate maps.", remove_count.to_string());

        final_id_lst
    } else {
        map_id_lst
    }
}

async fn download_one(map_id: u32, multi: Arc<MultiProgress>) -> Result<(), Box<dyn Error + Send>> {
    let url = SOURCE.replace("{id}", map_id.to_string().as_str());
    let client = Client::new();
    let mut res = client.get(&url).send().await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
    
    // 提取文件名
    let filename = extract_filename(&res, &url, map_id);
    let mut path = PathBuf::from(DOWNLOAD_DIR);
    path.push(filename.as_str());
    
    // 获取文件总长度
    let total_size = res
        .content_length()
        .ok_or_else(|| format!("Failed to get content length from '{}'", url))
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)) as Box<dyn Error + Send>)?;

    // --- 指示器设置 ---
    let pb = multi.add(ProgressBar::new(total_size));
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({percentage}%)")
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())) as Box<dyn Error + Send>)?
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", filename));

    // --- 下载与文件写入 ---
    let mut file = fs::File::create(&path).await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
    let mut downloaded: u64 = 0;
    
    loop {
        match res.chunk().await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)? {
            Some(chunk) => {
                // 使用异步写入
                file.write_all(&chunk).await
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
                let new = min(downloaded + (chunk.len() as u64), total_size);
                downloaded = new;
                pb.set_position(new);
            }
            None => break,
        }
    }

    pb.finish_with_message(format!("Downloaded {}", filename));
    Ok(())
}