use std::{cmp::min, error::Error, fs::File, io::Write};
use threadpool::ThreadPool;
use futures_util::{FutureExt, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use tokio::runtime::Runtime;

// const SOURCE: &str = "https://catboy.best/d/{id}";
const SOURCE: &str = "https://txy1.sayobot.cn/beatmaps/download/mini/{id}?server=auto";


pub fn download_map_req(map_id_lst: Vec<u32>) {
    let pool = ThreadPool::new(5);
    for map_id in map_id_lst {
        pool.execute(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(download_one(map_id));
        });
    }
}

async fn download_one(map_id: u32) -> Result<(), Box<dyn Error>> {
    let url = "https://releases.ubuntu.com/22.04.3/ubuntu-22.04.3-desktop-amd64.iso"; // 示例大文件
    let path = "ubuntu.iso";
    
    let client = Client::new();
    let res = client.get(url).send().await?;
    
    // 获取文件总长度
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", url))?;

    // --- 指示器设置 ---
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({percentage}%)")?
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", url));

    // --- 下载与文件写入 ---
    let mut file = File::create(path)?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes().into_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("Downloaded {} to {}", url, path));
    Ok(())
}