pub mod db;
pub mod pubchem;
pub mod chemistry;
pub mod ai;

use db::{get_db_path, init_db};
use pubchem::{fetch_from_pubchem, CompoundData};
use rusqlite::{params, Connection};

/// Helper function to check if a query string contains only valid SMILES characters
pub fn is_smiles(query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return false;
    }
    let chars: Vec<char> = trimmed.chars().collect();
    let mut i = 0;
    let mut has_atom = false;

    while i < chars.len() {
        let c = chars[i];
        if c == 'C' && i + 1 < chars.len() && chars[i + 1] == 'l' {
            has_atom = true;
            i += 2;
            continue;
        }
        if c == 'B' && i + 1 < chars.len() && chars[i + 1] == 'r' {
            has_atom = true;
            i += 2;
            continue;
        }

        match c {
            'C' | 'c' | 'N' | 'n' | 'O' | 'o' | 'S' | 's' | 'P' | 'p' | 'F' | 'H' | 'I' | 'B' => {
                has_atom = true;
            }
            '=' | '#' | '(' | ')' | '[' | ']' | '-' | '+' | '@' | '/' | '\\' | '.' | '%' | '0'..='9' => {}
            _ => return false,
        }
        i += 1;
    }

    has_atom
}

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
                    p.mw, p.logp, p.boiling_point, p.melting_point, p.functional_groups,
                    p.atoms_json, p.bonds_json
             FROM compounds c
             LEFT JOIN properties p ON c.id = p.compound_id
             WHERE c.canonical_name LIKE ?1 OR c.common_name LIKE ?1 OR c.formula = ?2",
        )
        .map_err(|e| e.to_string())?;

    let local_res = stmt.query_row(params![format!("%{}%", query), query], |row| {
        let f_groups_json: String = row.get(9).unwrap_or_else(|_| "[]".to_string());
        let f_groups: Vec<String> = serde_json::from_str(&f_groups_json).unwrap_or_default();
        let atoms_json: String = row.get(10).unwrap_or_else(|_| "[]".to_string());
        let atoms: Vec<pubchem::AtomData> = serde_json::from_str(&atoms_json).unwrap_or_default();
        let bonds_json: String = row.get(11).unwrap_or_else(|_| "[]".to_string());
        let bonds: Vec<pubchem::BondData> = serde_json::from_str(&bonds_json).unwrap_or_default();
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
            atoms,
            bonds,
        })
    });

    if let Ok(mut compound) = local_res {
        if compound.functional_groups.is_empty() {
            compound.functional_groups = chemistry::detect_functional_groups(&compound.smiles);
        }
        if compound.atoms.is_empty() {
            let (atoms, bonds) = chemistry::generate_fallback_coordinates(&compound.smiles);
            compound.atoms = atoms;
            compound.bonds = bonds;
        }
        return Ok(compound);
    }

    // 3. Crawler PubChem or offline fallback
    let mut pubchem_comp = match fetch_from_pubchem(&query) {
        Ok(comp) => comp,
        Err(_err) => {
            if is_smiles(&query) {
                let (atoms, bonds) = chemistry::generate_fallback_coordinates(&query);
                CompoundData {
                    id: 0,
                    canonical_name: query.clone(),
                    common_name: query.clone(),
                    formula: "".to_string(),
                    smiles: query.clone(),
                    mw: 0.0,
                    logp: None,
                    boiling_point: None,
                    melting_point: None,
                    functional_groups: vec![],
                    atoms,
                    bonds,
                }
            } else {
                return Err(format!(
                    "Offline: Could not find compound '{}' locally and cannot search PubChem.",
                    query
                ));
            }
        }
    };

    // Ensure functional groups and atoms/bonds are populated
    if pubchem_comp.functional_groups.is_empty() {
        pubchem_comp.functional_groups = chemistry::detect_functional_groups(&pubchem_comp.smiles);
    }
    if pubchem_comp.atoms.is_empty() {
        let (atoms, bonds) = chemistry::generate_fallback_coordinates(&pubchem_comp.smiles);
        pubchem_comp.atoms = atoms;
        pubchem_comp.bonds = bonds;
    }

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
    let atoms_json = serde_json::to_string(&pubchem_comp.atoms)
        .unwrap_or_else(|_| "[]".to_string());
    let bonds_json = serde_json::to_string(&pubchem_comp.bonds)
        .unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "INSERT OR REPLACE INTO properties (compound_id, mw, logp, boiling_point, melting_point, functional_groups, atoms_json, bonds_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            comp_id,
            pubchem_comp.mw,
            pubchem_comp.logp,
            pubchem_comp.boiling_point,
            pubchem_comp.melting_point,
            f_groups_json,
            atoms_json,
            bonds_json
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
        .prepare("SELECT query FROM search_history GROUP BY query ORDER BY MAX(id) DESC LIMIT 10")
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
fn save_note(app_handle: tauri::AppHandle, compound_id: i32, note: String) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO user_notes (compound_id, content, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)",
        params![compound_id, note],
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

