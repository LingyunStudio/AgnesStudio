> ## Documentation Index
> Fetch the complete documentation index at: https://wiki.agnes-ai.com/llms.txt
> Use this file to discover all available pages before exploring further.

# Agnes Image 2.0 Flash

> 面向文生图、图生图和多图合成的高性能图像生成与编辑模型。

<Info>
  Agnes Image 2.0 Flash 是 Agnes AI 的高性能图像生成与图像编辑模型，支持文生图、图生图和多图合成，适合创意设计、营销视觉、电商产品图和社交内容生产。
</Info>

<CardGroup cols={2}>
  <Card title="模型名称" icon="cube">
    `agnes-image-2.0-flash`
  </Card>

  <Card title="API Endpoint" icon="link">
    `POST /v1/images/generations`
  </Card>

  <Card title="支持工作流" icon="wand-magic-sparkles">
    文生图、图生图、多图合成。
  </Card>

  <Card title="当前价格" icon="tag">
    生成图像当前为 `$0 / 张`
  </Card>
</CardGroup>

## 概述

Agnes Image 2.0 Flash 针对快速、高质量、低成本的图像生产工作流进行了优化。

该模型已登上 Artificial Analysis 图像编辑排行榜，获得 **ELO 评分 1,184**，进入 **Top 20** 区间，展现了较强的图像编辑能力。

## 核心能力

<CardGroup cols={2}>
  <Card title="文生图" icon="wand-magic-sparkles">
    通过文本提示词生成图像。
  </Card>

  <Card title="图生图" icon="images">
    编辑、变换或增强现有图像。
  </Card>

  <Card title="多图输入" icon="layer-group">
    使用多张参考图像组合生成新图像。
  </Card>

  <Card title="图像编辑" icon="paintbrush">
    修改构图、风格、物体、背景和场景。
  </Card>

  <Card title="风格控制" icon="palette">
    控制艺术风格、光照、布局和视觉方向。
  </Card>

  <Card title="快速生成" icon="bolt">
    适合高频、快速、生产级创意工作流。
  </Card>
</CardGroup>

## 适用场景

<CardGroup cols={2}>
  <Card title="创意设计" icon="pen-ruler">
    海报、概念艺术、社交媒体视觉素材。
  </Card>

  <Card title="营销内容" icon="bullhorn">
    产品广告、活动创意和横幅图。
  </Card>

  <Card title="图像编辑" icon="paintbrush">
    物体替换、背景更换、风格迁移和局部编辑。
  </Card>

  <Card title="角色合成" icon="users">
    将多个角色或参考图像组合到同一场景。
  </Card>

  <Card title="电商" icon="cart-shopping">
    产品图像增强、场景化和营销主图。
  </Card>

  <Card title="社交内容" icon="share-nodes">
    表情包、头像、缩略图和生活方式视觉素材。
  </Card>
</CardGroup>

## API Reference

### Endpoint

```text theme={null}
POST https://apihub.agnes-ai.com/v1/images/generations
```

### 请求头

```bash theme={null}
-H "Authorization: Bearer YOUR_API_KEY"
-H "Content-Type: application/json"
```

### 请求参数

| 参数                           | 类型        | 必填    | 说明                                             |
| ---------------------------- | --------- | ----- | ---------------------------------------------- |
| `model`                      | string    | 是     | 模型名称，使用 `agnes-image-2.0-flash`。               |
| `prompt`                     | string    | 是     | 描述目标图像或编辑指令的文本提示词。                             |
| `size`                       | string    | 是     | 输出图像尺寸，例如 `1024x768`、`1024x1024` 或 `768x1024`。 |
| `image`                      | string\[] | 图生图必填 | 输入图像数组，支持公网 URL 或 Data URI Base64。             |
| `return_base64`              | boolean   | 否     | 文生图需要返回 Base64 时使用。                            |
| `extra_body.response_format` | string    | 否     | 输出格式，常用值为 `url` 或 `b64_json`。                  |

## 重要说明

<Warning>
  请勿将 `response_format` 放在请求体顶层。需要 URL 或 Base64 输出时，请将它放在 `extra_body` 内部。
</Warning>

