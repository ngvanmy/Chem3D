# Thiết kế: Tìm kiếm hợp chất theo Tên thông dụng tiếng Việt

Tài liệu này đặc tả thiết kế kỹ thuật để hỗ trợ tìm kiếm hợp chất hóa học bằng các tên thông dụng bằng tiếng Việt (ví dụ: nước, muối ăn, giấm ăn, cồn, baking soda...) thông qua cơ chế dịch tĩnh sang tiếng Anh trước khi truy vấn và hỗ trợ gợi ý autocomplete.

---

## 1. Mục tiêu
- **Dịch tìm kiếm**: Khi người dùng tìm "nước", "muối ăn", v.v., hệ thống tự động nhận diện và gửi truy vấn tương ứng ("water", "sodium chloride") tới local DB và PubChem.
- **Tối ưu hóa Database**: Lưu trữ tên tiếng Việt gốc vào trường `common_name` của bảng `compounds` để lần tìm kiếm sau có thể trực tiếp so khớp offline.
- **Gợi ý thông minh (Autocomplete)**: Khi người dùng gõ cụm từ tiếng Việt (ví dụ: "nướ", "muố"), danh sách gợi ý sẽ đề xuất các tên thông dụng tiếng Việt tương ứng.

---

## 2. Thiết kế chi tiết các thay đổi

### A. Mô-đun dịch thuật mới: `src-tauri/src/translation.rs`
Tạo tệp mới định nghĩa bản đồ đối chiếu tĩnh:
- Hàm `translate_vietnamese_name(query: &str) -> Option<&'static str>`: Trả về tên tiếng Anh chuẩn của hợp chất.
- Hàm `suggest_vietnamese_names(prefix: &str) -> Vec<String>`: Tìm tất cả các tên tiếng Việt khớp tiền tố (prefix) phục vụ cho tính năng gợi ý từ động.

### B. Cập nhật `search_compound` trong `src-tauri/src/lib.rs`
- Dịch truy vấn trước khi tìm kiếm:
  ```rust
  let mut search_term = query.clone();
  if let Some(translated) = translation::translate_vietnamese_name(&query) {
      search_term = translated.to_string();
  }
  ```
- Khi lưu hợp chất mới tải về từ PubChem, lưu `query` tiếng Việt gốc làm `common_name` trong bảng `compounds`.

### C. Cập nhật `get_autocomplete_suggestions` trong `src-tauri/src/lib.rs`
- Kiểm tra nếu `query` khớp tiền tố với các tên tiếng Việt thông dụng, ưu tiên chèn các đề xuất tiếng Việt này vào đầu danh sách gợi ý.

---

## 3. Kế hoạch xác thực (Verification Plan)

### Kiểm thử tự động (Automated Tests)
- Thêm unit test mới trong `src-tauri/src/translation.rs` để kiểm tra khả năng dịch và gợi ý từ tiền tố.
- Chạy: `cargo test --manifest-path src-tauri/Cargo.toml`

### Kiểm thử thủ công (Manual Verification)
- Chạy `npm run tauri dev`.
- Tìm kiếm từ khóa:
  - **nước**: Hệ thống phải tìm ra nước (Water, H2O).
  - **muối ăn**: Hệ thống phải tìm ra muối ăn (Sodium Chloride, NaCl).
  - **giấm ăn**: Hệ thống phải tìm ra giấm ăn (Acetic Acid).
- Thử gõ vào ô tìm kiếm:
  - **muố**: Kiểm tra xem "muối ăn" có hiện lên trong danh sách gợi ý tự động (autocomplete suggestions) không.
