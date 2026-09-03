> ## Documentation Index
> Fetch the complete documentation index at: https://wiki.agnes-ai.com/llms.txt
> Use this file to discover all available pages before exploring further.

# Agnes Video 2.5 Flash

> 使用 OpenAI Videos 兼容 API 接入 Agnes Video 2.5 Flash，支持文生视频、首尾帧控制和图片参考生成。

<Info>
  Agnes Video 2.5 Flash 复用 Agnes Video 2.5 的模型能力和异步任务接口。除本页列出的 Flash 专属限制外，其他请求参数、响应字段和查询方式均与 [Agnes Video 2.5](/zh-Hans/docs/agnes-video-25) 一致。
</Info>

<CardGroup cols={2}>
  <Card title="模型 ID" icon="cube">
    `agnes-video-2.5-flash`
  </Card>

  <Card title="创建任务" icon="video">
    `POST /v1/videos`
  </Card>

  <Card title="查询任务" icon="clock">
    `GET /agnesapi?video_id=<VIDEO_ID>&model_name=agnes-video-2.5-flash`
  </Card>

  <Card title="当前价格" icon="tag">
    原价 ~~`$0.025 / 秒`~~，现价 `$0 / 秒`
  </Card>
</CardGroup>

## 与 Agnes Video 2.5 的差异

| 校验项              | Flash 规则           | 校验失败响应                                     |
| ---------------- | ------------------ | ------------------------------------------ |
| `size`           | 仅支持字符串 `"720P"`    | HTTP 400：`size must be 720P`               |
| `reference` 图片数量 | `images` 最多 5 张    | HTTP 400：`images length must not exceed 5` |
| `reference` 视频输入 | 不支持有效的 `videos` 内容 | HTTP 400：`videos is not supported`         |

<Warning>
  Flash 专属校验在任务创建、排队、计费和推理前执行。校验失败的请求不会创建视频任务，也不会产生费用。
</Warning>

除上述限制外，`agnes-video-2.5-flash` 沿用 `agnes-video-2.5` 的公共参数能力和校验逻辑。

## 快速接入

### 1. 设置环境变量

```bash theme={null}
export AGNES_API_KEY="YOUR_API_KEY"
export AGNES_BASE_URL="https://apihub.agnes-ai.com/v1"
```

### 2. 创建视频任务

```bash theme={null}
curl -sS -X POST "$AGNES_BASE_URL/videos" \
  -H "Authorization: Bearer $AGNES_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "agnes-video-2.5-flash",
    "prompt": "雨后的未来城市街道，霓虹灯倒映在地面，一辆银色跑车缓慢驶过，电影级运镜，自然环境声",
    "seconds": "5",
    "mode": "text",
    "size": "720P",
    "aspect_ratio": "16:9"
  }'
```

创建成功后，保存响应中的 `video_id`。`id` 和 `task_id` 是任务 ID，`video_id` 用于查询任务。

### 3. 查询任务结果

<Tabs>
  <Tab title="推荐方式：video_id + model_name">
    ```bash theme={null}
    curl -sS "https://apihub.agnes-ai.com/agnesapi?video_id=VIDEO_ID&model_name=agnes-video-2.5-flash" \
      -H "Authorization: Bearer $AGNES_API_KEY"
    ```

    适用于 `text`、`keyframe` 和 `reference` 全部模式，是 Agnes Video 2.5 Flash 的推荐轮询方式。
  </Tab>

  <Tab title="仅 video_id（仅 text 模式）">
    ```bash theme={null}
    curl -sS "https://apihub.agnes-ai.com/agnesapi?video_id=VIDEO_ID" \
      -H "Authorization: Bearer $AGNES_API_KEY"
    ```

    仅适用于创建任务时使用 `mode: "text"` 的任务。`keyframe` 和 `reference` 模式必须指定 `model_name=agnes-video-2.5-flash`。
  </Tab>
</Tabs>

建议每隔 `1–2` 秒查询一次，直至 `status` 变为 `completed` 或 `failed`。

## 请求参数

### 公共参数

| 参数             | 类型      | 必填 | 说明                                                       |
| -------------- | ------- | -- | -------------------------------------------------------- |
| `model`        | string  | 是  | 使用 `agnes-video-2.5-flash`。                              |
| `prompt`       | string  | 是  | 视频内容描述。Reference 模式可使用 `<Picture N>` 和 `<Audio N>` 指代素材。 |
| `mode`         | string  | 是  | `text`、`keyframe` 或 `reference`。                         |
| `seconds`      | string  | 否  | 视频时长，支持字符串 `"4"`–`"12"`，默认 `"5"`。                        |
| `size`         | string  | 否  | Flash 固定为 `"720P"`；其他值返回 HTTP 400。                       |
| `aspect_ratio` | string  | 否  | 默认 `16:9`，支持值见“视频尺寸与画幅”。                                 |
| `seed`         | integer | 否  | 随机种子。                                                    |
| `n`            | integer | 否  | 当前仅支持 `1`，默认 `1`。                                        |

### 模式专用参数

