/*
* Citation: based on
* UTD Anonymization ToolBox
* The University of Texas at Dallas
*/

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap, HashMap, HashSet},
};

use itertools::Itertools;
use polars::{frame::row::Row, prelude::*};
use rayon::prelude::*;

use crate::{
    algorithms::anonymization_algorithm::{AlgorithmError, AnonymizationAlgorithm},
    data::{QIType, QuasiIdentifier, dataset::Dataset, qi::QuasiIdentifiers},
    taxonomy::TaxonomyManager,
};
struct DimensionTables {
    map: BTreeMap<QuasiIdentifier, usize>,
    tables: Vec<DataFrame>,
}
impl DimensionTables {
    pub fn new(map: BTreeMap<QuasiIdentifier, usize>, tables: Vec<DataFrame>) -> Self {
        Self { map, tables }
    }
}
impl std::ops::Index<&QuasiIdentifier> for DimensionTables {
    type Output = DataFrame;
    fn index(&self, index: &QuasiIdentifier) -> &Self::Output {
        &self.tables[self.map[index]]
    }
}
impl std::ops::Index<QuasiIdentifier> for DimensionTables {
    type Output = DataFrame;
    fn index(&self, index: QuasiIdentifier) -> &Self::Output {
        &self.tables[self.map[&index]]
    }
}


#[derive(Clone, Default)]
pub struct FrequencyTable {
    pub qis: QuasiIdentifiers,
    pub df: DataFrame,
}
impl FrequencyTable {
    pub fn is_k_anonymous(&self, k: u32) -> PolarsResult<bool> {
        let min_count = if self.df.height() <= 1 {
            self.df
                .column("count")?
                .as_materialized_series()
                .min::<u32>()?
                .unwrap_or(0)
        } else {
            self.df
                .column("count")?
                .as_series()
                .unwrap()
                .min::<u32>()?
                .unwrap_or(0)
        };
        Ok(min_count >= k)
    }
}

#[derive(Clone)]
pub struct LatticeEntry {
    pub attr: QuasiIdentifier,
    pub root: BTreeMap<QuasiIdentifier, usize>,
    pub freq_table: Arc<FrequencyTable>,
}

impl PartialEq for LatticeEntry {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
    }
}

impl PartialOrd for LatticeEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.root.partial_cmp(&other.root)
    }
}

impl Eq for LatticeEntry {}

impl Ord for LatticeEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.root.cmp(&other.root)
    }
}

impl std::hash::Hash for LatticeEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.root.hash(state);
    }
}

impl std::fmt::Display for LatticeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut keys: Vec<_> = self.root.keys().collect();
        
        let pairs: Vec<String> = keys
            .iter()
            .map(|x| format!("{}:{}", x.column_name, self.root.get(x).unwrap()))
            .collect();
            
        write!(f, "{}", pairs.join("_"))
    }
}
impl std::fmt::Debug for LatticeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LatticeEntry")
            .field("attr", &self.attr)
            .field("root", &self.root)
            .finish()
    }
}

impl LatticeEntry {
    pub fn from_root(root: BTreeMap<QuasiIdentifier, usize>, qis: &QuasiIdentifiers) -> Self {
        Self {
            root: root.clone(),
            attr: qis.0[0].clone(),
            freq_table: Arc::new(FrequencyTable::default()),
        }
    }
    pub fn from_parent(parent: &Self, attr: QuasiIdentifier) -> Self {
        let mut me = Self {
            attr,
            root: parent.root.clone(),
            freq_table: Arc::new(FrequencyTable::default()),
        };
        *me.root.get_mut(&me.attr).unwrap() += 1;
        me
    }

    pub fn omit(&self, qi: &QuasiIdentifier) -> Self {
        let mut new_root = self.root.clone();
        new_root.retain(|k, v| k.ne(qi));
        Self::from_root(
            new_root.clone(),
            &QuasiIdentifiers(new_root.keys().cloned().collect()),
        )
    }
    pub fn set_freq_table(&mut self, freq_table: Arc<FrequencyTable>) {
        self.freq_table = freq_table;
    }
}

