# Thiết kế: Bổ sung nguyên tố hóa học vào Mô hình 3D (CPK Styling & Offline SMILES Support)

Tài liệu này đặc tả thiết kế kỹ thuật để bổ sung các nguyên tố hóa học phổ biến khác vào hệ thống hiển thị 3D và bộ tạo cấu trúc phân tử offline.

---

## 1. Mục tiêu
- **Màu sắc & Kích thước chuẩn CPK**: Hiển thị chính xác màu sắc và bán kính biểu diễn cho nhiều loại nguyên tố (Na, K, Ca, Mg, Fe, Cu, Zn, B, Si, v.v.) khi tải phân tử từ PubChem hoặc từ dữ liệu cục bộ.
- **Tính năng offline nhất quán**: Hỗ trợ bộ sinh tọa độ offline nhận diện chính xác các nguyên tố mới này từ chuỗi SMILES.

---

## 2. Thiết kế chi tiết các thay đổi

### A. Tệp cấu hình nguyên tố mới
Tạo tệp mới `src/utils/elements.ts` để lưu trữ dữ liệu CPK của các nguyên tố:
- `ELEMENT_COLORS`: Ánh xạ ký hiệu nguyên tố sang mã màu HEX.
- `ELEMENT_RADII`: Ánh xạ ký hiệu nguyên tố sang bán kính 3D (được thiết kế tỉ lệ trực quan đẹp mắt).

### B. Cập nhật MoleculeViewer.tsx
- Thay thế các khai báo cục bộ `ELEMENT_COLORS` và `ELEMENT_RADII` bằng cách import từ `src/utils/elements.ts`.
- Sử dụng màu dự phòng xám `#909090` và bán kính dự phòng `0.5` cho các nguyên tố chưa được định nghĩa.

### C. Cải tiến Offline SMILES Parser ở Backend (Rust)
- **Cập nhật hàm `is_smiles`** trong `src-tauri/src/lib.rs` để cho phép các ký tự nguyên tố mới như `Na`, `Ca`, `Mg`, `Fe`, `Cu`, `Zn`, `Li`, `B`, `Si`, `I`, `K`.
- **Cập nhật hàm `generate_fallback_coordinates`** trong `src-tauri/src/chemistry.rs`:
  - Thêm xử lý phân tích các nguyên tố hai ký tự: `Na`, `Mg`, `Ca`, `Si`, `Fe`, `Cu`, `Zn`, `Li`.
  - Thêm xử lý phân tích các nguyên tố một ký tự mới: `I`, `B`, `K`.

---

## 3. Kế hoạch xác thực (Verification Plan)

### Kiểm thử tự động (Automated Tests)
- Chạy các test case của backend Rust bằng lệnh:
  ```bash
  cargo test --manifest-path src-tauri/Cargo.toml
  ```

### Kiểm thử thủ công (Manual Verification)
- Khởi chạy ứng dụng bằng `npm run tauri dev`.
- Tìm kiếm các hợp chất đặc trưng chứa các nguyên tố mới:
  - **Sodium Chloride (NaCl)**: Kiểm tra hiển thị nguyên tử Na (màu tím, lớn hơn) và Cl (màu xanh lá).
  - **Calcium Carbonate (CaCO3)**: Kiểm tra hiển thị nguyên tử Ca (màu xanh lục đậm).
  - **Ferrocene** hoặc các phức chất chứa **Fe/Cu**: Kiểm tra hiển thị nguyên tử Fe/Cu (màu cam-nâu/đồng).
  - **Silane (SiH4)** hoặc **Boron Trifluoride (BF3)**: Kiểm tra hiển thị nguyên tử Si/B.
