# Task 3 Implementation Report: Local SMILES parser & Offline 3D coordinate generator

## Summary of Work
1. **Module Creation (`src-tauri/src/chemistry.rs`)**:
   - Implemented `detect_functional_groups(smiles: &str)` supporting pattern matching for Alcohol, Carboxylic Acid, Aldehyde/Ketone, Ether, Benzene Ring (Aromatic), Alkene, Alkyne, and Alkane fallback.
   - Implemented `generate_fallback_coordinates(smiles: &str)` to parse SMILES elements (C, O, N, S, F, P, H, Cl, Br) and place them with alternating Y offsets (`0.5` / `-0.5`) and linear X increments (`1.5` distance) so that 3D viewers can render basic structures offline.

2. **Integration (`src-tauri/src/lib.rs`)**:
   - Module `chemistry` exported.
   - Updated `search_compound`: automatically calculates functional groups and populates fallback 3D coordinates when PubChem data is unavailable or offline.

3. **Testing**:
   - Added unit tests in `chemistry.rs` covering:
     - `test_alcohol_detection`
     - `test_carboxylic_detection`
     - `test_benzene_detection`
     - `test_alkene_and_alkyne_detection`
     - `test_fallback_coordinates_generation`
   - Verified that all 9 unit tests in `src-tauri` pass cleanly.

## Verification & Test Results
- Command: `cargo test`
- Output: 9 passed; 0 failed; 0 ignored.
- All functional group pattern detections and offline coordinate generators operate accurately as expected.

## Fix Report: Offline Compound Name vs. SMILES Distinction

### Changes Made
1. **Implemented `is_smiles(query: &str) -> bool`** in `src-tauri/src/lib.rs`:
   - Validates that the query contains valid SMILES characters (`C, c, N, n, O, o, S, s, P, p, F, H, I, B, Cl, Br, =, #, (, ), [, ], 1-9, -, +`).
   - Ensures the string contains at least one valid element token.
2. **Updated `search_compound` Offline Fallback Logic**:
   - Checks `is_smiles(&query)` when PubChem fetch fails.
   - If true, generates fallback 3D coordinates from SMILES.
   - If false (e.g. name queries like "Water", "Ethanol"), returns explicit error: `format!("Offline: Could not find compound '{}' locally and cannot search PubChem.", query)`.
3. **Added Unit Tests**:
   - `test_is_smiles_distinguishes_names_and_smiles` in `src-tauri/src/lib.rs`.

### Verification Results
- Command: `cargo test` (in `src-tauri`)
- Output: 10 passed; 0 failed; 0 ignored.

