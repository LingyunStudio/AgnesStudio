use base64::Engine;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const ENDPOINT: &str = "https://apihub.agnes-ai.com/v1/images/generations";
const VIDEO_CREATE: &str = "https://apihub.agnes-ai.com/v1/videos";
const VIDEO_RESULT: &str = "https://apihub.agnes-ai.com/agnesapi";

pub struct GenParams {
    pub api_key: String,
    pub model: String,
    pub prompt: String,
    pub size: String,
    /// 宽高比（仅 2.1 Flash 支持，配合档位式 size 如 "2K"）：如 "16:9"
    pub ratio: Option<String>,
    /// None = 文生图；Some(uri) = 图生图（公网 URL 或 data:image/...;base64,... ）
    pub input_image: Option<String>,
    /// "url" 或 "b64_json"
    pub output_format: String,
}

pub struct GenResult {
    pub url: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Serialize)]
struct ExtraBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Vec<String>>,
    response_format: String,
}

#[derive(Serialize)]
struct Request {
    model: String,
    prompt: String,
    size: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_base64: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_body: Option<ExtraBody>,
}

#[derive(Deserialize)]
struct GenResponse {
    data: Vec<GenData>,
}

#[derive(Deserialize)]
struct GenData {
    url: Option<String>,
    b64_json: Option<String>,
}

fn build_body(p: &GenParams) -> Request {
    let want_b64 = p.output_format == "b64_json";
    let img2img = p.input_image.is_some();

    // 文生图 Base64：文档要求顶层 return_base64=true（不带 extra_body）
    if want_b64 && !img2img {
        return Request {
            model: p.model.clone(),
            prompt: p.prompt.clone(),
            size: p.size.clone(),
            ratio: p.ratio.clone(),
            return_base64: Some(true),
            extra_body: None,
        };
    }

    // 其余情况：response_format 放进 extra_body（图生图同时把 image 放进 extra_body）
    let extra = ExtraBody {
        image: p.input_image.clone().map(|s| vec![s]),
        response_format: if want_b64 {
            "b64_json".to_string()
        } else {
            "url".to_string()
        },
    };
    Request {
        model: p.model.clone(),
        prompt: p.prompt.clone(),
        size: p.size.clone(),
        ratio: p.ratio.clone(),
        return_base64: None,
        extra_body: Some(extra),
    }
}

pub async fn generate(p: GenParams) -> Result<GenResult, String> {
    if p.api_key.trim().is_empty() {
        return Err("API key is not set. Add it under Settings.".to_string());
    }
    if p.prompt.trim().is_empty() {
        return Err("Prompt cannot be empty.".to_string());
    }

    let body = serde_json::to_string(&build_body(&p))
        .map_err(|e| format!("Failed to serialize request: {e}"))?;
    let resp = send_retry(
        ENDPOINT,
        reqwest::Method::POST,
        &p.api_key,
        Some(&body),
        3,
        Duration::from_secs(360),
    )
    .await?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;
    if !status.is_success() {
        return Err(format!("HTTP {status}\n{text}"));
    }

    let parsed: GenResponse =
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse response: {e}\nRaw: {text}"))?;

    let data = parsed
        .data
        .into_iter()
        .next()
        .ok_or_else(|| "Response contains no data.".to_string())?;

    if let Some(url) = data.url.filter(|s| !s.is_empty()) {
        let bytes = send_retry(&url, reqwest::Method::GET, "", None, 3, Duration::from_secs(360))
            .await?
            .bytes()
            .await
            .map_err(|e| format!("Failed to read image bytes: {e}"))?
            .to_vec();
        Ok(GenResult {
            url: Some(url),
            bytes,
        })
    } else if let Some(b64) = data.b64_json.filter(|s| !s.is_empty()) {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&b64)
            .map_err(|e| format!("Failed to decode base64: {e}"))?;
        Ok(GenResult {
            url: None,
            bytes,
        })
    } else {
        Err("Response contains neither url nor b64_json.".to_string())
    }
}

// ── 视频生成（异步任务流程）──────────────────────────────────────────────────────

