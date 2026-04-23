use std::collections::HashMap;
/**
    Numerical Taxomony Stucts
    and impl
*/
#[derive(Clone, Debug)]
pub struct NumericalNode {
    pub id: Sting,
    pub range: (u16, u16)
    pub level: usize,
    pub children: Vec<String>,
    pub parent: Option<String>,
}

pub struct NumericalTaxonomy {
    pub node: HashMap<String, TaxanomyNode>,
    // These min and max will be for
    // dynamically creating the intervals
    pub min_val: u16,
    pub max_val: u16,
    pub col_name: string
}

impl NumericalTaxonomy {
    pub fn create_from_data_range(
        col_name: &str,
        min_val: u16,
        max_val: u16,
        leaf_bucket_size: u16,
        grouping_factor: usize,
    ) -> Self {
        let mut tree_nodes = HashMap::new();

        // building bottom-up
        let leaf_nodes = Self::create_leaf_level(
            &mut nodes,
            col_name,
            min_val,
            max_val,
            leaf_bucket_size,
        )

        let mut current_level_nodes = leaf_nodes;
        let mut current_level = 0;

        while current_level_nodes.len() > 1 {
            current_level += 1;

            current_level_nodes = Self::create_parent_level(
                &mut nodes,
                col_name,
                &current_level_nodes,
                grouping_factor,
                current_level
            )
        }

        if let Some(root_id) = current_level_nodes.first() {
            if let Some(root) = nodes.get_mut(root_id) {
                root.level = current_level + 1;
            }
        }
 
        Self {
            nodes,
            min_val,
            max_val,
            col_name: col_name.to_string(),
        }
    }

    pub fn create_leaf_level(
        nodes: &mut HashMap<String, NumericalNode>,
        name: &str,
        min_val: i32,
        max_val: i32,
        bucket_size: i32,
    ) -> Vec<String> {

    }

    pub fn create_parent_level(
        nodes: &mut HashMap<String, NumericalNode>,
        name: &str,
        child_ids: &[String],
        group_size: usize,
        level: usize,
    ) -> Vec<String> {

    }
}