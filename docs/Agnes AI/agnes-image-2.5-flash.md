# agnes-image-2.5-flash

资料更新时间：2026-09-03

资料来源：https://wiki.agnes-ai.com/zh-Hans/docs/agnes-image-25-flash

## 概述

| 项目 | 内容 |
| --- | --- |
| 类型 | 最新一代图片生成与编辑模型 |
| 模型 ID | `agnes-image-2.5-flash` |
| Endpoint | `POST /v1/images/generations` |
| 接入兼容性 | 请求参数、响应参数、尺寸、价格与 `agnes-image-2.1-flash` 保持一致 |
| 主要提升 | 图像生成、编辑、构图、细节呈现、提示词遵循 |
| 当前价格 | 所有支持的输出分辨率档位和输入参考图片当前免费 |

## 能力摘要

| 能力 | 说明 |
| --- | --- |
| 文生图 | 根据自然语言提示生成图片。 |
| 图生图 | 使用输入图片进行转换、重绘、风格化。 |
| 多图合成 | 结合多张参考图生成新的复合图像。 |
| 高信息密度图像 | 更适合复杂场景、丰富构图和多层视觉元素。 |
| 构图保留 | 编辑输入图片时尽量保留原始构图和主体布局。 |
| URL/Base64 输出 | 支持 URL 或 Base64 数据返回。 |

## 请求参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `model` | string | 是 | 固定为 `agnes-image-2.5-flash`。 |
| `prompt` | string | 是 | 图片生成或图片编辑指令。 |
| `size` | string | 是 | 推荐 `1K`、`2K`、`3K`、`4K`。 |
| `ratio` | string | 否 | 支持 `1:1`、`3:4`、`4:3`、`16:9`、`9:16`、`2:3`、`3:2`、`21:9`。 |
| `extra_body.image` | string[] | 图生图/多图合成必填 | 输入图片数组，支持 URL 或 Data URI Base64。 |
| `return_base64` | boolean | 否 | 文生图 Base64 输出。 |
| `extra_body.response_format` | string | 否 | `url` 或 `b64_json`。 |

## 价格与计费

| 计费项 | 当前价格 | 刊例价 |
| --- | --- | --- |
| 1K 输出图片 | `$0` | `$0.010 / 张` |
| 2K 输出图片 | `$0` | `$0.018 / 张` |
| 3K 输出图片 | `$0` | `$0.021 / 张` |
| 4K 输出图片 | `$0` | `$0.024 / 张` |
| 第 4 张起输入参考图片 | `$0 / 张` | `$0.003 / 张` |

## 集成检查

| 检查项 | 要求 |
| --- | --- |
| 模型名称 | 使用 `agnes-image-2.5-flash`。 |
| Endpoint | 使用 `https://apihub.agnes-ai.com/v1/images/generations`。 |
| 文生图 | 至少提供 `model`、`prompt`、`size`。 |
| 图生图/多图合成 | 在 `extra_body.image` 中提供输入图像。 |
| 输出格式 | 不要把 `response_format` 放在顶层。 |
| 尺寸 | 优先使用档位式 `size` 和 `ratio`，不要依赖非原生精确尺寸。 |

