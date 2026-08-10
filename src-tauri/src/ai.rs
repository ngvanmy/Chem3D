use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize, Debug)]
pub struct LocalModelsResponse {
    pub data: Vec<LocalModelItem>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LocalModelItem {
    pub id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GeminiCandidate {
    pub content: GeminiContent,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GeminiContent {
    pub parts: Vec<GeminiPart>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GeminiPart {
    pub text: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OpenAIResponse {
    pub choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OpenAIChoice {
    pub message: OpenAIMessageResponse,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OpenAIMessageResponse {
    pub content: String,
}

pub fn test_connection(endpoint: &str) -> Result<Vec<String>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let url = format!("{}/models", endpoint.trim_end_matches('/'));

    let res = client.get(&url).send().map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("Server returned error status {}", res.status()));
    }

    let model_data: LocalModelsResponse = res.json().map_err(|e| e.to_string())?;
    let model_names = model_data.data.into_iter().map(|m| m.id).collect();
    Ok(model_names)
}

pub fn call_gemini(
    api_key: &str,
    model: &str,
    prompt: &str,
    context: &str,
) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let effective_model = if model.trim().is_empty() {
        "gemini-2.5-flash"
    } else {
        model.trim()
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        effective_model, api_key
    );

    let full_prompt = format!("Context: {}\n\nQuestion: {}", context, prompt);

    let body = json!({
        "contents": [{
            "parts": [{
                "text": full_prompt
            }]
        }]
    });

    let res = client
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Gemini API error: {}", res.status()));
    }

    let resp: GeminiResponse = res.json().map_err(|e| e.to_string())?;
    let text = resp
        .candidates
        .first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.clone())
        .ok_or_else(|| "Empty response from Gemini".to_string())?;

    Ok(text)
}

pub fn call_openai_compatible(
    endpoint: &str,
    api_key: Option<&str>,
    model: &str,
    prompt: &str,
    context: &str,
) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| e.to_string())?;
    let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));

    let full_prompt = format!("Context: {}\n\nQuestion: {}", context, prompt);

    let body = json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "You are a professional 3D Chemistry Learning Assistant. Answer clearly and concisely."
            },
            {
                "role": "user",
                "content": full_prompt
            }
        ]
    });

    let mut req = client.post(&url).json(&body);
    if let Some(key) = api_key {
        if !key.is_empty() {
            req = req.bearer_auth(key);
        }
    }

    let res = req.send().map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("API error: {}", res.status()));
    }

    let resp: OpenAIResponse = res.json().map_err(|e| e.to_string())?;
    let text = resp
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "Empty message choice from AI model".to_string())?;

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_response_deserialization() {
        let json_str = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {
                                "text": "Water molecule formula is H2O."
                            }
                        ]
                    }
                }
            ]
        }"#;

        let resp: GeminiResponse = serde_json::from_str(json_str).unwrap();
        let text = resp
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone());
        assert_eq!(text, Some("Water molecule formula is H2O.".to_string()));
    }

    #[test]
    fn test_openai_response_deserialization() {
        let json_str = r#"{
            "choices": [
                {
                    "message": {
                        "content": "Ethanol is an alcohol with formula C2H5OH."
                    }
                }
            ]
        }"#;

        let resp: OpenAIResponse = serde_json::from_str(json_str).unwrap();
        let text = resp
            .choices
            .first()
            .map(|c| c.message.content.clone());
        assert_eq!(
            text,
            Some("Ethanol is an alcohol with formula C2H5OH.".to_string())
        );
    }

    #[test]
    fn test_local_models_deserialization() {
        let json_str = r#"{
            "data": [
                { "id": "llama3:latest" },
                { "id": "mistral:7b" }
            ]
        }"#;

        let resp: LocalModelsResponse = serde_json::from_str(json_str).unwrap();
        let ids: Vec<String> = resp.data.into_iter().map(|m| m.id).collect();
        assert_eq!(ids, vec!["llama3:latest", "mistral:7b"]);
    }
}
