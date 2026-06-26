# Agnes Video V2.0

**Agnes-Video-V2.0 API 接入指南**

## 1. 概述

**Agnes-Video-V2.0** 是一款面向生产场景的视频生成模型，支持 **文生视频**、**图生视频**、**多图视频生成** 以及 **关键帧动画** 工作流。

开发者可以使用文本提示词、图片 URL 或多张参考图片生成高质量视频。该模型适用于故事讲述、营销视频、产品演示、社交媒体内容、应用动态素材以及 AI 创意工作流。

> **注意**：Agnes-Video-V2.0 采用基于异步任务的 API。您需要先创建一个视频生成任务，然后使用返回的 `video_id` 或 `task_id` 获取视频结果。

---

## 2. 支持能力

* **文生视频**：通过文本提示词直接生成视频
* **图生视频**：将静态图片转化为动态视频
* **多图视频生成**：使用多张参考图片引导视频生成
* **关键帧动画**：在多个关键帧之间生成流畅过渡
* **场景运动控制**：通过提示词控制主体动作、镜头运动和场景动态
* **视觉一致性**：在帧间保持一致的主体、风格和场景
* **电影级输出**：生成高质量电影级视频
* **异步 API**：先提交任务，再获取生成结果

---

## 3. 使用场景

* **故事讲述**：短片、角色场景、叙事片段
* **营销视频**：产品广告、宣传视频、推广内容
* **社交媒体内容**：Reels、Shorts、TikTok 风格视频
* **图片动画**：为肖像、产品、角色或场景添加动画效果
* **产品演示**：通过文本或图片生成产品展示视频
* **关键帧过渡**：在不同视觉状态之间生成流畅过渡
* **游戏/应用素材**：为数字产品生成动态视觉素材
* **沉浸式内容**：生成电影级 AI 场景和氛围视频

---

## 4. 前提条件

在接入之前，请确保您已满足以下条件：
1. 拥有有效的 Agnes AI API Key。
2. 具备访问 Agnes AI API 网关的网络条件。
3. 确认模型名称：`agnes-video-v2.0`。
4. 准备好用于视频生成的文本提示词。
5. 如果使用图生视频、多图视频或关键帧动画功能，需要提供可公开访问的图片 URL。

---

## 5. API 接口

### 创建视频任务
| 项目 | 说明 |
| :--- | :--- |
| **接口地址** | `https://apihub.agnes-ai.com/v1/videos` |
| **请求方法** | POST |
| **Content-Type** | application/json |
| **认证方式** | Bearer Token |
| **请求头** | `Authorization: Bearer YOUR_API_KEY` |

### 获取视频结果：推荐方式
视频任务创建成功后，响应中会包含一个 `video_id`。推荐使用 `video_id` 来获取视频结果。
| 项目 | 说明 |
| :--- | :--- |
| **接口地址** | `https://apihub.agnes-ai.com/agnesapi?video_id=<VIDEO_ID>` |
| **请求方法** | GET |
| **请求头** | `Authorization: Bearer YOUR_API_KEY` |

### 获取视频结果：兼容旧版方式
| 项目 | 说明 |
| :--- | :--- |
| **接口地址** | `https://apihub.agnes-ai.com/v1/videos/{task_id}` |
| **请求方法** | GET |
| **请求头** | `Authorization: Bearer YOUR_API_KEY` |

---

## 6. 请求参数

### 创建视频任务参数

| 参数 | 类型 | 必填 | 说明 |
| :--- | :--- | :--- | :--- |
| **model** | string | 是 | 模型名称。使用 `agnes-video-v2.0` |
| **prompt** | string | 是 | 视频内容的文本描述 |
| **image** | string / array | 否 | 图片 URL 或图片 URL 数组 |
| **mode** | string | 否 | 生成模式，例如 `ti2vid` 或 `keyframes` |
| **height** | integer | 否 | 视频高度。默认值：`768` |
| **width** | integer | 否 | 视频宽度。默认值：`1152` |
| **num_frames** | integer | 否 | 视频帧数。必须 ≤ 441 且遵循 `8n + 1` 规则 |
| **frame_rate** | number | 否 | 视频帧率。支持范围：`1–60` |
| **num_inference_steps**| integer | 否 | 推理步数 |
| **seed** | integer | 否 | 随机种子，用于生成可复现的结果 |
| **negative_prompt** | string | 否 | 反向提示词，描述需要避免的内容 |
| **extra_body.image** | array | 否 | 多图视频或关键帧模式下的输入图片 URL 数组 |
| **extra_body.mode** | string | 否 | 附加模式设置，例如 `keyframes` |

### 参数标准化说明
当提交的 `width`、`height` 或宽高比与模型支持的标准规格不完全匹配时，系统会自动识别最接近的分辨率档位和宽高比。
模型目前支持三个标准分辨率档位：**480p**、**720p** 和 **1080p**。

