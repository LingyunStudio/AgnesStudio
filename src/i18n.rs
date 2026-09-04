// 中英双语文案。t() 取普通文案；tf() 把 {name} 占位符替换成实参。
// 静态选项（模型名、尺寸预设等）用 Bi 双语结构体，由 app.rs 的常量表持有。

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Zh,
    En,
}

impl Lang {
    /// 配置文件取值："zh" | "en"（其余按中文处理）
    pub fn from_cfg(s: &str) -> Self {
        if s.eq_ignore_ascii_case("en") {
            Lang::En
        } else {
            Lang::Zh
        }
    }
    pub fn to_cfg(self) -> &'static str {
        match self {
            Lang::Zh => "zh",
            Lang::En => "en",
        }
    }
}

/// 双语文案对
pub struct Bi {
    pub zh: &'static str,
    pub en: &'static str,
}

impl Bi {
    pub fn l(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::Zh => self.zh,
            Lang::En => self.en,
        }
    }
}

/// 在常量上下文里构造 Bi
pub const fn bi(zh: &'static str, en: &'static str) -> Bi {
    Bi { zh, en }
}

pub fn t(lang: Lang, key: &str) -> &'static str {
    let (zh, en): (&str, &str) = match key {
        // ─ 顶栏 ──
        "tab.image" => ("图片", "Image"),
        "tab.video" => ("视频", "Video"),
        "link.web" => ("Agnes 官网", "Agnes Website"),
        "key.ok" => ("API Key 已就绪", "API Key ready"),
        "key.missing" => ("未设置 API Key", "API Key not set"),
        "key.com" => ("国际站", "International"),
        "key.cn" => ("国内站", "China"),
        "key.type.default" => ("免费", "Free"),
        "key.type.enterprise" => ("企业", "Enterprise"),
        "key.type.tokenplan" => ("Token Plan", "Token Plan"),
        "key.activate" => ("启用", "Activate"),
        "key.active" => ("✓ 使用中", "✓ Active"),
        "top.gen" => ("生成中", "Generating"),
        "upd.chip" => ("有新版本 v{v}", "New version v{v}"),

        // ─ 通用卡片标题 ──
        "card.model" => ("模型", "Model"),
        "card.mode" => ("模式", "Mode"),
        "card.prompt" => ("提示词", "Prompt"),
        "card.size" => ("尺寸", "Size"),
        "card.tier" => ("档位", "Tier"),
        "card.ratio" => ("宽高比", "Aspect Ratio"),
        "card.output" => ("输出格式", "Output Format"),
        "card.input" => ("输入图片", "Input Image"),

        // ─ 图片面板 ──
        "img.t2i" => ("文生图", "Text to Image"),
        "img.i2i" => ("图生图", "Image to Image"),
        "img.ph" => ("描述你想生成或编辑的图像…", "Describe the image you want to generate or edit…"),
        "img.file" => ("本地文件", "Local File"),
        "img.url" => ("图片 URL", "Image URL"),
        "img.pick" => ("选择图片…", "Choose Image…"),
        "img.clear" => ("清除", "Clear"),
        "img.nofile" => ("未选择图片", "No image selected"),
        "img.gen" => ("生成图像", "Generate Image"),
        "img.wait" => ("生成中…", "Generating…"),
        "img.exact" => ("输出约 {exact}（{size} · {ratio}）", "Output ≈ {exact} ({size} · {ratio})"),
        "img.cur" => ("当前：{s}", "Current: {s}"),
        "img.tieropt" => ("{t} 档", "{t}"),

        // ─ 设置 ──
        "set.show" => ("👁 显示", "👁 Show"),
        "set.hide" => ("🙈 隐藏", "🙈 Hide"),
        "set.key.com" => ("国际站（.com · 美元计费）", "International (.com · USD)"),
        "set.key.cn" => ("国内站（.cn · 人民币计费）", "China (.cn · CNY)"),
        "set.key.apply" => ("官网申请 ↗", "Get a Key ↗"),
        "set.key.note" => ("两站模型能力一致，仅接口地址不同（国际站美元、国内站人民币计费），账户与余额互不互通。每站可保存 3 类密钥，点击「启用」切换当前使用的密钥——同一时刻仅一个生效，不同类型使用不同的限制池。", "Both sites offer identical models and differ only in API endpoint (USD on .com, CNY on .cn); accounts and balances are separate. Each site can store 3 key types — click \"Activate\" to switch the key in use. Only one is active at a time; each type has its own rate-limit pool."),
        "set.key.type.default" => ("免费 / 默认密钥", "Free / Default key"),
        "set.key.type.enterprise" => ("企业认证密钥", "Enterprise key"),
        "set.key.type.tokenplan" => ("Token Plan 密钥", "Token Plan key"),
        "ratelimit.title" => ("模型速率限制", "Model Rate Limits"),
        "ratelimit.btn" => ("速率限制", "Rate Limits"),
        "set.dir" => ("保存目录", "Save Directory"),
        "set.browse" => ("浏览", "Browse"),
        "set.save" => ("保存设置", "Save Settings"),
        "set.reset" => ("恢复默认", "Reset Defaults"),
        "set.theme" => ("主题", "Theme"),
        "th.light" => ("浅色", "Light"),
        "th.dark" => ("深色", "Dark"),
        "th.sys" => ("跟随系统", "System"),
        "set.lang" => ("界面语言", "Language"),
        "set.version" => ("版本 v{v}", "Version v{v}"),
        "set.check" => ("检查更新", "Check for Updates"),
        "set.checking" => ("检查中…", "Checking…"),
        "set.uptodate" => ("当前已是最新版本", "You are on the latest version"),
        "set.saved" => ("设置已保存", "Settings saved"),
        "set.title" => ("设置", "Settings"),
        "set.done" => ("完成", "Done"),

        // ─ 主区（图片） ──
        "main.wait" => ("可能需要数秒到数十秒，请稍候", "This usually takes seconds to a minute"),
        "main.welcome" => ("欢迎使用 AgnesStudio", "Welcome to AgnesStudio"),
        "main.empty" => ("在左侧输入提示词后点击「生成图像」", "Enter a prompt on the left, then click \"Generate Image\""),
        "main.elapsed" => ("已用时 {s} 秒", "Elapsed {s}s"),
        "act.reg" => ("🔄 重新生成", "🔄 Regenerate"),
        "act.openimg" => ("🌐 打开原图", "🌐 Open Original"),
        "act.openvid" => ("🌐 打开原视频", "🌐 Open Original"),
        "act.save" => ("💾 保存", "💾 Save"),
        "viewer.prompt" => ("提示词", "Prompt"),

        // ─ 历史 ──
        "hist.img" => ("历史记录 {n}", "History {n}"),
        "hist.vid" => ("视频历史 {n}", "Video History {n}"),

        // ─ 图片预览 ──
        "pv.title" => ("原图预览 {w}×{h}", "Preview {w}×{h}"),
        "pv.hint" => ("滚轮以中心缩放 · 放大后可拖动查看 · 双击图片或点击百分比复位 · 点击空白处关闭", "Scroll to zoom from center · drag to pan when zoomed · double-click to reset · click the backdrop to close"),
        "pv.out" => ("缩小", "Zoom out"),
        "pv.in" => ("放大", "Zoom in"),
        "pv.fit" => ("复位到 100%", "Reset to 100%"),
        "pv.close" => ("关闭", "Close"),

        // ─ 更新弹窗 ──
        "upd.title" => ("发现新版本", "New Version Available"),
        "upd.nonotes" => ("（此版本未提供更新说明）", "(No release notes for this version)"),
        "upd.dl" => ("正在下载更新… {p}%", "Downloading update… {p}%"),
        "upd.dling" => ("下载中…", "Downloading…"),
        "upd.now" => ("立即更新", "Update Now"),
        "upd.goto" => ("前往下载", "Get It on GitHub"),
        "upd.later" => ("稍后再说", "Later"),

        // ─ 视频面板 ──
        "card.vmode" => ("生成模式", "Mode"),
        "v25.t2v" => ("文生视频", "Text"),
        "v25.kf" => ("首尾帧", "Keyframe"),
        "v25.ref" => ("参考生成", "Reference"),
        "v20.t2v" => ("文生视频", "Text"),
        "v20.i2v" => ("图生视频", "Image"),
        "v20.multi" => ("多图视频", "Multi"),
        "v20.kf" => ("关键帧", "Keyframes"),
        "vid.neg" => ("反向提示词（可选）", "Negative Prompt (optional)"),
        "vid.negph" => ("需要避免的内容…", "Content to avoid…"),
        "vid.ph" => ("描述视频内容：[主体与场景]+[动作与变化]+[镜头语言]+[视觉风格]+[声音]", "Describe the video: [subject & scene] + [motion & change] + [camera language] + [visual style] + [sound]"),
        "vid.ph.reffull" => ("描述视频内容，可用 <Picture 1>、<Audio 1>、<Video 1> 指代参考素材…", "Describe the video; refer to inputs as <Picture 1>, <Audio 1>, <Video 1>…"),
        "vid.ph.refflash" => ("描述视频内容，可用 <Picture 1>、<Audio 1> 指代参考素材…", "Describe the video; refer to inputs as <Picture 1>, <Audio 1>…"),
        "v25.kfcard" => ("首尾帧（Keyframe）", "Keyframes"),
        "v25.kfhint" => ("首帧与尾帧至少提供一个；仅给首帧 = 从首帧开始演绎，仅给尾帧 = 向尾帧过渡", "Provide at least one of first/last frame; first only = animate from it, last only = transition toward it"),
        "v25.first" => ("首帧图片 URL", "First frame image URL"),
        "v25.last" => ("尾帧图片 URL（可选）", "Last frame image URL (optional)"),
        "v25.refcard" => ("参考素材（Reference）", "References"),
        "v25.refhint" => ("图片 / 音频 / 视频 URL 需公网可访问；提示词中用 <Picture 1>、<Audio 1>、<Video 1> 指代", "Image / audio / video URLs must be publicly accessible; refer to them as <Picture 1>, <Audio 1>, <Video 1>"),
        "v25.refhintflash" => ("图片 / 音频 URL 需公网可访问；Flash 版图片参考最多 5 张、不支持视频参考；提示词中用 <Picture 1>、<Audio 1> 指代", "Image / audio URLs must be publicly accessible; Flash allows up to 5 reference images and no video; refer to them as <Picture 1>, <Audio 1>"),
        "v25.addimg" => ("＋ 图片", "＋ Image"),
        "v25.addaud" => ("＋ 音频", "＋ Audio"),
        "v25.addvid" => ("＋ 视频", "＋ Video"),
        "v25.lblimg" => ("图 #{n}", "IMG #{n}"),
        "v25.lblaud" => ("音 #{n}", "AUD #{n}"),
        "v25.lblvid" => ("视 #{n}", "VID #{n}"),
        "v25.fmt" => ("画幅与时长", "Format & Duration"),
        "v25.ar" => ("画幅比例（分辨率 720P）", "Aspect ratio (720P)"),
        "v25.secs" => ("时长", "Duration"),
        "v25.secopt" => ("约 {s} 秒", "~{s}s"),
        "v20.imgcard" => ("输入图片 URL", "Image URLs"),
        "v20.imghint" => ("视频 API 需要公网可访问的图片 URL（不支持本地文件）", "The video API needs publicly accessible image URLs (local files are not supported)"),
        "v20.add" => ("添加", "Add"),
        "v20.sizecard" => ("画面尺寸", "Frame Size"),
        "v20.cur" => ("当前：{w}×{h}（API 会自动归档到 480p/720p/1080p）", "Current: {w}×{h} (snapped to 480p/720p/1080p)"),
        "v20.durcard" => ("时长", "Duration"),
        "v20.est" => ("预计时长：{s} 秒（num_frames 需为 8n+1，≤441）", "Est. duration: {s}s (num_frames must be 8n+1, ≤441)"),
        "v20.frames" => ("帧 @ 24fps", "frames @ 24fps"),
        "vid.gen" => ("生成视频", "Generate Video"),
        "vid.info" => ("· {size} · {model} · {sec}秒", "· {size} · {model} · {sec}s"),

        // ─ 视频任务状态 ──
        "vs.submit" => ("提交任务中…", "Submitting task…"),
        "vs.created" => ("任务已创建，等待生成…", "Task created, waiting…"),
        "vs.dl" => ("正在下载视频…", "Downloading video…"),
        "vs.dldone" => ("下载完成", "Download complete"),
        "vs.done" => ("生成完成", "Generation complete"),
        "vs.queued" => ("排队中…", "Queued…"),
        "vs.running" => ("生成中…", "Generating…"),
        "vs.fail" => ("生成失败", "Generation failed"),
        "vs.retry429" => ("查询限流，{n} 秒后重试…", "Rate limited, retrying in {n}s…"),
        "vs.retry" => ("查询失败，{n} 秒后重试…", "Query failed, retrying in {n}s…"),
        "vs.nourl" => ("任务完成但未返回视频地址", "Task completed but no video URL was returned"),

        // ─ 视频主区 ──
        "vid.emptyt" => ("视频生成", "Video Generation"),
        "vid.empty" => ("在左侧输入提示词，选择尺寸与时长后点击「生成视频」", "Enter a prompt, pick size and duration on the left, then click \"Generate Video\""),
        "vid.wait" => ("视频生成通常需要 1~5 分钟，请耐心等待", "Video generation usually takes 1–5 minutes"),
        "vid.elapsed" => ("已用时 {s} 秒 · {p}%", "Elapsed {s}s · {p}%"),

        // ─ 校验错误 ──
        "err.nokey" => ("未设置 API Key，请点击右上角 ⚙ 打开设置并填写。", "No API Key set. Click ⚙ in the top-right corner to open Settings and add one."),
        "err.noprompt" => ("提示词不能为空。", "Prompt cannot be empty."),
        "err.noinput" => ("图生图模式下需要提供输入图片。", "Image-to-image mode requires an input image."),
        "err.title" => ("无法生成", "Cannot Generate"),
        "err.open_settings" => ("⚙ 打开设置", "⚙ Open Settings"),
        "err.dismiss" => ("知道了", "Got It"),
        "err.kf" => ("首尾帧模式至少需要提供首帧或尾帧图片 URL", "Keyframe mode needs at least a first or last frame image URL"),
        "err.ref" => ("参考生成模式至少需要一种参考素材（图片 / 音频 / 视频）", "Reference mode needs at least one reference (image / audio / video)"),
        "err.flashimg" => ("Flash 版参考图片最多 5 张（当前已添加超出，请移除多余的图片）", "Flash allows at most 5 reference images; remove the extras"),
        "err.flashvid" => ("Flash 版不支持视频参考素材，请移除视频或切换到 Agnes Video 2.5", "Flash does not support video references; remove them or switch to Agnes Video 2.5"),
        "err.1img" => ("图生视频需要 1 张图片 URL", "Image-to-video needs 1 image URL"),
        "err.2img" => ("该模式至少需要 2 张图片 URL", "This mode needs at least 2 image URLs"),

        // ─ 通知 ──
        "nt.saved" => ("已保存：{p}", "Saved: {p}"),
        "nt.savefail" => ("保存失败：{e}", "Save failed: {e}"),
        "nt.mkdir" => ("创建目录失败", "Failed to create directory"),
        "nt.redl" => ("本地缓存无效，正在重新下载视频…", "Local cache invalid, re-downloading video…"),
        "nt.novurl" => ("视频地址为空，无法保存", "Video URL is empty, cannot save"),

        // 兜底：未知 key 返回空串（所有 key 都是编译期字面量，实际不会命中）
        _ => ("", ""),
    };
    match lang {
        Lang::Zh => zh,
        Lang::En => en,
    }
}

/// 带占位符的文案：把 "{name}" 替换成 args 里对应的值。
pub fn tf(lang: Lang, key: &str, args: &[(&str, &str)]) -> String {
    let mut s = t(lang, key).to_string();
    for (k, v) in args {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}