| 参数            | 类型        | 适用模式        | 说明                                   |
| ------------- | --------- | ----------- | ------------------------------------ |
| `first_frame` | string    | `keyframe`  | 首帧图片 URL；与 `last_frame` 至少提供一个。      |
| `last_frame`  | string    | `keyframe`  | 尾帧图片 URL；与 `first_frame` 至少提供一个。     |
| `images`      | string\[] | `reference` | 参考图片 URL 列表，Flash 最多支持 5 张。          |
| `audios`      | string\[] | `reference` | 参考音频 URL 列表，沿用 Agnes Video 2.5 公共规则。 |
| `videos`      | object\[] | `reference` | Flash 不支持；传入有效内容返回 HTTP 400。         |

### 生成模式规则

| `mode`      | 用途          | 必需媒体                              | 不允许的媒体字段                                              |
| ----------- | ----------- | --------------------------------- | ----------------------------------------------------- |
| `text`      | 纯文本生成视频     | 无                                 | `first_frame`、`last_frame`、`images`、`audios`、`videos` |
| `keyframe`  | 首帧、尾帧或首尾帧控制 | `first_frame` 与 `last_frame` 至少一个 | `images`、`audios`、`videos`                            |
| `reference` | 图片或音频参考生成   | `images` 或 `audios` 至少一类非空        | `first_frame`、`last_frame`、`videos`                   |

所有媒体 URL 都应可由 Agnes AI 服务公开访问，并在任务完成前保持有效。

## 请求示例

<Tabs>
  <Tab title="文生视频">
    ```bash theme={null}
    curl -sS -X POST "$AGNES_BASE_URL/videos" \
      -H "Authorization: Bearer $AGNES_API_KEY" \
      -H "Content-Type: application/json" \
      -d '{
        "model": "agnes-video-2.5-flash",
        "prompt": "夜晚森林中三只猫组成微型铜管乐队向前行进，镜头平稳后退，月光穿过树叶",
        "seconds": "5",
        "mode": "text",
        "size": "720P",
        "aspect_ratio": "16:9"
      }'
    ```
  </Tab>

  <Tab title="首尾帧控制">
    ```bash theme={null}
    curl -sS -X POST "$AGNES_BASE_URL/videos" \
      -H "Authorization: Bearer $AGNES_API_KEY" \
      -H "Content-Type: application/json" \
      -d '{
        "model": "agnes-video-2.5-flash",
        "prompt": "人物从首帧姿态自然转身走向窗边，镜头缓慢推进并平滑过渡到尾帧",
        "seconds": "5",
        "mode": "keyframe",
        "size": "720P",
        "first_frame": "https://example.com/first.png",
        "last_frame": "https://example.com/last.png"
      }'
    ```
  </Tab>

  <Tab title="图片参考">
    ```bash theme={null}
    curl -sS -X POST "$AGNES_BASE_URL/videos" \
      -H "Authorization: Bearer $AGNES_API_KEY" \
      -H "Content-Type: application/json" \
      -d '{
        "model": "agnes-video-2.5-flash",
        "prompt": "以 <Picture 1> 中的角色和美术风格为参考，角色在花田中自然奔跑，保持外观一致",
        "seconds": "5",
        "mode": "reference",
        "size": "720P",
        "aspect_ratio": "16:9",
        "images": ["https://example.com/character.png"]
      }'
    ```
  </Tab>
</Tabs>

## 视频尺寸与画幅

`size` 必须使用 `"720P"`。具体输出尺寸通过 `aspect_ratio` 选择：

| `aspect_ratio` | 输出像素       |
| -------------- | ---------- |
| `21:9`         | `1680x720` |
| `16:9`         | `1280x720` |
| `4:3`          | `960x720`  |
| `1:1`          | `720x720`  |
| `3:4`          | `720x960`  |
| `9:16`         | `720x1280` |

## Flash 专属错误

同一次请求存在多个 Flash 参数错误时，接口按照 `size`、`images`、`videos` 的顺序返回首个检测到的错误。

<Tabs>
  <Tab title="size 非 720P">
    ```json theme={null}
    {
      "detail": "size must be 720P"
    }
    ```
  </Tab>

  <Tab title="图片超过 5 张">
    ```json theme={null}
    {
      "detail": "images length must not exceed 5"
    }
    ```
  </Tab>

  <Tab title="传入参考视频">
    ```json theme={null}
    {
      "detail": "videos is not supported"
    }
    ```
  </Tab>
</Tabs>

以上响应的 HTTP 状态码均为 `400`。其他错误码、任务响应字段和失败任务格式与 Agnes Video 2.5 一致。

## 接入检查清单

* 模型 ID 使用 `agnes-video-2.5-flash`。
* `size` 固定为字符串 `"720P"`。
* `mode=reference` 时，`images` 不超过 5 张。
* `mode=reference` 时不要传入有效的 `videos` 内容。
* `seconds` 使用字符串 `"4"`–`"12"`，`n` 固定为 `1`。
* 所有模式推荐使用 `video_id` 和 `model_name=agnes-video-2.5-flash` 查询；不带 `model_name` 的纯 `video_id` 查询仅适用于 `mode: "text"`。
* 不要在前端代码、日志或公开仓库中暴露 API Key。

## 计费规则

<Info>
  Agnes Video 2.5 Flash 的 720P 原价与 Agnes Video 2.5 相同，当前为限时免费。
</Info>

| 输出分辨率 | 原价               | 现价           |
| ----- | ---------------- | ------------ |
| 720P  | ~~`$0.025 / 秒`~~ | **`$0 / 秒`** |

当前免费期间，输出视频秒数、输入素材和参考图片均按 `$0` 计费。免费政策如有调整，以 Agnes AI 平台最新公告为准。