<CardGroup cols={2}>
  <Card title="文生图" icon="wand-magic-sparkles">
    仅需 `model`、`prompt` 和 `size`，不需要传 `image`。
  </Card>

  <Card title="图生图" icon="images">
    需要在 `extra_body.image` 中传入图片 URL 或 Data URI Base64。
  </Card>

  <Card title="不需要 tags" icon="ban">
    图生图请求不需要 `tags: ["img2img"]`。
  </Card>

  <Card title="安全示例" icon="key">
    公开文档中请统一使用 `YOUR_API_KEY`，不要暴露真实密钥。
  </Card>
</CardGroup>

## 请求示例

<Tabs>
  <Tab title="文生图：URL 输出">
    ```bash theme={null}
    curl https://apihub.agnes-ai.com/v1/images/generations \
      -H "Authorization: Bearer YOUR_API_KEY" \
      -H "Content-Type: application/json" \
      -d '{
        "model": "agnes-image-2.0-flash",
        "prompt": "A clean product photo of a glass cube on a white studio background, soft shadows, high detail",
        "size": "1024x768",
        "extra_body": {
          "response_format": "url"
        }
      }'
    ```

    返回路径：`data[0].url`
  </Tab>

  <Tab title="文生图：Base64 输出">
    ```bash theme={null}
    curl https://apihub.agnes-ai.com/v1/images/generations \
      -H "Authorization: Bearer YOUR_API_KEY" \
      -H "Content-Type: application/json" \
      -d '{
        "model": "agnes-image-2.0-flash",
        "prompt": "A clean product photo of a glass cube on a white studio background, soft shadows, high detail",
        "size": "1024x768",
        "return_base64": true
      }'
    ```

    返回路径：`data[0].b64_json`
  </Tab>

  <Tab title="图生图：URL 输出">
    ```bash theme={null}
    curl https://apihub.agnes-ai.com/v1/images/generations \
      -H "Authorization: Bearer YOUR_API_KEY" \
      -H "Content-Type: application/json" \
      -d '{
        "model": "agnes-image-2.0-flash",
        "prompt": "Transform this image into a cinematic cyberpunk style while preserving the main subject and composition",
        "size": "1024x768",
        "extra_body": {
          "image": [
            "https://example.com/input-image.png"
          ],
          "response_format": "url"
        }
      }'
    ```

    返回路径：`data[0].url`
  </Tab>

  <Tab title="图生图：Base64 输出">
    ```bash theme={null}
    curl https://apihub.agnes-ai.com/v1/images/generations \
      -H "Authorization: Bearer YOUR_API_KEY" \
      -H "Content-Type: application/json" \
      -d '{
        "model": "agnes-image-2.0-flash",
        "prompt": "Make the object orange while preserving the original composition",
        "size": "1024x768",
        "extra_body": {
          "image": [
            "https://example.com/input.png"
          ],
          "response_format": "b64_json"
        }
      }'
    ```

    返回路径：`data[0].b64_json`
  </Tab>

  <Tab title="多图合成">
    ```bash theme={null}
    curl https://apihub.agnes-ai.com/v1/images/generations \
      -H "Authorization: Bearer YOUR_API_KEY" \
      -H "Content-Type: application/json" \
      -d '{
        "model": "agnes-image-2.0-flash",
        "prompt": "Combine the two characters into an intense fantasy battle scene, dynamic lighting, detailed background, cinematic composition",
        "size": "1024x768",
        "extra_body": {
          "image": [
            "https://example.com/character-1.png",
            "https://example.com/character-2.png"
          ],
          "response_format": "url"
        }
      }'
    ```
  </Tab>

  <Tab title="Data URI 输入">
    ```text theme={null}
    data:image/png;base64,BASE64_HERE
    ```

    ```bash theme={null}
    curl https://apihub.agnes-ai.com/v1/images/generations \
      -H "Authorization: Bearer YOUR_API_KEY" \
      -H "Content-Type: application/json" \
      -d '{
        "model": "agnes-image-2.0-flash",
        "prompt": "Make the object matte black while preserving the original composition",
        "size": "1024x768",
        "extra_body": {
          "image": [
            "data:image/png;base64,BASE64_HERE"
          ],
          "response_format": "b64_json"
        }
      }'
    ```
  </Tab>
