# Agnes Video V2.0

面向文生视频、图生视频和关键帧动画的异步视频生成 API。

Agnes Video V2.0 是面向生产场景的视频生成模型，支持文生视频、图生视频和关键帧动画。视频生成采用异步任务 API：先创建任务，再通过 `video_id` 或 `task_id` 获取结果。

### 基本信息

- **模型名称**: `agnes-video-v2.0`
- **创建任务**: `POST /v1/videos`
- **获取结果**: `GET /agnesapi?video_id=<VIDEO_ID>`
- **当前价格**: 视频时长当前为 **$0 / 秒**

------

## 概述

开发者可以使用文本提示词或图片 URL 生成高质量视频。该模型适用于故事讲述、营销视频、产品演示、社交媒体内容、应用动态素材和 AI 创意工作流。

## 核心能力

- **文生视频**: 通过文本提示词直接生成视频。
- **图生视频**: 将静态图片转化为动态视频。
- **关键帧动画**: 在多个关键帧之间生成流畅过渡。
- **场景运动控制**: 通过提示词控制主体动作、镜头运动和场景动态。
- **视觉一致性**: 在帧间保持主体、风格和场景一致。
- **电影级输出**: 生成高质量电影级视频内容。
- **异步 API**: 创建任务后再轮询或查询生成结果。

## 适用场景

- **故事讲述**: 短片、角色场景和叙事片段。
- **营销视频**: 产品广告、宣传视频和推广内容。
- **社交媒体内容**: Reels、Shorts、TikTok 风格视频。
- **图片动画**: 为肖像、产品、角色或场景添加动画效果。
- **产品演示**: 通过文本或图片生成产品展示视频。
- **关键帧过渡**: 在不同视觉状态之间生成流畅过渡。

## 前提条件

在接入之前，请确认拥有有效的 Agnes AI API Key，网络可访问 Agnes AI API 网关，并已准备好用于视频生成的文本提示词。图生视频或关键帧动画还需要提供可公开访问的图片 URL。

------

## API Reference

### 1. 创建视频任务

**请求方式**: `POST https://apihub.agnes-ai.com/v1/videos`

**请求头**:

Bash

```
-H "Authorization: Bearer YOUR_API_KEY"
-H "Content-Type: application/json"
```

**创建任务参数**:

| **参数**              | **类型** | **必填** | **说明**                                      |
| --------------------- | -------- | -------- | --------------------------------------------- |
| `model`               | string   | 是       | 模型名称，使用 `agnes-video-v2.0`。           |
| `prompt`              | string   | 是       | 视频内容的文本描述。                          |
| `image`               | string   | 否       | 图生视频使用的图片 URL。                      |
| `mode`                | string   | 否       | 生成模式，例如 `ti2vid` 或 `keyframes`。      |
| `height`              | integer  | 否       | 视频高度，默认值为 `768`。                    |
| `width`               | integer  | 否       | 视频宽度，默认值为 `1152`。                   |
| `num_frames`          | integer  | 否       | 视频帧数，必须 `≤ 441` 且遵循 `8n + 1` 规则。 |
| `frame_rate`          | number   | 否       | 视频帧率，支持范围为 `1–60`。                 |
| `num_inference_steps` | integer  | 否       | 推理步数。                                    |
| `seed`                | integer  | 否       | 随机种子，用于生成可复现结果。                |
| `negative_prompt`     | string   | 否       | 反向提示词，描述需要避免的内容。              |
| `extra_body.image`    | array    | 否       | 关键帧模式下的输入图片 URL 数组。             |
| `extra_body.mode`     | string   | 否       | 附加模式设置，例如 `keyframes`。              |

**创建任务示例**:

Bash

```
curl -X POST https://apihub.agnes-ai.com/v1/videos \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "agnes-video-v2.0",
    "prompt": "A cinematic shot of a cat walking on the beach at sunset, soft ocean waves, warm golden lighting, realistic motion",
    "height": 768,
    "width": 1152,
    "num_frames": 121,
    "frame_rate": 24
  }'
```

**创建任务响应**:

JSON

```
{
  "id": "task_YOUR_TASK_ID",
  "task_id": "task_YOUR_TASK_ID",
  "video_id": "video_YOUR_VIDEO_ID",
  "object": "video",
  "model": "agnes-video-v2.0",
  "status": "queued",
  "progress": 0,
  "created_at": 1780457477,
  "seconds": "10.0",
  "size": "1280x768"
}
```

### 2. 获取视频结果

**推荐方式**:

Bash