pub const MODEL_VIDEO_V20: &str = "agnes-video-v2.0";
pub const MODEL_VIDEO_V25: &str = "agnes-video-2.5";
pub const MODEL_VIDEO_V25_FLASH: &str = "agnes-video-2.5-flash";

/// 展开错误原因链，让 reqwest 顶层 "error sending request for url" 之外的
/// 真正原因（超时 / 连接重置 / DNS 等）可见。
fn err_chain(e: &dyn std::error::Error) -> String {
    let mut s = format!("{e}");
    let mut src = e.source();
    while let Some(c) = src {
        s.push_str(&format!("\n  ↳ {c}"));
        src = c.source();
    }
    s
}

/// 构建一个带合理超时的 HTTP client。每次重试都新建 client，避免连接池里
/// 半死的连接导致重试也卡在同一条连接上。强制 HTTP/1.1：reqwest 默认协商
/// HTTP/2，若服务端某端点的 h2 实现有问题会一直挂起（operation timed out），
/// 而 HTTP/1.1 与 curl 行为一致，更稳。
fn build_client(timeout: Duration) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(timeout)
        .connect_timeout(Duration::from_secs(15))
        .http1_only()
        .pool_idle_timeout(Duration::from_secs(10))
        .tcp_keepalive(Duration::from_secs(30))
        .user_agent("agnes-studio/0.1")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))
}

/// 对请求发送做自动重试。仅在发送/读取阶段失败时重试；拿到响应后交由调用方
/// 处理状态码。每次重试重建 client，绕开可能卡住的连接。
async fn send_retry(
    url: &str,
    method: reqwest::Method,
    api_key: &str,
    body: Option<&str>,
    attempts: u32,
    timeout: Duration,
) -> Result<reqwest::Response, String> {
    let mut last: Option<String> = None;
    for attempt in 0..attempts {
        if attempt > 0 {
            // 退避：2s → 5s → 8s
            let secs = match attempt { 1 => 2, 2 => 5, _ => 8 };
            tokio::time::sleep(Duration::from_secs(secs)).await;
        }
        let client = match build_client(timeout) {
            Ok(c) => c,
            Err(e) => return Err(e),
        };
        let mut req = client.request(method.clone(), url);
        // 仅在提供有效 key 时加鉴权头；空 key 下载公开签名 URL 时不能带
        // Authorization 头，否则服务器把空 Bearer 当无效鉴权返回 401。
        if !api_key.is_empty() {
            req = req.bearer_auth(api_key);
        }
        if let Some(b) = body {
            req = req.header("Content-Type", "application/json").body(b.to_string());
        }
        match req.send().await {
            Ok(r) => return Ok(r),
            Err(e) => {
                last = Some(err_chain(&e));
            }
        }
    }
    Err(format!(
        "Request failed after {attempts} attempts: {}",
        last.unwrap_or_default()
    ))
}

/// Agnes Video 2.5 的生成模式
#[derive(Clone, Copy, PartialEq)]
pub enum V25Mode {
    Text,
    Keyframe,
    Reference,
}

/// 两代视频模型的请求参数。V2.0 走 width/height/num_frames/frame_rate；
/// 2.5 走 mode/seconds/size=720P/aspect_ratio，且媒体字段随模式变化。
pub enum VideoKind {
    V20 {
        negative_prompt: String,
        width: i32,
        height: i32,
        num_frames: i32,
        frame_rate: i32,
        seed: Option<i64>,
        /// 输入图片 URL 列表：空=文生视频；1张=图生视频；多张=多图/关键帧
        images: Vec<String>,
        keyframes: bool,
    },
    V25 {
        mode: V25Mode,
        /// 时长 "4"–"12"（API 要求字符串）
        seconds: String,
        /// 画幅比例，如 "16:9"
        aspect_ratio: String,
        seed: Option<i64>,
        /// 首帧图片 URL（keyframe 模式，与 last_frame 至少一个）
        first_frame: Option<String>,
        /// 尾帧图片 URL（keyframe 模式）
        last_frame: Option<String>,
        /// 参考图片 URL（reference 模式；Flash 最多 5 张）
        images: Vec<String>,
        /// 参考音频 URL（reference 模式）
        audios: Vec<String>,
        /// 参考视频 URL（reference 模式；Flash 不支持，序列化时跳过）
        videos: Vec<String>,
        /// true = agnes-video-2.5-flash（图片最多 5 张、不支持视频参考）
        flash: bool,
    },
}