</Tabs>

## 响应格式

<Tabs>
  <Tab title="URL 输出">
    ```json theme={null}
    {
      "created": 1780000000,
      "data": [
        {
          "url": "https://storage.googleapis.com/agnes-aigc/xxx.png",
          "b64_json": null,
          "revised_prompt": null
        }
      ]
    }
    ```
  </Tab>

  <Tab title="Base64 输出">
    ```json theme={null}
    {
      "created": 1780000000,
      "data": [
        {
          "url": null,
          "b64_json": "iVBORw0KGgoAAAANSUhEUgAA...",
          "revised_prompt": null
        }
      ]
    }
    ```
  </Tab>
</Tabs>

### 响应字段

| 字段                      | 类型            | 说明                             |
| ----------------------- | ------------- | ------------------------------ |
| `created`               | integer       | 请求创建时间戳。                       |
| `data`                  | array         | 生成的图像结果列表。                     |
| `data[].url`            | string / null | 生成图像 URL，Base64 输出时通常为 `null`。 |
| `data[].b64_json`       | string / null | Base64 图像数据，URL 输出时通常为 `null`。 |
| `data[].revised_prompt` | string / null | 修正后的提示词；没有时为 `null`。           |

## 最佳实践

<AccordionGroup>
  <Accordion title="文生图提示词">
    建议包含主体、场景、风格、光照、构图和质量要求。

    ```text theme={null}
    A professional product photo of a wireless headphone on a clean white background, soft studio lighting, sharp details, commercial photography style
    ```
  </Accordion>

  <Accordion title="图像编辑提示词">
    请同时说明需要改变的内容和需要保持不变的内容。

    ```text theme={null}
    Change the background to a futuristic city at night while keeping the person's face, outfit, and pose unchanged
    ```
  </Accordion>

  <Accordion title="多图合成提示词">
    请明确说明多张输入图之间的关系。

    ```text theme={null}
    Place the person from the first image beside the robot from the second image in a cinematic sci-fi battle scene
    ```
  </Accordion>

  <Accordion title="推荐提示词结构">
    ```text theme={null}
    [主体] + [场景/背景] + [风格] + [光照] + [构图] + [质量要求]
    ```

    ```text theme={null}
    [编辑指令] + [需要保留的元素] + [目标风格/场景] + [光照] + [构图] + [质量要求]
    ```
  </Accordion>
</AccordionGroup>

## 常见问题

<AccordionGroup>
  <Accordion title="是否支持文生图？">
    支持。文生图请求不需要 `image` 参数，仅需 `model`、`prompt` 和 `size`。
  </Accordion>

  <Accordion title="是否支持图生图？">
    支持。图生图请求需要在 `extra_body.image` 中传入图片数组。
  </Accordion>

  <Accordion title="图生图是否需要 tags？">
    不需要。请勿传递 `tags: ["img2img"]`。
  </Accordion>

  <Accordion title="为什么 response_format 放在顶层会报错？">
    当前 API 结构中，`response_format` 应放在 `extra_body` 内部，例如 `extra_body.response_format: "url"`。
  </Accordion>

  <Accordion title="输入图像 URL 无法访问怎么办？">
    请使用公网可访问的 HTTPS 图像 URL；如果图像无法公开访问，请改用 Data URI Base64。
  </Accordion>

  <Accordion title="请求超时怎么办？">
    图像生成可能需要数秒到数十秒，客户端超时时间建议设置为 `60s - 360s`。
  </Accordion>
</AccordionGroup>

## 定价

| 类型   | 标准价格         | 当前价格     |
| ---- | ------------ | -------- |
| 生成图像 | `$0.003 / 张` | `$0 / 张` |

## 接入检查清单

<Check>
  请求 URL 为 `https://apihub.agnes-ai.com/v1/images/generations`。
</Check>

<Check>
  模型名称为 `agnes-image-2.0-flash`。
</Check>

<Check>
  文生图请求不传 `image`。
</Check>

<Check>
  图生图请求在 `extra_body.image` 中传入图片数组。
</Check>

<Check>
  `response_format` 放在 `extra_body` 内部。
</Check>
