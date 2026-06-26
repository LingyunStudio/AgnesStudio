# Agnes Image 2.0 Flash

## 1. 模型介绍

**Agnes-Image-2.0-Flash** 是由 **Sapiens AI** 开发的高性能图像生成与图像编辑模型。 该模型支持 **文生图**、**图生图** 和 **多图合成** 工作流程，适用于快速创意制作、图像增强、营销视觉设计、电商产品图、社交内容生成以及专业视觉内容创作。 Agnes-Image-2.0-Flash 已登上 **Artificial Analysis 图像编辑排行榜**，获得 **ELO 评分 1,184**（动态调整），进入 **Top 20** 区间，展现了在主流图像模型中强大的图像编辑能力。

---

## 2. 模型概览

Agnes-Image-2.0-Flash 针对快速、高质量的图像生成和图像编辑任务进行了优化。 该模型支持以下能力：

* **文生图**：通过文本提示词生成图像
* **图生图**：编辑、变换或增强现有图像
* **多图输入**：将多张参考图像组合为一张新图像
* **图像编辑**：修改构图、风格、物体、背景、场景和视觉细节
* **风格控制**：调整艺术风格、光照、布局和视觉方向
* **快速生成**：针对快速且高性价比的生产工作流进行优化
* **OpenAI 兼容 API**：使用与 OpenAI Images API 兼容的请求结构

---

## 3. 使用场景

* **创意设计**：海报、概念艺术、社交媒体视觉素材
* **营销内容**：产品广告、活动创意、横幅
* **文生图**：通过提示词生成产品图、插画、场景图和概念艺术
* **图像编辑**：物体替换、背景更换、风格迁移、局部图像编辑
* **角色合成**：将多个角色或参考图像组合到同一场景中
* **视觉制作**：为应用、网站、游戏和视频生成素材
* **电商**：产品图像增强、产品场景化、营销主图
* **社交内容**：表情包、头像、缩略图、生活方式视觉素材

---

## 4. API 信息

* **Base URL**: `https://apihub.agnes-ai.com`
* **Endpoint**: `POST https://apihub.agnes-ai.com/v1/images/generations`
* **请求头**: 
    * `-H "Authorization: Bearer YOUR_API_KEY"`
    * `-H "Content-Type: application/json"`

---

## 5. 模型名称

`agnes-image-2.0-flash`

---

## 6. 请求参数

| 参数 | 类型 | 必填 | 说明 |
| :--- | :--- | :--- | :--- |
| **model** | string | 是 | 模型名称，固定为 `agnes-image-2.0-flash` |
| **prompt** | string | 是 | 描述目标图像或编辑指令的文本提示词 |
| **size** | string | 是 | 输出图像尺寸，如 `1024x768`、`1024x1024` 或 `768x1024` |
| **image** | string[] | 图生图时必填 | 输入图像数组，支持公网 URL 或 Data URI Base64 |
| **return_base64** | boolean | 否 | 文生图返回 Base64 输出时使用 |
| **extra_body.response_format** | string | 否 | 输出格式，常用值：`url` 或 `b64_json` |

---

## 7. 重要说明

### 1. 文生图不需要 image 参数
文生图生成时，仅需以下字段：
```json
{
  "model": "agnes-image-2.0-flash",
  "prompt": "A clean product photo of a glass cube on a white studio background, soft shadows, high detail",
  "size": "1024x768"
}
```

### 2. 图生图需要 image 参数

图生图或多图合成时，需在顶层传入 `image` 数组：

JSON

```
{
  "image": [
    "[https://example.com/input.png](https://example.com/input.png)"
  ]
}
```

多图合成时，可以提供多个图像 URL：

JSON

```
{
  "image": [
    "[https://example.com/character-1.png](https://example.com/character-1.png)",
    "[https://example.com/character-2.png](https://example.com/character-2.png)"
  ]
}
```

### 3. 图生图不需要 tags 参数

在当前接入格式中，图生图请求不需要 `{"tags": ["img2img"]}`。仅需 `model`、`prompt`、`size` 和 `image` 参数。

