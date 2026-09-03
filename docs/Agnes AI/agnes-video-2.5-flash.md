# agnes-video-2.5-flash

资料更新时间：2026-09-03

资料来源：https://wiki.agnes-ai.com/zh-Hans/docs/agnes-video-25-flash

## 概述

| 项目 | 内容 |
| --- | --- |
| 类型 | OpenAI Videos 兼容异步视频生成模型 |
| 模型 ID | `agnes-video-2.5-flash` |
| 创建任务 | `POST /v1/videos` |
| 推荐查询 | `GET /agnesapi?video_id=<VIDEO_ID>&model_name=agnes-video-2.5-flash` |
| 与 2.5 的关系 | 沿用 `agnes-video-2.5` 的公共参数、响应字段和查询方式 |
| 当前价格 | `$0 / 秒` |

## Flash 专属限制

| 校验项 | Flash 规则 | 校验失败 |
| --- | --- | --- |
| `size` | 仅支持 `"720P"` | HTTP 400，`size must be 720P` |
| `images` | `reference` 模式最多 5 张 | HTTP 400，图片数量超限 |
| `audios` | `reference` 模式最多 3 段 | HTTP 400，音频数量超限 |
| `videos` | 不支持有效的视频参考输入 | HTTP 400，视频参考不支持 |

上述校验在创建任务、排队、计费和推理前执行；校验失败不会创建视频任务。

## 请求参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `model` | string | 是 | 固定为 `agnes-video-2.5-flash`。 |
| `prompt` | string | 是 | 视频描述；参考模式可使用 `<Picture N>` 和 `<Audio N>`。 |
| `mode` | string | 是 | `text`、`keyframe` 或 `reference`。 |
| `seconds` | string | 否 | 支持 `"4"` 到 `"12"`，默认 `"5"`。 |
| `size` | string | 否 | 必须为 `"720P"`。 |
| `aspect_ratio` | string | 否 | 默认 `16:9`。 |
| `seed` | integer | 否 | 随机种子。 |
| `n` | integer | 否 | 当前仅支持 `1`。 |
| `first_frame` | string | `keyframe` 必填其一 | 首帧图片 URL。 |
| `last_frame` | string | `keyframe` 必填其一 | 尾帧图片 URL。 |
| `images` | string[] | `reference` 可选 | 最多 5 张参考图片。 |
| `audios` | string[] | `reference` 可选 | 最多 3 段参考音频。 |

## 模式规则

| `mode` | 用途 | 必需媒体 | 不允许字段 |
| --- | --- | --- | --- |
| `text` | 纯文本生成视频 | 无 | `first_frame`、`last_frame`、`images`、`audios`、`videos` |
| `keyframe` | 首帧、尾帧或首尾帧控制 | `first_frame` 与 `last_frame` 至少一个 | `images`、`audios`、`videos` |
| `reference` | 图片或音频参考生成 | `images` 或 `audios` 至少一类非空 | `first_frame`、`last_frame`、`videos` |

## 720P 输出尺寸

| `aspect_ratio` | 输出像素 |
| --- | --- |
| `21:9` | `1680x720` |
| `16:9` | `1280x704` |
| `4:3` | `960x720` |
| `1:1` | `720x720` |
| `3:4` | `720x960` |
| `9:16` | `720x1280` |

## 计费规则

| 计费项 | 刊例价 | 当前价格 |
| --- | --- | --- |
| 720P 视频时长 | `$0.025 / 秒` | `$0 / 秒` |

Flash 采用与 `agnes-video-2.5` 相同的计费公式，但当前限时免费。由于 Flash 只支持 720P，且最多接受 5 张参考图片和 3 段参考音频，正常请求下当前各项费用为 `$0`。

## 集成检查

| 检查项 | 要求 |
| --- | --- |
| 模型名称 | 使用 `agnes-video-2.5-flash`。 |
| 尺寸 | `size` 固定为字符串 `"720P"`。 |
| 参考图片 | `mode=reference` 时最多 5 张。 |
| 参考音频 | `mode=reference` 时最多 3 段。 |
| 参考视频 | 不要传入有效的 `videos` 内容。 |
| 查询方式 | 所有模式推荐带 `model_name=agnes-video-2.5-flash` 查询。 |

