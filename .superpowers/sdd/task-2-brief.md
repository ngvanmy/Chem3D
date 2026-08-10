### Task 2: SQLite database integration & PubChem API crawler

**Files:**
- Create: `d:\Chem3D\src-tauri\src\db.rs`
- Create: `d:\Chem3D\src-tauri\src\pubchem.rs`
- Modify: `d:\Chem3D\src-tauri\Cargo.toml`, `d:\Chem3D\src-tauri\src\lib.rs`, `d:\Chem3D\src-tauri\src\main.rs`
- Test: Write a Rust integration test or use Tauri invoke command to check database initialization and PubChem scraping.

**Interfaces:**
- Consumes: `MoleculeViewer` (renders the crawled structures)
- Produces: Tauri Rust Commands:
  - `pub fn search_compound(query: String) -> Result<CompoundData, String>`
  - `pub fn get_search_history() -> Result<Vec<String>, String>`
  - `pub fn save_note(compound_id: i32, note: String) -> Result<(), String>`
  - `pub fn get_note(compound_id: i32) -> Result<String, String>`

- [ ] **Step 1: Update Cargo.toml dependencies**

Modify `d:\Chem3D\src-tauri\Cargo.toml` dependencies:
```toml
[dependencies]
tauri = { version = "2.0.0", features = [] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rusqlite = { version = "0.31.0", features = ["bundled"] }
reqwest = { version = "0.12", features = ["json", "blocking"] }
```

- [ ] **Step 2: Create DB module**

Create `d:\Chem3D\src-tauri\src\db.rs`:
```rust
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::PathBuf;

pub fn get_db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let mut path = app_handle.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    fs::create_dir_all(&path).ok();
    path.push("chem3d.db");
    path
}

pub fn init_db(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let path = get_db_path(app_handle);
    let conn = Connection::open(&path).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS compounds (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            canonical_name TEXT UNIQUE NOT NULL,
            common_name TEXT,
            formula TEXT,
            smiles TEXT UNIQUE NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS properties (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            compound_id INTEGER UNIQUE REFERENCES compounds(id) ON DELETE CASCADE,
            mw REAL,
            logp REAL,
            boiling_point REAL,
            melting_point REAL,
            functional_groups TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            compound_id INTEGER UNIQUE REFERENCES compounds(id) ON DELETE CASCADE,
            content TEXT,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS search_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            query TEXT NOT NULL,
            searched_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(())
}
```

- [ ] **Step 3: Create PubChem Crawler module**

