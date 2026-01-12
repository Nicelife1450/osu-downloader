use std::path::{Path, PathBuf};

use std::env;

/// 搜索本机上的 osu 可执行程序，返回所在的目录
///
/// 优先使用环境变量 `OSU_PATH`，否则按常见安装目录进行枚举。
/// 未找到时返回 `None`。
pub fn find_game_dir() -> Option<PathBuf> {
    let exe_names = ["osu!.exe", "osu.exe"];

    // 如果用户显式配置了路径，先尝试该路径
    if let Ok(custom) = env::var("OSU_PATH") {
        let candidate = PathBuf::from(custom);
        // 如果直接给到 exe，返回其父目录
        if candidate.is_file() {
            if let Some(parent) = candidate.parent() {
                return Some(parent.to_path_buf());
            }
        }
        // 如果给到目录，检查是否包含可执行文件
        if candidate.is_dir() {
            for exe in exe_names {
                if candidate.join(exe).is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    // 收集可能的安装目录
    let mut search_roots: Vec<PathBuf> = Vec::new();

    if let Ok(current) = env::current_dir() {
        search_roots.push(current);
    }

    if let Ok(home) = env::var("HOME") {
        let home = PathBuf::from(home);
        // Wine / osu!lazer 等常见位置
        search_roots.push(home.join(".local/share/osu"));
        search_roots.push(home.join(".local/share/osu-wine"));
        search_roots.push(home.join("AppData/Local/osu")); // Windows 子目录（在 Wine 下亦常见）
    }

    for key in ["LOCALAPPDATA", "PROGRAMFILES", "PROGRAMFILES(X86)"] {
        if let Ok(dir) = env::var(key) {
            search_roots.push(PathBuf::from(dir).join("osu"));
        }
    }

    for drive in ['C', 'D', 'E', 'F'] {
        search_roots.push(PathBuf::from(format!("{drive}:\\osu")));
        search_roots.push(PathBuf::from(format!("{drive}:\\Games\\osu")));
    }

    for root in search_roots {
        for exe in exe_names {
            let candidate = if root.is_file() {
                // root 已是文件
                root.clone()
            } else {
                root.join(exe)
            };

            if candidate.is_file() {
                // 返回包含 exe 的目录
                if let Some(parent) = candidate.parent() {
                    return Some(parent.to_path_buf());
                }
            }
        }
    }

    None
}

// 从响应头或URL中提取文件名
pub fn extract_filename(res: &reqwest::Response, url: &str, map_id: u32) -> String {
    let mut filename = String::new();
    
    // 首先尝试从 Content-Disposition 头中获取文件名
    if let Some(content_disposition) = res.headers().get("content-disposition") {
        if let Ok(disposition_str) = content_disposition.to_str() {
            // 查找 filename= 或 filename*= 参数
            for part in disposition_str.split(';') {
                let part = part.trim();
                if part.starts_with("filename*=UTF-8''") {
                    // 处理 RFC 5987 格式: filename*=UTF-8''encoded-name
                    if let Some(encoded) = part.strip_prefix("filename*=UTF-8''") {
                        filename = percent_decode(encoded);
                        if !filename.is_empty() {
                            break;
                        }
                    }
                } else if part.starts_with("filename*=") {
                    // 处理其他 filename* 格式
                    if let Some(encoded) = part.strip_prefix("filename*=") {
                        if let Some(utf8_part) = encoded.strip_prefix("UTF-8''") {
                            filename = percent_decode(utf8_part);
                            if !filename.is_empty() {
                                break;
                            }
                        }
                    }
                } else if part.starts_with("filename=") {
                    // 处理标准 filename= 格式
                    let raw_filename = part.strip_prefix("filename=").unwrap_or("").trim();
                    // 移除可能的引号
                    let raw_filename = raw_filename.trim_matches('"').trim_matches('\'');
                    if !raw_filename.is_empty() {
                        filename = percent_decode(raw_filename);
                        break;
                    }
                }
            }
        }
    }
    
    // 如果响应头中没有，尝试从 URL 路径中提取
    if filename.is_empty() {
        if let Ok(url_obj) = reqwest::Url::parse(url) {
            if let Some(segments) = url_obj.path_segments() {
                for segment in segments.rev() {
                    if !segment.is_empty() && segment != "/" {
                        // 检查是否有文件扩展名
                        if Path::new(segment).extension().is_some() {
                            filename = percent_decode(segment);
                            break;
                        }
                    }
                }
            }
        }
    }
    
    // 如果 URL 解析失败或路径中没有文件名，尝试直接从原始 URL 字符串中提取
    if filename.is_empty() {
        if let Some(last_slash) = url.rfind('/') {
            if let Some(query_start) = url[last_slash + 1..].find('?') {
                let potential_filename = &url[last_slash + 1..last_slash + 1 + query_start];
                if !potential_filename.is_empty() && Path::new(potential_filename).extension().is_some() {
                    filename = percent_decode(potential_filename);
                }
            } else {
                let potential_filename = &url[last_slash + 1..];
                if !potential_filename.is_empty() && Path::new(potential_filename).extension().is_some() {
                    filename = percent_decode(potential_filename);
                }
            }
        }
    }
    
    // 如果都没有，使用默认名称
    if filename.is_empty() {
        filename = format!("{}.osz", map_id);
    }
    
    // 最后统一进行一次 URL 解码，确保没有残留的编码
    // 循环解码直到没有更多的 %XX 编码（最多解码3次，避免无限循环）
    let mut decoded = filename;
    for _ in 0..3 {
        if decoded.contains('%') {
            let new_decoded = percent_decode(&decoded);
            if new_decoded == decoded {
                // 如果解码后没有变化，说明已经没有可解码的内容了
                break;
            }
            decoded = new_decoded;
        } else {
            break;
        }
    }
    
    decoded
}

// 百分号解码函数（URL 解码）
fn percent_decode(input: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = input.chars();
    
    while let Some(ch) = chars.next() {
        if ch == '%' {
            // 尝试读取两个十六进制字符
            if let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
                // 检查是否是有效的十六进制字符
                if c1.is_ascii_hexdigit() && c2.is_ascii_hexdigit() {
                    let hex = format!("{}{}", c1, c2);
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        bytes.push(byte);
                        continue;
                    }
                }
                // 如果解码失败，保留原始字符（% + 两个字符）
                bytes.push(b'%');
                let mut buf = [0; 4];
                bytes.extend_from_slice(c1.encode_utf8(&mut buf).as_bytes());
                bytes.extend_from_slice(c2.encode_utf8(&mut buf).as_bytes());
            } else {
                // 如果只有一个字符或没有字符，保留 %
                bytes.push(b'%');
            }
        } else {
            // 对于非 % 字符，需要先转换为 UTF-8 字节
            let mut buf = [0; 4];
            let encoded = ch.encode_utf8(&mut buf);
            bytes.extend_from_slice(encoded.as_bytes());
        }
    }
    
    // 将字节序列转换为 UTF-8 字符串
    String::from_utf8_lossy(&bytes).to_string()
}


#[cfg(test)]
mod test {
    use crate::backend::download::utils::*;

    #[test]
    pub fn test_find_game_dir() {
        let game_dir = find_game_dir();
        if let Some(dir) = game_dir {
            println!("Find osu.exe in {}", dir.to_str().unwrap());
        } else {
            println!("Can't find osu.exe.");
        }
    }
}