#[tauri::command]
fn save_setting(app_handle: tauri::AppHandle, key: String, value: String) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?, ?)",
        params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_setting(app_handle: tauri::AppHandle, key: String) -> Result<String, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT value FROM app_settings WHERE key = ?")
        .map_err(|e| e.to_string())?;
    let val: Result<String, rusqlite::Error> = stmt.query_row(params![key], |row| row.get(0));
    match val {
        Ok(v) => Ok(v),
        Err(_) => Ok("".to_string()),
    }
}

#[tauri::command]
fn test_local_connection(endpoint: String) -> Result<Vec<String>, String> {
    ai::test_connection(&endpoint)
}

#[tauri::command]
fn query_ai(
    app_handle: tauri::AppHandle,
    provider: String,
    prompt: String,
    model: String,
    context: String,
) -> Result<String, String> {
    let db_path = get_db_path(&app_handle);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    match provider.as_str() {
        "gemini" => {
            let mut stmt = conn
                .prepare("SELECT value FROM app_settings WHERE key = 'gemini_api_key'")
                .map_err(|e| e.to_string())?;
            let key: String = stmt
                .query_row([], |row| row.get(0))
                .map_err(|_| "Gemini API Key missing in settings.".to_string())?;
            ai::call_gemini(&key, &prompt, &context)
        }
        "openai" => {
            let mut stmt = conn
                .prepare("SELECT value FROM app_settings WHERE key = 'openai_api_key'")
                .map_err(|e| e.to_string())?;
            let key: String = stmt
                .query_row([], |row| row.get(0))
                .map_err(|_| "OpenAI API Key missing in settings.".to_string())?;
            ai::call_openai_compatible(
                "https://api.openai.com/v1",
                Some(&key),
                &model,
                &prompt,
                &context,
            )
        }
        "local" => {
            let mut stmt_url = conn
                .prepare("SELECT value FROM app_settings WHERE key = 'local_ai_url'")
                .map_err(|e| e.to_string())?;
            let raw_url: String = stmt_url
                .query_row([], |row| row.get(0))
                .unwrap_or_else(|_| "http://localhost:11434/v1".to_string());
            let url = if raw_url.trim().is_empty() {
                "http://localhost:11434/v1".to_string()
            } else {
                raw_url
            };
            ai::call_openai_compatible(&url, None, &model, &prompt, &context)
        }
        _ => Err("Invalid AI Provider".to_string()),
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
            get_note,
            save_setting,
            get_setting,
            test_local_connection,
            query_ai
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_history_ordering_query() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE search_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                query TEXT NOT NULL,
                searched_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .unwrap();

        conn.execute("INSERT INTO search_history (query) VALUES ('H2O')", []).unwrap();
        conn.execute("INSERT INTO search_history (query) VALUES ('CO2')", []).unwrap();
        conn.execute("INSERT INTO search_history (query) VALUES ('H2O')", []).unwrap();

        let mut stmt = conn
            .prepare("SELECT query FROM search_history GROUP BY query ORDER BY MAX(id) DESC LIMIT 10")
            .unwrap();
        let history: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(history, vec!["H2O", "CO2"]);
    }

    #[test]
    fn test_local_query_exact_formula_and_atoms_cache() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE compounds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                canonical_name TEXT UNIQUE NOT NULL,
                common_name TEXT,
                formula TEXT,
                smiles TEXT UNIQUE NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE properties (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                compound_id INTEGER UNIQUE REFERENCES compounds(id) ON DELETE CASCADE,
                mw REAL,
                logp REAL,
                boiling_point REAL,
                melting_point REAL,
                functional_groups TEXT,
                atoms_json TEXT,
                bonds_json TEXT
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO compounds (canonical_name, common_name, formula, smiles) VALUES ('Water', 'water', 'H2O', 'O')",
            [],
        )
        .unwrap();
        let atoms = vec![pubchem::AtomData { element: "O".to_string(), x: 0.0, y: 0.0, z: 0.0 }];
        let bonds = vec![pubchem::BondData { source: 0, target: 1, r#type: 1 }];
        let atoms_json = serde_json::to_string(&atoms).unwrap();
        let bonds_json = serde_json::to_string(&bonds).unwrap();
        conn.execute(
            "INSERT INTO properties (compound_id, mw, atoms_json, bonds_json) VALUES (1, 18.015, ?1, ?2)",
            params![atoms_json, bonds_json],
        )
        .unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT c.id, c.canonical_name, c.common_name, c.formula, c.smiles, 
                        p.mw, p.logp, p.boiling_point, p.melting_point, p.functional_groups,
                        p.atoms_json, p.bonds_json
                 FROM compounds c
                 LEFT JOIN properties p ON c.id = p.compound_id
                 WHERE c.canonical_name LIKE ?1 OR c.common_name LIKE ?1 OR c.formula = ?2",
            )
            .unwrap();

        // Query by exact formula 'H2O'
        let query = "H2O";
        let res: CompoundData = stmt.query_row(params![format!("%{}%", query), query], |row| {
            let f_groups_json: String = row.get(9).unwrap_or_else(|_| "[]".to_string());
            let f_groups: Vec<String> = serde_json::from_str(&f_groups_json).unwrap_or_default();
            let atoms_str: String = row.get(10).unwrap_or_else(|_| "[]".to_string());
            let atoms_parsed: Vec<pubchem::AtomData> = serde_json::from_str(&atoms_str).unwrap_or_default();
            let bonds_str: String = row.get(11).unwrap_or_else(|_| "[]".to_string());
            let bonds_parsed: Vec<pubchem::BondData> = serde_json::from_str(&bonds_str).unwrap_or_default();
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
                atoms: atoms_parsed,
                bonds: bonds_parsed,
            })
        }).unwrap();

        assert_eq!(res.canonical_name, "Water");
        assert_eq!(res.atoms.len(), 1);
        assert_eq!(res.atoms[0].element, "O");
        assert_eq!(res.bonds.len(), 1);

        // Query by partial formula 'H2' should fail to match formula 'H2O'
        let query_partial = "H2";
        let res_partial = stmt.query_row(params![format!("%{}%", query_partial), query_partial], |_| Ok(()));
        assert!(res_partial.is_err());
    }

    #[test]
    fn test_is_smiles_distinguishes_names_and_smiles() {
        assert!(is_smiles("CCO"));
        assert!(is_smiles("CC(=O)O"));
        assert!(is_smiles("c1ccccc1"));
        assert!(is_smiles("ClCCBr"));
        assert!(is_smiles("C#N"));
        assert!(!is_smiles("Water"));
        assert!(!is_smiles("Ethanol"));
        assert!(!is_smiles("Aspirin"));
        assert!(!is_smiles(""));
    }
}