#[derive(Default)]
pub struct Incognito {}

impl AnonymizationAlgorithm for Incognito {
    fn name(&self) -> &str {
        "Incognito"
    }

    fn anonymize(
        &self,
        k: u32,
        dataset: Dataset,
        quasi_identifiers: QuasiIdentifiers,
    ) -> Result<Dataset, AlgorithmError> {
        if k < 2 {
            return Err(format!("k must be at least 2, got k = {k}").into());
        }

        let df_col_names: Vec<_> = dataset
            .df
            .get_column_names_owned()
            .iter()
            .map(|x| x.to_string())
            .collect();
        let mut resolve_id = "resolve_id".to_owned();
        while df_col_names.contains(&resolve_id) {
            resolve_id += "_";
        }

        // resolve_id column to rejoin table after anonymization
        let df = dataset.df.with_row_index(resolve_id.clone().into(), None)?;

        // converted numeric ranges to struct columns with { low: number, high: number }
        let aux = df
            .lazy()
            .with_columns(
                dataset
                    .taxonomies
                    .numerical_taxonomies
                    .iter()
                    .map(|(name, tax)| {
                        as_struct(vec![
                            col(name.to_owned()).alias("low"),
                            col(name.to_owned()).alias("high"),
                        ])
                        .alias(name.to_owned())
                    })
                    .collect::<Vec<_>>(),
            )
            .collect()?;

        let mut dgh_depths: HashMap<QuasiIdentifier, usize> = HashMap::default();
        for qi in quasi_identifiers.0.iter() {
            dgh_depths.insert(
                qi.clone(),
                dataset.taxonomies.get_qi_height(qi.column_name.to_owned()),
            );
        }

        let select_exprs: Vec<Expr> = quasi_identifiers
            .0
            .iter()
            // expressions that select each qi column (age, sex, etc) and renames them to ATT_0, ATT_1, ..., ATT_N
            // also selects the resolve_id column
            .map(|attr| col(attr.column_name.to_owned()).alias(attr.incognito_colname.clone()))
            .collect();

        // [resolve_id, ATT_0, ATT_1, ..., ATT_N]
        let qi_df = aux
            .clone()
            .lazy()
            .select(
                // run select_exprs on the aux table
                std::iter::once(col(resolve_id.clone()))
                    .chain(select_exprs.clone())
                    .collect::<Vec<_>>(),
            )
            .collect()?;


        let base_count_table = qi_df
            .clone()
            .lazy()
            .group_by(
                [all().exclude_cols([resolve_id.clone()])]
            )
            .agg([len().alias("count")]).collect()?;

        let dimension_tables = Self::generate_dimension_tables(&dataset)?;


        // S_{i-1}
        let mut s_prev: Vec<LatticeEntry> = Vec::new();
        for qi in &quasi_identifiers.0 {
            // a single { ATT_N } node
            let root = LatticeEntry::from_root(
                BTreeMap::from([(qi.clone(), 0)]),
                &QuasiIdentifiers(vec![qi.clone()]),
            );
            let freq = Self::make_frequency_set(&base_count_table, std::slice::from_ref(qi))?;
            let entries = Self::search_subset(root, freq, &dgh_depths, &dataset.taxonomies, k)?;
            for entry in &entries {
                s_prev.extend(Self::expand_search(entry, &dgh_depths));
            }
        }

        println!("Considered sets of size 1/{}", quasi_identifiers.0.len());
        // start at 2, because loop above already considered sets with 1 attribute.
        // they are in s_prev already
        for i in 2..=quasi_identifiers.0.len() {
            println!("Considered sets of size {i}/{}", quasi_identifiers.0.len());

            // c_i := candidate multi attribute generalization sets of size i
            // candidates are built by combining two (i-1) attribute entries from S_{i-1} that agree on their first i-2 attributes
            let mut c_i: Vec<LatticeEntry> = Vec::default();

            // p := lattice entry with i-1 attributes from S_{i-1}
            s_prev.sort_by_key(|x| x.root.values().sum::<usize>());
            for p in &s_prev {
                // q := lattice entry with i-1 attributes from S_{i-1}
                'skip_q: for q in &s_prev {
                    // attributes in p
                    let mut pkeys: Vec<_> = p.root.keys().cloned().collect();

                    // attributes in q
                    let mut qkeys: Vec<_> = q.root.keys().cloned().collect();

                    for index in 0..(pkeys.len() - 1) {
                        if pkeys[index] != qkeys[index] {
                            continue 'skip_q;
                        }
                        if p.root[&pkeys[index]] != q.root[&qkeys[index]] {
                            continue 'skip_q;
                        }
                    }

                    let p_last_attr = &pkeys[pkeys.len() - 1];
                    let q_last_attr = &qkeys[qkeys.len() - 1];

                    if p_last_attr.index < q_last_attr.index {
                        let qlast_height = q.root[q_last_attr];
                        let mut new_root = p.root.clone();
                        new_root.insert(q_last_attr.clone(), qlast_height);
                        let mut new_subset: Vec<_> = new_root.keys().cloned().collect();
                        c_i.push(LatticeEntry::from_root(
                            new_root,
                            &QuasiIdentifiers(new_subset),
                        ));
                    }
                }
            }


            #[allow(clippy::mutable_key_type)] // interior mutability is not accessed. 
            let s_prev_keys: HashSet<LatticeEntry> = s_prev.iter().cloned().collect();
            #[allow(clippy::mutable_key_type)] // interior mutability is not accessed. 
            let mut seen = HashSet::new();
            c_i.retain(|cand| {
                cand.root
                    .keys()
                    .all(|drop_attr| s_prev_keys.contains(&cand.omit(drop_attr))) && 
                seen.insert(cand.clone())
            });


            let mut s_i = Vec::new();
            for cand in c_i {
                let mut subset: Vec<QuasiIdentifier> = cand.root.keys().cloned().collect();
                let freq = Self::make_frequency_set(&base_count_table, &subset)?;
                let entries = Self::search_subset(cand.clone(), freq, &dgh_depths, &dataset.taxonomies, k)?;
                for entry in &entries {
                    s_i.extend(Self::expand_search(entry, &dgh_depths));
                }
            }

            if s_i.is_empty() {
                break;
            }
            s_prev = s_i;
        }

