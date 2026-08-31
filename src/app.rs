#![allow(non_snake_case)]
use base64::Engine;
use dioxus::prelude::*;
use std::cmp;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, mpsc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::api;
use crate::config;
use crate::i18n::{self, bi, Bi, Lang};
use crate::theme::{self, ThemeMode};
use crate::updater::{self, UpdateInfo};

// (模型 ID, 双语名称)
const MODELS: &[(&str, Bi)] = &[
    ("agnes-image-2.1-flash", bi("Agnes Image 2.1 Flash（默认）", "Agnes Image 2.1 Flash (default)")),
    ("agnes-image-2.0-flash", bi("Agnes Image 2.0 Flash", "Agnes Image 2.0 Flash")),
];
// (双语名称, 尺寸值)，最后一项为自定义
const SIZE_PRESETS: &[(Bi, &str)] = &[
    (bi("1024 × 1024（方形）", "1024 × 1024 (Square)"), "1024x1024"),
    (bi("1024 × 768（横版）", "1024 × 768 (Landscape)"), "1024x768"),
    (bi("768 × 1024（竖版）", "768 × 1024 (Portrait)"), "768x1024"),
    (bi("1280 × 720（HD 横）", "1280 × 720 (HD Landscape)"), "1280x720"),
    (bi("720 × 1280（HD 竖）", "720 × 1280 (HD Portrait)"), "720x1280"),
    (bi("1920 × 1080（FHD 横）", "1920 × 1080 (FHD Landscape)"), "1920x1080"),
    (bi("1080 × 1920（FHD 竖）", "1080 × 1920 (FHD Portrait)"), "1080x1920"),
    (bi("2048 × 2048（2K 方）", "2048 × 2048 (2K Square)"), "2048x2048"),
    (bi("2560 × 1440（2K 横）", "2560 × 1440 (2K Landscape)"), "2560x1440"),
    (bi("3840 × 2160（4K 横）", "3840 × 2160 (4K Landscape)"), "3840x2160"),
    (bi("2160 × 3840（4K 竖）", "2160 × 3840 (4K Portrait)"), "2160x3840"),
    (bi("自定义", "Custom"), ""),
];

// 2.1 Flash 档位式尺寸：size 档位 × ratio 宽高比（2.0 Flash 仍用精确尺寸）
const IMG_TIERS: &[&str] = &["1K", "2K", "3K", "4K"];
// (ratio, [1K, 2K, 3K, 4K 对应的精确像素)
const IMG_TIER_SIZES: &[(&str, [&str; 4])] = &[
    ("1:1",  ["1024x1024", "2048x2048", "3072x3072", "4096x4096"]),
    ("3:4",  ["864x1152",  "1728x2304", "2592x3456", "3456x4608"]),
    ("4:3",  ["1152x864",  "2304x1728", "3456x2592", "4608x3456"]),
    ("16:9", ["1312x736",  "2624x1472", "3936x2208", "5248x2944"]),
    ("9:16", ["736x1312",  "1472x2624", "2208x3936", "2944x5248"]),
    ("2:3",  ["832x1248",  "1664x2496", "2496x3744", "3328x4992"]),
    ("3:2",  ["1248x832",  "2496x1664", "3744x2496", "4992x3328"]),
    ("21:9", ["1568x672",  "3136x1344", "4704x2016", "6272x2688"]),
];

// ── 视频 ──────────────────────────────────────────────────────────────────────
// (双语名称, 宽, 高)，最后一项为自定义
const VIDEO_SIZE_PRESETS: &[(Bi, i32, i32)] = &[
    (bi("16:9 横版（1280×720）", "16:9 Landscape (1280×720)"), 1280, 720),
    (bi("9:16 竖版（720×1280）", "9:16 Portrait (720×1280)"), 720, 1280),
    (bi("1:1 方形（720×720）", "1:1 Square (720×720)"), 720, 720),
    (bi("4:3 横版（1024×768）", "4:3 Landscape (1024×768)"), 1024, 768),
    (bi("3:4 竖版（768×1024）", "3:4 Portrait (768×1024)"), 768, 1024),
    (bi("自定义", "Custom"), 0, 0),
];
// (双语名称, num_frames, frame_rate)
const VIDEO_DURATION_PRESETS: &[(Bi, i32, i32)] = &[
    (bi("约 3 秒（81 帧）", "~3s (81 frames)"), 81, 24),
    (bi("约 5 秒（121 帧）", "~5s (121 frames)"), 121, 24),
    (bi("约 10 秒（241 帧）", "~10s (241 frames)"), 241, 24),
    (bi("约 18 秒（441 帧）", "~18s (441 frames)"), 441, 24),
];
const VIDEO_MODELS: &[(&str, Bi)] = &[
    ("agnes-video-2.5-flash", bi("Agnes Video 2.5 Flash（限时免费）", "Agnes Video 2.5 Flash (free)")),
    ("agnes-video-2.5", bi("Agnes Video 2.5", "Agnes Video 2.5")),
    ("agnes-video-v2.0", bi("Agnes Video V2.0", "Agnes Video V2.0")),
];
// Video 2.5：画幅比例（size 固定 720P），(双语名称, ratio)
const V25_AR_PRESETS: &[(Bi, &str)] = &[
    (bi("16:9 横版（1280×720，默认）", "16:9 Landscape (1280×720, default)"), "16:9"),
    (bi("9:16 竖屏（720×1280）", "9:16 Portrait (720×1280)"), "9:16"),
    (bi("21:9 超宽（1680×720）", "21:9 Ultrawide (1680×720)"), "21:9"),
    (bi("4:3 横版（960×720）", "4:3 Landscape (960×720)"), "4:3"),
    (bi("1:1 方形（720×720）", "1:1 Square (720×720)"), "1:1"),
    (bi("3:4 竖版（720×960）", "3:4 Portrait (720×960)"), "3:4"),
];
// Video 2.5：时长 "4"–"12" 秒（API 要求字符串）
const V25_SECONDS: &[&str] = &["4", "5", "6", "7", "8", "9", "10", "11", "12"];

#[derive(Clone,Copy,PartialEq)]
enum Workspace{Image,Video}
#[derive(Clone,Copy,PartialEq)]
enum VMode{Text,Image,Multi,Keyframes}
/// Video 2.5 的生成模式
#[derive(Clone,Copy,PartialEq)]
enum V25Mode{Text,Keyframe,Reference}

#[derive(Clone,PartialEq)]
enum Mode{Text,Image}
#[derive(Clone,PartialEq)]
enum InputSrc{File,Url}
#[derive(Clone,PartialEq)]
enum OutFmt{Url,B64}
/// 底部通知的语义类型（渲染时映射到主题色）
#[derive(Clone,Copy,PartialEq)]
enum NoticeKind{Ok,Err,Info}
#[derive(Clone,PartialEq)]
struct Notice{kind:NoticeKind,text:String}
enum BgEvent{
    ImageDone{bytes:Vec<u8>,url:Option<String>,prompt:String,model:String,size:String},
    Error(String),FilePicked(Option<(String,String)>),DirPicked(Option<String>),
    VideoCreated{video_id:String,task_id:String,seconds:String,size:String,model:String},
    VideoStatus{done:bool,failed:bool,progress:f32,message:String,seconds:String,size:String,transient:bool},
    VideoReady{bytes:Vec<u8>,video_url:String,prompt:String,model:String,seconds:String,size:String},
    VideoSaved{index:usize,path:String},
    VideoSaveFailed{index:usize,error:String},
    // 自动更新
    UpdateFound(UpdateInfo),
    UpdateNone,
    UpdateCheckFailed(String),
    UpdateDownloadProgress(u32),
    UpdateDownloadDone(String),
    UpdateDownloadFailed(String),
}
#[derive(Clone)]
struct EventTx(mpsc::Sender<BgEvent>);
impl PartialEq for EventTx{fn eq(&self,_:&Self)->bool{true}}
#[derive(Clone,PartialEq)]
struct CachedImage{data_uri:String,url:Option<String>,prompt:String,model:String,size:String,dims:[usize;2],raw_bytes:Vec<u8>}

#[derive(Clone,PartialEq)]
struct CachedVideo{
    video_url:String,bytes:Vec<u8>,data_uri:String,prompt:String,model:String,size:String,seconds:String,
}
#[derive(Clone,PartialEq)]
struct VideoJob{video_id:String,task_id:String,prompt:String,model:String}

struct AppState{
    cfg:config::Config,images:Vec<CachedImage>,selected:usize,loading:bool,
    error:String,notice:Option<Notice>,
    prompt:String,mode:Mode,out_fmt:OutFmt,model_index:usize,size_preset_index:usize,
    custom_w:i32,custom_h:i32,input_src:InputSrc,input_url:String,input_file:Option<(String,String)>,
    api_key_visible:bool,show_popup:bool,popup_uri:String,popup_dims:[usize;2],
    popup_zoom:f32,popup_pan:[f32;2],
    gen_elapsed:f32,bg_tx:EventTx,bg_rx:Arc<Mutex<mpsc::Receiver<BgEvent>>>,
    // 2.1 Flash 档位式尺寸
    tier_index:usize,ratio_index:usize,
    // 视频工作台
    workspace:Workspace,
    videos:Vec<CachedVideo>,video_selected:usize,
    video_loading:bool,video_error:String,video_elapsed:f32,
    video_progress:f32,video_msg:String,video_job:Option<VideoJob>,
    video_prompt:String,video_neg:String,
    vmodel_index:usize,
    vsize_index:usize,vw_custom:i32,vh_custom:i32,
    vduration_index:usize,vframes_custom:i32,vfps_custom:i32,
    vmode:VMode,video_image_urls:Vec<String>,video_url_input:String,
    // Video 2.5 参数
    v25_mode:V25Mode,v25_seconds_index:usize,v25_ar_index:usize,
    v25_first:String,v25_last:String,
    v25_images:Vec<String>,v25_audios:Vec<String>,v25_videos:Vec<String>,v25_input:String,
    video_store:Arc<Mutex<HashMap<usize,Arc<Vec<u8>>>>>,
    reset_token:u32,
    // 自动更新
    update_info:Option<UpdateInfo>,
    update_checking:bool,
    update_uptodate:bool,
    update_downloading:bool,
    update_progress:u32,
    update_error:String,
    show_update_dialog:bool,
    // 设置弹窗
    show_settings:bool,
    // 界面偏好
    theme:ThemeMode,
    lang:Lang,
}

