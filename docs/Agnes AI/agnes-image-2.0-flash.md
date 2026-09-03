# agnes-image-2.0-flash

资料更新时间：2026-09-03

资料来源：https://wiki.agnes-ai.com/zh-Hans/docs/agnes-image-20-flash

## 概述

| 项目 | 内容 |
| --- | --- |
| 类型 | 图片生成与图片编辑模型 |
| 模型 ID | `agnes-image-2.0-flash` |
| Endpoint | `POST /v1/images/generations` |
| 主要能力 | 文生图、图生图、多图合成 |
| 输出格式 | URL 或 Base64 |
| 当前价格 | 所有支持的输出分辨率档位和输入参考图片当前免费 |

## 能力与场景

| 能力 | 说明 | 适合场景 |
| --- | --- | --- |
| 文生图 | 根据文本提示生成图片。 | 创意设计、营销视觉、产品图、社交媒体素材。 |
| 图生图 | 基于输入图片进行转换、重绘或风格化。 | 风格迁移、背景替换、重打光、商品图优化。 |
| 多图合成 | 使用多张参考图共同生成新图像。 | 角色与产品合成、海报草图、多参考视觉探索。 |

## 请求参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `model` | string | 是 | 固定为 `agnes-image-2.0-flash`。 |
| `prompt` | string | 是 | 图片生成或编辑指令。 |
| `size` | string | 是 | 可使用 `1K`、`2K`、`3K`、`4K`，也兼容部分历史精确尺寸写法。 |
| `ratio` | string | 否 | 与档位式 `size` 配合使用，如 `1:1`、`16:9`、`9:16`。 |
| `return_base64` | boolean | 否 | 文生图需要 Base64 返回时使用。 |
| `extra_body.image` | string[] | 图生图/多图合成必填 | 输入图片数组，支持公共 URL 或 Data URI Base64。 |
| `extra_body.response_format` | string | 否 | 可选 `url` 或 `b64_json`。 |

## 尺寸与价格

| 输出档位 | 当前价格 | 刊例价 |
| --- | --- | --- |
| 1K | `$0` | `$0.010 / 张` |
| 2K | `$0` | `$0.018 / 张` |
| 3K | `$0` | `$0.021 / 张` |
| 4K | `$0` | `$0.024 / 张` |
| 第 4 张起输入参考图片 | `$0 / 张` | `$0.003 / 张` |

## 集成检查

| 检查项 | 要求 |
| --- | --- |
| 模型名称 | 请求体 `model` 必须使用 `agnes-image-2.0-flash`。 |
| URL 输出 | 不要把 `response_format` 放在顶层；应放在 `extra_body.response_format`。 |
| 图生图 | 输入图片放在 `extra_body.image`，不需要传 `tags: ["img2img"]`。 |
| 输入图片 URL | 应可公开访问；私有地址建议改用 Data URI Base64。 |
| 客户端超时 | 图片生成可能耗时数秒到几十秒，建议设置较长超时。 |