### 4. 请勿将 response_format 放在顶层

**不要这样写：**

JSON

```
{ "response_format": "url" }
```

**推荐格式：**

JSON

```
{
  "extra_body": {
    "response_format": "url"
  }
}
```

将 `response_format` 放在顶层可能导致 400 错误。

------

## 8. 请求示例

### 1. 文生图：URL 输出

Bash

```
curl [https://apihub.agnes-ai.com/v1/images/generations](https://apihub.agnes-ai.com/v1/images/generations) \
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

生成的图像 URL 位于：`data[0].url`

### 2. 文生图：Base64 输出

Bash

```
curl [https://apihub.agnes-ai.com/v1/images/generations](https://apihub.agnes-ai.com/v1/images/generations) \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "agnes-image-2.0-flash",
    "prompt": "A clean product photo of a glass cube on a white studio background, soft shadows, high detail",
    "size": "1024x768",
    "return_base64": true
  }'
```

生成的图像 Base64 数据位于：data[0].b64_json

### 3. 图生图：URL 输入，URL 输出

用于编辑或变换现有图像。

Bash

```
curl [https://apihub.agnes-ai.com/v1/images/generations](https://apihub.agnes-ai.com/v1/images/generations) \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "agnes-image-2.0-flash",
    "prompt": "Transform this image into a cinematic cyberpunk style while preserving the main subject and composition",
    "size": "1024x768",
    "extra_body": {
      "image": [
        "[https://example.com/input-image.png](https://example.com/input-image.png)"
      ],
      "response_format": "url"
    }
  }'
```

生成的图像 URL 位于：`data[0].url`

### 4. 图生图：URL 输入，Base64 输出

Bash

```
curl [https://apihub.agnes-ai.com/v1/images/generations](https://apihub.agnes-ai.com/v1/images/generations) \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "agnes-image-2.0-flash",
    "prompt": "Make the object orange while preserving the original composition",
    "size": "1024x768",
    "extra_body": {
      "image": [
        "[https://example.com/input.png](https://example.com/input.png)"
      ],
      "response_format": "b64_json"
    }
  }'
