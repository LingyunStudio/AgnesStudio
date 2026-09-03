# agnes-video-2.5

资料更新时间：2026-09-03

资料来源：https://wiki.agnes-ai.com/zh-Hans/docs/agnes-video-25

## 概述

| 项目 | 内容 |
| --- | --- |
| 类型 | OpenAI Videos 兼容异步视频生成模型 |
| 模型 ID | `agnes-video-2.5` |
| 创建任务 | `POST /v1/videos` |
| 推荐查询 | `GET /agnesapi?video_id=<VIDEO_ID>&model_name=agnes-video-2.5` |
| 主要能力 | 文生视频、首尾帧控制、多模态参考、视频参考、音画协同 |
| 价格 | 720P `$0.025 / 秒`；2K `$0.055 / 秒` |

## 请求参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `model` | string | 是 | 固定为 `agnes-video-2.5`。 |
| `prompt` | string | 是 | 视频描述；参考模式可使用 `<Picture N>`、`<Audio N>`、`<Video N>`。 |
| `mode` | string | 是 | `text`、`keyframe` 或 `reference`。 |
| `seconds` | string | 否 | 支持 `"4"` 到 `"12"`，默认 `"5"`。 |
| `size` | string | 否 | 支持 `"720P"` 和 `"2K"`。 |
| `aspect_ratio` | string | 否 | 默认 `16:9`。 |
| `seed` | integer | 否 | 随机种子。 |
| `n` | integer | 否 | 当前仅支持 `1`。 |
| `first_frame` | string | `keyframe` 必填其一 | 首帧图片 URL。 |
| `last_frame` | string | `keyframe` 必填其一 | 尾帧图片 URL。 |
| `images` | string[] | `reference` 可选 | 参考图片 URL。 |
| `audios` | string[] | `reference` 可选 | 参考音频 URL。 |
| `videos` | object[] | `reference` 可选 | 参考视频对象数组。 |

## 模式规则

| `mode` | 用途 | 必需媒体 | 不允许字段 |
| --- | --- | --- | --- |
| `text` | 纯文本生成视频 | 无 | `first_frame`、`last_frame`、`images`、`audios`、`videos` |
| `keyframe` | 首帧、尾帧或首尾帧控制 | `first_frame` 与 `last_frame` 至少一个 | `images`、`audios`、`videos` |
| `reference` | 使用图片、音频或视频作为参考 | `images`、`audios`、`videos` 至少一类非空 | `first_frame`、`last_frame` |

## 尺寸与画幅

| `size` | 说明 |
| --- | --- |
| `720P` | 标准清晰度，适合快速生成和常规内容。 |
| `2K` | 更高分辨率，适合高质量交付。 |

| `aspect_ratio` | 720P 输出像素 |
| --- | --- |
| `21:9` | `1680x720` |
| `16:9` | `1280x720` |
| `4:3` | `960x720` |
| `1:1` | `720x720` |
| `3:4` | `720x960` |
| `9:16` | `720x1280` |

## 计费规则

| 计费项 | 单价 |
| --- | --- |
| 720P 输出视频 | `$0.025 / 秒` |
| 2K 输出视频 | `$0.055 / 秒` |
| 第 6 张起输入图片 | `$0.005 / 张` |

计费公式：

```text
视频总金额 = 输出秒数 * 输出分辨率单价
           + 输入视频秒数 * 输出分辨率单价
           + max(0, 图片数 - 5) * $0.005
```

## 响应与错误处理

| 字段 | 说明 |
| --- | --- |
| `video_id` | 查询任务进度和结果。 |
| `status` | `queued`、`in_progress`、`completed` 或 `failed`。 |
| `metadata.url` | 仅在 `completed` 后可作为成片地址。 |
| `error` | 任务失败时返回错误信息。 |

| 状态码 | 常见原因 | 建议处理 |
| --- | --- | --- |
| `400` | 参数缺失、模式与媒体不匹配、时长或画幅非法 | 检查请求字段和模式规则。 |
| `401` / `403` | API Key 无效、过期或权限不足 | 检查请求头和密钥权限。 |
| `404` | 视频 ID 不存在 | 确认使用创建响应中的 `video_id`。 |
| `429` | 请求过快 | 降低轮询频率并做指数退避。 |
| `500` | 服务端错误 | 稍后重试或联系技术支持。 |

