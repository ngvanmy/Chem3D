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

