> ## Documentation Index
> Fetch the complete documentation index at: https://wiki.agnes-ai.com/llms.txt
> Use this file to discover all available pages before exploring further.

# Agnes Video V2.0

> 面向文生视频、图生视频和关键帧动画的异步视频生成 API。

<Info>
  Agnes Video V2.0 是面向生产场景的视频生成模型，支持文生视频、图生视频和关键帧动画。视频生成采用异步任务 API：先创建任务，再通过 `video_id` 或 `task_id` 获取结果。
</Info>

<CardGroup cols={2}>
  <Card title="模型名称" icon="cube">
    `agnes-video-v2.0`
  </Card>

  <Card title="创建任务" icon="video">
    `POST /v1/videos`
  </Card>

  <Card title="获取结果" icon="link">
    `GET /agnesapi?video_id=<VIDEO_ID>`
  </Card>

  <Card title="当前价格" icon="tag">
    视频时长当前为 `$0 / 秒`
  </Card>
</CardGroup>

## 概述

开发者可以使用文本提示词或图片 URL 生成高质量视频。该模型适用于故事讲述、营销视频、产品演示、社交媒体内容、应用动态素材和 AI 创意工作流。

## 核心能力

<CardGroup cols={2}>
  <Card title="文生视频" icon="clapperboard">
    通过文本提示词直接生成视频。
  </Card>

  <Card title="图生视频" icon="image">
    将静态图片转化为动态视频。
  </Card>

  <Card title="关键帧动画" icon="timeline">
    在多个关键帧之间生成流畅过渡。
  </Card>

  <Card title="场景运动控制" icon="camera">
    通过提示词控制主体动作、镜头运动和场景动态。
  </Card>

  <Card title="视觉一致性" icon="eye">
    在帧间保持主体、风格和场景一致。
  </Card>

  <Card title="电影级输出" icon="film">
    生成高质量电影级视频内容。
  </Card>

  <Card title="异步 API" icon="clock">
    创建任务后再轮询或查询生成结果。
  </Card>
</CardGroup>

## 适用场景

<CardGroup cols={2}>
  <Card title="故事讲述" icon="book-open">
    短片、角色场景和叙事片段。
  </Card>

  <Card title="营销视频" icon="bullhorn">
    产品广告、宣传视频和推广内容。
  </Card>

  <Card title="社交媒体内容" icon="share-nodes">
    Reels、Shorts、TikTok 风格视频。
  </Card>

  <Card title="图片动画" icon="wand-magic-sparkles">
    为肖像、产品、角色或场景添加动画效果。
  </Card>

  <Card title="产品演示" icon="box">
    通过文本或图片生成产品展示视频。
  </Card>

  <Card title="关键帧过渡" icon="arrows-left-right">
    在不同视觉状态之间生成流畅过渡。
  </Card>
</CardGroup>

## 前提条件

<Note>
  在接入之前，请确认拥有有效的 Agnes AI API Key，网络可访问 Agnes AI API 网关，并已准备好用于视频生成的文本提示词。图生视频或关键帧动画还需要提供可公开访问的图片 URL。
</Note>

## API Reference

### 创建视频任务

```text theme={null}
POST https://apihub.agnes-ai.com/v1/videos
```

### 获取视频结果：推荐方式

```text theme={null}
GET https://apihub.agnes-ai.com/agnesapi?video_id=<VIDEO_ID>
```

### 获取视频结果：兼容旧版方式

```text theme={null}
GET https://apihub.agnes-ai.com/v1/videos/<TASK_ID>
```

### 请求头

```bash theme={null}
-H "Authorization: Bearer YOUR_API_KEY"
-H "Content-Type: application/json"
```

## 创建任务参数

| 参数                    | 类型      | 必填 | 说明                               |
| --------------------- | ------- | -- | -------------------------------- |
| `model`               | string  | 是  | 模型名称，使用 `agnes-video-v2.0`。      |
| `prompt`              | string  | 是  | 视频内容的文本描述。                       |
| `image`               | string  | 否  | 图生视频使用的图片 URL。                   |
| `mode`                | string  | 否  | 生成模式，例如 `ti2vid` 或 `keyframes`。  |
| `height`              | integer | 否  | 视频高度，默认值为 `768`。                 |
| `width`               | integer | 否  | 视频宽度，默认值为 `1152`。                |
| `num_frames`          | integer | 否  | 视频帧数，必须 `≤ 441` 且遵循 `8n + 1` 规则。 |
| `frame_rate`          | number  | 否  | 视频帧率，支持范围为 `1–60`。               |
| `num_inference_steps` | integer | 否  | 推理步数。                            |
| `seed`                | integer | 否  | 随机种子，用于生成可复现结果。                  |
| `negative_prompt`     | string  | 否  | 反向提示词，描述需要避免的内容。                 |
| `extra_body.image`    | array   | 否  | 关键帧模式下的输入图片 URL 数组。              |
| `extra_body.mode`     | string  | 否  | 附加模式设置，例如 `keyframes`。           |

## 参数标准化

<Note>
  Agnes Video V2.0 会对部分视频生成参数进行标准化处理。当提交的 `width`、`height` 或宽高比与模型支持规格不完全匹配时，系统会自动映射到最接近的标准输出尺寸。
