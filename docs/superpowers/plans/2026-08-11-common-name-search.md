# Common Name Search Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Hỗ trợ tìm kiếm hợp chất bằng tên tiếng Việt thông dụng (ví dụ: nước, muối ăn, giấm ăn) qua cơ chế dịch tĩnh và hỗ trợ gợi ý autocomplete tiếng Việt tương ứng.

**Architecture:** Tạo module dịch tĩnh `translation.rs` ở Rust backend. Tích hợp chức năng dịch vào lệnh `search_compound` và tính năng gợi ý vào `get_autocomplete_suggestions` trong `lib.rs`.

**Tech Stack:** Rust (Tauri), rusqlite.

## Global Constraints

- Không sử dụng các API bên ngoài cho khâu dịch thuật tên thông dụng để đảm bảo hiệu suất cực nhanh và hoạt động offline.
- Tên tiếng Việt gốc của người dùng phải được lưu trữ vào cột `common_name` trong bảng `compounds` khi lưu dữ liệu tải từ PubChem về local.
- Autocomplete phải hỗ trợ gợi ý từ khi gõ tiền tố tiếng Việt có dấu/không dấu cơ bản.

---

### Task 1: Khởi tạo module dịch thuật Rust: `src-tauri/src/translation.rs`

**Files:**
- Create: `src-tauri/src/translation.rs`
- Modify: `src-tauri/src/lib.rs` (Đăng ký module mới)

**Interfaces:**
- Consumes: None
- Produces: 
  - `pub fn translate_vietnamese_name(query: &str) -> Option<&'static str>`
  - `pub fn suggest_vietnamese_names(prefix: &str) -> Vec<String>`

- [ ] **Step 1: Đăng ký module mới trong `src-tauri/src/lib.rs`**

