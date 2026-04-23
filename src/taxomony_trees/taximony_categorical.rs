/**
    Categorical Taxomony Stucts
    and impl
*/
#[derive(Clone, Debug)]
pub struct CategoricalNode {
    pub id: String,
    pub value: String,
    pub level: usize,
    pub children: Vec<String>,
    pub parent: Option<String> 
}

pub struct CategoricalTaxomony {
    pub nodes: HashMap<String, CategoricalNode>,
    pub name: String,
    pub root_id: String
}   

#[derive(Clone, Debug)]
pub struct CategoricalHierarchy {
    pub name: String,
    pub children: Vec<CategoricalHierarchy>,
}

impl CategoricalHierarchy {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: vec![],
        }
    }

    pub fn with_children(mut self, children: Vec<CategoricalHierarchy>) -> Self {
        self.children = children;
        self
    }
}

impl CategoricalTaxomony {

}

