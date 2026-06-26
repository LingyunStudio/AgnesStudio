// 在 Windows 上以 GUI 子系统运行，双击 exe 时不再弹出命令行黑窗
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod api;
mod app;
mod config;
mod updater;

use dioxus::desktop::{Config, WindowBuilder};
use dioxus::desktop::tao::window::Icon;

// 内联图标，确保无论从哪个工作目录启动 exe 都能拿到窗口图标
const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

fn build_window_icon() -> Option<Icon> {
    let img = image::load_from_memory(ICON_PNG).ok()?;
    // 缩到 64x64，避免某些显卡驱动对超大图标的限制
    let small = image::imageops::resize(
        &img.to_rgba8(),
        64,
        64,
        image::imageops::FilterType::Lanczos3,
    );
    let (w, h) = small.dimensions();
    Icon::from_rgba(small.into_raw(), w, h).ok()
}

fn main() {
    let mut cfg = Config::new().with_window(WindowBuilder::new().with_title("AgnesStudio"));
    if let Some(icon) = build_window_icon() {
        cfg = cfg.with_icon(icon);
    }
    dioxus::LaunchBuilder::desktop().with_cfg(cfg).launch(app::App);
}