**推荐宽高比：**
* **16:9**：横版视频、产品演示、YouTube 风格内容
* **9:16**：竖版短视频、TikTok / Reels / Shorts 风格内容
* **1:1**：方形视频、社交媒体信息流
* **4:3**：传统横版格式及通用演示内容
* **3:4**：竖版演示、肖像或产品为主的视频

> **注意**：开发者应以 API 响应中返回的 `size`、`seconds` 等字段为准。

---

## 7. 创建视频任务示例

### 文生视频
使用此请求通过文本提示词直接生成视频。
```bash
curl -X POST [https://apihub.agnes-ai.com/v1/videos](https://apihub.agnes-ai.com/v1/videos) \
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

------

## 8. 创建任务响应

视频任务创建成功后，API 会返回任务信息。响应中同时包含 `task_id` 和 `video_id`。

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

------

## 9. 获取视频结果

### 1. 推荐方式：通过 video_id 获取

Bash

```
curl --location --request GET '[https://apihub.agnes-ai.com/agnesapi?video_id=video_xxxxxx](https://apihub.agnes-ai.com/agnesapi?video_id=video_xxxxxx)' \
  --header 'Authorization: Bearer YOUR_API_KEY'
```

*(可选)* 您还可以传入 `model_name` 来显式指定模型名称：`&model_name=agnes-video-v2.0`

### 2. 兼容旧版方式：通过 task_id 获取

Bash

```
curl --location --request GET '[https://apihub.agnes-ai.com/v1/videos/task_xxxxxx](https://apihub.agnes-ai.com/v1/videos/task_xxxxxx)' \
  --header 'Authorization: Bearer YOUR_API_KEY'
```

### 获取结果响应

任务完成后（`status` 为 `completed`），API 返回最终视频结果：

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
  "remixed_from_video_id": "[https://storage.googleapis.com/agnes-aigc/aigc/videos/2026/06/03/video_xxxxxx.mp4](https://storage.googleapis.com/agnes-aigc/aigc/videos/2026/06/03/video_xxxxxx.mp4)",
  "error": null
}
```

**任务状态说明：**

- `queued`: 任务正在队列中等待
- `in_progress`: 视频正在生成
- `completed`: 视频生成成功（此时生成视频的 URL 会在 `remixed_from_video_id` 字段中）
- `failed`: 视频生成失败

------

## 10. 视频时长控制

时长计算公式：`seconds = num_frames / frame_rate`

- `num_frames` 必须 **≤ 441**，且遵循 **8n + 1** 规则（如 81, 121, 161, 241, 441）。
- `frame_rate` 支持 **1 到 60** 之间的值。

| **目标时长** | **推荐参数**                        |
| ------------ | ----------------------------------- |
| **约 3 秒**  | `num_frames: 81`, `frame_rate: 24`  |
| **约 5 秒**  | `num_frames: 121`, `frame_rate: 24` |
| **约 10 秒** | `num_frames: 241`, `frame_rate: 24` |
| **约 18 秒** | `num_frames: 441`, `frame_rate: 24` |

------

## 11. 提示词最佳实践

- **文生视频提示词**：推荐结构 `[主体] + [动作] + [场景] + [镜头运动] + [光线] + [风格]`
- **图生视频提示词**：描述哪些内容应该运动，以及哪些关键主体元素应该保持稳定。
- **多图视频提示词**：描述输入图片之间的关系以及场景应该如何过渡。
- **关键帧动画提示词**：清晰描述关键帧之间的过渡关系。

------

## 12. 错误码与定价

### 错误码

| **状态码** | **说明**                   |
| ---------- | -------------------------- |
| **400**    | 请求无效。请检查请求参数   |
| **401**    | 未授权。请检查您的 API Key |
| **404**    | 任务或视频未找到           |
| **500**    | 服务器错误                 |
| **503**    | 服务繁忙。请稍后重试       |

### 定价

| **类型**     | **标准价格** | **当前价格** |
| ------------ | ------------ | ------------ |
| **视频时长** | $0.005 / 秒  | **$0 / 秒**  |

------

## 13. 注意事项清单



- [ ] 使用 `agnes-video-v2.0` 作为模型名称。
- [ ] 视频生成是异步的。您需要先创建视频任务，然后获取视频结果。
- [ ] 创建任务响应会同时返回 `task_id` 和 `video_id`，新接入的集成应使用 `video_id` 获取视频结果。
- [ ] 最终的视频 URL（`remixed_from_video_id`）仅在 `status` 为 `completed` 时可用。
- [ ] `num_frames` 必须小于或等于 **441**，且必须遵循 **8n + 1** 规则。
- [ ] **图生视频**任务需要通过 `image` 传入图片 URL。
- [ ] **多图视频**任务需要在 `extra_body.image` 中传入多个图片 URL。
- [ ] **关键帧动画**需要将 `extra_body.mode` 设置为 `keyframes`。