fn raw_to_data_uri(b:&[u8])->Result<String,String>{
    let img=image::load_from_memory(b).map_err(|e|format!("decode failed: {e}"))?;
    let rgba=img.to_rgba8();let mut out=std::io::Cursor::new(Vec::new());
    image::write_buffer_with_format(&mut out,&rgba,img.width(),img.height(),image::ExtendedColorType::Rgba8,image::ImageFormat::Png).map_err(|e|format!("encode failed: {e}"))?;
    let b64=base64::engine::general_purpose::STANDARD.encode(out.into_inner());
    Ok(format!("data:image/png;base64,{b64}"))
}
/// 当前图片模型是否为 2.1 Flash（支持档位式 size + ratio）
fn img_is_21(s:&AppState)->bool{MODELS[s.model_index].0=="agnes-image-2.1-flash"}
fn resolved_size(s:&AppState)->String{
    if img_is_21(s){
        IMG_TIERS[s.tier_index.min(IMG_TIERS.len()-1)].to_string()
    }else if s.size_preset_index<SIZE_PRESETS.len()-1{SIZE_PRESETS[s.size_preset_index].1.to_string()}else{format!("{}x{}",s.custom_w,s.custom_h)}
}
/// 2.1 Flash 的 ratio 参数；2.0 Flash 不支持，返回 None
fn resolved_ratio(s:&AppState)->Option<String>{
    if img_is_21(s){Some(IMG_TIER_SIZES[s.ratio_index.min(IMG_TIER_SIZES.len()-1)].0.to_string())}else{None}
}
/// 2.1 Flash 当前档位+比例对应的精确像素（仅展示用）
fn tier_exact_size(s:&AppState)->&'static str{
    let r=IMG_TIER_SIZES[s.ratio_index.min(IMG_TIER_SIZES.len()-1)];
    r.1[s.tier_index.min(IMG_TIERS.len()-1)]
}
/// 在档位表里查找精确尺寸（迁移旧配置用）：找到返回 (tier_idx, ratio_idx)
fn find_tier_combo(exact:&str)->Option<(usize,usize)>{
    for(ri,(_,sizes))in IMG_TIER_SIZES.iter().enumerate(){
        for(ti,sz)in sizes.iter().enumerate(){
            if*sz==exact{return Some((ti,ri));}
        }
    }
    None
}
fn cur_input(s:&AppState)->Option<String>{
    if s.mode!=Mode::Image{return None}
    match s.input_src{InputSrc::File=>s.input_file.as_ref().map(|(_,d)|d.clone()),InputSrc::Url=>{let t=s.input_url.trim();if t.is_empty(){None}else{Some(t.to_string())}}}
}
fn set_defaults(s:&mut AppState){
    s.prompt=s.cfg.last_prompt.clone();s.mode=if s.cfg.mode=="image"{Mode::Image}else{Mode::Text};
    s.out_fmt=if s.cfg.output_format=="b64_json"{OutFmt::B64}else{OutFmt::Url};
    s.model_index=MODELS.iter().position(|(id,_)|*id==s.cfg.model.as_str()).unwrap_or(0);
    s.theme=ThemeMode::from_cfg(&s.cfg.theme);s.lang=Lang::from_cfg(&s.cfg.lang);
    if img_is_21(s){
        // 优先用保存的档位/比例；没存过则尝试把旧精确尺寸映射到档位表
        let t=IMG_TIERS.iter().position(|t|*t==s.cfg.image_tier.as_str());
        let r=IMG_TIER_SIZES.iter().position(|(rt,_)|*rt==s.cfg.image_ratio.as_str());
        match(t,r){
            (Some(ti),Some(ri))=>{s.tier_index=ti;s.ratio_index=ri;}
            _=>{
                if let Some((ti,ri))=find_tier_combo(s.cfg.last_size.as_str()){s.tier_index=ti;s.ratio_index=ri;}
            }
        }
    }
    let sz=s.cfg.last_size.clone();
    if let Some(idx)=SIZE_PRESETS.iter().position(|(_,v)|*v==sz.as_str()){s.size_preset_index=idx;}
    else if let Some((w,h))=sz.split_once('x'){if let(Ok(w),Ok(h))=(w.trim().parse::<i32>(),h.trim().parse::<i32>()){s.custom_w=w.clamp(64,4096);s.custom_h=h.clamp(64,4096);s.size_preset_index=SIZE_PRESETS.len()-1;}}
    s.reset_token=s.reset_token.wrapping_add(1);
}
fn do_save(s:&mut AppState){
    if s.images.is_empty(){return}
    let sel=cmp::min(s.selected,s.images.len()-1);let raw=s.images[sel].raw_bytes.clone();
    let ex=match image::guess_format(&raw).unwrap_or(image::ImageFormat::Png){image::ImageFormat::Png=>"png",image::ImageFormat::Jpeg=>"jpg",image::ImageFormat::WebP=>"webp",image::ImageFormat::Bmp=>"bmp",_=>"png"};
    let secs=SystemTime::now().duration_since(UNIX_EPOCH).map(|d|d.as_secs()).unwrap_or(0);
    let fname=format!("agnes_{secs}.{ex}");let dir=PathBuf::from(&s.cfg.save_dir);
    if std::fs::create_dir_all(&dir).is_err(){s.notice(NoticeKind::Err,i18n::t(s.lang,"nt.mkdir").to_string());return;}
    let path=dir.join(&fname);
    match std::fs::write(&path,&raw){
        Ok(_)=>s.notice(NoticeKind::Ok,i18n::tf(s.lang,"nt.saved",&[("p",&path.display().to_string())])),
        Err(e)=>s.notice(NoticeKind::Err,i18n::tf(s.lang,"nt.savefail",&[("e",&e.to_string())])),
    }
}
/// 通知类型 -> CSS 类
fn nt_class(k:NoticeKind)->&'static str{
    match k{NoticeKind::Ok=>"nt nt-ok",NoticeKind::Err=>"nt nt-err",NoticeKind::Info=>"nt nt-info"}
}

fn render_markdown(md: &str) -> String {
    let parser = pulldown_cmark::Parser::new(md);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

fn open_url(url:&str){
    #[cfg(windows)]{
    use std::os::windows::process::CommandExt;
    // CREATE_NO_WINDOW = 0x08000000，避免 cmd 弹出空白控制台窗口
    let _=std::process::Command::new("cmd").args(["/C","start","",url]).creation_flags(0x0800_0000).spawn();
    }
    #[cfg(not(windows))]{let _=std::process::Command::new("xdg-open").arg(url).spawn();}
}

// 通过 dioxus 资源协议提供视频字节，支持 Range 请求以供 <video> 流式播放/拖动进度
fn serve_video_asset(
    store:&Arc<Mutex<HashMap<usize,Arc<Vec<u8>>>>>,
    req:&dioxus::desktop::wry::http::Request<Vec<u8>>,
    responder:dioxus::desktop::wry::RequestAsyncResponder,
){
    use dioxus::desktop::wry::http::{Response,StatusCode,header};
    // 路径形如 /video/{id}
    let id=req.uri().path().split('/').nth(2).and_then(|s|s.parse::<usize>().ok());
    let bytes=match(id,store.lock().ok()){
        (Some(i),Some(g))=>g.get(&i).cloned(),
        _=>None,
    };
    let bytes=match bytes{
        Some(b)=>b,
        None=>{let r=Response::builder().status(StatusCode::NOT_FOUND).body(Vec::new()).unwrap();return responder.respond(r);}
    };
    let total=bytes.len();
    if total==0{
        let r=Response::builder().status(StatusCode::NOT_FOUND).body(Vec::new()).unwrap();
        return responder.respond(r);
    }
    // 解析 Range: bytes=start-end
    let range=req.headers().get(header::RANGE).and_then(|v|v.to_str().ok()).map(|s|s.to_string());
    let has_range=range.is_some();
    let(start,end)=if let Some(ref rh)=range{
        if let Some(spec)=rh.strip_prefix("bytes="){
            let mut parts=spec.splitn(2,'-');
            let s_start=parts.next().unwrap_or("");
            let s_end=parts.next().unwrap_or("");
            let start: usize=if s_start.is_empty(){0}else{s_start.parse().unwrap_or(0)};
            let end: usize=if s_end.is_empty(){total-1}else{s_end.parse().unwrap_or(total-1).min(total-1)};
            (start.min(total-1),end)
        }else{(0,total-1)}
    }else{(0,total-1)};
    let start=start.min(total-1);
    let end=end.min(total-1).max(start);
    let chunk=bytes[start..=end].to_vec();
    let mut builder=Response::builder()
        .header(header::CONTENT_TYPE,"video/mp4")
        .header(header::ACCEPT_RANGES,"bytes")
        .header(header::CONTENT_LENGTH,chunk.len().to_string());
    if has_range{
        builder=builder.status(StatusCode::PARTIAL_CONTENT)
            .header(header::CONTENT_RANGE,format!("bytes {start}-{end}/{total}"));
    }else{
        builder=builder.status(StatusCode::OK);
    }
    let r=builder.body(chunk).unwrap();
    responder.respond(r);
}

// 提供 App 图标 PNG 字节，供顶栏 <img src="/icon"> 使用
fn serve_icon_asset(
    _req:&dioxus::desktop::wry::http::Request<Vec<u8>>,
    responder:dioxus::desktop::wry::RequestAsyncResponder,
){
    use dioxus::desktop::wry::http::{Response,StatusCode,header};
    static ICON: &[u8] = include_bytes!("../assets/icon.png");
    let r=Response::builder()
        .header(header::CONTENT_TYPE,"image/png")
        .header(header::CACHE_CONTROL,"max-age=86400")
        .header(header::CONTENT_LENGTH,ICON.len().to_string())
        .status(StatusCode::OK)
        .body(ICON.to_vec())
        .unwrap();
    responder.respond(r);
}

// ── 视频辅助 ──────────────────────────────────────────────────────────────────
fn video_dims(s:&AppState)->(i32,i32){
    if s.vsize_index<VIDEO_SIZE_PRESETS.len()-1{
        let(_,w,h)=VIDEO_SIZE_PRESETS[s.vsize_index];(w,h)
    }else{(s.vw_custom.clamp(64,4096),s.vh_custom.clamp(64,4096))}
}
fn video_frames(s:&AppState)->i32{
    if s.vduration_index<VIDEO_DURATION_PRESETS.len(){VIDEO_DURATION_PRESETS[s.vduration_index].1}
    else{s.vframes_custom.clamp(1,441)}
}
fn video_fps(s:&AppState)->i32{
    if s.vduration_index<VIDEO_DURATION_PRESETS.len(){VIDEO_DURATION_PRESETS[s.vduration_index].2}
    else{s.vfps_custom.clamp(1,60)}
}
fn video_seconds(s:&AppState)->String{
    let fps=video_fps(s) as f32;if fps<=0.0{return "0".to_string();}
    format!("{:.1}",video_frames(s) as f32/fps)
}
fn do_save_video(s:&mut AppState){
    if s.videos.is_empty(){return}
    let sel=cmp::min(s.video_selected,s.videos.len()-1);
    let entry=s.videos[sel].clone();
    let dir=PathBuf::from(&s.cfg.save_dir);
    if std::fs::create_dir_all(&dir).is_err(){s.notice(NoticeKind::Err,i18n::t(s.lang,"nt.mkdir").to_string());return;}

    // 缓存字节有效则直接写盘
    if is_valid_mp4(&entry.bytes){
        let secs=SystemTime::now().duration_since(UNIX_EPOCH).map(|d|d.as_secs()).unwrap_or(0);
        let path=dir.join(format!("agnes_video_{secs}.mp4"));
        match std::fs::write(&path,&entry.bytes){
            Ok(_)=>s.notice(NoticeKind::Ok,i18n::tf(s.lang,"nt.saved",&[("p",&path.display().to_string())])),
            Err(e)=>s.notice(NoticeKind::Err,i18n::tf(s.lang,"nt.savefail",&[("e",&e.to_string())])),
        }
        return;
    }

    // 缓存字节无效（可能下载时被 CDN 返回错误页）：用远程 URL 重新下载
    if entry.video_url.is_empty(){s.notice(NoticeKind::Err,i18n::t(s.lang,"nt.novurl").to_string());return;}
    s.notice(NoticeKind::Info,i18n::t(s.lang,"nt.redl").to_string());
    let url=entry.video_url.clone();let key=s.cfg.api_key.clone();let dir2=dir.clone();let idx=sel;
    let tx=s.bg_tx.0.clone();
    std::thread::spawn(move||{
        let rt=tokio::runtime::Runtime::new().expect("rt");
        match rt.block_on(api::download_video(&key,&url)){
            Ok(b)=>{
                let secs=SystemTime::now().duration_since(UNIX_EPOCH).map(|d|d.as_secs()).unwrap_or(0);
                let path=dir2.join(format!("agnes_video_{secs}.mp4"));
                match std::fs::write(&path,&b){Ok(_)=>{let _=tx.send(BgEvent::VideoSaved{index:idx,path:path.display().to_string()});},Err(e)=>{let _=tx.send(BgEvent::VideoSaveFailed{index:idx,error:e.to_string()});}}
            }
            Err(e)=>{let _=tx.send(BgEvent::VideoSaveFailed{index:idx,error:e});}
        }
    });
}
fn set_video_defaults(s:&mut AppState){
    s.video_prompt=s.cfg.last_video_prompt.clone();
    s.video_neg=s.cfg.video_neg_prompt.clone();
    s.vmodel_index=VIDEO_MODELS.iter().position(|(id,_)|*id==s.cfg.video_model.as_str()).unwrap_or(0);
    s.vsize_index=VIDEO_SIZE_PRESETS.iter().position(|(_,w,h)|*w==s.cfg.video_width&&*h==s.cfg.video_height).unwrap_or(0);
    if s.vsize_index==VIDEO_SIZE_PRESETS.len()-1{s.vw_custom=s.cfg.video_width.clamp(64,4096);s.vh_custom=s.cfg.video_height.clamp(64,4096);}
    s.vduration_index=s.cfg.video_duration_preset.min(VIDEO_DURATION_PRESETS.len()-1);
    s.vframes_custom=s.cfg.video_num_frames;s.vfps_custom=s.cfg.video_frame_rate;
    s.vmode=match s.cfg.video_mode.as_str(){"image"=>VMode::Image,"multi"=>VMode::Multi,"keyframes"=>VMode::Keyframes,_=>VMode::Text};
    // Video 2.5
    s.v25_mode=match s.cfg.video25_mode.as_str(){"keyframe"=>V25Mode::Keyframe,"reference"=>V25Mode::Reference,_=>V25Mode::Text};
    s.v25_seconds_index=V25_SECONDS.iter().position(|x|*x==s.cfg.video25_seconds.as_str()).unwrap_or(1);
    s.v25_ar_index=V25_AR_PRESETS.iter().position(|(_,ar)|*ar==s.cfg.video25_ar.as_str()).unwrap_or(0);
    s.v25_first=s.cfg.video25_first_frame.clone();
    s.v25_last=s.cfg.video25_last_frame.clone();
    s.reset_token=s.reset_token.wrapping_add(1);
}
/// 当前选择的视频模型 ID
fn cur_video_model(s:&AppState)->&'static str{VIDEO_MODELS[s.vmodel_index.min(VIDEO_MODELS.len()-1)].0}
/// 是否为 2.5 系模型（2.5 / 2.5 Flash，共用同一套 mode/seconds/aspect_ratio 参数）
fn is_video25(s:&AppState)->bool{matches!(cur_video_model(s),api::MODEL_VIDEO_V25|api::MODEL_VIDEO_V25_FLASH)}
/// 是否为 2.5 Flash（图片参考最多 5 张、不支持视频参考）
fn is_video25_flash(s:&AppState)->bool{cur_video_model(s)==api::MODEL_VIDEO_V25_FLASH}

