# Agnes Image 2.1 Flash

## 1. 模型概述

**Agnes Image 2.1 Flash** 是 Sapiens AI 推出的升级图像生成模型，支持 **文生图** 和 **图生图** 两种工作流。 

与之前版本相比，Agnes Image 2.1 Flash 在 **高信息密度图像** 生成方面性能有所提升，更适合需要复杂视觉细节、更丰富构图以及更清晰语义对齐的场景。 Agnes Image 2.1 Flash 可用于根据文本提示词生成图像、转换现有图像、在编辑过程中保留原始构图，并以图像 URL 或 Base64 数据形式返回结果。

---

## 2. 核心能力

* **文生图**：根据自然语言提示词生成高质量图像
* **图生图**：根据提示词指令转换或优化现有图像
* **高信息密度图像优化**：改进对细节丰富、布局复杂、视觉元素密集图像的处理效果
* **构图保留**：在编辑或转换输入图像时保留原始构图
* **灵活尺寸控制**：支持 1024x768 等自定义输出尺寸
* **URL 响应**：以可访问的图像 URL 形式返回生成的图像结果
* **Base64 响应**：按需以 Base64 数据形式返回生成的图像结果
* **URL 或 Data URI 输入**：图生图支持公共图像 URL 或 Data URI Base64 输入

---

## 3. 适用场景

Agnes Image 2.1 Flash 适用于：

* **创意设计**：概念艺术、视觉探索、海报草稿
* **营销内容**：活动图片、产品视觉、社交媒体创意
* **高密度视觉生成**：精细场景、丰富构图、复杂环境
* **图像转换**：风格迁移、场景重打光、背景变换
* **内容制作**：应用资源、缩略图、横幅、叙事视觉
* **产品可视化**：产品照片、模型图、商业视觉
* **社交媒体素材**：封面、横幅、缩略图、帖子图片

---

## 4. API 信息

* **Base URL**: `https://apihub.agnes-ai.com`
* **Endpoint**: `POST https://apihub.agnes-ai.com/v1/images/generations`
* **请求头**: 
    * `-H "Authorization: Bearer YOUR_API_KEY"`
    * `-H "Content-Type: application/json"`

---

## 5. 模型名称

文生图和图生图工作流均使用以下模型名称： 
`agnes-image-2.1-flash`

---

## 6. 重要说明

* 使用 `agnes-image-2.1-flash` 作为模型名称。
* 文生图生成时，`model`、`prompt` 和 `size` 为必填参数。
* 图生图生成时，请在顶层 `image` 数组中提供输入图像 URL 或 Data URI Base64。
* **请勿将 `response_format` 放在请求体的顶层。**
* 如果需要 URL 输出，请将 `"response_format": "url"` 放在 `extra_body` 内部。
* 如果文生图需要 Base64 输出，可以使用顶层参数 `"return_base64": true`。
* 图生图 Base64 输出，请在 `extra_body` 内部使用 `"response_format": "b64_json"`。
* 图生图请求不需要传递 `tags: ["img2img"]`。
* 请勿在公开文档中暴露临时 API 密钥。所有公开示例请使用 `YOUR_API_KEY`。

---

## 7. 请求参数

| 参数 | 类型 | 是否必填 | 描述 |
| :--- | :--- | :--- | :--- |
| **model** | string | 是 | 模型名称，请使用 `agnes-image-2.1-flash` |
| **prompt** | string | 是 | 图像生成或图像编辑的文本指令 |
| **size** | string | 是 | 输出图像尺寸，例如 `1024x768` |
| **image** | string[] | 图生图必填 | 输入图像数组，支持公共图像 URL 或 Data URI Base64 |
| **return_base64** | boolean | 否 | 文生图输出需要以 Base64 返回时使用 |
| **extra_body** | object | 否 | 高级工作流的附加参数 |
| **extra_body.response_format** | string | 否 | 输出格式，常见值：`url`, `b64_json` |

---

## 8. 调用示例

### 8.1 文生图请求（URL 输出）
使用此请求根据文本提示词生成图像，并以图像 URL 形式返回结果。
```bash
curl [https://apihub.agnes-ai.com/v1/images/generations](https://apihub.agnes-ai.com/v1/images/generations) \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "agnes-image-2.1-flash",
    "prompt": "A luminous floating city above a misty canyon at sunrise, cinematic realism",
    "size": "1024x768",
    "extra_body": {
      "response_format": "url"
    }
  }'
```

> 生成的图像 URL 返回路径：`data[0].url`

### 8.2 文生图请求（Base64 输出）

当您需要生成的图像以 Base64 数据形式返回时，请使用此请求。

Bash

```
curl [https://apihub.agnes-ai.com/v1/images/generations](https://apihub.agnes-ai.com/v1/images/generations) \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "agnes-image-2.1-flash",
    "prompt": "A clean product photo of a glass cube on a white studio background, soft shadows, high detail",
    "size": "1024x768",
    "return_base64": true
  }'
```

> 生成的 Base64 图像返回路径：`data[0].b64_json`

### 8.3 图生图请求（URL 输入 + URL 输出）

使用此请求转换现有图像，同时保留原始构图。

Bash

```
curl [https://apihub.agnes-ai.com/v1/images/generations](https://apihub.agnes-ai.com/v1/images/generations) \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "agnes-image-2.1-flash",
    "prompt": "Transform the scene into a rain-soaked cyberpunk night with neon reflections while preserving the original composition",
    "size": "1024x768",
    "extra_body": {
      "image": [
        "[https://example.com/input-image.png](https://example.com/input-image.png)"
      ],
      "response_format": "url"
    }
  }'
```

> 生成的图像 URL 返回路径：`data[0].url`

