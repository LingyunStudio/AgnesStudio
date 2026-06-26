// 把 assets/icon.png 转成多分辨率 Windows ICO（assets/icon.ico）
// 供 build.rs 嵌入 exe 资源使用，让资源管理器/任务栏显示 exe 图标。
use std::fs;
use std::io::{Cursor, Write};

fn main() {
    let src = fs::read("assets/icon.png").expect("read assets/icon.png");
    let img = image::load_from_memory(&src).expect("decode png").to_rgba8();

    // 多尺寸：16/32/48/64/128/256，覆盖 Windows 各显示场景
    let sizes: [u32; 6] = [16, 32, 48, 64, 128, 256];
    let mut frames: Vec<(u32, Vec<u8>)> = Vec::new();
    for &s in &sizes {
        let f = image::imageops::resize(&img, s, s, image::imageops::FilterType::Lanczos3);
        frames.push((s, f.into_raw()));
    }

    let ico = build_ico(&frames);
    fs::write("assets/icon.ico", &ico).expect("write assets/icon.ico");
    println!("wrote assets/icon.ico ({} bytes)", ico.len());
}

// 构造符合 ICO( CUR ) 规范的多帧文件
fn build_ico(frames: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let count = frames.len() as u16;
    let mut cur = Cursor::new(Vec::new());
    // ICONDIR
    cur.write_all(&[0u8, 0, 1, 0]).unwrap(); // reserved, type=1(icon), count
    cur.write_all(&count.to_le_bytes()).unwrap();

    let mut offset = 6 + count as usize * 16;
    let mut entries: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    for (s, rgba) in frames {
        let (w, h) = if *s >= 256 { (0u8, 0u8) } else { (*s as u8, *s as u8) };
        let png = encode_png(rgba, *s, *s);
        let mut e = Vec::new();
        e.push(w);
        e.push(h);
        e.push(0); // palette
        e.push(0); // reserved
        e.extend_from_slice(&1u16.to_le_bytes()); // color planes
        e.extend_from_slice(&32u16.to_le_bytes()); // bpp
        e.extend_from_slice(&(png.len() as u32).to_le_bytes());
        e.extend_from_slice(&(offset as u32).to_le_bytes());
        let len = png.len();
        entries.push((e, png));
        offset += len;
    }
    for (e, _) in &entries {
        cur.write_all(e).unwrap();
    }
    for (_, png) in &entries {
        cur.write_all(png).unwrap();
    }
    cur.into_inner()
}

fn encode_png(rgba: &[u8], w: u32, h: u32) -> Vec<u8> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    let img = image::RgbaImage::from_raw(w, h, rgba.to_vec()).unwrap();
    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(img.as_raw(), w, h, image::ExtendedColorType::Rgba8)
        .unwrap();
    out
}