```

生成的图像 Base64 数据位于：`data[0].b64_json`

### 5. 图生图：Data URI Base64 输入

如果输入图像不是公网 URL，可以使用 Data URI Base64 作为输入。

**Data URI 格式**：`data:image/png;base64,BASE64_HERE`

Bash

```
curl [https://apihub.agnes-ai.com/v1/images/generations](https://apihub.agnes-ai.com/v1/images/generations) \
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

### 6. 多图合成请求

用于将多张输入图像组合为新场景。

Bash

```
curl [https://apihub.agnes-ai.com/v1/images/generations](https://apihub.agnes-ai.com/v1/images/generations) \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "agnes-image-2.0-flash",
    "prompt": "Combine the two characters into an intense fantasy battle scene, dynamic lighting, detailed background, cinematic composition",
    "size": "1024x768",
    "extra_body": {
      "image": [
        "[https://example.com/character-1.png](https://example.com/character-1.png)",
        "[https://example.com/character-2.png](https://example.com/character-2.png)"
      ],
      "response_format": "url"
    }
  }'
```

------

## 9. 响应格式

### 1. URL 输出

JSON

```
{
  "created": 1780000000,
  "data": [
    {
      "url": "[https://storage.googleapis.com/agnes-aigc/xxx.png](https://storage.googleapis.com/agnes-aigc/xxx.png)",
      "b64_json": null,
      "revised_prompt": null
    }
  ]
}
```

### 2. Base64 输出

JSON

```
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

------

## 10. 响应字段

| **字段**                  | **类型**      | **说明**                                      |
| ------------------------- | ------------- | --------------------------------------------- |
| **created**               | integer       | 请求创建时间戳                                |
| **data**                  | array         | 生成的图像结果列表                            |
| **data[].url**            | string / null | 生成的图像 URL，使用 Base64 输出时通常为 null |
| **data[].b64_json**       | string / null | Base64 图像数据，使用 URL 输出时通常为 null   |
| **data[].revised_prompt** | string / null | 修正后的提示词（如有），否则为 null           |

------

## 11. 定价

| **类型**     | **原价**    | **当前价格** |
| ------------ | ----------- | ------------ |
| **生成图像** | $0.003 / 张 | **$0 / 张**  |

------

## 12. 功能与兼容性

Agnes-Image-2.0-Flash 支持以下能力：

- 文生图生成
- 图生图编辑
- 多图输入与合成
- 基于提示词的图像变换
- 稳定的风格和构图控制
- 公网 URL 图像输入
- Data URI Base64 图像输入
- URL 或 Base64 输出
- 面向生产工作流的快速生成
- 与 OpenAI Images API 兼容的请求结构

------

## 13. 最佳实践

### 1. 文生图提示词

为获得更好的生成质量，请在提示词中提供清晰的视觉指令，包括主体、场景、风格、光照、构图和质量要求。

> **示例**：`A professional product photo of a wireless headphone on a clean white background, soft studio lighting, sharp details, commercial photography style`

### 2. 图像编辑提示词

对于编辑任务，请清楚描述需要改变的内容和需要保持不变的内容。

> **示例**：`Change the background to a futuristic city at night while keeping the person's face, outfit, and pose unchanged`

### 3. 多图合成提示词

对于多图合成任务，请描述不同输入图像之间的关系。

> **示例**：`Place the person from the first image beside the robot from the second image in a cinematic sci-fi battle scene`

------

## 14. 推荐提示词结构

### 文生图提示词结构

```
[主体] + [场景/背景] + [风格] + [光照] + [构图] + [质量要求]
```

> **示例**：`A young explorer standing in an ancient temple, cinematic fantasy style, warm dramatic lighting, wide-angle composition, ultra detailed, high quality`

### 图生图提示词结构

```
[编辑指令] + [需要保留的元素] + [目标风格/场景] + [光照] + [构图] + [质量要求]
```

> **示例**：`Change the background into a cinematic fantasy temple while preserving the person's face, outfit, and pose, warm dramatic lighting, wide-angle composition, ultra detailed, high quality`

------

## 15. 常见问题

### 1. Agnes-Image-2.0-Flash 是否支持文生图？

支持。文生图请求不需要 `image` 参数，仅需 `model`、`prompt` 和 `size`。

### 2. Agnes-Image-2.0-Flash 是否支持图生图？

支持。图生图请求需要顶层 `image` 数组。

### 3. 图生图是否需要 tags: ["img2img"] ？

不需要。当前图生图请求仅需 `model`、`prompt`、`size` 和 `image`。

### 4. 为什么将 response_format 放在顶层会报错？

在当前 API 结构中，`response_format` 不应放在顶层。

推荐格式：`{"extra_body": {"response_format": "url"}}`

### 5. 输入图像 URL 无法访问怎么办？

如果服务器无法访问输入图像 URL，请求可能会失败。推荐方案：

- 使用公网可访问的 HTTPS 图像 URL
- 使用 Data URI Base64 输入

### 6. 请求超时怎么办？

图像生成可能需要数秒到数十秒不等。建议设置较长的客户端超时时间，例如：**60s - 360s**

------

## 16. 接入检查清单



接入前请确认：

- [ ] 您拥有有效的 API Key
- [ ] 请求 URL 为 `https://apihub.agnes-ai.com/v1/images/generations`
- [ ] 已包含 `Authorization: Bearer YOUR_API_KEY` 请求头
- [ ] 已包含 `Content-Type: application/json` 请求头
- [ ] 模型名称为 `agnes-image-2.0-flash`
- [ ] 文生图请求不包含 `image` 参数
- [ ] 图生图请求包含顶层 `image` 数组
- [ ] 图生图请求不需要 `tags: ["img2img"]`
- [ ] `response_format` 放在 `extra_body` 内部
- [ ] 输入图像 URL 为公网可访问，或使用 Data URI Base64
- [ ] 客户端超时建议设置为 60s 至 360s