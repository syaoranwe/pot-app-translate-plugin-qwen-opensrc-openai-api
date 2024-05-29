# Pot-App 通义千问1.5开源模型 翻译插件（OpenAI API格式调用）

## 介绍

1. 可自定义几乎所有的接口参数配置项，包括系统人设、提示词、采样温度、Top-P、惩罚因子等。
2. 由于Pot的插件不支持打印机流式效果，所以只支持获取到所有翻译内容后再一并显示。蹲一个Pot官方更新，以支持插件流式输出。
3. 代码里写死了超时时间为30秒，最大回复Token长度为2K。如有需要可自行修改。
4. 国内平台可能对输入输出存在审查，推荐自行部署或者使用国外MaaS平台。

## 配置参数

### 必填配置项

必填配置项包括三项，分别是API Key、请求地址、模型名。

下面以Infini-AI（无问苍穹）为例，介绍这三项配置。

1. API Key：请到此处生成：[Infini-AI密钥管理](https://cloud.infini-ai.com/genstudio/secret/key)
2. 请求地址：无问苍穹平台的API请求地址必须携带模型名，例如：
   - 72B版本：`https://cloud.infini-ai.com/maas/qwen1.5-72b-chat/nvidia/chat/completions`
   - 32B版本：`https://cloud.infini-ai.com/maas/qwen1.5-32b-chat/nvidia/chat/completions`
   - 14B版本：`https://cloud.infini-ai.com/maas/qwen1.5-14b-chat/nvidia/chat/completions`
   - 7B版本：`https://cloud.infini-ai.com/maas/qwen1.5-7b-chat/nvidia/chat/completions`
3. 模型名：
   - 72B版本：`qwen1.5-72b-chat`
   - 32B版本：`qwen1.5-32b-chat`
   - 14B版本：`qwen1.5-14b-chat`
   - 7B版本：`qwen1.5-7b-chat`

对于TogetherAI平台，配置如下：

1. API Key：请到此处生成：[TogetherAI密钥管理](https://api.together.xyz/settings/api-keys)
2. 请求地址：`https://api.together.xyz/v1/chat/completions`
3. 模型名：
   - 110B版本：`Qwen/Qwen1.5-110B-Chat`
   - 72B版本：`Qwen/Qwen1.5-72B-Chat`
   - 32B版本：`Qwen/Qwen1.5-32B-Chat`
   - 14B版本：`Qwen/Qwen1.5-14B-Chat`
   - 7B版本：`Qwen/Qwen1.5-7B-Chat`
   - 4B版本：`Qwen/Qwen1.5-4B-Chat`
   - 1.8B版本：`Qwen/Qwen1.5-1.8B-Chat`
   - 0.5B版本：`Qwen/Qwen1.5-0.5B-Chat`

### 可选配置项
1. System人设：系统人设提示词字符串, 留空时默认为`You are a professional, authentic translation engine which takes context into full consideration. You only return the translated text, without any explanations.`
2. 翻译提示词：用户自定义提示词列表, 由一行json字符串表示, 列表中元素值中的role有两种：user: 表示用户；assistant: 表示对话助手, content表示内容。content中的`$to$`会自动替换为译文语言描述, 例如`Traditional Chinese(繁體中文)`, `$src_text$`会自动被替换为原文文本。 如果留空则使用默认提示词（你可以在 json.cn 上编辑该提示词，之后压缩为一行即可导入程序）：
```text
[{"role":"user","content":"You are a professional translation engine, skilled in translating text into accurate, professional, fluent, and natural translations, avoiding mechanical literal translations like machine translation. You only translate the text without interpreting it. You only respond with the translated text and do not include any additional content."},{"role":"assistant","content":"OK, I will only translate the text content you provided, never interpret it."},{"role":"user","content":"Translate the text delimited by ``` below to Simplified Chinese(简体中文), only return translation:\n```\nHello, world!\n```\n"},{"role":"assistant","content":"你好，世界！"},{"role":"user","content":"Translate the text delimited by ``` below to English, only return translation:\n```\n再见，小明\n```\n"},{"role":"assistant","content":"Bye, Xiaoming."},{"role":"user","content":"Translate the text delimited by ``` below to $to$, only return translation:\n```\n$src_text$\n```\n"}]
```
1. temperature：留空时默认0.75, 范围 [0, 2.0], 值越小, 生成的内容越固定。当取0时，模型生成时几乎总是会选取概率最大的Token。越低的采样发散度模型将越倾向于使用机翻风格逐句翻译，越高的采样发散度模型的译文将越随机，可能导致译文丢失部分信息，也有可能会给出更流畅的译文。
2. top_p：留空时默认为1.0, 取值范围：(0, 1.0], 值越大, 生成的内容多样性越丰富，但在temperature较低的情况下越接近1，提升效果越趋近于没有。建议只调整temperature，不建议调整top_p。
3. presence_penalty：留空时默认为0, 取值范围：[-2.0, 2.0]. 值越高，模型给出下一个字词时，越不可能选择那些已经在原文和已生成的译文中重复过的字词。为负数时，模型会被鼓励生成原文和已生成译文中已有的字词。如果你希望模型给出译文时使用表达更丰富的词汇，可以适当增加该值，但这有可能生成更多疑难少用词汇。默认值设置为0，意味着既不鼓励模型沿用重复旧词，也不反对。翻译时建议略大于0，如0.1。
4. frequency_penalty：留空时默认为0, 取值范围：[-2.0, 2.0]. 值越高，模型在给出下一个字词时，越不可能选择那些已经在原文和已生成的译文中重复了多次的字词。为负数时，相当于鼓励模型多采用重复了多次的字词。默认值设置为0，意味着既不鼓励模型多次沿用重复旧词，也不反对。翻译时建议略大于0，如0.1。