</Note>

模型目前支持三个标准分辨率档位：`480p`、`720p` 和 `1080p`。

| 宽高比    | 推荐场景                                      |
| ------ | ----------------------------------------- |
| `16:9` | 横版视频、产品演示、网站展示、YouTube 风格内容。              |
| `9:16` | 竖版短视频、移动端内容、TikTok / Reels / Shorts 风格内容。 |
| `1:1`  | 方形视频、社交媒体信息流、角色或产品展示。                     |
| `4:3`  | 传统横版格式和通用演示内容。                            |
| `3:4`  | 竖版演示、肖像或产品为主的内容。                          |

<Tip>
  展示任务信息、计算视频时长或排查生成结果问题时，请以 API 响应中的 `size`、`seconds` 和 `metadata.size_mapping` 等字段为准。
</Tip>

## 创建任务示例

<Tabs>
  <Tab title="文生视频">
    ```bash theme={null}
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
  </Tab>

  <Tab title="图生视频">
    ```bash theme={null}
    curl -X POST https://apihub.agnes-ai.com/v1/videos \
      -H "Authorization: Bearer YOUR_API_KEY" \
      -H "Content-Type: application/json" \
      -d '{
        "model": "agnes-video-v2.0",
        "prompt": "The woman slowly turns around and looks back at the camera, natural facial expression, cinematic camera movement",
        "image": "https://example.com/image.png",
        "num_frames": 121,
        "frame_rate": 24
      }'
    ```
  </Tab>

  <Tab title="关键帧动画">
    ```bash theme={null}
    curl -X POST https://apihub.agnes-ai.com/v1/videos \
      -H "Authorization: Bearer YOUR_API_KEY" \
      -H "Content-Type: application/json" \
      -d '{
        "model": "agnes-video-v2.0",
        "prompt": "Generate a smooth cinematic transition between the keyframes, maintaining visual consistency and natural camera movement",
        "extra_body": {
          "image": [
            "https://example.com/keyframe1.png",
            "https://example.com/keyframe2.png"
          ],
          "mode": "keyframes"
        },
        "num_frames": 121,
        "frame_rate": 24
      }'
    ```
  </Tab>
</Tabs>

## 创建任务响应

```json theme={null}
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

| 字段           | 类型      | 说明                  |
| ------------ | ------- | ------------------- |
| `id`         | string  | 任务 ID，可与旧版查询接口配合使用。 |
| `task_id`    | string  | 任务 ID，作用与 `id` 相同。  |
| `video_id`   | string  | 视频 ID，推荐用于获取视频结果。   |
| `object`     | string  | 对象类型，通常为 `video`。   |
| `model`      | string  | 当前任务使用的模型。          |
| `status`     | string  | 当前任务状态。             |
| `progress`   | integer | 当前任务进度百分比。          |
| `created_at` | integer | 任务创建时间戳。            |
| `seconds`    | string  | 视频时长，单位为秒。          |
| `size`       | string  | 视频分辨率。              |

## 获取视频结果

<Tabs>
  <Tab title="推荐方式：video_id">
    ```bash theme={null}
    curl --location --request GET 'https://apihub.agnes-ai.com/agnesapi?video_id=<VIDEO_ID>' \
      --header 'Authorization: Bearer YOUR_API_KEY'
    ```
  </Tab>

  <Tab title="指定 model_name">
    ```bash theme={null}
    curl --location --request GET 'https://apihub.agnes-ai.com/agnesapi?video_id=<VIDEO_ID>&model_name=agnes-video-v2.0' \
      --header 'Authorization: Bearer YOUR_API_KEY'
    ```

    适用于使用上游原始视频 ID、非默认模型，或需要显式指定模型名称的场景。
  </Tab>

  <Tab title="兼容旧版：task_id">
    ```bash theme={null}
    curl --location --request GET 'https://apihub.agnes-ai.com/v1/videos/<TASK_ID>' \
      --header 'Authorization: Bearer YOUR_API_KEY'
    ```
  </Tab>
</Tabs>

## 获取结果响应

任务完成后，最终生成的视频 URL 位于 `metadata.url`。

```json theme={null}
{
  "id": "task_YOUR_TASK_ID",
  "video_id": "task_YOUR_TASK_ID",
  "task_id": "task_YOUR_TASK_ID",
  "object": "video",
  "model": "agnes-video-v2.0",
  "status": "completed",
  "progress": 100,
  "created_at": 1784530473,
  "completed_at": 1784530510,
  "seconds": "1.0",
  "size": "832x448",
  "metadata": {
    "size_mapping": {
      "adjusted": true,
      "height": 448,
      "message": "Input size 1024x576 was mapped to nearest preset 480p/16:9 (832x448)",
      "ratio": "16:9",
      "requested_height": 576,
      "requested_width": 1024,
      "resolution": "480p",
      "width": 832
    },
    "url": "https://platform-outputs.agnes-ai.space/videos/agnes-video-v2.0/task_YOUR_TASK_ID.mp4"
  }
}
```