### 8.4 图生图请求（URL 输入 + Base64 输出）

当输入图像以公共 URL 形式提供，且生成结果需要以 Base64 数据形式返回时，请使用此请求。

Bash

```
curl [https://apihub.agnes-ai.com/v1/images/generations](https://apihub.agnes-ai.com/v1/images/generations) \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "agnes-image-2.1-flash",
    "prompt": "Make the object orange while preserving the original composition",
    "size": "1024x768",
    "extra_body": {
      "image": [
        "[https://example.com/input-image.png](https://example.com/input-image.png)"
      ],
      "response_format": "b64_json"
    }
  }'
```

> 生成的 Base64 图像返回路径：`data[0].b64_json`

### 8.5 图生图请求（Data URI Base64 输入）

图生图也支持 Data URI Base64 输入。

*Data URI 格式*：`data:image/png;base64,BASE64_HERE`

Bash

```
curl [https://apihub.agnes-ai.com/v1/images/generations](https://apihub.agnes-ai.com/v1/images/generations) \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "agnes-image-2.1-flash",
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

------

## 9. 响应格式

### URL 输出

当 `extra_body.response_format` 设置为 `url` 时，响应格式如下：

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

### Base64 输出

启用 Base64 输出时，响应格式如下：

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

## 10. 推荐提示词结构

为获得更好的图像生成效果，请使用清晰的提示词结构：

```
[主体] + [场景 / 环境] + [风格] + [光照] + [构图] + [质量要求]
```

- **文生图示例**：日出时分薄雾峡谷上方的发光浮空城市，电影级写实风格，广角构图，丰富的建筑细节，柔和的金色光线，高视觉密度
- **图生图示例**：将场景转换为霓虹倒影的雨浸赛博朋克夜晚，同时保留原始构图和主体布局。

------

## 11. 最佳实践

### 文生图最佳实践

生成复杂图像时请使用详细的提示词，包含主体、环境、风格、光照、相机角度以及期望的细节级别。

- **优秀示例**：充满飞行器、全息招牌、密集人群、霓虹灯光的未来城市市场，电影级写实风格，超精细，高信息密度构图
- **推荐包含的元素**：主要主体、场景或环境、视觉风格、光照、相机角度、构图、细节级别、质量要求

### 图生图最佳实践

编辑现有图像时，请同时明确指定转换要求和保留要求。

- **优秀示例**：将图像转换为奇幻冬季景观，添加雪花、温暖的窗灯和神奇氛围，同时保留原始建筑结构和相机角度。
- **推荐结构**：`[改变要求] + [新风格 / 场景] + [需要添加或移除的元素] + [需要保留的元素]`
- **示例**：将白天街道场景改为电影级赛博朋克夜景，添加霓虹招牌和湿滑路面倒影，同时保留原始街道布局、相机角度和主要建筑形状。

### 高信息密度图像最佳实践

Agnes Image 2.1 Flash 针对复杂且细节丰富的视觉效果进行了优化。为获得最佳效果，请清晰描述视觉层次结构。

- **推荐包含的元素**：主要主体、背景环境、重要的次要细节、风格和光照、构图约束、(如果使用图生图)需要保持不变的内容。
- **优秀示例**：建在悬崖上的大型奇幻港口城市，数百艘小船，层叠的石桥，发光的窗户，远山，多云的日落天空，电影级奇幻写实风格，广角构图，丰富的建筑细节，高视觉密度。

------

## 12. 常见错误与故障排除

### 1. 顶层放置 response_format 会导致错误

请勿将 `response_format` 放在顶层。

- **错误写法**：`{"model": "...", "response_format": "url"}`
- **正确写法**：`{"model": "...", "extra_body": {"response_format": "url"}}`

### 2. 图生图不需要 tags

请勿传递：`{"tags": ["img2img"]}`

图生图只需在 `image` 数组中提供输入图像即可。

### 3. 输入图像 URL 无法访问

如果服务器无法访问输入图像 URL，请求可能会失败。

- **推荐解决方案**：使用公共 HTTPS 图像 URL；确保图像 URL 不需要登录、cookie 或私有请求头；如果图像无法公开访问，请使用 Data URI Base64 输入。

### 4. 请求超时

根据提示词复杂度、图像尺寸和服务器负载情况，图像生成可能需要几秒到几十秒不等。

- **推荐的客户端超时时间**：60s 到 360s

### 5. 图生图请求缺少 image 参数

图生图生成时，`image` 数组为必填项。

- **错误写法**：仅提供 `model`, `prompt`, `size`，缺失图像输入。
- **正确写法**：在 `extra_body`（或顶层）中正确传入 `{"image": ["https://..."]}`。

------

## 13. 定价

| **类型**     | **原价**    | **当前价格** |
| ------------ | ----------- | ------------ |
| **生成图像** | $0.003 / 张 | **$0 / 张**  |

------

## 14. 注意事项



- 使用 `agnes-image-2.1-flash` 作为模型名称。
- 使用 `https://apihub.agnes-ai.com/v1/images/generations` 作为 API 端点。
- 文生图生成时，`model`、`prompt` 和 `size` 为必填参数。
- 图生图生成时，请在顶层 `image` 数组下提供输入图像 URL 或 Data URI Base64。
- 需要生成结果以图像 URL 返回时，使用 `extra_body.response_format: "url"`。
- 文生图 Base64 输出，请使用 `return_base64: true`。
- 图生图 Base64 输出，请使用 `extra_body.response_format: "b64_json"`。
- 请勿将 `response_format` 放在顶层。
- 请勿传递 `tags: ["img2img"]`。
- 请勿在公开文档中暴露临时 API 密钥。所有公开示例请使用 `YOUR_API_KEY`。