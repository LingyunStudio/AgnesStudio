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
use crate::updater::{self, UpdateInfo};

const MODELS: &[(&str, &str)] = &[
    ("Agnes Image 2.1 Flash（默认）","agnes-image-2.1-flash"),
    ("Agnes Image 2.0 Flash","agnes-image-2.0-flash"),
];
const SIZE_PRESETS: &[(&str, &str)] = &[
    ("1024 × 1024（方形）","1024x1024"),("1024 × 768（横版）","1024x768"),
    ("768 × 1024（竖版）","768x1024"),("1280 × 720（HD 横）","1280x720"),
    ("720 × 1280（HD 竖）","720x1280"),("1920 × 1080（FHD 横）","1920x1080"),
    ("1080 × 1920（FHD 竖）","1080x1920"),("2048 × 2048（2K 方）","2048x2048"),
    ("2560 × 1440（2K 横）","2560x1440"),("3840 × 2160（4K 横）","3840x2160"),
    ("2160 × 3840（4K 竖）","2160x3840"),("自定义",""),
];

// ── 视频 ──────────────────────────────────────────────────────────────────────
const VIDEO_SIZE_PRESETS: &[(&str, i32, i32)] = &[
    ("16:9 横版（1280×720）", 1280, 720),
    ("9:16 竖版（720×1280）", 720, 1280),
    ("1:1 方形（720×720）", 720, 720),
    ("4:3 横版（1024×768）", 1024, 768),
    ("3:4 竖版（768×1024）", 768, 1024),
    ("自定义", 0, 0),
];
// (名称, num_frames, frame_rate, 秒数说明)
const VIDEO_DURATION_PRESETS: &[(&str, i32, i32, &str)] = &[
    ("约 3 秒（81 帧）", 81, 24, "3"),
    ("约 5 秒（121 帧）", 121, 24, "5"),
    ("约 10 秒（241 帧）", 241, 24, "10"),
    ("约 18 秒（441 帧）", 441, 24, "18"),
];
const VIDEO_MODELS: &[(&str, &str)] = &[("Agnes Video V2.0", "agnes-video-v2.0")];

#[derive(Clone,Copy,PartialEq)]
enum Workspace{Image,Video}
#[derive(Clone,Copy,PartialEq)]
enum VMode{Text,Image,Multi,Keyframes}

#[derive(Clone,PartialEq)]
enum Mode{Text,Image}
#[derive(Clone,PartialEq)]
enum InputSrc{File,Url}
#[derive(Clone,PartialEq)]
enum OutFmt{Url,B64}
enum BgEvent{
    ImageDone{bytes:Vec<u8>,url:Option<String>,prompt:String,model:String,size:String},
    Error(String),FilePicked(Option<(String,String)>),DirPicked(Option<String>),
    VideoCreated{video_id:String,task_id:String,seconds:String,size:String},
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
    error:String,notice_text:String,notice_color:String,
    prompt:String,mode:Mode,out_fmt:OutFmt,model_index:usize,size_preset_index:usize,
    custom_w:i32,custom_h:i32,input_src:InputSrc,input_url:String,input_file:Option<(String,String)>,
    api_key_visible:bool,show_popup:bool,popup_uri:String,popup_dims:[usize;2],
    popup_zoom:f32,popup_pan:[f32;2],
    gen_elapsed:f32,bg_tx:EventTx,bg_rx:Arc<Mutex<mpsc::Receiver<BgEvent>>>,
    // 视频工作台
    workspace:Workspace,
    videos:Vec<CachedVideo>,video_selected:usize,
    video_loading:bool,video_error:String,video_elapsed:f32,
    video_progress:f32,video_msg:String,video_job:Option<VideoJob>,
    video_prompt:String,video_neg:String,
    vsize_index:usize,vw_custom:i32,vh_custom:i32,
    vduration_index:usize,vframes_custom:i32,vfps_custom:i32,
    vmode:VMode,video_image_urls:Vec<String>,video_url_input:String,
    video_store:Arc<Mutex<HashMap<usize,Arc<Vec<u8>>>>>,
    // 自动更新
    update_info:Option<UpdateInfo>,
    update_checking:bool,
    update_downloading:bool,
    update_progress:u32,
    update_error:String,
    show_update_dialog:bool,
}

fn raw_to_data_uri(b:&[u8])->Result<String,String>{
    let img=image::load_from_memory(b).map_err(|e|format!("解码失败：{e}"))?;
    let rgba=img.to_rgba8();let mut out=std::io::Cursor::new(Vec::new());
    image::write_buffer_with_format(&mut out,&rgba,img.width(),img.height(),image::ExtendedColorType::Rgba8,image::ImageFormat::Png).map_err(|e|format!("编码失败：{e}"))?;
    let b64=base64::engine::general_purpose::STANDARD.encode(out.into_inner());
    Ok(format!("data:image/png;base64,{b64}"))
}
fn resolved_size(s:&AppState)->String{
    if s.size_preset_index<SIZE_PRESETS.len()-1{SIZE_PRESETS[s.size_preset_index].1.to_string()}else{format!("{}x{}",s.custom_w,s.custom_h)}
}
fn cur_input(s:&AppState)->Option<String>{
    if s.mode!=Mode::Image{return None}
    match s.input_src{InputSrc::File=>s.input_file.as_ref().map(|(_,d)|d.clone()),InputSrc::Url=>{let t=s.input_url.trim();if t.is_empty(){None}else{Some(t.to_string())}}}
}
fn set_defaults(s:&mut AppState){
    s.prompt=s.cfg.last_prompt.clone();s.mode=if s.cfg.mode=="image"{Mode::Image}else{Mode::Text};
    s.out_fmt=if s.cfg.output_format=="b64_json"{OutFmt::B64}else{OutFmt::Url};
    s.model_index=MODELS.iter().position(|(_,id)|*id==s.cfg.model.as_str()).unwrap_or(0);
    let sz=s.cfg.last_size.clone();
    if let Some(idx)=SIZE_PRESETS.iter().position(|(_,v)|*v==sz.as_str()){s.size_preset_index=idx;}
    else if let Some((w,h))=sz.split_once('x'){if let(Ok(w),Ok(h))=(w.trim().parse::<i32>(),h.trim().parse::<i32>()){s.custom_w=w.clamp(64,4096);s.custom_h=h.clamp(64,4096);s.size_preset_index=SIZE_PRESETS.len()-1;}}
}
fn do_save(s:&mut AppState){
    if s.images.is_empty(){return}
    let sel=cmp::min(s.selected,s.images.len()-1);let raw=s.images[sel].raw_bytes.clone();
    let ex=match image::guess_format(&raw).unwrap_or(image::ImageFormat::Png){image::ImageFormat::Png=>"png",image::ImageFormat::Jpeg=>"jpg",image::ImageFormat::WebP=>"webp",image::ImageFormat::Bmp=>"bmp",_=>"png"};
    let secs=SystemTime::now().duration_since(UNIX_EPOCH).map(|d|d.as_secs()).unwrap_or(0);
    let fname=format!("agnes_{secs}.{ex}");let dir=PathBuf::from(&s.cfg.save_dir);
    if std::fs::create_dir_all(&dir).is_err(){s.notice("创建目录失败".to_string(),"#e14646");return;}
    let path=dir.join(&fname);
    match std::fs::write(&path,&raw){Ok(_)=>s.notice(format!("已保存：{}",path.display()),"#2eb478"),Err(e)=>s.notice(format!("保存失败：{e}"),"#e14646")}
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
    if std::fs::create_dir_all(&dir).is_err(){s.notice("创建目录失败".to_string(),"#e14646");return;}

    // 缓存字节有效则直接写盘
    if is_valid_mp4(&entry.bytes){
        let secs=SystemTime::now().duration_since(UNIX_EPOCH).map(|d|d.as_secs()).unwrap_or(0);
        let path=dir.join(format!("agnes_video_{secs}.mp4"));
        match std::fs::write(&path,&entry.bytes){Ok(_)=>s.notice(format!("已保存：{}",path.display()),"#2eb478"),Err(e)=>s.notice(format!("保存失败：{e}"),"#e14646")}
        return;
    }

    // 缓存字节无效（可能下载时被 CDN 返回错误页）：用远程 URL 重新下载
    if entry.video_url.is_empty(){s.notice("视频地址为空，无法保存".to_string(),"#e14646");return;}
    s.notice("本地缓存无效，正在重新下载视频…".to_string(),"#7c5cff");
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
    s.vsize_index=VIDEO_SIZE_PRESETS.iter().position(|(_,w,h)|*w==s.cfg.video_width&&*h==s.cfg.video_height).unwrap_or(0);
    if s.vsize_index==VIDEO_SIZE_PRESETS.len()-1{s.vw_custom=s.cfg.video_width.clamp(64,4096);s.vh_custom=s.cfg.video_height.clamp(64,4096);}
    s.vduration_index=s.cfg.video_duration_preset.min(VIDEO_DURATION_PRESETS.len()-1);
    s.vframes_custom=s.cfg.video_num_frames;s.vfps_custom=s.cfg.video_frame_rate;
    s.vmode=match s.cfg.video_mode.as_str(){"image"=>VMode::Image,"multi"=>VMode::Multi,"keyframes"=>VMode::Keyframes,_=>VMode::Text};
}