Create `d:\Chem3D\src-tauri\src\pubchem.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AtomData {
    pub element: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BondData {
    pub source: usize,
    pub target: usize,
    pub r#type: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompoundData {
    pub id: i32,
    pub canonical_name: String,
    pub common_name: String,
    pub formula: String,
    pub smiles: String,
    pub mw: f64,
    pub logp: Option<f64>,
    pub boiling_point: Option<f64>,
    pub melting_point: Option<f64>,
    pub functional_groups: Vec<String>,
    pub atoms: Vec<AtomData>,
    pub bonds: Vec<BondData>,
}

#[derive(Deserialize)]
struct PubChemPropRow {
    #[serde(rename = "CID")]
    cid: i64,
    #[serde(rename = "CanonicalSMILES")]
    smiles: Option<String>,
    #[serde(rename = "MolecularFormula")]
    formula: Option<String>,
    #[serde(rename = "MolecularWeight")]
    mw: Option<String>,
    #[serde(rename = "IUPACName")]
    iupac: Option<String>,
}

#[derive(Deserialize)]
struct PubChemPropResponse {
    #[serde(rename = "PropertyTable")]
    property_table: PubChemPropTable,
}

#[derive(Deserialize)]
struct PubChemPropTable {
    #[serde(rename = "Properties")]
    properties: Vec<PubChemPropRow>,
}

pub fn fetch_from_pubchem(query: &str) -> Result<CompoundData, String> {
    let client = reqwest::blocking::Client::new();
    let prop_url = format!(
        "https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/name/{}/property/CanonicalSMILES,MolecularFormula,MolecularWeight,IUPACName/JSON",
        urlencoding::encode(query)
    );

    let res = client.get(&prop_url).send().map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("PubChem compound search failed with code {}", res.status()));
    }
    
    let prop_data: PubChemPropResponse = res.json().map_err(|e| e.to_string())?;
    let prop = prop_data.property_table.properties.first()
        .ok_or_else(|| "No properties found for compound".to_string())?;

    let cid = prop.cid;
    let smiles = prop.smiles.clone().unwrap_or_default();
    let formula = prop.formula.clone().unwrap_or_default();
    let name = prop.iupac.clone().unwrap_or_else(|| query.to_string());
    let mw = prop.mw.clone().unwrap_or_default().parse::<f64>().unwrap_or(0.0);

    // Fetch 3D coordinates in SDF format
    let sdf_url = format!(
        "https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/cid/{}/SDF?record_type=3d",
        cid
    );
    let sdf_res = client.get(&sdf_url).send().map_err(|e| e.to_string())?;
    let sdf_text = if sdf_res.status().is_success() {
        sdf_res.text().unwrap_or_default()
    } else {
        "".to_string()
    };

    let (atoms, bonds) = parse_sdf(&sdf_text);

    Ok(CompoundData {
        id: 0,
        canonical_name: name,
        common_name: query.to_string(),
        formula,
        smiles,
        mw,
        logp: None,
        boiling_point: None,
        melting_point: None,
        functional_groups: vec![],
        atoms,
        bonds,
    })
}

fn parse_sdf(sdf: &str) -> (Vec<AtomData>, Vec<BondData>) {
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();
    let lines: Vec<&str> = sdf.lines().collect();
    if lines.len() < 4 {
        return (atoms, bonds);
    }

    // Line 4 contains the count of atoms and bonds
    let count_line = lines[3];
    if count_line.len() < 6 {
        return (atoms, bonds);
    }
    let atom_count = count_line[0..3].trim().parse::<usize>().unwrap_or(0);
    let bond_count = count_line[3..6].trim().parse::<usize>().unwrap_or(0);

    let mut line_idx = 4;
    for _ in 0..atom_count {
        if line_idx >= lines.len() { break; }
        let line = lines[line_idx];
        if line.len() >= 34 {
            let x = line[0..10].trim().parse::<f64>().unwrap_or(0.0);
            let y = line[10..20].trim().parse::<f64>().unwrap_or(0.0);
            let z = line[20..30].trim().parse::<f64>().unwrap_or(0.0);
            let element = line[31..34].trim().to_string();
            atoms.push(AtomData { element, x, y, z });
        }
        line_idx += 1;
    }

    for _ in 0..bond_count {
        if line_idx >= lines.len() { break; }
        let line = lines[line_idx];
        if line.len() >= 9 {
            let source = line[0..3].trim().parse::<usize>().unwrap_or(1) - 1;
            let target = line[3..6].trim().parse::<usize>().unwrap_or(1) - 1;
            let r#type = line[6..9].trim().parse::<i32>().unwrap_or(1);
            bonds.push(BondData { source, target, r#type });
        }
        line_idx += 1;
    }

    (atoms, bonds)
}
```

- [ ] **Step 4: Connect commands in lib.rs**

