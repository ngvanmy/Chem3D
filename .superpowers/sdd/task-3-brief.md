### Task 3: Tích hợp gợi ý từ tiền tố tiếng Việt vào Autocomplete Suggestions

**Files:**
- Modify: `src-tauri/src/lib.rs` (hàm `get_autocomplete_suggestions`)

**Interfaces:**
- Consumes: `translation::suggest_vietnamese_names` từ `translation.rs`.
- Produces: Hàm `get_autocomplete_suggestions` trả về danh sách kết hợp gợi ý tiếng Việt và PubChem/Local.

- [ ] **Step 1: Cập nhật hàm `get_autocomplete_suggestions` trong `src-tauri/src/lib.rs`**

Cập nhật hàm `get_autocomplete_suggestions` (dòng 350-386) để chèn gợi ý tiếng Việt vào đầu:
```rust
#[tauri::command]
fn get_autocomplete_suggestions(app_handle: tauri::AppHandle, query: String) -> Result<Vec<String>, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }

    let mut suggestions = translation::suggest_vietnamese_names(trimmed);

    // 1. Attempt PubChem online first
    if let Ok(online_sug) = pubchem::get_autocomplete_from_pubchem(trimmed) {
        for s in online_sug {
            if !suggestions.contains(&s) {
                suggestions.push(s);
            }
        }
    }

    // 2. Local SQLite fallback
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT canonical_name FROM compounds 
             WHERE canonical_name LIKE ?1 OR common_name LIKE ?1 OR formula = ?2
             LIMIT 6",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![format!("%{}%", trimmed), trimmed], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    for r in rows {
        if let Ok(name) = r {
            if !suggestions.contains(&name) {
                suggestions.push(name);
            }
        }
    }

    suggestions.truncate(6);
    Ok(suggestions)
}
```

- [ ] **Step 2: Chạy test suite**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS.

- [ ] **Step 3: Commit Task 3**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: integrate Vietnamese prefix matching in autocomplete suggestions"
```
