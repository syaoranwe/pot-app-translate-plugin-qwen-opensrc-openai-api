use reqwest::header;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;

// 默认的系统提示词
const DEFAULT_SYSTEM_PROMPT: &str = "You are a professional, authentic translation engine which takes context into full consideration. You only return the translated text, without any explanations.";
// 默认的提示词列表，使用JSON格式字符串表示
const DEFAULT_PROMPTS: &str = r#"[{"role":"user","content":"You are a professional translation engine, skilled in translating text into accurate, professional, fluent, and natural translations, avoiding mechanical literal translations like machine translation. You only translate the text without interpreting it. You only respond with the translated text and do not include any additional content."},{"role":"assistant","content":"OK, I will only translate the text content you provided, never interpret it."},{"role":"user","content":"Translate the text delimited by ``` below to Simplified Chinese(简体中文), only return translation:\n```\nHello, world!\n```\n"},{"role":"assistant","content":"你好，世界！"},{"role":"user","content":"Translate the text delimited by ``` below to English, only return translation:\n```\n再见，小明\n```\n"},{"role":"assistant","content":"Bye, Xiaoming."},{"role":"user","content":"Translate the text delimited by ``` below to $to$, only return translation:\n```\n$src_text$\n```\n"}]"#;
// 默认的temperature值
const DEFAULT_TEMPERATURE: &str = "0.75";
// 默认的top_p值
const DEFAULT_TOP_P: &str = "1.0";
// 默认的presence_penalty值
const DEFAULT_PRESENCE_PENALTY: &str = "0.0";
// 默认的frequency_penalty值
const DEFAULT_FREQUENCY_PENALTY: &str = "0.0";

