/*
* Citations
*
* UTD Anonymization ToolBox
* The University of Texas at Dallas
*
* Kristen LeFevre, David J. DeWitt, and Raghu Ramakrishnan.
* 2005. Incognito: efficient full-domain K-anonymity. In
* Proceedings of the 2005 ACM SIGMOD international conference
* on Management of data (SIGMOD '05). Association for
* Computing Machinery, New York, NY, USA, 49–60.
* https://doi.org/10.1145/1066157.1066164
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
    pub parent: Option<Arc<LatticeEntry>>,
}
impl std::fmt::Debug for LatticeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LatticeEntry")
            .field("root", &self.root)
            .finish()
    }
}
impl std::fmt::Display for LatticeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for k in self.root.keys() {
            write!(f, "{}:{}", k.column_name, self.root[k])?;
        }
        Ok(())
    }
}

impl PartialEq for LatticeEntry {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
    }
}

impl PartialOrd for LatticeEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
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

impl LatticeEntry {
    pub fn from_root(root: BTreeMap<QuasiIdentifier, usize>, qis: &QuasiIdentifiers) -> Self {
        Self {
            root: root.clone(),
            attr: qis.0[0].clone(),
            freq_table: Arc::new(FrequencyTable::default()),
            parent: None,
        }
    }
    pub fn from_parent(parent: &Self, attr: QuasiIdentifier) -> Self {
        let mut me = Self {
            attr,
            root: parent.root.clone(),
            freq_table: Arc::new(FrequencyTable::default()),
            parent: Some(Arc::new(parent.clone())),
        };
        *me.root.get_mut(&me.attr).unwrap() += 1;
        me
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
            .map(|attr| col(attr.column_name.to_owned()).alias(attr.incognito_colname.clone()))
            .collect();

        // [resolve_id, ATT_0, ATT_1, ..., ATT_N]
        let qi_df = aux
            .clone()
            .lazy()
            .select(
                // run select_exprs on the aux table, and include the resolve_id column
                std::iter::once(col(resolve_id.clone()))
                    .chain(select_exprs.clone())
                    .collect::<Vec<_>>(),
            )
            .collect()?;

        let base_count_table = qi_df
            .clone()
            .lazy()
            .group_by([all().exclude_cols([resolve_id.clone()])])
            .agg([len().alias("count")])
            .collect()?;

        let dimension_tables = Self::generate_dimension_tables(&dataset)?;

        // S_{i-1}
        let mut s_prev: Vec<LatticeEntry> = Vec::new();
        for qi in &quasi_identifiers.0 {
            // a single { ATT_N } node
            let root = LatticeEntry::from_root(
                BTreeMap::from([(qi.clone(), 0)]),
                &QuasiIdentifiers(vec![qi.clone()]),
            );
            let freq = Self::make_frequency_set(
                &base_count_table,
                QuasiIdentifiers(std::slice::from_ref(qi).to_vec()),
            )?;
            let entries = Self::search_subset(
                root.clone(),
                freq.clone(),
                &dgh_depths,
                &dimension_tables,
                k,
            )?;
            for entry in &entries {
                s_prev.extend(Self::expand_search(entry, &dgh_depths));
            }
        }

        // start at 2, because loop above already considered sets with 1 attribute.
        // they are in s_prev already
        for i in 2..=quasi_identifiers.0.len() {
            // c_i := candidate multi attribute generalization sets of size i
            // candidates are built by combining two (i-1) attribute entries from S_{i-1} that agree on their first i-2 attributes
            let mut c_i: Vec<LatticeEntry> = Vec::default();

            // p := lattice entry with i-1 attributes from S_{i-1}
            s_prev.sort_by_key(|x| x.root.values().sum::<usize>());
            for p in &s_prev {
                // q := lattice entry with i-1 attributes from S_{i-1}
                'skip_q: for q in &s_prev {
                    for (pkey, qkey) in p
                        .root
                        .keys()
                        .zip(q.root.keys())
                        .take(p.root.keys().len() - 1)
                    {
                        if pkey != qkey || p.root[pkey] != q.root[qkey] {
                            continue 'skip_q;
                        }
                    }

                    let q_last_attr = q.root.last_key_value().unwrap().0;
                    if p.root.last_key_value().unwrap().0.index < q_last_attr.index {
                        let qlast_height = q.root[q_last_attr];
                        let mut new_root = p.root.clone();
                        new_root.insert(q_last_attr.clone(), qlast_height);
                        let subset = &QuasiIdentifiers(new_root.keys().cloned().collect());
                        c_i.push(LatticeEntry::from_root(new_root, subset));
                    }
                }
            }

            #[allow(clippy::mutable_key_type)] // interior mutability is not accessed.
            let s_prev_keys: HashSet<LatticeEntry> = s_prev.iter().cloned().collect();
            #[allow(clippy::mutable_key_type)] // interior mutability is not accessed.
            let mut seen = HashSet::new();
            c_i.retain(|cand| {
                seen.insert(cand.clone())
                    && cand.root.keys().all(|drop_attr| {
                        let mut new_root = cand.root.clone();
                        new_root.retain(|k, v| k.ne(drop_attr));
                        s_prev_keys.contains(&LatticeEntry {
                            attr: cand.attr.clone(),
                            root: new_root,
                            parent: None,
                            freq_table: Arc::default(),
                        })
                    })
            });

            let mut s_i = Vec::new();
            for cand in c_i {
                let freq = Self::make_frequency_set(
                    &base_count_table,
                    QuasiIdentifiers(cand.root.keys().cloned().collect()),
                )?;
                let entries =
                    Self::search_subset(cand.clone(), freq, &dgh_depths, &dimension_tables, k)?;
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
            &dimension_tables,
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
        dimension_tables: &DimensionTables,
        k: u32,
    ) -> Result<Vec<LatticeEntry>, AlgorithmError> {
        let qis = QuasiIdentifiers(root.root.keys().cloned().collect());

        // marked := nodes shown k-anonymous via generalization property
        #[allow(clippy::mutable_key_type)] // interior mutability is not accessed.
        let mut marked: HashSet<LatticeEntry> = HashSet::default();
        #[allow(clippy::mutable_key_type)] // interior mutability is not accessed.
        let mut block: HashSet<LatticeEntry> = HashSet::default();
        let mut successful: Vec<LatticeEntry> = Vec::default();

        let mut queue: BinaryHeap<Reverse<LatticeEntry>> = BinaryHeap::default();
        queue.push(Reverse(root));

        while let Some(Reverse(mut node)) = queue.pop() {
            if marked.contains(&node) {
                continue;
            }

            let freq = if node.root.values().all(|&v| v == 0) {
                root_freq.clone()
            } else if let Some(ref parent) = node.parent {
                Self::rollup_from_parent(&qis, &node.attr, &parent.freq_table, dimension_tables)?
            } else {
                Self::rollup(&qis, &node, &root_freq, dimension_tables)?
            };

            let is_anonymous = freq.is_k_anonymous(k)?;
            node.set_freq_table(Arc::new(freq));
            if is_anonymous {
                // mark every direct generalization of this node
                for generalization in Self::direct_generalizations(&node, dgh_depths) {
                    marked.insert(generalization);
                }
                successful.push(node);
            } else {
                for generalization in Self::direct_generalizations(&node, dgh_depths) {
                    queue.push(Reverse(generalization));
                }
                block.insert(node);
            }
        }
        successful.retain(|node| !block.contains(node));
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
        subset: QuasiIdentifiers,
    ) -> PolarsResult<FrequencyTable> {
        let att_cols: Vec<Expr> = subset
            .0
            .iter()
            .map(|attr| col(attr.incognito_colname.clone()))
            .collect();

        let df = qi_df
            .clone()
            .lazy()
            .group_by(att_cols)
            .agg([len().cast(DataType::UInt32).alias("count")])
            .collect()?;

        Ok(FrequencyTable { qis: subset, df })
    }

    fn generalize_column(
        lf: LazyFrame,
        attr: &QuasiIdentifier,
        dimension_tables: &DimensionTables,
    ) -> PolarsResult<LazyFrame> {
        let col_name = attr.incognito_colname.clone();
        let dim_table = &dimension_tables[attr];

        let edges = dim_table.clone().lazy().select([
            col("from").alias(col_name.clone()),
            col("to").alias("__next"),
        ]);

        let post_join = lf
            .join(
                edges,
                [col(col_name.clone())],
                [col(col_name.clone())],
                JoinArgs::new(JoinType::Left),
            )
            .collect()?;

        Ok(post_join
            .lazy()
            .with_column(coalesce(&[col("__next"), col(col_name.clone())]).alias(col_name))
            .select([all().exclude_cols(["__next"]).as_expr()]))
    }

    fn rollup_from_parent(
        qis: &QuasiIdentifiers,
        changed_attr: &QuasiIdentifier,
        parent_freq: &FrequencyTable,
        dimension_tables: &DimensionTables,
    ) -> PolarsResult<FrequencyTable> {
        let lf = Self::generalize_column(
            parent_freq.df.clone().lazy(),
            changed_attr,
            dimension_tables,
        )?;

        let att_cols: Vec<Expr> = qis
            .0
            .iter()
            .map(|attr| col(attr.incognito_colname.clone()))
            .collect();
        let df = lf.group_by(att_cols).agg([col("count").sum()]).collect()?;

        Ok(FrequencyTable {
            qis: qis.clone(),
            df,
        })
    }

    /// generalize each attribute's distinct values, then regroup and `sum(count)`
    fn rollup(
        qis: &QuasiIdentifiers,
        node: &LatticeEntry,
        root_freq: &FrequencyTable,
        dimension_tables: &DimensionTables,
    ) -> PolarsResult<FrequencyTable> {
        let mut lf = root_freq.df.clone().lazy();
        for attr in &qis.0 {
            let levels = node.root[attr];
            for _ in 0..levels {
                lf = Self::generalize_column(lf, attr, dimension_tables)?;
            }
        }

        let att_cols: Vec<Expr> = qis
            .0
            .iter()
            .map(|attr| col(attr.incognito_colname.clone()))
            .collect();
        let df = lf.group_by(att_cols).agg([col("count").sum()]).collect()?;

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
        dimension_tables: &DimensionTables,
    ) -> PolarsResult<DataFrame> {
        let mut lf = qi_df.clone().lazy();
        for attr in &qis.0 {
            let levels = node.root[attr];
            for _ in 0..levels {
                lf = Self::generalize_column(lf, attr, dimension_tables)?;
            }
        }

        let select_cols: Vec<Expr> = std::iter::once(col(resolve_id.to_owned()))
            .chain(qis.0.iter().map(|attr| col(attr.incognito_colname.clone())))
            .collect();

        lf.select(select_cols).collect()
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
                    #[allow(clippy::type_complexity)]
                    // two columns, ranges. [current range (low, high), generalizes to range (low, high)]
                    let from_to: (Vec<(i64, i64)>, Vec<(i64, i64)>) = dataset
                        .taxonomies
                        .numerical_taxonomies[&qi.column_name]
                        .nodes
                        .iter()
                        .sorted_by_key(|(node_id, node)| Reverse(node.level))
                        .map(|(node_id, node)| {
                            node.parent.as_ref().map_or(
                                (node.range.to_owned(), node.range.to_owned()),
                                |parent_id| {
                                    (
                                        node.range.to_owned(),
                                        dataset.taxonomies.numerical_taxonomies[&qi.column_name]
                                            .nodes[parent_id]
                                            .range
                                            .to_owned(),
                                    )
                                },
                            )
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .unzip();

                    assert!(from_to.0.len() == from_to.1.len());

                    let from_low_high: (Vec<i64>, Vec<i64>) = from_to.0.into_iter().unzip();
                    let to_low_high: (Vec<i64>, Vec<i64>) = from_to.1.into_iter().unzip();

                    assert!(
                        from_low_high.0.len() == from_low_high.1.len()
                            && to_low_high.0.len() == to_low_high.1.len()
                            && from_low_high.0.len() == to_low_high.0.len()
                    );

                    let mut df = DataFrame::new(
                        from_low_high.0.len(),
                        vec![
                            StructChunked::from_series(
                                "from".into(),
                                from_low_high.0.len(),
                                [
                                    Series::new("low".into(), from_low_high.0),
                                    Series::new("high".into(), from_low_high.1),
                                ]
                                .iter(),
                            )?
                            .into_column(),
                            StructChunked::from_series(
                                "to".into(),
                                to_low_high.0.len(),
                                [
                                    Series::new("low".into(), to_low_high.0),
                                    Series::new("high".into(), to_low_high.1),
                                ]
                                .iter(),
                            )?
                            .into_column(),
                        ],
                    )?;
                    dim_tables.push(
                        df.unique_impl(
                            false,
                            Some(vec![PlSmallStr::from("from")]),
                            UniqueKeepStrategy::Any,
                            None,
                        )?
                        .with_row_index("id".into(), None)?,
                    );
                    map.insert(qi.clone(), dim_tables.len() - 1);
                }
                QIType::Categorical { .. } => {
                    let mut from_to: (Vec<String>, Vec<String>) =
                        dataset.taxonomies.categorical_taxonomies[&qi.column_name]
                            .nodes
                            .iter()
                            .flat_map(|(node_id, node)| {
                                node.parent.as_ref().map(|parent_id| {
                                    (
                                        node.value.to_owned(),
                                        dataset.taxonomies.categorical_taxonomies[&qi.column_name]
                                            .nodes[parent_id]
                                            .value
                                            .to_owned(),
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
                }
            }
        }
        Ok(DimensionTables::new(map, dim_tables))
    }
}
