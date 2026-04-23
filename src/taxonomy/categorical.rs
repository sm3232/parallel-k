use std::collections::HashMap;

/**
    Categorical taxanomy Stucts
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

pub struct CategoricalTaxonomy {
    pub nodes: HashMap<String, CategoricalNode>,
    pub col_name: String,
    pub root_id: String
}

#[derive(Clone, Debug)]
pub struct CategoricalHierarchy {
    pub col_name: String,
    pub children: Vec<CategoricalHierarchy>,
}

impl CategoricalHierarchy {
    pub fn new(col_name: &str) -> Self {
        Self {
            col_name: col_name.to_string(),
            children: vec![],
        }
    }

    pub fn with_children(mut self, children: Vec<CategoricalHierarchy>) -> Self {
        self.children = children;
        self
    }
}

impl CategoricalTaxonomy {
    pub fn create_from_hierarchy(
        col_name: &str,
        hierarchy: &CategoricalHierarchy,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut nodes = HashMap::new();

        let root_id = Self::build_tree(
            &mut nodes,
            col_name,
            hierarchy,
            None,
            0
        );

        Ok(Self {
            nodes,
            col_name: col_name.to_string(),
            root_id,
        })
    }

    // Recursive build with based on hierarchy def
    pub fn build_tree(
        nodes: &mut HashMap<String, CategoricalNode>,
        col_name: &str,
        hierarchy: &CategoricalHierarchy,
        parent_id: Option<String>,
        level: usize
    ) -> String {
        let node_id = format!("{}_{}_{}", col_name, level, hierarchy.col_name);

        let children_node_ids: Vec<String> = hierarchy
            .children
            .iter()
            .map(|child| {
                Self::build_tree(
                    nodes,
                    col_name,
                    child,
                    Some(node_id.clone()),
                    level + 1
                )
            })
            .collect();
        
        let node = CategoricalNode {
            id: node_id.clone(),
            value: hierarchy.col_name.clone(),
            level,
            children: children_node_ids,
            parent: parent_id,
        };
    
        nodes.insert(node_id.clone(), node);
        node_id
    }

    // pub fn find_lca(&self, values: &[String]) -> Option<String> {
    //     // to-do
    //     None
    // }

    pub fn print_categorical_taxanomy_tree(&self) {
        // to-do
    }
}