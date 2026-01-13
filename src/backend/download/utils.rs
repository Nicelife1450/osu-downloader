use std::path::{Path, PathBuf};

use std::env;

/// Search for the local osu executable and return its directory
///
/// Prefer the `OSU_PATH` environment variable; otherwise enumerate common install locations.
/// Returns `None` if not found.
pub fn find_game_dir() -> Option<PathBuf> {
    let exe_names = ["osu!.exe", "osu.exe"];

    // If the user explicitly configured a path, try it first
    if let Ok(custom) = env::var("OSU_PATH") {
        let candidate = PathBuf::from(custom);
        if candidate.is_file() {
            if let Some(parent) = candidate.parent() {
                return Some(parent.to_path_buf());
            }
        }
        if candidate.is_dir() {
            for exe in exe_names {
                if candidate.join(exe).is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    // Collect possible installation directories
    let mut search_roots: Vec<PathBuf> = Vec::new();

    if let Ok(current) = env::current_dir() {
        search_roots.push(current);
    }

    if let Ok(home) = env::var("HOME") {
        let home = PathBuf::from(home);
        // Common locations for Wine / osu!lazer
        search_roots.push(home.join(".local/share/osu"));
        search_roots.push(home.join(".local/share/osu!"));
        search_roots.push(home.join(".local/share/osu-wine"));
        search_roots.push(home.join("AppData/Local/osu")); // Windows subdirectory (also common under Wine)
        search_roots.push(home.join("AppData/Local/osu!")); // Windows subdirectory (also common under Wine)
    }

    for key in ["LOCALAPPDATA", "PROGRAMFILES", "PROGRAMFILES(X86)"] {
        if let Ok(dir) = env::var(key) {
            search_roots.push(PathBuf::from(&dir).join("osu"));
            search_roots.push(PathBuf::from(&dir).join("osu!"));
        }
    }

    for drive in ['C', 'D', 'E', 'F'] {
        search_roots.push(PathBuf::from(format!("{drive}:\\osu")));
        search_roots.push(PathBuf::from(format!("{drive}:\\osu!")));
        search_roots.push(PathBuf::from(format!("{drive}:\\Games\\osu")));
        search_roots.push(PathBuf::from(format!("{drive}:\\Games\\osu!")));
    }

    for root in search_roots {
        for exe in exe_names {
            let candidate = if root.is_file() {
                // root is already a file
                root.clone()
            } else {
                root.join(exe)
            };

            if candidate.is_file() {
                // Return the directory that contains the exe
                if let Some(parent) = candidate.parent() {
                    return Some(parent.to_path_buf());
                }
            }
        }
    }

    None
}

// Extract filename from response headers or URL
pub fn extract_filename(res: &reqwest::Response, url: &str, map_id: u32) -> String {
    let mut filename = String::new();
    
    if let Some(content_disposition) = res.headers().get("content-disposition") {
        if let Ok(disposition_str) = content_disposition.to_str() {
            for part in disposition_str.split(';') {
                let part = part.trim();
                if part.starts_with("filename*=UTF-8''") {
                    if let Some(encoded) = part.strip_prefix("filename*=UTF-8''") {
                        filename = percent_decode(encoded);
                        if !filename.is_empty() {
                            break;
                        }
                    }
                } else if part.starts_with("filename*=") {
                    if let Some(encoded) = part.strip_prefix("filename*=") {
                        if let Some(utf8_part) = encoded.strip_prefix("UTF-8''") {
                            filename = percent_decode(utf8_part);
                            if !filename.is_empty() {
                                break;
                            }
                        }
                    }
                } else if part.starts_with("filename=") {
                    let raw_filename = part.strip_prefix("filename=").unwrap_or("").trim();
                    let raw_filename = raw_filename.trim_matches('"').trim_matches('\'');
                    if !raw_filename.is_empty() {
                        filename = percent_decode(raw_filename);
                        break;
                    }
                }
            }
        }
    }
    
    // If not in headers, try extracting from the URL path
    if filename.is_empty() {
        if let Ok(url_obj) = reqwest::Url::parse(url) {
            if let Some(segments) = url_obj.path_segments() {
                for segment in segments.rev() {
                    if !segment.is_empty() && segment != "/" {
                        // Check whether a file extension exists
                        if Path::new(segment).extension().is_some() {
                            filename = percent_decode(segment);
                            break;
                        }
                    }
                }
            }
        }
    }
    
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
    
    if filename.is_empty() {
        filename = format!("{}.osz", map_id);
    }
    
    let mut decoded = filename;
    for _ in 0..3 {
        if decoded.contains('%') {
            let new_decoded = percent_decode(&decoded);
            if new_decoded == decoded {
                break;
            }
            decoded = new_decoded;
        } else {
            break;
        }
    }
    
    decoded
}

// Percent-decoding function (URL decode)
fn percent_decode(input: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = input.chars();
    
    while let Some(ch) = chars.next() {
        if ch == '%' {
            // Try to read two hexadecimal characters
            if let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
                if c1.is_ascii_hexdigit() && c2.is_ascii_hexdigit() {
                    let hex = format!("{}{}", c1, c2);
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        bytes.push(byte);
                        continue;
                    }
                }
                bytes.push(b'%');
                let mut buf = [0; 4];
                bytes.extend_from_slice(c1.encode_utf8(&mut buf).as_bytes());
                bytes.extend_from_slice(c2.encode_utf8(&mut buf).as_bytes());
            } else {
                bytes.push(b'%');
            }
        } else {
            let mut buf = [0; 4];
            let encoded = ch.encode_utf8(&mut buf);
            bytes.extend_from_slice(encoded.as_bytes());
        }
    }
    
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
