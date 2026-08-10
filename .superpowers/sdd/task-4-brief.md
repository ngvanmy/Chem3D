### Task 4: Hybrid AI client integration

**Files:**
- Create: `d:\Chem3D\src-tauri\src\ai.rs`
- Modify: `d:\Chem3D\src-tauri\src\lib.rs`
- Test: Manual verification with curl/reqwest to local mock server or Ollama local service.

**Interfaces:**
- Consumes: `app_settings` SQLite keys
- Produces: Tauri Rust Commands:
  - `pub fn test_local_connection(endpoint: String) -> Result<Vec<String>, String>`
  - `pub fn query_ai(provider: String, prompt: String, model: String, context: String) -> Result<String, String>`
  - `pub fn save_setting(key: String, value: String) -> Result<(), String>`
  - `pub fn get_setting(key: String) -> Result<String, String>`

- [ ] **Step 1: Implement AI module**

Create `d:\Chem3D\src-tauri\src\ai.rs`:
```rust
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
struct LocalModelsResponse {
    data: Vec<LocalModelItem>,
}

#[derive(Deserialize)]
struct LocalModelItem {
    id: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageResponse,
}

#[derive(Deserialize)]
struct OpenAIMessageResponse {
    content: String,
}

pub fn test_connection(endpoint: &str) -> Result<Vec<String>, String> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/models", endpoint.trim_end_matches('/'));
    
    let res = client.get(&url).send().map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("Server returned error status {}", res.status()));
    }

    let model_data: LocalModelsResponse = res.json().map_err(|e| e.to_string())?;
    let model_names = model_data.data.into_iter().map(|m| m.id).collect();
    Ok(model_names)
}

pub fn call_gemini(api_key: &str, prompt: &str, context: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let full_prompt = format!("Context: {}\n\nQuestion: {}", context, prompt);

    let body = json!({
        "contents": [{
            "parts": [{
                "text": full_prompt
            }]
        }]
    });

    let res = client.post(&url)
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Gemini API error: {}", res.status()));
    }

    let resp: GeminiResponse = res.json().map_err(|e| e.to_string())?;
    let text = resp.candidates.first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.clone())
        .ok_or_else(|| "Empty response from Gemini".to_string())?;

    Ok(text)
}

pub fn call_openai_compatible(endpoint: &str, api_key: Option<&str>, model: &str, prompt: &str, context: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
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
    let text = resp.choices.first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "Empty message choice from AI model".to_string())?;

    Ok(text)
}
```

- [ ] **Step 2: Add AI and Settings Commands to lib.rs**

Modify `d:\Chem3D\src-tauri\src\lib.rs`:
1. Add `pub mod ai;` at the top.
2. Implement commands:
```rust
#[tauri::command]
fn save_setting(app_handle: tauri::AppHandle, key: String, value: String) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?, ?)",
        params![key, value],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_setting(app_handle: tauri::AppHandle, key: String) -> Result<String, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = ?").map_err(|e| e.to_string())?;
    let val: Result<String, rusqlite::Error> = stmt.query_row(params![key], |row| row.get(0));
    match val {
        Ok(v) => Ok(v),
        Err(_) => Ok("".to_string()),
    }
}

#[tauri::command]
fn test_local_connection(endpoint: String) -> Result<Vec<String>, String> {
    ai::test_connection(&endpoint)
}

#[tauri::command]
fn query_ai(app_handle: tauri::AppHandle, provider: String, prompt: String, model: String, context: String) -> Result<String, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    
    match provider.as_str() {
        "gemini" => {
            let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = 'gemini_api_key'").map_err(|e| e.to_string())?;
            let key: String = stmt.query_row([], |row| row.get(0)).map_err(|_| "Gemini API Key missing in settings.".to_string())?;
            ai::call_gemini(&key, &prompt, &context)
        }
        "openai" => {
            let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = 'openai_api_key'").map_err(|e| e.to_string())?;
            let key: String = stmt.query_row([], |row| row.get(0)).map_err(|_| "OpenAI API Key missing in settings.".to_string())?;
            ai::call_openai_compatible("https://api.openai.com/v1", Some(&key), &model, &prompt, &context)
        }
        "local" => {
            let mut stmt_url = conn.prepare("SELECT value FROM app_settings WHERE key = 'local_ai_url'").map_err(|e| e.to_string())?;
            let url: String = stmt_url.query_row([], |row| row.get(0)).unwrap_or_else(|_| "http://localhost:11434/v1".to_string());
            ai::call_openai_compatible(&url, None, &model, &prompt, &context)
        }
        _ => Err("Invalid AI Provider".to_string())
    }
}
```
3. Register new commands in `run()`:
```rust
        .invoke_handler(tauri::generate_handler![
            search_compound,
            get_search_history,
            save_note,
            get_note,
            save_setting,
            get_setting,
            test_local_connection,
            query_ai
        ])
```

- [ ] **Step 3: Verify build**

Run:
```powershell
npm run tauri dev
```
Ensure compile completes without errors.

- [ ] **Step 4: Commit**

Run:
```powershell
git add src-tauri
git commit -m "feat: integrate Cloud AI and Ollama/LM Studio client proxies"
```

---

