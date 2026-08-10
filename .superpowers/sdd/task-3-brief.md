### Task 3: Local SMILES parser & Offline 3D coordinate generator

**Files:**
- Create: `d:\Chem3D\src-tauri\src\chemistry.rs`
- Modify: `d:\Chem3D\src-tauri\src\lib.rs`, `d:\Chem3D\src-tauri\src\pubchem.rs`
- Test: Unit tests in `chemistry.rs` to verify SMILES parser matching.

**Interfaces:**
- Consumes: `pubchem::CompoundData`
- Produces: `pub fn detect_functional_groups(smiles: &str) -> Vec<String>` inside Rust backend and integration with `search_compound` when PubChem coordinate extraction fails.

- [ ] **Step 1: Create Rust SMILES pattern matcher**

Create `d:\Chem3D\src-tauri\src\chemistry.rs`:
```rust
use crate::pubchem::{AtomData, BondData};

// Identify functional groups using SMILES patterns
pub fn detect_functional_groups(smiles: &str) -> Vec<String> {
    let mut groups = Vec::new();
    let smiles_upper = smiles.to_uppercase();

    // 1. Carboxylic Acid (C(=O)O or COOH)
    if smiles_upper.contains("C(=O)O") || smiles_upper.contains("COOH") {
        groups.push("Carboxylic Acid".to_string());
    }

    // 2. Alcohol (C-OH)
    if smiles_upper.contains("O") {
        // Simple check: oxygen connected to carbon but not double-bonded in C(=O)
        let is_alcohol = smiles_upper.contains("CO") || smiles_upper.contains("OC") || smiles_upper.contains("CCO");
        let is_carbonyl = smiles_upper.contains("C(=O)");
        if is_alcohol && (!is_carbonyl || smiles_upper.matches("O").count() > 1) {
            groups.push("Alcohol".to_string());
        }
    }

    // 3. Aldehyde / Ketone (Carbonyl C=O)
    if smiles_upper.contains("C(=O)") {
        if smiles_upper.contains("C(=O)H") || smiles_upper.ends_with("=O") {
            groups.push("Aldehyde".to_string());
        } else {
            groups.push("Ketone/Carbonyl".to_string());
        }
    }

    // 4. Benzene ring / Aromatic
    if smiles.contains("c1ccccc1") || smiles_upper.contains("C1=CC=CC=C1") {
        groups.push("Benzene ring (Aromatic)".to_string());
    }

    // 5. Alkene (Double bonds between carbons)
    if smiles.contains("C=C") {
        groups.push("Alkene".to_string());
    }

    // 6. Alkyne (Triple bonds between carbons)
    if smiles.contains("C#C") {
        groups.push("Alkyne".to_string());
    }

    if groups.is_empty() {
        groups.push("Alkane".to_string());
    }

    groups
}

// Basic coordinate generator for offline fallback
pub fn generate_fallback_coordinates(smiles: &str) -> (Vec<AtomData>, Vec<BondData>) {
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();

    // Fallback: Just place atoms along a line with alternating offset coordinates
    // Parse SMILES characters and create nodes
    let chars: Vec<char> = smiles.chars().collect();
    let mut i = 0;
    let mut atom_idx = 0;

    while i < chars.len() {
        let c = chars[i];
        let element = match c {
            'C' | 'c' => Some("C"),
            'O' | 'o' => Some("O"),
            'N' | 'n' => Some("N"),
            'S' | 's' => Some("S"),
            'F' => Some("F"),
            'H' => Some("H"),
            _ => None,
        };

        if let Some(el) = element {
            let offset_y = if atom_idx % 2 == 0 { 0.5 } else { -0.5 };
            atoms.push(AtomData {
                element: el.to_string(),
                x: (atom_idx as f64) * 1.5,
                y: offset_y,
                z: 0.0,
            });

            if atom_idx > 0 {
                bonds.push(BondData {
                    source: atom_idx - 1,
                    target: atom_idx,
                    r#type: 1,
                });
            }
            atom_idx += 1;
        }
        i += 1;
    }

    (atoms, bonds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alcohol_detection() {
        let smiles = "CCO"; // Ethanol
        let groups = detect_functional_groups(smiles);
        assert!(groups.contains(&"Alcohol".to_string()));
    }

    #[test]
    fn test_carboxylic_detection() {
        let smiles = "CC(=O)O"; // Acetic Acid
        let groups = detect_functional_groups(smiles);
        assert!(groups.contains(&"Carboxylic Acid".to_string()));
    }
}
```

- [ ] **Step 2: Update Search Compound command to use SMILES module**

Modify `d:\Chem3D\src-tauri\src\lib.rs`:
1. Add `pub mod chemistry;` at the top.
2. In `search_compound`:
   - Compute functional groups: `pubchem_comp.functional_groups = chemistry::detect_functional_groups(&pubchem_comp.smiles);`
   - If `pubchem_comp.atoms` is empty, generate coordinates offline:
     ```rust
     if pubchem_comp.atoms.is_empty() {
         let (atoms, bonds) = chemistry::generate_fallback_coordinates(&pubchem_comp.smiles);
         pubchem_comp.atoms = atoms;
         pubchem_comp.bonds = bonds;
     }
     ```

- [ ] **Step 3: Run Rust cargo tests**

Run:
```powershell
cd src-tauri
cargo test
```
Expected: Tests pass.

- [ ] **Step 4: Commit**

Run:
```powershell
git add src-tauri
git commit -m "feat: add local SMILES functional group parser and offline 3D coordinate fallback"
```

---