pub struct VideoParams {
    pub api_key: String,
    pub prompt: String,
    pub kind: VideoKind,
}

/// 创建任务后返回的标识
pub struct VideoTask {
    pub video_id: String,
    pub task_id: String,
    pub seconds: String,
    pub size: String,
}

#[derive(Serialize)]
struct VideoExtraBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
}

#[derive(Serialize)]
struct VideoRequest {
    model: String,
    prompt: String,
    width: i32,
    height: i32,
    num_frames: i32,
    frame_rate: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    negative_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_body: Option<VideoExtraBody>,
}

/// 参考视频对象：2.5 的 videos[] 数组元素（start_seconds 默认 0，require_audio 默认 false）
#[derive(Serialize)]
struct Video25RefVideo {
    url: String,
}

#[derive(Serialize)]
struct Video25Request {
    model: String,
    prompt: String,
    /// "text" | "keyframe" | "reference"
    mode: String,
    seconds: String,
    /// 当前仅支持 "720P"
    size: String,
    aspect_ratio: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_frame: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_frame: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    images: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    audios: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    videos: Vec<Video25RefVideo>,
}

/// V2.0 响应带 video_id/task_id；2.5 响应只有 id（查询时作为 video_id 使用）
#[derive(Deserialize)]
struct VideoCreateResp {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    video_id: Option<String>,
    #[serde(default)]
    task_id: Option<String>,
    #[serde(default)]
    seconds: Option<String>,
    #[serde(default)]
    size: Option<String>,
}

#[derive(Deserialize)]
struct VideoStatusResp {
    status: Option<String>,
    progress: Option<f32>,
    #[serde(default)]
    seconds: Option<String>,
    #[serde(default)]
    size: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    metadata: Option<VideoMeta>,
    #[serde(default)]
    remixed_from_video_id: Option<String>,
    /// V2.0 可能是字符串，2.5 是对象 {"message": "..."}
    #[serde(default)]
    error: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct VideoMeta {
    #[serde(default)]
    url: Option<String>,
}

/// 从 error 字段（字符串或对象）提取可读信息
fn error_message(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(m) => m
            .get("message")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| v.to_string()),
        _ => v.to_string(),
    }
}

fn build_video_body(p: &VideoParams) -> VideoRequest {
    let (negative_prompt, width, height, num_frames, frame_rate, seed, images, keyframes) =
        match &p.kind {
            VideoKind::V20 {
                negative_prompt,
                width,
                height,
                num_frames,
                frame_rate,
                seed,
                images,
                keyframes,
            } => (
                negative_prompt.clone(),
                *width,
                *height,
                *num_frames,
                *frame_rate,
                *seed,
                images.clone(),
                *keyframes,
            ),
            VideoKind::V25 { .. } => unreachable!("V2.0 请求体不应由 V25 参数构建"),
        };
    let multi = images.len() > 1 || keyframes;
    let (top_image, extra) = if multi {
        // 多图 / 关键帧：图片放进 extra_body.image，关键帧再设 extra_body.mode
        (
            None,
            Some(VideoExtraBody {
                image: if images.is_empty() { None } else { Some(images.clone()) },
                mode: if keyframes { Some("keyframes".to_string()) } else { None },
            }),
        )
    } else if images.len() == 1 {
        // 单图图生视频：image 放顶层（API 要求字符串，非数组）
        (Some(images[0].clone()), None)
    } else {
        (None, None)
    };

    VideoRequest {
        model: MODEL_VIDEO_V20.to_string(),
        prompt: p.prompt.clone(),
        width,
        height,
        num_frames,
        frame_rate,
        negative_prompt: if negative_prompt.trim().is_empty() {
            None
        } else {
            Some(negative_prompt)
        },
        seed,
        image: top_image,
        extra_body: extra,
    }
}

