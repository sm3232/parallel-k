#[derive(Clone, Debug, Eq, PartialOrd, Ord)]
pub struct QuasiIdentifier {
    pub column_name: std::string::String,
    pub incognito_colname: std::string::String,
    pub index: usize,
    pub qi_type: QIType
}

impl std::hash::Hash for QuasiIdentifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.column_name.hash(state);
        self.index.hash(state);
        self.qi_type.hash(state);
    }
}
impl std::cmp::PartialEq for QuasiIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.column_name == other.column_name && self.qi_type == other.qi_type && self.index == other.index
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum QIType {
    Numerical { leaf_bucket_size: i64 },
    Categorical { path_to_json_hierarchy: String }
}

#[derive(Clone,Default)]
pub struct QuasiIdentifiers(pub Vec<QuasiIdentifier>/* or maybe a polars series?*/);
impl QuasiIdentifiers {
    pub fn from_csv() -> Self { todo!() }
    
    pub fn from_column_names(names: &[(&str, QIType)]) -> Self { 
        let qis = names.iter().enumerate()
            .map(|(i, (name, qi_type))| QuasiIdentifier {
                index: i,
                column_name: name.to_string(),
                incognito_colname: format!("ATT_{}", i),
                qi_type: qi_type.clone(),
            })
            .collect();
        Self(qis)
    }
}
