use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AtomData {
    pub element: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
    mw: Option<serde_json::Value>,
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
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Chem3D/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let prop_url = format!(
        "https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/name/{}/property/CanonicalSMILES,MolecularFormula,MolecularWeight,IUPACName/JSON",
        urlencoding::encode(query)
    );

    let res = client.get(&prop_url).send().map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("PubChem compound search failed with code {}", res.status()));
    }

    let prop_data: PubChemPropResponse = res.json().map_err(|e| e.to_string())?;
    let prop = prop_data
        .property_table
        .properties
        .first()
        .ok_or_else(|| "No properties found for compound".to_string())?;

    let cid = prop.cid;
    let smiles = prop.smiles.clone().unwrap_or_default();
    let formula = prop.formula.clone().unwrap_or_default();
    let name = prop.iupac.clone().unwrap_or_else(|| query.to_string());

    let mw = match &prop.mw {
        Some(serde_json::Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(serde_json::Value::String(s)) => s.parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    };

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

pub fn parse_sdf(sdf: &str) -> (Vec<AtomData>, Vec<BondData>) {
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
        if line_idx >= lines.len() {
            break;
        }
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
        if line_idx >= lines.len() {
            break;
        }
        let line = lines[line_idx];
        if line.len() >= 9 {
            let source = line[0..3].trim().parse::<usize>().unwrap_or(1).saturating_sub(1);
            let target = line[3..6].trim().parse::<usize>().unwrap_or(1).saturating_sub(1);
            let r#type = line[6..9].trim().parse::<i32>().unwrap_or(1);
            bonds.push(BondData { source, target, r#type });
        }
        line_idx += 1;
    }

    (atoms, bonds)
}

#[derive(Deserialize)]
pub struct PubChemAutocompleteResponse {
    #[serde(rename = "dictionary_terms")]
    pub dictionary_terms: Option<PubChemDictTerms>,
}

#[derive(Deserialize)]
pub struct PubChemDictTerms {
    pub compound: Option<Vec<String>>,
}

pub fn get_autocomplete_from_pubchem(query: &str) -> Result<Vec<String>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!(
        "https://pubchem.ncbi.nlm.nih.gov/rest/autocomplete/compound/{}/json?limit=6",
        urlencoding::encode(query)
    );

    let res = client.get(&url).send().map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("PubChem autocomplete API error: {}", res.status()));
    }

    let autocomplete_data: PubChemAutocompleteResponse = res.json().map_err(|e| e.to_string())?;
    let terms = autocomplete_data.dictionary_terms
        .and_then(|dt| dt.compound)
        .unwrap_or_default();

    Ok(terms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocomplete_deserialization() {
        let mock_json = r#"
        {
            "dictionary_terms": {
                "compound": ["water", "water-d2", "water-18o"]
            }
        }
        "#;
        let autocomplete_data: PubChemAutocompleteResponse = serde_json::from_str(mock_json).unwrap();
        let terms = autocomplete_data.dictionary_terms.unwrap().compound.unwrap();
        assert_eq!(terms.len(), 3);
        assert_eq!(terms[0], "water");
    }


    #[test]
    fn test_parse_sdf_sample() {
        let sample_sdf = "\
2244
  -OEChem-08102615383D

 3 2  0  0  0  0  0  0  0  0999 V2000
    0.0000    0.0000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
    0.9600    0.0000    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
   -0.2400    0.9200    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
  1  2  1  0  0  0  0
  1  3  1  0  0  0  0
M  END
$$$$";
        let (atoms, bonds) = parse_sdf(sample_sdf);
        assert_eq!(atoms.len(), 3);
        assert_eq!(atoms[0].element, "O");
        assert_eq!(atoms[1].element, "H");
        assert_eq!(atoms[2].element, "H");
        assert_eq!(atoms[1].x, 0.96);

        assert_eq!(bonds.len(), 2);
        assert_eq!(bonds[0].source, 0);
        assert_eq!(bonds[0].target, 1);
        assert_eq!(bonds[0].r#type, 1);
    }
}