// 视频字节是否为有效 mp4（4~8 字节含 "ftyp"）
fn is_valid_mp4(b:&[u8])->bool{
    b.len()>12&&(&b[4..8]==b"ftyp")&&b[0]==0&&b[1]==0&&b[2]==0
}

impl AppState{fn notice(&mut self,t:String,c:&str){self.notice_text=t;self.notice_color=c.to_string();}}

// 手动触发检查更新（设置卡片按钮）：检查中显示状态，失败显示错误
fn CheckUpdate(mut st:Signal<AppState>){
    if st.read().update_checking{return;}
    st.write().update_checking=true;
    st.write().update_error.clear();
    let tx=st.read().bg_tx.0.clone();
    tokio::spawn(async move{
        match updater::check_latest().await{
            Ok(Some(info))=>{let _=tx.send(BgEvent::UpdateFound(info));}
            Ok(None)=>{let _=tx.send(BgEvent::UpdateNone);let _=tx.send(BgEvent::UpdateCheckFailed("当前已是最新版本".to_string()));}
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
    let mut init=AppState{cfg,images:vec![],selected:0,loading:false,error:String::new(),notice_text:String::new(),notice_color:String::new(),prompt:String::new(),mode:Mode::Text,out_fmt:OutFmt::Url,model_index:0,size_preset_index:0,custom_w:1024,custom_h:1024,input_src:InputSrc::File,input_url:String::new(),input_file:None,api_key_visible:false,show_popup:false,popup_uri:String::new(),popup_dims:[0,0],popup_zoom:1.0,popup_pan:[0.0,0.0],gen_elapsed:0.0,bg_tx:EventTx(bg_tx),bg_rx,workspace:Workspace::Image,videos:vec![],video_selected:0,video_loading:false,video_error:String::new(),video_elapsed:0.0,video_progress:0.0,video_msg:String::new(),video_job:None,video_prompt:String::new(),video_neg:String::new(),vsize_index:0,vw_custom:1152,vh_custom:768,vduration_index:1,vframes_custom:121,vfps_custom:24,vmode:VMode::Text,video_image_urls:vec![],video_url_input:String::new(),video_store:Arc::new(Mutex::new(HashMap::new())),
        update_info:None,update_checking:false,update_downloading:false,update_progress:0,update_error:String::new(),show_update_dialog:false};
    set_defaults(&mut init);set_video_defaults(&mut init);let st=use_signal(||init);

    {let mut s2=st.clone();use_future(move||async move{loop{let evs={let state=s2.read();let rx=state.bg_rx.clone();let evs=rx.lock().unwrap().try_iter().collect::<Vec<_>>();evs};for ev in evs{let mut s=s2.write();match ev{
        BgEvent::ImageDone{bytes,url,prompt,model,size}=>{s.loading=false;match raw_to_data_uri(&bytes){Ok(uri)=>match image::load_from_memory(&bytes){Ok(img)=>{s.images.push(CachedImage{data_uri:uri,url,prompt,model,size,dims:[img.width()as usize,img.height()as usize],raw_bytes:bytes});s.selected=s.images.len()-1;s.error.clear();},Err(e)=>s.error=format!("解码图片失败：{e}")},Err(e)=>s.error=e}}
        BgEvent::Error(e)=>{s.loading=false;s.error=e}
        BgEvent::FilePicked(f)=>{s.input_file=f}
        BgEvent::DirPicked(d)=>{if let Some(d)=d{s.cfg.save_dir=d}}
        BgEvent::VideoCreated{video_id,task_id,seconds,size}=>{
            s.video_job=Some(VideoJob{video_id,task_id,prompt:s.video_prompt.clone(),model:VIDEO_MODELS[0].1.to_string()});
            s.video_progress=1.0;s.video_msg="任务已创建，等待生成…".to_string();
            s.video_error.clear();
            let _=(seconds,size);
        }
        BgEvent::VideoStatus{done,failed,progress,message,seconds,size,transient}=>{
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
                if done{s.video_msg="下载完成".to_string();}
            }
        }
        BgEvent::VideoReady{bytes,video_url,prompt,model,seconds,size}=>{
            s.video_loading=false;s.video_error.clear();
            let idx=s.videos.len();
            let arc=Arc::new(bytes);
            if let Ok(mut g)=s.video_store.lock(){g.insert(idx,arc.clone());}
            s.videos.push(CachedVideo{video_url,bytes:arc.to_vec(),data_uri:String::new(),prompt,model,size,seconds});
            s.video_selected=s.videos.len()-1;
            s.video_progress=100.0;s.video_msg="生成完成".to_string();
        }
        BgEvent::VideoSaved{index,path}=>{
            // 重新下载成功后，顺便用有效字节更新缓存
            if let Ok(b)=std::fs::read(&path){if is_valid_mp4(&b)&&index<s.videos.len(){s.videos[index].bytes=b;}}
            s.notice(format!("已保存：{path}"),"#2eb478");
        }
        BgEvent::VideoSaveFailed{index,error}=>{
            let _=index;
            s.notice(format!("保存失败：{error}"),"#e14646");
        }
        BgEvent::UpdateFound(info)=>{s.update_checking=false;s.update_info=Some(info);s.show_update_dialog=true;}
        BgEvent::UpdateNone=>{s.update_checking=false;}
        BgEvent::UpdateCheckFailed(e)=>{s.update_checking=false;s.update_error=e;}
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
            let(job,key,loading,prompt,mdl)={let s=s2.read();(s.video_job.clone(),s.cfg.api_key.clone(),s.video_loading,s.video_prompt.clone(),VIDEO_MODELS[0].1.to_string())};
            if loading&&job.is_some(){
                let j=job.unwrap();
                match api::fetch_video_status(&key,&j.video_id,&j.task_id).await{
                    Ok(st_)=>{
                        // 成功拿到状态：重置退避
                        interval=5;fails=0;
                        if st_.failed{
                            let _=s2.write().bg_tx.0.send(BgEvent::VideoStatus{done:false,failed:true,progress:st_.progress,message:st_.message,seconds:st_.seconds,size:st_.size,transient:false});
                        }else if st_.done{
                            if let Some(url)=st_.video_url{
                                let _=s2.write().bg_tx.0.send(BgEvent::VideoStatus{done:true,failed:false,progress:100.0,message:"正在下载视频…".to_string(),seconds:st_.seconds.clone(),size:st_.size.clone(),transient:false});
                                match api::download_video(&key,&url).await{
                                    Ok(b)=>{let _=s2.write().bg_tx.0.send(BgEvent::VideoReady{bytes:b,video_url:url,prompt,model:mdl,seconds:st_.seconds,size:st_.size});}
                                    Err(e)=>{let _=s2.write().bg_tx.0.send(BgEvent::VideoStatus{done:false,failed:true,progress:0.0,message:e,seconds:String::new(),size:String::new(),transient:false});}
                                }
                            }else{let _=s2.write().bg_tx.0.send(BgEvent::VideoStatus{done:false,failed:true,progress:0.0,message:"任务完成但未返回视频地址".to_string(),seconds:String::new(),size:String::new(),transient:false});}
                        }else{
                            let _=s2.write().bg_tx.0.send(BgEvent::VideoStatus{done:false,failed:false,progress:st_.progress,message:st_.message,seconds:st_.seconds,size:st_.size,transient:false});
                        }
                    }
                    Err(e)=>{
                        // 暂时性错误（429 / 网络抖动）：不中断，渐进退避
                        fails+=1;
                        interval=(interval*2).min(30);
                        let hint=if e.contains("429")||e.to_lowercase().contains("rate limit"){
                            format!("查询限流，{interval} 秒后重试…")
                        }else{format!("查询失败，{interval} 秒后重试…")};
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

    rsx!{
        div{style:"display:flex;flex-direction:column;height:100vh;background:#f4f5fa;font-family:system-ui,-apple-system,'Segoe UI',Roboto,sans-serif;",
            TopBar{st:st.clone()}
            div{style:"display:flex;flex:1;overflow:hidden;",
                if st.read().workspace==Workspace::Video{
                    VideoSidePanel{st:st.clone()}
                    VideoMainArea{st:st.clone()}
                }else{
                    SidePanel{st:st.clone()}
                    MainArea{st:st.clone()}
                }
            }
            HistoryBar{st:st.clone()}
            PreviewModal{st:st.clone()}
            UpdateDialog{st:st.clone()}
            style{"{CSS}"}
        }
    }
}

const CSS:&str=r#".k{background:#fff;border:1px solid #e8eaf2;border-radius:14px;padding:16px;margin:6px 2px;box-shadow:0 2px 10px rgba(0,0,0,0.04)}
.kd{display:flex;align-items:center;margin-bottom:8px;gap:6px}.kc{width:6px;height:6px;border-radius:3px;background:#7c5cff}.kt{font-size:13px;font-weight:700;color:#1c1e2e}
.h{font-size:13px;font-weight:700;color:#1c1e2e;margin-bottom:8px}
.g{padding:6px 12px;border:1px solid #e8eaf2;border-radius:9px;background:transparent;color:#828698;font-size:13px;cursor:pointer;white-space:nowrap}
.sg{display:flex;gap:4px;background:#f4f5fa;border-radius:10px;padding:4px}.s1{flex:1;padding:6px 12px;border:none;border-radius:7px;font-size:13px;font-weight:600;text-align:center}
.sel{width:100%;padding:8px 10px;border:1px solid #e8eaf2;border-radius:10px;font-size:13px;color:#1c1e2e;background:#f8f9fd;outline:none}
.ta{width:100%;min-height:100px;padding:10px;border:1px solid #e8eaf2;border-radius:10px;font-size:13.5px;color:#1c1e2e;background:#f8f9fd;resize:vertical;outline:none;box-sizing:border-box;font-family:inherit}
.ix{width:100%;padding:8px 10px;border:1px solid #e8eaf2;border-radius:10px;font-size:13px;color:#1c1e2e;background:#f8f9fd;outline:none;box-sizing:border-box}
.b2{width:100%;height:44px;border:none;border-radius:11px;background:#7c5cff;color:#fff;font-size:15px;font-weight:700;cursor:pointer}
.er{font-size:12.5px;color:#e14646;margin-top:4px}
@keyframes s{to{transform:rotate(360deg)}}@keyframes m{0%{transform:translateX(-100%)}100%{transform:translateX(350%)}}
"#;

// ── TopBar ──────────────────────────────────────────────────────────────────────

#[component]
fn TopBar(st:Signal<AppState>)->Element{
    let key_ok=!st.read().cfg.api_key.trim().is_empty();
    let ws=st.read().workspace;
    let mdl=if ws==Workspace::Video{
        VIDEO_MODELS.get(0).map(|m|m.1).unwrap_or("").to_string()
    }else{
        MODELS.get(st.read().model_index).map(|m|m.1).unwrap_or("").to_string()
    };
    let img_loading=st.read().loading;
    let vid_loading=st.read().video_loading;
    let loading=match ws{Workspace::Image=>img_loading,Workspace::Video=>vid_loading};
    let dot_c=if key_ok{"#2eb478"}else{"#e14646"};
    let txc=if key_ok{"#828698"}else{"#e14646"};
    let txt=if key_ok{"API Key 已就绪"}else{"未设置 API Key"};
    let gen_el=if loading{Some(rsx!{span{style:"font-size:12.5px;color:#7c5cff;margin-right:14px;","● 生成中"}})}else{None};

    let upd_el=if st.read().update_info.is_some(){
        let ver=st.read().update_info.as_ref().unwrap().version.clone();
        Some(rsx!{
            span{style:"font-size:12px;color:#7c5cff;background:#ebe6ff;padding:2px 8px;border-radius:4px;cursor:pointer;margin-right:10px;",
                onclick:move|_|st.write().show_update_dialog=true,
                "● 有新版本 v{ver}"
            }
        })
    }else{None};

    let on_img=move|_|st.write().workspace=Workspace::Image;
    let on_vid=move|_|st.write().workspace=Workspace::Video;
    let is_img=ws==Workspace::Image;
    let img_bg=if is_img{"#7c5cff"}else{"transparent"};
    let img_c=if is_img{"#fff"}else{"#828698"};
    let vid_bg=if!is_img{"#7c5cff"}else{"transparent"};
    let vid_c=if!is_img{"#fff"}else{"#828698"};

    rsx!{
        div{style:"display:flex;align-items:center;height:56px;padding:0 20px;background:#ffffff;box-shadow:0 1px 6px rgba(0,0,0,0.04);flex-shrink:0;gap:10px;",
            img{src:"/icon",alt:"AgnesStudio",style:"width:22px;height:22px;flex-shrink:0;"}
            span{style:"font-size:16px;font-weight:700;color:#1c1e2e;","AgnesStudio"}
            span{style:"font-size:11px;color:#7c5cff;background:#ebe6ff;padding:2px 8px;border-radius:4px;","{mdl}"}
            div{style:"width:8px;"}
            div{style:"display:flex;gap:2px;background:#f4f5fa;border-radius:10px;padding:4px;",
                span{style:"padding:5px 16px;border:none;border-radius:7px;font-size:13px;font-weight:600;text-align:center;cursor:pointer;color:{img_c};background:{img_bg};",onclick:on_img,"🖼 图片"}
                span{style:"padding:5px 16px;border:none;border-radius:7px;font-size:13px;font-weight:600;text-align:center;cursor:pointer;color:{vid_c};background:{vid_bg};",onclick:on_vid,"🎬 视频"}
            }
            div{style:"flex:1;"}
            span{style:"font-size:12px;color:#828698;cursor:pointer;text-decoration:underline;",onclick:move|_|open_url("https://agnes-ai.com/"),"Agnes 官网"}
            span{style:"font-size:12px;color:#828698;cursor:pointer;text-decoration:underline;margin-left:14px;",onclick:move|_|open_url("https://github.com/LingyunStudio/AgnesStudio"),"GitHub"}
            {upd_el}
            {gen_el}
            div{style:"width:8px;height:8px;border-radius:4px;background:{dot_c};flex-shrink:0;"}
            span{style:"font-size:12.5px;color:{txc};","{txt}"}
        }
    }
}

// ── SidePanel ───────────────────────────────────────────────────────────────────

#[component]
fn SidePanel(st:Signal<AppState>)->Element{
    let midx=st.read().model_index;
    let loading=st.read().loading;
    let btn_op=if loading{"0.6"}else{"1"};
    let sidx=st.read().size_preset_index;
    let err=st.read().error.clone();

    let mut mopts:Vec<Element>=Vec::new();
    for(i,(nm,_))in MODELS.iter().enumerate(){mopts.push(rsx!{option{selected:i==midx,value:"{i}","{nm}"}});}
    let mut sopts:Vec<Element>=Vec::new();
    for(i,(nm,_))in SIZE_PRESETS.iter().enumerate(){sopts.push(rsx!{option{selected:i==sidx,value:"{i}","{nm}"}});}

    rsx!{
        div{style:"display:flex;flex-direction:column;width:384px;min-width:320px;max-width:480px;overflow-y:auto;padding:14px 16px;background:#ffffff;flex-shrink:0;",

            Card{title:"模型",
                select{class:"sel",onchange:move|e|{if let Ok(i)=e.value().parse::<usize>(){st.write().model_index=i;}},value:"{midx}",{mopts.into_iter()}}
                div{style:"height:8px;"}div{class:"h","模式"}
                SegBtns{st:st.clone(),sel:if matches!(st.read().mode,Mode::Image){1}else{0},opts:&["文生图","图生图"],on_set:move|i|st.write().mode=if i==0{Mode::Text}else{Mode::Image}}
            }

            Card{title:"提示词",
                textarea{class:"ta",placeholder:"描述你想生成或编辑的图像…",value:"{st.read().prompt}",oninput:move|e|st.write().prompt=e.value()}
            }

            if st.read().mode==Mode::Image{InputSection{st:st.clone()}}

            Card{title:"尺寸",
                select{class:"sel",onchange:move|e|{if let Ok(i)=e.value().parse::<usize>(){st.write().size_preset_index=i;}},value:"{sidx}",{sopts.into_iter()}}
                if sidx==SIZE_PRESETS.len()-1{CustomSize{st:st.clone()}}
                div{style:"margin-top:4px;font-size:12px;color:#828698;","当前：{resolved_size(&st.read())}"}
            }

            Card{title:"输出格式",
                SegBtns{st:st.clone(),sel:if matches!(st.read().out_fmt,OutFmt::B64){1}else{0},opts:&["URL","Base64"],on_set:move|i|st.write().out_fmt=if i==0{OutFmt::Url}else{OutFmt::B64}}
            }

            div{style:"padding:0 2px;margin-top:2px;",
                button{class:"b2",style:"opacity:{btn_op};",disabled:loading,onclick:move|_|on_gen(st.clone()),if loading{"生成中…"}else{"✨  生成图像"}}
            }
            if!err.is_empty(){div{class:"er","⚠ {err}"}}

            Card{title:"设置",SettingsBody{st:st.clone()}}
            div{style:"height:6px;"}
        }
    }
}

#[component]
fn Card(title:String,children:Element)->Element{
    rsx!{div{class:"k",div{class:"kd",div{class:"kc"}span{class:"kt","{title}"}}{children}}}
}

#[component]
fn SegBtns(st:Signal<AppState>,sel:usize,opts:&'static[&'static str],on_set:EventHandler<usize>)->Element{
    let mut btns:Vec<Element>=Vec::new();
    for(i,opt)in opts.iter().enumerate(){
        let on=i==sel;
        let c=if on{"#ffffff"}else{"#828698"};
        let b=if on{"#7c5cff"}else{"transparent"};
        btns.push(rsx!{
            span{key:"{i}",class:"s1",style:"cursor:pointer;color:{c};background:{b};",
                onclick:move|_|on_set.call(i),
                "{opt}"
            }
        });
    }
    rsx!{div{class:"sg",{btns.into_iter()}}}
}

#[component]
fn InputSection(st:Signal<AppState>)->Element{
    let srci=if matches!(st.read().input_src,InputSrc::Url){1}else{0};
    let is_file=st.read().input_src==InputSrc::File;
    let has_f=st.read().input_file.is_some();
    let fname=if let Some((ref n,_))=st.read().input_file{format!("📎 {}",n)}else{"未选择图片".to_string()};
    let tx=st.read().bg_tx.clone();

    rsx!{
        Card{title:"输入图片",
            SegBtns{st:st.clone(),sel:srci,opts:&["本地文件","图片 URL"],on_set:move|i|st.write().input_src=if i==0{InputSrc::File}else{InputSrc::Url}}
            div{style:"height:6px;"}
            if is_file{
                FileInputArea{st:st.clone(),tx:tx.clone(),has_f:has_f,fname:fname}
            }else{
                input{class:"ix",placeholder:"https://...",value:"{st.read().input_url}",oninput:move|e|st.write().input_url=e.value()}
            }
        }
    }
}

#[component]
fn CustomSize(st:Signal<AppState>)->Element{
    rsx!{
        div{style:"display:flex;gap:8px;margin-top:4px;align-items:center;",
            input{class:"ix",style:"width:80px;text-align:center;",r#type:"number",min:64,max:4096,value:"{st.read().custom_w}",oninput:move|e|{if let Ok(v)=e.value().parse::<i32>(){st.write().custom_w=v.clamp(64,4096);}}}
            span{style:"color:#828698;","×"}
            input{class:"ix",style:"width:80px;text-align:center;",r#type:"number",min:64,max:4096,value:"{st.read().custom_h}",oninput:move|e|{if let Ok(v)=e.value().parse::<i32>(){st.write().custom_h=v.clamp(64,4096);}}}
        }
    }
}

#[component]
fn SettingsBody(st:Signal<AppState>)->Element{
    let pwd=if st.read().api_key_visible{"text"}else{"password"};
    let eye=if st.read().api_key_visible{"🙈 隐藏".to_string()}else{"👁 显示".to_string()};
    let tx=st.read().bg_tx.clone();
    let upd_has_error=!st.read().update_error.is_empty();
    let upd_err_color=if st.read().update_error.contains("已是最新"){"#2eb478"}else{"#e14646"};

    rsx!{
        div{style:"display:flex;justify-content:space-between;align-items:center;",
            span{style:"font-size:12px;color:#828698;","API Key"}
            button{class:"g",onclick:move|_|{let vis=st.read().api_key_visible;st.write().api_key_visible=!vis;},"{eye}"}
        }
        input{class:"ix",r#type:"{pwd}",placeholder:"Bearer token",value:"{st.read().cfg.api_key}",oninput:move|e|st.write().cfg.api_key=e.value()}
        div{style:"height:6px;"}
        span{style:"font-size:12px;color:#828698;","保存目录"}
        div{style:"display:flex;gap:8px;margin-top:4px;",
            input{class:"ix",style:"flex:1;",value:"{st.read().cfg.save_dir}",oninput:move|e|st.write().cfg.save_dir=e.value()}
            button{class:"g",onclick:move|_|browse_dir(tx.clone()),"浏览"}
        }
        div{style:"height:8px;"}
        div{style:"display:flex;gap:8px;",
            button{class:"g",onclick:move|_|{config::save(&st.read().cfg);st.write().notice("设置已保存".to_string(),"#2eb478");},"保存设置"}
            button{class:"g",onclick:move|_|{let k=st.read().cfg.api_key.clone();st.write().cfg=config::Config::default();st.write().cfg.api_key=k;set_defaults(&mut st.write());},"恢复默认"}
        }
        div{style:"height:10px;"}
        div{style:"border-top:1px solid #e8eaf2;padding-top:10px;",
            div{style:"display:flex;align-items:center;gap:8px;",
                span{style:"font-size:12px;color:#828698;","版本 v{updater::CURRENT_VERSION}"}
                if st.read().update_checking{
                    span{style:"font-size:12px;color:#7c5cff;","检查中…"}
                }
                div{style:"flex:1;"}
                button{class:"g",onclick:move|_|CheckUpdate(st.clone()),
                    disabled:st.read().update_checking,
                    if st.read().update_checking{"检查中…"}else{"检查更新"}
                }
            }
            if upd_has_error{
                div{style:"font-size:12px;color:{upd_err_color};margin-top:6px;",
                    "{st.read().update_error}"
                }
            }
        }
    }
}

#[component]
fn FileInputArea(st:Signal<AppState>,tx:EventTx,has_f:bool,fname:String)->Element{
    rsx!{
        div{style:"display:flex;gap:8px;",
            button{class:"g",onclick:move|_|pick_file(tx.clone(),st.clone()),"选择图片…"}
            if has_f{button{class:"g",onclick:move|_|st.write().input_file=None,"清除"}}
        }
        span{style:"font-size:12.5px;color:#2eb478;","{fname}"}
    }
}

#[component]
fn HistoryButtons(st:Signal<AppState>)->Element{
    let ws=st.read().workspace;
    let prev=move|_|{
        let mut s=st.write();
        if s.workspace==Workspace::Video{
            s.video_selected=if s.video_selected==0{s.videos.len()-1}else{s.video_selected-1};
        }else{
            s.selected=if s.selected==0{s.images.len()-1}else{s.selected-1};
        }
    };
    let next=move|_|{
        let mut s=st.write();
        if s.workspace==Workspace::Video{
            s.video_selected=(s.video_selected+1)%s.videos.len();
        }else{
            s.selected=(s.selected+1)%s.images.len();
        }
    };
    let _=ws;
    rsx!{
        button{class:"g",onclick:prev,"◀"}
        span{style:"width:4px;"}
        button{class:"g",onclick:next,"▶"}
    }
}

// ── MainArea ───────────────────────────────────────────────────────────────────

#[component]
fn MainArea(st:Signal<AppState>)->Element{
    if st.read().loading{
        let elapsed=st.read().gen_elapsed;
        return rsx!{
            div{style:"flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;background:#f4f5fa;",
                div{style:"width:48px;height:48px;border-radius:24px;background:#ebe6ff;display:flex;align-items:center;justify-content:center;margin-bottom:12px;",div{style:"width:28px;height:28px;border:3px solid #7c5cff;border-top-color:transparent;border-radius:14px;animation:s .8s linear infinite;"}}
                span{style:"font-size:16px;font-weight:700;color:#1c1e2e;","生成中…"}
                span{style:"font-size:12.5px;color:#828698;margin-top:4px;","已用时 {elapsed:.1} 秒"}
                div{style:"width:240px;height:6px;background:#f5f6fc;border-radius:3px;margin-top:10px;overflow:hidden;",div{style:"width:40%;height:100%;background:#7c5cff;border-radius:3px;animation:m 1.2s ease-in-out infinite;"}}
                span{style:"font-size:12px;color:#828698;margin-top:8px;","可能需要数秒到数十秒，请稍候"}
            }
        };
    }

    let has_imgs=!st.read().images.is_empty();
    if!has_imgs{
        let err=st.read().error.clone();
        if!err.is_empty(){
            return rsx!{div{style:"flex:1;display:flex;align-items:center;justify-content:center;background:#f4f5fa;font-size:15px;color:#e14646;","⚠ {err}"}};
        }
        return rsx!{
            div{style:"flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;background:#f4f5fa;",
                div{style:"width:56px;height:56px;border-radius:28px;background:#ebe6ff;border:1.5px solid #7c5cff;margin-bottom:14px;"}
                span{style:"font-size:19px;font-weight:700;color:#1c1e2e;","欢迎使用 AgnesStudio"}
                span{style:"font-size:13px;color:#828698;margin-top:6px;","在左侧输入提示词后点击「生成图像」"}
            }
        };
    }

    let sel=cmp::min(st.read().selected,st.read().images.len().saturating_sub(1));
    let entry=st.read().images[sel].clone();
    rsx!{ImageViewer{st:st.clone(),entry:entry}}
}

#[component]
fn ImageViewer(st:Signal<AppState>,entry:CachedImage)->Element{
    let ntxt=st.read().notice_text.clone();let ncol=st.read().notice_color.clone();
    let dims=entry.dims;let has_url=entry.url.is_some();
    let url_act=entry.url.clone().unwrap_or_default();
    let d_uri=entry.data_uri.clone();

    let sz_str=format!("· {} · {}x{}",entry.size,dims[0],dims[1]);
    let mut ar:Vec<Element>=vec![];
    ar.push(rsx!{span{style:"font-size:12.5px;font-weight:700;color:#7c5cff;","{entry.model}"}});
    ar.push(rsx!{span{style:"font-size:12.5px;color:#828698;","{sz_str}"}});
    ar.push(rsx!{div{style:"flex:1;"}});
    ar.push(rsx!{button{class:"g",onclick:move|_|on_gen(st.clone()),"🔄 重新生成"}});
    if has_url{ar.push(rsx!{button{class:"g",onclick:move|_|open_url(&url_act),"🌐 打开原图"}});}
    ar.push(rsx!{button{class:"g",onclick:move|_|do_save(&mut st.write()),"💾 保存"}});

    let ntel=if!ntxt.is_empty(){Some(rsx!{div{style:"margin-top:4px;font-size:12px;color:{ncol};","{ntxt}"}})}else{None};

    rsx!{
        div{style:"flex:1;display:flex;flex-direction:column;padding:20px;overflow:hidden;background:#f4f5fa;",
            div{style:"flex:1;display:flex;align-items:center;justify-content:center;overflow:hidden;",
                div{style:"background:#ffffff;padding:10px;border-radius:14px;box-shadow:0 4px 18px rgba(0,0,0,0.06);display:inline-block;cursor:pointer;",
                    img{src:"{d_uri}",style:"max-width:70vw;max-height:calc(100vh - 300px);object-fit:contain;border-radius:8px;display:block;",
                        onclick:move|_|{let mut s=st.write();s.show_popup=true;s.popup_uri=d_uri.clone();s.popup_dims=dims;s.popup_zoom=1.0;s.popup_pan=[0.0,0.0];}
                    }
                }
            }
            div{style:"margin-top:8px;background:#ffffff;border:1px solid #e8eaf2;border-radius:14px;padding:14px;box-shadow:0 2px 10px rgba(0,0,0,0.04);",
                div{style:"display:flex;align-items:center;gap:8px;flex-wrap:wrap;",{ar.into_iter()}}
                hr{style:"border:none;border-top:1px solid #e8eaf2;margin:6px 0 4px;"}
                div{style:"font-size:12.5px;color:#1c1e2e;",span{style:"font-size:12px;color:#828698;","提示词  "}"{entry.prompt}"}
                {ntel}
            }
        }
    }
}

// ── HistoryBar ─────────────────────────────────────────────────────────────────

#[component]
fn HistoryBar(st:Signal<AppState>)->Element{
    let ws=st.read().workspace;
    if ws==Workspace::Video{
        let vids=st.read().videos.clone();
        if vids.is_empty(){return rsx!{}}
        let count=vids.len();
        let cur=st.read().video_selected;
        let mut thumbs:Vec<Element>=Vec::new();
        for(i,_v)in vids.iter().enumerate(){
            let border=if i==cur{"2.5px solid #7c5cff".to_string()}else{"2.5px solid transparent".to_string()};
            let label=format!("🎬 {}",i+1);
            thumbs.push(rsx!{
                div{key:"{i}",style:"flex-shrink:0;cursor:pointer;border-radius:8px;padding:1.5px;border:{border};",
                    onclick:move|_|st.write().video_selected=i,
                    div{style:"width:94px;height:72px;border-radius:6px;background:#1c1e2e;color:#fff;display:flex;align-items:center;justify-content:center;font-size:12px;","{label}"}
                }
            });
        }
        return rsx!{
            div{style:"flex-shrink:0;max-height:140px;background:#ffffff;box-shadow:0 -1px 6px rgba(0,0,0,0.04);padding:10px 16px;",
                div{style:"display:flex;align-items:center;margin-bottom:6px;",
                    span{style:"font-size:12.5px;color:#828698;","视频历史  {count}"}
                    div{style:"flex:1;"}
                    if count>1{HistoryButtons{st:st.clone()}}
                }
                div{style:"display:flex;flex-direction:row;flex-wrap:nowrap;gap:8px;overflow-x:auto;overflow-y:hidden;padding-bottom:4px;",{thumbs.into_iter()}}
            }
        };
    }

    if st.read().images.is_empty(){return rsx!{}}
    let count=st.read().images.len();

    let mut thumbs:Vec<Element>=Vec::new();
    for(i,img)in st.read().images.iter().enumerate(){
        let border=if i==st.read().selected{"2.5px solid #7c5cff".to_string()}else{"2.5px solid transparent".to_string()};
        let _st_h=st.clone();
        thumbs.push(rsx!{
            div{key:"{i}",style:"flex-shrink:0;cursor:pointer;border-radius:8px;padding:1.5px;border:{border};",
                onclick:move|_|st.write().selected=i,
                img{src:"{img.data_uri}",style:"width:94px;height:72px;object-fit:cover;border-radius:6px;display:block;"}
            }
        });
    }

    rsx!{
        div{style:"flex-shrink:0;max-height:140px;background:#ffffff;box-shadow:0 -1px 6px rgba(0,0,0,0.04);padding:10px 16px;",
            div{style:"display:flex;align-items:center;margin-bottom:6px;",
                span{style:"font-size:12.5px;color:#828698;","历史记录  {count}"}
                div{style:"flex:1;"}
                if count>1{HistoryButtons{st:st.clone()}}
            }
            div{style:"display:flex;flex-direction:row;flex-wrap:nowrap;gap:8px;overflow-x:auto;overflow-y:hidden;padding-bottom:4px;",{thumbs.into_iter()}}
        }
    }
}

// ── PreviewModal ───────────────────────────────────────────────────────────────

// 更新提示弹窗
#[component]
fn UpdateDialog(st:Signal<AppState>)->Element{
    if!st.read().show_update_dialog{return rsx!{}}
    let info=match st.read().update_info.clone(){Some(i)=>i,None=>return rsx!{}};
    let downloading=st.read().update_downloading;
    let progress=st.read().update_progress;
    let err=st.read().update_error.clone();
    let has_setup=info.setup_url.is_some();
    let cur=updater::CURRENT_VERSION.to_string();
    let notes_text=if info.notes.trim().is_empty(){"（此版本未提供更新说明）".to_string()}else{info.notes.clone()};
    let notes_html=render_markdown(&notes_text);
    let new_ver=info.version.clone();
    let html=info.html_url.clone();

    rsx!{
        div{style:"position:fixed;inset:0;z-index:1100;background:rgba(0,0,0,0.5);display:flex;align-items:center;justify-content:center;",
            onclick:move|_|{if!downloading{st.write().show_update_dialog=false;}},
            div{style:"background:#fff;border-radius:14px;padding:24px;width:min(92vw,520px);max-height:88vh;display:flex;flex-direction:column;box-shadow:0 8px 40px rgba(0,0,0,0.25);",
                onclick:move|e|e.stop_propagation(),
                div{style:"display:flex;align-items:center;gap:10px;margin-bottom:4px;",
                    div{style:"width:10px;height:10px;border-radius:5px;background:#7c5cff;"}
                    span{style:"font-size:18px;font-weight:700;color:#1c1e2e;","发现新版本"}
                }
                span{style:"font-size:13px;color:#828698;margin-bottom:14px;","v{cur}  →  v{new_ver}"}
                div{style:"flex:1;overflow:auto;background:#f8f9fd;border:1px solid #e8eaf2;border-radius:10px;padding:12px;margin-bottom:16px;font-size:13px;color:#1c1e2e;",
                    dangerous_inner_html:"{notes_html}"
                }
                if!err.is_empty(){
                    div{style:"font-size:12.5px;color:#e14646;margin-bottom:10px;","{err}"}
                }
                if downloading{
                    div{style:"margin-bottom:6px;",
                        div{style:"height:8px;background:#f0f1f6;border-radius:4px;overflow:hidden;",
                            div{style:"height:100%;width:{progress}%;background:#7c5cff;border-radius:4px;"}}
                        span{style:"font-size:12px;color:#828698;margin-top:6px;display:block;","正在下载更新… {progress}%"}
                    }
                    button{class:"b2",style:"opacity:0.6;cursor:default;","下载中…"}
                }
                if !downloading && has_setup{
                    button{class:"b2",onclick:move|_|StartUpdate(st.clone()),"立即更新"}
                }
                if !downloading && !has_setup{
                    button{class:"b2",onclick:move|_|open_url(&html),"前往下载"}
                }
                div{style:"height:10px;"}
                button{class:"g",style:"width:100%;",
                    onclick:move|_|st.write().show_update_dialog=false,"稍后再说"}
            }
        }
    }
}

#[component]
fn PreviewModal(st:Signal<AppState>)->Element{
    if!st.read().show_popup{return rsx!{}}
    let uri=st.read().popup_uri.clone();let dims=st.read().popup_dims;
    let zoom=st.read().popup_zoom;let pan=st.read().popup_pan;
    let title=format!("原图预览 {}x{}  ·  {:.0}%",dims[0],dims[1],zoom*100.0);

    // 拖拽平移用本地状态
    let mut dragging=use_signal(||false);
    let mut last=use_signal(||(0.0_f64,0.0_f64));

    let clamp_zoom=|z:f32|z.clamp(0.2,12.0);
    let cursor=if zoom>1.0{"grab"}else{"default"};
    let transform=format!("translate({}px,{}px) scale({})",pan[0],pan[1],zoom);

    rsx!{
        div{style:"position:fixed;inset:0;z-index:1000;background:rgba(0,0,0,0.7);display:flex;align-items:center;justify-content:center;",
            onclick:move|_|st.write().show_popup=false,
            div{style:"background:#141418;border-radius:12px;padding:16px;width:min(95vw,1400px);max-height:95vh;display:flex;flex-direction:column;box-shadow:0 8px 40px rgba(0,0,0,0.5);",
                onclick:move|e|e.stop_propagation(),
                div{style:"display:flex;justify-content:space-between;align-items:center;margin-bottom:8px;gap:8px;",
                    span{style:"font-size:14px;color:#ccc;","{title}"}
                    div{style:"flex:1;"}
                    // 缩放控件
                    div{style:"display:flex;align-items:center;gap:6px;",
                        button{class:"g",onclick:move|_|{let z=st.read().popup_zoom;st.write().popup_zoom=clamp_zoom(z/1.2);},"➖"}
                        button{class:"g",onclick:move|_|{let z=st.read().popup_zoom;st.write().popup_zoom=clamp_zoom(z*1.2);},"➕"}
                        button{class:"g",onclick:move|_|{st.write().popup_zoom=1.0;st.write().popup_pan=[0.0,0.0];},"⤢ 1:1"}
                    }
                    button{class:"g",onclick:move|_|st.write().show_popup=false,"✕"}
                }
                div{style:"flex:1;display:flex;align-items:center;justify-content:center;overflow:hidden;background:repeating-conic-gradient(#e8e8ee 0% 25%,#d4d4da 0% 50%) 0 0 / 32px 32px;",
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
                div{style:"margin-top:8px;font-size:11.5px;color:#828698;text-align:center;","滚轮缩放 · 放大后可拖动查看 · 按 ✕ 或点击空白处关闭"}
            }
        }
    }
}

// ── Handlers ────────────────────────────────────────────────────────────────────

fn on_gen(mut st:Signal<AppState>){
    let mut s=st.write();
    if s.loading{return}
    if s.cfg.api_key.trim().is_empty(){s.error="未设置 API Key，请先在\"设置\"里填写。".to_string();return}
    if s.prompt.trim().is_empty(){s.error="提示词不能为空。".to_string();return}
    if s.mode==Mode::Image&&cur_input(&s).is_none(){s.error="图生图模式下需要提供输入图片。".to_string();return}
    let size=resolved_size(&s);let input=cur_input(&s);let prompt=s.prompt.clone();
    let model=MODELS[s.model_index].1.to_string();let fmt=if s.out_fmt==OutFmt::Url{"url"}else{"b64_json"};
    s.loading=true;s.error.clear();s.notice_text.clear();s.gen_elapsed=0.0;
    s.cfg.last_prompt=s.prompt.clone();s.cfg.last_size=size.clone();s.cfg.model=model.clone();
    s.cfg.output_format=fmt.to_string();s.cfg.mode=if s.mode==Mode::Text{"text".to_string()}else{"image".to_string()};
    config::save(&s.cfg);let api_key=s.cfg.api_key.clone();drop(s);
    let tx=st.read().bg_tx.0.clone();
    std::thread::spawn(move||{
        let rt=tokio::runtime::Runtime::new().expect("rt");
        let p=api::GenParams{api_key:api_key.clone(),model:model.clone(),prompt:prompt.clone(),size:size.clone(),input_image:input.clone(),output_format:fmt.to_string()};
        match rt.block_on(api::generate(p)){Ok(r)=>{let _=tx.send(BgEvent::ImageDone{bytes:r.bytes,url:r.url,prompt:prompt.clone(),model:model.clone(),size:size.clone()});}Err(e)=>{let _=tx.send(BgEvent::Error(e));}}
    });
}
fn on_gen_video(mut st:Signal<AppState>){
    let mut s=st.write();
    if s.video_loading{return}
    if s.cfg.api_key.trim().is_empty(){s.video_error="未设置 API Key，请先在\"设置\"里填写。".to_string();return}
    if s.video_prompt.trim().is_empty(){s.video_error="提示词不能为空。".to_string();return}
    let need_imgs=s.vmode!=VMode::Text;
    let min_imgs=if s.vmode==VMode::Text{0}else if s.vmode==VMode::Image{1}else{2};
    let valid:Vec<String>=s.video_image_urls.iter().map(|u|u.trim().to_string()).filter(|u|!u.is_empty()).collect();
    if need_imgs&&valid.len()<min_imgs{
        s.video_error=if s.vmode==VMode::Image{"图生视频需要 1 张图片 URL".to_string()}else{"该模式至少需要 2 张图片 URL".to_string()};
        return;
    }
    let(w,h)=video_dims(&s);let frames=video_frames(&s);let fps=video_fps(&s);
    let prompt=s.video_prompt.clone();let neg=s.video_neg.clone();
    let keyframes=s.vmode==VMode::Keyframes;
    let images=valid.clone();
    s.video_loading=true;s.video_error.clear();s.video_elapsed=0.0;s.video_progress=0.0;
    s.video_msg="提交任务中…".to_string();s.video_job=None;
    // 持久化视频配置
    s.cfg.last_video_prompt=prompt.clone();s.cfg.video_neg_prompt=neg.clone();
    s.cfg.video_width=w;s.cfg.video_height=h;s.cfg.video_num_frames=frames;s.cfg.video_frame_rate=fps;
    s.cfg.video_duration_preset=s.vduration_index;
    s.cfg.video_mode=match s.vmode{VMode::Text=>"text".to_string(),VMode::Image=>"image".to_string(),VMode::Multi=>"multi".to_string(),VMode::Keyframes=>"keyframes".to_string()};
    config::save(&s.cfg);
    let api_key=s.cfg.api_key.clone();drop(s);
    let tx=st.read().bg_tx.0.clone();
    std::thread::spawn(move||{
        let rt=tokio::runtime::Runtime::new().expect("rt");
        let p=api::VideoParams{api_key,prompt,negative_prompt:neg,width:w,height:h,num_frames:frames,frame_rate:fps,seed:None,images,keyframes};
        match rt.block_on(api::create_video_task(&p)){
            Ok(t)=>{let _=tx.send(BgEvent::VideoCreated{video_id:t.video_id,task_id:t.task_id,seconds:t.seconds,size:t.size});}
            Err(e)=>{let _=tx.send(BgEvent::VideoStatus{done:false,failed:true,progress:0.0,message:e,seconds:String::new(),size:String::new(),transient:false});}
        }
    });
}

// ── 视频侧栏 ──────────────────────────────────────────────────────────────────

#[component]
fn VideoSidePanel(st:Signal<AppState>)->Element{
    let loading=st.read().video_loading;
    let btn_op=if loading{"0.6"}else{"1"};
    let sidx=st.read().vsize_index;
    let didx=st.read().vduration_index;
    let vm=st.read().vmode.clone();
    let err=st.read().video_error.clone();
    let ntxt=st.read().notice_text.clone();let ncol=st.read().notice_color.clone();
    let urls=st.read().video_image_urls.clone();
    let url_input=st.read().video_url_input.clone();
    let secs=video_seconds(&st.read());
    let(w,h)=video_dims(&st.read());

    let mut sopts:Vec<Element>=Vec::new();
    for(i,(nm,_,_))in VIDEO_SIZE_PRESETS.iter().enumerate(){sopts.push(rsx!{option{selected:i==sidx,value:"{i}","{nm}"}});}
    let mut dopts:Vec<Element>=Vec::new();
    for(i,(nm,_,_,_))in VIDEO_DURATION_PRESETS.iter().enumerate(){dopts.push(rsx!{option{selected:i==didx,value:"{i}","{nm}"}});}

    let mode_sel=match vm{VMode::Text=>0,VMode::Image=>1,VMode::Multi=>2,VMode::Keyframes=>3};

    let mut url_items:Vec<Element>=Vec::new();
    for(i,u)in urls.iter().enumerate(){
        let idx=i;
        let label=format!("#{}",i+1);
        url_items.push(rsx!{
            div{key:"{idx}",style:"display:flex;align-items:center;gap:6px;margin-top:4px;",
                span{style:"font-size:12px;color:#828698;flex-shrink:0;width:22px;","{label}"}
                span{style:"font-size:12px;color:#1c1e2e;flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;","{u}"}
                button{class:"g",style:"padding:2px 8px;",onclick:move|_|{st.write().video_image_urls.remove(idx);},"✕"}
            }
        });
    }

    rsx!{
        div{style:"display:flex;flex-direction:column;width:384px;min-width:320px;max-width:480px;overflow-y:auto;padding:14px 16px;background:#ffffff;flex-shrink:0;",

            Card{title:"模型",
                div{style:"font-size:13px;color:#1c1e2e;padding:4px 0;","Agnes Video V2.0"}
                div{class:"h","生成模式"}
                SegBtns{st:st.clone(),sel:mode_sel,opts:&["文生视频","图生视频","多图视频","关键帧"],on_set:move|i|st.write().vmode=match i{0=>VMode::Text,1=>VMode::Image,2=>VMode::Multi,_=>VMode::Keyframes}}
            }

            Card{title:"提示词",
                textarea{class:"ta",placeholder:"描述视频内容：[主体]+[动作]+[场景]+[镜头运动]+[光线]+[风格]",value:"{st.read().video_prompt}",oninput:move|e|st.write().video_prompt=e.value()}
                div{style:"height:6px;"}div{class:"h","反向提示词（可选）"}
                textarea{class:"ta",style:"min-height:50px;",placeholder:"需要避免的内容…",value:"{st.read().video_neg}",oninput:move|e|st.write().video_neg=e.value()}
            }

            if st.read().vmode!=VMode::Text{
                Card{title:"输入图片 URL",
                    div{style:"font-size:11.5px;color:#828698;margin-bottom:4px;","视频 API 需要公网可访问的图片 URL（不支持本地文件）"}
                    div{style:"display:flex;gap:8px;",
                        input{class:"ix",style:"flex:1;",placeholder:"https://...",value:"{url_input}",oninput:move|e|st.write().video_url_input=e.value()}
                        button{class:"g",onclick:move|_|{
                            let v=st.read().video_url_input.trim().to_string();
                            if!v.is_empty(){st.write().video_image_urls.push(v);st.write().video_url_input.clear();}
                        },"添加"}
                    }
                    {url_items.into_iter()}
                }
            }

            Card{title:"画面尺寸",
                select{class:"sel",onchange:move|e|{if let Ok(i)=e.value().parse::<usize>(){st.write().vsize_index=i;}},value:"{sidx}",{sopts.into_iter()}}
                if sidx==VIDEO_SIZE_PRESETS.len()-1{
                    div{style:"display:flex;gap:8px;margin-top:4px;align-items:center;",
                        input{class:"ix",style:"width:80px;text-align:center;",r#type:"number",min:64,max:4096,value:"{st.read().vw_custom}",oninput:move|e|{if let Ok(v)=e.value().parse::<i32>(){st.write().vw_custom=v.clamp(64,4096);}}}
                        span{style:"color:#828698;","×"}
                        input{class:"ix",style:"width:80px;text-align:center;",r#type:"number",min:64,max:4096,value:"{st.read().vh_custom}",oninput:move|e|{if let Ok(v)=e.value().parse::<i32>(){st.write().vh_custom=v.clamp(64,4096);}}}
                    }
                }
                div{style:"margin-top:4px;font-size:12px;color:#828698;","当前：{w}×{h}（API 会自动归档到 480p/720p/1080p）"}
            }

            Card{title:"时长",
                select{class:"sel",onchange:move|e|{if let Ok(i)=e.value().parse::<usize>(){st.write().vduration_index=i;}},value:"{didx}",{dopts.into_iter()}}
                if didx==VIDEO_DURATION_PRESETS.len(){
                    div{style:"display:flex;gap:8px;margin-top:4px;align-items:center;",
                        input{class:"ix",style:"width:90px;text-align:center;",r#type:"number",min:1,max:441,value:"{st.read().vframes_custom}",oninput:move|e|{if let Ok(v)=e.value().parse::<i32>(){st.write().vframes_custom=v.clamp(1,441);}}}
                        span{style:"color:#828698;font-size:12px;","帧 @ 24fps"}
                    }
                }
                div{style:"margin-top:4px;font-size:12px;color:#828698;","预计时长：{secs} 秒（num_frames 需为 8n+1，≤441）"}
            }

            div{style:"padding:0 2px;margin-top:2px;",
                button{class:"b2",style:"opacity:{btn_op};",disabled:loading,onclick:move|_|on_gen_video(st.clone()),if loading{"生成中…"}else{"🎬  生成视频"}}
            }
            if!err.is_empty(){div{class:"er","⚠ {err}"}}
            if!ntxt.is_empty(){div{style:"font-size:12px;color:{ncol};margin-top:4px;","{ntxt}"}}

            Card{title:"设置",SettingsBody{st:st.clone()}}
            div{style:"height:6px;"}
        }
    }
}

// ── 视频主区 ──────────────────────────────────────────────────────────────────

#[component]
fn VideoMainArea(st:Signal<AppState>)->Element{
    if st.read().video_loading{
        let elapsed=st.read().video_elapsed;
        let progress=st.read().video_progress;
        let msg=st.read().video_msg.clone();
        let err=st.read().video_error.clone();
        let bar=progress.clamp(0.0,100.0) as f32;
        let err_el=if!err.is_empty(){Some(rsx!{span{style:"font-size:12px;color:#e0a800;margin-top:6px;","⚠ {err}"}})}else{None};
        return rsx!{
            div{style:"flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;background:#f4f5fa;",
                div{style:"width:48px;height:48px;border-radius:24px;background:#ebe6ff;display:flex;align-items:center;justify-content:center;margin-bottom:12px;",div{style:"width:28px;height:28px;border:3px solid #7c5cff;border-top-color:transparent;border-radius:14px;animation:s .8s linear infinite;"}}
                span{style:"font-size:16px;font-weight:700;color:#1c1e2e;","{msg}"}
                span{style:"font-size:12.5px;color:#828698;margin-top:4px;","已用时 {elapsed:.1} 秒 · {progress:.0}%"}
                div{style:"width:280px;height:6px;background:#f5f6fc;border-radius:3px;margin-top:10px;overflow:hidden;",div{style:"width:{bar}%;height:100%;background:#7c5cff;border-radius:3px;transition:width .3s;"}}
                {err_el}
                span{style:"font-size:12px;color:#828698;margin-top:8px;","视频生成通常需要 1~5 分钟，请耐心等待"}
            }
        };
    }

    let has_vid=!st.read().videos.is_empty();
    if!has_vid{
        let err=st.read().video_error.clone();
        if!err.is_empty(){
            return rsx!{div{style:"flex:1;display:flex;align-items:center;justify-content:center;background:#f4f5fa;font-size:15px;color:#e14646;","⚠ {err}"}};
        }
        return rsx!{
            div{style:"flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;background:#f4f5fa;",
                div{style:"width:56px;height:56px;border-radius:28px;background:#ebe6ff;border:1.5px solid #7c5cff;margin-bottom:14px;display:flex;align-items:center;justify-content:center;font-size:24px;","🎬"}
                span{style:"font-size:19px;font-weight:700;color:#1c1e2e;","视频生成"}
                span{style:"font-size:13px;color:#828698;margin-top:6px;","在左侧输入提示词，选择尺寸与时长后点击「生成视频」"}
            }
        };
    }

    let sel=cmp::min(st.read().video_selected,st.read().videos.len().saturating_sub(1));
    let entry=st.read().videos[sel].clone();
    rsx!{VideoViewer{key:"{sel}",st:st.clone(),entry:entry,index:sel}}
}

#[component]
fn VideoViewer(st:Signal<AppState>,entry:CachedVideo,index:usize)->Element{
    let ntxt=st.read().notice_text.clone();let ncol=st.read().notice_color.clone();
    let has_url=!entry.video_url.is_empty();
    let url_act=entry.video_url.clone();
    let info=format!("· {} · {} · {}秒",entry.size,entry.model,entry.seconds);

    let mut ar:Vec<Element>=vec![];
    ar.push(rsx!{span{style:"font-size:12.5px;font-weight:700;color:#7c5cff;","{entry.model}"}});
    ar.push(rsx!{span{style:"font-size:12.5px;color:#828698;","{info}"}});
    ar.push(rsx!{div{style:"flex:1;"}});
    ar.push(rsx!{button{class:"g",onclick:move|_|on_gen_video(st.clone()),"🔄 重新生成"}});
    if has_url{ar.push(rsx!{button{class:"g",onclick:move|_|open_url(&url_act),"🌐 打开原视频"}});}
    ar.push(rsx!{button{class:"g",onclick:move|_|do_save_video(&mut st.write()),"💾 保存"}});

    let ntel=if!ntxt.is_empty(){Some(rsx!{div{style:"margin-top:4px;font-size:12px;color:{ncol};","{ntxt}"}})}else{None};

    // 视频链接是公开 https URL，WebView2 可直接原生流式播放（与浏览器同机制）
    let vid_id=format!("agnes-vid-{index}");
    let src=entry.video_url.clone();

    rsx!{
        div{style:"flex:1;display:flex;flex-direction:column;padding:20px;overflow:hidden;background:#f4f5fa;",
            div{style:"flex:1;display:flex;align-items:center;justify-content:center;overflow:hidden;",
                div{style:"background:#000;padding:10px;border-radius:14px;box-shadow:0 4px 18px rgba(0,0,0,0.06);display:inline-block;",
                    video{id:"{vid_id}",src:"{src}",controls:true,preload:"auto",style:"max-width:70vw;max-height:calc(100vh - 300px);border-radius:8px;display:block;background:#000;"}
                }
            }
            div{style:"margin-top:8px;background:#ffffff;border:1px solid #e8eaf2;border-radius:14px;padding:14px;box-shadow:0 2px 10px rgba(0,0,0,0.04);",
                div{style:"display:flex;align-items:center;gap:8px;flex-wrap:wrap;",{ar.into_iter()}}
                hr{style:"border:none;border-top:1px solid #e8eaf2;margin:6px 0 4px;"}
                div{style:"font-size:12.5px;color:#1c1e2e;",span{style:"font-size:12px;color:#828698;","提示词  "}"{entry.prompt}"}
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
