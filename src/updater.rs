// 自动更新：检查 GitHub Releases、下载 setup.exe、触发静默安装并退出。
//
// 流程：check_latest() 比对版本 → 有新版返回 UpdateInfo →
//       download_setup() 下载到临时目录 → run_installer_and_exit() 启动
//       Inno Setup 静默安装（/VERYSILENT /CLOSEAPPLICATIONS /RESTARTAPPLICATIONS）
//       并立即退出当前进程，由安装器接管文件替换与重启。

use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

const REPO_LATEST: &str = "https://api.github.com/repos/LingyunStudio/AgnesStudio/releases/latest";
const SETUP_NAME_PREFIX: &str = "AgnesStudio-Setup-";

/// 当前编译版本号（编译期内联，与 Cargo.toml 一致）
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, PartialEq)]
pub struct UpdateInfo {
    /// 新版本号，如 "0.2.0"（已去掉 tag 的 v 前缀）
    pub version: String,
    /// Release notes（release body 原文）
    pub notes: String,
    /// 自动下载用的 setup.exe 直链；None 表示该 release 没有可用的安装包资源
    pub setup_url: Option<String>,
    /// Release 页面地址，setup_url 为 None 时引导用户浏览器打开
    pub html_url: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    #[serde(default)]
    tag_name: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    #[serde(default)]
    name: String,
    #[serde(default)]
    browser_download_url: String,
}

/// 把 "v0.2.0" / "0.2.0" 解析为 (0,2,0)；无法解析返回 None
fn version_tuple(v: &str) -> Option<(u64, u64, u64)> {
    let s = v.trim().trim_start_matches('v').trim_start_matches('V');
    let mut it = s.split('.');
    let a = it.next()?.parse::<u64>().ok()?;
    let b = it.next()?.parse::<u64>().ok()?;
    let c = it.next().unwrap_or("0").split('-').next()?.parse::<u64>().ok()?;
    Some((a, b, c))
}

/// 比较 latest 与当前版本；latest 严格大于当前才算有更新
fn is_newer(latest: &str, current: &str) -> bool {
    match (version_tuple(latest), version_tuple(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

/// 构建一个专用于 GitHub API 的 client：需要 User-Agent（GitHub 强制要求），HTTP/1.1。
fn gh_client(timeout: Duration) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(timeout)
        .connect_timeout(Duration::from_secs(15))
        .http1_only()
        .user_agent(format!("agnes-studio/{}", CURRENT_VERSION))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败：{e}"))
}

/// 检查 GitHub 最新 release。返回 Ok(None) 表示无更新。
pub async fn check_latest() -> Result<Option<UpdateInfo>, String> {
    let client = gh_client(Duration::from_secs(20))?;
    let resp = client
        .get(REPO_LATEST)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("请求 GitHub 失败：{e}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("GitHub 返回 HTTP {status}"));
    }
    let rel: GithubRelease = resp
        .json()
        .await
        .map_err(|e| format!("解析 GitHub 响应失败：{e}"))?;

    if !is_newer(&rel.tag_name, CURRENT_VERSION) {
        return Ok(None);
    }
    // 找名字以 AgnesStudio-Setup- 开头、.exe 结尾的资源
    let setup_url = rel
        .assets
        .iter()
        .find(|a| a.name.starts_with(SETUP_NAME_PREFIX) && a.name.to_lowercase().ends_with(".exe"))
        .map(|a| a.browser_download_url.clone());
    let version = version_tuple(&rel.tag_name)
        .map(|(a, b, c)| format!("{a}.{b}.{c}"))
        .unwrap_or_else(|| rel.tag_name.trim_start_matches('v').to_string());
    Ok(Some(UpdateInfo {
        version,
        notes: rel.body,
        setup_url,
        html_url: rel.html_url,
    }))
}

/// 下载 setup.exe 到 dest。带 10 分钟超时与 3 次重试。
/// 通过 progress 回调上报已下载字节数与总字节数，供 UI 显示进度。
pub async fn download_setup<F: Fn(u64, u64)>(
    url: &str,
    dest: &Path,
    progress: F,
) -> Result<(), String> {
    let client = gh_client(Duration::from_secs(600))?;
    let mut last_err = String::new();
    for attempt in 0..3u32 {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let total = resp.content_length().unwrap_or(0);
                use futures_util::StreamExt;
                let mut stream = resp.bytes_stream();
                let mut file = tokio::fs::File::create(dest)
                    .await
                    .map_err(|e| format!("创建临时文件失败：{e}"))?;
                let mut got: u64 = 0;
                let mut ok = true;
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(b) => {
                            use tokio::io::AsyncWriteExt;
                            if file.write_all(&b).await.is_err() {
                                ok = false;
                                break;
                            }
                            got += b.len() as u64;
                            progress(got, total);
                        }
                        Err(e) => {
                            last_err = format!("下载中断：{e}");
                            ok = false;
                            break;
                        }
                    }
                }
                use tokio::io::AsyncWriteExt;
                let _ = file.flush().await;
                if ok {
                    return Ok(());
                }
            }
            Ok(resp) => {
                last_err = format!("下载失败：HTTP {}", resp.status());
            }
            Err(e) => {
                last_err = format!("下载失败：{e}");
            }
        }
    }
    Err(if last_err.is_empty() {
        "下载失败".to_string()
    } else {
        last_err
    })
}

/// 启动 setup.exe 静默安装并立即退出当前进程。
/// Inno Setup 的 /VERYSILENT + /CLOSEAPPLICATIONS + /RESTARTAPPLICATIONS
/// 会自动关闭正在运行的旧 exe、覆盖安装、安装完成后重启应用。
pub fn run_installer_and_exit(setup_path: &Path) -> ! {
    let _ = std::process::Command::new(setup_path)
        .args([
            "/VERYSILENT",
            "/CLOSEAPPLICATIONS",
            "/RESTARTAPPLICATIONS",
            "/NOCANCEL",
            "/SP-",
        ])
        .spawn();
    std::process::exit(0);
}
