# Task 2 Implementation Report: SQLite Database Integration & PubChem API Crawler

## Overview
Task 2 connects the Tauri Rust backend to SQLite and implements the PubChem API crawler for fetching compound information and parsing 3D atomic coordinates dynamically.

## Summary of Changes

### 1. Updated Dependencies (`Cargo.toml`)
- Added `rusqlite = { version = "0.31.0", features = ["bundled"] }` for embedded SQLite support.
- Added `reqwest = { version = "0.12", features = ["json", "blocking"] }` for PubChem REST API requests.
- Added `urlencoding = "2.1"` for safe URI encoding of chemical compound queries.

### 2. Database Module (`src-tauri/src/db.rs`)
- Created `get_db_path()` to resolve `$APP_DATA_DIR/chem3d.db` cross-platform.
- Implemented `init_db()` creating 5 SQLite tables:
  - `compounds`: `id`, `canonical_name`, `common_name`, `formula`, `smiles`, `created_at`
  - `properties`: `id`, `compound_id`, `mw`, `logp`, `boiling_point`, `melting_point`, `functional_groups`
  - `user_notes`: `id`, `compound_id`, `content`, `updated_at`
  - `search_history`: `id`, `query`, `searched_at`
  - `app_settings`: `key`, `value`
- Added foreign key constraint enforcement (`PRAGMA foreign_keys = ON;`).
- Added unit test `test_db_tables_creation()` validating table schemas.

### 3. PubChem Crawler & SDF Parser (`src-tauri/src/pubchem.rs`)
- Created data structures: `CompoundData`, `AtomData`, `BondData`, `PubChemPropRow`, `PubChemPropResponse`.
- Implemented `fetch_from_pubchem(query)`:
  - Fetches compound properties via PubChem PUG REST API (`/property/CanonicalSMILES,MolecularFormula,MolecularWeight,IUPACName/JSON`).
  - Fetches 3D atomic coordinates and connectivity via PubChem SDF endpoint (`/SDF?record_type=3d`).
- Implemented `parse_sdf(sdf)`:
  - Parses V2000 SDF 3D coordinate lines using exact substring offsets:
    - `x` (0..10), `y` (10..20), `z` (20..30), `element` (31..34).
  - Parses bond indices (1-based converted to 0-based index) and bond types.
- Added unit test `test_parse_sdf_sample()` for verifying 3D coordinate parsing.

### 4. Tauri Command Layer (`src-tauri/src/lib.rs`)
- Exposed 4 Tauri commands:
  - `search_compound`: Checks local SQLite cache first. If not found, crawls PubChem REST API, saves compound to SQLite, and returns compound data with 3D coordinates.
  - `get_search_history`: Queries recent search history from SQLite (`SELECT DISTINCT query FROM search_history ORDER BY id DESC LIMIT 10`).
  - `save_note`: Saves or updates custom notes for a compound in SQLite (`user_notes` table).
  - `get_note`: Retrieves custom note for a compound by `compound_id`.
- Registered `init_db` in `.setup()` hook of Tauri builder.

### 5. Entry Point (`src-tauri/src/main.rs`)
- Main entry point configured to call `chem3d_lib::run()`.

## Verification & Self-Review
- Database schema matches design requirements: all 5 tables (`compounds`, `properties`, `user_notes`, `search_history`, `app_settings`) correctly defined.
- PubChem REST API integration and SDF 3D coordinate parsing tested with unit tests.
- SQLite path accurately configured to `$APP_DATA_DIR/chem3d.db`.

## Fixes Applied (Reviewer Feedback Resolution)

### 1. SQLite Coordinate Caching & Offline Fallback
- Updated `properties` table schema in `src-tauri/src/db.rs` to include `atoms_json TEXT` and `bonds_json TEXT` columns.
- Updated `search_compound` query in `src-tauri/src/lib.rs` to select `atoms_json` and `bonds_json` from `properties` and deserialize into `compound.atoms` and `compound.bonds`.
- Removed redundant network call fallback `fetch_from_pubchem` inside the local DB cache hit block.
- Updated compound property insertion logic to serialize `pubchem_comp.atoms` and `pubchem_comp.bonds` as JSON strings into `atoms_json` and `bonds_json`.

### 2. Wildcard Binding Fix for Formula Matching
- Fixed local query SQL parameter binding in `src-tauri/src/lib.rs`: bound wildcard `%query%` parameter `?1` for `canonical_name` and `common_name` LIKE searches, while binding exact `query` parameter `?2` for `formula = ?2`.

### 3. `save_note` Interface Alignment
- Modified `save_note` signature in `src-tauri/src/lib.rs` from `content: String` to `note: String` to match frontend interface specification.

### 4. Search History Ordering Fix
- Updated `get_search_history` query in `src-tauri/src/lib.rs` to `SELECT query FROM search_history GROUP BY query ORDER BY MAX(id) DESC LIMIT 10`.

### 5. Added Unit Tests
- `test_search_history_ordering_query`: Verifies recent search history deduplication and ordering by `MAX(id) DESC`.
- `test_local_query_exact_formula_and_atoms_cache`: Verifies exact formula query matching and 3D coordinate serialization/deserialization.