fn build_video25_body(p: &VideoParams) -> Video25Request {
    let (
        mode,
        seconds,
        aspect_ratio,
        seed,
        first_frame,
        last_frame,
        images,
        audios,
        videos,
        flash,
    ) = match &p.kind {
        VideoKind::V25 {
            mode,
            seconds,
            aspect_ratio,
            seed,
            first_frame,
            last_frame,
            images,
            audios,
            videos,
            flash,
        } => (
            *mode,
            seconds.clone(),
            aspect_ratio.clone(),
            *seed,
            first_frame.clone(),
            last_frame.clone(),
            images.clone(),
            audios.clone(),
            videos.clone(),
            *flash,
        ),
        VideoKind::V20 { .. } => unreachable!("V2.5 请求体不应由 V20 参数构建"),
    };
    let model = if flash {
        MODEL_VIDEO_V25_FLASH
    } else {
        MODEL_VIDEO_V25
    };
    // 模式规则（官方文档）：text 不允许任何媒体字段；keyframe 仅 first/last_frame；
    // reference 仅 images/audios/videos。切换生成模式后另一模式的媒体若仍残留在
    // 状态里，会同时出现在请求体中，服务端返回 400
    // "keyframe media and reference media cannot be combined"，因此这里按模式过滤。
    let kf_media = matches!(mode, V25Mode::Keyframe);
    let ref_media = matches!(mode, V25Mode::Reference);
    Video25Request {
        model: model.to_string(),
        prompt: p.prompt.clone(),
        mode: match mode {
            V25Mode::Text => "text",
            V25Mode::Keyframe => "keyframe",
            V25Mode::Reference => "reference",
        }
        .to_string(),
        seconds,
        size: "720P".to_string(),
        aspect_ratio,
        seed,
        first_frame: if kf_media {
            first_frame.filter(|s| !s.trim().is_empty())
        } else {
            None
        },
        last_frame: if kf_media {
            last_frame.filter(|s| !s.trim().is_empty())
        } else {
            None
        },
        images: if ref_media {
            images
                .iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            vec![]
        },
        audios: if ref_media {
            audios
                .iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            vec![]
        },
        // Flash 不支持视频参考（传入有效内容返回 400），直接不序列化
        videos: if ref_media && !flash {
            videos
                .iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .map(|url| Video25RefVideo { url })
                .collect()
        } else {
            vec![]
        },
    }
}

pub async fn create_video_task(p: &VideoParams) -> Result<VideoTask, String> {
    if p.api_key.trim().is_empty() {
        return Err("API key is not set. Add it under Settings.".to_string());
    }
    if p.prompt.trim().is_empty() {
        return Err("Prompt cannot be empty.".to_string());
    }
    let body = match &p.kind {
        VideoKind::V20 { .. } => serde_json::to_string(&build_video_body(p)),
        VideoKind::V25 { .. } => serde_json::to_string(&build_video25_body(p)),
    }
    .map_err(|e| format!("Failed to serialize request: {e}"))?;
    let resp = send_retry(
        VIDEO_CREATE,
        reqwest::Method::POST,
        &p.api_key,
        Some(&body),
        3,
        Duration::from_secs(180),
    )
    .await?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;
    if !status.is_success() {
        return Err(format!("Task creation failed: HTTP {status}\n{text}"));
    }
    let parsed: VideoCreateResp =
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse response: {e}\nRaw: {text}"))?;

    // V2.0：video_id + task_id；2.5：仅 id。统一取第一个非空值。
    let video_id = parsed
        .video_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .or(parsed.id.as_deref().filter(|s| !s.is_empty()))
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Response contains no video_id.\nRaw: {text}"))?;
    let task_id = parsed
        .task_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .or(parsed.id.as_deref().filter(|s| !s.is_empty()))
        .map(|s| s.to_string())
        .unwrap_or_default();
    Ok(VideoTask {
        video_id,
        task_id,
        seconds: parsed.seconds.unwrap_or_default(),
        size: parsed.size.unwrap_or_default(),
    })
}

