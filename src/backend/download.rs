use std::{cmp::min, error::Error, path::PathBuf, sync::Arc};
use futures_util::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use tokio::{fs, io::AsyncWriteExt};
use utils::extract_filename;
use tokio::sync::Semaphore;

use crate::backend::download::utils::find_game_dir;

mod utils;

const SOURCE: &str = "https://txy1.sayobot.cn/beatmaps/download/mini/{id}?server=auto";
const DOWNLOAD_DIR: &str = "./Songs";

/// Download multiple map files concurrently
pub async fn download_maps(map_id_lst: Vec<u32>) -> Result<String, Box<dyn Error + Send>> {
    fs::create_dir_all(DOWNLOAD_DIR).await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

    let map_id_lst = remove_duplicates(map_id_lst);
    let concurrent_limit = 5;
    let multi = Arc::new(MultiProgress::new());
    let semaphore = Arc::new(Semaphore::new(concurrent_limit));
    
    let tasks: Vec<_> = map_id_lst
        .into_iter()
        .map(|map_id| {
            let multi_clone = Arc::clone(&multi);
            let semaphore_clone = Arc::clone(&semaphore);
            
            tokio::spawn(async move {
                // Acquire semaphore permit to control concurrency
                let _permit = semaphore_clone.acquire().await.unwrap();
                
                // Perform the download; errors are logged and won't interrupt other tasks
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
    
    Ok(format!("{} succeeded, {} failed.", success_count, fail_count))
}

fn remove_duplicates(map_id_lst: Vec<u32>) -> Vec<u32>{
    if let Some(mut song_dir) = find_game_dir() {
        song_dir.push("Songs");
        println!("Found songs directory of Osu in {}. Removing duplicate maps...", song_dir.to_str().unwrap());
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
                    Ok(id) => {
                        // println!("Found duplicate map {}. Removing...", id);
                        Some(id)
                    },
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
        println!("Can't find songs directory. Download all found maps.");
        map_id_lst
    }
}

async fn download_one(map_id: u32, multi: Arc<MultiProgress>) -> Result<(), Box<dyn Error + Send>> {
    let url = SOURCE.replace("{id}", map_id.to_string().as_str());
    let client = Client::new();
    let mut res = client.get(&url).send().await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
    
    // Extract the filename
    let filename = extract_filename(&res, &url, map_id);
    let mut path = PathBuf::from(DOWNLOAD_DIR);
    path.push(filename.as_str());
    
    // Get total content length
    let total_size = res
        .content_length()
        .ok_or_else(|| format!("Failed to get content length from '{}'", url))
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)) as Box<dyn Error + Send>)?;
    if total_size == 0 {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Map has content_length of 0.")) as Box<dyn Error + Send>);
    }

    // --- Progress indicator setup ---
    let pb = multi.add(ProgressBar::new(total_size));
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({percentage}%)")
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())) as Box<dyn Error + Send>)?
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", filename));

    // --- Download and file writing ---
    let mut file = fs::File::create(&path).await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
    let mut downloaded: u64 = 0;
    
    loop {
        match res.chunk().await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)? {
            Some(chunk) => {
                // Use asynchronous writes
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
