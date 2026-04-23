use polars::prelude

pub struct TaxomonyManager {
    pub numerical_taxomonies: HashMap<String, NumericalTaxonomy>,
    pub categorical_taxomonies: HashMap<String, CategoricalTaxomony>
}

impl TaxomonyManager {
    pub fn new() -> Self {
        Self {
            numerical_taxonomies: HashMap::new(),
            categorical_taxonomies: HashMap::new(),
        }
    }

    pub fn build_all(
        df: &DataFrame,
        numerical_qis: &[&str],
        categorical_qis: &[(&str, CategoricalHierarchy)],
        leaf_bucket_size: i32,
        grouping_factor: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut manager = Self::new();

        // Build numerical taxonomies
        println!("=== Building Numerical Taxonomies ===");
        for qi_name in numerical_qis {
            println!("\n--- {} ---", qi_name);

            let col = df.column(qi_name)?;
            let series = col.i32()?;
            
            let min_val = series.min().unwrap_or(0);
            let max_val = series.max().unwrap_or(100);

            println!("Range: {} to {}", min_val, max_val);

            let taxonomy = NumericalTaxonomy::from_data_range(
                qi_name,
                min_val,
                max_val,
                leaf_bucket_size,
                grouping_factor,
            );

            taxonomy.print_tree();
            manager.numerical_taxonomies.insert(qi_name.to_string(), taxonomy);
        }

        // Build categorical taxonomies
        println!("\n=== Building Categorical Taxonomies ===");
        for (qi_name, hierarchy) in categorical_qis {
            println!("\n--- {} ---", qi_name);

            let col = df.column(qi_name)?;
            let unique_values = col
                .unique()?
                .str()?
                .into_iter()
                .filter_map(|opt| opt.map(|s| s.to_string()))
                .collect::<Vec<_>>();

            println!("Found {} unique values", unique_values.len());

            let taxonomy = CategoricalTaxonomy::from_hierarchy(
                qi_name,
                &hierarchy,
                &unique_values,
            )?;

            taxonomy.print_tree();
            manager.categorical_taxonomies.insert(qi_name.to_string(), taxonomy);
        }

        Ok(manager)
    }

    pub fn get_numerical(&self, qi_name: &str) -> Option<&NumericalTaxonomy> {
        self.numerical_taxonomies.get(qi_name)
    }

    pub fn get_categorical(&self, qi_name: &str) -> Option<&CategoricalTaxonomy> {
        self.categorical_taxonomies.get(qi_name)
    }
}