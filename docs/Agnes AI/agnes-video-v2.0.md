# agnes-video-v2.0

资料更新时间：2026-09-03

资料来源：https://wiki.agnes-ai.com/zh-Hans/docs/agnes-video-v20

## 概述

| 项目 | 内容 |
| --- | --- |
| 类型 | 异步视频生成模型 |
| 模型 ID | `agnes-video-v2.0` |
| 创建任务 | `POST /v1/videos` |
| 推荐查询 | `GET /agnesapi?video_id=<VIDEO_ID>` |
| 兼容旧版查询 | `GET /v1/videos/<TASK_ID>` |
| 主要能力 | 文生视频、图生视频、关键帧动画 |
| 当前价格 | `$0 / 秒` |

## 请求参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `model` | string | 是 | 固定为 `agnes-video-v2.0`。 |
| `prompt` | string | 是 | 视频内容描述。 |
| `image` | string | 否 | 图生视频输入图片 URL。 |
| `mode` | string | 否 | 生成模式，例如 `ti2vid` 或 `keyframes`。 |
| `width` | integer | 否 | 视频宽度；可能被标准化。 |
| `height` | integer | 否 | 视频高度；可能被标准化。 |
| `num_frames` | integer | 否 | 必须小于等于 `441`，并遵循 `8n + 1`。 |
| `frame_rate` | number | 否 | 支持 `1-60`。 |
| `seed` | integer | 否 | 用于提高可复现性。 |
| `negative_prompt` | string | 否 | 描述需要避免的内容。 |
| `extra_body.image` | string[] | 关键帧模式必填 | 关键帧图片 URL 数组。 |
| `extra_body.mode` | string | 关键帧模式必填 | 关键帧模式使用 `keyframes`。 |

## 尺寸与时长

| 项目 | 规则 |
| --- | --- |
| 标准分辨率档位 | `480p`、`720p`、`1080p` |
| 宽高比 | 常见支持 `16:9`、`9:16`、`1:1`、`4:3`、`3:4` |
| 时长计算 | `seconds = num_frames / frame_rate` |
| 帧数规则 | `num_frames <= 441`，且符合 `8n + 1` |

| 目标时长 | 推荐参数 |
| --- | --- |
| 约 3 秒 | `num_frames: 81`, `frame_rate: 24` |
| 约 5 秒 | `num_frames: 121`, `frame_rate: 24` |
| 约 10 秒 | `num_frames: 241`, `frame_rate: 24` |
| 约 18 秒 | `num_frames: 441`, `frame_rate: 24` |

## 响应与状态

| 字段 | 说明 |
| --- | --- |
| `id` / `task_id` | 异步任务 ID。 |
| `video_id` | 推荐用于查询结果的视频 ID，应视为不透明 ID。 |
| `status` | `queued`、`in_progress`、`completed`、`failed`。 |
| `progress` | 任务进度百分比。 |
| `metadata.url` | 完成后的视频 URL。 |
| `metadata.size_mapping` | 尺寸标准化信息。 |
| `error` | 失败时的错误信息。 |

## 错误码与检查

| 状态码 | 含义 |
| --- | --- |
| `400` | 请求参数无效。 |
| `401` | API Key 无效或缺失。 |
| `404` | 任务或视频未找到。 |
| `500` | 服务端错误。 |
| `503` | 服务繁忙。 |

| 检查项 | 要求 |
| --- | --- |
| 异步流程 | 先创建任务，再用 `video_id` 查询。 |
| 图生视频 | 使用顶层 `image`。 |
| 关键帧动画 | 使用 `extra_body.image` 与 `extra_body.mode: "keyframes"`。 |
| 视频结果 | 仅当 `status` 为 `completed` 时使用 `metadata.url`。 |

