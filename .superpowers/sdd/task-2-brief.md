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

