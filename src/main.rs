// 在 Windows 上以 GUI 子系统运行，双击 exe 时不再弹出命令行黑窗
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#![allow(non_snake_case)]

mod api;
mod app;
mod config;
mod updater;

use dioxus::desktop::{Config, WindowBuilder};
use dioxus::desktop::tao::window::Icon;

const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

fn build_window_icon() -> Option<Icon> {
    let img = image::load_from_memory(ICON_PNG).ok()?;
    let small = image::imageops::resize(
        &img.to_rgba8(),
        64,
        64,
        image::imageops::FilterType::Lanczos3,
    );
    let (w, h) = small.dimensions();
    Icon::from_rgba(small.into_raw(), w, h).ok()
}

fn msgbox(title: &str, msg: &str) {
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        let t: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
        let m: Vec<u16> = OsStr::new(msg).encode_wide().chain(std::iter::once(0)).collect();
        unsafe {
            extern "system" {
                fn MessageBoxW(h: isize, t: *const u16, c: *const u16, flags: u32) -> i32;
            }
            MessageBoxW(0, m.as_ptr(), t.as_ptr(), 0x10);
        }
    }
}

fn is_webview2_installed() -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use std::ffi::OsStr;
        unsafe {
            extern "system" {
                fn RegOpenKeyExW(
                    hKey: isize,
                    lpSubKey: *const u16,
                    ulOptions: u32,
                    samDesired: u32,
                    phkResult: *mut isize,
                ) -> i32;
                fn RegCloseKey(hKey: isize) -> i32;
            }
            const HKEY_LOCAL_MACHINE: isize = -2147483646i64 as isize;
            const HKEY_CURRENT_USER: isize = -2147483647i64 as isize;
            const KEY_READ: u32 = 0x20019;

            let key: Vec<u16> = OsStr::new("SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}")
                .encode_wide().chain(std::iter::once(0)).collect();

            for &root in &[HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
                let mut hkey: isize = 0;
                if RegOpenKeyExW(root, key.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
                    RegCloseKey(hkey);
                    return true;
                }
            }
        }
    }
    // 非 Windows 平台不需要 WebView2
    true
}

fn main() {
    // panic 时弹框，避免双击无反应
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };
        msgbox("AgnesStudio 运行错误", &msg);
        default_hook(info);
    }));

    if !is_webview2_installed() {
        msgbox(
            "AgnesStudio 启动失败",
            "未检测到 WebView2 运行时。\n\n请安装 Microsoft Edge WebView2 Runtime：\nhttps://go.microsoft.com/fwlink/p/?LinkId=2124703",
        );
        return;
    }

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run_app)) {
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown error".to_string()
            };
            msgbox("AgnesStudio 启动失败", &format!("{msg}\n\n请确认已安装 WebView2 运行时：\nhttps://go.microsoft.com/fwlink/p/?LinkId=2124703"));
        }
        _ => {}
    }
}

fn run_app() {
    let mut cfg = Config::new().with_window(WindowBuilder::new().with_title("AgnesStudio"));
    if let Some(icon) = build_window_icon() {
        cfg = cfg.with_icon(icon);
    }
    dioxus::LaunchBuilder::desktop()
        .with_cfg(cfg)
        .launch(app::App);
}
