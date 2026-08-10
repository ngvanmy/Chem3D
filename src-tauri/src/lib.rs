pub mod db;
pub mod pubchem;

use db::{get_db_path, init_db};
use pubchem::{fetch_from_pubchem, CompoundData};
use rusqlite::{params, Connection};

#[tauri::command]
fn search_compound(app_handle: tauri::AppHandle, query: String) -> Result<CompoundData, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    // 1. Save history query
    conn.execute(
        "INSERT INTO search_history (query) VALUES (?)",
        params![query],
    )
    .ok();

    // 2. Query in local compounds DB first
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.canonical_name, c.common_name, c.formula, c.smiles, 
                    p.mw, p.logp, p.boiling_point, p.melting_point, p.functional_groups
             FROM compounds c
             LEFT JOIN properties p ON c.id = p.compound_id
             WHERE c.canonical_name LIKE ?1 OR c.common_name LIKE ?1 OR c.formula = ?1",
        )
        .map_err(|e| e.to_string())?;

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
        if let Ok(pubchem_comp) = fetch_from_pubchem(&compound.canonical_name) {
            compound.atoms = pubchem_comp.atoms;
            compound.bonds = pubchem_comp.bonds;
        }
        return Ok(compound);
    }

    // 3. Crawler PubChem
    let mut pubchem_comp = fetch_from_pubchem(&query)?;

    // 4. Save to Database
    conn.execute(
        "INSERT OR IGNORE INTO compounds (canonical_name, common_name, formula, smiles) VALUES (?, ?, ?, ?)",
        params![
            pubchem_comp.canonical_name,
            pubchem_comp.common_name,
            pubchem_comp.formula,
            pubchem_comp.smiles
        ],
    )
    .map_err(|e| e.to_string())?;

    let comp_id: i32 = conn
        .query_row(
            "SELECT id FROM compounds WHERE canonical_name = ?1 OR smiles = ?2",
            params![pubchem_comp.canonical_name, pubchem_comp.smiles],
            |row| row.get(0),
        )
        .unwrap_or(0);
    pubchem_comp.id = comp_id;

    let f_groups_json = serde_json::to_string(&pubchem_comp.functional_groups)
        .unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "INSERT OR REPLACE INTO properties (compound_id, mw, logp, boiling_point, melting_point, functional_groups) VALUES (?, ?, ?, ?, ?, ?)",
        params![
            comp_id,
            pubchem_comp.mw,
            pubchem_comp.logp,
            pubchem_comp.boiling_point,
            pubchem_comp.melting_point,
            f_groups_json
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(pubchem_comp)
}

#[tauri::command]
fn get_search_history(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT DISTINCT query FROM search_history ORDER BY id DESC LIMIT 10")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?;
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
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_note(app_handle: tauri::AppHandle, compound_id: i32) -> Result<String, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT content FROM user_notes WHERE compound_id = ?")
        .map_err(|e| e.to_string())?;
    let note: Result<String, rusqlite::Error> =
        stmt.query_row(params![compound_id], |row| row.get(0));
    match note {
        Ok(content) => Ok(content),
        Err(_) => Ok("".to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