// 视频字节是否为有效 mp4（4~8 字节含 "ftyp"）
fn is_valid_mp4(b:&[u8])->bool{
    b.len()>12&&(&b[4..8]==b"ftyp")&&b[0]==0&&b[1]==0&&b[2]==0
}

impl AppState{fn notice(&mut self,kind:NoticeKind,text:String){self.notice=Some(Notice{kind,text});}}

// 手动触发检查更新（设置卡片按钮）：检查中显示状态，失败显示错误
fn CheckUpdate(mut st:Signal<AppState>){
    if st.read().update_checking{return;}
    st.write().update_checking=true;
    st.write().update_error.clear();
    st.write().update_uptodate=false;
    let tx=st.read().bg_tx.0.clone();
    tokio::spawn(async move{
        match updater::check_latest().await{
            Ok(Some(info))=>{let _=tx.send(BgEvent::UpdateFound(info));}
            Ok(None)=>{let _=tx.send(BgEvent::UpdateNone);}
            Err(e)=>{let _=tx.send(BgEvent::UpdateCheckFailed(e));}
        }
    });
}

// 开始下载并安装更新：后台下载 setup.exe，进度通过 bg_tx 回传
fn StartUpdate(mut st:Signal<AppState>){
    let info=match st.read().update_info.clone(){Some(i)=>i,None=>return};
    let url=match info.setup_url.clone(){Some(u)=>u,None=>{ // 无安装包资源，引导浏览器
        open_url(&info.html_url);return;
    }};
    st.write().update_downloading=true;
    st.write().update_error.clear();
    st.write().update_progress=0;
    let tx=st.read().bg_tx.0.clone();
    tokio::spawn(async move{
        // 临时文件：%TEMP%\agnes-studio-update.exe
        let tmp=std::env::temp_dir().join("agnes-studio-update.exe");
        let tx2=tx.clone();
        let dest=tmp.clone();
        let res=updater::download_setup(&url,&dest,move|got,total|{
            if total>0{let p=((got as f64/total as f64)*100.0) as u32;let _=tx2.send(BgEvent::UpdateDownloadProgress(p.min(99)));}
        }).await;
        match res{
            Ok(())=>{let _=tx.send(BgEvent::UpdateDownloadDone(dest.to_string_lossy().to_string()));}
            Err(e)=>{let _=tx.send(BgEvent::UpdateDownloadFailed(e));}
        }
    });
}

// ── Root ───────────────────────────────────────────────────────────────────────

#[component]
pub fn App()->Element{
    let(bg_tx,bg_rx)=mpsc::channel::<BgEvent>();
    let bg_rx=Arc::new(Mutex::new(bg_rx));let cfg=config::load();
    let mut init=AppState{cfg,images:vec![],selected:0,loading:false,error:String::new(),notice:None,prompt:String::new(),mode:Mode::Text,out_fmt:OutFmt::Url,model_index:0,size_preset_index:0,custom_w:1024,custom_h:1024,input_src:InputSrc::File,input_url:String::new(),input_file:None,api_key_visible:false,show_popup:false,popup_uri:String::new(),popup_dims:[0,0],popup_zoom:1.0,popup_pan:[0.0,0.0],gen_elapsed:0.0,bg_tx:EventTx(bg_tx),bg_rx,workspace:Workspace::Image,tier_index:0,ratio_index:0,videos:vec![],video_selected:0,video_loading:false,video_error:String::new(),video_elapsed:0.0,video_progress:0.0,video_msg:String::new(),video_job:None,video_prompt:String::new(),video_neg:String::new(),vmodel_index:0,vsize_index:0,vw_custom:1152,vh_custom:768,vduration_index:1,vframes_custom:121,vfps_custom:24,vmode:VMode::Text,video_image_urls:vec![],video_url_input:String::new(),v25_mode:V25Mode::Text,v25_seconds_index:1,v25_ar_index:0,v25_first:String::new(),v25_last:String::new(),v25_images:vec![],v25_audios:vec![],v25_videos:vec![],v25_input:String::new(),video_store:Arc::new(Mutex::new(HashMap::new())),
        update_info:None,update_checking:false,update_uptodate:false,update_downloading:false,update_progress:0,update_error:String::new(),show_update_dialog:false,show_settings:false,reset_token:0,theme:ThemeMode::System,lang:Lang::Zh};
    set_defaults(&mut init);set_video_defaults(&mut init);let st=use_signal(||init);
    let css=use_hook(||theme::css());

    {let mut s2=st.clone();use_future(move||async move{loop{let evs={let state=s2.read();let rx=state.bg_rx.clone();let evs=rx.lock().unwrap().try_iter().collect::<Vec<_>>();evs};for ev in evs{let mut s=s2.write();match ev{
        BgEvent::ImageDone{bytes,url,prompt,model,size}=>{s.loading=false;match raw_to_data_uri(&bytes){Ok(uri)=>match image::load_from_memory(&bytes){Ok(img)=>{s.images.push(CachedImage{data_uri:uri,url,prompt,model,size,dims:[img.width()as usize,img.height()as usize],raw_bytes:bytes});s.selected=s.images.len()-1;s.error.clear();},Err(e)=>s.error=format!("decode failed: {e}")},Err(e)=>s.error=e}}
        BgEvent::Error(e)=>{s.loading=false;s.error=e}
        BgEvent::FilePicked(f)=>{s.input_file=f}
        BgEvent::DirPicked(d)=>{if let Some(d)=d{s.cfg.save_dir=d}}
        BgEvent::VideoCreated{video_id,task_id,seconds,size,model}=>{
            s.video_job=Some(VideoJob{video_id,task_id,prompt:s.video_prompt.clone(),model});
            s.video_progress=1.0;s.video_msg=i18n::t(s.lang,"vs.created").to_string();
            s.video_error.clear();
            let _=(seconds,size);
        }
        BgEvent::VideoStatus{done,failed,progress,message,seconds,size,transient}=>{
            let L=s.lang;
            // 后端状态码（queued/in_progress）与空失败信息按当前语言本地化
            let message=if failed&&message.is_empty(){
                i18n::t(L,"vs.fail").to_string()
            }else{
                match message.as_str(){
                    "queued"=>i18n::t(L,"vs.queued").to_string(),
                    "in_progress"=>i18n::t(L,"vs.running").to_string(),
                    m=>m.to_string(),
                }
            };
            s.video_progress=progress;s.video_msg=message.clone();
            let _=(seconds,size);
            if transient{
                // 暂时性错误（429 限流 / 网络抖动）：保持轮询，不中断任务
                s.video_error=message.clone();
            }else if failed{
                s.video_loading=false;s.video_error=message.clone();
            }else{
                // 正常进度 / 完成：清除临时错误
                s.video_error.clear();
                if done{s.video_msg=i18n::t(L,"vs.dldone").to_string();}
            }
        }
        BgEvent::VideoReady{bytes,video_url,prompt,model,seconds,size}=>{
            s.video_loading=false;s.video_error.clear();
            let idx=s.videos.len();
            let arc=Arc::new(bytes);
            if let Ok(mut g)=s.video_store.lock(){g.insert(idx,arc.clone());}
            s.videos.push(CachedVideo{video_url,bytes:arc.to_vec(),data_uri:String::new(),prompt,model,size,seconds});
            s.video_selected=s.videos.len()-1;
            s.video_progress=100.0;s.video_msg=i18n::t(s.lang,"vs.done").to_string();
        }
        BgEvent::VideoSaved{index,path}=>{
            // 重新下载成功后，顺便用有效字节更新缓存
            if let Ok(b)=std::fs::read(&path){if is_valid_mp4(&b)&&index<s.videos.len(){s.videos[index].bytes=b;}}
            let msg=i18n::tf(s.lang,"nt.saved",&[("p",&path)]);
            s.notice(NoticeKind::Ok,msg);
        }
        BgEvent::VideoSaveFailed{index,error}=>{
            let _=index;
            let msg=i18n::tf(s.lang,"nt.savefail",&[("e",&error)]);
            s.notice(NoticeKind::Err,msg);
        }
        BgEvent::UpdateFound(info)=>{s.update_checking=false;s.update_uptodate=false;s.update_info=Some(info);s.show_update_dialog=true;}
        BgEvent::UpdateNone=>{s.update_checking=false;s.update_uptodate=true;s.update_error.clear();}
        BgEvent::UpdateCheckFailed(e)=>{s.update_checking=false;s.update_uptodate=false;s.update_error=e;}
        BgEvent::UpdateDownloadProgress(p)=>{s.update_progress=p;}
        BgEvent::UpdateDownloadDone(path)=>{
            s.update_downloading=false;
            // 下载完成，触发静默安装并退出（不会返回）
            let p=PathBuf::from(&path);
            updater::run_installer_and_exit(&p);
        }
        BgEvent::UpdateDownloadFailed(e)=>{
            s.update_downloading=false;s.update_error=e;
        }
    }}tokio::time::sleep(std::time::Duration::from_millis(50)).await;}});}
    {let mut s2=st.clone();use_future(move||async move{loop{tokio::time::sleep(std::time::Duration::from_millis(100)).await;let mut s=s2.write();if s.loading{s.gen_elapsed+=0.1;}if s.video_loading{s.video_elapsed+=0.1;}}});}
    // 启动后延迟 3 秒静默检查更新（失败不打扰用户）
    {let s2=st.clone();use_future(move||async move{
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        match updater::check_latest().await{
            Ok(Some(info))=>{let _=s2.read().bg_tx.0.send(BgEvent::UpdateFound(info));}
            Ok(None)=>{let _=s2.read().bg_tx.0.send(BgEvent::UpdateNone);}
            Err(_)=>{} // 启动期静默失败，不弹错
        }
    });}
    // 视频任务轮询 + 下载（渐进退避，429/网络错误视为暂时性，不中断）
    {let mut s2=st.clone();use_future(move||async move{
        let mut interval=5u64;     // 起始 5 秒
        let mut fails=0u32;
        loop{
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            let(job,key,loading,prompt)={let s=s2.read();(s.video_job.clone(),s.cfg.api_key.clone(),s.video_loading,s.video_prompt.clone())};
            if loading&&job.is_some(){
                let j=job.unwrap();
                match api::fetch_video_status(&key,&j.video_id,&j.task_id,&j.model).await{
                    Ok(st_)=>{
                        // 成功拿到状态：重置退避
                        interval=5;fails=0;
                        if st_.failed{
                            let _=s2.write().bg_tx.0.send(BgEvent::VideoStatus{done:false,failed:true,progress:st_.progress,message:st_.message,seconds:st_.seconds,size:st_.size,transient:false});
                        }else if st_.done{
                            if let Some(url)=st_.video_url{
                                let L=s2.read().lang;
                                let _=s2.write().bg_tx.0.send(BgEvent::VideoStatus{done:true,failed:false,progress:100.0,message:i18n::t(L,"vs.dl").to_string(),seconds:st_.seconds.clone(),size:st_.size.clone(),transient:false});
                                match api::download_video(&key,&url).await{
                                    Ok(b)=>{let _=s2.write().bg_tx.0.send(BgEvent::VideoReady{bytes:b,video_url:url,prompt,model:j.model.clone(),seconds:st_.seconds,size:st_.size});}
                                    Err(e)=>{let _=s2.write().bg_tx.0.send(BgEvent::VideoStatus{done:false,failed:true,progress:0.0,message:e,seconds:String::new(),size:String::new(),transient:false});}
                                }
                            }else{
                                let L=s2.read().lang;
                                let _=s2.write().bg_tx.0.send(BgEvent::VideoStatus{done:false,failed:true,progress:0.0,message:i18n::t(L,"vs.nourl").to_string(),seconds:String::new(),size:String::new(),transient:false});
                            }
                        }else{
                            let _=s2.write().bg_tx.0.send(BgEvent::VideoStatus{done:false,failed:false,progress:st_.progress,message:st_.message,seconds:st_.seconds,size:st_.size,transient:false});
                        }
                    }
                    Err(e)=>{
                        // 暂时性错误（429 / 网络抖动）：不中断，渐进退避
                        fails+=1;
                        interval=(interval*2).min(30);
                        let L=s2.read().lang;
                        let hint=if e.contains("429")||e.to_lowercase().contains("rate limit"){
                            i18n::tf(L,"vs.retry429",&[("n",&interval.to_string())])
                        }else{i18n::tf(L,"vs.retry",&[("n",&interval.to_string())])};
                        let prog=s2.read().video_progress;
                        let _=s2.write().bg_tx.0.send(BgEvent::VideoStatus{done:false,failed:false,progress:prog,message:hint,seconds:String::new(),size:String::new(),transient:true});
                        // 连续 20 次（约累计数分钟）仍失败才判定为真失败
                        if fails>=20{
                            let _=s2.write().bg_tx.0.send(BgEvent::VideoStatus{done:false,failed:true,progress:0.0,message:e,seconds:String::new(),size:String::new(),transient:false});
                        }
                    }
                }
            }else{
                // 没有在生成：重置退避
                interval=5;fails=0;
            }
        }
    });}

    // 默认窗口尺寸：显示器 80%，居中；并取消开发模式默认的置顶
    use_effect(move || {
        let win=dioxus::desktop::window();
        win.set_title("AgnesStudio");
        win.set_always_on_top(false);
        if let Some(mon)=win.primary_monitor(){
            let ms=mon.size();let(mw,mh)=(ms.width as f64,ms.height as f64);
            let wpx=(mw*0.8) as u32;let hpx=(mh*0.8) as u32;
            win.set_inner_size(dioxus::desktop::tao::dpi::PhysicalSize::new(wpx,hpx));
            let mp=mon.position();
            let x=mp.x+((mw as i64-wpx as i64)/2) as i32;
            let y=mp.y+((mh as i64-hpx as i64)/2) as i32;
            win.set_outer_position(dioxus::desktop::tao::dpi::PhysicalPosition::new(x,y));
        }
    });

    // 注册视频资源协议：通过 /video/{id} 流式播放，支持 Range 请求
    {
        let store=st.read().video_store.clone();
        use_hook(move||{
            dioxus::desktop::window().register_asset_handler("video".to_string(),move|req,responder|{
                serve_video_asset(&store,&req,responder);
            });
            // 注册图标资源：通过 /icon 提供内嵌 PNG
            dioxus::desktop::window().register_asset_handler("icon".to_string(),move|req,responder|{
                serve_icon_asset(&req,responder);
            });
        });
    }

    let theme_attr=st.read().theme.attr().to_string();
    rsx!{
        div{class:"app","data-theme":"{theme_attr}",
            TopBar{st:st.clone()}
            div{class:"body",
                if st.read().workspace==Workspace::Video{
                    VideoSidePanel{st:st.clone()}
                }else{
                    SidePanel{st:st.clone()}
                }
                div{class:"work",
                    if st.read().workspace==Workspace::Video{
                        VideoMainArea{st:st.clone()}
                    }else{
                        MainArea{st:st.clone()}
                    }
                    HistoryBar{st:st.clone()}
                }
            }
            PreviewModal{st:st.clone()}
            UpdateDialog{st:st.clone()}
            SettingsDialog{st:st.clone()}
            style{"{css}"}
        }
    }
}