#[no_mangle]
pub fn translate(
    text: &str,  // 待翻译的文本
    _from: &str,  // 不使用，忽略
    to: &str,  // 目标语言，例如：English
    _detect: &str,  // 不使用，忽略
    needs: HashMap<String, String>,  // 传入的自定义参数，例如：api_key, request_url, model_string等
) -> Result<Value, Box<dyn Error>> {
    // 检查needs是否包含必要的参数，如果没有则报错
    let api_key = needs.get("api_key").ok_or("缺少必要参数: api_key: 接口访问密钥")?;
    let request_url = needs.get("request_url").ok_or("缺少必要参数: request_url: 请求地址")?;
    let model_string = needs.get("model_string").ok_or("缺少必要参数: model_string: 模型名")?;

    // 使用needs中的可选参数为变量赋值，如果没有则使用默认值
    // 使用.to_owned() 将字符串字面量转换为String类型
    let system_prompt = needs.get("system_prompt").map(String::to_owned).unwrap_or_else(|| DEFAULT_SYSTEM_PROMPT.to_owned());
    let prompts = needs.get("prompts").map(String::to_owned).unwrap_or_else(|| DEFAULT_PROMPTS.to_owned());

    let temperature = needs.get("temperature").map(String::to_owned).unwrap_or_else(|| DEFAULT_TEMPERATURE.to_owned());
    let top_p = needs.get("top_p").map(String::to_owned).unwrap_or_else(|| DEFAULT_TOP_P.to_owned());
    let presence_penalty = needs.get("presence_penalty").map(String::to_owned).unwrap_or_else(|| DEFAULT_PRESENCE_PENALTY.to_owned());
    let frequency_penalty = needs.get("frequency_penalty").map(String::to_owned).unwrap_or_else(|| DEFAULT_FREQUENCY_PENALTY.to_owned());

    // 将temperature、top_p、presence_penalty、frequency_penalty转换为浮点数，同时检查是否在范围里，如果不在范围则报错
    let temperature: f64 = temperature.parse().map_err(|_| "temperature参数值转换错误")?;
    let top_p: f64 = top_p.parse().map_err(|_| "top_p参数值转换错误")?;
    let presence_penalty: f64 = presence_penalty.parse().map_err(|_| "presence_penalty参数值转换错误")?;
    let frequency_penalty: f64 = frequency_penalty.parse().map_err(|_| "frequency_penalty参数值转换错误")?;

    // temperature的范围是[0, 2.0]
    if !(0.0 <= temperature && temperature <= 2.0) {
        return Err("temperature参数范围有误，正确的范围是[0, 2.0]".into());
    }
    // top_p的范围是(0, 1.0]
    if !(0.0 < top_p && top_p <= 1.0) {
        return Err("top_p参数范围有误，正确的范围是(0, 1.0]".into());
    }
    // presence_penalty的范围是[-2.0, 2.0]
    if !(-2.0 <= presence_penalty && presence_penalty <= 2.0) {
        return Err("presence_penalty参数范围有误，正确的范围是[-2.0, 2.0]".into());
    }
    // frequency_penalty的范围是[-2.0, 2.0]
    if !(-2.0 <= frequency_penalty && frequency_penalty <= 2.0) {
        return Err("frequency_penalty参数范围有误，正确的范围是[-2.0, 2.0]".into());
    }

    // 构造请求的payload: 将prompts中的$to$替换为to, $src_text$替换为text, 然后转换为json格式payload
    // 将prompts转换为Value类型
    let prompts_value: Value = serde_json::from_str(&prompts)?;
    // 在prompts中替换$to$和$src_text$
    let prompts_list = prompts_value.as_array().ok_or("提示词列表格式有误")?;
    let mut new_prompts_list = Vec::new();
    // 先在新Prompts列表的开始加入System提示词: {"role":"system","content":system_prompt}
    new_prompts_list.push(json!({
        "role": "system",
        "content": system_prompt
    }));
    for prompt in prompts_list {
        let mut new_prompt = prompt.clone();
        if let Some(content) = new_prompt.get("content").and_then(|v| v.as_str()) {
            let new_content = content.replace("$to$", to).replace("$src_text$", text);
            new_prompt["content"] = json!(new_content);
        }
        new_prompts_list.push(new_prompt);
    }

    // 构造请求的payload
    let payload = json!({
        "model": model_string,  // 模型名
        "messages": new_prompts_list,  // 提示词列表
        "stream": false,  // Pot插件不支持stream模式，固定为false
        "temperature": temperature,  // 采样温度参数
        "top_p": top_p,  // 采样top_p参数
        "presence_penalty": presence_penalty,  // presence_penalty参数
        "frequency_penalty": frequency_penalty,  // frequency_penalty参数
        "max_output_tokens": 2048  // 不同模型和平台输出速度不一致，此处30s内若无返回则认为超时
    });

    // 发送请求并处理响应; 请求的url == request_url
    let client = reqwest::blocking::ClientBuilder::new()
        .timeout(std::time::Duration::from_secs(30))  // 设置请求超时时间
        .build()?;
    let response = client
        .post(request_url)
        .header(header::CONTENT_TYPE, "application/json")
        .header("Authorization", format!("Bearer {}", api_key))  // 设置请求头中的Authorization字段为Bearer api_key
        .json(&payload)
        .send()?;

    // 返回结果，如果请求失败则返回错误信息
    if response.status().is_success() {
        let result: Value = response.json()?;
        // 按照result["choices"][0]["message"]["content"]的路径返回翻译结果
        match result["choices"][0]["message"]["content"].as_str() {
            Some(result_text) => Ok(Value::String(result_text.to_string())),
            None => Err("响应中未找到翻译结果".into()),
        }
    } else {
        let error_msg = response.text().unwrap_or_else(|_| "请求失败".to_string());
        Err(format!("请求失败: {}", error_msg).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_request() {
        let mut needs = HashMap::new();
        needs.insert("api_key".to_string(), "your_api_key".to_string());
        needs.insert("request_url".to_string(), "https://cloud.infini-ai.com/maas/qwen1.5-32b-chat/nvidia/chat/completions".to_string());
        needs.insert("model_string".to_string(), "qwen1.5-32b-chat".to_string());
        needs.insert("temperature".to_string(), "0.7".to_string());
        let result = translate("本函数是一个测试函数，主要用于测试中文到英文的翻译接口是否正常工作", "auto", "English", "Chinese", needs).unwrap();
        println!("{result}");
    }
}
