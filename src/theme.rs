// 主题系统：浅色 / 深色 / 跟随系统。
//
// 视觉语言：苹果式「液态玻璃」——
//   - 环境底色为柔和的中性光斑（暖沙 / 雾灰 / 灰绿），不使用蓝紫等彩色
//   - 卡片、顶栏、历史栏为半透明玻璃：backdrop-filter 模糊 + 饱和度提升，
//     顶部一道高光内阴影（specular highlight），圆角 16-18px
//   - 主按钮为石墨黑（浅色）/ 纯白（深色），中性克制
//
// 根节点 .app 挂 data-theme="light|dark|system"：
//   - light  ：.app 基础规则里的浅色变量
//   - dark   ：.app[data-theme="dark"] 的深色变量
//   - system ：默认浅色，@media (prefers-color-scheme: dark) 下套用同一组深色变量
// 系统主题热切换由 WebView2 的媒体查询即时生效，无需轮询或重启；
// color-scheme 声明让原生控件（select 下拉、滚动条、数字输入框）一并跟随主题。

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

impl ThemeMode {
    /// 配置文件取值："light" | "dark" | "system"（其余按跟随系统处理）
    pub fn from_cfg(s: &str) -> Self {
        match s {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::System,
        }
    }
    pub fn to_cfg(self) -> &'static str {
        match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
            ThemeMode::System => "system",
        }
    }
    /// 根节点 data-theme 属性值
    pub fn attr(self) -> &'static str {
        self.to_cfg()
    }
    /// 顶栏快捷按钮图标（浅色 -> 深色 -> 跟随系统 循环切换）
    pub fn icon(self) -> &'static str {
        match self {
            ThemeMode::Light => "☀️",
            ThemeMode::Dark => "🌙",
            ThemeMode::System => "🖥️",
        }
    }
    pub fn cycle(self) -> Self {
        match self {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::System,
            ThemeMode::System => ThemeMode::Light,
        }
    }
}

