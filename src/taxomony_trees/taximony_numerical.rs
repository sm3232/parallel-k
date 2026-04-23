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

}