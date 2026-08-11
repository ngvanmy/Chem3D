const VIETNAMESE_DICTIONARY: &[(&str, &str)] = &[
    ("nước", "water"),
    ("muối ăn", "sodium chloride"),
    ("giấm", "acetic acid"),
    ("giấm ăn", "acetic acid"),
    ("cồn", "ethanol"),
    ("cồn ăn", "ethanol"),
    ("rượu", "ethanol"),
    ("rượu etylic", "ethanol"),
    ("đường", "sucrose"),
    ("đường mía", "sucrose"),
    ("aspirin", "aspirin"),
    ("amoniac", "ammonia"),
    ("khí cacbonic", "carbon dioxide"),
    ("cacbonic", "carbon dioxide"),
    ("cacbon đioxit", "carbon dioxide"),
    ("metan", "methane"),
    ("khí metan", "methane"),
    ("khí oxi", "oxygen"),
    ("oxi", "oxygen"),
    ("axit axetic", "acetic acid"),
    ("axit sunfuric", "sulfuric acid"),
    ("axit clohidric", "hydrochloric acid"),
    ("axit nitric", "nitric acid"),
    ("baking soda", "sodium bicarbonate"),
    ("thuốc muối", "sodium bicarbonate"),
    ("vôi sống", "calcium oxide"),
    ("vôi tôi", "calcium hydroxide"),
    ("đá vôi", "calcium carbonate"),
    ("aceton", "acetone"),
    ("axeton", "acetone"),
    ("benzen", "benzene"),
    ("glucozơ", "glucose"),
    ("glucose", "glucose"),
    ("fructozơ", "fructose"),
    ("fructose", "fructose"),
    ("urê", "urea"),
    ("ure", "urea"),
    ("axetylen", "acetylene"),
    ("etylen", "ethylene"),
    ("phenol", "phenol"),
];

/// Dịch tên tiếng Việt thông dụng sang tiếng Anh
pub fn translate_vietnamese_name(query: &str) -> Option<&'static str> {
    let clean_query = query.trim().to_lowercase();
    for &(vi, en) in VIETNAMESE_DICTIONARY {
        if vi == clean_query {
            return Some(en);
        }
    }
    None
}

/// Gợi ý các tên tiếng Việt dựa trên tiền tố (prefix)
pub fn suggest_vietnamese_names(prefix: &str) -> Vec<String> {
    let clean_prefix = prefix.trim().to_lowercase();
    if clean_prefix.is_empty() {
        return vec![];
    }
    let mut suggestions = Vec::new();
    for &(vi, _) in VIETNAMESE_DICTIONARY {
        if vi.starts_with(&clean_prefix) {
            suggestions.push(vi.to_string());
        }
    }
    suggestions.dedup();
    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation() {
        assert_eq!(translate_vietnamese_name("nước"), Some("water"));
        assert_eq!(translate_vietnamese_name("  muối ăn  "), Some("sodium chloride"));
        assert_eq!(translate_vietnamese_name("không có thực"), None);
    }

    #[test]
    fn test_suggestions() {
        let sug = suggest_vietnamese_names("ax");
        assert!(sug.contains(&"axit axetic".to_string()));
        assert!(sug.contains(&"axeton".to_string()));
    }
}