```
curl --location --request GET 'https://apihub.agnes-ai.com/agnesapi?video_id=<VIDEO_ID>' \
  --header 'Authorization: Bearer YOUR_API_KEY'
```

*(兼容旧版方式： `GET https://apihub.agnes-ai.com/v1/videos/<TASK_ID>`)*

**获取结果响应**:

JSON

```
{
  "id": "task_YOUR_TASK_ID",
  "video_id": "video_YOUR_VIDEO_ID",
  "model": "agnes-video-v2.0",
  "object": "video",
  "status": "completed",
  "progress": 100,
  "seconds": "10.0",
  "size": "1280x768",
  "url": "https://platform-outputs.agnes-ai.space/videos/agnes-video-v2.0/video_xxxxxx.mp4",
  "error": null
}
```

**任务状态**:

| **状态**      | **说明**             |
| ------------- | -------------------- |
| `queued`      | 任务正在队列中等待。 |
| `in_progress` | 视频正在生成。       |
| `completed`   | 视频生成成功。       |
| `failed`      | 视频生成失败。       |

------

## 参数配置与最佳实践

### 参数标准化

Agnes Video V2.0 会对部分视频生成参数进行标准化处理。当提交的 `width`、`height` 或宽高比与模型支持规格不完全匹配时，系统会自动映射到最接近的标准输出尺寸。模型目前支持三个标准分辨率档位：`480p`、`720p` 和 `1080p`。

| **宽高比** | **推荐场景**                                               |
| ---------- | ---------------------------------------------------------- |
| `16:9`     | 横版视频、产品演示、网站展示、YouTube 风格内容。           |
| `9:16`     | 竖版短视频、移动端内容、TikTok / Reels / Shorts 风格内容。 |
| `1:1`      | 方形视频、社交媒体信息流、角色或产品展示。                 |
| `4:3`      | 传统横版格式和通用演示内容。                               |
| `3:4`      | 竖版演示、肖像或产品为主的内容。                           |

> **提示:** 展示任务信息、计算视频时长或排查生成结果问题时，请以 API 响应中的 `size`、`seconds` 等字段为准。

### 视频时长控制

视频时长由 `num_frames` 和 `frame_rate` 控制（`seconds = num_frames / frame_rate`）。`num_frames` 必须 **≤ 441**，并且必须遵循 **8n + 1** 规则。

| **目标时长** | **推荐参数**                        |
| ------------ | ----------------------------------- |
| 约 3 秒      | `num_frames: 81`, `frame_rate: 24`  |
| 约 5 秒      | `num_frames: 121`, `frame_rate: 24` |
| 约 10 秒     | `num_frames: 241`, `frame_rate: 24` |
| 约 18 秒     | `num_frames: 441`, `frame_rate: 24` |

### 提示词最佳实践

- **文生视频**
  - **推荐结构**: [主体] + [动作] + [场景] + [镜头运动] + [光线] + [风格]
  - **示例**: *A young astronaut walking across a red desert planet, dust blowing in the wind, slow cinematic tracking shot, dramatic sunset lighting, realistic sci-fi style*
- **图生视频**
  - **要求**: 描述哪些内容应该运动，以及哪些关键主体元素应该保持稳定。
  - **示例**: *Animate the character with subtle breathing motion, hair moving gently in the wind, background lights flickering softly, while keeping the face and outfit consistent*
- **关键帧动画**
  - **要求**: 清晰描述关键帧之间的过渡关系。
  - **示例**: *Create a smooth transition from the first keyframe to the second keyframe, maintaining character identity, consistent camera angle, and natural motion between scenes*

------

## 附加信息

### 错误码

| **状态码** | **说明**                   |
| ---------- | -------------------------- |
| `400`      | 请求无效。请检查请求参数。 |
| `401`      | 未授权。请检查 API Key。   |
| `404`      | 任务或视频未找到。         |
| `500`      | 服务器错误。               |
| `503`      | 服务繁忙。请稍后重试。     |

### 定价

| **类型** | **标准价格** | **当前价格** |
| -------- | ------------ | ------------ |
| 视频时长 | $0.005 / 秒  | **$0 / 秒**  |

### 接入检查清单



- [ ] 使用 `agnes-video-v2.0` 作为模型名称。
- [ ] 视频生成是异步任务，需要先创建任务，再获取结果。
- [ ] 创建任务响应会同时返回 `task_id` 和 `video_id`，新接入建议使用 `video_id`。
- [ ] `num_frames` 必须小于或等于 `441`，并遵循 `8n + 1` 规则。
- [ ] 图生视频使用 `image`，关键帧动画使用 `extra_body.image`。