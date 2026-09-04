// 模型速率限制（内置只读文档）。
// 内容整理自 docs/Agnes AI/模型速率限制说明.md，更新数据时两处同步修改。

use crate::i18n::Lang;

const MD_ZH: &str = r#"数据更新：2026-09-04 · [官方文档](https://agnes-ai.com/zh-Hans/docs/tokenplan)

## 访问类型（API 密钥类型）

Agnes AI 有三种访问类型，同一用户可同时持有多种，不同类型的密钥使用不同的限制池：

| 密钥类型 | 适用用户 | 限制方式 |
| --- | --- | --- |
| 免费 / 默认密钥 | 所有用户 | 免费 / 默认 RPM 池 |
| 企业认证密钥 | 完成企业认证的用户 | 企业认证 RPM 池 |
| Token Plan 密钥 | 订阅 Token Plan 的用户 | Token Plan RPM 池 + 订阅配额池 |

限制按**密钥类型**共享，而不是按单个密钥计算——创建多个相同类型的密钥，不会增加总 RPM 或总配额。

## RPM 限制（每分钟请求数）

RPM（Requests Per Minute）控制请求频率。分辨率越高消耗算力越大，RPM 通常越低。

### 图片模型（按输出档位区分）

| 用户类型 | 1K | 2K | 3K | 4K |
| --- | --- | --- | --- | --- |
| 免费 / 默认 | 30（实际 20） | 20（实际 10） | 2（实际 1） | 1 |
| 企业认证 | 60（实际 40） | 40（实际 20） | 2（实际 1） | 2（实际 1） |
| Token Plan | 120（实际 100） | 120（实际 80） | 2（实际 1） | 2（实际 1） |

- `1K` `2K` `3K` `4K` 是与 `ratio` 配合使用的输出档位，不等同于 1080p / 1440p / UHD 等显示器分辨率。
- Token Plan 用户在 1K / 2K 档位拥有明显更高的生成能力；3K / 4K 档位各类型限制都较为严格。

### 视频模型

| 用户类型 | 允许发起 RPM | 实际 RPM |
| --- | --- | --- |
| 免费 / 默认 | 2 | 1 |
| 企业认证 | 2 | 2 |
| Token Plan | 6 | 5 |

视频生成成本较高，RPM 低于文本与低分辨率图片；视频模型同时受 RPM 与每日秒数配额限制。

## 订阅配额（Token Plan）

除 RPM 外，Token Plan 用户还受订阅配额限制，两者**同时生效**：

- 图片模型配额按生成图片**张数**计数。
- 视频模型配额按生成视频**时长（秒）**计数。
- Token Plan 用户每日可生成 4,000 张图片、500 秒视频（Starter / Plus / Pro 相同）。
- 例如：Pro 用户文本模型支持 1000 RPM，但仍受每 5 小时 30,000 次、每周 300,000 次的配额限制。

## Token Plan 套餐价格

| 套餐 | 国际站（.com · 现 5 折） | 国内站（.cn · 无折扣） | 每 5 小时模型请求 |
| --- | --- | --- | --- |
| Starter | $2 / 月（原价 $4） | ¥25 / 月 | 1,500 次 |
| Plus | $5 / 月（原价 $10） | ¥60 / 月 | 7,500 次 |
| Pro | $25 / 月（原价 $50） | ¥350 / 月 | 30,000 次 |

所有套餐均：由 Agnes-2.5-Flash 提供支持（约 100–150 TPS）、更高的 RPM 限制、兼容主流 Coding 工具、可使用图像理解及图片/视频模型。国内站无首月折扣，正常价格折算后略低于国际站。"#;

const MD_EN: &str = r#"Data as of 2026-09-04 · [Official docs](https://agnes-ai.com/zh-Hans/docs/tokenplan)

## Access Types (API Key Types)

Agnes AI has three access types. One user may hold several at once; each key type draws from its own rate-limit pool.

| Key type | Who it's for | Limits |
| --- | --- | --- |
| Free / default key | Everyone | Free / default RPM pool |
| Enterprise key | Enterprise-verified users | Enterprise RPM pool |
| Token Plan key | Token Plan subscribers | Token Plan RPM pool + subscription quotas |

Limits are shared **per key type**, not per individual key — creating multiple keys of the same type does not increase your total RPM or quota.

## RPM Limits (requests per minute)

RPM controls request frequency. Higher resolutions cost more compute, so RPM is usually lower.

### Image models (per output tier)

| User type | 1K | 2K | 3K | 4K |
| --- | --- | --- | --- | --- |
| Free / default | 30 (effective 20) | 20 (effective 10) | 2 (effective 1) | 1 |
| Enterprise | 60 (effective 40) | 40 (effective 20) | 2 (effective 1) | 2 (effective 1) |
| Token Plan | 120 (effective 100) | 120 (effective 80) | 2 (effective 1) | 2 (effective 1) |

- 1K–4K are output tiers used together with `ratio` — not display resolutions such as 1080p or 4K UHD.
- Token Plan users get much higher capacity at 1K / 2K; the 3K / 4K tiers are strict for every type.

### Video models

| User type | Allowed RPM | Effective RPM |
| --- | --- | --- |
| Free / default | 2 | 1 |
| Enterprise | 2 | 2 |
| Token Plan | 6 | 5 |

Video generation is expensive, so RPM is lower than for text and low-resolution images; video is also capped by a daily seconds quota.

## Subscription Quotas (Token Plan)

Besides RPM, Token Plan users are limited by subscription quotas — both apply **at the same time**:

- Image quotas count generated **images**; video quotas count generated **seconds**.
- Token Plan users can generate 4,000 images and 500 seconds of video per day (same for Starter / Plus / Pro).
- Example: Pro supports 1000 RPM on text models but is still capped at 30,000 requests per 5 hours and 300,000 per week.

## Token Plan Pricing

| Plan | International (.com · 50% off) | China (.cn · no discount) | Requests / 5 h |
| --- | --- | --- | --- |
| Starter | $2 / mo (was $4) | ¥25 / mo | 1,500 |
| Plus | $5 / mo (was $10) | ¥60 / mo | 7,500 |
| Pro | $25 / mo (was $50) | ¥350 / mo | 30,000 |

All plans are powered by Agnes-2.5-Flash (~100–150 TPS) with higher RPM limits, compatible with mainstream coding tools, and include image understanding plus image/video models. The China site's regular price converts to slightly less than the international site."#;

pub fn markdown(lang: Lang) -> &'static str {
    match lang {
        Lang::Zh => MD_ZH,
        Lang::En => MD_EN,
    }
}
