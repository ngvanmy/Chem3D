# Extended Elements Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Thêm cấu hình thuộc tính cho hơn 30 nguyên tố hóa học chuẩn CPK vào giao diện hiển thị 3D và tối ưu bộ phân tích SMILES offline ở Rust backend.

**Architecture:** Tạo tệp constants chứa thông tin màu và bán kính của các nguyên tố ở frontend. Import vào MoleculeViewer. Cập nhật Rust backend parser để nhận dạng các nguyên tố hai ký tự và một ký tự phổ biến khi phân tích SMILES offline.

**Tech Stack:** React, TypeScript, Three.js, Rust (Tauri).

## Global Constraints

- Màu sắc chuẩn CPK của các nguyên tố phải được tuân thủ (ví dụ: H màu trắng, C màu xám, O màu đỏ, N xanh dương).
- Bán kính biểu diễn nguyên tử 3D phải tương quan cân đối (ví dụ: H nhỏ nhất, C/O vừa, Na/K/Ca lớn hơn).
- Bộ phân tích SMILES offline ở backend phải hoạt động chính xác và không bị lỗi crash khi gặp các nguyên tố mới.

---

### Task 1: Khởi tạo tệp hằng số nguyên tố hóa học ở Frontend

**Files:**
- Create: `src/utils/elements.ts`

**Interfaces:**
- Consumes: None
- Produces: `ELEMENT_COLORS` (Record<string, string>) và `ELEMENT_RADII` (Record<string, number>).

- [ ] **Step 1: Tạo và viết nội dung tệp `src/utils/elements.ts`**

