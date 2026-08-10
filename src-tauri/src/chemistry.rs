use crate::pubchem::{AtomData, BondData};

/// Identify functional groups from SMILES string using pattern matching
pub fn detect_functional_groups(smiles: &str) -> Vec<String> {
    let mut groups = Vec::new();
    let smiles_upper = smiles.to_uppercase();

    // 1. Carboxylic Acid (C(=O)O or COOH)
    if smiles_upper.contains("C(=O)O") || smiles_upper.contains("COOH") || smiles_upper.contains("C(=O)[OH]") {
        groups.push("Carboxylic Acid".to_string());
    }

    // 2. Aldehyde / Ketone (Carbonyl C=O)
    let is_carboxylic = groups.contains(&"Carboxylic Acid".to_string());
    if smiles_upper.contains("C(=O)") || smiles_upper.contains("C=O") {
        if !is_carboxylic {
            if smiles_upper.contains("C(=O)H") || smiles_upper.contains("CH=O") || smiles_upper.ends_with("=O") {
                groups.push("Aldehyde".to_string());
            } else {
                groups.push("Ketone/Carbonyl".to_string());
            }
        }
    }

    // 3. Alcohol (Hydroxyl group -OH connected to aliphatic/aromatic carbon)
    if smiles_upper.contains("O") {
        let is_alcohol = smiles_upper.contains("CO") || smiles_upper.contains("OC") || smiles_upper.contains("OH");
        let carbonyl_count = smiles_upper.matches("C(=O)").count() + smiles_upper.matches("C=O").count();
        let oxygen_count = smiles_upper.matches("O").count();

        if is_alcohol && (!is_carboxylic || oxygen_count > carbonyl_count) {
            if !groups.contains(&"Alcohol".to_string()) && !smiles_upper.contains("COC") {
                groups.push("Alcohol".to_string());
            } else if smiles_upper.contains("OH") && !groups.contains(&"Alcohol".to_string()) {
                groups.push("Alcohol".to_string());
            }
        }
    }

    // 4. Ether (C-O-C)
    if smiles_upper.contains("COC") || smiles_upper.contains("CCOCC") {
        if !groups.contains(&"Ether".to_string()) {
            groups.push("Ether".to_string());
        }
    }

    // 5. Benzene ring (Aromatic)
    if smiles.contains("c1ccccc1") || smiles_upper.contains("C1=CC=CC=C1") {
        groups.push("Benzene ring (Aromatic)".to_string());
    }

    // 6. Alkene (Double bond between carbons)
    if smiles.contains("C=C") || smiles.contains("c=c") {
        groups.push("Alkene".to_string());
    }

    // 7. Alkyne (Triple bond between carbons)
    if smiles.contains("C#C") || smiles.contains("c#c") {
        groups.push("Alkyne".to_string());
    }

    // Default to Alkane if no major functional group detected
    if groups.is_empty() {
        groups.push("Alkane".to_string());
    }

    groups
}

/// Basic 3D coordinate generator for offline fallback
pub fn generate_fallback_coordinates(smiles: &str) -> (Vec<AtomData>, Vec<BondData>) {
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();

    let chars: Vec<char> = smiles.chars().collect();
    let mut i = 0;
    let mut atom_idx = 0;

    while i < chars.len() {
        let c = chars[i];

        // Handle two-character elements like Cl, Br
        let (elem_str, skip) = if c == 'C' && i + 1 < chars.len() && chars[i + 1] == 'l' {
            ("Cl", 1)
        } else if c == 'B' && i + 1 < chars.len() && chars[i + 1] == 'r' {
            ("Br", 1)
        } else {
            let element = match c {
                'C' | 'c' => Some("C"),
                'O' | 'o' => Some("O"),
                'N' | 'n' => Some("N"),
                'S' | 's' => Some("S"),
                'F' => Some("F"),
                'P' | 'p' => Some("P"),
                'H' => Some("H"),
                _ => None,
            };

            if let Some(el) = element {
                (el, 0)
            } else {
                i += 1;
                continue;
            }
        };

        let offset_y = if atom_idx % 2 == 0 { 0.5 } else { -0.5 };
        atoms.push(AtomData {
            element: elem_str.to_string(),
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
        i += 1 + skip;
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

    #[test]
    fn test_benzene_detection() {
        let smiles = "c1ccccc1"; // Benzene
        let groups = detect_functional_groups(smiles);
        assert!(groups.contains(&"Benzene ring (Aromatic)".to_string()));

        let smiles_kelvin = "C1=CC=CC=C1";
        let groups_kelvin = detect_functional_groups(smiles_kelvin);
        assert!(groups_kelvin.contains(&"Benzene ring (Aromatic)".to_string()));
    }

    #[test]
    fn test_alkene_and_alkyne_detection() {
        let alkene = "C=C";
        assert!(detect_functional_groups(alkene).contains(&"Alkene".to_string()));

        let alkyne = "C#C";
        assert!(detect_functional_groups(alkyne).contains(&"Alkyne".to_string()));

        let alkane = "CC";
        assert!(detect_functional_groups(alkane).contains(&"Alkane".to_string()));
    }

    #[test]
    fn test_fallback_coordinates_generation() {
        let smiles = "CCO";
        let (atoms, bonds) = generate_fallback_coordinates(smiles);
        assert_eq!(atoms.len(), 3);
        assert_eq!(atoms[0].element, "C");
        assert_eq!(atoms[1].element, "C");
        assert_eq!(atoms[2].element, "O");

        // Verify zig-zag y positions
        assert_eq!(atoms[0].y, 0.5);
        assert_eq!(atoms[1].y, -0.5);
        assert_eq!(atoms[2].y, 0.5);

        // Verify linear x positions
        assert_eq!(atoms[0].x, 0.0);
        assert_eq!(atoms[1].x, 1.5);
        assert_eq!(atoms[2].x, 3.0);

        // Verify bonds
        assert_eq!(bonds.len(), 2);
        assert_eq!(bonds[0].source, 0);
        assert_eq!(bonds[0].target, 1);
        assert_eq!(bonds[1].source, 1);
        assert_eq!(bonds[1].target, 2);
    }
}