Modify `d:\Chem3D\src-tauri\src\lib.rs`:
```rust
pub mod db;
pub mod pubchem;

use db::{get_db_path, init_db};
use pubchem::{fetch_from_pubchem, CompoundData, AtomData, BondData};
use rusqlite::{params, Connection};
use std::sync::Mutex;
use tauri::State;

pub struct DbState {
    pub conn_mutex: Mutex<Option<String>>, // Keep it simple and pass handle to query
}

#[tauri::command]
fn search_compound(app_handle: tauri::AppHandle, query: String) -> Result<CompoundData, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    // 1. Save history query
    conn.execute(
        "INSERT INTO search_history (query) VALUES (?)",
        params![query],
    ).ok();

    // 2. Query in local compounds DB first
    let mut stmt = conn.prepare(
        "SELECT c.id, c.canonical_name, c.common_name, c.formula, c.smiles, 
                p.mw, p.logp, p.boiling_point, p.melting_point, p.functional_groups
         FROM compounds c
         LEFT JOIN properties p ON c.id = p.compound_id
         WHERE c.canonical_name LIKE ?1 OR c.common_name LIKE ?1 OR c.formula = ?1"
    ).map_err(|e| e.to_string())?;

    let local_res = stmt.query_row(params![format!("%{}%", query)], |row| {
        let f_groups_json: String = row.get(9).unwrap_or_else(|_| "[]".to_string());
        let f_groups: Vec<String> = serde_json::from_str(&f_groups_json).unwrap_or_default();
        Ok(CompoundData {
            id: row.get(0)?,
            canonical_name: row.get(1)?,
            common_name: row.get(2)?,
            formula: row.get(3)?,
            smiles: row.get(4)?,
            mw: row.get(5)?,
            logp: row.get(6).ok(),
            boiling_point: row.get(7).ok(),
            melting_point: row.get(8).ok(),
            functional_groups: f_groups,
            atoms: vec![],
            bonds: vec![],
        })
    });

    if let Ok(mut compound) = local_res {
        // Fetch PubChem fallback for 3D coordinates if missing in local query response
        // Note: Coordinates are not saved to SQLite directly to save space, crawled dynamically if empty
        if let Ok(pubchem_comp) = fetch_from_pubchem(&compound.canonical_name) {
            compound.atoms = pubchem_comp.atoms;
            compound.bonds = pubchem_comp.bonds;
        }
        return Ok(compound);
    }

    // 3. Crawler Pubchem
    let mut pubchem_comp = fetch_from_pubchem(&query)?;

    // 4. Save to Database
    conn.execute(
        "INSERT OR IGNORE INTO compounds (canonical_name, common_name, formula, smiles) VALUES (?, ?, ?, ?)",
        params![pubchem_comp.canonical_name, pubchem_comp.common_name, pubchem_comp.formula, pubchem_comp.smiles],
    ).map_err(|e| e.to_string())?;

    let comp_id: i32 = conn.last_insert_rowid() as i32;
    pubchem_comp.id = comp_id;

    let f_groups_json = serde_json::to_string(&pubchem_comp.functional_groups).unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "INSERT OR IGNORE INTO properties (compound_id, mw, logp, boiling_point, melting_point, functional_groups) VALUES (?, ?, ?, ?, ?, ?)",
        params![comp_id, pubchem_comp.mw, pubchem_comp.logp, pubchem_comp.boiling_point, pubchem_comp.melting_point, f_groups_json],
    ).map_err(|e| e.to_string())?;

    Ok(pubchem_comp)
}

#[tauri::command]
fn get_search_history(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT DISTINCT query FROM search_history ORDER BY searched_at DESC LIMIT 10").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| row.get(0)).map_err(|e| e.to_string())?;
    let mut history = Vec::new();
    for r in rows {
        if let Ok(query) = r {
            history.push(query);
        }
    }
    Ok(history)
}

#[tauri::command]
fn save_note(app_handle: tauri::AppHandle, compound_id: i32, content: String) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO user_notes (compound_id, content, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)",
        params![compound_id, content],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_note(app_handle: tauri::AppHandle, compound_id: i32) -> Result<String, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT content FROM user_notes WHERE compound_id = ?").map_err(|e| e.to_string())?;
    let note: Result<String, rusqlite::Error> = stmt.query_row(params![compound_id], |row| row.get(0));
    match note {
        Ok(content) => Ok(content),
        Err(_) => Ok("".to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            init_db(app.handle()).expect("Failed to initialize database");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search_compound,
            get_search_history,
            save_note,
            get_note
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Modify `d:\Chem3D\src-tauri\src\main.rs` to point to `lib.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    chem3d_lib::run();
}
```

- [ ] **Step 5: Verify build & DB creation**

Run:
```powershell
npm run tauri dev
```
Check if database `chem3d.db` is created in `$APP_DATA_DIR`. Under Windows, this is usually at `C:\Users\ngvan\AppData\Roaming\com.tauri.dev\chem3d.db` or similar.

- [ ] **Step 6: Commit**

Run:
```powershell
git add src-tauri
git commit -m "feat: integrate SQLite db & PubChem Rest API crawler"
```

---