        let best_entry = s_prev
            .into_iter()
            .min_by_key(|entry| entry.root.values().sum::<usize>())
            .ok_or_else(|| {
                format!("No k-anonymous full domain generalization exists for k = {k}").to_string()
            })?;

        let mut best_subset: Vec<_> = best_entry.root.keys().cloned().collect();
        let best_qis = QuasiIdentifiers(best_subset);

        let generalized = Self::materialize(
            &qi_df,
            &resolve_id,
            &best_qis,
            &best_entry,
            &dataset.taxonomies,
        )?;

        // create a table that has all columns not in the generalized qi set already.
        let resolve_df = aux
            .clone()
            .lazy()
            .select([all()
                .exclude_cols(best_qis.0.iter().map(|attr| attr.column_name.to_owned()))
                .into()])
            .collect()?;

        // add data that wasn't anonymized back to the table
        let mut result = generalized
            .lazy()
            .join(
                resolve_df.lazy(),
                [col(resolve_id.clone())],
                [col(resolve_id.clone())],
                JoinArgs::new(JoinType::Inner),
            )
            .collect()?;

        // rename ATT_N back to the column name.
        let renames = best_qis
            .0
            .iter()
            .map(|x| {
                (
                    x.incognito_colname.clone(),
                    PlSmallStr::from(x.column_name.to_owned()),
                )
            })
            .collect::<Vec<_>>();
        let renames_refs: Vec<_> = renames
            .iter()
            .map(|(str, plsm)| (str.as_str(), plsm.clone()))
            .collect();
        result.rename_many(renames_refs.into_iter())?.rechunk_mut();