// ── CSS 变量（浅色）──────────────────────────────────────────────────────────
const VARS_LIGHT: &str = "color-scheme:light;
--bg:#eef0f4;
--ambient:radial-gradient(1100px 750px at 6% 10%,rgba(226,210,188,.55),transparent 62%),radial-gradient(1000px 700px at 94% 6%,rgba(198,209,224,.55),transparent 60%),radial-gradient(1200px 900px at 88% 92%,rgba(200,214,202,.45),transparent 65%),radial-gradient(900px 700px at 10% 90%,rgba(216,216,226,.4),transparent 60%);
--text:#1d1d1f;
--text2:rgba(60,60,67,.62);
--text3:rgba(60,60,67,.35);
--glass:rgba(255,255,255,.56);
--glass-strong:rgba(255,255,255,.74);
--glass-border:rgba(120,120,128,.20);
--glass-hi:rgba(255,255,255,.65);
--fill:rgba(120,120,128,.10);
--fill-2:rgba(120,120,128,.16);
--fill-hover:rgba(120,120,128,.22);
--input-bg:rgba(255,255,255,.62);
--input-border:rgba(120,120,128,.24);
--input-focus:rgba(60,60,67,.5);
--ring:rgba(120,120,128,.22);
--seg-on:#ffffff;
--border:rgba(60,60,67,.12);
--cta:#1d1d1f;
--cta-fg:#ffffff;
--cta-shadow:rgba(0,0,0,.2);
--cta-hi:rgba(255,255,255,.16);
--ok:#1f8a4d;
--err:#d93636;
--err-weak:rgba(217,54,54,.10);
--warn:#a86e00;
--warn-weak:rgba(199,146,0,.14);
--shadow:0 1px 2px rgba(20,20,30,.05),0 8px 28px rgba(20,20,30,.07);
--shadowlg:0 2px 6px rgba(20,20,30,.06),0 18px 52px rgba(20,20,30,.14);
--backdrop:rgba(25,25,32,.32);
--sel-menu:#ffffff;
--bar:linear-gradient(90deg,#4a4a50,#1d1d1f)";

// ── CSS 变量（深色；dark 与 system@暗色媒体共用）─────────────────────────────
const VARS_DARK: &str = "color-scheme:dark;
--bg:#0a0b0e;
--ambient:radial-gradient(1100px 750px at 6% 10%,rgba(74,64,52,.4),transparent 62%),radial-gradient(1000px 700px at 94% 6%,rgba(44,54,70,.45),transparent 60%),radial-gradient(1200px 900px at 88% 92%,rgba(40,56,48,.35),transparent 65%),radial-gradient(900px 700px at 10% 90%,rgba(56,50,62,.35),transparent 60%);
--text:#f5f5f7;
--text2:rgba(235,235,245,.6);
--text3:rgba(235,235,245,.3);
--glass:rgba(30,30,36,.52);
--glass-strong:rgba(34,34,40,.78);
--glass-border:rgba(255,255,255,.14);
--glass-hi:rgba(255,255,255,.09);
--fill:rgba(120,120,128,.22);
--fill-2:rgba(120,120,128,.32);
--fill-hover:rgba(120,120,128,.4);
--input-bg:rgba(255,255,255,.07);
--input-border:rgba(255,255,255,.16);
--input-focus:rgba(235,235,245,.55);
--ring:rgba(235,235,245,.2);
--seg-on:rgba(255,255,255,.17);
--border:rgba(255,255,255,.1);
--cta:#f6f6f8;
--cta-fg:#101014;
--cta-shadow:rgba(0,0,0,.5);
--cta-hi:rgba(255,255,255,.6);
--ok:#3ac76f;
--err:#ff6b6b;
--err-weak:rgba(255,90,90,.13);
--warn:#e5b455;
--warn-weak:rgba(229,180,85,.15);
--shadow:0 1px 2px rgba(0,0,0,.35),0 8px 28px rgba(0,0,0,.35);
--shadowlg:0 2px 6px rgba(0,0,0,.45),0 18px 52px rgba(0,0,0,.55);
--backdrop:rgba(0,0,0,.5);
--sel-menu:#26262c;
--bar:linear-gradient(90deg,#c9c9cf,#ffffff)";

// ── 组件样式（引用上面的变量）───────────────────────────────────────────────
const COMPONENT_CSS: &str = r#"
*{box-sizing:border-box}
html,body{margin:0;padding:0;height:100%;overflow:hidden}
.app{display:flex;flex-direction:column;height:100vh;font-family:-apple-system,'SF Pro Text','Segoe UI Variable Text','Segoe UI',system-ui,'PingFang SC','Microsoft YaHei',sans-serif;font-size:13.5px;color:var(--text);background-color:var(--bg);background-image:var(--ambient);background-repeat:no-repeat;-webkit-font-smoothing:antialiased}
.app,.app *{transition:background-color .18s ease,border-color .18s ease,color .18s ease,box-shadow .18s ease}
::-webkit-scrollbar{width:8px;height:8px}
::-webkit-scrollbar-track{background:transparent}
::-webkit-scrollbar-thumb{background:rgba(120,120,128,.35);border-radius:4px}
::-webkit-scrollbar-thumb:hover{background:rgba(120,120,128,.55)}
button{font-family:inherit}

/* 顶栏（玻璃条） */
.topbar{display:flex;align-items:center;height:54px;padding:0 18px;background:var(--glass);backdrop-filter:blur(24px) saturate(180%);-webkit-backdrop-filter:blur(24px) saturate(180%);border-bottom:1px solid var(--glass-border);flex-shrink:0;gap:10px;position:relative;z-index:20;box-shadow:inset 0 1px 0 var(--glass-hi)}
.brand{font-size:16px;font-weight:800;letter-spacing:.1px;color:var(--text);flex-shrink:0}
.modelbadge{font-size:11px;font-weight:600;color:var(--text2);background:var(--fill);padding:2px 9px;border-radius:999px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:220px}
.tabs{display:flex;gap:2px;background:var(--fill);border-radius:10px;padding:3px;flex-shrink:0}
.tab{padding:5px 15px;border:none;border-radius:7px;font-size:13px;font-weight:600;cursor:pointer;color:var(--text2);background:transparent;transition:color .15s ease,background-color .15s ease,box-shadow .15s ease}
.tab:hover{color:var(--text);background:var(--fill-2)}
.tab.on{background:var(--seg-on);color:var(--text);box-shadow:0 1px 4px rgba(0,0,0,.14),inset 0 1px 0 var(--glass-hi)}
.link{font-size:12.5px;color:var(--text2);cursor:pointer;border:none;background:transparent;padding:4px 7px;border-radius:6px;flex-shrink:0;transition:color .15s ease,background-color .15s ease}
.link:hover{color:var(--text);background:var(--fill)}
.iconbtn{width:30px;height:30px;display:inline-flex;align-items:center;justify-content:center;border:none;background:var(--fill);border-radius:8px;cursor:pointer;font-size:14px;color:var(--text2);padding:0;flex-shrink:0;transition:background-color .15s ease,color .15s ease,transform .15s ease}
.iconbtn:hover{background:var(--fill-hover);color:var(--text);transform:translateY(-1px)}
.iconbtn:active{transform:scale(.9)}
.iconbtn svg{display:block}
.chip{font-size:12px;font-weight:600;color:var(--text);background:var(--fill);padding:3px 10px;border-radius:999px;white-space:nowrap;flex-shrink:0;transition:background-color .15s ease}
.chip.pulse{animation:pulse 1.8s ease infinite}
.chip.warn{background:var(--warn-weak);color:var(--warn)}
.chip.click{cursor:pointer}
.chip.click:hover{background:var(--fill-hover)}
@keyframes pulse{0%,100%{opacity:1}50%{opacity:.55}}
.keychip{display:flex;align-items:center;gap:6px;font-size:12.5px;color:var(--text2);white-space:nowrap;flex-shrink:0}
.dot{width:7px;height:7px;border-radius:50%;flex-shrink:0}

/* 主体布局：左侧栏（滚动区 + 固定生成条）+ 右侧工作区（主区 + 历史栏）
   历史栏只铺在工作区下方，不会盖住左栏底部的生成按钮 */
.body{display:flex;flex:1;min-height:0;overflow:hidden}
.side{display:flex;flex-direction:column;width:380px;min-width:320px;max-width:480px;flex-shrink:0;min-height:0}
.side-scroll{flex:1;min-height:0;overflow-y:auto;padding:12px 14px 4px}
/* 底部生成条：浮动圆角玻璃面板，与上方卡片左右对齐 */
.side-action{flex-shrink:0;margin:0 14px 14px;padding:10px 12px 12px;background:var(--glass);backdrop-filter:blur(24px) saturate(180%);-webkit-backdrop-filter:blur(24px) saturate(180%);border:1px solid var(--glass-border);border-radius:14px;box-shadow:var(--shadow),inset 0 1px 0 var(--glass-hi)}
.work{display:flex;flex-direction:column;flex:1;min-width:0;min-height:0}

/* 卡片（玻璃，进场时轻微上浮淡入） */
.card{background:var(--glass);backdrop-filter:blur(22px) saturate(170%);-webkit-backdrop-filter:blur(22px) saturate(170%);border:1px solid var(--glass-border);border-radius:16px;padding:14px 15px;margin:0 0 10px;box-shadow:var(--shadow),inset 0 1px 0 var(--glass-hi);animation:cardIn .3s ease}
@keyframes cardIn{from{opacity:0;transform:translateY(6px)}}
.cardh{display:flex;align-items:center;gap:7px;margin-bottom:10px}
.cardh::before{content:"";width:3px;height:13px;border-radius:2px;background:var(--text2);opacity:.55;flex-shrink:0}
.cardt{font-size:12.5px;font-weight:700;color:var(--text);letter-spacing:.15px}
.subh{font-size:12px;font-weight:600;color:var(--text2);margin:8px 0 6px}
.hint{font-size:12px;color:var(--text2);margin-top:7px;line-height:1.5}
.lbl{font-size:12px;color:var(--text2);flex-shrink:0}

/* 表单 */
.sel,.ix,.ta{width:100%;padding:8px 10px;border:1px solid var(--input-border);border-radius:9px;font-size:13px;color:var(--text);background:var(--input-bg);outline:none;font-family:inherit}
.ta{min-height:96px;resize:vertical;line-height:1.55;font-size:13.5px}
.ix:focus,.sel:focus,.ta:focus{border-color:var(--input-focus);box-shadow:0 0 0 3.5px var(--ring)}
.sel{appearance:none;-webkit-appearance:none;padding-right:28px;cursor:pointer;background-image:url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M1 1l4 4 4-4' fill='none' stroke='%23787880' stroke-width='1.6' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");background-repeat:no-repeat;background-position:right 10px center}
.sel option{background:var(--sel-menu);color:var(--text)}

/* 按钮（微交互：悬停上浮 + 按压回弹） */
.g{padding:6px 12px;border:none;border-radius:8px;background:var(--fill);color:var(--text);font-size:12.5px;font-weight:600;cursor:pointer;white-space:nowrap;transition:background-color .16s ease,transform .16s ease,box-shadow .16s ease}
.g:hover:not(:disabled){background:var(--fill-hover);transform:translateY(-1px);box-shadow:0 3px 10px rgba(20,20,30,.1)}
.g:active:not(:disabled){transform:translateY(0) scale(.96);box-shadow:none}
.g:disabled{opacity:.45;cursor:default}
.b2{width:100%;height:44px;border:none;border-radius:12px;background:var(--cta);color:var(--cta-fg);font-size:15px;font-weight:700;cursor:pointer;letter-spacing:.3px;box-shadow:0 6px 18px var(--cta-shadow),inset 0 1px 0 var(--cta-hi);transition:filter .2s,transform .18s ease,box-shadow .25s ease}
.b2:hover:not(:disabled){filter:brightness(1.12);transform:translateY(-1px);box-shadow:0 10px 28px var(--cta-shadow),inset 0 1px 0 var(--cta-hi)}
.b2:active:not(:disabled){transform:scale(.97);box-shadow:0 3px 10px var(--cta-shadow),inset 0 1px 0 var(--cta-hi)}
.b2:disabled{opacity:.55;cursor:default}

/* 分段选择（macOS 风格：轨道 + 白色浮起滑块） */
.sg{display:flex;gap:2px;background:var(--fill);border-radius:9px;padding:3px}
.s1{flex:1;padding:6px 8px;border:none;border-radius:6px;font-size:12.5px;font-weight:600;text-align:center;cursor:pointer;color:var(--text2);background:transparent;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;transition:color .15s ease,background-color .15s ease,box-shadow .15s ease}
.s1:hover{color:var(--text);background:var(--fill-2)}
.s1.on{background:var(--seg-on);color:var(--text);box-shadow:0 1px 3px rgba(0,0,0,.14),inset 0 1px 0 var(--glass-hi)}

/* 提示 / 通知 */
.nt{font-size:12px;line-height:1.5;word-break:break-all}
.nt-ok{color:var(--ok)}
.nt-err{color:var(--err)}
.nt-info{color:var(--text2)}
.warnerr{font-size:12px;color:var(--warn);margin-top:6px;line-height:1.5}
.oklbl{font-size:12.5px;color:var(--ok);margin-top:6px;word-break:break-all}
.row{display:flex;align-items:center;gap:8px}
.between{justify-content:space-between}
.urlitem{display:flex;align-items:center;gap:6px;margin-top:5px}
.urlitem .ulbl{font-size:12px;color:var(--text2);flex-shrink:0;width:42px}
.urlitem .utxt{font-size:12px;color:var(--text);flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}

/* 主区：图片面板 / 信息面板 / 历史栏三块同宽（均为 main 内容列宽度），
   统一 14px 圆角 + 玻璃边框 */
.main{flex:1;display:flex;flex-direction:column;padding:18px 18px 14px;overflow:hidden;min-height:0}
.stage{flex:1;display:flex;align-items:center;justify-content:center;overflow:hidden;min-height:0}
.imgwrap{background:var(--glass-strong);backdrop-filter:blur(22px) saturate(160%);-webkit-backdrop-filter:blur(22px) saturate(160%);border:1px solid var(--glass-border);padding:10px;border-radius:14px;box-shadow:var(--shadowlg),inset 0 1px 0 var(--glass-hi);display:flex;align-items:center;justify-content:center;width:100%;height:100%;min-height:0;cursor:zoom-in;overflow:hidden}
.imgwrap:hover{box-shadow:var(--shadowlg),0 18px 48px var(--cta-shadow),inset 0 1px 0 var(--glass-hi)}
.vidwrap{background:rgba(0,0,0,.72);padding:10px;border-radius:14px;border:1px solid var(--glass-border);box-shadow:var(--shadowlg);display:flex;align-items:center;justify-content:center;width:100%;height:100%;min-height:0;overflow:hidden}
.meta{margin-top:12px;background:var(--glass);backdrop-filter:blur(22px) saturate(170%);-webkit-backdrop-filter:blur(22px) saturate(170%);border:1px solid var(--glass-border);border-radius:14px;padding:13px 15px;box-shadow:var(--shadow),inset 0 1px 0 var(--glass-hi);flex-shrink:0}
.metatop{display:flex;align-items:center;gap:10px;flex-wrap:wrap}
.mmodel{font-size:12.5px;font-weight:700;color:var(--text)}
.minfo{font-size:12.5px;color:var(--text2)}
.mprompt{font-size:12.5px;color:var(--text);line-height:1.5;margin-top:2px;word-break:break-word}
.divider{border:none;border-top:1px solid var(--border);margin:8px 0 6px}

/* 错误卡片（主生成区展示） */
.errcard{background:var(--glass-strong);backdrop-filter:blur(26px) saturate(170%);-webkit-backdrop-filter:blur(26px) saturate(170%);border:1px solid var(--glass-border);border-radius:18px;padding:26px 30px;max-width:min(88vw,540px);display:flex;flex-direction:column;align-items:center;text-align:center;box-shadow:var(--shadowlg),inset 0 1px 0 var(--glass-hi);animation:popIn .25s cubic-bezier(.16,1,.3,1)}
.erricon{width:44px;height:44px;border-radius:50%;background:var(--err-weak);color:var(--err);display:flex;align-items:center;justify-content:center;font-size:19px;margin-bottom:12px;flex-shrink:0}
.errtitle{font-size:16px;font-weight:700;color:var(--text)}
.errmsg{font-size:12.5px;color:var(--text2);margin-top:8px;line-height:1.65;word-break:break-word;white-space:pre-wrap;max-width:100%}

/* 加载 / 空态 */
.center{flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;min-height:0}
.spinner{width:46px;height:46px;border-radius:50%;background:var(--fill);display:flex;align-items:center;justify-content:center;margin-bottom:14px}
.spin{width:26px;height:26px;border:2.5px solid var(--text2);border-top-color:transparent;border-radius:50%;animation:rot .8s linear infinite}
@keyframes rot{to{transform:rotate(360deg)}}
.loadtitle{font-size:16px;font-weight:700;color:var(--text)}
.loadsub{font-size:12.5px;color:var(--text2);margin-top:5px}
.bar{width:260px;height:6px;background:var(--fill);border-radius:3px;margin-top:12px;overflow:hidden}
.barfill{height:100%;background:var(--bar);border-radius:3px;transition:width .3s}
.barind{width:40%;height:100%;background:var(--bar);border-radius:3px;animation:shimmer 1.2s ease-in-out infinite}
@keyframes shimmer{0%{transform:translateX(-100%)}100%{transform:translateX(250%)}}
.emptylogo{width:58px;height:58px;border-radius:16px;background:var(--glass);backdrop-filter:blur(18px);-webkit-backdrop-filter:blur(18px);border:1px solid var(--glass-border);box-shadow:var(--shadow),inset 0 1px 0 var(--glass-hi);display:flex;align-items:center;justify-content:center;font-size:26px;margin-bottom:16px}
.emptyt{font-size:19px;font-weight:800;color:var(--text)}
.emptys{font-size:13px;color:var(--text2);margin-top:7px;text-align:center;max-width:400px;line-height:1.6}

/* 历史栏：浮动圆角玻璃面板，与上方图片/信息面板同宽对齐 */
.hist{flex-shrink:0;margin:0 18px 14px;background:var(--glass);backdrop-filter:blur(24px) saturate(180%);-webkit-backdrop-filter:blur(24px) saturate(180%);border:1px solid var(--glass-border);border-radius:14px;padding:10px 14px 12px;box-shadow:var(--shadow),inset 0 1px 0 var(--glass-hi)}
.histh{display:flex;align-items:center;margin-bottom:8px}
.histt{font-size:12px;font-weight:600;color:var(--text2)}
.thumbs{display:flex;gap:8px;overflow-x:auto;overflow-y:hidden;padding-bottom:4px}
.thumb{flex-shrink:0;cursor:pointer;border-radius:9px;padding:2px;border:2px solid transparent;transition:border-color .15s ease,transform .15s ease}
.thumb:hover{border-color:var(--fill-hover);transform:translateY(-2px)}
.thumb.on{border-color:var(--text)}
.thumbimg{width:94px;height:70px;object-fit:cover;border-radius:6px;display:block;background:var(--fill)}
.thumbvid{width:94px;height:70px;border-radius:6px;background:var(--fill-2);color:var(--text2);display:flex;align-items:center;justify-content:center;font-size:12px;font-weight:600}

/* 弹窗（玻璃）：遮罩淡入 + 面板弹性放大进场 */
.mask{position:fixed;inset:0;z-index:1100;background:var(--backdrop);backdrop-filter:blur(8px);-webkit-backdrop-filter:blur(8px);display:flex;align-items:center;justify-content:center;animation:fadeIn .16s ease}
.dialog{background:var(--glass-strong);backdrop-filter:blur(30px) saturate(180%);-webkit-backdrop-filter:blur(30px) saturate(180%);border:1px solid var(--glass-border);border-radius:18px;padding:22px;width:min(92vw,520px);max-height:88vh;display:flex;flex-direction:column;box-shadow:var(--shadowlg),inset 0 1px 0 var(--glass-hi);animation:popIn .24s cubic-bezier(.16,1,.3,1)}
@keyframes fadeIn{from{opacity:0}}
@keyframes popIn{from{opacity:0;transform:scale(.94) translateY(10px)}}
.dtitle{font-size:17px;font-weight:800;color:var(--text)}
.dsub{font-size:12.5px;color:var(--text2);margin:4px 0 14px}
.notes{flex:1;overflow:auto;background:var(--fill);border:none;border-radius:10px;padding:12px;margin-bottom:14px;font-size:13px;color:var(--text);line-height:1.6}
.notes h1,.notes h2,.notes h3{font-size:15px;margin:10px 0 4px;color:var(--text)}
.notes p{margin:6px 0}
.notes a{color:var(--text);text-decoration:underline}
.notes ul,.notes ol{padding-left:18px;margin:6px 0}
.notes code{background:var(--fill-2);padding:1px 5px;border-radius:4px;font-size:12px}
.notes pre{background:var(--fill-2);padding:8px 10px;border-radius:8px;overflow:auto}
.notes img{max-width:100%}

/* 图片预览弹窗（深色玻璃观感，不随主题）
   缩放锚点在图片中心：img 绝对定位到舞台中心，transform-origin:center，
   缩放只会围绕中心进行，平移量叠加在居中 translate 上 */
.pvbox{background:rgba(19,19,24,.92);backdrop-filter:blur(24px) saturate(160%);-webkit-backdrop-filter:blur(24px) saturate(160%);border:1px solid rgba(255,255,255,.1);border-radius:18px;padding:14px;width:min(95vw,1400px);height:min(92vh,920px);display:flex;flex-direction:column;box-shadow:0 16px 56px rgba(0,0,0,.6),inset 0 1px 0 rgba(255,255,255,.08);animation:popIn .24s cubic-bezier(.16,1,.3,1)}
.pvhead{display:flex;align-items:center;gap:10px;margin-bottom:12px}
.pvtitle{font-size:13.5px;color:#cfd3de;font-weight:600;letter-spacing:.2px}
.pvseg{display:flex;align-items:center;background:rgba(255,255,255,.07);border:1px solid rgba(255,255,255,.14);border-radius:9px;overflow:hidden}
.pvseg .pvbtn{border:none;background:transparent;border-radius:0;padding:5px 11px;color:#d5d8e2}
.pvseg .pvbtn:hover{background:rgba(255,255,255,.12);color:#fff}
.pvz{font-size:12px;color:#cfd3de;min-width:48px;text-align:center;cursor:pointer;padding:5px 4px;user-select:none;font-variant-numeric:tabular-nums}
.pvz:hover{background:rgba(255,255,255,.12);color:#fff}
.pvbtn{padding:5px 12px;border:1px solid rgba(255,255,255,.22);border-radius:9px;background:rgba(255,255,255,.06);color:#d5d8e2;font-size:12.5px;cursor:pointer}
.pvbtn:hover{border-color:rgba(255,255,255,.45);color:#fff;background:rgba(255,255,255,.1)}
.pvclose{width:32px;height:28px;display:inline-flex;align-items:center;justify-content:center;padding:0}
.pvstage{flex:1;position:relative;overflow:hidden;background:repeating-conic-gradient(#222228 0% 25%,#1a1a20 0% 50%) 0 0/28px 28px;border-radius:12px;border:1px solid rgba(255,255,255,.06)}
.pvstage img{position:absolute;left:50%;top:50%;max-width:100%;max-height:100%;object-fit:contain;border-radius:3px;box-shadow:0 8px 32px rgba(0,0,0,.5);transform-origin:center;user-select:none;-webkit-user-drag:none}
.pvhint{margin-top:10px;font-size:11.5px;color:#8a8f9e;text-align:center}
"#;

/// 完整样式表：浅色变量挂在 .app 基础规则；深色变量同时挂在
/// .app[data-theme="dark"] 与（暗色媒体查询下的）.app[data-theme="system"]。
pub fn css() -> String {
    format!(
        ".app{{{VARS_LIGHT}}}\n.app[data-theme=\"dark\"]{{{VARS_DARK}}}\n@media (prefers-color-scheme:dark){{.app[data-theme=\"system\"]{{{VARS_DARK}}}}}\n{COMPONENT_CSS}"
    )
}
