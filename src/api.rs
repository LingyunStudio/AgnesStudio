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
        return_base64: None,
        extra_body: Some(extra),
    }
}

pub async fn generate(p: GenParams) -> Result<GenResult, String> {
    if p.api_key.trim().is_empty() {
        return Err("未设置 API Key，请先在“设置”里填写。".to_string());
    }
    if p.prompt.trim().is_empty() {
        return Err("提示词不能为空。".to_string());
    }

    let body = serde_json::to_string(&build_body(&p))
        .map_err(|e| format!("序列化请求失败：{e}"))?;
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
        .map_err(|e| format!("读取响应失败：{e}"))?;
    if !status.is_success() {
        return Err(format!("HTTP {status}\n{text}"));
    }

    let parsed: GenResponse =
        serde_json::from_str(&text).map_err(|e| format!("解析响应失败：{e}\n原始：{text}"))?;

    let data = parsed
        .data
        .into_iter()
        .next()
        .ok_or_else(|| "响应中没有 data。".to_string())?;

    if let Some(url) = data.url.filter(|s| !s.is_empty()) {
        let bytes = send_retry(&url, reqwest::Method::GET, "", None, 3, Duration::from_secs(360))
            .await?
            .bytes()
            .await
            .map_err(|e| format!("读取图片字节失败：{e}"))?
            .to_vec();
        Ok(GenResult {
            url: Some(url),
            bytes,
        })
    } else if let Some(b64) = data.b64_json.filter(|s| !s.is_empty()) {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&b64)
            .map_err(|e| format!("解码 Base64 失败：{e}"))?;
        Ok(GenResult {
            url: None,
            bytes,
        })
    } else {
        Err("响应中既没有 url 也没有 b64_json。".to_string())
    }
}

// ── 视频生成（异步任务流程）──────────────────────────────────────────────────────

const VIDEO_MODEL: &str = "agnes-video-v2.0";

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
        .map_err(|e| format!("创建 HTTP 客户端失败：{e}"))
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
        "请求发送失败（已重试 {attempts} 次）：{}",
        last.unwrap_or_default()
    ))
}

pub struct VideoParams {
    pub api_key: String,
    pub prompt: String,
    pub negative_prompt: String,
    pub width: i32,
    pub height: i32,
    pub num_frames: i32,
    pub frame_rate: i32,
    pub seed: Option<i64>,
    /// 输入图片 URL 列表：空=文生视频；1张=图生视频；多张=多图/关键帧
    pub images: Vec<String>,
    pub keyframes: bool,
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

#[derive(Deserialize)]
struct VideoCreateResp {
    video_id: Option<String>,
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
    remixed_from_video_id: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

fn build_video_body(p: &VideoParams) -> VideoRequest {
    let multi = p.images.len() > 1 || p.keyframes;
    let (top_image, extra) = if multi {
        // 多图 / 关键帧：图片放进 extra_body.image，关键帧再设 extra_body.mode
        (
            None,
            Some(VideoExtraBody {
                image: if p.images.is_empty() { None } else { Some(p.images.clone()) },
                mode: if p.keyframes { Some("keyframes".to_string()) } else { None },
            }),
        )
    } else if p.images.len() == 1 {
        // 单图图生视频：image 放顶层（API 要求字符串，非数组）
        (Some(p.images[0].clone()), None)
    } else {
        (None, None)
    };

    VideoRequest {
        model: VIDEO_MODEL.to_string(),
        prompt: p.prompt.clone(),
        width: p.width,
        height: p.height,
        num_frames: p.num_frames,
        frame_rate: p.frame_rate,
        negative_prompt: if p.negative_prompt.trim().is_empty() {
            None
        } else {
            Some(p.negative_prompt.clone())
        },
        seed: p.seed,
        image: top_image,
        extra_body: extra,
    }
}

pub async fn create_video_task(p: &VideoParams) -> Result<VideoTask, String> {
    if p.api_key.trim().is_empty() {
        return Err("未设置 API Key，请先在\"设置\"里填写。".to_string());
    }
    if p.prompt.trim().is_empty() {
        return Err("提示词不能为空。".to_string());
    }
    let body = serde_json::to_string(&build_video_body(p))
        .map_err(|e| format!("序列化请求失败：{e}"))?;
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
        .map_err(|e| format!("读取响应失败：{e}"))?;
    if !status.is_success() {
        return Err(format!("创建任务失败：HTTP {status}\n{text}"));
    }
    let parsed: VideoCreateResp =
        serde_json::from_str(&text).map_err(|e| format!("解析响应失败：{e}\n原始：{text}"))?;

    let video_id = parsed
        .video_id
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("响应中缺少 video_id。\n原始：{text}"))?;
    let task_id = parsed.task_id.unwrap_or_default();
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
) -> Result<VideoStatus, String> {
    // 优先用 video_id 查询；video_id 失败回退 task_id
    let (url, use_task) = if !video_id.is_empty() {
        (
            format!("{VIDEO_RESULT}?video_id={video_id}&model_name={VIDEO_MODEL}"),
            false,
        )
    } else {
        (format!("{VIDEO_CREATE}/{task_id}"), true)
    };

    let resp = send_retry(&url, reqwest::Method::GET, api_key, None, 3, Duration::from_secs(60))
        .await?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("读取响应失败：{e}"))?;
    if !status.is_success() {
        return Err(format!("查询任务失败：HTTP {status}\n{text}"));
    }
    let parsed: VideoStatusResp =
        serde_json::from_str(&text).map_err(|e| format!("解析响应失败：{e}\n原始：{text}"))?;

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
            out.video_url = parsed.remixed_from_video_id.filter(|s| !s.is_empty());
        }
        "failed" => {
            out.failed = true;
            out.message = parsed.error.unwrap_or_else(|| "生成失败".to_string());
        }
        _ => {
            out.message = match st.as_str() {
                "queued" => "排队中…".to_string(),
                "in_progress" => "生成中…".to_string(),
                _ => st,
            };
        }
    }
    let _ = use_task;
    Ok(out)
}

/// 下载视频字节
/// 下载视频字节。视频 URL 通常是已签名的公开链接（storage.googleapis.com 等），
/// 浏览器能直接打开 = 无需鉴权。带 Authorization 头反而可能让某些 CDN/存储
/// 返回错误页（HTML）而非 mp4，导致写出的文件损坏无法播放，因此不带鉴权头。
pub async fn download_video(_api_key: &str, url: &str) -> Result<Vec<u8>, String> {
    let resp = send_retry(url, reqwest::Method::GET, "", None, 3, Duration::from_secs(600))
        .await?;
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
        .map_err(|e| format!("读取视频字节失败：{e}"))?
        .to_vec();
    if !status.is_success() {
        return Err(format!("下载视频失败：HTTP {status}"));
    }
    // mp4 文件 4~8 字节处含 "ftyp" 标识；HTML 错误页首字节是 '<'
    let valid_mp4 = bytes.len() > 12
        && (&bytes[4..8] == b"ftyp")
        && (bytes[0] == 0 && bytes[1] == 0 && bytes[2] == 0);
    if !valid_mp4 {
        let snippet = String::from_utf8_lossy(&bytes[..bytes.len().min(80)]);
        return Err(format!(
            "下载到的不是有效 mp4（Content-Type: {ctype}, {} 字节，开头：{snippet}）",
            bytes.len()
        ));
    }
    Ok(bytes)
}