        // remap { low: number, high: number } columns to a string range.
        result = result
            .lazy()
            .with_columns(
                quasi_identifiers
                    .0
                    .clone()
                    .into_iter()
                    .map(|x| match x.qi_type {
                        QIType::Numerical { .. } => concat_str(
                            [
                                col(x.column_name.clone()).struct_().field_by_name("low"),
                                col(x.column_name.clone()).struct_().field_by_name("high"),
                            ],
                            "-",
                            true,
                        )
                        .alias(x.column_name.clone()),
                        QIType::Categorical { .. } => col(x.column_name.clone()),
                    })
                    .collect::<Vec<_>>(),
            )
            .collect()?;

        result = result.drop_many([&resolve_id]);
        result.rechunk_mut();

        Ok(Dataset::from_anonymized(
            result,
            dataset.qis.clone(),
            dataset.taxonomies.clone(),
        ))
    }
}

impl Incognito {
    // Performs bfs over a qi subset's generalization lattice
    fn search_subset(
        root: LatticeEntry,
        root_freq: FrequencyTable,
        dgh_depths: &HashMap<QuasiIdentifier, usize>,
        taxonomy_man: &TaxonomyManager,
        k: u32,
    ) -> Result<Vec<LatticeEntry>, AlgorithmError> {
        let qis = QuasiIdentifiers({
            let mut attrs: Vec<_> = root.root.keys().cloned().collect();
            attrs
        });

        let root_key = root.to_string();

        // marked := nodes shown k-anonymous via generalization property
        let mut marked: HashSet<String> = HashSet::default();
        let mut enqueued: HashSet<String> = HashSet::default();
        let mut successful: Vec<LatticeEntry> = Vec::default();

        let mut queue: BinaryHeap<Reverse<LatticeEntry>> = BinaryHeap::default();
        enqueued.insert(root_key.clone());
        queue.push(Reverse(root));

        while let Some(Reverse(node)) = queue.pop() {
            let key = node.to_string();
            if marked.contains(&key) {
                continue;
            }

            let freq = if node.root.values().all(|&v| v == 0) {
                root_freq.clone()
            } else {
                Self::rollup(&qis, &node, &root_freq, taxonomy_man)?
            };

            if freq.is_k_anonymous(k)? {
                // mark every direct generalization of this node
                for edge in Self::direct_generalizations(&node, dgh_depths) {
                    marked.insert(edge.to_string());
                }
                let mut done = node;
                done.set_freq_table(Arc::new(freq));
                successful.push(done);
            } else {
                for edge in Self::direct_generalizations(&node, dgh_depths) {
                    let edge_key = edge.to_string();
                    if !marked.contains(&edge_key) && enqueued.insert(edge_key) {
                        queue.push(Reverse(edge));
                    }
                }
            }
        }

        Ok(successful)
    }

    // outgoing edges of node in the multi attribute generalization lattice.
    fn direct_generalizations(
        node: &LatticeEntry,
        dgh_depths: &HashMap<QuasiIdentifier, usize>,
    ) -> Vec<LatticeEntry> {
        let mut attrs: Vec<QuasiIdentifier> = node.root.keys().cloned().collect();
        attrs
            .into_iter()
            .filter(|qi| node.root[qi] < dgh_depths[qi])
            .map(|qi| LatticeEntry::from_parent(node, qi))
            .collect()
    }

    fn make_frequency_set(
        qi_df: &DataFrame,
        subset: &[QuasiIdentifier],
    ) -> PolarsResult<FrequencyTable> {
        let att_cols: Vec<Expr> = subset
            .iter()
            .map(|attr| col(attr.incognito_colname.clone()))
            .collect();

        let df = qi_df
            .clone()
            .lazy()
            .group_by(att_cols)
            .agg([len().cast(DataType::UInt32).alias("count")])
            .collect()?;

        Ok(FrequencyTable {
            qis: QuasiIdentifiers(subset.to_vec()),
            df,
        })
    }