// ── TopBar ──────────────────────────────────────────────────────────────────────

#[component]
fn TopBar(st:Signal<AppState>)->Element{
    let L=st.read().lang;
    let key_ok=!st.read().cfg.api_key.trim().is_empty();
    let ws=st.read().workspace;
    let mdl=if ws==Workspace::Video{
        VIDEO_MODELS.get(st.read().vmodel_index).map(|m|m.0).unwrap_or("").to_string()
    }else{
        MODELS.get(st.read().model_index).map(|m|m.0).unwrap_or("").to_string()
    };
    let loading=match ws{Workspace::Image=>st.read().loading,Workspace::Video=>st.read().video_loading};
    let dot_c=if key_ok{"var(--ok)"}else{"var(--err)"};
    let key_txt=if key_ok{i18n::t(L,"key.ok")}else{i18n::t(L,"key.missing")};
    let gen_txt=format!("● {}",i18n::t(L,"top.gen"));
    let upd_el=st.read().update_info.as_ref().map(|i|{
        let txt=format!("● {}",i18n::tf(L,"upd.chip",&[("v",&i.version)]));
        rsx!{
            span{class:"chip warn click",onclick:move|_|st.write().show_update_dialog=true,"{txt}"}
        }
    });
    let web_lnk=i18n::t(L,"link.web").to_string();
    let is_img=ws==Workspace::Image;
    let img_cls=if is_img{"tab on"}else{"tab"};
    let vid_cls=if is_img{"tab"}else{"tab on"};
    let img_tab=format!("🖼 {}",i18n::t(L,"tab.image"));
    let vid_tab=format!("🎬 {}",i18n::t(L,"tab.video"));
    let theme_icon=st.read().theme.icon().to_string();
    let theme_tip=match st.read().theme{ThemeMode::Light=>i18n::t(L,"th.light"),ThemeMode::Dark=>i18n::t(L,"th.dark"),ThemeMode::System=>i18n::t(L,"th.sys")}.to_string();
    let lang_lbl=if L==Lang::Zh{"EN"}else{"中"}.to_string();
    let set_tip=i18n::t(L,"set.title").to_string();
    let on_img=move|_|st.write().workspace=Workspace::Image;
    let on_vid=move|_|st.write().workspace=Workspace::Video;
    let cycle_theme=move|_|{
        let mut s=st.write();
        s.theme=s.theme.cycle();
        s.cfg.theme=s.theme.to_cfg().to_string();
        config::save(&s.cfg);
    };
    let toggle_lang=move|_|{
        let mut s=st.write();
        s.lang=if s.lang==Lang::Zh{Lang::En}else{Lang::Zh};
        s.cfg.lang=s.lang.to_cfg().to_string();
        config::save(&s.cfg);
    };

    rsx!{
        div{class:"topbar",
            img{src:"/icon",alt:"AgnesStudio",style:"width:23px;height:23px;flex-shrink:0;"},
            span{class:"brand","AgnesStudio"}
            span{class:"modelbadge","{mdl}"}
            div{style:"width:6px;flex-shrink:0;"}
            div{class:"tabs",
                button{class:"{img_cls}",onclick:on_img,"{img_tab}"}
                button{class:"{vid_cls}",onclick:on_vid,"{vid_tab}"}
            }
            div{style:"flex:1;"}
            {upd_el}
            if loading{
                span{class:"chip pulse","{gen_txt}"}
            }
            span{class:"link",onclick:move|_|open_url("https://agnes-ai.com/"),"{web_lnk}"}
            span{class:"link",onclick:move|_|open_url("https://github.com/LingyunStudio/AgnesStudio"),"GitHub"}
            button{class:"iconbtn",title:"{theme_tip}",onclick:cycle_theme,"{theme_icon}"}
            button{class:"iconbtn",style:"font-weight:800;font-size:12px;",title:"Language",onclick:toggle_lang,"{lang_lbl}"}
            button{class:"iconbtn",title:"{set_tip}",onclick:move|_|st.write().show_settings=true,"⚙"}
            div{class:"keychip",
                div{class:"dot",style:"background:{dot_c};"}
                span{"{key_txt}"}
            }
        }
    }
}

// ── SidePanel（图片）──────────────────────────────────────────────────────────

#[component]
fn SidePanel(st:Signal<AppState>)->Element{
    let L=st.read().lang;
    let midx=st.read().model_index;
    let loading=st.read().loading;
    let sidx=st.read().size_preset_index;
    let is_21=img_is_21(&st.read());
    let tidx=st.read().tier_index;
    let ridx=st.read().ratio_index;
    let reset=st.read().reset_token;
    let prompt=st.read().prompt.clone();
    let mode_sel=if matches!(st.read().mode,Mode::Image){1}else{0};
    let fmt_sel=if matches!(st.read().out_fmt,OutFmt::B64){1}else{0};

    let mut mopts:Vec<Element>=Vec::new();
    for(i,(_,b))in MODELS.iter().enumerate(){mopts.push(rsx!{option{selected:i==midx,value:"{i}","{b.l(L)}"}});}
    let mut sopts:Vec<Element>=Vec::new();
    for(i,(b,_))in SIZE_PRESETS.iter().enumerate(){sopts.push(rsx!{option{selected:i==sidx,value:"{i}","{b.l(L)}"}});}
    let mut topts:Vec<Element>=Vec::new();
    for(i,tier)in IMG_TIERS.iter().enumerate(){
        let lab=i18n::tf(L,"img.tieropt",&[("t",tier)]);
        topts.push(rsx!{option{selected:i==tidx,value:"{i}","{lab}"}});
    }
    let mut ropts:Vec<Element>=Vec::new();
    for(i,(rt,_))in IMG_TIER_SIZES.iter().enumerate(){ropts.push(rsx!{option{selected:i==ridx,value:"{i}","{rt}"}});}

    let ph=i18n::t(L,"img.ph").to_string();
    let mode_lbl=i18n::t(L,"card.mode").to_string();
    let tier_lbl=i18n::t(L,"card.tier").to_string();
    let ratio_lbl=i18n::t(L,"card.ratio").to_string();
    let gen_btn=if loading{i18n::t(L,"img.wait").to_string()}else{format!("✨  {}",i18n::t(L,"img.gen"))};
    let mode_opts=vec![i18n::t(L,"img.t2i").to_string(),i18n::t(L,"img.i2i").to_string()];
    let fmt_opts=vec!["URL".to_string(),"Base64".to_string()];
    let exact_hint=i18n::tf(L,"img.exact",&[("exact",tier_exact_size(&st.read())),("size",&resolved_size(&st.read())),("ratio",resolved_ratio(&st.read()).as_deref().unwrap_or(""))]);
    let cur_hint=i18n::tf(L,"img.cur",&[("s",&resolved_size(&st.read()))]);

    rsx!{
        div{class:"side",
            div{class:"side-scroll",

            Card{title:i18n::t(L,"card.model").to_string(),
                select{class:"sel",onchange:move|e|{if let Ok(i)=e.value().parse::<usize>(){st.write().model_index=i;}},value:"{midx}",{mopts.into_iter()}}
                div{class:"subh","{mode_lbl}"}
                SegBtns{sel:mode_sel,opts:mode_opts,on_set:move|i|st.write().mode=if i==0{Mode::Text}else{Mode::Image}}
            }

            Card{title:i18n::t(L,"card.prompt").to_string(),
                textarea{key:"img-prompt-{reset}",class:"ta",placeholder:"{ph}",initial_value:"{prompt}",oninput:move|e|st.write().prompt=e.value()}
            }

            if st.read().mode==Mode::Image{InputSection{st:st.clone()}}

            if is_21{
                Card{title:i18n::t(L,"card.size").to_string(),
                    div{class:"subh",style:"margin-top:0;","{tier_lbl}"}
                    select{class:"sel",onchange:move|e|{if let Ok(i)=e.value().parse::<usize>(){st.write().tier_index=i;}},value:"{tidx}",{topts.into_iter()}}
                    div{class:"subh","{ratio_lbl}"}
                    select{class:"sel",onchange:move|e|{if let Ok(i)=e.value().parse::<usize>(){st.write().ratio_index=i;}},value:"{ridx}",{ropts.into_iter()}}
                    div{class:"hint","{exact_hint}"}
                }
            }else{
                Card{title:i18n::t(L,"card.size").to_string(),
                    select{class:"sel",onchange:move|e|{if let Ok(i)=e.value().parse::<usize>(){st.write().size_preset_index=i;}},value:"{sidx}",{sopts.into_iter()}}
                    if sidx==SIZE_PRESETS.len()-1{CustomSize{st:st.clone()}}
                    div{class:"hint",style:"margin-top:6px;","{cur_hint}"}
                }
            }

            Card{title:i18n::t(L,"card.output").to_string(),
                SegBtns{sel:fmt_sel,opts:fmt_opts,on_set:move|i|st.write().out_fmt=if i==0{OutFmt::Url}else{OutFmt::B64}}
            }
            }

            // 生成按钮固定在侧栏底部，永远可见，不会被历史栏遮挡
            div{class:"side-action",
                button{class:"b2",disabled:loading,onclick:move|_|on_gen(st.clone()),"{gen_btn}"}
            }
        }
    }
}

#[component]
fn Card(title:String,children:Element)->Element{
    rsx!{div{class:"card",
        div{class:"cardh",span{class:"cardt","{title}"}}
        {children}
    }}
}

#[component]
fn SegBtns(sel:usize,opts:Vec<String>,on_set:EventHandler<usize>)->Element{
    let mut btns:Vec<Element>=Vec::new();
    for(i,opt)in opts.iter().enumerate(){
        let on=i==sel;
        let cls=if on{"s1 on"}else{"s1"};
        btns.push(rsx!{
            button{key:"{i}",class:"{cls}",onclick:move|_|on_set.call(i),"{opt}"}
        });
    }
    rsx!{div{class:"sg",{btns.into_iter()}}}
}

