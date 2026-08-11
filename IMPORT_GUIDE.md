# Hướng dẫn Nhập (Import) và Cấu hình Chem3D-Tauri trên máy tính mới

Tài liệu này hướng dẫn bạn cách thiết lập và mở dự án **Chem3D-Tauri** trong môi trường phát hành AI-first **Google Antigravity (AGY)** trên một thiết bị mới.

---

## 1. Chuẩn bị Môi trường Máy tính mới

Trước khi bắt đầu, hãy đảm bảo máy tính mới đã cài đặt các công cụ phát triển cơ bản:
1.  **Node.js** (Phiên bản LTS mới nhất).
2.  **Rust & Cargo**:
    *   Cài đặt thông qua [rustup.rs](https://rustup.rs/).
    *   Đảm bảo Rust Toolchain ở phiên bản `stable`.
3.  **Tauri CLI** (Cục bộ):
    *   Chạy `npm install -g @tauri-apps/cli` hoặc sử dụng thông qua `npx tauri`.

---

## 2. Sao chép và Nhập mã nguồn vào Antigravity

### Bước 1: Sao chép thư mục dự án
Sao chép toàn bộ thư mục `Chem3D` hoặc clone từ kho mã nguồn Git của bạn sang máy tính mới:
```bash
git clone <URL_KHO_MA_NGUON_CUA_BAN>
cd Chem3D
```

### Bước 2: Cài đặt thư viện Frontend
Tại thư mục gốc của dự án, cài đặt các gói dependencies cần thiết:
```bash
npm install
```

### Bước 3: Mở trong Antigravity CLI / IDE
Mở thư mục `Chem3D` bằng Antigravity để AI tự động nhận diện cấu trúc dự án:
*   **Antigravity CLI (`agy`)**:
    Di chuyển Terminal vào thư mục dự án và khởi chạy CLI:
    ```bash
    agy
    ```
*   **Antigravity IDE**:
    Mở Antigravity IDE và chọn **Open Folder** trỏ đến thư mục `Chem3D`.

*Antigravity sẽ tự động quét thư mục và nạp ngữ cảnh hóa lý, cơ sở dữ liệu SQLite, và sơ đồ Three.js của dự án.*

---

## 3. Cấu hình Cơ sở dữ liệu và AI Settings

Khi chạy ứng dụng lần đầu tiên trên thiết bị mới:
1.  **Cơ sở dữ liệu SQLite**: 
    Ứng dụng sẽ tự động khởi tạo file DB cục bộ tại thư mục dữ liệu ứng dụng của hệ điều hành:
    *   **Windows**: `C:\Users\<USERNAME>\AppData\Roaming\com.chem3d.app\chem3d.db`
    *   *Bạn không cần tự tạo file hay bảng vì Rust backend đã tự động thực hiện di cư dữ liệu (`migration`) lúc khởi động.*
2.  **Thiết lập AI Keys**:
    *   Khởi động ứng dụng bằng lệnh:
        ```bash
        npm run tauri dev
        ```
    *   Nhấp vào nút **Cài đặt** (Settings - biểu tượng bánh răng) ở góc trên bên phải.
    *   Nhập API Key cho các dịch vụ đám mây (Gemini API Key hoặc OpenAI API Key) hoặc cấu hình URL và Model của AI cục bộ (Ollama / LM Studio).
    *   Bấm **Lưu** để ghi nhớ cấu hình vào SQLite DB.

---

## 4. Cấu hình Đóng gói & Auto-Update trên máy mới

Do chữ ký bảo mật yêu cầu cặp khóa mã hóa đồng bộ, khi chuyển sang máy khác, bạn cần tạo lại hoặc import cặp khóa ký số:

### Cách 1: Tạo khóa mới trên thiết bị mới
1.  Chạy lệnh để sinh cặp khóa mới:
    ```bash
    npx tauri signer generate
    ```
2.  Lấy **Khóa công khai (Public Key)** mới tạo và ghi đè vào thuộc tính `"pubkey"` trong file [`src-tauri/tauri.conf.json`](file:///src-tauri/tauri.conf.json).
3.  Cập nhật **Khóa riêng tư (Private Key)** mới vào phần **GitHub Secrets** của repository với tên `TAURI_SIGNING_PRIVATE_KEY` để luồng CI/CD chạy bình thường.

### Cách 2: Chia sẻ khóa cũ (Nếu muốn giữ khóa cũ)
*   Sao chép chuỗi khóa riêng tư từ máy cũ và thiết lập vào biến môi trường `TAURI_SIGNING_PRIVATE_KEY` trên máy mới hoặc lưu trực tiếp vào GitHub Secrets để biên dịch tự động.

---

## 5. Khắc phục lỗi Biên dịch thường gặp

### Lỗi biên dịch `ring` trên máy Windows ARM64 (Snapdragon X, Virtual Machine Apple Silicon)
Nếu máy tính mới chạy hệ điều hành Windows ARM64, khi chạy build bạn sẽ gặp lỗi thiếu Clang:
```text
error occurred in cc-rs: failed to find tool "clang"
```
**Cách sửa**:
1.  Mở **Visual Studio Installer** trên máy mới.
2.  Nhấp vào **Modify** phiên bản Visual Studio hiện tại.
3.  Tích chọn các mục phụ trong phần phát triển C++:
    *   **C++ Clang Compiler for Windows**
    *   **C++ Clang-cl for v14x build tools**
    *   **MSVC v14x - VS 2022 C++ ARM64/ARM64EC build tools**
4.  Khởi động lại Terminal và chạy lại lệnh.