Tạo mới file [elements.ts](file:///d:/Chem3D/src/utils/elements.ts) với nội dung sau:
```typescript
export const ELEMENT_COLORS: Record<string, string> = {
  H: '#ffffff',   // Trắng
  He: '#c8ffff',  // Cyan nhạt
  Li: '#cc80ff',  // Tím hoa cà
  Be: '#c2ff00',  // Xanh lá mạ
  B: '#ffb5b5',   // Hồng đào
  C: '#808080',   // Xám
  N: '#3050f8',   // Xanh dương
  O: '#ff0d0d',   // Đỏ
  F: '#90e050',   // Xanh lá nhạt
  Ne: '#b3e3f5',  // Xanh lam nhạt
  Na: '#ab5cf2',  // Tím đậm
  Mg: '#8ff000',  // Xanh lục đậm
  Al: '#bfa6a6',  // Bạc xám
  Si: '#f0c8a0',  // Vàng cát
  P: '#ff8000',   // Cam
  S: '#ffff30',   // Vàng
  Cl: '#1ff01f',  // Xanh lá sáng
  Ar: '#80d1e3',  // Cyan sáng
  K: '#8f40d4',   // Tím trung bình
  Ca: '#3dff00',  // Xanh lục sáng
  Sc: '#e6e6e6',  // Xám sáng
  Ti: '#bfc2c7',  // Bạc
  V: '#a6a6ab',   // Xám đậm
  Cr: '#8a99c7',  // Xanh xám
  Mn: '#9c7ac7',  // Tím xám
  Fe: '#e06633',  // Cam gạch (Sắt)
  Co: '#f090a0',  // Hồng đậm
  Ni: '#50d050',  // Lục nhạt
  Cu: '#c87d32',  // Đồng
  Zn: '#7d80b0',  // Xanh xám tối
  Ga: '#c28f8f',  // Hồng đất
  Ge: '#668f8f',  // Lục xám
  As: '#bd80e3',  // Tím nhạt
  Se: '#ffa100',  // Cam sáng
  Br: '#a62929',  // Đỏ sẫm
  Kr: '#5cb8d1',  // Xanh ngọc
  Rb: '#702eb0',  // Tím đậm
  Sr: '#00ff00',  // Xanh lục nguyên bản
  Ag: '#c0c0c0',  // Bạc sáng
  Sn: '#668080',  // Xám thiếc
  I: '#940094',   // Tím đậm (I-ốt)
  Au: '#gold',    // Vàng gold
};

export const ELEMENT_RADII: Record<string, number> = {
  H: 0.3,
  He: 0.35,
  Li: 0.75,
  Be: 0.5,
  B: 0.45,
  C: 0.6,
  N: 0.55,
  O: 0.5,
  F: 0.45,
  Ne: 0.4,
  Na: 0.9,
  Mg: 0.8,
  Al: 0.7,
  Si: 0.65,
  P: 0.65,
  S: 0.7,
  Cl: 0.65,
  Ar: 0.6,
  K: 1.1,
  Ca: 1.0,
  Fe: 0.8,
  Co: 0.75,
  Ni: 0.75,
  Cu: 0.75,
  Zn: 0.75,
  Br: 0.75,
  Ag: 0.8,
  I: 0.85,
  Au: 0.85,
};
```

- [ ] **Step 2: Commit Task 1**

```bash
git add src/utils/elements.ts
git commit -m "feat: add elements config constants with CPK colors and radii"
```

---

### Task 2: Cập nhật MoleculeViewer để sử dụng tệp hằng số mới

**Files:**
- Modify: `src/components/MoleculeViewer.tsx`

**Interfaces:**
- Consumes: `ELEMENT_COLORS` và `ELEMENT_RADII` từ `src/utils/elements.ts`.
- Produces: MoleculeViewer hiển thị các nguyên tố theo cấu hình mới, dùng fallback nếu không có cấu hình nguyên tố.

- [ ] **Step 1: Sửa đổi imports và logic hiển thị của `src/components/MoleculeViewer.tsx`**

Thay thế các dòng từ 24 đến 45 trong [MoleculeViewer.tsx](file:///d:/Chem3D/src/components/MoleculeViewer.tsx):
```typescript
import { ELEMENT_COLORS, ELEMENT_RADII } from '../utils/elements';
```

Và cập nhật fallback ở dòng 73-74 trong hàm hiển thị của `MoleculeViewer`:
```typescript
            const color = ELEMENT_COLORS[atom.element] || '#909090';
            const radius = ELEMENT_RADII[atom.element] || 0.5;
```

- [ ] **Step 2: Xác thực biên dịch Frontend**

Chạy kiểm tra dự án chạy bình thường và không có lỗi TypeScript hoặc Vite:
Đảm bảo terminal chạy dev server hiện tại không báo lỗi gì.

- [ ] **Step 3: Commit Task 2**

```bash
git add src/components/MoleculeViewer.tsx
git commit -m "refactor: integrate MoleculeViewer with the new shared elements config"
```

---

### Task 3: Cập nhật Offline SMILES Parser ở Rust Backend

**Files:**
- Modify: `src-tauri/src/lib.rs` (hàm `is_smiles`)
- Modify: `src-tauri/src/chemistry.rs` (hàm `generate_fallback_coordinates`)

**Interfaces:**
- Consumes: None
- Produces: Hàm `is_smiles` cho phép các ký tự nguyên tố mới, hàm `generate_fallback_coordinates` hỗ trợ bóc tách thêm các nguyên tố hai ký tự (`Na`, `Mg`, `Ca`, `Si`, `Fe`, `Cu`, `Zn`, `Li`) và một ký tự mới (`I`, `B`, `K`).

- [ ] **Step 1: Cập nhật hàm `is_smiles` trong `src-tauri/src/lib.rs`**

Cập nhật dòng 34 trong `src-tauri/src/lib.rs` để chấp nhận thêm ký tự cho các nguyên tố mới:
```rust
            'C' | 'c' | 'N' | 'n' | 'O' | 'o' | 'S' | 's' | 'P' | 'p' | 'F' | 'H' | 'I' | 'B' | 'K' | 'L' | 'i' | 'a' | 'M' | 'g' | 'e' | 'u' | 'Z' => {
```

- [ ] **Step 2: Cập nhật hàm `generate_fallback_coordinates` trong `src-tauri/src/chemistry.rs`**

Cập nhật từ dòng 83 đến 97 trong `src-tauri/src/chemistry.rs`:
```rust
        // Handle two-character elements like Cl, Br, Na, Mg, Ca, Si, Fe, Cu, Zn, Li
        let (elem_str, skip) = if c == 'C' && i + 1 < chars.len() && chars[i + 1] == 'l' {
            ("Cl", 1)
        } else if c == 'B' && i + 1 < chars.len() && chars[i + 1] == 'r' {
            ("Br", 1)
        } else if c == 'N' && i + 1 < chars.len() && chars[i + 1] == 'a' {
            ("Na", 1)
        } else if c == 'M' && i + 1 < chars.len() && chars[i + 1] == 'g' {
            ("Mg", 1)
        } else if c == 'C' && i + 1 < chars.len() && chars[i + 1] == 'a' {
            ("Ca", 1)
        } else if c == 'S' && i + 1 < chars.len() && chars[i + 1] == 'i' {
            ("Si", 1)
        } else if c == 'F' && i + 1 < chars.len() && chars[i + 1] == 'e' {
            ("Fe", 1)
        } else if c == 'C' && i + 1 < chars.len() && chars[i + 1] == 'u' {
            ("Cu", 1)
        } else if c == 'Z' && i + 1 < chars.len() && chars[i + 1] == 'n' {
            ("Zn", 1)
        } else if c == 'L' && i + 1 < chars.len() && chars[i + 1] == 'i' {
            ("Li", 1)
        } else {
            let element = match c {
                'C' | 'c' => Some("C"),
                'O' | 'o' => Some("O"),
                'N' | 'n' => Some("N"),
                'S' | 's' => Some("S"),
                'F' => Some("F"),
                'P' | 'p' => Some("P"),
                'H' => Some("H"),
                'I' => Some("I"),
                'B' | 'b' => Some("B"),
                'K' | 'k' => Some("K"),
                _ => None,
            };

            if let Some(el) = element {
                (el, 0)
            } else {
                i += 1;
                continue;
            }
        };
```

- [ ] **Step 3: Chạy cargo test để kiểm thử backend**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: Tất cả các bài kiểm tra đều vượt qua (PASS).

- [ ] **Step 4: Commit Task 3**

```bash
git add src-tauri/src/lib.rs src-tauri/src/chemistry.rs
git commit -m "feat: enhance backend SMILES parser and validation to support more chemical elements"
```