#[component]
fn InputSection(st:Signal<AppState>)->Element{
    let L=st.read().lang;
    let srci=if matches!(st.read().input_src,InputSrc::Url){1}else{0};
    let is_file=st.read().input_src==InputSrc::File;
    let has_f=st.read().input_file.is_some();
    let fname=if let Some((ref n,_))=st.read().input_file{format!("📎 {n}")}else{i18n::t(L,"img.nofile").to_string()};
    let tx=st.read().bg_tx.clone();
    let input_url=st.read().input_url.clone();
    let src_opts=vec![i18n::t(L,"img.file").to_string(),i18n::t(L,"img.url").to_string()];

    rsx!{
        Card{title:i18n::t(L,"card.input").to_string(),
            SegBtns{sel:srci,opts:src_opts,on_set:move|i|st.write().input_src=if i==0{InputSrc::File}else{InputSrc::Url}}
            div{style:"height:6px;flex-shrink:0;"}
            if is_file{
                FileInputArea{st:st.clone(),tx:tx.clone(),has_f:has_f,fname:fname}
            }else{
                input{class:"ix",placeholder:"https://...",value:"{input_url}",oninput:move|e|st.write().input_url=e.value()}
            }
        }
    }
}

#[component]
fn CustomSize(st:Signal<AppState>)->Element{
    let cw=st.read().custom_w;let ch=st.read().custom_h;
    rsx!{
        div{class:"row",style:"margin-top:6px;",
            input{class:"ix",style:"width:80px;text-align:center;",r#type:"number",min:64,max:4096,value:"{cw}",oninput:move|e|{if let Ok(v)=e.value().parse::<i32>(){st.write().custom_w=v.clamp(64,4096);}}}
            span{style:"color:var(--text2);","×"}
            input{class:"ix",style:"width:80px;text-align:center;",r#type:"number",min:64,max:4096,value:"{ch}",oninput:move|e|{if let Ok(v)=e.value().parse::<i32>(){st.write().custom_h=v.clamp(64,4096);}}}
        }
    }
}

#[component]
fn SettingsBody(st:Signal<AppState>)->Element{
    let L=st.read().lang;
    let pwd=if st.read().api_key_visible{"text"}else{"password"};
    let eye=if st.read().api_key_visible{i18n::t(L,"set.hide")}else{i18n::t(L,"set.show")};
    let eye_txt=eye.to_string();
    let tx=st.read().bg_tx.clone();
    let api_key=st.read().cfg.api_key.clone();
    let save_dir=st.read().cfg.save_dir.clone();
    let theme_sel=match st.read().theme{ThemeMode::Light=>0,ThemeMode::Dark=>1,ThemeMode::System=>2};
    let lang_sel=if L==Lang::Zh{0}else{1};
    let checking=st.read().update_checking;
    let upd_err=st.read().update_error.clone();
    let uptodate=st.read().update_uptodate;
    let notice=st.read().notice.clone();
    let ver=i18n::tf(L,"set.version",&[("v",updater::CURRENT_VERSION)]);
    let theme_opts=vec![i18n::t(L,"th.light").to_string(),i18n::t(L,"th.dark").to_string(),i18n::t(L,"th.sys").to_string()];
    let lang_opts=vec!["中文".to_string(),"English".to_string()];
    let check_txt=if checking{i18n::t(L,"set.checking")}else{i18n::t(L,"set.check")};
    let save_txt=i18n::t(L,"set.save").to_string();
    let reset_txt=i18n::t(L,"set.reset").to_string();
    let browse_txt=i18n::t(L,"set.browse").to_string();
    let dir_lbl=i18n::t(L,"set.dir").to_string();
    let theme_lbl=i18n::t(L,"set.theme").to_string();
    let lang_lbl=i18n::t(L,"set.lang").to_string();
    let checkLbl=i18n::t(L,"set.checking").to_string();
    let ok_txt=i18n::t(L,"set.uptodate").to_string();
    let notice_el=notice.map(|n|{
        let cls=nt_class(n.kind);let txt=n.text;
        rsx!{div{class:"{cls}",style:"margin-top:6px;","{txt}"}}
    });

    let set_theme=move|i:usize|{
        let mut s=st.write();
        s.theme=match i{0=>ThemeMode::Light,1=>ThemeMode::Dark,_=>ThemeMode::System};
        s.cfg.theme=s.theme.to_cfg().to_string();
        config::save(&s.cfg);
    };
    let set_lang=move|i:usize|{
        let mut s=st.write();
        s.lang=if i==0{Lang::Zh}else{Lang::En};
        s.cfg.lang=s.lang.to_cfg().to_string();
        config::save(&s.cfg);
    };

    rsx!{
        div{class:"row between",
            span{class:"lbl","API Key"}
            button{class:"g",onclick:move|_|{let vis=st.read().api_key_visible;st.write().api_key_visible=!vis;},"{eye_txt}"}
        }
        input{class:"ix",r#type:"{pwd}",placeholder:"Bearer token",value:"{api_key}",oninput:move|e|st.write().cfg.api_key=e.value()}
        div{class:"subh","{dir_lbl}"}
        div{class:"row",
            input{class:"ix",style:"flex:1;",value:"{save_dir}",oninput:move|e|st.write().cfg.save_dir=e.value()}
            button{class:"g",onclick:move|_|browse_dir(tx.clone()),"{browse_txt}"}
        }
        div{class:"subh","{theme_lbl}"}
        SegBtns{sel:theme_sel,opts:theme_opts,on_set:set_theme}
        div{class:"subh","{lang_lbl}"}
        SegBtns{sel:lang_sel,opts:lang_opts,on_set:set_lang}
        div{style:"height:4px;flex-shrink:0;"}
        div{class:"row",
            button{class:"g",onclick:move|_|{config::save(&st.read().cfg);let msg=i18n::t(L,"set.saved").to_string();st.write().notice(NoticeKind::Ok,msg);},"{save_txt}"}
            button{class:"g",onclick:move|_|{
                // 恢复默认：保留 API Key 与界面偏好（主题/语言）
                let k=st.read().cfg.api_key.clone();
                let theme=st.read().theme;let lang=st.read().lang;
                st.write().cfg=config::Config::default();
                st.write().cfg.api_key=k;
                st.write().cfg.theme=theme.to_cfg().to_string();
                st.write().cfg.lang=lang.to_cfg().to_string();
                set_defaults(&mut st.write());
            },"{reset_txt}"}
        }
        {notice_el}
        div{style:"height:12px;flex-shrink:0;"}
        div{style:"border-top:1px solid var(--border);padding-top:10px;",
            div{class:"row",
                span{class:"lbl","{ver}"}
                if checking{span{class:"lbl",style:"color:var(--text2);","{checkLbl}"}}
                div{style:"flex:1;"}
                button{class:"g",onclick:move|_|CheckUpdate(st.clone()),
                    disabled:checking,
                    "{check_txt}"
                }
            }
            if uptodate{
                div{class:"nt nt-ok",style:"margin-top:6px;","{ok_txt}"}
            }
            if!upd_err.is_empty(){
                div{class:"nt nt-err",style:"margin-top:6px;","{upd_err}"}
            }
        }
    }
}

#[component]
fn FileInputArea(st:Signal<AppState>,tx:EventTx,has_f:bool,fname:String)->Element{
    let L=st.read().lang;
    let pick_txt=i18n::t(L,"img.pick").to_string();
    let clear_txt=i18n::t(L,"img.clear").to_string();
    rsx!{
        div{class:"row",
            button{class:"g",onclick:move|_|pick_file(tx.clone(),st.clone()),"{pick_txt}"}
            if has_f{button{class:"g",onclick:move|_|st.write().input_file=None,"{clear_txt}"}}
        }
        span{class:"oklbl","{fname}"}
    }
}

#[component]
fn HistoryButtons(st:Signal<AppState>)->Element{
    let prev=move|_|{
        let mut s=st.write();
        if s.workspace==Workspace::Video{
            s.video_selected=if s.video_selected==0{s.videos.len().max(1)-1}else{s.video_selected-1};
            s.video_error.clear();
        }else{
            s.selected=if s.selected==0{s.images.len().max(1)-1}else{s.selected-1};
            s.error.clear();
        }
    };
    let next=move|_|{
        let mut s=st.write();
        if s.workspace==Workspace::Video{
            s.video_selected=(s.video_selected+1)%s.videos.len().max(1);
            s.video_error.clear();
        }else{
            s.selected=(s.selected+1)%s.images.len().max(1);
            s.error.clear();
        }
    };
    rsx!{
        div{class:"row",
            button{class:"g",onclick:prev,"◀"}
            button{class:"g",onclick:next,"▶"}
        }
    }
}

// ── ErrorCard（主生成区的错误卡片）────────────────────────────────────────────

#[component]
fn ErrorCard(st:Signal<AppState>,error:String,on_dismiss:EventHandler<()>)->Element{
    let L=st.read().lang;
    // API Key 缺失时给出直达设置的入口
    let nokey=error==i18n::t(L,"err.nokey");
    let title=i18n::t(L,"err.title").to_string();
    let open_set=i18n::t(L,"err.open_settings").to_string();
    let dismiss_txt=i18n::t(L,"err.dismiss").to_string();
    rsx!{
        div{class:"center",
            div{class:"errcard",
                div{class:"erricon","⚠"}
                span{class:"errtitle","{title}"}
                span{class:"errmsg","{error}"}
                div{class:"row",style:"margin-top:16px;",
                    if nokey{
                        button{class:"g",onclick:move|_|st.write().show_settings=true,"{open_set}"}
                    }
                    button{class:"g",onclick:move|_|on_dismiss.call(()),"{dismiss_txt}"}
                }
            }
        }
    }
}

// ── MainArea（图片）──────────────────────────────────────────────────────────

#[component]
fn MainArea(st:Signal<AppState>)->Element{
    let L=st.read().lang;
    if st.read().loading{
        let elapsed=st.read().gen_elapsed;
        let el_txt=i18n::tf(L,"main.elapsed",&[("s",&format!("{elapsed:.1}"))]);
        let title_txt=i18n::t(L,"img.wait").to_string();
        let wait_txt=i18n::t(L,"main.wait").to_string();
        return rsx!{
            div{class:"center",
                div{class:"spinner",div{class:"spin"}}
                span{class:"loadtitle","{title_txt}"}
                span{class:"loadsub","{el_txt}"}
                div{class:"bar",div{class:"barind"}}
                span{class:"loadsub",style:"margin-top:10px;","{wait_txt}"}
            }
        };
    }

    // 错误统一在主生成区展示，左侧不再显示任何报错
    let err=st.read().error.clone();
    if!err.is_empty(){
        return rsx!{ErrorCard{st:st.clone(),error:err,on_dismiss:move|_|st.write().error.clear()}};
    }

    let has_imgs=!st.read().images.is_empty();
    if!has_imgs{
        let welcome_txt=i18n::t(L,"main.welcome").to_string();
        let empty_txt=i18n::t(L,"main.empty").to_string();
        return rsx!{
            div{class:"center",
                div{class:"emptylogo",img{src:"/icon",alt:"AgnesStudio",style:"width:34px;height:34px;"}}
                span{class:"emptyt","{welcome_txt}"}
                span{class:"emptys","{empty_txt}"}
            }
        };
    }

    let sel=cmp::min(st.read().selected,st.read().images.len().saturating_sub(1));
    let entry=st.read().images[sel].clone();
    rsx!{ImageViewer{st:st.clone(),entry:entry}}
}

#[component]
fn ImageViewer(st:Signal<AppState>,entry:CachedImage)->Element{
    let L=st.read().lang;
    let notice=st.read().notice.clone();
    let dims=entry.dims;let has_url=entry.url.is_some();
    let url_act=entry.url.clone().unwrap_or_default();
    let d_uri=entry.data_uri.clone();

    let sz_str=format!("· {} · {}x{}",entry.size,dims[0],dims[1]);
    let reg_txt=format!("🔄 {}",i18n::t(L,"act.reg"));
    let open_txt=format!("🌐 {}",i18n::t(L,"act.openimg"));
    let save_txt=format!("💾 {}",i18n::t(L,"act.save"));
    let pline=format!("{}  {}",i18n::t(L,"viewer.prompt"),entry.prompt);

    let mut ar:Vec<Element>=vec![];
    ar.push(rsx!{span{class:"mmodel","{entry.model}"}});
    ar.push(rsx!{span{class:"minfo","{sz_str}"}});
    ar.push(rsx!{div{style:"flex:1;"}});
    ar.push(rsx!{button{class:"g",onclick:move|_|on_gen(st.clone()),"{reg_txt}"}});
    if has_url{ar.push(rsx!{button{class:"g",onclick:move|_|open_url(&url_act),"{open_txt}"}});}
    ar.push(rsx!{button{class:"g",onclick:move|_|do_save(&mut st.write()),"{save_txt}"}});

    let ntel=notice.map(|n|{
        let cls=nt_class(n.kind);let txt=n.text;
        rsx!{div{class:"{cls}","{txt}"}}
    });

    rsx!{
        div{class:"main",
            div{class:"stage",
                div{class:"imgwrap",
                    img{src:"{d_uri}",style:"max-width:70vw;max-height:calc(100vh - 300px);object-fit:contain;border-radius:8px;display:block;",
                        onclick:move|_|{let mut s=st.write();s.show_popup=true;s.popup_uri=d_uri.clone();s.popup_dims=dims;s.popup_zoom=1.0;s.popup_pan=[0.0,0.0];}
                    }
                }
            }
            div{class:"meta",
                div{class:"metatop",{ar.into_iter()}}
                hr{class:"divider"}
                div{class:"mprompt","{pline}"}
                {ntel}
            }
        }
    }
}

// ── HistoryBar ───────────────────────────────────────────────────────────────

#[component]
fn HistoryBar(st:Signal<AppState>)->Element{
    let L=st.read().lang;
    let ws=st.read().workspace;
    if ws==Workspace::Video{
        let vids=st.read().videos.clone();
        if vids.is_empty(){return rsx!{}}
        let count=vids.len();
        let cur=st.read().video_selected;
        let title=i18n::tf(L,"hist.vid",&[("n",&count.to_string())]);
        let mut thumbs:Vec<Element>=Vec::new();
        for(i,_v)in vids.iter().enumerate(){
            let cls=if i==cur{"thumb on"}else{"thumb"};
            let label=format!("🎬 {}",i+1);
            thumbs.push(rsx!{
                div{key:"{i}",class:"{cls}",
                    onclick:move|_|{let mut s=st.write();s.video_selected=i;s.video_error.clear();},
                    div{class:"thumbvid","{label}"}
                }
            });
        }
        return rsx!{
            div{class:"hist",
                div{class:"histh",
                    span{class:"histt","{title}"}
                    div{style:"flex:1;"}
                    if count>1{HistoryButtons{st:st.clone()}}
                }
                div{class:"thumbs",{thumbs.into_iter()}}
            }
        };
    }

    if st.read().images.is_empty(){return rsx!{}}
    let count=st.read().images.len();
    let title=i18n::tf(L,"hist.img",&[("n",&count.to_string())]);

    let mut thumbs:Vec<Element>=Vec::new();
    for(i,img)in st.read().images.iter().enumerate(){
        let cls=if i==st.read().selected{"thumb on"}else{"thumb"};
        let uri=img.data_uri.clone();
        thumbs.push(rsx!{
            div{key:"{i}",class:"{cls}",
                onclick:move|_|{let mut s=st.write();s.selected=i;s.error.clear();},
                img{class:"thumbimg",src:"{uri}"}
            }
        });
    }

    rsx!{
        div{class:"hist",
            div{class:"histh",
                span{class:"histt","{title}"}
                div{style:"flex:1;"}
                if count>1{HistoryButtons{st:st.clone()}}
            }
            div{class:"thumbs",{thumbs.into_iter()}}
        }
    }
}

// ── UpdateDialog ───────────────────────────────────────────────────────────────

#[component]
fn UpdateDialog(st:Signal<AppState>)->Element{
    if!st.read().show_update_dialog{return rsx!{}}
    let info=match st.read().update_info.clone(){Some(i)=>i,None=>return rsx!{}};
    let L=st.read().lang;
    let downloading=st.read().update_downloading;
    let progress=st.read().update_progress;
    let err=st.read().update_error.clone();
    let has_setup=info.setup_url.is_some();
    let cur=updater::CURRENT_VERSION.to_string();
    let notes_text=if info.notes.trim().is_empty(){i18n::t(L,"upd.nonotes").to_string()}else{info.notes.clone()};
    let notes_html=render_markdown(&notes_text);
    let new_ver=info.version.clone();
    let html=info.html_url.clone();
    let verline=format!("v{cur}  ->  v{new_ver}");
    let title_txt=i18n::t(L,"upd.title").to_string();
    let dl_txt=i18n::tf(L,"upd.dl",&[("p",&progress.to_string())]);
    let dling_txt=i18n::t(L,"upd.dling").to_string();
    let now_txt=i18n::t(L,"upd.now").to_string();
    let goto_txt=i18n::t(L,"upd.goto").to_string();
    let later_txt=i18n::t(L,"upd.later").to_string();

    rsx!{
        div{class:"mask",
            onclick:move|_|{if!downloading{st.write().show_update_dialog=false;}},
            div{class:"dialog",
                onclick:move|e|e.stop_propagation(),
                div{class:"row",style:"gap:9px;margin-bottom:2px;",
                    div{style:"width:10px;height:10px;border-radius:5px;background:var(--warn);flex-shrink:0;"}
                    span{class:"dtitle","{title_txt}"}
                }
                span{class:"dsub","{verline}"}
                div{class:"notes",dangerous_inner_html:"{notes_html}"}
                if!err.is_empty(){
                    div{class:"nt nt-err",style:"margin:0 0 10px;","{err}"}
                }
                if downloading{
                    div{style:"margin-bottom:8px;",
                        div{style:"height:8px;background:var(--fill);border-radius:4px;overflow:hidden;",
                            div{style:"height:100%;width:{progress}%;background:var(--bar);border-radius:4px;transition:width .3s;"}}
                        span{class:"loadsub",style:"display:block;margin-top:6px;","{dl_txt}"}
                    }
                    button{class:"b2",style:"opacity:.6;cursor:default;","{dling_txt}"}
                }
                if !downloading&&has_setup{
                    button{class:"b2",onclick:move|_|StartUpdate(st.clone()),"{now_txt}"}
                }
                if !downloading&&!has_setup{
                    button{class:"b2",onclick:move|_|open_url(&html),"{goto_txt}"}
                }
                div{style:"height:10px;flex-shrink:0;"}
                button{class:"g",style:"width:100%;",onclick:move|_|st.write().show_update_dialog=false,"{later_txt}"}
            }
        }
    }
}

// ── SettingsDialog（设置弹窗）──────────────────────────────────────────────────

#[component]
fn SettingsDialog(st:Signal<AppState>)->Element{
    if!st.read().show_settings{return rsx!{}}
    let L=st.read().lang;
    let title=i18n::t(L,"set.title").to_string();
    let done_txt=i18n::t(L,"set.done").to_string();
    rsx!{
        // 层级低于更新弹窗（1100），检查到新版本时更新弹窗盖在上面
        div{class:"mask",style:"z-index:1090;",
            onclick:move|_|st.write().show_settings=false,
            div{class:"dialog",
                onclick:move|e|e.stop_propagation(),
                div{class:"row between",style:"margin-bottom:14px;",
                    span{class:"dtitle","{title}"}
                    button{class:"g",onclick:move|_|st.write().show_settings=false,"{done_txt}"}
                }
                div{style:"flex:1;overflow:auto;min-height:0;",SettingsBody{st:st.clone()}}
            }
        }
    }
}

// ── PreviewModal ───────────────────────────────────────────────────────────────

#[component]
fn PreviewModal(st:Signal<AppState>)->Element{
    if!st.read().show_popup{return rsx!{}}
    let L=st.read().lang;
    let uri=st.read().popup_uri.clone();let dims=st.read().popup_dims;
    let zoom=st.read().popup_zoom;let pan=st.read().popup_pan;
    let title=i18n::tf(L,"pv.title",&[("w",&dims[0].to_string()),("h",&dims[1].to_string()),("p",&format!("{:.0}",zoom*100.0))]);
    let hint=i18n::t(L,"pv.hint").to_string();

    // 拖拽平移用本地状态
    let mut dragging=use_signal(||false);
    let mut last=use_signal(||(0.0_f64,0.0_f64));

    let clamp_zoom=|z:f32|z.clamp(0.2,12.0);
    let cursor=if zoom>1.0{"grab"}else{"default"};
    let transform=format!("translate({}px,{}px) scale({})",pan[0],pan[1],zoom);

    rsx!{
        div{class:"mask",style:"z-index:1000;",
            onclick:move|_|st.write().show_popup=false,
            div{class:"pvbox",
                onclick:move|e|e.stop_propagation(),
                div{class:"row",style:"justify-content:space-between;margin-bottom:8px;",
                    span{class:"pvtitle","{title}"}
                    div{class:"pvctrls",
                        // 缩放控件
                        button{class:"pvbtn",onclick:move|_|{let z=st.read().popup_zoom;st.write().popup_zoom=clamp_zoom(z/1.2);},"➖"}
                        button{class:"pvbtn",onclick:move|_|{let z=st.read().popup_zoom;st.write().popup_zoom=clamp_zoom(z*1.2);},"➕"}
                        button{class:"pvbtn",onclick:move|_|{st.write().popup_zoom=1.0;st.write().popup_pan=[0.0,0.0];},"⤢ 1:1"}
                        button{class:"pvbtn",onclick:move|_|st.write().show_popup=false,"✕"}
                    }
                }
                div{class:"pvstage",
                    img{src:"{uri}",
                        style:"max-width:100%;max-height:80vh;object-fit:contain;transform-origin:0 0;transform:{transform};cursor:{cursor};user-select:none;-webkit-user-drag:none;",
                        onwheel:move|e|{
                            let dy=e.delta().strip_units().y;
                            let z=st.read().popup_zoom;
                            let nz=clamp_zoom(if dy>0.0{z/1.15}else{z*1.15});
                            st.write().popup_zoom=nz;
                            // 缩小到 1.0 以下时回到居中，避免偏移
                            if nz<=1.0{st.write().popup_pan=[0.0,0.0];}
                        },
                        onmousedown:move|e|{
                            if st.read().popup_zoom>1.0{
                                let p=e.client_coordinates();
                                dragging.set(true);last.set((p.x,p.y));
                            }
                        },
                        onmousemove:move|e|{
                            if*dragging.read(){
                                let(x,y)=*last.read();let p=e.client_coordinates();
                                let(dx,dy)=(p.x-x,p.y-y);last.set((p.x,p.y));
                                let mut s=st.write();s.popup_pan[0]+=dx as f32;s.popup_pan[1]+=dy as f32;
                            }
                        },
                        onmouseup:move|_|{dragging.set(false);},
                        onmouseleave:move|_|{dragging.set(false);},
                    }
                }
                div{class:"pvhint","{hint}"}
            }
        }
    }
}

// ── Handlers ────────────────────────────────────────────────────────────────────

fn on_gen(mut st:Signal<AppState>){
    let L=st.read().lang;
    let mut s=st.write();
    if s.loading{return}
    if s.cfg.api_key.trim().is_empty(){s.error=i18n::t(L,"err.nokey").to_string();return}
    if s.prompt.trim().is_empty(){s.error=i18n::t(L,"err.noprompt").to_string();return}
    if s.mode==Mode::Image&&cur_input(&s).is_none(){s.error=i18n::t(L,"err.noinput").to_string();return}
    let size=resolved_size(&s);let ratio=resolved_ratio(&s);let input=cur_input(&s);let prompt=s.prompt.clone();
    let model=MODELS[s.model_index].0.to_string();let fmt=if s.out_fmt==OutFmt::Url{"url"}else{"b64_json"};
    s.loading=true;s.error.clear();s.notice=None;s.gen_elapsed=0.0;
    s.cfg.last_prompt=s.prompt.clone();s.cfg.model=model.clone();
    s.cfg.output_format=fmt.to_string();s.cfg.mode=if s.mode==Mode::Text{"text".to_string()}else{"image".to_string()};
    // 2.1 档位尺寸存档位/比例（last_size 存精确像素便于展示与旧版回退）
    if let Some(r)=ratio.as_ref(){s.cfg.image_tier=size.clone();s.cfg.image_ratio=r.clone();s.cfg.last_size=tier_exact_size(&s).to_string();}
    else{s.cfg.last_size=size.clone();}
    config::save(&s.cfg);let api_key=s.cfg.api_key.clone();drop(s);
    let tx=st.read().bg_tx.0.clone();
    std::thread::spawn(move||{
        let rt=tokio::runtime::Runtime::new().expect("rt");
        let p=api::GenParams{api_key:api_key.clone(),model:model.clone(),prompt:prompt.clone(),size:size.clone(),ratio:ratio.clone(),input_image:input.clone(),output_format:fmt.to_string()};
        match rt.block_on(api::generate(p)){Ok(r)=>{let _=tx.send(BgEvent::ImageDone{bytes:r.bytes,url:r.url,prompt:prompt.clone(),model:model.clone(),size:size.clone()});}Err(e)=>{let _=tx.send(BgEvent::Error(e));}}
    });
}
fn on_gen_video(mut st:Signal<AppState>){
    let L=st.read().lang;
    let mut s=st.write();
    if s.video_loading{return}
    if s.cfg.api_key.trim().is_empty(){s.video_error=i18n::t(L,"err.nokey").to_string();return}
    if s.video_prompt.trim().is_empty(){s.video_error=i18n::t(L,"err.noprompt").to_string();return}
    let model=cur_video_model(&s).to_string();
    let v25=model!=api::MODEL_VIDEO_V20;
    let flash=model==api::MODEL_VIDEO_V25_FLASH;

    let kind=if v25{
        // ── Video 2.5 / 2.5 Flash：mode/seconds/aspect_ratio + 模式专属媒体 ──
        let first=s.v25_first.trim().to_string();
        let last=s.v25_last.trim().to_string();
        let images:Vec<String>=s.v25_images.iter().map(|u|u.trim().to_string()).filter(|u|!u.is_empty()).collect();
        let audios:Vec<String>=s.v25_audios.iter().map(|u|u.trim().to_string()).filter(|u|!u.is_empty()).collect();
        let videos:Vec<String>=s.v25_videos.iter().map(|u|u.trim().to_string()).filter(|u|!u.is_empty()).collect();
        match s.v25_mode{
            V25Mode::Keyframe=>{
                if first.is_empty()&&last.is_empty(){s.video_error=i18n::t(L,"err.kf").to_string();return}
            }
            V25Mode::Reference=>{
                if images.is_empty()&&audios.is_empty()&&videos.is_empty(){
                    s.video_error=i18n::t(L,"err.ref").to_string();return
                }
            }
            V25Mode::Text=>{}
        }
        // Flash 专属限制：图片参考最多 5 张、不支持视频参考
        if flash{
            if images.len()>5{s.video_error=i18n::t(L,"err.flashimg").to_string();return}
            if !videos.is_empty(){s.video_error=i18n::t(L,"err.flashvid").to_string();return}
        }
        api::VideoKind::V25{
            mode:match s.v25_mode{V25Mode::Text=>api::V25Mode::Text,V25Mode::Keyframe=>api::V25Mode::Keyframe,V25Mode::Reference=>api::V25Mode::Reference},
            seconds:V25_SECONDS[s.v25_seconds_index.min(V25_SECONDS.len()-1)].to_string(),
            aspect_ratio:V25_AR_PRESETS[s.v25_ar_index.min(V25_AR_PRESETS.len()-1)].1.to_string(),
            seed:None,
            first_frame:if first.is_empty(){None}else{Some(first)},
            last_frame:if last.is_empty(){None}else{Some(last)},
            images,audios,videos,flash,
        }
    }else{
        // ── Video V2.0：width/height/num_frames/frame_rate ──
        let need_imgs=s.vmode!=VMode::Text;
        let min_imgs=if s.vmode==VMode::Text{0}else if s.vmode==VMode::Image{1}else{2};
        let valid:Vec<String>=s.video_image_urls.iter().map(|u|u.trim().to_string()).filter(|u|!u.is_empty()).collect();
        if need_imgs&&valid.len()<min_imgs{
            s.video_error=if s.vmode==VMode::Image{i18n::t(L,"err.1img").to_string()}else{i18n::t(L,"err.2img").to_string()};
            return;
        }
        let(w,h)=video_dims(&s);let frames=video_frames(&s);let fps=video_fps(&s);
        let neg=s.video_neg.clone();
        let keyframes=s.vmode==VMode::Keyframes;
        api::VideoKind::V20{negative_prompt:neg,width:w,height:h,num_frames:frames,frame_rate:fps,seed:None,images:valid,keyframes}
    };

    let prompt=s.video_prompt.clone();
    s.video_loading=true;s.video_error.clear();s.video_elapsed=0.0;s.video_progress=0.0;
    s.video_msg=i18n::t(s.lang,"vs.submit").to_string();s.video_job=None;
    // 持久化视频配置
    s.cfg.video_model=model.clone();
    s.cfg.last_video_prompt=prompt.clone();
    if v25{
        s.cfg.video25_mode=match s.v25_mode{V25Mode::Text=>"text".to_string(),V25Mode::Keyframe=>"keyframe".to_string(),V25Mode::Reference=>"reference".to_string()};
        s.cfg.video25_seconds=V25_SECONDS[s.v25_seconds_index.min(V25_SECONDS.len()-1)].to_string();
        s.cfg.video25_ar=V25_AR_PRESETS[s.v25_ar_index.min(V25_AR_PRESETS.len()-1)].1.to_string();
        s.cfg.video25_first_frame=s.v25_first.clone();s.cfg.video25_last_frame=s.v25_last.clone();
    }else{
        s.cfg.video_neg_prompt=s.video_neg.clone();
        let(w,h)=video_dims(&s);
        s.cfg.video_width=w;s.cfg.video_height=h;
        s.cfg.video_num_frames=video_frames(&s);s.cfg.video_frame_rate=video_fps(&s);
        s.cfg.video_duration_preset=s.vduration_index;
        s.cfg.video_mode=match s.vmode{VMode::Text=>"text".to_string(),VMode::Image=>"image".to_string(),VMode::Multi=>"multi".to_string(),VMode::Keyframes=>"keyframes".to_string()};
    }
    config::save(&s.cfg);
    let api_key=s.cfg.api_key.clone();drop(s);
    let tx=st.read().bg_tx.0.clone();
    std::thread::spawn(move||{
        let rt=tokio::runtime::Runtime::new().expect("rt");
        let p=api::VideoParams{api_key,prompt,kind};
        match rt.block_on(api::create_video_task(&p)){
            Ok(t)=>{let _=tx.send(BgEvent::VideoCreated{video_id:t.video_id,task_id:t.task_id,seconds:t.seconds,size:t.size,model});}
            Err(e)=>{let _=tx.send(BgEvent::VideoStatus{done:false,failed:true,progress:0.0,message:e,seconds:String::new(),size:String::new(),transient:false});}
        }
    });
}

// ── 视频侧栏 ──────────────────────────────────────────────────────────────────

#[component]
fn VideoSidePanel(st:Signal<AppState>)->Element{
    let L=st.read().lang;
    let loading=st.read().video_loading;
    let v25=is_video25(&st.read());
    let vmidx=st.read().vmodel_index;
    let sidx=st.read().vsize_index;
    let didx=st.read().vduration_index;
    let vm=st.read().vmode.clone();
    let urls=st.read().video_image_urls.clone();
    let url_input=st.read().video_url_input.clone();
    let secs=video_seconds(&st.read());
    let(w,h)=video_dims(&st.read());
    // Video 2.5 状态
    let v25m=st.read().v25_mode;
    let s25idx=st.read().v25_seconds_index;
    let ar25idx=st.read().v25_ar_index;
    let v25_first=st.read().v25_first.clone();
    let v25_last=st.read().v25_last.clone();
    let v25_imgs=st.read().v25_images.clone();
    let v25_auds=st.read().v25_audios.clone();
    let v25_vids=st.read().v25_videos.clone();
    let v25_in=st.read().v25_input.clone();
    let v25flash=is_video25_flash(&st.read());
    let reset=st.read().reset_token;
    let video_prompt=st.read().video_prompt.clone();
    let video_neg=st.read().video_neg.clone();
    let prompt_ph=if v25m==V25Mode::Reference{
        if v25flash{i18n::t(L,"vid.ph.refflash")}else{i18n::t(L,"vid.ph.reffull")}
    }else{i18n::t(L,"vid.ph")}.to_string();
    let ref_hint=if v25flash{i18n::t(L,"v25.refhintflash")}else{i18n::t(L,"v25.refhint")}.to_string();

    let mut vmopts:Vec<Element>=Vec::new();
    for(i,(_,b))in VIDEO_MODELS.iter().enumerate(){vmopts.push(rsx!{option{selected:i==vmidx,value:"{i}","{b.l(L)}"}});}
    let mut sopts:Vec<Element>=Vec::new();
    for(i,(b,_,_))in VIDEO_SIZE_PRESETS.iter().enumerate(){sopts.push(rsx!{option{selected:i==sidx,value:"{i}","{b.l(L)}"}});}
    let mut dopts:Vec<Element>=Vec::new();
    for(i,(b,_,_))in VIDEO_DURATION_PRESETS.iter().enumerate(){dopts.push(rsx!{option{selected:i==didx,value:"{i}","{b.l(L)}"}});}
    let mut s25opts:Vec<Element>=Vec::new();
    for(i,s)in V25_SECONDS.iter().enumerate(){
        let lab=i18n::tf(L,"v25.secopt",&[("s",s)]);
        s25opts.push(rsx!{option{selected:i==s25idx,value:"{i}","{lab}"}});
    }
    let mut ar25opts:Vec<Element>=Vec::new();
    for(i,(b,_))in V25_AR_PRESETS.iter().enumerate(){ar25opts.push(rsx!{option{selected:i==ar25idx,value:"{i}","{b.l(L)}"}});}

    let mode_sel=match vm{VMode::Text=>0,VMode::Image=>1,VMode::Multi=>2,VMode::Keyframes=>3};
    let v25_mode_sel=match v25m{V25Mode::Text=>0,V25Mode::Keyframe=>1,V25Mode::Reference=>2};
    let v25_mode_opts=vec![i18n::t(L,"v25.t2v").to_string(),i18n::t(L,"v25.kf").to_string(),i18n::t(L,"v25.ref").to_string()];
    let v20_mode_opts=vec![i18n::t(L,"v20.t2v").to_string(),i18n::t(L,"v20.i2v").to_string(),i18n::t(L,"v20.multi").to_string(),i18n::t(L,"v20.kf").to_string()];

    let mut url_items:Vec<Element>=Vec::new();
    for(i,u)in urls.iter().enumerate(){
        let idx=i;
        let label=format!("#{}",i+1);
        url_items.push(rsx!{
            div{key:"u{idx}",class:"urlitem",
                span{class:"ulbl","{label}"}
                span{class:"utxt","{u}"}
                button{class:"g",style:"padding:2px 8px;",onclick:move|_|{st.write().video_image_urls.remove(idx);},"✕"}
            }
        });
    }

    // 2.5 参考素材列表（图片 / 音频 / 视频 三组合并展示）
    let mut v25_items:Vec<Element>=Vec::new();
    for(i,u)in v25_imgs.iter().enumerate(){
        let idx=i;
        let label=i18n::tf(L,"v25.lblimg",&[("n",&(i+1).to_string())]);
        v25_items.push(rsx!{
            div{key:"v25img{idx}",class:"urlitem",
                span{class:"ulbl","{label}"}
                span{class:"utxt","{u}"}
                button{class:"g",style:"padding:2px 8px;",onclick:move|_|{st.write().v25_images.remove(idx);},"✕"}
            }
        });
    }
    for(i,u)in v25_auds.iter().enumerate(){
        let idx=i;
        let label=i18n::tf(L,"v25.lblaud",&[("n",&(i+1).to_string())]);
        v25_items.push(rsx!{
            div{key:"v25aud{idx}",class:"urlitem",
                span{class:"ulbl","{label}"}
                span{class:"utxt","{u}"}
                button{class:"g",style:"padding:2px 8px;",onclick:move|_|{st.write().v25_audios.remove(idx);},"✕"}
            }
        });
    }
    for(i,u)in v25_vids.iter().enumerate(){
        let idx=i;
        let label=i18n::tf(L,"v25.lblvid",&[("n",&(i+1).to_string())]);
        v25_items.push(rsx!{
            div{key:"v25vid{idx}",class:"urlitem",
                span{class:"ulbl","{label}"}
                span{class:"utxt","{u}"}
                button{class:"g",style:"padding:2px 8px;",onclick:move|_|{st.write().v25_videos.remove(idx);},"✕"}
            }
        });
    }

    let gen_btn=if loading{i18n::t(L,"img.wait").to_string()}else{format!("🎬  {}",i18n::t(L,"vid.gen"))};
    // 文案常量
    let t_kfcard=i18n::t(L,"v25.kfcard").to_string();
    let t_kfhint=i18n::t(L,"v25.kfhint").to_string();
    let t_first=i18n::t(L,"v25.first").to_string();
    let t_last=i18n::t(L,"v25.last").to_string();
    let t_refcard=i18n::t(L,"v25.refcard").to_string();
    let t_addimg=i18n::t(L,"v25.addimg").to_string();
    let t_addaud=i18n::t(L,"v25.addaud").to_string();
    let t_addvid=i18n::t(L,"v25.addvid").to_string();
    let t_fmt=i18n::t(L,"v25.fmt").to_string();
    let t_ar=i18n::t(L,"v25.ar").to_string();
    let t_secs=i18n::t(L,"v25.secs").to_string();
    let t_imgcard=i18n::t(L,"v20.imgcard").to_string();
    let t_imghint=i18n::t(L,"v20.imghint").to_string();
    let t_add=i18n::t(L,"v20.add").to_string();
    let t_sizecard=i18n::t(L,"v20.sizecard").to_string();
    let t_durcard=i18n::t(L,"v20.durcard").to_string();
    let t_neg=i18n::t(L,"vid.neg").to_string();
    let t_negph=i18n::t(L,"vid.negph").to_string();
    let t_frames=i18n::t(L,"v20.frames").to_string();
    let vmode_lbl=i18n::t(L,"card.vmode").to_string();
    let cur_hint=i18n::tf(L,"v20.cur",&[("w",&w.to_string()),("h",&h.to_string())]);
    let est_hint=i18n::tf(L,"v20.est",&[("s",&secs)]);

    rsx!{
        div{class:"side",
            div{class:"side-scroll",

            Card{title:i18n::t(L,"card.model").to_string(),
                select{class:"sel",onchange:move|e|{if let Ok(i)=e.value().parse::<usize>(){st.write().vmodel_index=i;}},value:"{vmidx}",{vmopts.into_iter()}}
                div{class:"subh","{vmode_lbl}"}
                if v25{
                    SegBtns{sel:v25_mode_sel,opts:v25_mode_opts,on_set:move|i|st.write().v25_mode=match i{0=>V25Mode::Text,1=>V25Mode::Keyframe,_=>V25Mode::Reference}}
                }else{
                    SegBtns{sel:mode_sel,opts:v20_mode_opts,on_set:move|i|st.write().vmode=match i{0=>VMode::Text,1=>VMode::Image,2=>VMode::Multi,_=>VMode::Keyframes}}
                }
            }

            Card{title:i18n::t(L,"card.prompt").to_string(),
                textarea{key:"vid-prompt-{reset}",class:"ta",placeholder:"{prompt_ph}",initial_value:"{video_prompt}",oninput:move|e|st.write().video_prompt=e.value()}
                if!v25{
                    div{class:"subh","{t_neg}"}
                    textarea{key:"vid-neg-{reset}",class:"ta",style:"min-height:50px;",placeholder:"{t_negph}",initial_value:"{video_neg}",oninput:move|e|st.write().video_neg=e.value()}
                }
            }

            if v25{
                // ── Video 2.5 参数 ──
                if v25m==V25Mode::Keyframe{
                    Card{title:t_kfcard,
                        div{class:"hint",style:"margin:0 0 6px;","{t_kfhint}"}
                        div{class:"subh",style:"margin-top:0;","{t_first}"}
                        input{class:"ix",placeholder:"https://...",value:"{v25_first}",oninput:move|e|st.write().v25_first=e.value()}
                        div{class:"subh","{t_last}"}
                        input{class:"ix",placeholder:"https://...",value:"{v25_last}",oninput:move|e|st.write().v25_last=e.value()}
                    }
                }
                if v25m==V25Mode::Reference{
                    Card{title:t_refcard,
                        div{class:"hint",style:"margin:0 0 6px;","{ref_hint}"}
                        div{class:"row",
                            input{class:"ix",style:"flex:1;",placeholder:"https://...",value:"{v25_in}",oninput:move|e|st.write().v25_input=e.value()}
                        }
                        div{class:"row",style:"margin-top:6px;",
                            button{class:"g",onclick:move|_|{
                                let v=st.read().v25_input.trim().to_string();
                                if!v.is_empty(){st.write().v25_images.push(v);st.write().v25_input.clear();}
                            },"{t_addimg}"}
                            button{class:"g",onclick:move|_|{
                                let v=st.read().v25_input.trim().to_string();
                                if!v.is_empty(){st.write().v25_audios.push(v);st.write().v25_input.clear();}
                            },"{t_addaud}"}
                            if!v25flash{
                                button{class:"g",onclick:move|_|{
                                    let v=st.read().v25_input.trim().to_string();
                                    if!v.is_empty(){st.write().v25_videos.push(v);st.write().v25_input.clear();}
                                },"{t_addvid}"}
                            }
                        }
                        {v25_items.into_iter()}
                    }
                }
                Card{title:t_fmt,
                    div{class:"subh",style:"margin-top:0;","{t_ar}"}
                    select{class:"sel",onchange:move|e|{if let Ok(i)=e.value().parse::<usize>(){st.write().v25_ar_index=i;}},value:"{ar25idx}",{ar25opts.into_iter()}}
                    div{class:"subh","{t_secs}"}
                    select{class:"sel",onchange:move|e|{if let Ok(i)=e.value().parse::<usize>(){st.write().v25_seconds_index=i;}},value:"{s25idx}",{s25opts.into_iter()}}
                }
            }else{
                // ── Video V2.0 参数 ──
                if st.read().vmode!=VMode::Text{
                    Card{title:t_imgcard,
                        div{class:"hint",style:"margin:0 0 6px;","{t_imghint}"}
                        div{class:"row",
                            input{class:"ix",style:"flex:1;",placeholder:"https://...",value:"{url_input}",oninput:move|e|st.write().video_url_input=e.value()}
                            button{class:"g",onclick:move|_|{
                                let v=st.read().video_url_input.trim().to_string();
                                if!v.is_empty(){st.write().video_image_urls.push(v);st.write().video_url_input.clear();}
                            },"{t_add}"}
                        }
                        {url_items.into_iter()}
                    }
                }

                Card{title:t_sizecard,
                    select{class:"sel",onchange:move|e|{if let Ok(i)=e.value().parse::<usize>(){st.write().vsize_index=i;}},value:"{sidx}",{sopts.into_iter()}}
                    if sidx==VIDEO_SIZE_PRESETS.len()-1{
                        div{class:"row",style:"margin-top:6px;",
                            input{class:"ix",style:"width:80px;text-align:center;",r#type:"number",min:64,max:4096,value:"{st.read().vw_custom}",oninput:move|e|{if let Ok(v)=e.value().parse::<i32>(){st.write().vw_custom=v.clamp(64,4096);}}}
                            span{style:"color:var(--text2);","×"}
                            input{class:"ix",style:"width:80px;text-align:center;",r#type:"number",min:64,max:4096,value:"{st.read().vh_custom}",oninput:move|e|{if let Ok(v)=e.value().parse::<i32>(){st.write().vh_custom=v.clamp(64,4096);}}}
                        }
                    }
                    div{class:"hint",style:"margin-top:6px;","{cur_hint}"}
                }

                Card{title:t_durcard,
                    select{class:"sel",onchange:move|e|{if let Ok(i)=e.value().parse::<usize>(){st.write().vduration_index=i;}},value:"{didx}",{dopts.into_iter()}}
                    if didx==VIDEO_DURATION_PRESETS.len(){
                        div{class:"row",style:"margin-top:6px;",
                            input{class:"ix",style:"width:90px;text-align:center;",r#type:"number",min:1,max:441,value:"{st.read().vframes_custom}",oninput:move|e|{if let Ok(v)=e.value().parse::<i32>(){st.write().vframes_custom=v.clamp(1,441);}}}
                            span{style:"color:var(--text2);font-size:12px;","{t_frames}"}
                        }
                    }
                    div{class:"hint",style:"margin-top:6px;","{est_hint}"}
                }
            }
            }

            // 生成按钮固定在侧栏底部，永远可见，不会被历史栏遮挡
            div{class:"side-action",
                button{class:"b2",disabled:loading,onclick:move|_|on_gen_video(st.clone()),"{gen_btn}"}
            }
        }
    }
}

// ── 视频主区 ──────────────────────────────────────────────────────────────────

#[component]
fn VideoMainArea(st:Signal<AppState>)->Element{
    let L=st.read().lang;
    if st.read().video_loading{
        let elapsed=st.read().video_elapsed;
        let progress=st.read().video_progress;
        let msg=st.read().video_msg.clone();
        let err=st.read().video_error.clone();
        let bar=progress.clamp(0.0,100.0) as f32;
        let el_txt=i18n::tf(L,"vid.elapsed",&[("s",&format!("{elapsed:.1}")),("p",&format!("{progress:.0}"))]);
        let err_el=if!err.is_empty(){Some(rsx!{span{class:"warnerr","⚠ {err}"}})}else{None};
        let wait_txt=i18n::t(L,"vid.wait").to_string();
        return rsx!{
            div{class:"center",
                div{class:"spinner",div{class:"spin"}}
                span{class:"loadtitle","{msg}"}
                span{class:"loadsub","{el_txt}"}
                div{class:"bar",div{class:"barfill",style:"width:{bar}%;"}}
                {err_el}
                span{class:"loadsub",style:"margin-top:10px;","{wait_txt}"}
            }
        };
    }

    // 错误统一在主生成区展示，左侧不再显示任何报错
    let err=st.read().video_error.clone();
    if!err.is_empty(){
        return rsx!{ErrorCard{st:st.clone(),error:err,on_dismiss:move|_|st.write().video_error.clear()}};
    }

    let has_vid=!st.read().videos.is_empty();
    if!has_vid{
        let emptyt_txt=i18n::t(L,"vid.emptyt").to_string();
        let empty_txt=i18n::t(L,"vid.empty").to_string();
        return rsx!{
            div{class:"center",
                div{class:"emptylogo","🎬"}
                span{class:"emptyt","{emptyt_txt}"}
                span{class:"emptys","{empty_txt}"}
            }
        };
    }

    let sel=cmp::min(st.read().video_selected,st.read().videos.len().saturating_sub(1));
    let entry=st.read().videos[sel].clone();
    rsx!{VideoViewer{key:"{sel}",st:st.clone(),entry:entry,index:sel}}
}

#[component]
fn VideoViewer(st:Signal<AppState>,entry:CachedVideo,index:usize)->Element{
    let L=st.read().lang;
    let notice=st.read().notice.clone();
    let has_url=!entry.video_url.is_empty();
    let url_act=entry.video_url.clone();
    let info=i18n::tf(L,"vid.info",&[("size",&entry.size),("model",&entry.model),("sec",&entry.seconds)]);

    let reg_txt=format!("🔄 {}",i18n::t(L,"act.reg"));
    let open_txt=format!("🌐 {}",i18n::t(L,"act.openvid"));
    let save_txt=format!("💾 {}",i18n::t(L,"act.save"));
    let pline=format!("{}  {}",i18n::t(L,"viewer.prompt"),entry.prompt);

    let mut ar:Vec<Element>=vec![];
    ar.push(rsx!{span{class:"mmodel","{entry.model}"}});
    ar.push(rsx!{span{class:"minfo","{info}"}});
    ar.push(rsx!{div{style:"flex:1;"}});
    ar.push(rsx!{button{class:"g",onclick:move|_|on_gen_video(st.clone()),"{reg_txt}"}});
    if has_url{ar.push(rsx!{button{class:"g",onclick:move|_|open_url(&url_act),"{open_txt}"}});}
    ar.push(rsx!{button{class:"g",onclick:move|_|do_save_video(&mut st.write()),"{save_txt}"}});

    let ntel=notice.map(|n|{
        let cls=nt_class(n.kind);let txt=n.text;
        rsx!{div{class:"{cls}","{txt}"}}
    });

    // 视频链接是公开 https URL，WebView2 可直接原生流式播放（与浏览器同机制）
    let vid_id=format!("agnes-vid-{index}");
    let src=entry.video_url.clone();

    rsx!{
        div{class:"main",
            div{class:"stage",
                div{class:"vidwrap",
                    video{id:"{vid_id}",src:"{src}",controls:true,preload:"auto",style:"max-width:70vw;max-height:calc(100vh - 300px);border-radius:8px;display:block;background:#000;"}
                }
            }
            div{class:"meta",
                div{class:"metatop",{ar.into_iter()}}
                hr{class:"divider"}
                div{class:"mprompt","{pline}"}
                {ntel}
            }
        }
    }
}


fn pick_file(tx:EventTx,_st:Signal<AppState>){
    let tx2=tx.0.clone();
    std::thread::spawn(move||{
        let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let r=rt.block_on(async{
            let f=rfd::AsyncFileDialog::new().add_filter("图片",&["png","jpg","jpeg","webp","bmp"]).pick_file().await?;
            let nm=f.file_name().to_string();let d=tokio::fs::read(f.path()).await.ok()?;
            let ex=std::path::Path::new(&nm).extension().and_then(|e|e.to_str()).map(|s|s.to_lowercase());
            let m=match ex.as_deref(){Some("png")=>"image/png",Some("jpg")|Some("jpeg")=>"image/jpeg",Some("webp")=>"image/webp",Some("bmp")=>"image/bmp",_=>"image/png"};
            let b64=base64::engine::general_purpose::STANDARD.encode(&d);Some((nm,format!("data:{m};base64,{b64}")))
        });let _=tx2.send(BgEvent::FilePicked(r));
    });
}
fn browse_dir(tx:EventTx){
    let tx2=tx.0.clone();
    std::thread::spawn(move||{
        let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let r=rt.block_on(async{rfd::AsyncFileDialog::new().pick_folder().await.map(|d|d.path().to_string_lossy().to_string())});
        let _=tx2.send(BgEvent::DirPicked(r));
    });
}
