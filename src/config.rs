use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    /// 旧版单 Key 字段（0.9.0 及以前固定使用国际站），仅作迁移来源，不再使用
    pub api_key: String,
    /// 国际站（apihub.agnes-ai.com）免费 / 默认密钥
    #[serde(default)]
    pub api_key_com: String,
    /// 国际站企业认证密钥
    #[serde(default)]
    pub api_key_com_enterprise: String,
    /// 国际站 Token Plan 密钥
    #[serde(default)]
    pub api_key_com_tokenplan: String,
    /// 国内站（api.agnes-ai.cn）免费 / 默认密钥
    #[serde(default)]
    pub api_key_cn: String,
    /// 国内站企业认证密钥
    #[serde(default)]
    pub api_key_cn_enterprise: String,
    /// 国内站 Token Plan 密钥
    #[serde(default)]
    pub api_key_cn_tokenplan: String,
    /// 当前启用的站点："com" | "cn"。与 key_type 共同指向一个已启用的密钥槽位
    #[serde(default)]
    pub site: String,
    /// 当前启用的密钥类型："default" | "enterprise" | "tokenplan"
    #[serde(default)]
    pub key_type: String,
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
    // 图像 Flash 系（2.1 / 2.5）档位式尺寸：size 档位 + 宽高比
    #[serde(default)]
    pub image_tier: String,  // "1K" | "2K" | "3K" | "4K"，空 = 未设置
    #[serde(default)]
    pub image_ratio: String, // "1:1" 等，空 = 未设置
    // 视频模型与 Agnes Video 2.5 参数
    #[serde(default)]
    pub video_model: String, // "agnes-video-2.5-flash" | "agnes-video-2.5" | "agnes-video-v2.0"，空 = 默认 2.5 Flash
    #[serde(default)]
    pub video25_mode: String, // "text" | "keyframe" | "reference"
    #[serde(default)]
    pub video25_seconds: String, // "4"–"12"
    #[serde(default)]
    pub video25_ar: String, // "16:9" 等
    #[serde(default)]
    pub video25_first_frame: String,
    #[serde(default)]
    pub video25_last_frame: String,
    // 界面偏好
    #[serde(default)]
    pub theme: String, // "light" | "dark" | "system"，空 = 跟随系统
    #[serde(default)]
    pub lang: String,  // "zh" | "en"，空 = 中文
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
            api_key_com: String::new(),
            api_key_com_enterprise: String::new(),
            api_key_com_tokenplan: String::new(),
            api_key_cn: String::new(),
            api_key_cn_enterprise: String::new(),
            api_key_cn_tokenplan: String::new(),
            site: "com".to_string(),
            key_type: "default".to_string(),
            save_dir,
            model: "agnes-image-2.5-flash".to_string(),
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
            image_tier: "1K".to_string(),
            image_ratio: "1:1".to_string(),
            video_model: "agnes-video-2.5-flash".to_string(),
            video25_mode: "text".to_string(),
            video25_seconds: "5".to_string(),
            video25_ar: "16:9".to_string(),
            video25_first_frame: String::new(),
            video25_last_frame: String::new(),
            theme: "system".to_string(),
            lang: "zh".to_string(),
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
    let mut cfg = match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Config::default(),
    };
    // 旧版单 Key 迁移：0.9.0 及以前固定走国际站，老用户的 Key 归入国际站
    if cfg.api_key_com.trim().is_empty() && !cfg.api_key.trim().is_empty() {
        cfg.api_key_com = cfg.api_key.clone();
    }
    // 站点字段归一化：非 "cn" 一律按 "com" 处理
    if !cfg.site.eq_ignore_ascii_case("cn") {
        cfg.site = "com".to_string();
    }
    cfg
}

pub fn save(cfg: &Config) {
    let dir = config_dir();
    let _ = fs::create_dir_all(&dir);
    let path = config_path();
    if let Ok(s) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(&path, s);
    }
}