| 字段                      | 类型            | 说明                                           |
| ----------------------- | ------------- | -------------------------------------------- |
| `id`                    | string        | 任务 ID。                                       |
| `video_id`              | string        | API 返回的视频 ID。请将其视为不透明 ID；该值可能与 `task_id` 相同。 |
| `task_id`               | string        | 任务 ID，作用与 `id` 相同。                           |
| `model`                 | string        | 当前任务使用的模型。                                   |
| `object`                | string        | 对象类型。                                        |
| `status`                | string        | 任务状态。                                        |
| `progress`              | integer       | 任务进度百分比。                                     |
| `created_at`            | integer       | 任务创建时间戳。                                     |
| `completed_at`          | integer       | 任务完成时间戳。                                     |
| `seconds`               | string        | 视频时长，单位为秒。                                   |
| `size`                  | string        | 标准化后的实际输出视频分辨率。                              |
| `metadata`              | object        | 结果附加元数据。                                     |
| `metadata.url`          | string        | 最终生成的视频 URL，仅在 `status` 为 `completed` 时可用。   |
| `metadata.size_mapping` | object        | 尺寸标准化信息，包括请求尺寸、实际输出尺寸、宽高比和分辨率档位。             |
| `error`                 | object / null | 任务失败时返回的错误信息。成功响应中该字段可能不存在。                  |

## 任务状态

| 状态            | 说明         |
| ------------- | ---------- |
| `queued`      | 任务正在队列中等待。 |
| `in_progress` | 视频正在生成。    |
| `completed`   | 视频生成成功。    |
| `failed`      | 视频生成失败。    |

## 视频时长控制

视频时长由 `num_frames` 和 `frame_rate` 控制。

```text theme={null}
seconds = num_frames / frame_rate
```

<Warning>
  `num_frames` 必须小于或等于 `441`，并且必须遵循 `8n + 1` 规则。
</Warning>

| 目标时长   | 推荐参数                                |
| ------ | ----------------------------------- |
| 约 3 秒  | `num_frames: 81`, `frame_rate: 24`  |
| 约 5 秒  | `num_frames: 121`, `frame_rate: 24` |
| 约 10 秒 | `num_frames: 241`, `frame_rate: 24` |
| 约 18 秒 | `num_frames: 441`, `frame_rate: 24` |

## 推荐参数

| 场景       | 推荐设置                                                              |
| -------- | ----------------------------------------------------------------- |
| 标准视频生成   | `width: 1152`, `height: 768`, `num_frames: 121`, `frame_rate: 24` |
| 社交短视频    | `num_frames: 81` 或 `121`, `frame_rate: 24`                        |
| 较长视频     | 增大 `num_frames` 或降低 `frame_rate`。                                 |
| 更流畅的运动   | 使用 `frame_rate: 24` 或 `30`。                                       |
| 可复现结果    | 设置固定 `seed`。                                                      |
| 关键帧过渡    | 使用 `extra_body.mode: "keyframes"`。                                |
| 避免不需要的内容 | 使用 `negative_prompt`。                                             |

## 提示词最佳实践

<AccordionGroup>
  <Accordion title="文生视频提示词">
    推荐结构：

    ```text theme={null}
    [主体] + [动作] + [场景] + [镜头运动] + [光线] + [风格]
    ```

    示例：

    ```text theme={null}
    A young astronaut walking across a red desert planet, dust blowing in the wind, slow cinematic tracking shot, dramatic sunset lighting, realistic sci-fi style
    ```
  </Accordion>

  <Accordion title="图生视频提示词">
    描述哪些内容应该运动，以及哪些关键主体元素应该保持稳定。

    ```text theme={null}
    Animate the character with subtle breathing motion, hair moving gently in the wind, background lights flickering softly, while keeping the face and outfit consistent
    ```
  </Accordion>

  <Accordion title="关键帧动画提示词">
    清晰描述关键帧之间的过渡关系。

    ```text theme={null}
    Create a smooth transition from the first keyframe to the second keyframe, maintaining character identity, consistent camera angle, and natural motion between scenes
    ```
  </Accordion>
</AccordionGroup>

## 错误码

| 状态码   | 说明               |
| ----- | ---------------- |
| `400` | 请求无效。请检查请求参数。    |
| `401` | 未授权。请检查 API Key。 |
| `404` | 任务或视频未找到。        |
| `500` | 服务器错误。           |
| `503` | 服务繁忙。请稍后重试。      |

## 定价

| 类型   | 标准价格         | 当前价格     |
| ---- | ------------ | -------- |
| 视频时长 | `$0.005 / 秒` | `$0 / 秒` |

## 接入检查清单

<Check>
  使用 `agnes-video-v2.0` 作为模型名称。
</Check>

<Check>
  视频生成是异步任务，需要先创建任务，再获取结果。
</Check>

<Check>
  创建任务响应会同时返回 `task_id` 和 `video_id`，新接入建议使用 `video_id`。
</Check>

<Check>
  `num_frames` 必须小于或等于 `441`，并遵循 `8n + 1` 规则。
</Check>

<Check>
  图生视频使用 `image`，关键帧动画使用 `extra_body.image`。
</Check>