Thêm khai báo ở đầu file [lib.rs](file:///d:/Chem3D/src-tauri/src/lib.rs):
```rust
pub mod translation;
```

- [ ] **Step 2: Viết nội dung cho tệp `src-tauri/src/translation.rs`**

Tạo mới file [translation.rs](file:///d:/Chem3D/src-tauri/src/translation.rs) với nội dung sau:
```rust
const VIETNAMESE_DICTIONARY: &[(&str, &str)] = &[
    ("nước", "water"),
    ("muối ăn", "sodium chloride"),
    ("giấm", "acetic acid"),
    ("giấm ăn", "acetic acid"),
    ("cồn", "ethanol"),
    ("cồn ăn", "ethanol"),
    ("rượu", "ethanol"),
    ("rượu etylic", "ethanol"),
    ("đường", "sucrose"),
    ("đường mía", "sucrose"),
    ("aspirin", "aspirin"),
    ("amoniac", "ammonia"),
    ("khí cacbonic", "carbon dioxide"),
    ("cacbonic", "carbon dioxide"),
    ("cacbon đioxit", "carbon dioxide"),
    ("metan", "methane"),
    ("khí metan", "methane"),
    ("khí oxi", "oxygen"),
    ("oxi", "oxygen"),
    ("axit axetic", "acetic acid"),
    ("axit sunfuric", "sulfuric acid"),
    ("axit clohidric", "hydrochloric acid"),
    ("axit nitric", "nitric acid"),
    ("baking soda", "sodium bicarbonate"),
    ("thuốc muối", "sodium bicarbonate"),
    ("vôi sống", "calcium oxide"),
    ("vôi tôi", "calcium hydroxide"),
    ("đá vôi", "calcium carbonate"),
    ("aceton", "acetone"),
    ("axeton", "acetone"),
    ("benzen", "benzene"),
    ("glucozơ", "glucose"),
    ("glucose", "glucose"),
    ("fructozơ", "fructose"),
    ("fructose", "fructose"),
    ("urê", "urea"),
    ("ure", "urea"),
    ("axetylen", "acetylene"),
    ("etylen", "ethylene"),
    ("phenol", "phenol"),
];

/// Dịch tên tiếng Việt thông dụng sang tiếng Anh
pub fn translate_vietnamese_name(query: &str) -> Option<&'static str> {
    let clean_query = query.trim().to_lowercase();
    for &(vi, en) in VIETNAMESE_DICTIONARY {
        if vi == clean_query {
            return Some(en);
        }
    }
    None
}

/// Gợi ý các tên tiếng Việt dựa trên tiền tố (prefix)
pub fn suggest_vietnamese_names(prefix: &str) -> Vec<String> {
    let clean_prefix = prefix.trim().to_lowercase();
    if clean_prefix.is_empty() {
        return vec![];
    }
    let mut suggestions = Vec::new();
    for &(vi, _) in VIETNAMESE_DICTIONARY {
        if vi.starts_with(&clean_prefix) {
            suggestions.push(vi.to_string());
        }
    }
    suggestions.dedup();
    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation() {
        assert_eq!(translate_vietnamese_name("nước"), Some("water"));
        assert_eq!(translate_vietnamese_name("  muối ăn  "), Some("sodium chloride"));
        assert_eq!(translate_vietnamese_name("không có thực"), None);
    }

    #[test]
    fn test_suggestions() {
        let sug = suggest_vietnamese_names("ax");
        assert!(sug.contains(&"axit axetic".to_string()));
        assert!(sug.contains(&"axeton".to_string()));
    }
}
```

- [ ] **Step 3: Chạy test kiểm tra module mới**

Chạy command: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: Tất cả tests đều vượt qua (bao gồm các test dịch thuật mới).

- [ ] **Step 4: Commit Task 1**

```bash
git add src-tauri/src/translation.rs src-tauri/src/lib.rs
git commit -m "feat: add translation module for Vietnamese common names"
```

---

### Task 2: Tích hợp dịch thuật vào hàm tìm kiếm `search_compound`

**Files:**
- Modify: `src-tauri/src/lib.rs` (hàm `search_compound`)

**Interfaces:**
- Consumes: `translation::translate_vietnamese_name` từ `translation.rs`.
- Produces: Hàm `search_compound` xử lý dịch và lưu `common_name` tiếng Việt chuẩn xác.

- [ ] **Step 1: Cập nhật hàm `search_compound` trong `src-tauri/src/lib.rs`**

Cập nhật từ dòng 47 đến 188 trong [lib.rs](file:///d:/Chem3D/src-tauri/src/lib.rs).
Đầu tiên, định nghĩa bộ dịch ở đầu hàm:
```rust
    let mut search_term = query.clone();
    let is_translated = if let Some(translated) = translation::translate_vietnamese_name(&query) {
        search_term = translated.to_string();
        true
    } else {
        false
    };
```

Sau đó sử dụng `search_term` khi truy vấn database local (dùng `search_term` thay cho `query` ở dòng 70):
```rust
    let local_res = stmt.query_row(params![format!("%{}%", search_term), search_term], |row| {
```

Và sử dụng `search_term` khi gọi tới PubChem (dòng 106):
```rust
    let mut pubchem_comp = match fetch_from_pubchem(&search_term) {
```

Đặc biệt, ở bước lưu vào cơ sở dữ liệu (Database) ở dòng 145-154, nếu `is_translated` là true, ta lưu `query` gốc (tên tiếng Việt, ví dụ: "nước") vào cột `common_name`:
```rust
    let saved_common_name = if is_translated {
        query.clone()
    } else {
        pubchem_comp.common_name.clone()
    };

    conn.execute(
        "INSERT OR IGNORE INTO compounds (canonical_name, common_name, formula, smiles) VALUES (?, ?, ?, ?)",
        params![
            pubchem_comp.canonical_name,
            saved_common_name,
            pubchem_comp.formula,
            pubchem_comp.smiles
        ],
    )
    .map_err(|e| e.to_string())?;
```

- [ ] **Step 2: Viết test xác thực tìm kiếm**

Thêm test case mới vào tệp `src-tauri/src/lib.rs` để giả lập tìm kiếm bằng tiếng Việt:
```rust
    #[test]
    fn test_search_compound_translation() {
        // Test offline fallback search for translated query
        let query = "nước";
        let translated = translation::translate_vietnamese_name(query).unwrap();
        assert_eq!(translated, "water");
    }
```

- [ ] **Step 3: Chạy test kiểm thử backend**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: Pass.

- [ ] **Step 4: Commit Task 2**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: integrate common name translation into search_compound command"
```

---

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