/// 视频任务当前状态
pub struct VideoStatus {
    pub done: bool,
    pub failed: bool,
    pub progress: f32, // 0..=100
    pub message: String,
    pub video_url: Option<String>,
    pub seconds: String,
    pub size: String,
}

pub async fn fetch_video_status(
    api_key: &str,
    video_id: &str,
    task_id: &str,
    model: &str,
) -> Result<VideoStatus, String> {
    // 优先用 video_id 查询（model_name 显式指定模型）；video_id 失败回退 task_id
    let url = if !video_id.is_empty() {
        format!("{VIDEO_RESULT}?video_id={video_id}&model_name={model}")
    } else {
        format!("{VIDEO_CREATE}/{task_id}")
    };

    let resp = send_retry(&url, reqwest::Method::GET, api_key, None, 3, Duration::from_secs(60))
        .await?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("Failed to read response: {e}"))?;
    if !status.is_success() {
        return Err(format!("Status query failed: HTTP {status}\n{text}"));
    }
    let parsed: VideoStatusResp =
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse response: {e}\nRaw: {text}"))?;

    // message 保留原始状态码（queued / in_progress / …），展示层按当前语言本地化
    let st = parsed.status.unwrap_or_default();
    let mut out = VideoStatus {
        done: false,
        failed: false,
        progress: parsed.progress.unwrap_or(0.0),
        message: st.clone(),
        video_url: None,
        seconds: parsed.seconds.unwrap_or_default(),
        size: parsed.size.unwrap_or_default(),
    };
    match st.as_str() {
        "completed" => {
            out.done = true;
            out.progress = 100.0;
            // 2.5 在顶层 url；V2.0 文档写 metadata.url（实际接口为顶层 url）
            out.video_url = parsed
                .url
                .or(parsed.metadata.and_then(|m| m.url))
                .or(parsed.remixed_from_video_id)
                .filter(|s| !s.is_empty());
        }
        "failed" => {
            out.failed = true;
            // 有错误详情用详情，没有则留空由展示层兜底
            out.message = parsed
                .error
                .as_ref()
                .map(|e| error_message(e))
                .filter(|s| !s.is_empty())
                .unwrap_or_default();
        }
        _ => {}
    }
    Ok(out)
}

/// 下载视频字节。视频 URL 通常是已签名的公开链接（storage.googleapis.com 等），
/// 浏览器能直接打开 = 无需鉴权。带 Authorization 头反而可能让某些 CDN/存储
/// 返回错误页（HTML）而非 mp4，导致写出的文件损坏无法播放，因此先不带鉴权头；
/// 仅当返回 401/403 时才带鉴权重试一次。
pub async fn download_video(api_key: &str, url: &str) -> Result<Vec<u8>, String> {
    let mut resp = send_retry(url, reqwest::Method::GET, "", None, 3, Duration::from_secs(600))
        .await?;
    if (resp.status() == reqwest::StatusCode::UNAUTHORIZED
        || resp.status() == reqwest::StatusCode::FORBIDDEN)
        && !api_key.trim().is_empty()
    {
        resp =
            send_retry(url, reqwest::Method::GET, api_key, None, 3, Duration::from_secs(600))
                .await?;
    }
    let status = resp.status();
    // 检查 Content-Type，避免把 HTML 错误页当视频存下来
    let ctype = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Failed to read video bytes: {e}"))?
        .to_vec();
    if !status.is_success() {
        return Err(format!("Video download failed: HTTP {status}"));
    }
    // mp4 文件 4~8 字节处含 "ftyp" 标识；HTML 错误页首字节是 '<'
    let valid_mp4 = bytes.len() > 12
        && (&bytes[4..8] == b"ftyp")
        && (bytes[0] == 0 && bytes[1] == 0 && bytes[2] == 0);
    if !valid_mp4 {
        let snippet = String::from_utf8_lossy(&bytes[..bytes.len().min(80)]);
        return Err(format!(
            "Downloaded content is not a valid mp4 (Content-Type: {ctype}, {} bytes, head: {snippet})",
            bytes.len()
        ));
    }
    Ok(bytes)
}
