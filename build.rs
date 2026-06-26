// build.rs — 把 assets/icon.rc（引用 icon.ico）中的 Windows 图标资源编译进 exe，
// 让资源管理器、任务栏、窗口标题栏都显示 AgnesStudio 图标。
//
// 工具链适配：
//   - GNU (x86_64-pc-windows-gnu)：用 mingw 的 windres 输出 COFF 目标文件，直接链接。
//   - MSVC (x86_64-pc-windows-msvc)：优先用 SDK 的 rc.exe 生成 .res（由 link.exe 直接消费）；
//     若没有 rc.exe 则尝试 windres 生成的 .res（MSVC link 也能接受）。
fn main() {
    println!("cargo:rerun-if-changed=assets/icon.rc");
    println!("cargo:rerun-if-changed=assets/icon.ico");

    #[cfg(target_os = "windows")]
    {
        let rc = std::path::Path::new("assets/icon.rc");
        if !rc.exists() {
            return;
        }
        let out_dir =
            std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR not set"));
        let target = std::env::var("TARGET").unwrap_or_default();

        if target.contains("pc-windows-gnu") {
            // GNU 工具链：windres 输出目标文件，用 gcc 链接
            let obj = out_dir.join("icon.o");
            let ok = std::process::Command::new("windres")
                .arg("-i")
                .arg("icon.rc")
                .arg("-o")
                .arg(&obj)
                .current_dir("assets")
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                println!("cargo:rustc-link-search=native={}", out_dir.display());
                println!("cargo:rustc-link-lib=static:+whole-archive=icon");
                return;
            }
        } else if target.contains("pc-windows-msvc") {
            // MSVC：先找 SDK 的 rc.exe 生成 .res
            let res = out_dir.join("icon.res");
            for exe in ["rc.exe", "llvm-rc.exe"] {
                let ok = std::process::Command::new(exe)
                    .arg("/fo")
                    .arg(&res)
                    .arg("icon.rc")
                    .current_dir("assets")
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if ok {
                    // 把 .res 作为对象传给链接器：通过 link-arg
                    println!("cargo:rustc-link-arg={}", res.display());
                    return;
                }
            }
            // 回退：用 mingw windres 生成 .res（MSVC link.exe 也能接受 .res）
            let ok = std::process::Command::new("windres")
                .arg("-i")
                .arg("icon.rc")
                .arg("-O")
                .arg("res")
                .arg("-o")
                .arg(&res)
                .current_dir("assets")
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                println!("cargo:rustc-link-arg={}", res.display());
                return;
            }
        }

        println!(
            "cargo:warning=无法编译图标资源（缺少 rc.exe/windres）；资源管理器图标未嵌入。"
        );
    }
}
