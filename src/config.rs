use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub api_key: String,
    pub save_dir: String,
    pub model: String,
    pub output_format: String,
    pub mode: String,
    pub last_prompt: String,
    pub last_size: String,
    #[serde(default)]
    pub last_video_prompt: String,
    #[serde(default)]
    pub video_neg_prompt: String,
    #[serde(default)]
    pub video_width: i32,
    #[serde(default)]
    pub video_height: i32,
    #[serde(default)]
    pub video_num_frames: i32,
    #[serde(default)]
    pub video_frame_rate: i32,
    #[serde(default)]
    pub video_duration_preset: usize,
    #[serde(default)]
    pub video_mode: String, // "text" | "image" | "multi" | "keyframes"
}

impl Default for Config {
    fn default() -> Self {
        let save_dir = dirs::picture_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
            .join("AgnesStudio")
            .to_string_lossy()
            .to_string();
        Self {
            api_key: String::new(),
            save_dir,
            model: "agnes-image-2.1-flash".to_string(),
            output_format: "url".to_string(),
            mode: "text".to_string(),
            last_prompt: String::new(),
            last_size: "1024x1024".to_string(),
            last_video_prompt: String::new(),
            video_neg_prompt: String::new(),
            video_width: 1152,
            video_height: 768,
            video_num_frames: 121,
            video_frame_rate: 24,
            video_duration_preset: 1,
            video_mode: "text".to_string(),
        }
    }
}

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
        .join("agnes-studio")
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load() -> Config {
    let path = config_path();
    match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save(cfg: &Config) {
    let dir = config_dir();
    let _ = fs::create_dir_all(&dir);
    let path = config_path();
    if let Ok(s) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(&path, s);
    }
}