    fn generalize_column(
        raw: &Column,
        attr: &QuasiIdentifier,
        levels: usize,
        taxonomy_man: &TaxonomyManager,
    ) -> PolarsResult<Column> {
        let col_name = attr.incognito_colname.clone();

        Ok(match attr.qi_type {
            QIType::Numerical { .. } => {
                let tax = taxonomy_man
                    .numerical_taxonomies
                    .get(&attr.column_name)
                    .unwrap();
                let struct_ = raw.struct_()?;
                let lo = struct_.field_by_name("low")?;
                let hi = struct_.field_by_name("high")?;
                let lo = lo.i64()?;
                let hi = hi.i64()?;

                let (lows, highs): (Vec<i64>, Vec<i64>) = lo
                    .into_iter()
                    .zip(hi.into_iter())
                    .map(|(l, h)| {
                        let mut range = (l.unwrap(), h.unwrap());
                        for level in 0..levels {
                            range = tax.generalize(range, level);
                        }
                        range
                    })
                    .unzip();

                StructChunked::from_series(
                    col_name.into(),
                    lows.len(),
                    [
                        Series::new("low".into(), lows),
                        Series::new("high".into(), highs),
                    ]
                    .iter(),
                )?
                .into_column()
            }
            QIType::Categorical { .. } => {
                let tax = taxonomy_man
                    .categorical_taxonomies
                    .get(&attr.column_name)
                    .unwrap();
                let strs = raw.str()?;

                let values: Vec<String> = strs
                    .into_iter()
                    .map(|s| {
                        let mut category = s.unwrap().to_owned();
                        for _ in 0..levels {
                            category = tax.generalize(category);
                        }
                        category
                    })
                    .collect();

                Series::new(col_name.into(), values).into_column()
            }
        })
    }

    /// generalize each attribute's distinct values, then regroup and `sum(count)`
    fn rollup(
        qis: &QuasiIdentifiers,
        node: &LatticeEntry,
        root_freq: &FrequencyTable,
        taxonomy_man: &TaxonomyManager,
    ) -> PolarsResult<FrequencyTable> {
        let df = &root_freq.df;
        let n_rows = df.height();

        let mut out_columns: Vec<Column> = vec![df.column("count")?.clone()];
        for attr in &qis.0 {
            let levels = node.root[attr];
            let raw = df.column(&attr.incognito_colname)?;
            out_columns.push(Self::generalize_column(raw, attr, levels, taxonomy_man)?);
        }
        let generalized = DataFrame::new(n_rows, out_columns)?;

        let att_cols: Vec<Expr> = qis
            .0
            .iter()
            .map(|attr| col(attr.incognito_colname.clone()))
            .collect();
        let df = generalized
            .lazy()
            .group_by(att_cols)
            .agg([col("count").sum()])
            .collect()?;

        Ok(FrequencyTable {
            qis: qis.clone(),
            df,
        })
    }

    // apply `node` generalization to each row
    fn materialize(
        qi_df: &DataFrame,
        resolve_id: &str,
        qis: &QuasiIdentifiers,
        node: &LatticeEntry,
        taxonomy_man: &TaxonomyManager,
    ) -> PolarsResult<DataFrame> {
        let n_rows = qi_df.height();
        let mut out_columns: Vec<Column> = vec![qi_df.column(resolve_id)?.clone()];

        for attr in &qis.0 {
            let levels = node.root[attr];
            let raw = qi_df.column(&attr.incognito_colname)?;
            out_columns.push(Self::generalize_column(raw, attr, levels, taxonomy_man)?);
        }

        DataFrame::new(n_rows, out_columns)
    }

    fn expand_search(
        entry: &LatticeEntry,
        dgh_depths: &HashMap<QuasiIdentifier, usize>,
    ) -> Vec<LatticeEntry> {
        let mut attrs: Vec<QuasiIdentifier> = entry.root.keys().cloned().collect();
        let ranges: Vec<Vec<usize>> = attrs
            .par_iter()
            .map(|a| (entry.root[a]..=dgh_depths[a]).collect())
            .collect();

        ranges
            .into_iter()
            .multi_cartesian_product()
            .map(|combo| {
                let root: BTreeMap<_, _> = attrs.iter().cloned().zip(combo).collect();
                let mut e = LatticeEntry::from_root(root, &QuasiIdentifiers(attrs.clone()));
                e.set_freq_table(entry.freq_table.clone());
                e
            })
            .collect()
    }

    fn generate_dimension_tables(dataset: &Dataset) -> Result<DimensionTables, PolarsError> {
        let mut map: BTreeMap<QuasiIdentifier, usize> = BTreeMap::default();
        let mut dim_tables: Vec<DataFrame> = Vec::with_capacity(
            dataset.taxonomies.numerical_taxonomies.len()
            + dataset.taxonomies.categorical_taxonomies.len(),
        );

        for qi in &dataset.qis.0 {
            match qi.qi_type {
                QIType::Numerical { .. } => {
                    #[allow(clippy::type_complexity)] // two columns, ranges
                    let from_to: (Vec<(i64, i64)>, Vec<(i64, i64)>) = dataset.taxonomies.numerical_taxonomies[&qi.column_name]
                        .nodes
                        .iter()
                        .flat_map(|(node_id, node)| {
                            node.parent.as_ref().map(|parent_id| {
                                (
                                    node.range.to_owned(),
                                    dataset.taxonomies.numerical_taxonomies[&qi.column_name].nodes[parent_id].range.to_owned(),
                                )
                            })
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .unzip();

                    assert!(from_to.0.len() == from_to.1.len());

                    let from_low_high: (Vec<i64>, Vec<i64>) = from_to.0.into_iter().unzip();
                    let to_low_high: (Vec<i64>, Vec<i64>) = from_to.1.into_iter().unzip();

                    assert!(from_low_high.0.len() == from_low_high.1.len() && to_low_high.0.len() == to_low_high.1.len() && from_low_high.0.len() == to_low_high.0.len());

                    dim_tables
                        .push(DataFrame::new(from_low_high.0.len(), 
                            vec![
                                StructChunked::from_series(
                                    "from".into(),
                                    from_low_high.0.len(),
                                    [
                                        Series::new("low".into(), from_low_high.0),
                                        Series::new("high".into(), from_low_high.1),
                                    ].iter(),
                                )?.into_column(),
                                StructChunked::from_series(
                                    "to".into(),
                                    to_low_high.0.len(),
                                    [
                                        Series::new("low".into(), to_low_high.0),
                                        Series::new("high".into(), to_low_high.1),
                                    ].iter(),
                                )?.into_column(),
                            ])?.with_row_index("id".into(), None)?);
                    map.insert(qi.clone(), dim_tables.len() - 1);
                },
                QIType::Categorical { .. } => {
                    let mut from_to: (Vec<String>, Vec<String>) = dataset.taxonomies.categorical_taxonomies[&qi.column_name]
                        .nodes
                        .iter()
                        .flat_map(|(node_id, node)| {
                            node.parent.as_ref().map(|parent_id| {
                                (
                                    node.value.to_owned(),
                                    dataset.taxonomies.categorical_taxonomies[&qi.column_name].nodes[parent_id].value.to_owned(),
                                )
                            })
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .unzip();
                    assert!(from_to.0.len() == from_to.1.len());
                    dim_tables.push(
                        DataFrame::new(
                            from_to.0.len(),
                            vec![
                                Column::new("from".into(), from_to.0),
                                Column::new("to".into(), from_to.1),
                            ],
                        )?
                            .with_row_index("id".into(), None)?,
                    );
                    map.insert(qi.clone(), dim_tables.len() - 1);
                },
            }
        }
        Ok(DimensionTables::new(map, dim_tables))
    }
}
